use std::{collections::HashMap, fs, path::Path};

use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarState},
    Frame,
};

use crate::{
    widgets::{LogFileWdiget, PathListWidget},
    WhatToDo,
};

pub trait AppState {
    fn handle_input(
        &mut self,
        area: Rect,
        event: &KeyEvent,
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
}

impl<'a> AppState for FileChooserState<'a> {
    fn handle_input(
        &mut self,
        area: Rect,
        key: &KeyEvent,
        clipboard: &mut ClipboardContext,
    ) -> WhatToDo {
        if key.kind == KeyEventKind::Press {
            match key.code {
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
                KeyCode::Enter => {
                    return WhatToDo::OpenFile(self.log_paths[self.highlighted].to_string())
                }
                _ => (),
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
            ScrollbarState::new(self.log_paths.len()).position(self.highlighted);
        frame.render_stateful_widget(
            scrollbar,
            files_areas.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
        let bottom_area = Rect::new(area.x, area.y + area.height - 3, area.width, 3);
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
            .title("Instructions")
            .border_set(border::THICK);
        let instruction_paragraph = Paragraph::new(instructions)
            .block(instruction_block)
            .alignment(Alignment::Center);
        frame.render_widget(instruction_paragraph, bottom_area);
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
            .map(|s| s.lines().map(|s| s.to_string()).collect::<Vec<String>>())
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
    fn handle_input(&mut self, area: Rect, key: &KeyEvent, _: &mut ClipboardContext) -> WhatToDo {
        if key.kind == KeyEventKind::Press {
            match key.code {
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
            }
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
        let mut scrollbar_state = ScrollbarState::new(self.lines.len()).position(self.highlighted);
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
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
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
            Span::raw("Toggle select "),
            Span::styled("<Space>", Style::default().fg(Color::Blue)),
            Span::raw(" | Move "),
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
