use chrono::Local;
use pulse::{crossterm::event::KeyCode, prelude::*};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};
use std::{collections::VecDeque, time::Duration};

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
        // Beautiful gradient background
        let background = Block::default().style(Style::default().bg(Color::Rgb(10, 15, 30)));
        frame.render_widget(background, area);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Min(8),    // Content grid
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header with animated title
        HeaderComponent.render(main_chunks[0], frame);

        // 2x2 grid layout for components
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

        // Render components
        AnimatedClockComponent.render(left_chunks[0], frame);
        ProgressBarComponent.render(left_chunks[1], frame);
        DataStreamComponent.render(right_chunks[0], frame);
        SystemMonitorComponent.render(right_chunks[1], frame);

        // Footer
        FooterComponent.render(main_chunks[2], frame);
    }
}

#[derive(Clone)]
struct HeaderComponent;

impl Component for HeaderComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (animation_frame, set_animation_frame) = use_state(|| 0u8);

        // Animate title every 200ms using use_interval
        use_interval(
            {
                let set_animation_frame = set_animation_frame.clone();
                move || {
                    set_animation_frame.update(|frame| (frame + 1) % 8);
                }
            },
            Duration::from_millis(200),
        );

        let sparkles = ["✨", "⭐", "🌟", "💫", "✨", "⭐", "🌟", "💫"];
        let sparkle = sparkles[animation_frame.get() as usize];

        let title_text = Text::from(vec![
            Line::from(vec![
                Span::styled(sparkle, Style::default().fg(Color::Yellow)),
                Span::styled(
                    " PULSE INTERVAL SHOWCASE ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(sparkle, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::styled(
                "Beautiful real-time animations powered by use_interval hooks",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )),
        ]);

        let header = Paragraph::new(title_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(header, area);
    }
}

#[derive(Clone)]
struct AnimatedClockComponent;

impl Component for AnimatedClockComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (current_time, set_current_time) = use_state(Local::now);
        let (tick_animation, set_tick_animation) = use_state(|| 0u8);

        // Update time every second using use_interval
        use_interval(
            {
                let set_current_time = set_current_time.clone();
                let set_tick_animation = set_tick_animation.clone();
                move || {
                    set_current_time.set(Local::now());
                    set_tick_animation.update(|tick| (tick + 1) % 4);
                }
            },
            Duration::from_secs(1),
        );

        let tick_chars = ["🕐", "🕑", "🕒", "🕓"];
        let tick_char = tick_chars[tick_animation.get() as usize];

        let time_str = current_time.get().format("%H:%M:%S").to_string();
        let date_str = current_time.get().format("%Y-%m-%d").to_string();

        let content = Text::from(vec![
            Line::from(vec![
                Span::styled(tick_char, Style::default().fg(Color::Yellow)),
                Span::styled(
                    " LIVE CLOCK",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("⏰ ", Style::default().fg(Color::Blue)),
                Span::styled(
                    &time_str,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("📅 ", Style::default().fg(Color::Blue)),
                Span::styled(&date_str, Style::default().fg(Color::Cyan)),
            ]),
        ]);

        let clock = Paragraph::new(content)
            .block(
                Block::default()
                    .title("🕒 Animated Clock")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(clock, area);
    }
}

#[derive(Clone)]
struct ProgressBarComponent;

impl Component for ProgressBarComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (progress, set_progress) = use_state(|| 0u16);
        let (direction, set_direction) = use_state(|| 1i8);

        // Animate progress bar every 100ms using use_interval
        use_interval(
            {
                let set_progress = set_progress.clone();
                let set_direction = set_direction.clone();
                move || {
                    set_progress.update(|current| {
                        let dir = direction.get();
                        let new_progress = (*current as i16 + dir as i16).clamp(0, 100) as u16;

                        if new_progress == 0 || new_progress == 100 {
                            set_direction.update(|d| -d);
                        }

                        new_progress
                    });
                }
            },
            Duration::from_millis(100),
        );

        let progress_value = progress.get();
        let color = match progress_value {
            0..=33 => Color::Red,
            34..=66 => Color::Yellow,
            _ => Color::Green,
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title("📊 Animated Progress")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_set(border::ROUNDED),
            )
            .gauge_style(Style::default().fg(color))
            .percent(progress_value)
            .label(format!("{}%", progress_value));

        frame.render_widget(gauge, area);
    }
}

#[derive(Clone)]
struct DataStreamComponent;

impl Component for DataStreamComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (data_points, set_data_points) = use_state(VecDeque::<u64>::new);

        // Generate data every 300ms using use_interval
        use_interval(
            {
                let set_data_points = set_data_points.clone();
                move || {
                    let mut rng = rand::rng();
                    set_data_points.update(|current| {
                        let mut new_data = current.clone();
                        new_data.push_back(rng.random_range(20..80));
                        if new_data.len() > 15 {
                            new_data.pop_front();
                        }
                        new_data
                    });
                }
            },
            Duration::from_millis(300),
        );

        let data: Vec<u64> = data_points.get().iter().cloned().collect();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title("📈 Live Data Stream")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_set(border::ROUNDED),
            )
            .data(&data)
            .style(Style::default().fg(Color::Green));

        frame.render_widget(sparkline, area);
    }
}

#[derive(Clone)]
struct SystemMonitorComponent;

impl Component for SystemMonitorComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (cpu_usage, set_cpu_usage) = use_state(|| 45u16);
        let (memory_usage, set_memory_usage) = use_state(|| 60u16);
        let (status_icon, set_status_icon) = use_state(|| 0u8);

        // Update system stats every 500ms using use_interval
        use_interval(
            {
                let set_cpu_usage = set_cpu_usage.clone();
                let set_memory_usage = set_memory_usage.clone();
                let set_status_icon = set_status_icon.clone();
                move || {
                    let mut rng = rand::rng();

                    set_cpu_usage.update(|current| {
                        let change = rng.random_range(-5i16..5);
                        ((*current as i16 + change).clamp(20, 90)) as u16
                    });

                    set_memory_usage.update(|current| {
                        let change = rng.random_range(-3i16..3);
                        ((*current as i16 + change).clamp(30, 85)) as u16
                    });

                    set_status_icon.update(|icon| (icon + 1) % 4);
                }
            },
            Duration::from_millis(500),
        );

        let status_icons = ["🟢", "🟡", "🔴", "🟠"];
        let status_icon_char = status_icons[status_icon.get() as usize];

        let content = Text::from(vec![
            Line::from(vec![
                Span::styled(status_icon_char, Style::default()),
                Span::styled(
                    " SYSTEM MONITOR",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("CPU: ", Style::default().fg(Color::Blue)),
                Span::styled(
                    format!("{}%", cpu_usage.get()),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("MEM: ", Style::default().fg(Color::Blue)),
                Span::styled(
                    format!("{}%", memory_usage.get()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]);

        let monitor = Paragraph::new(content)
            .block(
                Block::default()
                    .title("🖥️  System Stats")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(monitor, area);
    }
}

#[derive(Clone)]
struct FooterComponent;

impl Component for FooterComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let footer_text = Text::from(vec![Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "'q'",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to quit • All animations powered by ",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                "use_interval",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" hooks", Style::default().fg(Color::Gray)),
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
