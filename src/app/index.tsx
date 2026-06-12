import { useMemo, useState } from "react";
import { AppShell } from "@/components/app-shell";
import { FloatingList } from "@/components/floating-list";
import { Titlebar } from "@/components/titlebar";
import { TodoModal } from "@/components/todo-modal";
import { TodoWorkspace } from "@/components/todo-workspace";
import { defaultDraft, defaultFilters } from "@/constants/todo";
import { useTodoStore } from "@/hooks/use-todo-store";
import { useWindowState } from "@/hooks/use-window-state";
import { filterTodos, getTodoCounts, isActiveTodo } from "@/utils/todo";
import type { TodoFilters, TodoModalMode } from "@/types/app";

/**
 * App 是 Todo 桌面应用的渲染进程根组件。
 *
 * 组件负责组合窗口状态、任务数据、筛选条件和弹窗编辑态；具体 UI 展示与副作用
 * 分别下沉到 src/components 与 src/hooks。
 */
export function App() {
  const [draft, setDraft] = useState<TodoInput>(defaultDraft);
  const [filters, setFilters] = useState<TodoFilters>(defaultFilters);
  const [modalMode, setModalMode] = useState<TodoModalMode | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const { changeWindowMode, isBusy, windowState } = useWindowState();
  const {
    addTodo,
    deleteTodo: removeTodo,
    isTodoBusy,
    setTodoStatus,
    todos,
    updateTodo,
  } = useTodoStore();

  const isFloating = windowState.mode !== "normal";
  const isFloatingCollapsed = windowState.mode === "miniFloating";

  const visibleTodos = useMemo(
    () => filterTodos(todos, filters),
    [filters, todos],
  );
  const floatingTodos = useMemo(() => todos.filter(isActiveTodo), [todos]);
  const todoCounts = useMemo(() => getTodoCounts(todos), [todos]);
  const unfinishedCount = floatingTodos.length;

  function openCreateModal() {
    setEditingId(null);
    setDraft(defaultDraft);
    setModalMode("create");
  }

  function openEditModal(todo: TodoItem) {
    setEditingId(todo.id);
    setDraft({
      note: todo.note,
      status: todo.status,
      title: todo.title,
    });
    setModalMode("edit");
  }

  function closeModal() {
    setModalMode(null);
    setEditingId(null);
    setDraft(defaultDraft);
  }

  async function submitTodo(input: TodoInput) {
    const nextTodos =
      modalMode === "edit" && editingId
        ? await updateTodo(editingId, input)
        : await addTodo(input);

    if (nextTodos) {
      closeModal();
    }
  }

  async function deleteTodo(todoId: string) {
    const nextTodos = await removeTodo(todoId);

    if (nextTodos && editingId === todoId) {
      closeModal();
    }
  }

  async function changeTodoStatus(todoId: string, status: TodoStatus) {
    await setTodoStatus(todoId, status);
  }

  return (
    <>
      <AppShell
        isFloating={isFloating}
        isFloatingCollapsed={isFloatingCollapsed}
        titlebar={
          <Titlebar
            isBusy={isBusy}
            isFloating={isFloating}
            isFloatingCollapsed={isFloatingCollapsed}
            unfinishedCount={unfinishedCount}
            windowMode={windowState.mode}
            onWindowModeChange={(mode) => void changeWindowMode(mode)}
          />
        }
      >
        {isFloating ? (
          <FloatingList
            isTodoBusy={isTodoBusy}
            todos={floatingTodos}
            onStatusChange={(todoId, status) =>
              void changeTodoStatus(todoId, status)
            }
          />
        ) : (
          <TodoWorkspace
            counts={todoCounts}
            filters={filters}
            isTodoBusy={isTodoBusy}
            todos={visibleTodos}
            totalTodoCount={todos.length}
            onCreate={openCreateModal}
            onDelete={(todoId) => void deleteTodo(todoId)}
            onEdit={openEditModal}
            onFiltersChange={setFilters}
            onStatusChange={(todoId, status) =>
              void changeTodoStatus(todoId, status)
            }
          />
        )}
      </AppShell>

      <TodoModal
        draft={draft}
        isBusy={isTodoBusy}
        isFloating={isFloating}
        mode={modalMode}
        onCancel={closeModal}
        onSubmit={(input) => void submitTodo(input)}
      />
    </>
  );
}
