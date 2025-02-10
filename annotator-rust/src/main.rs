use std::{collections::HashMap, fs::{self, DirEntry, File}, io::{self, BufWriter, Write}, path::{Path, PathBuf}};

use clap::{Parser, Subcommand};
use indicatif::ProgressStyle;
use rand::{Rng, SeedableRng};
use regex::Regex;

mod apollo;
mod arthemis;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct AppArgs {
    #[command(subcommand)]
    command: Commands,
    // The path to the dataset
    dataset: String,
    /// Path to a file containing a list of paths from the dataset to use (instead of randomly
    /// selecting the paths)
    #[arg(short, long)]
    paths: Option<String>
}

#[derive(Debug, Subcommand)]
enum Commands {
    Annotate(apollo::ApolloArgs),
    Visu(arthemis::ArthemisArgs),
}

fn main() -> io::Result<()> {
    let args = AppArgs::parse();
    let log_paths = match args.paths {
        None => list_log_paths(&args.dataset),
        Some(file_path) => list_log_paths_from_file(&file_path)
    };
    match args.command {
        Commands::Annotate(apollo_args) => apollo::bootstrap(apollo_args, &args.dataset, log_paths),
        Commands::Visu(arthemis_args) => arthemis::bootstrap(arthemis_args, &args.dataset, log_paths),
    }
}
pub enum WhatToDo {
    Exit,
    StayOnSameState,
    OpenFile((usize, usize)),
    ListDir,
}
/// List the paths containing the log pairs to annotate.
/// It selects 100 paths.
/// The vec is unsorted.
pub fn list_log_paths(dataset_path: &str) -> Vec<PathBuf> {
    // list every log pairs from the dataset
    let mut all_paths = vec![];
    let dataset_path = Path::new(dataset_path);
    let bar = indicatif::ProgressBar::new_spinner();
    bar.set_style(ProgressStyle::with_template("{spinner} {prefix} {msg}").unwrap());
    bar.set_prefix("Listing dataset files: ");
    let mut callback = |entry: &DirEntry| {
        bar.tick();
        let _ = entry.path().strip_prefix(dataset_path).map(|path| {
            all_paths.push(path.to_path_buf());
            bar.set_message(path.to_str().unwrap().to_string());
        });
    };
    let _ = visit_dirs(dataset_path, &mut callback);
    bar.finish_and_clear();
    // select 100 logs from the dataset
    all_paths.sort();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(123456789);
    let mut paths = vec![];
    while paths.len() < 100 {
        let i = rng.gen_range(0..all_paths.len());
        let p = &all_paths[i];
        if !paths.contains(p) {
            paths.push(p.to_path_buf());
        }
    }
    let f = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("paths.txt")
        .unwrap();
    let mut writer = BufWriter::new(f);
    for ele in paths.iter() {
        let _ = writeln!(writer, "{}", ele.to_str().unwrap());
    }
    let _ = writer.flush();
    paths
}

/// List the paths containing the log pairs to annotate from a file containing them
pub fn list_log_paths_from_file(file_path: &str) -> Vec<PathBuf> {
    if let Ok(content) = fs::read_to_string(file_path) {
        content.lines().map(|p|Path::new(p).to_path_buf()).collect()
    } else {
        vec![]
    }
}

/// Load the annotations from a specific file.
pub fn load_annotations(filename: &str) -> HashMap<String, Vec<usize>> {
    match fs::read_to_string(filename) {
        Ok(content) => toml::from_str(&content).unwrap_or(HashMap::new()),
        Err(_) => HashMap::new(),
    }
}

/// Save the annotations to the give file
pub fn save_annotations(filename: &str, annotations: HashMap<String, Vec<usize>>, to_csv: bool) {
    if let Ok(content) = toml::to_string(&annotations) {
        let _ = fs::write(filename, content);
    }
    if to_csv {
        // remove the .toml and replace it by a .csv
        let filename = format!("{}.csv", &filename[..(filename.chars().count() - 5)]);
        if let Ok(mut output) = csv::WriterBuilder::new().from_path(filename) {
            let _ = output.write_record(["path", "type", "line"]);
            for (log_path, selected_lines) in annotations {
                for n in selected_lines {
                    let _ = output.write_record([log_path.to_string(), "human".to_string(), n.to_string()]);
                }
            }
        };
    }
}

/// Visit every directory recurively and call the callback when the directory contains the files
/// `failure.log` and `succes.log` and both files are less than 1000 lines
fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.join("success.log").exists() && path.join("failure.log").exists() {
                if linecount::count_lines(fs::File::open(path.join("failure.log")).unwrap()).unwrap() <= 1000
                {
                    cb(&entry);
                }
            } else {
                visit_dirs(&path, cb)?;
            }
        }
    }
    Ok(())
}

/// Parse the file. By default, we remove the github timestamp at the begining of each line, and
/// remove any ANSI escape code
pub fn parse_file(file_content: String) -> Vec<String> {
    let timestamp_regex = Regex::new(r"(?:\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{7}Z ?)?(.*)").unwrap();
    let ansi_color_regex = Regex::new(r"\x1b?\[(?:\d+)?(?:;\d+)*m").unwrap();
    let mut lines = vec![];
    for line in file_content.lines() {
        let caps = timestamp_regex.captures(line).unwrap();
        let content = &caps[1];
        let cleaned = ansi_color_regex.replace_all(content, "");
        if !cleaned.trim().is_empty() {
            lines.push(cleaned.to_string());
        }
    }
    lines
}
