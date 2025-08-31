use chrono::Local;
use crossterm::event::{KeyCode, KeyEventKind};
use pulse::{crossterm::event::Event, prelude::*};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};
use std::{cell::RefCell, collections::VecDeque};

// Global log state for tracking mount/unmount events
thread_local! {
    static GLOBAL_LOG: RefCell<VecDeque<String>> = const { RefCell::new(VecDeque::new()) };
}

fn add_log_message(message: String) {
    GLOBAL_LOG.with(|log| {
        let mut log = log.borrow_mut();
        log.push_back(message);
        if log.len() > 10 {
            log.pop_front();
        }
    });
}

fn get_log_messages() -> VecDeque<String> {
    GLOBAL_LOG.with(|log| log.borrow().clone())
}

fn clear_log_messages() {
    GLOBAL_LOG.with(|log| log.borrow_mut().clear());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    pulse::render(|| App)
}

#[derive(Clone)]
struct App;

impl Component for App {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (active_panels, set_active_panels) = use_state(|| vec![true, true, false, false]);

        // Handle key events
        if let Some(event) = use_event()
            && let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('1') => {
                    let mut panels = active_panels.get().clone();
                    panels[0] = !panels[0];
                    set_active_panels.set(panels);
                }
                KeyCode::Char('2') => {
                    let mut panels = active_panels.get().clone();
                    panels[1] = !panels[1];
                    set_active_panels.set(panels);
                }
                KeyCode::Char('3') => {
                    let mut panels = active_panels.get().clone();
                    panels[2] = !panels[2];
                    set_active_panels.set(panels);
                }
                KeyCode::Char('4') => {
                    let mut panels = active_panels.get().clone();
                    panels[3] = !panels[3];
                    set_active_panels.set(panels);
                }
                KeyCode::Char('c') => {
                    clear_log_messages();
                }
                _ => {}
            }
        }

        // Main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Content
                Constraint::Length(5), // Log panel
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        HeaderComponent.render(main_chunks[0], frame);

        // Content area layout
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[1]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[1]);

        // Create static component instances
        static SYSTEM_PANEL: SystemStatsPanel = SystemStatsPanel;
        static NETWORK_PANEL: NetworkMonitorPanel = NetworkMonitorPanel;
        static TASK_PANEL: TaskManagerPanel = TaskManagerPanel;
        static WEATHER_PANEL: WeatherPanel = WeatherPanel;

        // Render panels conditionally using persistent instances
        let panels = active_panels.get();

        if panels[0] {
            SYSTEM_PANEL.render_with_mount(left_chunks[0], frame);
        }

        if panels[1] {
            NETWORK_PANEL.render_with_mount(left_chunks[1], frame);
        }

        if panels[2] {
            TASK_PANEL.render_with_mount(right_chunks[0], frame);
        }

        if panels[3] {
            WEATHER_PANEL.render_with_mount(right_chunks[1], frame);
        }

        // Render log panel
        LogPanel {
            messages: get_log_messages(),
        }
        .render(main_chunks[2], frame);

        // Render footer
        FooterComponent.render(main_chunks[3], frame);
    }
}

#[derive(Clone)]
struct HeaderComponent;

impl Component for HeaderComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let title = Paragraph::new("🚀 Pulse TUI - Component Lifecycle Dashboard")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .border_set(border::ROUNDED),
            );
        frame.render_widget(title, area);
    }
}

#[derive(Clone)]
struct SystemStatsPanel;

impl Component for SystemStatsPanel {
    fn component_id(&self) -> String {
        "system_stats_panel".to_string()
    }

    fn on_mount(&self) {
        add_log_message(format!(
            "[{}] 📊 System Stats Panel mounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn on_unmount(&self) {
        add_log_message(format!(
            "[{}] 📊 System Stats Panel unmounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let (cpu_usage, _) = use_state(|| 45);
        let (memory_usage, _) = use_state(|| 67);

        let block = Block::default()
            .title("📊 System Stats")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_set(border::ROUNDED);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(inner);

        // CPU gauge
        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU"))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(cpu_usage.get())
            .label(format!("{}%", cpu_usage.get()));
        frame.render_widget(cpu_gauge, chunks[0]);

        // Memory gauge
        let memory_gauge = Gauge::default()
            .block(Block::default().title("Memory"))
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(memory_usage.get())
            .label(format!("{}%", memory_usage.get()));
        frame.render_widget(memory_gauge, chunks[1]);

        // Status text
        let status = Paragraph::new("✅ All systems operational")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);
    }
}

#[derive(Clone)]
struct NetworkMonitorPanel;

impl Component for NetworkMonitorPanel {
    fn component_id(&self) -> String {
        "network_monitor_panel".to_string()
    }

    fn on_mount(&self) {
        add_log_message(format!(
            "[{}] 🌐 Network Monitor mounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn on_unmount(&self) {
        add_log_message(format!(
            "[{}] 🌐 Network Monitor unmounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let connections = [
            "🔗 api.example.com:443 - Connected",
            "🔗 db.internal:5432 - Connected",
            "🔗 cache.redis:6379 - Connected",
            "⚠️  backup.service:22 - Timeout",
        ];

        let items: Vec<ListItem> = connections
            .iter()
            .map(|conn| {
                let style = if conn.contains("⚠️") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                };
                ListItem::new(*conn).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("🌐 Network Monitor")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .border_set(border::ROUNDED),
        );

        frame.render_widget(list, area);
    }
}

#[derive(Clone)]
struct TaskManagerPanel;

impl Component for TaskManagerPanel {
    fn component_id(&self) -> String {
        "task_manager_panel".to_string()
    }

    fn on_mount(&self) {
        add_log_message(format!(
            "[{}] ⚙️  Task Manager mounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn on_unmount(&self) {
        add_log_message(format!(
            "[{}] ⚙️  Task Manager unmounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let tasks = [
            "🟢 Data Sync - Running",
            "🟡 Backup Job - Pending",
            "🔴 Log Rotation - Failed",
            "🟢 Health Check - Running",
            "🟡 Cache Cleanup - Queued",
        ];

        let items: Vec<ListItem> = tasks
            .iter()
            .map(|task| {
                let style = if task.contains("🔴") {
                    Style::default().fg(Color::Red)
                } else if task.contains("🟡") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                };
                ListItem::new(*task).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("⚙️  Task Manager")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .border_set(border::ROUNDED),
        );

        frame.render_widget(list, area);
    }
}

#[derive(Clone)]
struct WeatherPanel;

impl Component for WeatherPanel {
    fn component_id(&self) -> String {
        "weather_panel".to_string()
    }

    fn on_mount(&self) {
        add_log_message(format!(
            "[{}] 🌤️  Weather Panel mounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn on_unmount(&self) {
        add_log_message(format!(
            "[{}] 🌤️  Weather Panel unmounted",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let weather_text = Text::from(vec![
            Line::from(vec![
                Span::styled("🌤️  ", Style::default().fg(Color::Yellow)),
                Span::styled("Partly Cloudy", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🌡️  ", Style::default().fg(Color::Red)),
                Span::styled("22°C", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("💨 ", Style::default().fg(Color::Cyan)),
                Span::styled("15 km/h NW", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("💧 ", Style::default().fg(Color::Blue)),
                Span::styled("65% Humidity", Style::default().fg(Color::White)),
            ]),
        ]);

        let paragraph = Paragraph::new(weather_text)
            .block(
                Block::default()
                    .title("🌤️  Weather")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

#[derive(Clone)]
struct LogPanel {
    messages: VecDeque<String>,
}

impl Component for LogPanel {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let items: Vec<ListItem> = self
            .messages
            .iter()
            .map(|msg| {
                let style = if msg.contains("mounted") {
                    Style::default().fg(Color::Green)
                } else if msg.contains("unmounted") {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(msg.as_str()).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("📋 Component Lifecycle Log")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .border_set(border::ROUNDED),
        );

        frame.render_widget(list, area);
    }
}

#[derive(Clone)]
struct FooterComponent;

impl Component for FooterComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let help_text = "Press 1-4 to toggle panels | Press 'c' to clear log | Press 'q' to quit";
        let footer = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .border_set(border::ROUNDED),
            );
        frame.render_widget(footer, area);
    }
}
