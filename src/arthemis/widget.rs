use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Widget,
};

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

impl Widget for PathListWidget<'_> {
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

impl Widget for LogFileWdiget<'_> {
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
                None => &vec![false, false, false, false, false, false, false],
            };

            // if selected by anyone
            let style = if selection[1] || selection[2] || selection[3] || selection[4] {
                style.fg(Color::Green)
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
                Span::styled(
                    [
                        if selection[1] { "☘" } else { " " },
                        if selection[2] { "⚐" } else { " " },
                        if selection[3] { "⚙" } else { " " },
                        if selection[4] { "⚷" } else { " " },
                        if selection[5] { "☍" } else { " " },
                        if selection[6] { "⛆" } else { " " },
                    ]
                    .join(""),
                    if selection.iter().all(|b| *b) {
                        style.fg(Color::Yellow)
                    } else {
                        style
                    },
                ),
                Span::styled(" ", style),
                Span::styled(
                    &text,
                    if selection[0] {
                        style.bg(Color::Blue)
                    } else {
                        style
                    },
                ),
            ]);
            buf.set_line(area.x, area.y + i, &line, area.width);
        }
    }
}
