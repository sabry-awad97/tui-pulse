use pulse::prelude::*;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Counter {
    value: Arc<Mutex<i32>>,
}

impl Counter {
    fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(0)),
        }
    }

    fn increment(&self) {
        if let Ok(mut val) = self.value.lock() {
            *val += 1;
        }
    }

    fn decrement(&self) {
        if let Ok(mut val) = self.value.lock() {
            *val -= 1;
        }
    }

    fn get_value(&self) -> i32 {
        *self.value.lock().unwrap_or_else(|poisoned| {
            // Handle poisoned mutex by recovering the data
            poisoned.into_inner()
        })
    }
}

impl Component for Counter {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(5), // Counter display
                Constraint::Length(3), // Instructions
                Constraint::Min(0),    // Spacer
            ])
            .split(area);

        // Title
        let title = Paragraph::new("TUI Pulse Counter App")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Counter display
        let value = self.get_value();
        let counter_text = vec![
            Line::from(vec![
                Span::raw("Current Value: "),
                Span::styled(
                    format!("{}", value),
                    Style::default()
                        .fg(if value >= 0 { Color::Green } else { Color::Red })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    if value == 0 {
                        "Zero"
                    } else if value > 0 {
                        "Positive"
                    } else {
                        "Negative"
                    },
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ];

        let counter_widget = Paragraph::new(counter_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Counter"));
        frame.render_widget(counter_widget, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Press 'q' to quit, '+' to increment, '-' to decrement",
        )])
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(instructions, chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let counter = Counter::new();

    // For now, just render once with some demo increments
    counter.increment();
    counter.increment();
    counter.increment();
    counter.increment();
    counter.decrement();

    pulse::render(|| counter.clone())
}
