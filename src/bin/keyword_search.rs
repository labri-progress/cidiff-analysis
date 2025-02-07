use std::{
    fs::{self, File},
    io::{BufWriter, Result, Write},
    path::PathBuf,
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;

#[derive(Debug, Parser)]
struct Args {
    dataset: String,
    logs_file: String,
}

const KEYWORDS: [&str; 6] = ["failure", "fail", "failed", "error", "exception", "panic"];

/// The experiment on a simple algorithm. The algorithm detect useful lines only by keyword search.
/// i.e. if a specific keyword is present in the line, then the line is considered useful.
fn main() -> Result<()> {
    let args = Args::parse();
    let logs_file_content = fs::read_to_string(args.logs_file).expect("Unable to read the logs file");
    let paths: Vec<_> = logs_file_content.lines().map(PathBuf::from).collect();
    let bar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("[{pos}/{len}] {msg} {wide_bar}").unwrap());
    let dataset_path = PathBuf::from(args.dataset);
    let mut output = BufWriter::new(File::create("keyword.csv")?);
    writeln!(output, "path,type,line")?;
    for path in paths {
        let s = path.to_str().unwrap();
        bar.inc(1);
        bar.set_message(path.to_str().unwrap().to_string());
        let log_path = dataset_path.join(&path).join("failure.log");
        let log_content = parse_file(fs::read_to_string(&log_path).unwrap());
        for (i, line) in log_content.iter().enumerate() {
            let lower = line.to_lowercase();
            for keyword in KEYWORDS {
                if lower.contains(keyword) {
                    writeln!(output, "{},keyword,{}", s, i)?;
                    break;
                }
            }
        }
    }
    bar.finish_and_clear();

    output.flush()?;

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
