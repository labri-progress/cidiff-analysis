use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Widget,
};

pub struct PathListWidget<'a> {
    lines: &'a Vec<&'a str>,
    start: usize,
    highlighted: usize,
    annotated: Vec<&'a str>,
}

impl<'a> PathListWidget<'a> {
    pub fn new(files: &'a Vec<&'a str>) -> Self {
        Self {
            lines: files,
            start: 0,
            highlighted: 0,
            annotated: vec![],
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

    pub fn annotated(mut self, annotated: Vec<&'a str>) -> Self {
        self.annotated = annotated;
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
            let annotated = self.annotated.contains(&self.lines[index]);
            let style = if annotated {
                Style::new().fg(Color::Green)
            } else {
                Style::new().fg(Color::Red)
            };
            let style = if self.highlighted == index {
                style.bold().bg(Color::Blue)
            } else {
                style
            };
            let line = Line::from(vec![
                Span::styled(if self.highlighted == index { ">" } else { " " }, style),
                Span::styled(if annotated { "✓" } else { " " }, style),
                Span::styled(self.lines[index], style),
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
    annotated: Option<&'a Vec<usize>>,
}

impl<'a> LogFileWdiget<'a> {
    pub fn new(lines: &'a Vec<String>, annotated: Option<&'a Vec<usize>>) -> Self {
        Self {
            lines,
            start: 0,
            line_start: 0,
            highlighted: 0,
            annotated,
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
            let annotated = self.annotated.map(|v| v.contains(&index)).unwrap_or(false);

            let style = if self.highlighted == index {
                Style::new().bold().underlined()
            } else {
                Style::new()
            };
            let style = if self.lines[index].to_lowercase().contains("warn") {
                style.fg(Color::Yellow)
            } else if self.lines[index].to_lowercase().contains("error") {
                style.fg(Color::Red)
            } else {
                style
            };
            let style = if annotated {
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
                if annotated {
                    Span::styled(" ✓ ", style.fg(Color::Blue))
                } else {
                    Span::styled("   ", style)
                },
                Span::styled(&text, style),
            ]);
            buf.set_line(area.x, area.y + i, &line, area.width);
        }
    }
}
