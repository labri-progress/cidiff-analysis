mod states;
mod widgets;

use std::{collections::HashMap, io, path::PathBuf};

use apollo::{list_log_paths, load_annotations, save_annotations, WhatToDo};
use clap::Parser;
use copypasta::ClipboardContext;
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
    /// The path of the dataset
    dataset: String,
    /// The file to save the annotations to
    #[arg(short, long, default_value_t = String::from("annotations.toml"))]
    output: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let log_paths = list_log_paths(&args.dataset);
    let annotations = load_annotations(&args.output);
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableFocusChange, EnableMouseCapture)?;
    terminal.clear()?;
    let app_result = run(terminal, &args.dataset, log_paths, annotations);
    execute!(std::io::stdout(), DisableFocusChange, DisableMouseCapture)?;
    match app_result {
        Ok(annotation) => {
            save_annotations(&args.output, annotation);
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
    let mut last_position = (0, 0);
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
            WhatToDo::OpenFile((start, path_index)) => {
                last_position = (start, path_index);
                state = Box::new(FileOpenedState::new(
                    dataset_path,
                    log_paths[path_index].to_string(),
                    state.annotations(),
                ));
            }
            WhatToDo::ListDir => {
                state = Box::new(
                    FileChooserState::new(&log_paths, state.annotations())
                        .start(last_position.0)
                        .highlighted(last_position.1),
                );
            }
        }
    }
}

