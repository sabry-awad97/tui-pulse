use chrono::{DateTime, Local};
use pulse::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use uuid::Uuid;

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
        let background = Block::default().style(Style::default().bg(Color::Rgb(15, 20, 35)));
        frame.render_widget(background, area);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(5), // Header
                Constraint::Min(10),   // TodoList
                Constraint::Length(3), // Footer
            ])
            .split(area);

        HeaderComponent.render(main_chunks[0], frame);
        TodoListComponent.render(main_chunks[1], frame);
        FooterComponent.render(main_chunks[2], frame);
    }
}

#[derive(Clone)]
struct HeaderComponent;

impl Component for HeaderComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let title_text = Text::from(vec![
            Line::from(vec![
                Span::styled("📝", Style::default().fg(Color::Yellow)),
                Span::styled(
                    " PULSE TODOLIST ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("📝", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::styled(
                "Professional task management with use_reducer hook",
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

#[derive(Clone, Debug)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
    created_at: DateTime<Local>,
    priority: Priority,
}

#[derive(Clone, Debug, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    #[allow(dead_code)]
    fn color(&self) -> Color {
        match self {
            Priority::Low => Color::Green,
            Priority::Medium => Color::Yellow,
            Priority::High => Color::Red,
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Priority::Low => "🟢",
            Priority::Medium => "🟡",
            Priority::High => "🔴",
        }
    }
}

#[derive(Clone, Debug)]
struct TodoState {
    todos: Vec<Todo>,
    filter: Filter,
    selected_index: usize,
    input_mode: bool,
    input_text: String,
    dialog: Option<DialogState>,
}

#[derive(Clone, Debug)]
struct DialogState {
    dialog_type: DialogType,
    title: String,
    message: String,
    todo_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
enum DialogType {
    DeleteConfirmation,
    ClearCompleted,
    #[allow(dead_code)]
    Exit,
}

#[derive(Clone, Debug, PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

#[derive(Clone, Debug)]
enum TodoAction {
    AddTodo(String, Priority),
    ToggleTodo(Uuid),
    #[allow(dead_code)]
    DeleteTodo(Uuid),
    SetFilter(Filter),
    SelectNext,
    SelectPrevious,
    ToggleInputMode,
    UpdateInput(String),
    #[allow(dead_code)]
    ClearCompleted,
    ShowDialog(DialogType, String, String, Option<Uuid>),
    CloseDialog,
    ConfirmDialog,
}

fn todo_reducer(state: TodoState, action: TodoAction) -> TodoState {
    match action {
        TodoAction::AddTodo(text, priority) => {
            if text.trim().is_empty() {
                return state;
            }

            let new_todo = Todo {
                id: Uuid::new_v4(),
                text: text.trim().to_string(),
                completed: false,
                created_at: Local::now(),
                priority,
            };

            let mut new_todos = state.todos.clone();
            new_todos.push(new_todo);

            TodoState {
                todos: new_todos,
                input_text: String::new(),
                input_mode: false,
                ..state
            }
        }
        TodoAction::ToggleTodo(id) => {
            let new_todos = state
                .todos
                .iter()
                .map(|todo| {
                    if todo.id == id {
                        Todo {
                            completed: !todo.completed,
                            ..todo.clone()
                        }
                    } else {
                        todo.clone()
                    }
                })
                .collect();

            TodoState {
                todos: new_todos,
                ..state
            }
        }
        TodoAction::DeleteTodo(id) => {
            let new_todos = state
                .todos
                .iter()
                .filter(|todo| todo.id != id)
                .cloned()
                .collect::<Vec<_>>();

            let new_selected =
                if state.selected_index > 0 && state.selected_index >= new_todos.len() {
                    new_todos.len().saturating_sub(1)
                } else {
                    state.selected_index
                };

            TodoState {
                todos: new_todos,
                selected_index: new_selected,
                ..state
            }
        }
        TodoAction::SetFilter(filter) => TodoState {
            filter,
            selected_index: 0,
            ..state
        },
        TodoAction::SelectNext => {
            let filtered_todos = filter_todos(&state.todos, &state.filter);
            let new_index = if filtered_todos.is_empty() {
                0
            } else {
                (state.selected_index + 1) % filtered_todos.len()
            };
            TodoState {
                selected_index: new_index,
                ..state
            }
        }
        TodoAction::SelectPrevious => {
            let filtered_todos = filter_todos(&state.todos, &state.filter);
            let new_index = if filtered_todos.is_empty() {
                0
            } else if state.selected_index == 0 {
                filtered_todos.len() - 1
            } else {
                state.selected_index - 1
            };
            TodoState {
                selected_index: new_index,
                ..state
            }
        }
        TodoAction::ToggleInputMode => TodoState {
            input_mode: !state.input_mode,
            ..state
        },
        TodoAction::UpdateInput(text) => TodoState {
            input_text: text,
            ..state
        },
        TodoAction::ClearCompleted => {
            let new_todos = state
                .todos
                .iter()
                .filter(|todo| !todo.completed)
                .cloned()
                .collect();

            TodoState {
                todos: new_todos,
                selected_index: 0,
                ..state
            }
        }
        TodoAction::ShowDialog(dialog_type, title, message, todo_id) => TodoState {
            dialog: Some(DialogState {
                dialog_type,
                title,
                message,
                todo_id,
            }),
            ..state
        },
        TodoAction::CloseDialog => TodoState {
            dialog: None,
            ..state
        },
        TodoAction::ConfirmDialog => {
            if let Some(dialog) = &state.dialog {
                match dialog.dialog_type {
                    DialogType::DeleteConfirmation => {
                        if let Some(todo_id) = dialog.todo_id {
                            let new_todos = state
                                .todos
                                .iter()
                                .filter(|todo| todo.id != todo_id)
                                .cloned()
                                .collect::<Vec<_>>();

                            let new_selected = if state.selected_index > 0
                                && state.selected_index >= new_todos.len()
                            {
                                new_todos.len().saturating_sub(1)
                            } else {
                                state.selected_index
                            };

                            TodoState {
                                todos: new_todos,
                                selected_index: new_selected,
                                dialog: None,
                                ..state
                            }
                        } else {
                            TodoState {
                                dialog: None,
                                ..state
                            }
                        }
                    }
                    DialogType::ClearCompleted => {
                        let new_todos = state
                            .todos
                            .iter()
                            .filter(|todo| !todo.completed)
                            .cloned()
                            .collect();

                        TodoState {
                            todos: new_todos,
                            selected_index: 0,
                            dialog: None,
                            ..state
                        }
                    }
                    DialogType::Exit => {
                        // Exit confirmation - this would trigger app exit
                        TodoState {
                            dialog: None,
                            ..state
                        }
                    }
                }
            } else {
                state
            }
        }
    }
}

fn filter_todos<'a>(todos: &'a [Todo], filter: &Filter) -> Vec<&'a Todo> {
    todos
        .iter()
        .filter(|todo| match filter {
            Filter::All => true,
            Filter::Active => !todo.completed,
            Filter::Completed => todo.completed,
        })
        .collect()
}

#[derive(Clone)]
struct TodoListComponent;

impl Component for TodoListComponent {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let initial_state = TodoState {
            todos: vec![
                Todo {
                    id: Uuid::new_v4(),
                    text: "Learn Rust TUI development".to_string(),
                    completed: false,
                    created_at: Local::now(),
                    priority: Priority::High,
                },
                Todo {
                    id: Uuid::new_v4(),
                    text: "Build awesome apps with Pulse".to_string(),
                    completed: true,
                    created_at: Local::now(),
                    priority: Priority::Medium,
                },
                Todo {
                    id: Uuid::new_v4(),
                    text: "Master use_reducer hook".to_string(),
                    completed: false,
                    created_at: Local::now(),
                    priority: Priority::Low,
                },
            ],
            filter: Filter::All,
            selected_index: 0,
            input_mode: false,
            input_text: String::new(),
            dialog: None,
        };

        let (state, dispatch) = use_reducer(todo_reducer, initial_state);
        let current_state = state.get();

        // Handle keyboard input
        if let Some(event) = use_event()
            && let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('a') if !current_state.input_mode => {
                    dispatch.call(TodoAction::ToggleInputMode);
                }
                KeyCode::Char('1') if !current_state.input_mode => {
                    dispatch.call(TodoAction::SetFilter(Filter::All));
                }
                KeyCode::Char('2') if !current_state.input_mode => {
                    dispatch.call(TodoAction::SetFilter(Filter::Active));
                }
                KeyCode::Char('3') if !current_state.input_mode => {
                    dispatch.call(TodoAction::SetFilter(Filter::Completed));
                }
                KeyCode::Char('c')
                    if !current_state.input_mode && current_state.dialog.is_none() =>
                {
                    dispatch.call(TodoAction::ShowDialog(
                        DialogType::ClearCompleted,
                        "Clear Completed Tasks".to_string(),
                        "Are you sure you want to clear all completed tasks? This action cannot be undone.".to_string(),
                        None,
                    ));
                }
                KeyCode::Up if !current_state.input_mode => {
                    dispatch.call(TodoAction::SelectPrevious);
                }
                KeyCode::Down if !current_state.input_mode => {
                    dispatch.call(TodoAction::SelectNext);
                }
                KeyCode::Enter if !current_state.input_mode => {
                    let filtered_todos = filter_todos(&current_state.todos, &current_state.filter);
                    if let Some(todo) = filtered_todos.get(current_state.selected_index) {
                        dispatch.call(TodoAction::ToggleTodo(todo.id));
                    }
                }
                KeyCode::Delete if !current_state.input_mode && current_state.dialog.is_none() => {
                    let filtered_todos = filter_todos(&current_state.todos, &current_state.filter);
                    if let Some(todo) = filtered_todos.get(current_state.selected_index) {
                        dispatch.call(TodoAction::ShowDialog(
                            DialogType::DeleteConfirmation,
                            "Delete Task".to_string(),
                            format!("Are you sure you want to delete '{}'?", todo.text),
                            Some(todo.id),
                        ));
                    }
                }
                KeyCode::Enter if current_state.input_mode => {
                    dispatch.call(TodoAction::AddTodo(
                        current_state.input_text.clone(),
                        Priority::Medium,
                    ));
                }
                KeyCode::Esc if current_state.input_mode => {
                    dispatch.call(TodoAction::ToggleInputMode);
                }
                KeyCode::Char(c) if current_state.input_mode => {
                    let mut new_text = current_state.input_text.clone();
                    new_text.push(c);
                    dispatch.call(TodoAction::UpdateInput(new_text));
                }
                KeyCode::Backspace if current_state.input_mode => {
                    let mut new_text = current_state.input_text.clone();
                    new_text.pop();
                    dispatch.call(TodoAction::UpdateInput(new_text));
                }
                // Dialog controls
                KeyCode::Enter if current_state.dialog.is_some() => {
                    dispatch.call(TodoAction::ConfirmDialog);
                }
                KeyCode::Esc if current_state.dialog.is_some() => {
                    dispatch.call(TodoAction::CloseDialog);
                }
                KeyCode::Char('y') if current_state.dialog.is_some() => {
                    dispatch.call(TodoAction::ConfirmDialog);
                }
                KeyCode::Char('n') if current_state.dialog.is_some() => {
                    dispatch.call(TodoAction::CloseDialog);
                }
                _ => {}
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        // Main todo list
        self.render_todo_list(&current_state, chunks[0], frame);

        // Sidebar with stats and controls
        self.render_sidebar(&current_state, chunks[1], frame);

        // Render dialog overlay if present
        if current_state.dialog.is_some() {
            self.render_dialog(&current_state, area, frame);
        }
    }
}

impl TodoListComponent {
    fn render_todo_list(&self, state: &TodoState, area: Rect, frame: &mut Frame) {
        let filtered_todos = filter_todos(&state.todos, &state.filter);

        let items: Vec<ListItem> = filtered_todos
            .iter()
            .enumerate()
            .map(|(i, todo)| {
                let checkbox = if todo.completed { "☑" } else { "☐" };
                let style = if i == state.selected_index {
                    Style::default()
                        .bg(Color::Rgb(40, 50, 70))
                        .add_modifier(Modifier::BOLD)
                } else if todo.completed {
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = Line::from(vec![
                    Span::styled(format!("{} ", checkbox), Style::default().fg(Color::Green)),
                    Span::styled(format!("{} ", todo.priority.icon()), Style::default()),
                    Span::styled(&todo.text, style),
                    Span::styled(
                        format!(" ({})", todo.created_at.format("%m/%d %H:%M")),
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]);

                ListItem::new(content).style(if i == state.selected_index {
                    Style::default().bg(Color::Rgb(40, 50, 70))
                } else {
                    Style::default()
                })
            })
            .collect();

        let filter_text = match state.filter {
            Filter::All => "All Tasks",
            Filter::Active => "Active Tasks",
            Filter::Completed => "Completed Tasks",
        };

        let list = List::new(items).block(
            Block::default()
                .title(format!(
                    "📋 {} ({}/{})",
                    filter_text,
                    filtered_todos.len(),
                    state.todos.len()
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .border_set(border::ROUNDED),
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(area);

        frame.render_widget(list, chunks[0]);

        // Input box
        if state.input_mode {
            let input = Paragraph::new(state.input_text.as_str())
                .block(
                    Block::default()
                        .title("✏️  Add New Task (Enter to save, Esc to cancel)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))
                        .border_set(border::ROUNDED),
                )
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(input, chunks[1]);
        }
    }

    fn render_sidebar(&self, state: &TodoState, area: Rect, frame: &mut Frame) {
        let total = state.todos.len();
        let completed = state.todos.iter().filter(|t| t.completed).count();
        let active = total - completed;
        let high_priority = state
            .todos
            .iter()
            .filter(|t| t.priority == Priority::High && !t.completed)
            .count();

        let stats_text = Text::from(vec![
            Line::from(vec![
                Span::styled("📊 ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "STATISTICS",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Total: ", Style::default().fg(Color::White)),
                Span::styled(
                    total.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Active: ", Style::default().fg(Color::White)),
                Span::styled(
                    active.to_string(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Completed: ", Style::default().fg(Color::White)),
                Span::styled(
                    completed.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("High Priority: ", Style::default().fg(Color::White)),
                Span::styled(
                    high_priority.to_string(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🎯 ", Style::default().fg(Color::Green)),
                Span::styled(
                    "CONTROLS",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from("A - Add task"),
            Line::from("↑↓ - Navigate"),
            Line::from("Enter - Toggle"),
            Line::from("Del - Delete"),
            Line::from("1/2/3 - Filter"),
            Line::from("C - Clear done"),
        ]);

        let stats = Paragraph::new(stats_text).block(
            Block::default()
                .title("📈 Dashboard")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .border_set(border::ROUNDED),
        );

        frame.render_widget(stats, area);
    }

    fn render_dialog(&self, state: &TodoState, area: Rect, frame: &mut Frame) {
        if let Some(dialog) = &state.dialog {
            // Center the dialog
            let dialog_width = 60;
            let dialog_height = 12;
            let x = (area.width.saturating_sub(dialog_width)) / 2;
            let y = (area.height.saturating_sub(dialog_height)) / 2;

            let dialog_area = Rect {
                x: area.x + x,
                y: area.y + y,
                width: dialog_width,
                height: dialog_height,
            };

            // Clear the area for the dialog overlay
            frame.render_widget(Clear, dialog_area);

            // Dialog content
            let dialog_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(4),    // Message
                    Constraint::Length(3), // Buttons
                ])
                .split(dialog_area);

            // Title
            let title_icon = match dialog.dialog_type {
                DialogType::DeleteConfirmation => "🗑️",
                DialogType::ClearCompleted => "🧹",
                DialogType::Exit => "🚪",
            };

            let title = Paragraph::new(format!("{} {}", title_icon, dialog.title))
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);

            // Message
            let message = Paragraph::new(dialog.message.as_str())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });

            // Buttons with better spacing and styling
            let buttons_text = Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "  [Y] ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Yes", Style::default().fg(Color::Green)),
                    Span::styled(
                        "    [N] ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("No", Style::default().fg(Color::Red)),
                    Span::styled(
                        "    [ESC] ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Cancel", Style::default().fg(Color::Gray)),
                ]),
            ]);

            let buttons = Paragraph::new(buttons_text).alignment(Alignment::Center);

            frame.render_widget(title, dialog_chunks[0]);
            frame.render_widget(message, dialog_chunks[1]);
            frame.render_widget(buttons, dialog_chunks[2]);
        }
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
                "use_reducer",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" hook", Style::default().fg(Color::Gray)),
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
