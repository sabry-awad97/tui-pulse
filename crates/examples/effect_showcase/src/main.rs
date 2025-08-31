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
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pulse::render_async(|| async { App }).await
}

#[derive(Clone)]
struct App;

impl Component for App {
    fn on_mount(&self) {
        // Set up a global keyboard event handler
        on_global_event(KeyCode::Char('q'), || {
            request_exit();
            false
        });
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        HeaderComponent.render(main_chunks[0], frame);

        // Content layout - 2x2 grid
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

        // Render components that showcase different use_effect patterns
        RealTimeClockComponent.render(left_chunks[0], frame);
        DataFetcherComponent.render(left_chunks[1], frame);
        AnimatedProgressComponent.render(right_chunks[0], frame);
        SystemMonitorComponent.render(right_chunks[1], frame);

        // Render footer
        FooterComponent.render(main_chunks[2], frame);
    }
}

#[derive(Clone)]
struct HeaderComponent;

impl Component for HeaderComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let title = Paragraph::new("✨ use_effect Showcase - Real-time TUI Components")
            .style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_set(border::ROUNDED),
            );
        frame.render_widget(title, area);
    }
}

#[derive(Clone)]
struct RealTimeClockComponent;

impl Component for RealTimeClockComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (current_time, set_time) = use_state(Local::now);

        // Effect that updates time every second
        use_effect(
            {
                move || {
                    let mut interval = interval(Duration::from_secs(1));

                    tokio::spawn(async move {
                        loop {
                            interval.tick().await;
                            set_time.set(Local::now());
                        }
                    });

                    // Cleanup function
                    Some(|| {
                        // In a real implementation, we'd cancel the timer here
                        tracing::info!("🕐 Clock timer cleanup");
                    })
                }
            },
            (), // Run once on mount
        );

        let time_str = current_time.get().format("%H:%M:%S").to_string();
        let date_str = current_time.get().format("%Y-%m-%d").to_string();

        let time_text = Text::from(vec![
            Line::from(vec![
                Span::styled("🕐 ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    &time_str,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📅 ", Style::default().fg(Color::Green)),
                Span::styled(&date_str, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "⏰ Updates every second",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )]),
        ]);

        let paragraph = Paragraph::new(time_text)
            .block(
                Block::default()
                    .title("🕐 Real-time Clock")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

#[derive(Clone)]
struct AnimatedProgressComponent;

impl Component for AnimatedProgressComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (progress, set_progress) = use_state(|| 0u16);
        let (direction, set_direction) = use_state(|| 1i16); // 1 for forward, -1 for backward
        let (animation_frame, set_frame) = use_state(|| 0u8);

        // Effect for animated progress bar
        use_effect(
            {
                move || {
                    let mut interval = interval(Duration::from_millis(100));

                    tokio::spawn(async move {
                        let mut current_progress = 0u16;
                        let mut current_direction = 1i16;
                        let mut frame_count = 0u8;

                        loop {
                            interval.tick().await;

                            // Update progress
                            if current_direction == 1 {
                                current_progress = (current_progress + 2).min(100);
                                if current_progress >= 100 {
                                    current_direction = -1;
                                }
                            } else {
                                current_progress = current_progress.saturating_sub(2);
                                if current_progress == 0 {
                                    current_direction = 1;
                                }
                            }

                            frame_count = (frame_count + 1) % 4;

                            set_progress.set(current_progress);
                            set_direction.set(current_direction);
                            set_frame.set(frame_count);
                        }
                    });

                    Some(|| {
                        tracing::info!("🎬 Animation cleanup");
                    })
                }
            },
            (), // Run once on mount
        );

        let animation_chars = ["⠋", "⠙", "⠹", "⠸"];
        let spinner = animation_chars[animation_frame.get() as usize];

        let direction_arrow = if direction.get() == 1 { "→" } else { "←" };

        let block = Block::default()
            .title("🎬 Animated Progress")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_set(border::ROUNDED);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Progress bar
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Status
                Constraint::Min(1),    // Animation info
            ])
            .split(inner);

        // Animated progress bar
        let progress_gauge = Gauge::default()
            .block(Block::default().title(format!("{} Progress {}", direction_arrow, spinner)))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(progress.get())
            .label(format!("{}%", progress.get()));
        frame.render_widget(progress_gauge, chunks[0]);

        // Status text
        let status = format!(
            "Direction: {} | Frame: {}",
            if direction.get() == 1 {
                "Forward"
            } else {
                "Backward"
            },
            animation_frame.get()
        );
        let status_paragraph = Paragraph::new(status)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(status_paragraph, chunks[2]);

        // Animation info
        let info_text = Text::from(vec![
            Line::from(vec![Span::styled(
                "⚡ Updates every 100ms",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )]),
            Line::from(vec![Span::styled(
                "🔄 Auto-reversing animation",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )]),
        ]);
        let info_paragraph = Paragraph::new(info_text).alignment(Alignment::Center);
        frame.render_widget(info_paragraph, chunks[3]);
    }
}

#[derive(Clone)]
struct SystemMonitorComponent;

impl Component for SystemMonitorComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (cpu_data, set_cpu) = use_state(VecDeque::<u64>::new);
        let (memory_usage, set_memory) = use_state(|| 45u16);
        let (network_activity, set_network) = use_state(VecDeque::<u64>::new);
        let (last_update, set_update_time) = use_state(Local::now);

        // Effect for system monitoring with multiple data sources
        use_effect_once(move || {
            let set_cpu = set_cpu.clone();
            let set_memory = set_memory.clone();
            let set_network = set_network.clone();
            let set_update_time = set_update_time.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut interval = interval(Duration::from_millis(500));
                    let mut rng = rand::rng();

                    loop {
                        interval.tick().await;

                        // Generate mock CPU data
                        set_cpu.update(|current| {
                            let mut new_data = current.clone();
                            new_data.push_back(rng.random_range(20..80));
                            if new_data.len() > 20 {
                                new_data.pop_front();
                            }
                            new_data
                        });

                        // Generate mock memory usage
                        set_memory.update(|current| {
                            let change = rng.random_range(-5i16..5);
                            (*current as i16 + change).clamp(30, 90) as u16
                        });

                        // Generate mock network activity
                        set_network.update(|current| {
                            let mut new_data = current.clone();
                            new_data.push_back(rng.random_range(0..100));
                            if new_data.len() > 15 {
                                new_data.pop_front();
                            }
                            new_data
                        });

                        set_update_time.set(Local::now());
                    }
                });
            });

            // Return cleanup function
            move || {
                tracing::info!("🖥️  System monitor cleanup");
            }
        });

        let block = Block::default()
            .title("🖥️  System Monitor")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .border_set(border::ROUNDED);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // CPU sparkline
                Constraint::Length(2), // Memory gauge
                Constraint::Length(3), // Network sparkline
                Constraint::Min(1),    // Last update
            ])
            .split(inner);

        // CPU usage sparkline
        let cpu_values: Vec<u64> = cpu_data.get().iter().cloned().collect();
        if !cpu_values.is_empty() {
            let cpu_sparkline = Sparkline::default()
                .block(Block::default().title("CPU Usage"))
                .data(&cpu_values)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(cpu_sparkline, chunks[0]);
        }

        // Memory usage gauge
        let memory_gauge = Gauge::default()
            .block(Block::default().title("Memory"))
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(memory_usage.get())
            .label(format!("{}%", memory_usage.get()));
        frame.render_widget(memory_gauge, chunks[1]);

        // Network activity sparkline
        let network_values: Vec<u64> = network_activity.get().iter().cloned().collect();
        if !network_values.is_empty() {
            let network_sparkline = Sparkline::default()
                .block(Block::default().title("Network"))
                .data(&network_values)
                .style(Style::default().fg(Color::Green));
            frame.render_widget(network_sparkline, chunks[2]);
        }

        // Last update time
        let update_text = format!("Last update: {}", last_update.get().format("%H:%M:%S%.3f"));
        let update_paragraph = Paragraph::new(update_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(update_paragraph, chunks[3]);
    }
}

#[derive(Clone)]
struct DataFetcherComponent;

impl Component for DataFetcherComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (data, set_data) = use_state(|| "Loading...".to_string());
        let (fetch_count, set_fetch_count) = use_state(|| 0);
        let (is_loading, set_is_loading) = use_state(|| true);

        // Effect that simulates data fetching every 3 seconds
        use_effect(
            {
                let set_data = set_data.clone();
                let set_fetch_count = set_fetch_count.clone();
                let set_is_loading = set_is_loading.clone();

                move || {
                    let mut interval = interval(Duration::from_secs(3));

                    tokio::spawn(async move {
                        let mut count = 0;
                        loop {
                            interval.tick().await;
                            count += 1;

                            set_is_loading.call(true);

                            // Simulate network delay
                            tokio::time::sleep(Duration::from_millis(500)).await;

                            let mock_data = [
                                "🚀 Rocket launch successful",
                                "🌟 New star discovered",
                                "🔬 Scientific breakthrough",
                                "🎯 Mission accomplished",
                                "💡 Innovation detected",
                                "🌍 Global update received",
                            ];

                            let random_data = mock_data[count % mock_data.len()];
                            set_data.call(random_data.to_string());
                            set_fetch_count.call(count);
                            set_is_loading.call(false);
                        }
                    });

                    Some(|| {
                        tracing::info!("📡 Data fetcher cleanup");
                    })
                }
            },
            (), // Run once on mount
        );

        let status_text = if is_loading.get() {
            "🔄 Fetching..."
        } else {
            "✅ Data ready"
        };

        let content = Text::from(vec![
            Line::from(vec![
                Span::styled("📡 ", Style::default().fg(Color::Blue)),
                Span::styled(
                    "Live Data Feed",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                data.get(),
                Style::default().fg(Color::Green),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("📊 Fetch #{}", fetch_count.get()),
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![Span::styled(
                status_text,
                Style::default().fg(if is_loading.get() {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            )]),
        ]);

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("📡 Data Fetcher")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

#[derive(Clone)]
struct FooterComponent;

impl Component for FooterComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let help_text =
            "🎯 All components use use_effect for real-time updates | Press 'q' to quit";
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
