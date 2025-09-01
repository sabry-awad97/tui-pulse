# 🎨 Beautiful Task Manager - Storage Hook Showcase

A stunning interactive terminal application that demonstrates the power of the `use_local_storage` hook with persistent state management.

## ✨ Features

- **🎯 Smart Task Management**: Create, edit, and organize tasks with priorities
- **💾 Automatic Persistence**: All data saves automatically - never lose your work!
- **🎨 Beautiful Themes**: Multiple gorgeous color schemes that persist across sessions
- **📊 Live Statistics**: Real-time progress tracking and analytics
- **🔍 Interactive UI**: Smooth navigation with keyboard shortcuts
- **📁 Organized Storage**: Clean JSON files in `./beautiful_tasks/` directory

## 🚀 Quick Start

```bash
# Run the beautiful task manager
cargo run --bin task_manager

# Or run from the showcase directory
cd crates/examples/storage_showcase
cargo run
```

## 🎮 Controls

| Key | Action |
|-----|--------|
| `↑↓` | Navigate tasks |
| `Enter` | Toggle task completion |
| `n` | Create new task |
| `t` | Cycle through themes |
| `h` | Show/hide help |
| `q` | Quit (auto-saves) |

## 🎨 Themes

- **🌊 Ocean**: Calming blues and teals
- **🌲 Forest**: Natural greens and earth tones  
- **🌅 Sunset**: Warm oranges and yellows

## 💾 Storage Features Demonstrated

### 1. **Complex Data Structures**
```rust
#[derive(Serialize, Deserialize)]
pub struct AppData {
    pub tasks: Vec<Task>,
    pub theme: Theme,
    pub total_sessions: u64,
}

let (app_data, set_app_data) = use_local_storage("task_app".to_string(), AppData::default());
```

### 2. **Automatic Persistence**
```rust
// Any change automatically saves to disk!
set_app_data.update(|data| {
    let mut new_data = data.clone();
    new_data.tasks.push(new_task);
    new_data
});
```

### 3. **Beautiful Storage Configuration**
```rust
set_storage_config(LocalStorageConfig {
    storage_dir: PathBuf::from("./beautiful_tasks"),
    create_dir: true,
    file_extension: "json".to_string(),
    pretty_json: true, // Human-readable JSON files!
});
```

### 4. **Type Safety & Error Handling**
- Full serde serialization support
- Graceful fallbacks for storage errors
- Thread-safe operations

## 📁 Storage Structure

The app creates beautiful, organized storage:

```
./beautiful_tasks/
├── task_app.json          # Main application data
├── user_preferences.json  # UI preferences
└── session_stats.json     # Analytics data
```

## 🏗️ Architecture Highlights

- **Reactive State**: Changes trigger automatic UI updates
- **Persistent Storage**: Seamless file-based persistence
- **Component-Based**: Clean, modular component architecture
- **Theme System**: Dynamic styling with persistent preferences
- **Error Resilience**: Graceful handling of storage failures

## 🎯 Learning Outcomes

This example teaches you how to:

1. **Build persistent TUI applications** with automatic data saving
2. **Manage complex application state** with multiple storage keys
3. **Create beautiful, themed interfaces** that remember user preferences
4. **Handle real-world data structures** with proper serialization
5. **Implement reactive patterns** in terminal applications

## 🔧 Customization

Extend the example by adding:
- Task categories and projects
- Due dates and reminders
- Export/import functionality
- Collaborative features
- Custom themes and styling

---

**💡 Pro Tip**: Check out the generated JSON files in `./beautiful_tasks/` to see how your data is beautifully structured and persisted!
