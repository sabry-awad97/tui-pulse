use crossterm::event::{Event as CEvent, KeyCode};
use pulse::{crossterm::event::KeyEventKind, prelude::*};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Line,
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

        // Handle key events
        if let Some(CEvent::Key(key)) = use_event()
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('+') => counter.set(count + 1),
                KeyCode::Char('-') => counter.set(count - 1),
                _ => {}
            }
        }

        let counter_widget = Paragraph::new(vec![
            Line::from(""),
            Line::from(format!("Counter: {}", count)),
            Line::from(""),
            Line::from("Press '+' to increment"),
            Line::from("Press '-' to decrement"),
        ])
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

        // Handle key events
        if let Some(CEvent::Key(key)) = use_event()
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Char('n')
        {
            let new_name = if name == "User" { "Guest" } else { "User" };
            user.set(new_name.to_string());
        }

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

impl App {
    fn new() -> Self {
        // Register a global event handler for the 't' key
        on_global_event(KeyCode::Char('t'), || {
            let current = USER_NAME.get();
            if current == "Test" {
                USER_NAME.set("Guest".to_string());
            } else {
                USER_NAME.set("Test".to_string());
            }
            true // Stop event propagation
        });

        App
    }
}

// 实现Component trait于App结构体
impl Component for App {
    fn on_mount(&self) {
        // Set up a global keyboard event handler
        on_global_event(KeyCode::Char('q'), || {
            request_exit();
            false
        });

        // Set up a global keyboard event handler for the counter
        on_global_event(KeyCode::Char('+'), || {
            let counter = COUNTER.get();
            COUNTER.set(counter + 1);
            true
        });

        on_global_event(KeyCode::Char('-'), || {
            let counter = COUNTER.get();
            COUNTER.set(counter - 1);
            true
        });
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        // Create a vertical layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(3),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        let header = Paragraph::new("Signal Example (Press 'q' to quit)")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(header, layout[0]);

        // Render content
        let content = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Counter
                Constraint::Length(6), // User greeting
            ])
            .split(layout[1]);

        // Render counter (handles its own events)
        CounterDisplay.render(content[0], frame);

        // Render user greeting (handles its own events)
        UserGreeting.render(content[1], frame);

        // Render footer with instructions
        let instructions = vec![
            Line::from("Counter: '+' to increment, '-' to decrement"),
            Line::from("Greeting: 'n' to toggle name"),
            Line::from("Press 'q' to quit"),
        ];
        let footer = Paragraph::new(instructions)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(footer, layout[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pulse::render(App::new)
}
