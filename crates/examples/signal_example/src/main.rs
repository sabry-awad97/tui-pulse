use crossterm::event::{KeyCode, KeyEvent};
use pulse::{
    crossterm::event::{Event, KeyEventKind},
    prelude::*,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

// Define global signals
static COUNTER: GlobalSignal<i32> = Signal::global(|| 0);
static USER_NAME: GlobalSignal<String> = Signal::global(|| String::from("Guest"));

// A component that displays the counter value
struct CounterDisplay;

impl Component for CounterDisplay {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // Use the global counter signal
        let counter = use_global_signal(&COUNTER);
        let count = counter.get();

        // Create a styled display of the counter
        let counter_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("Count: {}", count),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::from("Status: "),
                Span::styled(
                    if count == 0 {
                        "Zero"
                    } else if count > 0 {
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

        frame.render_widget(counter_widget, area);
    }
}

// A component that displays the user name
struct UserGreeting;

impl Component for UserGreeting {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // Use the global user name signal
        let user = use_global_signal(&USER_NAME);
        let name = user.get();

        let greeting = Paragraph::new(vec![
            Line::from(""),
            Line::from(format!("Hello, {}!", name)),
            Line::from(""),
            Line::from("Press 'n' to change name"),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("User"));

        frame.render_widget(greeting, area);
    }
}

// The main app component
struct App;

impl Component for App {
    fn on_mount(&self) {
        // Set initial values for our signals
        COUNTER.set(0);
        USER_NAME.set("Guest".to_string());
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        // Create a layout with three sections: header, content, and controls
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Header
        let title = Paragraph::new("Global Signal Example 🚀")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Content area with counter and user greeting
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Counter
                Constraint::Percentage(50), // User greeting
            ])
            .split(chunks[1]);

        // Render the counter display
        CounterDisplay.render(content[0], frame);

        // Render the user greeting
        UserGreeting.render(content[1], frame);

        // Controls
        let instructions = Paragraph::new(vec![
            Line::from("Press 'q' to quit, '+' to increment, '-' to decrement"),
            Line::from("Press 'r' to reset counter, 'n' to change name"),
        ])
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));

        frame.render_widget(instructions, chunks[2]);

        // Handle keyboard input
        if let Some(Event::Key(KeyEvent { code, kind, .. })) = frame.event()
            && *kind == KeyEventKind::Press
        {
            match code {
                KeyCode::Char('q') => request_exit(),
                KeyCode::Char('+') => COUNTER.update(|c| c + 1),
                KeyCode::Char('-') => COUNTER.update(|c| c - 1),
                KeyCode::Char('r') => COUNTER.reset(),
                KeyCode::Char('n') => {
                    // Toggle between Guest and User
                    let current = USER_NAME.get();
                    if current == "Guest" {
                        USER_NAME.set("User".to_string());
                    } else {
                        USER_NAME.set("Guest".to_string());
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Render the app with hooks support
    pulse::render(|| App)
}
