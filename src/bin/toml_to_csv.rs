use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
};

use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    input: String,
    output: String,
}

fn main() {
    let args = Args::parse();
    if let Ok(content) = fs::read_to_string(args.input) {
        let map: HashMap<String, Vec<usize>> = toml::from_str(&content).unwrap_or(HashMap::new());
        let f = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(args.output)
            .unwrap();
        //let f = File::open(args.output).unwrap();
        let mut writer = BufWriter::new(f);
        let _ = writeln!(writer, "path,type,line");
        for (key, value) in map {
            for i in value {
                let _ = writeln!(writer, "{},human,{}", key, i);
            }
        }
    }
}
