use std::{collections::HashMap, io, path::PathBuf};

use crate::{
    arthemis::state::{FileChooser, FileOpened},
    WhatToDo,
};
use clap::Parser;
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

/// A tool to visualise the results of the annotation
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct ArthemisArgs {
    /// The path to the human annotation csv
    human_path: String,
    /// The path to the anotation produced by the algorithms seed, lcs, gpt, keyword,
    /// bigram-raw, and bigram-drain
    merged_path: String,
}

pub fn bootstrap(args: ArthemisArgs, dataset_path: &str, log_paths: Vec<PathBuf>) -> io::Result<()> {
    let annotations = load_merged_selections(args.merged_path);
    println!("selection computed");
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableFocusChange, EnableMouseCapture)?;
    terminal.clear()?;
    run(terminal, dataset_path, log_paths, annotations)?;
    execute!(std::io::stdout(), DisableFocusChange, DisableMouseCapture)?;
    ratatui::restore();
    Ok(())
}

type Record = (String, String, usize);

/// Load the selection of the algorithms as a map:
/// log path -> map of (line_number -> selections_by_algorithm)
/// selections_by_algorithm: `[human, cidiff, lcs, gpt, keyword]`
fn load_merged_selections(merged_path: String) -> HashMap<String, HashMap<usize, Vec<bool>>> {
    let mut map = HashMap::new();
    let reader = csv::ReaderBuilder::new().from_path(&merged_path);
    if let Ok(mut reader) = reader {
        // to compute the size of the csv, iter over the record, then go back to the start of the csv
        let start = reader.position().clone();
        let size = reader.records().count();
        let _ = reader.seek(start);
        for (csv_index, record) in reader.deserialize::<Record>().enumerate() {
            if let Ok(record) = record {
                let path = record.0;
                let selected_line = record.2;
                print!("\rreading line {}/{}", csv_index, size);
                let i = match &record.1[..] {
                    "human" => 0,
                    "seed" => 1,
                    "lcs" => 2,
                    "gpt" => 3,
                    "keyword" => 4,
                    "bigram-raw" => 5,
                    "bigram-drain" => 6,
                    _ => 10,
                };
                map.entry(path.to_string())
                    .and_modify(|a: &mut HashMap<usize, Vec<bool>>| {
                        a.entry(selected_line)
                            .and_modify(|selections| selections[i] = true)
                            .or_insert(vec![i == 0, i == 1, i == 2, i == 3, i == 4, i == 5, i == 6]);
                    })
                    .or_insert(HashMap::from([(
                        selected_line,
                        vec![i == 0, i == 1, i == 2, i == 3, i == 4, i == 5, i == 6],
                    )]));
            } else {
                println!("error reading line {}: {}\n", csv_index, record.unwrap_err());
            }
        }
        println!();
    }
    map
}

fn run(
    mut terminal: DefaultTerminal,
    dataset_path: &str,
    log_paths: Vec<PathBuf>,
    annotations: HashMap<String, HashMap<usize, Vec<bool>>>,
) -> io::Result<()> {
    let mut log_paths = log_paths
        .iter()
        .map(|p| p.to_str().unwrap())
        .collect::<Vec<&str>>();
    log_paths.sort();
    let mut clipboard = ClipboardContext::new().unwrap();
    let mut state: Box<dyn State> = Box::new(FileChooser::new(&log_paths));
    let mut last_position = (0, 0);
    loop {
        let completer_frames = terminal.draw(|frame| {
            state.draw(frame);
        })?;
        let area = completer_frames.area;
        let e = event::read()?;
        let what_to_do = state.handle_input(area, &e, &mut clipboard);
        match what_to_do {
            WhatToDo::Exit => return Ok(()),
            WhatToDo::StayOnSameState => {}
            WhatToDo::OpenFile((start, path_index)) => {
                last_position = (start, path_index);
                state = Box::new(FileOpened::new(
                    dataset_path,
                    log_paths[path_index].to_string(),
                    annotations.get(log_paths[path_index]).unwrap().clone(),
                ));
            }
            WhatToDo::ListDir => {
                state = Box::new(
                    FileChooser::new(&log_paths)
                        .start(last_position.0)
                        .highlighted(last_position.1),
                );
            }
        }
    }
}

pub trait State {
    fn handle_input(&mut self, area: Rect, event: &Event, clipboard: &mut ClipboardContext) -> WhatToDo;
    fn draw(&self, frame: &mut Frame);
}
