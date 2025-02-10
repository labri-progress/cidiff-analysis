use std::{fs, path::Path};

use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// Mean length of the logs (in word count)
struct Args {
    /// The path of the dataset
    dataset: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let dataset_path = Path::new(&args.dataset);
    let mut v = vec![];
    let mut v2 = vec![];
    for path in fs::read_to_string(Path::new("paths.txt"))?.lines() {
        let p = dataset_path.join(path).join("failure.log");
        let c = fs::read_to_string(p)?.split_whitespace().count();
        v.push(c);
        let p2 = dataset_path.join(path).join("success.log");
        let c2 = fs::read_to_string(p2)?.split_whitespace().count();
        v2.push(c2);
    }

    let words_f: f32 = v.iter().sum::<usize>() as f32;
    println!("mean failure: {}", words_f / v.len() as f32);
    println!("sum failure: {}", words_f);
    // 1000 tokens = 750 words
    println!("tokens failure: {}", words_f * 1000.0 / 750.0);

    let words_s: f32 = v2.iter().sum::<usize>() as f32;
    println!("mean success: {}", words_s / v2.len() as f32);
    println!("sum success: {}", words_s);
    println!("tokens success: {}", words_s * 1000.0 / 750.0);

    let words_a = words_s + words_f;
    println!("mean: {}", words_a / v2.len() as f32);
    println!("sum: {}", words_a);
    println!("tokens: {}", words_a * 1000.0 / 750.0);

    Ok(())
}
