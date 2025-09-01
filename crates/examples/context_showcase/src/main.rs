//! Beautiful Context Provider Showcase
//!
//! This example demonstrates the elegant context API for sharing state between
//! components without prop drilling. Features a beautiful theme system and
//! user management with nested component hierarchy.

use chrono::{DateTime, Utc};
use crossterm::event::Event;
use pulse::prelude::*;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Beautiful theme context with multiple color schemes
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeContext {
    pub name: String,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
}

/// Application settings context
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsContext {
    pub show_animations: bool,
    pub auto_save: bool,
    pub notification_sound: bool,
    pub theme_index: usize,
}

impl ThemeContext {
    pub fn ocean() -> Self {
        Self {
            name: "🌊 Ocean".to_string(),
            primary: Color::Rgb(52, 152, 219),
            secondary: Color::Rgb(41, 128, 185),
            accent: Color::Rgb(241, 196, 15),
            background: Color::Rgb(44, 62, 80),
        }
    }

    pub fn forest() -> Self {
        Self {
            name: "🌲 Forest".to_string(),
            primary: Color::Rgb(39, 174, 96),
            secondary: Color::Rgb(46, 204, 113),
            accent: Color::Rgb(230, 126, 34),
            background: Color::Rgb(39, 55, 70),
        }
    }

    pub fn sunset() -> Self {
        Self {
            name: "🌅 Sunset".to_string(),
            primary: Color::Rgb(231, 76, 60),
            secondary: Color::Rgb(192, 57, 43),
            accent: Color::Rgb(255, 193, 7),
            background: Color::Rgb(52, 73, 94),
        }
    }

    pub fn neon() -> Self {
        Self {
            name: "⚡ Neon".to_string(),
            primary: Color::Rgb(255, 20, 147),
            secondary: Color::Rgb(138, 43, 226),
            accent: Color::Rgb(0, 255, 255),
            background: Color::Rgb(25, 25, 25),
        }
    }

    pub fn all_themes() -> Vec<Self> {
        vec![Self::ocean(), Self::forest(), Self::sunset(), Self::neon()]
    }
}

/// User context for authentication and personalization
#[derive(Debug, Clone, PartialEq)]
pub struct UserContext {
    pub name: String,
    pub role: String,
    pub avatar: String,
    pub preferences: UserPreferences,
    pub last_login: DateTime<Utc>,
    pub session_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserPreferences {
    pub notifications: bool,
    pub dark_mode: bool,
    pub language: String,
}

impl UserContext {
    pub fn admin() -> Self {
        Self {
            name: "Alice Admin".to_string(),
            role: "Administrator".to_string(),
            avatar: "👩‍💼".to_string(),
            preferences: UserPreferences {
                notifications: true,
                dark_mode: true,
                language: "English".to_string(),
            },
            last_login: Utc::now(),
            session_count: 42,
        }
    }

    pub fn user() -> Self {
        Self {
            name: "Bob Developer".to_string(),
            role: "Developer".to_string(),
            avatar: "👨‍💻".to_string(),
            preferences: UserPreferences {
                notifications: true,
                dark_mode: false,
                language: "English".to_string(),
            },
            last_login: Utc::now(),
            session_count: 15,
        }
    }

    pub fn guest() -> Self {
        Self {
            name: "Guest User".to_string(),
            role: "Guest".to_string(),
            avatar: "👤".to_string(),
            preferences: UserPreferences {
                notifications: false,
                dark_mode: false,
                language: "English".to_string(),
            },
            last_login: Utc::now(),
            session_count: 1,
        }
    }
}

/// Main application component that provides contexts
#[derive(Clone)]
pub struct ContextApp;

impl Component for ContextApp {
    fn render(&self, area: Rect, frame: &mut Frame) {
        // App state for theme switching and user switching
        let (theme_index, set_theme_index) = use_state(|| 0usize);
        let (user_index, set_user_index) = use_state(|| 0usize);
        let (show_help, set_show_help) = use_state(|| false);

        // Get current theme and user
        let themes = ThemeContext::all_themes();
        let users = vec![
            UserContext::admin(),
            UserContext::user(),
            UserContext::guest(),
        ];
        let current_theme = themes[theme_index.get() % themes.len()].clone();
        let current_user = users[user_index.get() % users.len()].clone();

        // Provide contexts
        let theme = use_context_provider(|| current_theme);
        let _user = use_context_provider(|| current_user);
        let _settings = use_context_provider(|| SettingsContext {
            show_animations: true,
            auto_save: true,
            notification_sound: false,
            theme_index: theme_index.get(),
        });

        // Handle input events
        if let Some(Event::Key(key)) = use_event()
            && key.kind == crossterm::event::KeyEventKind::Press
        {
            match key.code {
                crossterm::event::KeyCode::Char('q') => request_exit(),
                crossterm::event::KeyCode::Char('t') => {
                    set_theme_index.update(|i| (*i + 1) % themes.len());
                }
                crossterm::event::KeyCode::Char('u') => {
                    set_user_index.update(|i| (*i + 1) % users.len());
                }
                crossterm::event::KeyCode::Char('h') => {
                    set_show_help.update(|h| !h);
                }
                _ => {}
            }
        }

        if show_help.get() {
            render_help_overlay(area, frame, &theme);
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

        // Render header
        HeaderComponent.render(chunks[0], frame);

        // Content area with three columns
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(chunks[1]);

        UserProfileCard.render(content_chunks[0], frame);
        ThemeShowcase.render(content_chunks[1], frame);
        SettingsPanel.render(content_chunks[2], frame);

        // Footer
        FooterComponent.render(chunks[2], frame);
    }
}

/// Header component that consumes theme context
#[derive(Clone)]
pub struct HeaderComponent;

impl Component for HeaderComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let theme = use_context::<ThemeContext>();

        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled("✨ ", Style::default().fg(theme.accent)),
            Span::styled(
                "Context Provider Showcase",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ✨", Style::default().fg(theme.accent)),
        ])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary)),
        )
        .alignment(Alignment::Center);

        frame.render_widget(header, area);
    }
}

/// User profile card that consumes both theme and user contexts
#[derive(Clone)]
pub struct UserProfileCard;

impl Component for UserProfileCard {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let theme = use_context::<ThemeContext>();
        let user = use_context::<UserContext>();

        let profile_text = vec![
            Line::from(vec![Span::styled(
                "👤 User Profile",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.secondary)),
                Span::styled(&user.name, Style::default().fg(theme.primary)),
            ]),
            Line::from(vec![
                Span::styled("Role: ", Style::default().fg(theme.secondary)),
                Span::styled(&user.role, Style::default().fg(theme.primary)),
            ]),
            Line::from(vec![
                Span::styled("Avatar: ", Style::default().fg(theme.secondary)),
                Span::styled(&user.avatar, Style::default().fg(theme.primary)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Preferences:",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("  Notifications: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    if user.preferences.notifications {
                        "✅ On"
                    } else {
                        "❌ Off"
                    },
                    Style::default().fg(if user.preferences.notifications {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Dark Mode: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    if user.preferences.dark_mode {
                        "🌙 On"
                    } else {
                        "☀️ Off"
                    },
                    Style::default().fg(theme.primary),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Language: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    &user.preferences.language,
                    Style::default().fg(theme.primary),
                ),
            ]),
        ];

        let profile_card = Paragraph::new(profile_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(" 👤 Profile "),
            )
            .style(Style::default().bg(theme.background));

        frame.render_widget(profile_card, area);
    }
}

/// Theme showcase that demonstrates theme context usage
#[derive(Clone)]
pub struct ThemeShowcase;

impl Component for ThemeShowcase {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let theme = use_context::<ThemeContext>();

        let theme_text = vec![
            Line::from(vec![Span::styled(
                "🎨 Theme Showcase",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Current Theme: ", Style::default().fg(theme.secondary)),
                Span::styled(&theme.name, Style::default().fg(theme.primary)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Color Palette:",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("  Primary: ", Style::default().fg(Color::White)),
                Span::styled("████", Style::default().fg(theme.primary)),
                Span::styled(" ", Style::default()),
                Span::styled("Primary", Style::default().fg(theme.primary)),
            ]),
            Line::from(vec![
                Span::styled("  Secondary: ", Style::default().fg(Color::White)),
                Span::styled("████", Style::default().fg(theme.secondary)),
                Span::styled(" ", Style::default()),
                Span::styled("Secondary", Style::default().fg(theme.secondary)),
            ]),
            Line::from(vec![
                Span::styled("  Accent: ", Style::default().fg(Color::White)),
                Span::styled("████", Style::default().fg(theme.accent)),
                Span::styled(" ", Style::default()),
                Span::styled("Accent", Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("  Background: ", Style::default().fg(Color::White)),
                Span::styled("████", Style::default().fg(theme.background)),
                Span::styled(" ", Style::default()),
                Span::styled("Background", Style::default().fg(theme.background)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Context Demo:",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("This component automatically"),
            Line::from("receives theme colors from"),
            Line::from("the parent context provider!"),
        ];

        let theme_card = Paragraph::new(theme_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(" 🎨 Theme "),
            )
            .style(Style::default().bg(theme.background));

        frame.render_widget(theme_card, area);
    }
}

/// Footer component with controls
#[derive(Clone)]
pub struct FooterComponent;

impl Component for FooterComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let theme = use_context::<ThemeContext>();

        let footer_text = vec![Line::from(vec![
            Span::styled(
                "Controls: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("t", Style::default().fg(theme.primary)),
            Span::styled(" Theme | ", Style::default().fg(Color::Gray)),
            Span::styled("u", Style::default().fg(theme.primary)),
            Span::styled(" User | ", Style::default().fg(Color::Gray)),
            Span::styled("h", Style::default().fg(theme.primary)),
            Span::styled(" Help | ", Style::default().fg(Color::Gray)),
            Span::styled("q", Style::default().fg(theme.primary)),
            Span::styled(" Quit", Style::default().fg(Color::Gray)),
        ])];

        let footer = Paragraph::new(footer_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(footer, area);
    }
}

/// Settings panel component that demonstrates multiple context consumption
#[derive(Clone)]
pub struct SettingsPanel;

impl Component for SettingsPanel {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let theme = use_context::<ThemeContext>();
        let settings = use_context::<SettingsContext>();
        let user = use_context::<UserContext>();

        let settings_text = vec![
            Line::from(vec![Span::styled(
                "⚙️ Settings",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("User: ", Style::default().fg(theme.secondary)),
                Span::styled(&user.name, Style::default().fg(theme.primary)),
            ]),
            Line::from(vec![
                Span::styled("Role: ", Style::default().fg(theme.secondary)),
                Span::styled(&user.role, Style::default().fg(theme.primary)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "App Settings:",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("  Animations: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    if settings.show_animations {
                        "✅ On"
                    } else {
                        "❌ Off"
                    },
                    Style::default().fg(if settings.show_animations {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Auto-Save: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    if settings.auto_save {
                        "💾 On"
                    } else {
                        "❌ Off"
                    },
                    Style::default().fg(if settings.auto_save {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Sound: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    if settings.notification_sound {
                        "🔊 On"
                    } else {
                        "🔇 Off"
                    },
                    Style::default().fg(if settings.notification_sound {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Session Info:",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("  Sessions: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    user.session_count.to_string(),
                    Style::default().fg(theme.primary),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Theme #: ", Style::default().fg(theme.secondary)),
                Span::styled(
                    (settings.theme_index + 1).to_string(),
                    Style::default().fg(theme.primary),
                ),
            ]),
        ];

        let settings_card = Paragraph::new(settings_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(" ⚙️ Settings "),
            )
            .style(Style::default().bg(theme.background));

        frame.render_widget(settings_card, area);
    }
}

/// Help overlay that demonstrates context usage in modals
fn render_help_overlay(area: Rect, frame: &mut Frame, theme: &ThemeContext) {
    let popup_area = centered_rect(70, 80, area);

    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "🎯 Context Provider Showcase Help",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "What is Context?",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("Context allows components to share state"),
        Line::from("without passing props through every level."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  t - Cycle through themes"),
        Line::from("  u - Switch between users"),
        Line::from("  h - Toggle this help"),
        Line::from("  q - Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Context Features:",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  🎨 Theme context shared across all components"),
        Line::from("  👤 User context with role-based data"),
        Line::from("  ⚙️ Settings context for app configuration"),
        Line::from("  🔄 Real-time updates without prop drilling"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Architecture:",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  • ContextApp provides all contexts"),
        Line::from("  • Child components consume contexts"),
        Line::from("  • No manual prop passing required"),
        Line::from("  • Type-safe context access"),
        Line::from(""),
        Line::from("Press 'h' again to close"),
    ];

    let help_popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" 📖 Help "),
        )
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    frame.render_widget(help_popup, popup_area);
}

/// Utility function to create a centered rectangle for popups
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
    pulse::render_async(|| async { ContextApp }).await?;
    Ok(())
}
