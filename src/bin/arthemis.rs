use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use apollo::{list_log_paths, parse_file, WhatToDo};
use clap::Parser;
use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::{
    crossterm::{
        event::{
            self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
            KeyCode, KeyEventKind, KeyModifiers, MouseEventKind,
        },
        execute,
    },
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{block::Title, Block, Paragraph, Scrollbar, ScrollbarState, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path of the dataset
    dataset: String,
    /// The path to the human annotation csv
    human_path: String,
    /// The path to the algorithms annotation csv
    algos_path: String,
    /// The path to the chatgpt annotation csv
    gpt_path: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let log_paths = list_log_paths(&args.dataset);
    let annotations = load_selections(args.human_path, args.algos_path, args.gpt_path);
    println!("selection computed");
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableFocusChange, EnableMouseCapture)?;
    terminal.clear()?;
    run(terminal, &args.dataset, log_paths, annotations)?;
    execute!(std::io::stdout(), DisableFocusChange, DisableMouseCapture)?;
    ratatui::restore();
    Ok(())
}

fn load_selections(
    human_path: String,
    algos_path: String,
    gpt_path: String,
) -> HashMap<String, HashMap<usize, Vec<bool>>> {
    let mut map = HashMap::new();
    if let Ok(content) = fs::read_to_string(&human_path) {
        println!("reading human selection");
        let size = content.lines().count() - 1;
        content.lines().skip(1).enumerate().for_each(|(i, line)| {
            let s: Vec<&str> = line.split(",").collect();
            let path = s[0];
            let line: usize = s[2].parse().unwrap();
            //{
            //    Ok(l) => l,
            //    Err(e) => {
            //        eprintln!("error parsing `{}` from {}: {}", line, &human_path, e);
            //        0
            //    }
            //};
            print!("\rreading line {}/{}", i, size);
            map.entry(path.to_string())
                .and_modify(|a: &mut HashMap<usize, Vec<bool>>| {
                    a.entry(line)
                        .and_modify(|line| line[0] = true)
                        .or_insert(vec![true, false, false, false]);
                })
                .or_insert(HashMap::from([(line, vec![true, false, false, false])]));
            if human_path.contains("Milan") {
                println!("{:?}", map.entry(path.to_string()));
            }
        });
        println!();
    }
    if let Ok(content) = fs::read_to_string(algos_path) {
        let size = content.lines().count() - 1;
        println!("reading algos selection");
        content.lines().skip(1).enumerate().for_each(|(i, line)| {
            let s: Vec<&str> = line.split(",").collect();
            let path = s[0];
            let line: usize = s[2].parse().unwrap();
            print!("\rreading line {}/{}", i, size);
            map.entry(path.to_string())
                .and_modify(|a: &mut HashMap<usize, Vec<bool>>| {
                    let i = match s[1] {
                        "seed" => 1,
                        "lcs" => 2,
                        _ => 10,
                    };
                    a.entry(line).and_modify(|line| line[i] = true).or_insert(vec![
                        false,
                        i == 1,
                        i == 2,
                        false,
                    ]);
                })
                .or_insert(HashMap::from([(line, vec![false, i == 1, i == 2, false])]));
        });
        println!();
    }
    if let Ok(content) = fs::read_to_string(gpt_path) {
        let size = content.lines().count() - 1;
        println!("reading gpt selection");
        content.lines().skip(1).enumerate().for_each(|(i, line)| {
            let s: Vec<&str> = line.split(",").collect();
            let path = &s[0][..(s[0].len() - 12)];
            let line: usize = s[2].parse().unwrap();
            print!("\rreading line {}/{}", i, size);
            map.entry(path.to_string())
                .and_modify(|a: &mut HashMap<usize, Vec<bool>>| {
                    a.entry(line)
                        .and_modify(|line| line[3] = true)
                        .or_insert(vec![false, false, false, true]);
                })
                .or_insert(HashMap::from([(line, vec![false, false, false, true])]));
        });
        println!();
    }
    //map.iter().for_each(|(p,m)| {
    //    m.iter().for_each(|(l,v)| {
    //        if v[3] {
    //            println!("found for {} {}", p, l);
    //        }
    //    });
    //});
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
    let mut state: Box<dyn AppState> = Box::new(FileChooserState::new(&log_paths));
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
                state = Box::new(FileOpenedState::new(
                    dataset_path,
                    log_paths[path_index].to_string(),
                    annotations.get(log_paths[path_index]).unwrap().clone(),
                ));
            }
            WhatToDo::ListDir => {
                state = Box::new(
                    FileChooserState::new(&log_paths)
                        .start(last_position.0)
                        .highlighted(last_position.1),
                );
            }
        }
    }
}

pub trait AppState {
    fn handle_input(&mut self, area: Rect, event: &Event, clipboard: &mut ClipboardContext) -> WhatToDo;
    fn draw(&self, frame: &mut Frame);
}

pub struct FileChooserState<'a> {
    start: usize,
    highlighted: usize,
    log_paths: &'a Vec<&'a str>,
}

impl<'a> FileChooserState<'a> {
    pub fn new(log_paths: &'a Vec<&'a str>) -> Self {
        Self {
            start: 0,
            highlighted: 0,
            log_paths,
        }
    }

    pub fn start(mut self, start: usize) -> Self {
        self.start = start;
        self
    }

    pub fn highlighted(mut self, highlighted: usize) -> Self {
        self.highlighted = highlighted;
        self
    }
}

impl<'a> AppState for FileChooserState<'a> {
    fn handle_input(&mut self, area: Rect, e: &Event, clipboard: &mut ClipboardContext) -> WhatToDo {
        match e {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => return WhatToDo::Exit,
                KeyCode::Char('j') => self.highlighted += 1,
                KeyCode::Char('k') => self.highlighted = self.highlighted.saturating_sub(1),
                KeyCode::Char('d') => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        self.highlighted += (area.height / 2) as usize;
                    }
                }
                KeyCode::Char('u') => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        self.highlighted = self.highlighted.saturating_sub((area.height as usize) / 2);
                    }
                }
                KeyCode::Char('g') => self.highlighted = 0,
                KeyCode::Char('G') => self.highlighted = self.log_paths.len() - 1,
                KeyCode::Char('y') => {
                    let _ = clipboard.set_contents(self.log_paths[self.highlighted].to_string());
                }
                KeyCode::Enter => return WhatToDo::OpenFile((self.start, self.highlighted)),
                _ => (),
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    if self.start >= 2 {
                        self.start = self.start.saturating_sub(2);
                        self.highlighted = self.highlighted.saturating_sub(2);
                    }
                }
                MouseEventKind::ScrollDown => {
                    if self.start + 2 + area.height as usize - 4 < self.log_paths.len() {
                        self.start += 2;
                        self.highlighted += 2;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        if self.highlighted >= self.log_paths.len() {
            self.highlighted = self.log_paths.len() - 1;
        }
        if self.start > self.highlighted {
            self.start = self.highlighted;
        }
        if self.start + area.height as usize - 4 < self.highlighted {
            self.start = self.highlighted - (area.height as usize - 4);
        }
        WhatToDo::StayOnSameState
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let files_areas = Rect::new(area.x, area.y, area.width, area.height - 3);
        let files = PathListWidget::new(self.log_paths)
            .highlighted(self.highlighted)
            .start(self.start);
        frame.render_widget(files, files_areas);

        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL);
        let mut scrollbar_state =
            ScrollbarState::new(self.log_paths.len() - files_areas.height as usize).position(self.start);
        frame.render_stateful_widget(
            scrollbar,
            files_areas.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );

        let instruction_area = Rect::new(area.x, area.y + area.height - 3, area.width, 3);
        let instructions = Line::from(vec![
            Span::raw("Open File "),
            Span::styled("<Enter>", Style::default().fg(Color::Blue)),
            Span::raw(" | Move "),
            Span::styled("<j> ", Style::default().fg(Color::Blue)),
            Span::styled("<k>", Style::default().fg(Color::Blue)),
            Span::raw(" | Top "),
            Span::styled("<g>", Style::default().fg(Color::Blue)),
            Span::raw(" | Bottom "),
            Span::styled("<G>", Style::default().fg(Color::Blue)),
            Span::raw(" | Exit "),
            Span::styled("<q>", Style::default().fg(Color::Blue)),
        ]);
        let instruction_block = Block::bordered()
            .title(Title::from("Instructions").alignment(Alignment::Center))
            .border_set(border::THICK);
        let instruction_paragraph = Paragraph::new(instructions)
            .block(instruction_block)
            .alignment(Alignment::Center);
        frame.render_widget(instruction_paragraph, instruction_area);
    }
}

pub struct PathListWidget<'a> {
    paths: &'a Vec<&'a str>,
    start: usize,
    highlighted: usize,
}

impl<'a> PathListWidget<'a> {
    pub fn new(files: &'a Vec<&'a str>) -> Self {
        Self {
            paths: files,
            start: 0,
            highlighted: 0,
        }
    }

    pub fn highlighted(mut self, highlighted: usize) -> Self {
        self.highlighted = highlighted;
        self
    }

    pub fn start(mut self, start: usize) -> Self {
        self.start = start;
        self
    }
}

impl<'a> Widget for PathListWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for i in 0..area.height {
            let index = i as usize + self.start;
            if index >= self.paths.len() {
                break;
            }

            let style = if self.highlighted == index {
                Style::new().bold().bg(Color::Blue)
            } else {
                Style::new()
            };
            let line = Line::from(vec![
                Span::styled(if self.highlighted == index { ">" } else { " " }, style),
                Span::styled(self.paths[index], style),
            ]);
            buf.set_line(area.x, area.y + i, &line, area.width);
        }
    }
}
pub struct FileOpenedState {
    start: usize,
    line_start: usize,
    highlighted: usize,
    log_path: String,
    lines: Vec<String>,
    selections: HashMap<usize, Vec<bool>>,
}
impl FileOpenedState {
    pub fn new(dataset_path: &str, log_path: String, selections: HashMap<usize, Vec<bool>>) -> Self {
        let lines = fs::read_to_string(Path::new(dataset_path).join(&log_path).join("failure.log"))
            .map(parse_file)
            .unwrap_or_default();
        Self {
            start: 0,
            line_start: 0,
            highlighted: 0,
            log_path,
            lines,
            selections,
        }
    }
}

impl AppState for FileOpenedState {
    fn handle_input(&mut self, area: Rect, e: &Event, _: &mut ClipboardContext) -> WhatToDo {
        match e {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => {
                    return WhatToDo::ListDir;
                }
                KeyCode::Char('j') => self.highlighted += 1,
                KeyCode::Char('k') => self.highlighted = self.highlighted.saturating_sub(1),
                KeyCode::Char('l') => self.line_start += 1,
                KeyCode::Char('L') => self.line_start += 10,
                KeyCode::Char('h') => self.line_start = self.line_start.saturating_sub(1),
                KeyCode::Char('H') => self.line_start = self.line_start.saturating_sub(10),
                KeyCode::Char('d') => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        self.highlighted += (area.height / 2) as usize;
                    }
                }
                KeyCode::Char('u') => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        self.highlighted = self.highlighted.saturating_sub((area.height as usize) / 2);
                    }
                }
                KeyCode::Char('g') => self.highlighted = 0,
                KeyCode::Char('G') => self.highlighted = self.lines.len() - 1,
                _ => (),
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    if self.start >= 2 {
                        self.start = self.start.saturating_sub(2);
                        self.highlighted = self.highlighted.saturating_sub(2);
                    }
                }
                MouseEventKind::ScrollDown => {
                    if self.start + 2 + area.height as usize - 4 < self.lines.len() {
                        self.start += 2;
                        self.highlighted += 2;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        if self.highlighted >= self.lines.len() {
            self.highlighted = self.lines.len() - 1;
        }
        if self.start > self.highlighted {
            self.start = self.highlighted;
        }
        if self.start + area.height as usize - 4 < self.highlighted {
            self.start = self.highlighted - (area.height as usize - 4);
        }
        WhatToDo::StayOnSameState
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let widget_area = Rect::new(area.x, area.y, area.width, area.height - 3);
        let widget = LogFileWdiget::new(&self.lines, self.selections.clone())
            .start(self.start)
            .line_start(self.line_start)
            .highlighted(self.highlighted);

        frame.render_widget(widget, widget_area);

        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL);
        let mut scrollbar_state =
            ScrollbarState::new(self.lines.len() - widget_area.height as usize).position(self.start);
        frame.render_stateful_widget(
            scrollbar,
            widget_area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );

        let bottom_area = Rect::new(area.x, area.y + area.height - 3, area.width, 3);
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(bottom_area);

        let status_block = Block::bordered().title("Status").border_set(border::THICK);
        let status_text = Line::from(vec![
            Span::styled(&self.log_path[..], Style::default().fg(Color::Cyan)),
            "  ".into(),
            format!("{}", self.highlighted).into(),
            "/".into(),
            format!("{}", self.lines.len()).into(),
        ]);
        let status = Paragraph::new(status_text).block(status_block);
        frame.render_widget(status, layout[0]);

        let instructions = Line::from(vec![
            Span::raw("Move "),
            Span::styled("<h> ", Style::default().fg(Color::Blue)),
            Span::styled("<j> ", Style::default().fg(Color::Blue)),
            Span::styled("<k> ", Style::default().fg(Color::Blue)),
            Span::styled("<l>", Style::default().fg(Color::Blue)),
            Span::raw(" | Top "),
            Span::styled("<g>", Style::default().fg(Color::Blue)),
            Span::raw(" | Bottom "),
            Span::styled("<G>", Style::default().fg(Color::Blue)),
            Span::raw(" | Return "),
            Span::styled("<q>", Style::default().fg(Color::Blue)),
        ]);
        let instruction_block = Block::bordered().title("Instructions").border_set(border::THICK);
        let instruction_paragraph = Paragraph::new(instructions)
            .block(instruction_block)
            .alignment(Alignment::Center);
        frame.render_widget(instruction_paragraph, layout[1]);
    }
}

pub struct LogFileWdiget<'a> {
    lines: &'a Vec<String>,
    start: usize,
    line_start: usize,
    highlighted: usize,
    selected: HashMap<usize, Vec<bool>>,
}

impl<'a> LogFileWdiget<'a> {
    pub fn new(lines: &'a Vec<String>, selected: HashMap<usize, Vec<bool>>) -> Self {
        Self {
            lines,
            start: 0,
            line_start: 0,
            highlighted: 0,
            selected,
        }
    }

    pub fn highlighted(mut self, highlighted: usize) -> Self {
        self.highlighted = highlighted;
        self
    }

    pub fn start(mut self, start: usize) -> Self {
        self.start = start;
        self
    }

    pub fn line_start(mut self, line_start: usize) -> Self {
        self.line_start = line_start;
        self
    }
}

impl<'a> Widget for LogFileWdiget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for i in 0..area.height {
            let index = i as usize + self.start;
            if index >= self.lines.len() {
                break;
            }

            let style = if self.highlighted == index {
                Style::new().bold().underlined()
            } else {
                Style::new()
            };

            let selection = match &self.selected.get(&index) {
                Some(s) => s,
                None => &vec![false, false, false, false],
            };

            // fg blue if selected by gpt 
            let style = if selection[3] {
                style.fg(Color::Blue)
            } else {
                style
            };

            // bg green if selected by gpt 
            let style = if selection[0] {
                style.bg(Color::Green)
            } else {
                style
            };

            let text: String = self.lines[index].chars().skip(self.line_start).collect();
            let line = Line::from(vec![
                Span::styled(
                    format!("{:1$}", index, self.lines.len().to_string().chars().count()),
                    style.fg(Color::DarkGray),
                ),
                if self.highlighted == index {
                    Span::styled(" > ", style)
                } else {
                    Span::styled("   ", style)
                },
                Span::styled("[", style),
                Span::styled(
                    selection
                        .iter()
                        .map(|b| if *b { "✓" } else { " " })
                        .collect::<String>(),
                    style,
                ),
                Span::styled("]", style),
                Span::styled(&text, style),
            ]);
            buf.set_line(area.x, area.y + i, &line, area.width);
        }
    }
}
