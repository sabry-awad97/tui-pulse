use pulse::prelude::*;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
struct CounterProps {
    initial_value: i32,
}

#[derive(Clone)]
struct Counter {
    props: CounterProps,
}

impl Counter {
    fn new(initial_value: i32) -> Self {
        Self {
            props: CounterProps { initial_value },
        }
    }
}

impl Component for Counter {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // Use the state hook to manage the counter value
        let (count, _set_count) = use_state(|| self.props.initial_value);

        // Handle keyboard events (this is a simplified example - in a real app,
        // you'd want to handle events in an event loop)
        // For this example, we'll just show how to use the state

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
        let title = Paragraph::new("Counter App with Hooks 🎣")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Counter display
        let counter_text = vec![
            Line::from(""), // Empty line for spacing
            Line::from(vec![Span::styled(
                format!("Count: {}", count.get()),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""), // Empty line for spacing
            Line::from(vec![
                Span::from("Status: "),
                Span::styled(
                    if count.get() == 0 {
                        "Zero"
                    } else if count.get() > 0 {
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
        let instructions = Paragraph::new(vec![
            Line::from("Press 'q' to quit, '+' to increment, '-' to decrement"),
            Line::from(format!("Current value: {}", count.get())),
        ])
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(instructions, chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Render the counter app with hooks support
    pulse::render(|| {
        // Create a new counter with initial value of 0
        Counter::new(0)
    })
}
