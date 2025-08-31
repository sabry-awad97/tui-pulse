use chrono::Local;
use pulse::{crossterm::event::KeyCode, prelude::*};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::time::Duration;

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
        on_global_event(KeyCode::Esc, || {
            request_exit();
            false
        });
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        // Beautiful gradient background
        let background = Block::default().style(Style::default().bg(Color::Rgb(5, 10, 25)));
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

        // Header
        HeaderComponent.render(main_chunks[0], frame);

        // 2x2 grid layout for future examples
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

        // Render future showcase components
        DataFetcherComponent.render(left_chunks[0], frame);
        ProgressDownloadComponent.render(left_chunks[1], frame);
        WeatherApiComponent.render(right_chunks[0], frame);
        FileProcessorComponent.render(right_chunks[1], frame);

        // Footer
        FooterComponent.render(main_chunks[2], frame);
    }
}

#[derive(Clone)]
struct HeaderComponent;

impl Component for HeaderComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let title_text = Text::from(vec![
            Line::from(vec![
                Span::styled("🚀", Style::default().fg(Color::Yellow)),
                Span::styled(
                    " PULSE FUTURE SHOWCASE ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("🚀", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::styled(
                "Async operations with progress tracking and state management",
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
struct DataFetcherComponent;

impl Component for DataFetcherComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (refresh_trigger, set_refresh_trigger) = use_state(|| 0u32);

        // Simulate data fetching with random delay
        let data_future = use_future::<u32, _, _, _, _>(
            {
                let current_trigger = refresh_trigger.get();
                move || async move {
                    // Simulate network delay
                    let delay = rand::rng().random_range(500..2000);
                    tokio::time::sleep(Duration::from_millis(delay)).await;

                    let data = [
                        "📊 Sales Report",
                        "📈 Analytics Dashboard",
                        "🎯 Performance Metrics",
                        "💰 Revenue Summary",
                        "📋 User Statistics",
                    ];

                    let random_data = data[rand::rng().random_range(0..data.len())];
                    Ok::<String, String>(format!(
                        "{} (#{}) - {}",
                        random_data,
                        current_trigger,
                        Local::now().format("%H:%M:%S")
                    ))
                }
            },
            Some(refresh_trigger.get()),
        );

        // Auto-refresh every 5 seconds
        use_interval(
            {
                let set_refresh_trigger = set_refresh_trigger.clone();
                move || {
                    set_refresh_trigger.update(|trigger| trigger + 1);
                }
            },
            Duration::from_secs(5),
        );

        let (content, color) = match data_future.state() {
            FutureState::Pending => ("🔄 Fetching data...".to_string(), Color::Yellow),
            FutureState::Progress(_) => ("⚡ Processing...".to_string(), Color::Blue),
            FutureState::Resolved(data) => (format!("✅ {}", data), Color::Green),
            FutureState::Error(err) => (format!("❌ Error: {}", err), Color::Red),
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("📡 Data Fetcher")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))
                    .border_set(border::ROUNDED),
            )
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

#[derive(Clone)]
struct ProgressDownloadComponent;

impl Component for ProgressDownloadComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (download_trigger, set_download_trigger) = use_state(|| 0u32);

        // Simulate file download with progress tracking
        let download_future = use_future_with_progress::<u32, _, _, _, _>(
            {
                let current_trigger = download_trigger.get();
                move |progress_callback| async move {
                    let total_steps = 20;
                    for i in 0..=total_steps {
                        // Simulate download chunks
                        tokio::time::sleep(Duration::from_millis(150)).await;

                        // Report progress
                        progress_callback(i as f32 / total_steps as f32);
                    }

                    Ok::<String, String>(format!(
                        "Download #{} completed successfully!",
                        current_trigger
                    ))
                }
            },
            Some(download_trigger.get()),
        );

        // Start new download every 8 seconds
        use_interval(
            {
                let set_download_trigger = set_download_trigger.clone();
                move || {
                    set_download_trigger.update(|trigger| trigger + 1);
                }
            },
            Duration::from_secs(8),
        );

        let (content, gauge_percent, color) = match download_future.state() {
            FutureState::Pending => ("🚀 Starting download...".to_string(), 0, Color::Cyan),
            FutureState::Progress(progress) => {
                let percent = (progress * 100.0) as u16;
                (
                    format!("📥 Downloading... {}%", percent),
                    percent,
                    Color::Blue,
                )
            }
            FutureState::Resolved(data) => (format!("✅ {}", data), 100, Color::Green),
            FutureState::Error(err) => (format!("❌ {}", err), 0, Color::Red),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        // Progress gauge
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title("💾 Download Progress")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))
                    .border_set(border::ROUNDED),
            )
            .gauge_style(Style::default().fg(color))
            .percent(gauge_percent);

        frame.render_widget(gauge, chunks[0]);

        // Status text
        let status = Paragraph::new(content)
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(status, chunks[1]);
    }
}

#[derive(Clone)]
struct WeatherApiComponent;

impl Component for WeatherApiComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (location_id, set_location_id) = use_state(|| 1u32);

        // Simulate weather API calls with different locations
        let weather_future = use_future::<u32, _, _, _, _>(
            {
                let current_location = location_id.get();
                move || async move {
                    // Simulate API call delay
                    tokio::time::sleep(Duration::from_millis(800)).await;

                    let locations = [
                        ("🏙️ New York", "22°C, Sunny"),
                        ("🌴 Miami", "28°C, Partly Cloudy"),
                        ("🏔️ Denver", "15°C, Snow"),
                        ("🌊 Seattle", "18°C, Rainy"),
                        ("🏜️ Phoenix", "35°C, Clear"),
                    ];

                    let location_data =
                        locations[(current_location - 1) as usize % locations.len()];
                    Ok::<String, String>(format!("{}: {}", location_data.0, location_data.1))
                }
            },
            Some(location_id.get()),
        );

        // Cycle through locations every 4 seconds
        use_interval(
            {
                let set_location_id = set_location_id.clone();
                move || {
                    set_location_id.update(|id| (id % 5) + 1);
                }
            },
            Duration::from_secs(4),
        );

        let (content, color) = match weather_future.state() {
            FutureState::Pending => ("🌐 Fetching weather...".to_string(), Color::Cyan),
            FutureState::Progress(_) => ("⚡ Processing API...".to_string(), Color::Blue),
            FutureState::Resolved(data) => (data, Color::Green),
            FutureState::Error(err) => (format!("🌩️ API Error: {}", err), Color::Red),
        };

        let weather = Paragraph::new(content)
            .block(
                Block::default()
                    .title("🌤️  Weather API")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))
                    .border_set(border::ROUNDED),
            )
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(weather, area);
    }
}

#[derive(Clone)]
struct FileProcessorComponent;

impl Component for FileProcessorComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let (process_trigger, set_process_trigger) = use_state(|| 0u32);

        // Simulate file processing with detailed progress
        let process_future = use_future_with_progress::<u32, _, _, _, _>(
            {
                let current_trigger = process_trigger.get();
                move |progress_callback| async move {
                    let files = [
                        "config.json",
                        "data.csv",
                        "report.pdf",
                        "image.png",
                        "video.mp4",
                    ];
                    let file_name = files[current_trigger as usize % files.len()];

                    let steps = 15;
                    for i in 0..=steps {
                        // Simulate processing time
                        tokio::time::sleep(Duration::from_millis(200)).await;

                        // Report progress
                        progress_callback(i as f32 / steps as f32);
                    }

                    Ok::<String, String>(format!("📁 {} processed successfully", file_name))
                }
            },
            Some(process_trigger.get()),
        );

        // Start new processing every 6 seconds
        use_interval(
            {
                let set_process_trigger = set_process_trigger.clone();
                move || {
                    set_process_trigger.update(|trigger| trigger + 1);
                }
            },
            Duration::from_secs(6),
        );

        let (content, progress_percent, color) = match process_future.state() {
            FutureState::Pending => ("🔧 Initializing processor...".to_string(), 0, Color::Cyan),
            FutureState::Progress(progress) => {
                let percent = (progress * 100.0) as u16;
                (
                    format!("⚙️ Processing file... {}%", percent),
                    percent,
                    Color::Magenta,
                )
            }
            FutureState::Resolved(data) => (data, 100, Color::Green),
            FutureState::Error(err) => (format!("🔥 Process failed: {}", err), 0, Color::Red),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        // Status text
        let status = Paragraph::new(content)
            .block(
                Block::default()
                    .title("🔧 File Processor")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))
                    .border_set(border::ROUNDED),
            )
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(status, chunks[0]);

        // Progress bar at bottom
        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(color))
            .percent(progress_percent);

        frame.render_widget(progress_bar, chunks[1]);
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
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled(
                "ESC",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to quit • Powered by ", Style::default().fg(Color::Gray)),
            Span::styled(
                "use_future",
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
