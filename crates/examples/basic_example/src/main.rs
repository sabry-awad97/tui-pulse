//! # Basic Ratatui Example
//!
//! This example demonstrates basic Ratatui widget rendering without terminal setup.
//! It shows how to:
//! - Create widgets programmatically
//! - Render widgets to a buffer
//! - Output rendered content to stdout
//!
//! This approach is useful for testing, debugging, or creating simple text-based output
//! without the complexity of terminal management.

use pulse::prelude::ratatui;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::io::{self, Write, stdout};

/// Represents different types of content that can be rendered
#[derive(Debug)]
enum ContentType {
    Header,
    Info,
    Data(usize),
    Footer,
}

impl ContentType {
    /// Returns the styled text for this content type
    fn to_styled_text(&self) -> Text<'static> {
        match self {
            ContentType::Header => Text::from(vec![Line::from(vec![
                Span::styled(
                    "TUI Pulse",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Basic Example"),
            ])]),
            ContentType::Info => Text::from(vec![
                Line::from("This demonstrates basic widget rendering"),
                Line::from("without terminal setup or event handling."),
            ]),
            ContentType::Data(index) => Text::from(vec![Line::from(vec![
                Span::raw("Data Item "),
                Span::styled(
                    format!("#{}", index),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": "),
                Span::styled(
                    format!("Value_{}", index * 42),
                    Style::default().fg(Color::Green),
                ),
            ])]),
            ContentType::Footer => Text::from(vec![Line::from(vec![
                Span::raw("Rendered with "),
                Span::styled(
                    "Ratatui",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::ITALIC),
                ),
                Span::raw(" buffer system"),
            ])]),
        }
    }

    /// Returns the appropriate block style for this content type
    fn block_style(&self) -> Block<'static> {
        match self {
            ContentType::Header => Block::default()
                .borders(Borders::ALL)
                .title("Header")
                .style(Style::default().fg(Color::Cyan)),
            ContentType::Info => Block::default()
                .borders(Borders::ALL)
                .title("Information")
                .style(Style::default().fg(Color::Blue)),
            ContentType::Data(_) => Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .style(Style::default().fg(Color::White)),
            ContentType::Footer => Block::default()
                .borders(Borders::ALL)
                .title("Footer")
                .style(Style::default().fg(Color::Gray)),
        }
    }
}

/// Renders a widget to a buffer and outputs it to stdout
fn render_widget_to_stdout<W>(widget: W, area: Rect) -> io::Result<()>
where
    W: Widget,
{
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    let mut stdout = stdout();

    // Output each line of the buffer
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buffer.cell((x, y)) {
                write!(stdout, "{}", cell.symbol())?;
            }
        }
        writeln!(stdout)?;
    }

    stdout.flush()?;
    Ok(())
}

/// Creates a layout for the example content
fn create_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(4), // Info
            Constraint::Min(1),    // Data items
            Constraint::Length(3), // Footer
        ])
        .split(area)
        .to_vec()
}

fn main() -> io::Result<()> {
    println!("=== TUI Pulse Basic Example ===\n");

    // Define the rendering area (width=60, height=20)
    let total_area = Rect::new(0, 0, 60, 20);
    let layout = create_layout(total_area);

    // Content to render
    let content_items = [
        ContentType::Header,
        ContentType::Info,
        ContentType::Data(1),
        ContentType::Data(2),
        ContentType::Data(3),
        ContentType::Footer,
    ];

    // Render header
    let header_widget =
        Paragraph::new(content_items[0].to_styled_text()).block(content_items[0].block_style());
    render_widget_to_stdout(header_widget, layout[0])?;

    // Render info section
    let info_widget =
        Paragraph::new(content_items[1].to_styled_text()).block(content_items[1].block_style());
    render_widget_to_stdout(info_widget, layout[1])?;

    // Render data items in the middle section
    let data_area = layout[2];
    let data_height = 2; // Height per data item

    for (i, content) in content_items[2..5].iter().enumerate() {
        let item_area = Rect::new(
            data_area.x,
            data_area.y + (i as u16 * data_height),
            data_area.width,
            data_height,
        );

        let data_widget = Paragraph::new(content.to_styled_text()).block(content.block_style());
        render_widget_to_stdout(data_widget, item_area)?;
    }

    // Render footer
    let footer_widget =
        Paragraph::new(content_items[5].to_styled_text()).block(content_items[5].block_style());
    render_widget_to_stdout(footer_widget, layout[3])?;

    println!("\n=== Example Complete ===");
    println!("This example demonstrates:");
    println!("• Widget creation and styling");
    println!("• Layout management");
    println!("• Buffer-based rendering");
    println!("• Structured content organization");

    Ok(())
}
