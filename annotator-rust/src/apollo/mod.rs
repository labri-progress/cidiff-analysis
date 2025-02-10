use std::{collections::HashMap, io, path::PathBuf};

use crate::{
    apollo::state::{FileChooser, FileOpened},
    load_annotations, save_annotations, WhatToDo,
};
use clap::Args;
use copypasta::ClipboardContext;
use ratatui::{
    crossterm::{
        event::{
            self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
        },
        execute,
    },
    layout::Rect,
    DefaultTerminal, Frame,
};

mod state;
mod widget;

/// A tool to annotate a list of log files
#[derive(Debug, Args)]
pub struct ApolloArgs {
    /// The tom file to save/load the annotations to/from.
    #[arg(short, long, default_value_t = String::from("annotations.toml"))]
    output: String,
    /// If the program should output the annotations as a csv too. (using the `output` filename)
    #[arg(short, long)]
    to_csv: bool,
}
pub fn bootstrap(args: ApolloArgs, dataset_path: &str, log_paths: Vec<PathBuf>) -> io::Result<()> {
    let annotations = load_annotations(&args.output);
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableFocusChange, EnableMouseCapture)?;
    terminal.clear()?;
    let app_result = run(terminal, dataset_path, log_paths, annotations);
    execute!(std::io::stdout(), DisableFocusChange, DisableMouseCapture)?;
    match app_result {
        Ok(annotation) => {
            save_annotations(&args.output, annotation, args.to_csv);
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
    let mut state: Box<dyn State> = Box::new(FileChooser::new(&log_paths, annotations));
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
                state = Box::new(FileOpened::new(
                    dataset_path,
                    log_paths[path_index].to_string(),
                    state.annotations(),
                ));
            }
            WhatToDo::ListDir => {
                state = Box::new(
                    FileChooser::new(&log_paths, state.annotations())
                        .start(last_position.0)
                        .highlighted(last_position.1),
                );
            }
        }
    }
}
trait State {
    fn handle_input(&mut self, area: Rect, event: &Event, clipboard: &mut ClipboardContext) -> WhatToDo;
    fn draw(&self, frame: &mut Frame);
    fn annotations(&self) -> HashMap<String, Vec<usize>>;
}
