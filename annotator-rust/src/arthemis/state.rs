use std::{collections::HashMap, fs, path::Path};

use crate::{
    arthemis::{
        widget::{LogFileWdiget, PathListWidget},
        State,
    },
    parse_file, WhatToDo,
};
use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{block::Title, Block, Paragraph, Scrollbar, ScrollbarState},
    Frame,
};

pub struct FileChooser<'a> {
    start: usize,
    highlighted: usize,
    log_paths: &'a Vec<&'a str>,
}
pub struct FileOpened {
    start: usize,
    line_start: usize,
    highlighted: usize,
    log_path: String,
    lines: Vec<String>,
    selections: HashMap<usize, Vec<bool>>,
}
impl<'a> FileChooser<'a> {
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

impl State for FileChooser<'_> {
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

impl FileOpened {
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

impl State for FileOpened {
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
                        self.start += (area.height / 2) as usize;
                    }
                }
                KeyCode::Char('u') => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        self.highlighted = self.highlighted.saturating_sub((area.height as usize) / 2);
                        self.start = self.start.saturating_sub((area.height as usize) / 2);
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
        let widget = LogFileWdiget::new(
            &self.lines,
            self.selections.clone(),
            //self.drain.keys().map(|k| *k).collect(),
        )
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
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ])
            .split(bottom_area);

        let file_block = Block::bordered().title("File").border_set(border::THICK);
        let file_text = Span::styled(&self.log_path[..], Style::default().fg(Color::Cyan));
        let file_paragraph = Paragraph::new(file_text).block(file_block);
        frame.render_widget(file_paragraph, layout[0]);

        let symbols_block = Block::bordered().title("Symbols").border_set(border::THICK);
        let symbols_text = Span::raw("☘ Cidiff | ⚐ Lcs | ⚙ Gpt | ⚷ Keyword | ☍ Bigram | ⛆ Bigram-drain");
        let symbols_paragraph = Paragraph::new(symbols_text).block(symbols_block);

        frame.render_widget(symbols_paragraph, layout[1]);

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
        frame.render_widget(instruction_paragraph, layout[2]);
    }
}
