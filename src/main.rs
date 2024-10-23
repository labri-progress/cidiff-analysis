mod states;
mod widgets;

use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

use clap::Parser;
use copypasta::ClipboardContext;
use indicatif::ProgressStyle;
use rand::{Rng, SeedableRng};
use ratatui::{
    crossterm::{
        event::{self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture},
        execute,
    },
    DefaultTerminal,
};
use states::{AppState, FileChooserState, FileOpenedState};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    dataset: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let log_paths = list_log_paths(&args.dataset);
    let annotations = load_annotations("annotations.toml");
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableFocusChange, EnableMouseCapture)?;
    terminal.clear()?;
    let app_result = run(terminal, &args.dataset, log_paths, annotations);
    execute!(std::io::stdout(), DisableFocusChange, DisableMouseCapture)?;
    match app_result {
        Ok(annotation) => {
            save_annotations("annotations.toml", annotation);
            ratatui::restore();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn run(
    mut terminal: DefaultTerminal,
    dataset_path: &str,
    log_paths: Vec<PathBuf>,
    annotations: HashMap<String, Vec<usize>>,
) -> io::Result<HashMap<String, Vec<usize>>> {
    let mut log_paths = log_paths
        .iter()
        .map(|p| p.to_str().unwrap())
        .collect::<Vec<&str>>();
    log_paths.sort();
    let mut clipboard = ClipboardContext::new().unwrap();
    let mut state: Box<dyn AppState> = Box::new(FileChooserState::new(&log_paths, annotations));
    loop {
        let completer_frames = terminal.draw(|frame| {
            state.draw(frame);
        })?;
        let area = completer_frames.area;
        let e = event::read()?;
        let what_to_do = state.handle_input(area, &e, &mut clipboard);
        match what_to_do {
            WhatToDo::Exit => return Ok(state.annotations()),
            WhatToDo::StayOnSameState => {}
            WhatToDo::OpenFile(log_path) => {
                state = Box::new(FileOpenedState::new(
                    dataset_path,
                    log_path,
                    state.annotations(),
                ));
            }
            WhatToDo::ListDir => {
                state = Box::new(FileChooserState::new(&log_paths, state.annotations()));
            }
        }
    }
}

enum WhatToDo {
    Exit,
    StayOnSameState,
    OpenFile(String),
    ListDir,
}

/// List the paths containing the log pairs to annotate.
/// It selects 100 paths.
/// The vec is unsorted.
fn list_log_paths(dataset_path: &str) -> Vec<PathBuf> {
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
    paths
}

/// Load the annotations from a specific file.
fn load_annotations(filename: &str) -> HashMap<String, Vec<usize>> {
    match fs::read_to_string(filename) {
        Ok(content) => toml::from_str(&content).unwrap_or(HashMap::new()),
        Err(_) => HashMap::new(),
    }
}

/// Save the annotations to the give file
fn save_annotations(filename: &str, annotations: HashMap<String, Vec<usize>>) {
    if let Ok(content) = toml::to_string_pretty(&annotations) {
        let _ = fs::write(filename, content);
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
                if linecount::count_lines(fs::File::open(path.join("failure.log")).unwrap())
                    .unwrap()
                    <= 1000
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
