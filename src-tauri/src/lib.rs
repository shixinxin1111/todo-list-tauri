use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow, WindowEvent,
};
use uuid::Uuid;

const WINDOW_MODE_CHANGED_EVENT: &str = "todo-window:mode-changed";
const TRAY_SHOW_NORMAL: &str = "show-normal";
const TRAY_SHOW_FLOATING: &str = "show-floating";
const TRAY_SHOW_MINI: &str = "show-mini-floating";
const TRAY_QUIT: &str = "quit-app";

const VISIBLE_WINDOW_BACKGROUND: &str = "#f4f1ea";
const FLOATING_WINDOW_MARGIN: i32 = 16;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum TodoWindowMode {
    Normal,
    Floating,
    MiniFloating,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TodoWindowStatePayload {
    mode: TodoWindowMode,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WindowCursorPositionPayload {
    x: f64,
    y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TodoStatus {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TodoItem {
    id: String,
    note: String,
    status: TodoStatus,
    title: String,
}

#[derive(Clone, Debug, Deserialize)]
struct TodoInput {
    note: String,
    status: Option<TodoStatus>,
    title: String,
}

struct TodoStore {
    todos_file_path: PathBuf,
}

struct AppState {
    todo_store: Mutex<TodoStore>,
    window_state: Mutex<TodoWindowStatePayload>,
    is_quitting: Mutex<bool>,
}

#[derive(Debug)]
struct AppError(String);

impl AppError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(value: tauri::Error) -> Self {
        Self(value.to_string())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl TodoStore {
    fn new(todos_file_path: PathBuf) -> Self {
        Self { todos_file_path }
    }

    fn list(&self) -> Result<Vec<TodoItem>, AppError> {
        match fs::read_to_string(&self.todos_file_path) {
            Ok(content) => validate_todos_file(serde_json::from_str(&content)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
    }

    fn add(&self, input: TodoInput) -> Result<Vec<TodoItem>, AppError> {
        let mut todos = self.list()?;
        todos.push(create_todo_item(input)?);
        self.write(&todos)
    }

    fn update(&self, id: &str, input: TodoInput) -> Result<Vec<TodoItem>, AppError> {
        let normalized = normalize_todo_input(input)?;
        let mut did_update = false;
        let todos = self
            .list()?
            .into_iter()
            .map(|todo| {
                if todo.id != id {
                    return todo;
                }

                did_update = true;
                TodoItem {
                    id: todo.id,
                    note: normalized.note.clone(),
                    status: normalized.status.clone().unwrap_or(todo.status),
                    title: normalized.title.clone(),
                }
            })
            .collect::<Vec<_>>();

        if !did_update {
            return Err(AppError::new("没有找到要更新的任务。"));
        }

        self.write(&todos)
    }

    fn delete(&self, id: &str) -> Result<Vec<TodoItem>, AppError> {
        let todos = self.list()?;
        let todo_count = todos.len();
        let next_todos = todos
            .into_iter()
            .filter(|todo| todo.id != id)
            .collect::<Vec<_>>();

        if next_todos.len() == todo_count {
            return Err(AppError::new("没有找到要删除的任务。"));
        }

        self.write(&next_todos)
    }

    fn set_status(&self, id: &str, status: TodoStatus) -> Result<Vec<TodoItem>, AppError> {
        let mut did_update = false;
        let todos = self
            .list()?
            .into_iter()
            .map(|todo| {
                if todo.id != id {
                    return todo;
                }

                did_update = true;
                TodoItem {
                    status: status.clone(),
                    ..todo
                }
            })
            .collect::<Vec<_>>();

        if !did_update {
            return Err(AppError::new("没有找到要更新的任务。"));
        }

        self.write(&todos)
    }

    fn write(&self, todos: &[TodoItem]) -> Result<Vec<TodoItem>, AppError> {
        if let Some(parent) = self.todos_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(
            &self.todos_file_path,
            format!("{}\n", serde_json::to_string_pretty(todos)?),
        )?;

        Ok(todos.to_vec())
    }
}

fn validate_todos_file(value: serde_json::Value) -> Result<Vec<TodoItem>, AppError> {
    serde_json::from_value(value)
        .map_err(|_| AppError::new("任务数据文件格式不正确，请检查本地 todos.json。"))
}

fn normalize_todo_input(input: TodoInput) -> Result<TodoInput, AppError> {
    let title = input.title.trim().to_string();
    let note = input.note.trim().to_string();

    if title.is_empty() {
        return Err(AppError::new("任务标题不能为空。"));
    }

    Ok(TodoInput {
        note,
        status: input.status,
        title,
    })
}

fn create_todo_item(input: TodoInput) -> Result<TodoItem, AppError> {
    let normalized = normalize_todo_input(input)?;

    Ok(TodoItem {
        id: Uuid::new_v4().to_string(),
        note: normalized.note,
        status: normalized.status.unwrap_or(TodoStatus::NotStarted),
        title: normalized.title,
    })
}

fn safe_top_right_bounds(
    work_area_min_x: f64,
    work_area_min_y: f64,
    work_area_max_x: f64,
    work_area_max_y: f64,
    current_right: f64,
    current_top: f64,
    next_width: f64,
    next_height: f64,
) -> LogicalPosition<f64> {
    let min_x = work_area_min_x + FLOATING_WINDOW_MARGIN as f64;
    let max_x = work_area_max_x - next_width - FLOATING_WINDOW_MARGIN as f64;
    let min_y = work_area_min_y + FLOATING_WINDOW_MARGIN as f64;
    let max_y = work_area_max_y - next_height - FLOATING_WINDOW_MARGIN as f64;

    let target_x = (current_right - next_width).clamp(min_x.min(max_x), max_x.max(min_x));
    let target_y = current_top.clamp(min_y.min(max_y), max_y.max(min_y));

    LogicalPosition::new(target_x, target_y)
}

fn emit_window_state(
    app: &AppHandle,
    state: &TodoWindowStatePayload,
) -> Result<(), AppError> {
    app.emit(WINDOW_MODE_CHANGED_EVENT, state)?;
    Ok(())
}

fn get_main_window(app: &AppHandle) -> Result<WebviewWindow, AppError> {
    app.get_webview_window("main")
        .ok_or_else(|| AppError::new("主窗口不存在。"))
}

fn apply_window_mode(
    app: &AppHandle,
    mode: TodoWindowMode,
) -> Result<TodoWindowStatePayload, AppError> {
    let window = get_main_window(app)?;
    let scale_factor = window.scale_factor()?;
    let current_position = window
        .outer_position()?
        .to_logical::<f64>(scale_factor);
    let current_size = window.outer_size()?.to_logical::<f64>(scale_factor);
    let monitor = window
        .current_monitor()?
        .ok_or_else(|| AppError::new("无法获取当前显示器信息。"))?;
    let work_area = monitor.work_area();
    let work_area_position = work_area.position.to_logical::<f64>(scale_factor);
    let work_area_size = work_area.size.to_logical::<f64>(scale_factor);

    let next_state = TodoWindowStatePayload { mode };

    let (next_width, next_height) = match mode {
        TodoWindowMode::Normal => (1000.0_f64, 720.0_f64),
        TodoWindowMode::Floating => (390.0_f64, 600.0_f64),
        TodoWindowMode::MiniFloating => (260.0_f64, 42.0_f64),
    };

    let current_right = current_position.x + current_size.width;
    let current_top = current_position.y;

    let next_position = safe_top_right_bounds(
        work_area_position.x,
        work_area_position.y,
        work_area_position.x + work_area_size.width,
        work_area_position.y + work_area_size.height,
        current_right,
        current_top,
        next_width,
        next_height,
    );

    if mode == TodoWindowMode::Normal {
        window.set_always_on_top(false)?;
        window.set_visible_on_all_workspaces(false)?;
        window.set_skip_taskbar(false)?;
        window.set_resizable(true)?;
        window.set_min_size(Some(LogicalSize::new(850.0, 520.0)))?;
    } else {
        window.set_always_on_top(true)?;
        window.set_visible_on_all_workspaces(true)?;
        window.set_skip_taskbar(true)?;
        window.set_resizable(false)?;
        window.set_min_size(Some(LogicalSize::new(next_width, next_height)))?;
    }

    // 始终保留原生窗口装饰（含圆角与阴影），悬浮态仅隐藏交通灯按钮。
    set_traffic_lights_hidden(&window, mode != TodoWindowMode::Normal);

    // 先定位再缩放，避免窗口先变大触发屏幕边界夹取，导致右上角参考点丢失。
    window.set_position(next_position)?;
    window.set_size(LogicalSize::new(next_width, next_height))?;
    // 缩放后再次锚定一次，确保 macOS 的尺寸动画结束后右上角仍对齐。
    window.set_position(next_position)?;
    window.set_title("Todo List App")?;
    window.set_background_color(Some(hex_to_rgba(VISIBLE_WINDOW_BACKGROUND)?))?;
    window.show()?;
    window.set_focus()?;

    {
        let state = app.state::<AppState>();
        let mut current = state
            .window_state
            .lock()
            .map_err(|_| AppError::new("窗口状态锁获取失败。"))?;
        *current = next_state.clone();
    }

    emit_window_state(app, &next_state)?;
    Ok(next_state)
}

fn hex_to_rgba(hex: &str) -> Result<tauri::window::Color, AppError> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(AppError::new("窗口背景色格式不正确。"));
    }

    let red = u8::from_str_radix(&hex[0..2], 16).map_err(|_| AppError::new("颜色解析失败。"))?;
    let green =
        u8::from_str_radix(&hex[2..4], 16).map_err(|_| AppError::new("颜色解析失败。"))?;
    let blue = u8::from_str_radix(&hex[4..6], 16).map_err(|_| AppError::new("颜色解析失败。"))?;

    Ok(tauri::window::Color(red, green, blue, 255))
}

#[cfg(target_os = "macos")]
fn set_traffic_lights_hidden(window: &WebviewWindow, hidden: bool) {
    use cocoa::appkit::{NSWindow, NSWindowButton};
    use objc::{msg_send, sel, sel_impl};

    let Ok(ns_window) = window.ns_window() else {
        return;
    };
    let ns_window = ns_window as cocoa::base::id;
    if ns_window.is_null() {
        return;
    }

    unsafe {
        for button in [
            NSWindowButton::NSWindowCloseButton,
            NSWindowButton::NSWindowMiniaturizeButton,
            NSWindowButton::NSWindowZoomButton,
        ] {
            let btn: cocoa::base::id = ns_window.standardWindowButton_(button);
            if !btn.is_null() {
                let _: () = msg_send![btn, setHidden: hidden];
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn set_traffic_lights_hidden(_window: &WebviewWindow, _hidden: bool) {}

fn setup_tray(app: &AppHandle) -> Result<(), AppError> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| AppError::new("无法读取默认应用图标。"))?;
    let menu = MenuBuilder::new(app)
        .text(TRAY_SHOW_NORMAL, "显示主窗口")
        .text(TRAY_SHOW_FLOATING, "显示悬浮窗")
        .text(TRAY_SHOW_MINI, "显示迷你悬浮窗")
        .separator()
        .text(TRAY_QUIT, "退出应用")
        .build()?;

    TrayIconBuilder::with_id("todo-tray")
        .icon(icon)
        .tooltip("Todo List App")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            TRAY_SHOW_NORMAL => {
                let _ = apply_window_mode(app, TodoWindowMode::Normal);
            }
            TRAY_SHOW_FLOATING => {
                let _ = apply_window_mode(app, TodoWindowMode::Floating);
            }
            TRAY_SHOW_MINI => {
                let _ = apply_window_mode(app, TodoWindowMode::MiniFloating);
            }
            TRAY_QUIT => {
                if let Ok(state) = app.state::<AppState>().is_quitting.lock() {
                    drop(state);
                }
                if let Ok(mut is_quitting) = app.state::<AppState>().is_quitting.lock() {
                    *is_quitting = true;
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = apply_window_mode(tray.app_handle(), TodoWindowMode::Normal);
            }
        })
        .build(app)?;

    Ok(())
}

fn attach_window_events(window: &WebviewWindow) {
    let app = window.app_handle().clone();
    let window_handle = window.clone();
    window.clone().on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let should_hide = app
                .state::<AppState>()
                .is_quitting
                .lock()
                .ok()
                .map(|is_quitting| !*is_quitting)
                .unwrap_or(true);
            let mode = app
                .state::<AppState>()
                .window_state
                .lock()
                .ok()
                .map(|state| state.mode)
                .unwrap_or(TodoWindowMode::Normal);

            if should_hide && mode != TodoWindowMode::Normal {
                api.prevent_close();
                let _ = window_handle.hide();
            }
        }
    });
}

#[tauri::command]
fn get_window_mode(state: tauri::State<AppState>) -> Result<TodoWindowStatePayload, AppError> {
    state
        .window_state
        .lock()
        .map(|window_state| window_state.clone())
        .map_err(|_| AppError::new("窗口状态读取失败。"))
}

#[tauri::command]
fn get_window_cursor_position(
    app: AppHandle,
) -> Result<Option<WindowCursorPositionPayload>, AppError> {
    let window = get_main_window(&app)?;
    let cursor_position = window.cursor_position()?;
    let window_position = window.inner_position()?;
    let window_size = window.inner_size()?;
    let scale_factor = window.scale_factor()?;

    let relative_x = (cursor_position.x - f64::from(window_position.x)) / scale_factor;
    let relative_y = (cursor_position.y - f64::from(window_position.y)) / scale_factor;
    let width = f64::from(window_size.width) / scale_factor;
    let height = f64::from(window_size.height) / scale_factor;

    if relative_x < 0.0 || relative_y < 0.0 || relative_x > width || relative_y > height {
        return Ok(None);
    }

    Ok(Some(WindowCursorPositionPayload {
        x: relative_x,
        y: relative_y,
    }))
}

#[tauri::command]
fn set_window_mode(
    app: AppHandle,
    mode: TodoWindowMode,
) -> Result<TodoWindowStatePayload, AppError> {
    apply_window_mode(&app, mode)
}

#[tauri::command]
fn list_todos(state: tauri::State<AppState>) -> Result<Vec<TodoItem>, AppError> {
    state
        .todo_store
        .lock()
        .map_err(|_| AppError::new("任务数据锁获取失败。"))?
        .list()
}

#[tauri::command]
fn add_todo(
    state: tauri::State<AppState>,
    input: TodoInput,
) -> Result<Vec<TodoItem>, AppError> {
    state
        .todo_store
        .lock()
        .map_err(|_| AppError::new("任务数据锁获取失败。"))?
        .add(input)
}

#[tauri::command]
fn update_todo(
    state: tauri::State<AppState>,
    id: String,
    input: TodoInput,
) -> Result<Vec<TodoItem>, AppError> {
    state
        .todo_store
        .lock()
        .map_err(|_| AppError::new("任务数据锁获取失败。"))?
        .update(&id, input)
}

#[tauri::command]
fn delete_todo(
    state: tauri::State<AppState>,
    id: String,
) -> Result<Vec<TodoItem>, AppError> {
    state
        .todo_store
        .lock()
        .map_err(|_| AppError::new("任务数据锁获取失败。"))?
        .delete(&id)
}

#[tauri::command]
fn set_todo_status(
    state: tauri::State<AppState>,
    id: String,
    status: TodoStatus,
) -> Result<Vec<TodoItem>, AppError> {
    state
        .todo_store
        .lock()
        .map_err(|_| AppError::new("任务数据锁获取失败。"))?
        .set_status(&id, status)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| AppError::new(error.to_string()))?;
            let todos_path = app_data_dir.join("todos.json");

            app.manage(AppState {
                todo_store: Mutex::new(TodoStore::new(todos_path)),
                window_state: Mutex::new(TodoWindowStatePayload {
                    mode: TodoWindowMode::Normal,
                }),
                is_quitting: Mutex::new(false),
            });

            let main_window = get_main_window(&app.handle())?;
            attach_window_events(&main_window);
            main_window.set_background_color(Some(hex_to_rgba(VISIBLE_WINDOW_BACKGROUND)?))?;
            setup_tray(&app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_window_mode,
            get_window_cursor_position,
            set_window_mode,
            list_todos,
            add_todo,
            update_todo,
            delete_todo,
            set_todo_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
