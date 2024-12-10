use std::{collections::HashMap, fs, path::Path, usize};

use apollo::parse_file;
use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{block::Title, Block, Paragraph, Scrollbar, ScrollbarState},
    Frame,
};
use regex::Regex;

use crate::{
    widgets::{LogFileWdiget, PathListWidget},
    WhatToDo,
};

pub trait AppState {
    fn handle_input(
        &mut self,
        area: Rect,
        event: &Event,
        clipboard: &mut ClipboardContext,
    ) -> WhatToDo;
    fn draw(&self, frame: &mut Frame);
    fn annotations(&self) -> HashMap<String, Vec<usize>>;
}

pub struct FileChooserState<'a> {
    start: usize,
    highlighted: usize,
    log_paths: &'a Vec<&'a str>,
    annotations: HashMap<String, Vec<usize>>,
}

impl<'a> FileChooserState<'a> {
    pub fn new(log_paths: &'a Vec<&'a str>, annotations: HashMap<String, Vec<usize>>) -> Self {
        Self {
            start: 0,
            highlighted: 0,
            log_paths,
            annotations,
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
    fn handle_input(
        &mut self,
        area: Rect,
        e: &Event,
        clipboard: &mut ClipboardContext,
    ) -> WhatToDo {
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
                        self.highlighted =
                            self.highlighted.saturating_sub((area.height as usize) / 2);
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
            .start(self.start)
            .annotated(self.annotations.keys().map(|s| &s[..]).collect());
        frame.render_widget(files, files_areas);

        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL);
        let mut scrollbar_state =
            ScrollbarState::new(self.log_paths.len() - files_areas.height as usize)
                .position(self.start);
        frame.render_stateful_widget(
            scrollbar,
            files_areas.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );

        let bottom_area = Rect::new(area.x, area.y + area.height - 3, area.width, 3);
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(15), Constraint::Percentage(85)])
            .split(bottom_area);
        let n = self.annotations.keys().len();
        let color = if n < 25 {
            Color::Red
        } else if n < 50 {
            Color::Yellow
        } else if n < 75 {
            Color::Cyan
        } else {
            Color::Green
        };
        let completion = Line::from(vec![Span::styled(
            format!("{}/100", n),
            Style::default().fg(color),
        )]);
        let completion_paragraph = Paragraph::new(completion)
            .block(
                Block::bordered()
                    .title(Title::from("Completion").alignment(Alignment::Center))
                    .border_set(border::THICK),
            )
            .alignment(Alignment::Center);
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
        frame.render_widget(completion_paragraph, layout[0]);
        frame.render_widget(instruction_paragraph, layout[1]);
    }

    fn annotations(&self) -> HashMap<String, Vec<usize>> {
        self.annotations.clone()
    }
}

pub struct FileOpenedState {
    start: usize,
    line_start: usize,
    highlighted: usize,
    log_path: String,
    lines: Vec<String>,
    annotations: HashMap<String, Vec<usize>>,
}

impl FileOpenedState {
    pub fn new(
        dataset_path: &str,
        log_path: String,
        annotations: HashMap<String, Vec<usize>>,
    ) -> Self {
        let lines = fs::read_to_string(Path::new(dataset_path).join(&log_path).join("failure.log"))
            .map(parse_file)
            .unwrap_or_default();
        Self {
            start: 0,
            line_start: 0,
            highlighted: 0,
            log_path,
            lines,
            annotations,
        }
    }
}

impl AppState for FileOpenedState {
    fn handle_input(&mut self, area: Rect, e: &Event, _: &mut ClipboardContext) -> WhatToDo {
        let mut toggle = || {
            self.annotations
                .entry(self.log_path.clone())
                .and_modify(|v| {
                    if v.contains(&self.highlighted) {
                        v.remove(v.iter().position(|x| x == &self.highlighted).unwrap());
                    } else {
                        v.push(self.highlighted)
                    }
                })
                .or_insert(vec![self.highlighted]);
        };
        match e {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => {
                    if self
                        .annotations
                        .get(&self.log_path)
                        .map(|v| v.is_empty())
                        .unwrap_or(false)
                    {
                        self.annotations.remove(&self.log_path);
                    }
                    return WhatToDo::ListDir;
                }
                KeyCode::Char('j') => self.highlighted += 1,
                KeyCode::Char('J') => {
                    toggle();
                    self.highlighted += 1;
                    self.start += 1;
                }
                KeyCode::Char('k') => self.highlighted = self.highlighted.saturating_sub(1),
                KeyCode::Char('K') => {
                    toggle();
                    self.highlighted = self.highlighted.saturating_sub(1);
                    self.start = self.start.saturating_sub(1);
                }
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
                        self.highlighted =
                            self.highlighted.saturating_sub((area.height as usize) / 2);
                    }
                }
                KeyCode::Char('g') => self.highlighted = 0,
                KeyCode::Char('G') => self.highlighted = self.lines.len() - 1,
                KeyCode::Char(' ') => {
                    self.annotations
                        .entry(self.log_path.clone())
                        .and_modify(|v| {
                            if v.contains(&self.highlighted) {
                                v.remove(v.iter().position(|x| x == &self.highlighted).unwrap());
                            } else {
                                v.push(self.highlighted)
                            }
                        })
                        .or_insert(vec![self.highlighted]);
                }
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
        let widget = LogFileWdiget::new(&self.lines, self.annotations.get(&self.log_path))
            .start(self.start)
            .line_start(self.line_start)
            .highlighted(self.highlighted);

        frame.render_widget(widget, widget_area);

        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL);
        let mut scrollbar_state =
            ScrollbarState::new(self.lines.len() - widget_area.height as usize)
                .position(self.start);
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
            Span::raw("Toggle line "),
            Span::styled("<Space>", Style::default().fg(Color::Blue)),
            Span::raw(" | Move "),
            Span::styled("<h> ", Style::default().fg(Color::Blue)),
            Span::styled("<j> ", Style::default().fg(Color::Blue)),
            Span::styled("<k> ", Style::default().fg(Color::Blue)),
            Span::styled("<l>", Style::default().fg(Color::Blue)),
            Span::raw(" | Toggle & move "),
            Span::styled("<J> ", Style::default().fg(Color::Blue)),
            Span::styled("<K> ", Style::default().fg(Color::Blue)),
            Span::raw(" | Top "),
            Span::styled("<g>", Style::default().fg(Color::Blue)),
            Span::raw(" | Bottom "),
            Span::styled("<G>", Style::default().fg(Color::Blue)),
            Span::raw(" | Return "),
            Span::styled("<q>", Style::default().fg(Color::Blue)),
        ]);
        let instruction_block = Block::bordered()
            .title("Instructions")
            .border_set(border::THICK);
        let instruction_paragraph = Paragraph::new(instructions)
            .block(instruction_block)
            .alignment(Alignment::Center);
        frame.render_widget(instruction_paragraph, layout[1]);
    }
    fn annotations(&self) -> HashMap<String, Vec<usize>> {
        self.annotations.clone()
    }
}
