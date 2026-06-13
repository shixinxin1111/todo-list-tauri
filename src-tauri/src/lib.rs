#![cfg_attr(target_os = "macos", allow(unexpected_cfgs))]

use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use uuid::Uuid;

/// 托盘窗口刚被显示后的“静默期”：在该时间窗口内不响应失焦事件，
/// 用于规避 macOS 点击 tray 图标瞬间的 key 焦点抖动。
const TRAY_FOCUS_DEBOUNCE: Duration = Duration::from_millis(250);

/// 鼠标在托盘图标上按下后的“保护期”：在该时间窗口内全局鼠标监听不应
/// 隐藏托盘窗，避免出现 “按下时隐藏、松手时 toggle 又显示” 的闪烁。
const TRAY_MOUSE_DOWN_GUARD: Duration = Duration::from_millis(500);

const TODOS_CHANGED_EVENT: &str = "todo-store:changed";
const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_WINDOW_LABEL: &str = "tray";
const TRAY_TOGGLE_WINDOW: &str = "toggle-tray-window";
const TRAY_SHOW_MAIN: &str = "show-main-window";
const TRAY_QUIT: &str = "quit-app";

const APP_TITLE: &str = "Todo List App";
const VISIBLE_WINDOW_BACKGROUND: &str = "#f4f1ea";
const TRAY_WINDOW_WIDTH: f64 = 390.0;
const TRAY_WINDOW_HEIGHT: f64 = 600.0;
const TRAY_WINDOW_MARGIN: f64 = 4.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TodoStatus {
    NotStarted,
    InProgress,
    Completed,
}

impl PartialEq for TodoStatus {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for TodoStatus {}

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
    is_quitting: Mutex<bool>,
    /// 缓存最近一次托盘图标的屏幕矩形，菜单项触发时用它定位托盘窗口。
    last_tray_rect: Mutex<Option<Rect>>,
    /// 托盘窗口最近一次被显示的时间，用于失焦防抖。
    tray_window_shown_at: Mutex<Option<Instant>>,
    /// 鼠标在托盘图标上按下的时间。在该时刻 ~500ms 内全局鼠标监听不应隐藏
    /// 托盘窗，否则会出现 “按下隐藏 → 松手又显示” 的闪烁。
    tray_mouse_down_at: Mutex<Option<Instant>>,
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

fn get_main_window(app: &AppHandle) -> Result<WebviewWindow, AppError> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| AppError::new("主窗口不存在。"))
}

fn get_tray_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(TRAY_WINDOW_LABEL)
}

fn hide_tray_window(app: &AppHandle) -> Result<(), AppError> {
    if let Some(window) = get_tray_window(app) {
        if window.is_visible()? {
            window.hide()?;
        }
    }

    Ok(())
}

fn focus_main_window(app: &AppHandle) -> Result<(), AppError> {
    let window = get_main_window(app)?;
    hide_tray_window(app)?;

    if window.is_minimized()? {
        window.unminimize()?;
    }

    set_traffic_lights_hidden(&window, false);
    window.show()?;
    window.set_focus()?;
    Ok(())
}

fn get_tray_window_position(
    tray_rect: Rect,
    monitor_position: LogicalPosition<f64>,
    monitor_size: LogicalSize<f64>,
    scale_factor: f64,
) -> LogicalPosition<f64> {
    let tray_position = tray_rect.position.to_logical::<f64>(scale_factor);
    let tray_size = tray_rect.size.to_logical::<f64>(scale_factor);
    let work_min_x = monitor_position.x;
    let work_max_x = monitor_position.x + monitor_size.width;
    let work_min_y = monitor_position.y;
    let target_x = (tray_position.x + (tray_size.width - TRAY_WINDOW_WIDTH) / 2.0)
        .clamp(
            work_min_x + TRAY_WINDOW_MARGIN,
            (work_max_x - TRAY_WINDOW_WIDTH - TRAY_WINDOW_MARGIN).max(work_min_x),
        );
    let target_y = (tray_position.y + tray_size.height + TRAY_WINDOW_MARGIN).max(work_min_y);

    LogicalPosition::new(target_x, target_y)
}

fn create_tray_window(app: &AppHandle) -> Result<WebviewWindow, AppError> {
    if let Some(window) = get_tray_window(app) {
        return Ok(window);
    }

    let window = WebviewWindowBuilder::new(
        app,
        TRAY_WINDOW_LABEL,
        WebviewUrl::App("index.html?view=tray".into()),
    )
    .title(APP_TITLE)
    .inner_size(TRAY_WINDOW_WIDTH, TRAY_WINDOW_HEIGHT)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .closable(true)
    .fullscreen(false)
    .decorations(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    // 透明窗口开启后，NSWindow 的阴影会沿着内容 alpha 形状（即圆角）投射，
    // 圆角阴影由系统直接绘制；CSS 不再叠加 box-shadow，避免被窗口矩形边界裁切。
    .shadow(true)
    .focused(true)
    .visible(false)
    .transparent(true)
    .build()?;

    // 透明窗口下，背景色完全交给 webview 的 CSS（含圆角），
    // 这里不再调用 set_background_color，避免覆盖透明通道。
    attach_tray_window_events(&window);
    Ok(window)
}

fn toggle_tray_window(app: &AppHandle, tray_rect: Rect) -> Result<(), AppError> {
    cache_tray_rect(app, tray_rect);
    let window = create_tray_window(app)?;

    if window.is_visible()? {
        window.hide()?;
        clear_tray_shown_at(app);
        return Ok(());
    }

    let monitor = get_main_window(app)
        .ok()
        .and_then(|window| window.current_monitor().ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten())
        .ok_or_else(|| AppError::new("无法获取当前显示器信息。"))?;
    let scale_factor = monitor.scale_factor();
    let monitor_position = monitor.position().to_logical::<f64>(scale_factor);
    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    let tray_position =
        get_tray_window_position(tray_rect, monitor_position, monitor_size, scale_factor);

    set_traffic_lights_hidden(&window, true);
    window.set_position(tray_position)?;
    window.set_size(LogicalSize::new(TRAY_WINDOW_WIDTH, TRAY_WINDOW_HEIGHT))?;
    window.show()?;
    mark_tray_shown_now(app);
    window.set_focus()?;

    Ok(())
}

fn cache_tray_rect(app: &AppHandle, rect: Rect) {
    if let Ok(mut last) = app.state::<AppState>().last_tray_rect.lock() {
        *last = Some(rect);
    }
}

fn get_cached_tray_rect(app: &AppHandle) -> Option<Rect> {
    app.state::<AppState>()
        .last_tray_rect
        .lock()
        .ok()
        .and_then(|guard| *guard)
}

fn mark_tray_shown_now(app: &AppHandle) {
    if let Ok(mut shown_at) = app.state::<AppState>().tray_window_shown_at.lock() {
        *shown_at = Some(Instant::now());
    }
}

fn clear_tray_shown_at(app: &AppHandle) {
    if let Ok(mut shown_at) = app.state::<AppState>().tray_window_shown_at.lock() {
        *shown_at = None;
    }
}

fn is_tray_window_in_debounce(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .tray_window_shown_at
        .lock()
        .ok()
        .and_then(|guard| *guard)
        .map(|shown_at| shown_at.elapsed() < TRAY_FOCUS_DEBOUNCE)
        .unwrap_or(false)
}

fn mark_tray_mouse_down(app: &AppHandle) {
    if let Ok(mut guard) = app.state::<AppState>().tray_mouse_down_at.lock() {
        *guard = Some(Instant::now());
    }
}

fn is_tray_mouse_down_guard_active(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .tray_mouse_down_at
        .lock()
        .ok()
        .and_then(|guard| *guard)
        .map(|at| at.elapsed() < TRAY_MOUSE_DOWN_GUARD)
        .unwrap_or(false)
}

fn emit_todos_changed(app: &AppHandle, todos: &[TodoItem]) -> Result<(), AppError> {
    app.emit(TODOS_CHANGED_EVENT, todos)?;
    update_tray(app, todos)?;
    Ok(())
}

fn with_todo_store_mutation(
    app: &AppHandle,
    state: tauri::State<AppState>,
    mutate: impl FnOnce(&TodoStore) -> Result<Vec<TodoItem>, AppError>,
) -> Result<Vec<TodoItem>, AppError> {
    let todos = {
        let store = state
            .todo_store
            .lock()
            .map_err(|_| AppError::new("任务数据锁获取失败。"))?;
        mutate(&store)?
    };

    emit_todos_changed(app, &todos)?;
    Ok(todos)
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
#[allow(deprecated, unexpected_cfgs)]
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

/// macOS 上 `Focused(false)` 不能可靠覆盖“点击菜单栏空白 / 系统菜单 / 其他
/// 状态项”这类场景：点击菜单栏不会切换 key window，托盘窗收不到失焦事件。
/// 这里参考 Electron 版的实现（`NSMenuDidBeginTrackingNotification` + 全局
/// 鼠标监听）补全：
/// 1. 任何菜单开始 tracking 时主动隐藏（覆盖系统菜单 / 应用菜单 / 状态项菜单）
/// 2. 全局 MouseDown 事件 + 命中菜单栏区域时主动隐藏（覆盖菜单栏空白处点击）
///
/// 注意：`addObserverForName:` 与 `addGlobalMonitorForEventsMatchingMask:` 都
/// 返回需要持有的 opaque token，必须 leak / forget 让其常驻进程生命周期，
/// 否则会被 ARC 立即释放、回调不再触发。
#[cfg(target_os = "macos")]
#[allow(deprecated, unexpected_cfgs)]
fn observe_menu_bar_clicks(app: &AppHandle) {
    use block::ConcreteBlock;
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSPoint, NSRect, NSString};
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    // 1) 任何菜单开始 tracking 时隐藏托盘窗
    {
        let app_handle = app.clone();
        let block = ConcreteBlock::new(move |_note: *mut Object| {
            let _ = hide_tray_window(&app_handle);
        });
        let block = block.copy();

        unsafe {
            let center: id = msg_send![class!(NSNotificationCenter), defaultCenter];
            let name = NSString::alloc(nil).init_str("NSMenuDidBeginTrackingNotification");
            // 必须保留 observer token，否则会被释放
            let observer: id = msg_send![
                center,
                addObserverForName: name
                object: nil
                queue: nil
                usingBlock: &*block
            ];
            std::mem::forget(block);
            let _ = Box::leak(Box::new(observer));
        }
    }

    // 2) 全局鼠标点击监听，命中菜单栏区域则隐藏托盘窗
    {
        let app_handle = app.clone();
        let block = ConcreteBlock::new(move |event: *mut Object| unsafe {
            if event.is_null() {
                return;
            }
            // 鼠标在托盘图标上按下时，全局监听不应隐藏托盘窗——否则按下时
            // hide、松手时 toggle 又 show，体验上变成无法关闭。
            if is_tray_mouse_down_guard_active(&app_handle) {
                return;
            }
            let screens: id = msg_send![class!(NSScreen), screens];
            let count: usize = msg_send![screens, count];
            if count == 0 {
                return;
            }
            let main_screen: id = msg_send![screens, objectAtIndex: 0usize];
            let frame: NSRect = msg_send![main_screen, frame];
            let visible: NSRect = msg_send![main_screen, visibleFrame];
            let menu_bar_height =
                frame.size.height - (visible.origin.y - frame.origin.y + visible.size.height);
            if menu_bar_height <= 0.0 {
                return;
            }
            let menu_bar_top = frame.origin.y + frame.size.height;
            let menu_bar_bottom = menu_bar_top - menu_bar_height;

            let location: NSPoint = msg_send![class!(NSEvent), mouseLocation];
            if location.y < menu_bar_bottom || location.y > menu_bar_top {
                return;
            }
            if location.x < frame.origin.x || location.x > frame.origin.x + frame.size.width {
                return;
            }

            // 排除点在 Todo 自己托盘图标上的情况，那里由 tray click handler toggle
            let cached_rect = app_handle
                .state::<AppState>()
                .last_tray_rect
                .lock()
                .ok()
                .and_then(|guard| guard.clone());
            if let Some(rect) = cached_rect {
                let scale: f64 = msg_send![main_screen, backingScaleFactor];
                let scale = if scale <= 0.0 { 1.0 } else { scale };
                let pos = rect.position.to_logical::<f64>(scale);
                let size = rect.size.to_logical::<f64>(scale);
                let icon_top = frame.origin.y + frame.size.height - pos.y;
                let icon_bottom = icon_top - size.height;
                let icon_left = pos.x;
                let icon_right = pos.x + size.width;
                if location.x >= icon_left
                    && location.x <= icon_right
                    && location.y <= icon_top
                    && location.y >= icon_bottom
                {
                    return;
                }
            }

            let _ = hide_tray_window(&app_handle);
        });
        let block = block.copy();

        unsafe {
            // NSEventMaskLeftMouseDown(1<<1) | NSEventMaskRightMouseDown(1<<3)
            //   | NSEventMaskOtherMouseDown(1<<25)
            let mask: u64 = (1u64 << 1) | (1u64 << 3) | (1u64 << 25);
            // 必须保留 monitor token，否则会被释放
            let monitor: id = msg_send![
                class!(NSEvent),
                addGlobalMonitorForEventsMatchingMask: mask
                handler: &*block
            ];
            std::mem::forget(block);
            let _ = Box::leak(Box::new(monitor));
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn observe_menu_bar_clicks(_app: &AppHandle) {}

fn update_tray(app: &AppHandle, todos: &[TodoItem]) -> Result<(), AppError> {
    let Some(tray) = app.tray_by_id("todo-tray") else {
        return Ok(());
    };
    let active_count = todos
        .iter()
        .filter(|todo| todo.status != TodoStatus::Completed)
        .count();

    tray.set_tooltip(Some(format!("Todo List App - 未完成 {} 项", active_count)))?;
    #[cfg(target_os = "macos")]
    tray.set_title(Some(if active_count > 0 {
        active_count.to_string()
    } else {
        String::new()
    }))?;

    Ok(())
}

fn list_todos_for_tray(app: &AppHandle) -> Result<Vec<TodoItem>, AppError> {
    app.state::<AppState>()
        .todo_store
        .lock()
        .map_err(|_| AppError::new("任务数据锁获取失败。"))?
        .list()
}

fn setup_tray(app: &AppHandle) -> Result<(), AppError> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| AppError::new("无法读取默认应用图标。"))?;
    let menu = MenuBuilder::new(app)
        .text(TRAY_TOGGLE_WINDOW, "打开托盘清单")
        .text(TRAY_SHOW_MAIN, "显示主窗口")
        .separator()
        .text(TRAY_QUIT, "退出应用")
        .build()?;

    TrayIconBuilder::with_id("todo-tray")
        .icon(icon)
        .tooltip("Todo List App")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            TRAY_TOGGLE_WINDOW => {
                if let Some(rect) = resolve_tray_anchor_rect(app) {
                    let _ = toggle_tray_window(app, rect);
                }
            }
            TRAY_SHOW_MAIN => {
                let _ = focus_main_window(app);
            }
            TRAY_QUIT => {
                if let Ok(mut is_quitting) = app.state::<AppState>().is_quitting.lock() {
                    *is_quitting = true;
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            // 注意：tauri 2.x 的 `MouseButtonState` 注释表明
            // `Up` 表示按下（pressed），`Down` 表示松开（released）。
            // 这里依据语义而非字面名称使用。
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } => {
                cache_tray_rect(tray.app_handle(), rect);
                mark_tray_mouse_down(tray.app_handle());
            }
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                rect,
                ..
            } => {
                let _ = toggle_tray_window(tray.app_handle(), rect);
            }
            TrayIconEvent::Enter { rect, .. }
            | TrayIconEvent::Move { rect, .. }
            | TrayIconEvent::Leave { rect, .. } => {
                cache_tray_rect(tray.app_handle(), rect);
            }
            _ => {}
        })
        .build(app)?;

    let todos = list_todos_for_tray(app)?;
    update_tray(app, &todos)?;
    Ok(())
}

/// 解析托盘菜单项触发时应使用的锚点矩形：
/// 1. 优先使用最近一次缓存的 tray rect（来自鼠标进出 / 移动 / 点击事件）；
/// 2. 兜底使用主窗外框，避免完全无法定位的情况。
fn resolve_tray_anchor_rect(app: &AppHandle) -> Option<Rect> {
    if let Some(rect) = get_cached_tray_rect(app) {
        return Some(rect);
    }

    let main_window = get_main_window(app).ok()?;
    let position = main_window.outer_position().ok()?;
    let size = main_window.outer_size().ok()?;
    Some(Rect {
        position: position.into(),
        size: size.into(),
    })
}

fn attach_main_window_events(window: &WebviewWindow) {
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

            if should_hide {
                api.prevent_close();
                let _ = window_handle.hide();
            }
        }

        if let WindowEvent::Focused(true) = event {
            let _ = hide_tray_window(&app);
        }
    });
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), AppError> {
    focus_main_window(&app)
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
    app: AppHandle,
    state: tauri::State<AppState>,
    input: TodoInput,
) -> Result<Vec<TodoItem>, AppError> {
    with_todo_store_mutation(&app, state, |store| store.add(input))
}

#[tauri::command]
fn update_todo(
    app: AppHandle,
    state: tauri::State<AppState>,
    id: String,
    input: TodoInput,
) -> Result<Vec<TodoItem>, AppError> {
    with_todo_store_mutation(&app, state, |store| store.update(&id, input))
}

#[tauri::command]
fn delete_todo(
    app: AppHandle,
    state: tauri::State<AppState>,
    id: String,
) -> Result<Vec<TodoItem>, AppError> {
    with_todo_store_mutation(&app, state, |store| store.delete(&id))
}

#[tauri::command]
fn set_todo_status(
    app: AppHandle,
    state: tauri::State<AppState>,
    id: String,
    status: TodoStatus,
) -> Result<Vec<TodoItem>, AppError> {
    with_todo_store_mutation(&app, state, |store| store.set_status(&id, status))
}

fn attach_tray_window_events(window: &WebviewWindow) {
    let app = window.app_handle().clone();
    let window_handle = window.clone();
    window.clone().on_window_event(move |event| match event {
        WindowEvent::Focused(false) => {
            // macOS 在点击托盘图标的瞬间会将 key 焦点临时交给状态栏，
            // 刚 show 出来的托盘窗口会立刻收到一个 Focused(false)。
            // 通过短暂的防抖窗口忽略这一类抖动，避免窗口刚出现就被关掉。
            if is_tray_window_in_debounce(&app) {
                return;
            }
            // 若用户正在托盘图标上按下，意味着即将由 Click(Up) 自行 toggle。
            // 此时 macOS 会瞬时把 key 焦点交给状态栏触发 Focused(false)，
            // 若立刻 hide，松手时 toggle 会再次 show，出现闪烁。
            if is_tray_mouse_down_guard_active(&app) {
                return;
            }

            let _ = window_handle.hide();
            clear_tray_shown_at(&app);
        }
        WindowEvent::CloseRequested { api, .. } => {
            let should_hide = app
                .state::<AppState>()
                .is_quitting
                .lock()
                .ok()
                .map(|is_quitting| !*is_quitting)
                .unwrap_or(true);

            if should_hide {
                api.prevent_close();
                let _ = window_handle.hide();
                clear_tray_shown_at(&app);
            }
        }
        _ => {}
    });
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
                is_quitting: Mutex::new(false),
                last_tray_rect: Mutex::new(None),
                tray_window_shown_at: Mutex::new(None),
                tray_mouse_down_at: Mutex::new(None),
            });

            let main_window = get_main_window(&app.handle())?;
            attach_main_window_events(&main_window);
            main_window.set_background_color(Some(hex_to_rgba(VISIBLE_WINDOW_BACKGROUND)?))?;
            main_window.set_title(APP_TITLE)?;
            setup_tray(&app.handle())?;
            observe_menu_bar_clicks(&app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_main_window,
            list_todos,
            add_todo,
            update_todo,
            delete_todo,
            set_todo_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
