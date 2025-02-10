use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema,
    },
    Client,
};
use chrono::Local;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Parser)]
struct Args {
    dataset: String,
    logs_file: String,
    #[arg(short, long = "dry")]
    dry_run: bool,
    #[arg(short, long)]
    fixed: Option<String>,
}

fn cidiff_gh_parse(file_content: String) -> Vec<(usize, String)> {
    let timestamp = Regex::new(r"(?:\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{7}Z ?)?(.*)").unwrap();
    let ansi = Regex::new(r"\x1b?\[(?:\d+)?(?:;\d+)*m").unwrap();
    let mut v = vec![];
    let mut i = 0;
    for ele in file_content.lines() {
        let caps = timestamp.captures(ele).unwrap();
        let content = &caps[1];
        let cleaned = ansi.replace_all(content, "");
        if !cleaned.trim().is_empty() {
            v.push((i, cleaned.to_string()));
            i += 1;
        }
        //v.push(format!("$${}$$ {}", i, cleaned).to_string());
    }
    //println!("log:\n{:?}", v);
    //println!("bytes 268: {:?}", v[268].1.chars());
    //v.join("\n")
    v
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let openai_key = env::var("OPENAI_KEY").expect("The OpenAI key is not in the environment variables");

    let args = Args::parse();

    if args.dry_run {
        println!("Running in dry run");
    }

    let dataset_path = Path::new(&args.dataset);

    let config = OpenAIConfig::new().with_api_key(openai_key);
    let client = Client::with_config(config);

    if let Some(fixed) = args.fixed {
        do_simple(&client, fixed, args.dry_run).await;
        return Ok(());
    }

    let binding = fs::read_to_string(args.logs_file).expect("Unable to read the logs file");
    let paths: Vec<PathBuf> = binding
        .lines()
        .map(PathBuf::from)
        //.map(Path::new)
        //.map(|p| p.join("failure.log"))
        .collect();
    let bar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("[{pos}/{len}] {msg} {wide_bar}").unwrap());
    //let multi = MultiProgress::new();
    //let bar = multi.add(bar);

    let output_dir: PathBuf = [
        "./generated",
        Local::now().format("%Y-%m-%d#%H-%M").to_string().as_str(),
    ]
    .iter()
    .collect();

    // create output dir
    let _ = fs::create_dir_all(&output_dir);

    let mut result = BufWriter::new(File::create(format!(
        "{}/gpt.csv",
        output_dir.to_str().unwrap(),
    ))?);
    writeln!(result, "path,type,line")?;
    //result.flush()?;

    for path in paths {
        bar.inc(1);

        //let log_bar = multi.add(ProgressBar::new_spinner());
        let log_bar = ProgressBar::new_spinner();
        log_bar.enable_steady_tick(Duration::from_millis(100));

        let log_path = dataset_path.join(&path).join("failure.log");
        log_bar.set_message(format!(
            "Reading the log file {}",
            log_path.to_str().unwrap_or("<err>")
        ));
        let log_content = cidiff_gh_parse(fs::read_to_string(&log_path).unwrap());

        log_bar.set_message("Request sent to chatgpt, awaiting response");
        if let Some(response) = ask_gpt(
            &client,
            log_content
                .iter()
                .map(|(i, s)| format!("$${}$$ {}", i, s))
                .collect::<Vec<String>>()
                .join("\n"),
            args.dry_run,
        )
        .await
        {
            log_bar.set_message("Got response!");
            let output_path = format!(
                "{}/{}.json",
                output_dir.to_str().unwrap(),
                path.to_str().unwrap().replace("/", "#"),
            );
            match fs::write(&output_path, &response) {
                Ok(_) => log_bar.set_message(format!("written in {}", output_path)),
                Err(e) => eprintln!("error writting {}: {}", output_path, e),
            };
            match serde_json::from_str::<Resp>(&response) {
                Ok(resp) => {
                    println!("checking truth for {}", log_path.to_str().unwrap());
                    check_truth(&resp, &log_content);
                    for line in resp.lines {
                        if line.contains("$$") {
                            match line.split("$$").collect::<Vec<&str>>()[1].parse::<usize>() {
                                Ok(i) => {
                                    writeln!(result, "{},gpt,{}", path.to_str().unwrap(), i)?;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "error writing csv for {} {} {}",
                                        path.to_str().unwrap(),
                                        line,
                                        e
                                    );
                                }
                            };
                        } else {
                            eprintln!(
                                "error writing csv, `{}` doesn't have the $$ prefix ({})",
                                line,
                                path.to_str().unwrap()
                            );
                        }
                        //result.flush()?;
                    }
                }
                Err(e) => {
                    eprintln!("Error deserializing the response: {}", e);
                }
            };
        }
        log_bar.finish_and_clear();
        //multi.remove(&log_bar);
    }
    bar.finish_with_message("Done! The results are written in ./generated/ and gpt.csv.");

    //println!("{}", serde_json::to_string(&request).unwrap());

    Ok(())
}
#[derive(Serialize, Deserialize)]
struct Resp {
    lines: Vec<String>,
    steps: Vec<String>,
}
fn check_truth(response: &Resp, log_content: &[(usize, String)]) {
    response
        .lines
        .iter()
        .enumerate()
        .filter(|(i, line)| **line == log_content[*i].1)
        //.map(|(i, line)| *line == log_content[i].1)
        .for_each(|(i, line)| println!("line {} {} is not {}", i, line, log_content[i].1));
    //.collect();
}

async fn ask_gpt(client: &Client<OpenAIConfig>, log_content: String, dry_run: bool) -> Option<String> {
    let schema = json!({
        "type": "object",
        "properties": {
            "lines": {
                "type": "array",
                "items": { "type": "string" }
            },
            "steps": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": [ "lines", "steps" ],
        "additionalProperties": false
    });
    let response_format = ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            description: None,
            name: "results".into(),
            schema: Some(schema),
            strict: Some(true),
        },
    };

    // dire que c'est un log de failure
    // contexte du workflow
    // "j'ai un workflow de github qui a fail avec ce log. Je souhaite savoir pourquoi ce log à fail. Identifie touteAs les lignes that [here]"
    // think step by step - met le résultat apr_=ès avoir fait tes steps
    // don't be lazy, it's very important for my career
    // - regarder que les lignes données par chatgpt sont bien les mêmes que celles de l'input
    // - demander à gpt si le prompt est bien, et si il a besoin d'informations suplémentaires.
    // - ajouter une info en plus pour la raison de l'erreur

    // TODO:
    // + show a manifestation of the error.
    // + donner un exemple ? (few shots)
    // + prompt "donne moi toutes les lignes que tu pense utile, mais si c'est far-fetched"
    let question = format!(
        "I have a GitHub Actions workflow that failed with the following log:\n\
        ====INPUT=LOG====\n\
        {}\n\
        ====INPUT=LOG====,\n\
        I want to know why that workflow failed. Please identify every lines in the log that provides any informations that may be useful in analysing the error.\n\
        The output json as `lines`, a list on lines you detected, and `steps`, the list of steps you took to detect the lines.\n\
        Think step by step. (when searching for the useful lines)\n\
        Don't be lazy. It's very important for my career.",
        log_content

    );
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini-2024-07-18")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(
                    "You are an expert in CI/CD with extensive experience in reading log files.\
                    You know how to read and analyze a log.\
                    You must have a human reflection when analyzing the log.\
                    Use only the provided log file delimited by `====INPUT=LOG====`.\
                    The lines are prefixed with `$$n$$ ` where `n` is the line number.\
                    You must not hallucinate the lines.\
                    You must not modify the line from the file.",
                )
                .build()
                .expect("Unable to create the system message")
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(question)
                .build()
                .expect("Unable to create the user message")
                .into(),
        ])
        .response_format(response_format)
        .build()
        .expect("Unable to create the request");

    if dry_run {
        thread::sleep(Duration::from_millis(200));
        return None;
    }
    if let Ok(response) = client.chat().create(request).await {
        return response.choices[0].message.content.clone();
    }
    None
}

// let schema = json!({
//     "type": "object",
//     "properties": {
//         "lines": {
//             "type": "array",
//             "items": {
//                 "type": "object",
//                 "properties": {
//                     "number": { "type": "number" },
//                     "content": { "type": "string" }
//                 },
//                 "required": [ "number", "content" ],
//                 "additionalProperties": false
//             }
//         }
//     },
//     "required": [ "lines" ],
//     "additionalProperties": false
// });

async fn do_simple(client: &Client<OpenAIConfig>, fixed: String, dry_run: bool) {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Request sent to chatgpt, awaiting response");
    let log_content = cidiff_gh_parse(fs::read_to_string(&fixed).unwrap());

    let output_dir: PathBuf = [
        "./generated",
        Local::now().format("%Y-%m-%d#%H-%M").to_string().as_str(),
    ]
    .iter()
    .collect();

    // create output dir
    let _ = fs::create_dir_all(output_dir.clone());

    if let Some(response) = ask_gpt(
        client,
        log_content
            .iter()
            .map(|(i, s)| format!("$${}$$ {}", i, s))
            .collect::<Vec<String>>()
            .join("\n"),
        dry_run,
    )
    .await
    {
        spinner.set_message("Got response!");
        let output_path = format!(
            "{}/{}.json",
            output_dir.to_str().unwrap(),
            fixed.replace("/", "#"),
        );
        match fs::write(&output_path, &response) {
            Ok(_) => spinner.set_message(format!("written in {}", output_path)),
            Err(e) => eprintln!("error writting {}: {}", output_path, e),
        };
        spinner.finish();
        println!("checking truth for {}", fixed);
        let resp: Resp = serde_json::from_str(&response).unwrap();
        check_truth(&resp, &log_content);
    }
}
