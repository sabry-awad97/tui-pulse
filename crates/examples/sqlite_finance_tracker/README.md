# 🏦 Personal Finance Tracker - SQLite Backend Showcase

A beautiful terminal-based personal finance tracker that demonstrates the power of the SQLite storage backend with async operations.

## ✨ Features

- **💰 Transaction Management**: Track income and expenses with categories
- **📊 Real-time Balance**: Automatic balance calculations
- **🎯 Budget Tracking**: Set budgets and monitor spending with visual progress bars
- **💾 SQLite Persistence**: All data stored in a local SQLite database
- **⚡ Async Operations**: Non-blocking database operations with connection pooling
- **🎨 Beautiful UI**: Modern themed interface with icons and colors
- **📱 Responsive Layout**: Adaptive layout for different terminal sizes

## 🚀 Quick Start

```bash
# Navigate to the example directory
cd crates/examples/sqlite_finance_tracker

# Run the application
cargo run
```

## 🎮 Controls

| Key | Action |
|-----|--------|
| `Tab` | Switch between Dashboard, Transactions, and Budgets tabs |
| `A` | Add new transaction (modal) |
| `S` | Save data to SQLite database |
| `Q` | Quit application |

## 🏗️ Architecture

### SQLite Backend Integration

The application showcases advanced SQLite backend usage:

```rust
// Initialize SQLite backend with custom table
let backend = SqliteStorageBackend::new_with_table(
    "sqlite:finance_tracker.db",
    "finance_data"
).await?;

// Async data operations
let data = backend.read_async("finance_data").await?;
backend.write_async("finance_data", &json_data).await?;
```

### Data Structures

- **Transaction**: Individual financial transactions with categories and timestamps
- **Budget**: Category-based spending limits with progress tracking
- **FinanceData**: Main application state with transactions and budgets

### Real-time Features

- **Balance Calculation**: Automatically computed from all transactions
- **Budget Progress**: Visual gauges showing spending vs. limits
- **Category Icons**: Emoji-based visual categorization
- **Color Coding**: Status-based color schemes (green/yellow/red)

## 📊 Database Schema

The SQLite backend automatically creates the following table structure:

```sql
CREATE TABLE finance_data (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at DATETIME DEFAULT (datetime('now')),
    updated_at DATETIME DEFAULT (datetime('now'))
);
```

## 🎨 UI Components

### Dashboard Tab
- Current balance display
- Recent transactions list
- Budget status overview

### Transactions Tab
- Complete transaction history
- Category icons and colors
- Date and amount formatting

### Budgets Tab
- Visual progress bars for each category
- Spending vs. limit comparisons
- Over-budget warnings

## 🔧 Technical Highlights

### Async Integration
```rust
// Tokio runtime integration
let rt = tokio::runtime::Handle::current();
let data = rt.block_on(async {
    backend.read_async("finance_data").await
});
```

### Error Handling
```rust
match backend.read_async("finance_data").await {
    Ok(Some(data)) => serde_json::from_str(&data).unwrap_or_default(),
    _ => FinanceData::default(),
}
```

### State Management
```rust
let (data, set_data) = use_state(|| finance_data);
let (selected_tab, set_selected_tab) = use_state(|| 0);
```

## 📁 File Structure

```
sqlite_finance_tracker/
├── Cargo.toml          # Dependencies and configuration
├── README.md           # This documentation
└── src/
    └── main.rs         # Complete application implementation
```

## 🎯 Learning Objectives

This example demonstrates:

1. **SQLite Backend Setup**: How to initialize and configure the SQLite storage backend
2. **Async Operations**: Integration of async database operations with TUI components
3. **Data Modeling**: Designing serializable data structures for persistence
4. **Error Handling**: Graceful handling of database errors and fallbacks
5. **UI Design**: Creating beautiful, functional terminal interfaces
6. **State Management**: Managing complex application state with hooks

## 🚀 Extensions

Consider extending this example with:

- **Transaction Categories**: Add/edit custom categories
- **Date Filtering**: Filter transactions by date ranges
- **Export Features**: Export data to CSV/JSON
- **Multiple Accounts**: Support for multiple bank accounts
- **Recurring Transactions**: Automatic recurring income/expenses
- **Data Visualization**: Charts and graphs for spending patterns

## 💡 SQLite Backend Benefits

This example showcases why SQLite is perfect for desktop applications:

- **Zero Configuration**: No server setup required
- **ACID Compliance**: Reliable transactions and data integrity
- **Performance**: Fast queries with proper indexing
- **Portability**: Single file database, easy to backup/share
- **SQL Power**: Full SQL query capabilities for complex operations
- **Concurrent Access**: Multiple readers, single writer model

## 🏃‍♂️ Next Steps

1. Run the example and explore the interface
2. Add your own transactions and budgets
3. Examine the SQLite database file created
4. Modify the code to add new features
5. Study the async integration patterns
