//! Minimal Ratatui example that prints a single OSC 8 hyperlink inline in the existing buffer.

use color_eyre::{eyre::Context, Result};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Widget};
use ratatui::{Frame, TerminalOptions, Viewport};
use std::ops::Range;

fn main() -> Result<()> {
    color_eyre::install()?;
    let options = TerminalOptions {
        viewport: Viewport::Inline(2),
    };
    let mut terminal =
        ratatui::try_init_with_options(options).context("failed to initialize terminal")?;
    terminal.draw(render)?;
    ratatui::try_restore().context("failed to restore terminal")
}

fn render(frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let hyperlink_style = Style::new()
        .fg(Color::LightBlue)
        .add_modifier(Modifier::UNDERLINED);
    let prefix = Span::raw("This is a link to ");
    let prefix_width = prefix.width();
    let link = Span::styled("example", hyperlink_style);
    let link_width = link.width();
    let suffix = Span::raw(" website!");

    let line = Line::from(vec![prefix, link, suffix]);
    let hyperlink = HyperlinkLine::new(
        line,
        prefix_width..(prefix_width + link_width),
        "https://example.com/",
    );
    frame.render_widget(hyperlink, area);
}

/// Renders a single line of text with an OSC 8 hyperlink applied to a span of cells.
struct HyperlinkLine<'a> {
    line: Line<'a>,
    hyperlink_range: Range<usize>,
    url: &'a str,
}

impl<'a> HyperlinkLine<'a> {
    fn new(line: Line<'a>, hyperlink_range: Range<usize>, url: &'a str) -> Self {
        Self {
            line,
            hyperlink_range,
            url,
        }
    }
}

impl Widget for HyperlinkLine<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.line.clone()).render(area, buffer);

        if area.width == 0 || area.height == 0 {
            return;
        }

        let start = self.hyperlink_range.start.min(area.width as usize - 1);
        let end = self.hyperlink_range.end.min(area.width as usize);
        if end <= start {
            return;
        }

        let row = area.y;
        let start_x = area.x + start as u16;
        let end_x = area.x + (end - 1) as u16;
        let open = format!("\x1B]8;;{}\x07", self.url);
        let close = "\x1B]8;;\x07";

        let start_symbol = buffer[(start_x, row)].symbol().to_string();
        if start_x == end_x {
            buffer[(start_x, row)].set_symbol(&format!("{open}{start_symbol}{close}"));
        } else {
            buffer[(start_x, row)].set_symbol(&format!("{open}{start_symbol}"));
            let end_symbol = buffer[(end_x, row)].symbol().to_string();
            buffer[(end_x, row)].set_symbol(&format!("{end_symbol}{close}"));
        }
    }
}
