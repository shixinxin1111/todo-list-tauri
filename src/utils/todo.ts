import type { TodoCounts, TodoFilters } from "@/types/app";

/**
 * isActiveTodo 判断任务是否属于“未完成”集合。
 *
 * 悬浮窗只展示未开始和进行中的任务，已完成任务不会进入悬浮窗列表。
 */
export function isActiveTodo(todo: TodoItem) {
  return todo.status === "notStarted" || todo.status === "inProgress";
}

/**
 * filterTodos 根据主窗口筛选条件计算可见任务列表。
 *
 * 该函数保持纯函数形式，便于组件通过 useMemo 复用结果，同时避免把筛选细节散落到 UI。
 */
export function filterTodos(todos: TodoItem[], filters: TodoFilters) {
  const keyword = filters.keyword.trim().toLocaleLowerCase();

  return todos.filter((todo) => {
    const matchesKeyword =
      !keyword || todo.title.toLocaleLowerCase().includes(keyword);
    const matchesStatus =
      filters.statuses.length === 0 || filters.statuses.includes(todo.status);

    return matchesKeyword && matchesStatus;
  });
}

/**
 * getTodoCounts 汇总全部任务在各状态下的数量。
 *
 * 统计口径基于完整任务集合，而不是当前筛选后的可见任务集合。
 */
export function getTodoCounts(todos: TodoItem[]): TodoCounts {
  return todos.reduce<TodoCounts>(
    (counts, todo) => ({
      completed: counts.completed + (todo.status === "completed" ? 1 : 0),
      inProgress: counts.inProgress + (todo.status === "inProgress" ? 1 : 0),
      notStarted: counts.notStarted + (todo.status === "notStarted" ? 1 : 0),
    }),
    {
      completed: 0,
      inProgress: 0,
      notStarted: 0,
    },
  );
}
