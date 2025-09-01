//! Personal Task Manager - A beautiful showcase of the local storage hook
//!
//! This example demonstrates persistent state management with an interactive
//! task manager that automatically saves your data.

use chrono::{DateTime, Utc};
use crossterm::event::{Event, KeyCode};
use pulse::{crossterm::event::KeyEventKind, prelude::*};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Beautiful themes with color schemes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Ocean,
    Forest,
    Sunset,
}

impl Theme {
    fn name(&self) -> &str {
        match self {
            Theme::Ocean => "🌊 Ocean",
            Theme::Forest => "🌲 Forest",
            Theme::Sunset => "🌅 Sunset",
        }
    }

    fn primary_color(&self) -> Color {
        match self {
            Theme::Ocean => Color::Rgb(52, 152, 219),
            Theme::Forest => Color::Rgb(39, 174, 96),
            Theme::Sunset => Color::Rgb(255, 87, 34),
        }
    }

    fn accent_color(&self) -> Color {
        match self {
            Theme::Ocean => Color::Rgb(241, 196, 15),
            Theme::Forest => Color::Rgb(255, 193, 7),
            Theme::Sunset => Color::Rgb(255, 235, 59),
        }
    }
}

/// Task with priority and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub completed: bool,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Priority {
    Low,
    High,
    Urgent,
}

impl Priority {
    fn symbol(&self) -> &str {
        match self {
            Priority::Low => "🟢",
            Priority::High => "🟡",
            Priority::Urgent => "🔴",
        }
    }
}

/// Application state that persists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub tasks: Vec<Task>,
    pub theme: Theme,
    pub total_sessions: u64,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            tasks: vec![Task {
                id: 1,
                title: "Welcome! Press 'n' to add tasks 🎉".to_string(),
                completed: false,
                priority: Priority::High,
                created_at: Utc::now(),
            }],
            theme: Theme::Ocean,
            total_sessions: 1,
        }
    }
}

/// Main application component
#[derive(Clone)]
pub struct TaskManager;

impl Component for TaskManager {
    fn on_mount(&self) {
        // Configure beautiful storage location
        set_storage_config(LocalStorageConfig {
            storage_dir: PathBuf::from("./beautiful_tasks"),
            create_dir: true,
            file_extension: "json".to_string(),
            pretty_json: true,
        });
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        // Persistent app data
        let (app_data, set_app_data) =
            use_local_storage("task_app".to_string(), AppData::default());

        // UI state
        let (selected_index, set_selected_index) = use_state(|| 0usize);
        let (show_help, set_show_help) = use_state(|| false);

        // Handle input
        if let Some(Event::Key(key)) = use_event()
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => request_exit(),
                KeyCode::Char('h') => set_show_help.update(|h| !h),
                KeyCode::Char('t') => {
                    set_app_data.update(|data| AppData {
                        theme: match data.theme {
                            Theme::Ocean => Theme::Forest,
                            Theme::Forest => Theme::Sunset,
                            Theme::Sunset => Theme::Ocean,
                        },
                        ..data.clone()
                    });
                }
                KeyCode::Char('n') => {
                    let new_task = Task {
                        id: chrono::Utc::now().timestamp_millis() as u64,
                        title: format!("New Task #{}", app_data.get().tasks.len() + 1),
                        completed: false,
                        priority: Priority::High,
                        created_at: Utc::now(),
                    };
                    set_app_data.update(|data| {
                        let mut new_data = data.clone();
                        new_data.tasks.push(new_task);
                        new_data
                    });
                }
                KeyCode::Enter => {
                    let tasks = &app_data.get().tasks;
                    if !tasks.is_empty() && selected_index.get() < tasks.len() {
                        set_app_data.update(|data| {
                            let mut new_data = data.clone();
                            new_data.tasks[selected_index.get()].completed =
                                !new_data.tasks[selected_index.get()].completed;
                            new_data
                        });
                    }
                }
                KeyCode::Up => {
                    set_selected_index.update(|i| {
                        if *i > 0 {
                            *i - 1
                        } else {
                            app_data.get().tasks.len().saturating_sub(1)
                        }
                    });
                }
                KeyCode::Down => {
                    set_selected_index.update(|i| (*i + 1) % app_data.get().tasks.len().max(1));
                }
                _ => {}
            }
        }

        let data = app_data.get();
        let theme = &data.theme;

        if show_help.get() {
            render_help_overlay(area, frame, theme);
            return;
        }

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header with theme colors
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled("✨ ", Style::default().fg(theme.accent_color())),
            Span::styled(
                "Beautiful Task Manager",
                Style::default()
                    .fg(theme.primary_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ✨", Style::default().fg(theme.accent_color())),
        ])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary_color())),
        )
        .alignment(Alignment::Center);

        frame.render_widget(header, chunks[0]);

        // Content area
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // Task list
        render_task_list(content_chunks[0], frame, &data, selected_index.get(), theme);

        // Statistics panel
        render_stats_panel(content_chunks[1], frame, &data, theme);

        // Footer with controls
        let footer_text = vec![Line::from(vec![
            Span::styled(
                "Controls: ",
                Style::default()
                    .fg(theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("↑↓", Style::default().fg(theme.primary_color())),
            Span::styled(" Navigate | ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(theme.primary_color())),
            Span::styled(" Toggle | ", Style::default().fg(Color::Gray)),
            Span::styled("n", Style::default().fg(theme.primary_color())),
            Span::styled(" New | ", Style::default().fg(Color::Gray)),
            Span::styled("t", Style::default().fg(theme.primary_color())),
            Span::styled(" Theme | ", Style::default().fg(Color::Gray)),
            Span::styled("h", Style::default().fg(theme.primary_color())),
            Span::styled(" Help | ", Style::default().fg(Color::Gray)),
            Span::styled("q", Style::default().fg(theme.primary_color())),
            Span::styled(" Quit", Style::default().fg(Color::Gray)),
        ])];

        let footer = Paragraph::new(footer_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary_color())),
            )
            .alignment(Alignment::Center);

        frame.render_widget(footer, chunks[2]);
    }
}

fn render_task_list(area: Rect, frame: &mut Frame, data: &AppData, selected: usize, theme: &Theme) {
    let items: Vec<ListItem> = data
        .tasks
        .iter()
        .map(|task| {
            let status = if task.completed { "✅" } else { "⭕" };
            let style = if task.completed {
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::raw(format!("{} {} ", status, task.priority.symbol())),
                Span::styled(&task.title, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary_color()))
                .title(" 📋 Tasks "),
        )
        .highlight_style(Style::default().bg(theme.primary_color()).fg(Color::Black))
        .highlight_symbol("▶ ");

    let mut list_state = ratatui::widgets::ListState::default();
    if !data.tasks.is_empty() {
        list_state.select(Some(selected.min(data.tasks.len() - 1)));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_stats_panel(area: Rect, frame: &mut Frame, data: &AppData, theme: &Theme) {
    let completed = data.tasks.iter().filter(|t| t.completed).count();
    let total = data.tasks.len();
    let completion_rate = if total > 0 {
        completed as f64 / total as f64
    } else {
        0.0
    };

    // Use the priority counting function
    let priority_counts = count_tasks_by_priority(&data.tasks);
    let urgent_count = priority_counts.get(&Priority::Urgent).unwrap_or(&0);
    let high_count = priority_counts.get(&Priority::High).unwrap_or(&0);
    let low_count = priority_counts.get(&Priority::Low).unwrap_or(&0);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Progress gauge
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📊 Progress "),
        )
        .gauge_style(Style::default().fg(theme.accent_color()))
        .percent((completion_rate * 100.0) as u16)
        .label(format!("{}/{}", completed, total));

    frame.render_widget(gauge, chunks[0]);

    // Statistics with priority breakdown
    let stats_text = vec![
        Line::from(vec![
            Span::styled("🎯 Sessions: ", Style::default().fg(theme.accent_color())),
            Span::styled(
                data.total_sessions.to_string(),
                Style::default().fg(theme.primary_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("🎨 Theme: ", Style::default().fg(theme.accent_color())),
            Span::styled(theme.name(), Style::default().fg(theme.primary_color())),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Priority Breakdown:",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("🔴 Urgent: ", Style::default().fg(Color::Red)),
            Span::styled(urgent_count.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("🟡 High: ", Style::default().fg(Color::Yellow)),
            Span::styled(high_count.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("🟢 Low: ", Style::default().fg(Color::Green)),
            Span::styled(low_count.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("💾 Auto-Save: ", Style::default().fg(theme.accent_color())),
            Span::styled("✅ Enabled", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("📁 Location: ", Style::default().fg(theme.accent_color())),
            Span::styled("./beautiful_tasks/", Style::default().fg(Color::Gray)),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(" ℹ️  Info "))
        .style(Style::default().fg(Color::White));

    frame.render_widget(stats, chunks[1]);
}

fn render_help_overlay(area: Rect, frame: &mut Frame, theme: &Theme) {
    let popup_area = centered_rect(60, 70, area);

    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "🎯 Task Manager Help",
            Style::default()
                .fg(theme.primary_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑↓ - Navigate tasks"),
        Line::from("  Enter - Toggle task completion"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  n - Create new task"),
        Line::from("  t - Cycle themes"),
        Line::from("  h - Toggle this help"),
        Line::from("  q - Quit (auto-saves!)"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Features:",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ✨ Beautiful animated UI"),
        Line::from("  💾 Automatic persistence"),
        Line::from("  🎨 Multiple themes"),
        Line::from("  📊 Progress tracking"),
        Line::from(""),
        Line::from("Press 'h' again to close"),
    ];

    let help_popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary_color()))
                .title(" 📖 Help "),
        )
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    frame.render_widget(help_popup, popup_area);
}

fn count_tasks_by_priority(tasks: &[Task]) -> std::collections::HashMap<Priority, usize> {
    let mut counts = std::collections::HashMap::new();
    for task in tasks {
        *counts.entry(task.priority.clone()).or_insert(0) += 1;
    }
    counts
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pulse::render_async(|| async { TaskManager }).await?;

    Ok(())
}
