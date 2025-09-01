//! 🏦 Personal Finance Tracker - SQLite Backend Showcase
//!
//! This example demonstrates the power of the SQLite storage backend with a beautiful
//! personal finance tracking application. Features include:
//! - 💰 Transaction management with categories
//! - 📊 Real-time balance calculations
//! - 🎯 Budget tracking and alerts
//! - 💾 Persistent SQLite storage
//! - 🎨 Beautiful themed UI
//! - ⚡ Async database operations

use chrono::{DateTime, Utc};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pulse::prelude::*;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Transaction data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub amount: f64,
    pub description: String,
    pub category: TransactionCategory,
    pub transaction_type: TransactionType,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    Income,
    Expense,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionCategory {
    Food,
    Transportation,
    Entertainment,
    Shopping,
    Bills,
    Healthcare,
    Education,
    Salary,
    Investment,
    Other,
}

impl TransactionCategory {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Food => "🍔",
            Self::Transportation => "🚗",
            Self::Entertainment => "🎬",
            Self::Shopping => "🛍️",
            Self::Bills => "📄",
            Self::Healthcare => "🏥",
            Self::Education => "📚",
            Self::Salary => "💼",
            Self::Investment => "📈",
            Self::Other => "📦",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Food => Color::Yellow,
            Self::Transportation => Color::Blue,
            Self::Entertainment => Color::Magenta,
            Self::Shopping => Color::Cyan,
            Self::Bills => Color::Red,
            Self::Healthcare => Color::Green,
            Self::Education => Color::LightBlue,
            Self::Salary => Color::LightGreen,
            Self::Investment => Color::LightMagenta,
            Self::Other => Color::Gray,
        }
    }
}

/// Budget tracking structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    pub category: TransactionCategory,
    pub limit: f64,
    pub spent: f64,
}

impl Budget {
    pub fn percentage_used(&self) -> f64 {
        if self.limit == 0.0 {
            0.0
        } else {
            (self.spent / self.limit * 100.0).min(100.0)
        }
    }

    pub fn is_over_budget(&self) -> bool {
        self.spent > self.limit
    }
}

/// Application state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinanceData {
    pub transactions: Vec<Transaction>,
    pub budgets: Vec<Budget>,
    pub balance: f64,
}

impl Default for FinanceData {
    fn default() -> Self {
        Self {
            transactions: vec![
                Transaction {
                    id: Uuid::new_v4(),
                    amount: 3500.0,
                    description: "Monthly Salary".to_string(),
                    category: TransactionCategory::Salary,
                    transaction_type: TransactionType::Income,
                    date: Utc::now(),
                },
                Transaction {
                    id: Uuid::new_v4(),
                    amount: -45.50,
                    description: "Grocery Shopping".to_string(),
                    category: TransactionCategory::Food,
                    transaction_type: TransactionType::Expense,
                    date: Utc::now(),
                },
                Transaction {
                    id: Uuid::new_v4(),
                    amount: -12.99,
                    description: "Netflix Subscription".to_string(),
                    category: TransactionCategory::Entertainment,
                    transaction_type: TransactionType::Expense,
                    date: Utc::now(),
                },
            ],
            budgets: vec![
                Budget {
                    category: TransactionCategory::Food,
                    limit: 400.0,
                    spent: 45.50,
                },
                Budget {
                    category: TransactionCategory::Entertainment,
                    limit: 100.0,
                    spent: 12.99,
                },
                Budget {
                    category: TransactionCategory::Transportation,
                    limit: 200.0,
                    spent: 0.0,
                },
            ],
            balance: 3441.51,
        }
    }
}

/// Application theme
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub background: Color,
    pub text: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Rgb(99, 102, 241),    // Indigo
            secondary: Color::Rgb(107, 114, 128), // Gray
            accent: Color::Rgb(34, 197, 94),      // Green
            success: Color::Rgb(34, 197, 94),     // Green
            warning: Color::Rgb(251, 191, 36),    // Amber
            danger: Color::Rgb(239, 68, 68),      // Red
            background: Color::Rgb(17, 24, 39),   // Dark
            text: Color::Rgb(243, 244, 246),      // Light gray
        }
    }
}

/// Main application component
#[derive(Clone)]
pub struct FinanceTracker {
    backend: Arc<SqliteStorageBackend>,
}

impl FinanceTracker {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize SQLite backend with a beautiful database
        // Use a more robust path that ensures the directory exists
        let db_path = std::env::temp_dir().join("finance_tracker.db");
        let db_url = format!("sqlite:{}", db_path.display());

        println!("📁 Database path: {}", db_path.display());

        let backend = SqliteStorageBackend::new_with_table(&db_url, "finance_data").await?;

        Ok(Self {
            backend: Arc::new(backend),
        })
    }
}

impl Component for FinanceTracker {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let theme = Theme::default();

        // Initialize with default data - we'll load from SQLite using effects
        let (data, set_data) = use_state(FinanceData::default);

        // Load data from SQLite on component mount
        let backend_clone = self.backend.clone();
        let data_loader = use_future(
            move || {
                let backend = backend_clone.clone();
                async move {
                    match backend.read_async("finance_data").await {
                        Ok(Some(json_data)) => serde_json::from_str::<FinanceData>(&json_data)
                            .map_err(|e| format!("Failed to parse data: {}", e)),
                        Ok(None) => Ok(FinanceData::default()),
                        Err(e) => Err(format!("Failed to load data: {}", e)),
                    }
                }
            },
            (), // Load once on mount
        );

        // Update state when data is loaded
        if let Some(loaded_data) = data_loader.value()
            && data.get().transactions.is_empty()
            && !loaded_data.transactions.is_empty()
        {
            set_data.set(loaded_data);
        }
        let (selected_tab, set_selected_tab) = use_state(|| 0);
        let (show_add_transaction, set_show_add_transaction) = use_state(|| false);
        let (save_trigger, set_save_trigger) = use_state(|| 0);

        // Auto-save data to SQLite when save_trigger changes
        let data_clone = data.get();
        let backend_clone = self.backend.clone();
        let save_trigger_value = save_trigger.get();
        let _save_future = use_future(
            move || async move {
                if save_trigger_value > 0 {
                    let json_data = serde_json::to_string(&data_clone)
                        .map_err(|e| format!("Failed to serialize data: {}", e))?;
                    backend_clone
                        .write_async("finance_data", &json_data)
                        .await
                        .map_err(|e| format!("Failed to save data: {}", e))?;
                }
                Ok::<(), String>(())
            },
            save_trigger_value, // Trigger save when save_trigger changes
        );

        // Handle keyboard input
        if let Some(Event::Key(key)) = use_event()
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => request_exit(),
                KeyCode::Tab => {
                    set_selected_tab.update(|tab| (*tab + 1) % 3);
                }
                KeyCode::Char('a') => {
                    set_show_add_transaction.update(|show| !show);
                }
                KeyCode::Char('s') => {
                    // Trigger save by updating save_trigger
                    set_save_trigger.update(|trigger| *trigger + 1);
                }
                _ => {}
            }
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
        render_header(chunks[0], frame, &theme);

        // Render content based on selected tab
        match selected_tab.get() {
            0 => render_dashboard(chunks[1], frame, &data.get(), &theme),
            1 => render_transactions(chunks[1], frame, &data.get(), &theme),
            2 => render_budgets(chunks[1], frame, &data.get(), &theme),
            _ => {}
        }

        // Render footer
        render_footer(chunks[2], frame, &theme);

        // Render add transaction modal if shown
        if show_add_transaction.get() {
            render_add_transaction_modal(area, frame, &theme, &set_show_add_transaction, &set_data);
        }
    }
}

fn render_header(area: Rect, frame: &mut Frame, theme: &Theme) {
    let header = Paragraph::new("🏦 Personal Finance Tracker - SQLite Backend Showcase")
        .style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary)),
        );
    frame.render_widget(header, area);
}

fn render_dashboard(area: Rect, frame: &mut Frame, data: &FinanceData, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Balance card
            Constraint::Min(0),    // Charts and stats
        ])
        .split(area);

    // Balance card
    let balance_color = if data.balance >= 0.0 {
        theme.success
    } else {
        theme.danger
    };
    let balance_text = format!("💰 Current Balance: ${:.2}", data.balance);

    let balance_card = Paragraph::new(balance_text)
        .style(
            Style::default()
                .fg(balance_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 💳 Account Overview ")
                .border_style(Style::default().fg(theme.accent)),
        );
    frame.render_widget(balance_card, chunks[0]);

    // Stats section
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Recent transactions
    render_recent_transactions(stats_chunks[0], frame, data, theme);

    // Budget overview
    render_budget_overview(stats_chunks[1], frame, data, theme);
}

fn render_recent_transactions(area: Rect, frame: &mut Frame, data: &FinanceData, theme: &Theme) {
    let recent_transactions: Vec<ListItem> = data
        .transactions
        .iter()
        .take(5)
        .map(|transaction| {
            let amount_color = match transaction.transaction_type {
                TransactionType::Income => theme.success,
                TransactionType::Expense => theme.danger,
            };

            let amount_text = format!("{:+.2}", transaction.amount);

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(transaction.category.icon(), Style::default()),
                    Span::raw(" "),
                    Span::styled(&transaction.description, Style::default().fg(theme.text)),
                ]),
                Line::from(vec![Span::styled(
                    amount_text,
                    Style::default()
                        .fg(amount_color)
                        .add_modifier(Modifier::BOLD),
                )]),
            ])
        })
        .collect();

    let transactions_list = List::new(recent_transactions).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 📋 Recent Transactions ")
            .border_style(Style::default().fg(theme.secondary)),
    );

    frame.render_widget(transactions_list, area);
}

fn render_budget_overview(area: Rect, frame: &mut Frame, data: &FinanceData, theme: &Theme) {
    let budget_items: Vec<ListItem> = data
        .budgets
        .iter()
        .map(|budget| {
            let percentage = budget.percentage_used();
            let color = if budget.is_over_budget() {
                theme.danger
            } else if percentage > 80.0 {
                theme.warning
            } else {
                theme.success
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(budget.category.icon(), Style::default()),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:?}", budget.category),
                        Style::default().fg(theme.text),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    format!(
                        "${:.2} / ${:.2} ({:.1}%)",
                        budget.spent, budget.limit, percentage
                    ),
                    Style::default().fg(color),
                )]),
            ])
        })
        .collect();

    let budgets_list = List::new(budget_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 🎯 Budget Status ")
            .border_style(Style::default().fg(theme.secondary)),
    );

    frame.render_widget(budgets_list, area);
}

fn render_transactions(area: Rect, frame: &mut Frame, data: &FinanceData, theme: &Theme) {
    let transaction_items: Vec<ListItem> = data
        .transactions
        .iter()
        .map(|transaction| {
            let amount_color = match transaction.transaction_type {
                TransactionType::Income => theme.success,
                TransactionType::Expense => theme.danger,
            };

            let date_str = transaction.date.format("%m/%d %H:%M").to_string();

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(transaction.category.icon(), Style::default()),
                    Span::raw(" "),
                    Span::styled(&transaction.description, Style::default().fg(theme.text)),
                    Span::raw(" - "),
                    Span::styled(date_str, Style::default().fg(theme.secondary)),
                ]),
                Line::from(vec![Span::styled(
                    format!("{:+.2}", transaction.amount),
                    Style::default()
                        .fg(amount_color)
                        .add_modifier(Modifier::BOLD),
                )]),
            ])
        })
        .collect();

    let transactions_list = List::new(transaction_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 📊 All Transactions ")
            .border_style(Style::default().fg(theme.primary)),
    );

    frame.render_widget(transactions_list, area);
}

fn render_budgets(area: Rect, frame: &mut Frame, data: &FinanceData, theme: &Theme) {
    let budget_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            data.budgets
                .iter()
                .map(|_| Constraint::Length(4))
                .collect::<Vec<_>>(),
        )
        .split(area);

    for (i, budget) in data.budgets.iter().enumerate() {
        if i < budget_chunks.len() {
            let percentage = budget.percentage_used() as u16;
            let color = if budget.is_over_budget() {
                theme.danger
            } else if percentage > 80 {
                theme.warning
            } else {
                theme.success
            };

            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(format!(
                    " {} {:?} Budget ",
                    budget.category.icon(),
                    budget.category
                )))
                .gauge_style(Style::default().fg(color))
                .percent(percentage)
                .label(format!("${:.2} / ${:.2}", budget.spent, budget.limit));

            frame.render_widget(gauge, budget_chunks[i]);
        }
    }
}

fn render_add_transaction_modal(
    area: Rect,
    frame: &mut Frame,
    theme: &Theme,
    _set_show_modal: &StateSetter<bool>,
    _set_data: &StateSetter<FinanceData>,
) {
    let popup_area = centered_rect(60, 70, area);

    frame.render_widget(Clear, popup_area);

    let modal = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Add New Transaction",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Press 'Esc' to close this modal"),
        Line::from(""),
        Line::from("🚧 Transaction form coming soon!"),
        Line::from("This would include:"),
        Line::from("• Amount input"),
        Line::from("• Description field"),
        Line::from("• Category selection"),
        Line::from("• Date picker"),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" ➕ Add Transaction ")
            .border_style(Style::default().fg(theme.primary)),
    );

    frame.render_widget(modal, popup_area);
}

fn render_footer(area: Rect, frame: &mut Frame, theme: &Theme) {
    let footer_text = vec![Line::from(vec![
        Span::styled(
            "Tab",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Switch tabs | "),
        Span::styled(
            "A",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Add transaction | "),
        Span::styled(
            "S",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Save to SQLite | "),
        Span::styled(
            "Q",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Quit"),
    ])];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary)),
        );

    frame.render_widget(footer, area);
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
    println!("🏦 Starting Personal Finance Tracker with SQLite Backend...");
    println!("💾 Database: finance_tracker.db");
    println!("🎯 Features: Transactions, Budgets, Real-time Balance");
    println!("⚡ Backend: Async SQLite with connection pooling");

    // Initialize the finance tracker with SQLite backend first
    let app = match FinanceTracker::new().await {
        Ok(app) => {
            println!("✅ SQLite backend initialized successfully");
            app
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize SQLite backend: {}", e);
            return Err(e);
        }
    };

    // Run the application
    pulse::render_async(move || {
        let app_clone = app.clone();
        async move { app_clone }
    })
    .await?;

    Ok(())
}
