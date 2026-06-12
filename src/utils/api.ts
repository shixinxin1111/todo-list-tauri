import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

const WINDOW_MODE_CHANGED_EVENT = "todo-window:mode-changed";

const todoWindowApi: TodoWindowApi = {
  getMode() {
    return invoke<TodoWindowState>("get_window_mode");
  },
  onModeChange(callback) {
    let detach: UnlistenFn | null = null;
    let isDisposed = false;

    void listen<TodoWindowState>(
      WINDOW_MODE_CHANGED_EVENT,
      (event) => {
        callback(event.payload);
      },
    ).then((unlisten) => {
      if (isDisposed) {
        unlisten();
        return;
      }

      detach = unlisten;
    });

    return () => {
      isDisposed = true;
      detach?.();
    };
  },
  setMode(mode) {
    return invoke<TodoWindowState>("set_window_mode", { mode });
  },
};

const todoStoreApi: TodoStoreApi = {
  add(input) {
    return invoke<TodoItem[]>("add_todo", { input });
  },
  list() {
    return invoke<TodoItem[]>("list_todos");
  },
  remove(id) {
    return invoke<TodoItem[]>("delete_todo", { id });
  },
  setStatus(id, status) {
    return invoke<TodoItem[]>("set_todo_status", { id, status });
  },
  update(id, input) {
    return invoke<TodoItem[]>("update_todo", { id, input });
  },
};

export function getTodoWindowApi() {
  return todoWindowApi;
}

export function getTodoStoreApi() {
  return todoStoreApi;
}

/**
 * getErrorMessage 将未知错误转换为用户可展示的兜底文案。
 *
 * 只有标准 Error 且 message 非空时才透出原始信息，否则返回调用方提供的
 * 业务上下文文案。
 */
export function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error && error.message ? error.message : fallback;
}
