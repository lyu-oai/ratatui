//! A Ratatui example that renders a compact inline panel with a red "Hello World" message.
//!
//! Unlike the fullscreen examples, this uses an inline viewport so it draws inside the existing
//! terminal buffer. It also includes a minimal prompt that echoes input, clearing the text when
//! you press Enter.
use std::ops::Range;
use std::time::Duration;

use color_eyre::{Result, eyre::Context};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame, TerminalOptions, Viewport};

fn main() -> Result<()> {
    color_eyre::install()?;
    let options = TerminalOptions {
        viewport: Viewport::Inline(8),
    };
    let mut terminal =
        ratatui::try_init_with_options(options).context("failed to initialize terminal")?;
    let result = run(&mut terminal);
    ratatui::try_restore().context("failed to restore terminal")?;
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| render(frame, &app))?;
        if handle_events(&mut app)? {
            break;
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let [output_area, input_area] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(3)]).areas(area);

    let output_block = Block::default().title("Output").borders(Borders::ALL);
    let output_inner = output_block.inner(output_area);
    frame.render_widget(output_block, output_area);

    let greeting = Paragraph::new(Line::from(vec![
        Span::raw("Hello "),
        Span::styled("World", Style::new().fg(Color::Red)),
        Span::raw(" 123 (press Ctrl+C to quit)"),
    ]));
    let [greeting_area, link_line_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(output_inner);
    frame.render_widget(greeting, greeting_area);

    let hyperlink_style = Style::new()
        .fg(Color::LightBlue)
        .add_modifier(Modifier::UNDERLINED);
    let prefix_span = Span::raw("Try clicking ");
    let prefix_width = prefix_span.width();
    let hyperlink_span = Span::styled(app.hyperlink_label, hyperlink_style);
    let hyperlink_width = hyperlink_span.width();
    let suffix_span = Span::raw(" here");
    let link_line = Line::from(vec![prefix_span, hyperlink_span, suffix_span]);
    frame.render_widget(
        HyperlinkLine::new(
            link_line,
            prefix_width..(prefix_width + hyperlink_width),
            app.hyperlink_url,
        ),
        link_line_area,
    );

    let input_block = Block::default().title("Input").borders(Borders::ALL);
    let input_inner = input_block.inner(input_area);
    frame.render_widget(input_block, input_area);
    let prompt = if app.input.is_empty() {
        "> ".to_string()
    } else {
        format!("> {}", app.input)
    };
    frame.render_widget(Paragraph::new(prompt), input_inner);
}

fn handle_events(app: &mut App) -> Result<bool> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        match event::read().context("event read failed")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(true);
                }
                KeyCode::Enter => {
                    app.input.clear();
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Char(c) => {
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    {
                        app.input.push(c);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(false)
}

/// Renders a single line of text with an OSC 8 hyperlink applied to a span of cells.
struct HyperlinkLine<'content> {
    line: Line<'content>,
    hyperlink_range: Range<usize>,
    url: &'content str,
}

impl<'content> HyperlinkLine<'content> {
    fn new(line: Line<'content>, hyperlink_range: Range<usize>, url: &'content str) -> Self {
        Self {
            line,
            hyperlink_range,
            url,
        }
    }
}

impl Widget for HyperlinkLine<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let HyperlinkLine {
            line,
            hyperlink_range,
            url,
        } = self;

        Paragraph::new(line).render(area, buffer);

        if area.height == 0 || area.width == 0 {
            return;
        }

        let start = hyperlink_range.start;
        let end = hyperlink_range.end;
        if start >= end {
            return;
        }

        let width = area.width as usize;
        if width == 0 || start >= width {
            return;
        }

        let start = start.min(width - 1);
        let end = end.min(width);
        if end <= start {
            return;
        }

        let row = area.y;
        let start_x = area.x + start as u16;
        let end_x = area.x + (end - 1) as u16;

        let open = format!("\x1B]8;;{}\x07", url);
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

struct App {
    input: String,
    hyperlink_label: &'static str,
    hyperlink_url: &'static str,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            hyperlink_label: "Google",
            hyperlink_url: "https://google.com",
        }
    }
}
