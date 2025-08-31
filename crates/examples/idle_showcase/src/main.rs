use chrono::Local;
use pulse::{crossterm::event::KeyCode, prelude::*};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph},
};

#[derive(Clone)]
struct IdleState {
    is_idle_short: bool,
    is_idle_medium: bool,
    is_idle_long: bool,
    activity_count: u32,
    last_activity: chrono::DateTime<chrono::Local>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pulse::render_async(|| async { App }).await
}

#[derive(Clone)]
struct App;

impl Component for App {
    fn on_mount(&self) {
        on_global_event(KeyCode::Char('q'), || {
            request_exit();
            false
        });
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let background = Block::default().style(Style::default().bg(Color::Rgb(10, 15, 25)));
        frame.render_widget(background, area);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        HeaderComponent.render(main_chunks[0], frame);
        IdleDetectionComponent.render(main_chunks[1], frame);
        FooterComponent.render(main_chunks[2], frame);
    }
}

#[derive(Clone)]
struct HeaderComponent;

impl Component for HeaderComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let title_text = Text::from(vec![
            Line::from(vec![
                Span::styled("💤", Style::default().fg(Color::Cyan)),
                Span::styled(
                    " IDLE DETECTION SHOWCASE ",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("💤", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(Span::styled(
                "Beautiful idle state management with use_idle hook",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )),
        ]);

        let header = Paragraph::new(title_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(header, area);
    }
}

#[derive(Clone)]
struct IdleDetectionComponent;

impl Component for IdleDetectionComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // Multiple idle timeouts for different effects
        let is_idle_short = use_idle(3000); // 3 seconds
        let is_idle_medium = use_idle(8000); // 8 seconds  
        let is_idle_long = use_idle(15000); // 15 seconds

        // Activity counter
        let (activity_count, set_activity_count) = use_state(|| 0u32);
        let (last_activity_time, set_last_activity_time) = use_state(Local::now);

        // Update activity counter when user becomes active
        use_effect(
            {
                let set_activity_count = set_activity_count.clone();
                let set_last_activity_time = set_last_activity_time.clone();
                move || {
                    if !is_idle_short {
                        set_activity_count.update(|count| count + 1);
                        set_last_activity_time.set(Local::now());
                    }
                    None::<fn()>
                }
            },
            is_idle_short,
        );

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Main idle display
        if is_idle_long {
            self.render_screensaver(chunks[0], frame);
        } else if is_idle_medium {
            self.render_idle_warning(chunks[0], frame);
        } else if is_idle_short {
            self.render_getting_idle(chunks[0], frame);
        } else {
            self.render_active_state(chunks[0], frame);
        }

        // Stats sidebar
        let idle_state = IdleState {
            is_idle_short,
            is_idle_medium,
            is_idle_long,
            activity_count: activity_count.get(),
            last_activity: last_activity_time.get(),
        };
        self.render_stats(chunks[1], frame, idle_state);
    }
}

impl IdleDetectionComponent {
    fn render_active_state(&self, area: Rect, frame: &mut Frame) {
        let content = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("✨ ", Style::default().fg(Color::Green)),
                Span::styled(
                    "ACTIVE STATE",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from("🎯 You are actively using the application"),
            Line::from("⌨️  Keep typing or moving to stay active"),
            Line::from("🔄 Activity is being tracked in real-time"),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "💡 Tip: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Stop interacting to see idle detection in action!",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]);

        let block = Paragraph::new(content)
            .block(
                Block::default()
                    .title("🟢 Active Mode")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(block, area);
    }

    fn render_getting_idle(&self, area: Rect, frame: &mut Frame) {
        let content = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "GETTING IDLE",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from("😴 No activity detected for 3+ seconds"),
            Line::from("⚠️  Entering idle detection mode"),
            Line::from("🕐 Continue waiting to see more idle states"),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Press any key",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " to return to active state",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]);

        let block = Paragraph::new(content)
            .block(
                Block::default()
                    .title("🟡 Short Idle (3s+)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(block, area);
    }

    fn render_idle_warning(&self, area: Rect, frame: &mut Frame) {
        let content = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("⚠️ ", Style::default().fg(Color::Red)),
                Span::styled(
                    "IDLE WARNING",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from("💤 User has been idle for 8+ seconds"),
            Line::from("🔔 This could trigger notifications"),
            Line::from("💾 Auto-save might be triggered"),
            Line::from("🔒 Security timeout warnings could appear"),
            Line::from(""),
            Line::from(vec![
                Span::styled("⌨️ ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "Move or type to prevent screensaver",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]);

        let block = Paragraph::new(content)
            .block(
                Block::default()
                    .title("🟠 Medium Idle (8s+)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(block, area);
    }

    fn render_screensaver(&self, area: Rect, frame: &mut Frame) {
        // Generate floating elements for screensaver effect using time-based animation
        let time_seed = Local::now().timestamp_millis() as u64;

        let stars: Vec<(u16, u16, char, Color)> = (0..20)
            .map(|i| {
                let x = ((time_seed + i * 1337) % (area.width as u64)) as u16;
                let y = ((time_seed + i * 2749) % (area.height as u64)) as u16;
                let chars = ['✦', '✧', '⋆', '✩', '✪', '✫', '✬', '✭', '✮', '✯'];
                let colors = [
                    Color::Cyan,
                    Color::Magenta,
                    Color::Yellow,
                    Color::Blue,
                    Color::Green,
                ];
                let char = chars[(i as usize) % chars.len()];
                let color = colors[(i as usize) % colors.len()];
                (x, y, char, color)
            })
            .collect();

        let mut lines = vec![Line::from("")];

        // Create animated text
        for y in 0..area.height.saturating_sub(4) {
            let mut spans = Vec::new();
            for x in 0..area.width.saturating_sub(2) {
                if let Some((_, _, char, color)) =
                    stars.iter().find(|(sx, sy, _, _)| *sx == x && *sy == y)
                {
                    spans.push(Span::styled(char.to_string(), Style::default().fg(*color)));
                } else {
                    spans.push(Span::styled(" ", Style::default()));
                }
            }
            lines.push(Line::from(spans));
        }

        // Add screensaver message
        lines.extend(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("🌟 ", Style::default().fg(Color::Magenta)),
                Span::styled(
                    "SCREENSAVER MODE",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" 🌟", Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from("💤 User has been idle for 15+ seconds"),
            Line::from("✨ Enjoying the beautiful star field"),
            Line::from("🎨 This demonstrates advanced idle detection"),
        ]);

        let screensaver = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("🔴 Deep Idle (15s+) - Screensaver Active")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(screensaver, area);
    }

    fn render_stats(&self, area: Rect, frame: &mut Frame, idle_state: IdleState) {
        let status_icon = if idle_state.is_idle_long {
            "🔴"
        } else if idle_state.is_idle_medium {
            "🟠"
        } else if idle_state.is_idle_short {
            "🟡"
        } else {
            "🟢"
        };

        let status_text = if idle_state.is_idle_long {
            "Deep Idle (Screensaver)"
        } else if idle_state.is_idle_medium {
            "Medium Idle (Warning)"
        } else if idle_state.is_idle_short {
            "Short Idle (Detected)"
        } else {
            "Active"
        };

        let stats_text = Text::from(vec![
            Line::from(vec![
                Span::styled("📊 ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "IDLE MONITOR",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::White)),
                Span::styled(status_icon, Style::default()),
                Span::styled(
                    format!(" {}", status_text),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Activity Count: ", Style::default().fg(Color::White)),
                Span::styled(
                    idle_state.activity_count.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Last Activity:",
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                idle_state.last_activity.format("%H:%M:%S").to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("⏱️ ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "TIMEOUTS",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(format!(
                "Short:  {}  3s",
                if idle_state.is_idle_short {
                    "🟡"
                } else {
                    "⚪"
                }
            )),
            Line::from(format!(
                "Medium: {}  8s",
                if idle_state.is_idle_medium {
                    "🟠"
                } else {
                    "⚪"
                }
            )),
            Line::from(format!(
                "Long:   {} 15s",
                if idle_state.is_idle_long {
                    "🔴"
                } else {
                    "⚪"
                }
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("🎮 ", Style::default().fg(Color::Green)),
                Span::styled(
                    "CONTROLS",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from("Any key - Reset timer"),
            Line::from("Mouse - Reset timer"),
            Line::from("Q - Quit application"),
        ]);

        let stats = Paragraph::new(stats_text).block(
            Block::default()
                .title("📈 Dashboard")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .border_set(border::ROUNDED),
        );

        frame.render_widget(stats, area);
    }
}

#[derive(Clone)]
struct FooterComponent;

impl Component for FooterComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let footer_text = Text::from(vec![Line::from(vec![
            Span::styled("🚀 Powered by ", Style::default().fg(Color::Gray)),
            Span::styled(
                "use_idle",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " hook • Stop interacting to see ",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                "idle detection",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" in action!", Style::default().fg(Color::Gray)),
        ])]);

        let footer = Paragraph::new(footer_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(footer, area);
    }
}

// Enhanced idle component with progress bars and animations
#[derive(Clone)]
#[allow(dead_code)]
struct IdleProgressComponent;

impl Component for IdleProgressComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let is_idle_3s = use_idle(3000);
        let is_idle_8s = use_idle(8000);
        let is_idle_15s = use_idle(15000);

        // Simulate progress towards idle state (this would need timing integration)
        let progress_3s = if is_idle_3s { 100 } else { 0 };
        let progress_8s = if is_idle_8s { 100 } else { 0 };
        let progress_15s = if is_idle_15s { 100 } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        // Progress bars for each timeout level
        let gauge_3s = Gauge::default()
            .block(
                Block::default()
                    .title("3s Timeout")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(progress_3s)
            .label(format!("{}%", progress_3s));

        let gauge_8s = Gauge::default()
            .block(
                Block::default()
                    .title("8s Timeout")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .gauge_style(Style::default().fg(Color::Red))
            .percent(progress_8s)
            .label(format!("{}%", progress_8s));

        let gauge_15s = Gauge::default()
            .block(
                Block::default()
                    .title("15s Timeout")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(progress_15s)
            .label(format!("{}%", progress_15s));

        frame.render_widget(gauge_3s, chunks[0]);
        frame.render_widget(gauge_8s, chunks[1]);
        frame.render_widget(gauge_15s, chunks[2]);
    }
}
