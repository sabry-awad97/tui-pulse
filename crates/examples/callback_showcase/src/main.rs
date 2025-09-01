//! Callback Showcase - A beautiful demonstration of the use_callback hook
//!
//! This example shows memoized callbacks, dependency tracking, and performance
//! optimization in an interactive counter application.

use crossterm::event::{Event, KeyCode, KeyEventKind};
use pulse::prelude::*;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::time::Instant;

/// Beautiful themes for the callback showcase
#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Electric,
    Neon,
    Cyber,
}

impl Theme {
    fn name(&self) -> &str {
        match self {
            Theme::Electric => " Electric",
            Theme::Neon => " Neon",
            Theme::Cyber => " Cyber",
        }
    }

    fn primary_color(&self) -> Color {
        match self {
            Theme::Electric => Color::Rgb(255, 215, 0),
            Theme::Neon => Color::Rgb(255, 20, 147),
            Theme::Cyber => Color::Rgb(0, 255, 255),
        }
    }

    fn accent_color(&self) -> Color {
        match self {
            Theme::Electric => Color::Rgb(138, 43, 226),
            Theme::Neon => Color::Rgb(50, 205, 50),
            Theme::Cyber => Color::Rgb(255, 165, 0),
        }
    }
}

/// Callback performance metrics
#[derive(Debug, Clone, Default)]
struct CallbackMetrics {
    execution_count: usize,
    last_execution: Option<Instant>,
    dependency_changes: usize,
}

/// Application state for the callback showcase
#[derive(Debug, Clone)]
struct AppState {
    counter: i32,
    multiplier: i32,
    step_size: i32,
    theme: Theme,
    metrics: CallbackMetrics,
    message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 0,
            multiplier: 2,
            step_size: 1,
            theme: Theme::Electric,
            metrics: CallbackMetrics::default(),
            message: "🚀 Welcome to Callback Showcase!".to_string(),
        }
    }
}

/// Main application component showcasing callbacks
#[derive(Clone)]
pub struct CallbackShowcase;

impl Component for CallbackShowcase {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // Application state
        let (app_state, set_app_state) = use_state(AppState::default);
        let (show_help, set_show_help) = use_state(|| false);

        let state = app_state.get();

        // Memoized increment callback - recreates only when multiplier or step_size changes
        let increment_callback = use_callback(
            {
                let set_state = set_app_state.clone();
                move |amount: i32| {
                    set_state.update(|state| {
                        let mut new_state = state.clone();
                        new_state.counter += amount * new_state.multiplier * new_state.step_size;
                        new_state.metrics.execution_count += 1;
                        new_state.metrics.last_execution = Some(Instant::now());
                        new_state.message = format!(
                            "⚡ Incremented by {} (×{} step {})",
                            amount, new_state.multiplier, new_state.step_size
                        );
                        new_state
                    });
                }
            },
            (state.multiplier, state.step_size),
        );

        // Memoized reset callback - no dependencies, created once
        let reset_callback = use_callback_once({
            let set_state = set_app_state.clone();
            move |_: ()| {
                set_state.update(|state| {
                    let mut new_state = state.clone();
                    new_state.counter = 0;
                    new_state.metrics.execution_count += 1;
                    new_state.message = "🔄 Counter reset to zero!".to_string();
                    new_state
                });
            }
        });

        // Theme change callback - depends on current theme
        let theme_callback = use_callback(
            {
                let set_state = set_app_state.clone();
                move |_: ()| {
                    set_state.update(|state| {
                        let mut new_state = state.clone();
                        new_state.theme = match new_state.theme {
                            Theme::Electric => Theme::Neon,
                            Theme::Neon => Theme::Cyber,
                            Theme::Cyber => Theme::Electric,
                        };
                        new_state.metrics.dependency_changes += 1;
                        new_state.message =
                            format!("🎨 Theme changed to {}", new_state.theme.name());
                        new_state
                    });
                }
            },
            format!("{:?}", state.theme), // Use string representation for dependency
        );

        // Handle keyboard input
        if let Some(Event::Key(key)) = use_event()
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => request_exit(),
                KeyCode::Char('h') => set_show_help.update(|h| !h),
                KeyCode::Char('+') | KeyCode::Up => increment_callback.emit(1),
                KeyCode::Char('-') | KeyCode::Down => increment_callback.emit(-1),
                KeyCode::Char('r') => reset_callback.emit(()),
                KeyCode::Char('t') => theme_callback.emit(()),
                KeyCode::Char('m') => {
                    set_app_state.update(|state| {
                        let mut new_state = state.clone();
                        new_state.multiplier = if new_state.multiplier >= 5 {
                            1
                        } else {
                            new_state.multiplier + 1
                        };
                        new_state.metrics.dependency_changes += 1;
                        new_state.message = format!(" Multiplier set to {}", new_state.multiplier);
                        new_state
                    });
                }
                KeyCode::Char('s') => {
                    set_app_state.update(|state| {
                        let mut new_state = state.clone();
                        new_state.step_size = if new_state.step_size >= 10 {
                            1
                        } else {
                            new_state.step_size + 1
                        };
                        new_state.metrics.dependency_changes += 1;
                        new_state.message = format!(" Step size set to {}", new_state.step_size);
                        new_state
                    });
                }
                _ => {}
            }
        }

        if show_help.get() {
            render_help_overlay(area, frame, &state.theme);
            return;
        }

        render_main_ui(area, frame, &state);
    }
}

fn render_main_ui(area: Rect, frame: &mut Frame, state: &AppState) {
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(4), // Footer
        ])
        .split(area);

    // Header with dynamic theme colors
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled("🔥 ", Style::default().fg(state.theme.accent_color())),
        Span::styled(
            "Callback Performance Showcase",
            Style::default()
                .fg(state.theme.primary_color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 🔥", Style::default().fg(state.theme.accent_color())),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(state.theme.primary_color())),
    )
    .alignment(Alignment::Center);

    frame.render_widget(header, chunks[0]);

    // Content area - split into counter and metrics
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Counter display
    render_counter_panel(content_chunks[0], frame, state);

    // Metrics and callback info
    render_metrics_panel(content_chunks[1], frame, state);

    // Footer with controls
    render_footer(chunks[2], frame, state);
}

fn render_counter_panel(area: Rect, frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Counter display
            Constraint::Length(3), // Current settings
            Constraint::Min(0),    // Message area
        ])
        .split(area);

    // Large counter display
    let counter_text = format!("{}", state.counter);
    let counter_display = Paragraph::new(counter_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.theme.primary_color()))
                .title(" 🔢 Counter "),
        )
        .style(
            Style::default()
                .fg(state.theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    frame.render_widget(counter_display, chunks[0]);

    // Current settings
    let settings_text = vec![Line::from(vec![
        Span::styled("Multiplier: ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.multiplier.to_string(),
            Style::default().fg(state.theme.primary_color()),
        ),
        Span::styled(" | Step: ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.step_size.to_string(),
            Style::default().fg(state.theme.primary_color()),
        ),
        Span::styled(" | Theme: ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.theme.name(),
            Style::default().fg(state.theme.accent_color()),
        ),
    ])];

    let settings = Paragraph::new(settings_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.theme.primary_color()))
                .title(" ⚙️  Settings "),
        )
        .alignment(Alignment::Center);

    frame.render_widget(settings, chunks[1]);

    // Message area
    let message = Paragraph::new(state.message.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.theme.primary_color()))
                .title(" 💬 Status "),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    frame.render_widget(message, chunks[2]);
}

fn render_metrics_panel(area: Rect, frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Callback metrics
            Constraint::Min(0),    // Performance info
        ])
        .split(area);

    // Callback metrics
    let metrics_text = vec![
        Line::from(vec![
            Span::styled(
                "📊 Callback Executions: ",
                Style::default().fg(state.theme.accent_color()),
            ),
            Span::styled(
                state.metrics.execution_count.to_string(),
                Style::default().fg(state.theme.primary_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "🔄 Dependency Changes: ",
                Style::default().fg(state.theme.accent_color()),
            ),
            Span::styled(
                state.metrics.dependency_changes.to_string(),
                Style::default().fg(state.theme.primary_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "⏱️  Last Execution: ",
                Style::default().fg(state.theme.accent_color()),
            ),
            Span::styled(
                state
                    .metrics
                    .last_execution
                    .map(|t| format!("{:.2}s ago", t.elapsed().as_secs_f64()))
                    .unwrap_or_else(|| "Never".to_string()),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "🧠 Memoization Benefits:",
            Style::default()
                .fg(state.theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("• Callbacks recreate only on dependency changes"),
        Line::from("• Improved performance with complex operations"),
        Line::from("• Prevents unnecessary re-renders"),
    ];

    let metrics = Paragraph::new(metrics_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.theme.primary_color()))
                .title(" 📈 Callback Metrics "),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(metrics, chunks[0]);

    // Performance explanation
    let perf_text = vec![
        Line::from(vec![Span::styled(
            "🔍 Hook Demonstrations:",
            Style::default()
                .fg(state.theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("🔹 use_callback: Memoizes increment function"),
        Line::from("   Dependencies: [multiplier, step_size]"),
        Line::from(""),
        Line::from("🔹 use_callback_once: Memoizes reset function"),
        Line::from("   Dependencies: [] (created once)"),
        Line::from(""),
        Line::from("🔹 use_event_handler: Theme change handler"),
        Line::from("   Dependencies: [current_theme]"),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 Tip: ", Style::default().fg(state.theme.accent_color())),
            Span::styled(
                "Change multiplier/step to see callback recreation!",
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    let perf_info = Paragraph::new(perf_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.theme.primary_color()))
                .title(" 🎯 Performance Info "),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(perf_info, chunks[1]);
}

fn render_footer(area: Rect, frame: &mut Frame, state: &AppState) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled(
                "Controls: ",
                Style::default()
                    .fg(state.theme.accent_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("↑↓/+- ", Style::default().fg(state.theme.primary_color())),
            Span::styled("Counter | ", Style::default().fg(Color::Gray)),
            Span::styled("m ", Style::default().fg(state.theme.primary_color())),
            Span::styled("Multiplier | ", Style::default().fg(Color::Gray)),
            Span::styled("s ", Style::default().fg(state.theme.primary_color())),
            Span::styled("Step | ", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("r ", Style::default().fg(state.theme.primary_color())),
            Span::styled("Reset | ", Style::default().fg(Color::Gray)),
            Span::styled("t ", Style::default().fg(state.theme.primary_color())),
            Span::styled("Theme | ", Style::default().fg(Color::Gray)),
            Span::styled("h ", Style::default().fg(state.theme.primary_color())),
            Span::styled("Help | ", Style::default().fg(Color::Gray)),
            Span::styled("q ", Style::default().fg(state.theme.primary_color())),
            Span::styled("Quit", Style::default().fg(Color::Gray)),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.theme.primary_color())),
        )
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

fn render_help_overlay(area: Rect, frame: &mut Frame, theme: &Theme) {
    let popup_area = centered_rect(70, 80, area);
    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "🔥 Callback Showcase Help",
            Style::default()
                .fg(theme.primary_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "🎮 Controls:",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/+ - Increment counter"),
        Line::from("  ↓/- - Decrement counter"),
        Line::from("  r   - Reset counter to zero"),
        Line::from("  m   - Change multiplier (1-5)"),
        Line::from("  s   - Change step size (1-10)"),
        Line::from("  t   - Cycle through themes"),
        Line::from("  h   - Toggle this help"),
        Line::from("  q   - Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "🧠 Callback Concepts:",
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("• use_callback: Memoizes functions with dependencies"),
        Line::from("• use_callback_once: Creates callback without dependencies"),
        Line::from("• use_event_handler: Specialized for event handling"),
        Line::from(""),
        Line::from("🔍 Watch the metrics panel to see callback behavior!"),
        Line::from("Changing multiplier/step recreates the increment callback."),
        Line::from("The reset callback is created only once."),
        Line::from(""),
        Line::from("Press 'h' again to close this help."),
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
    pulse::render_async(|| async { CallbackShowcase }).await?;
    Ok(())
}
