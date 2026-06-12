/**
 * TodoModalMode 描述任务弹窗当前承担的业务动作。
 *
 * create 用于新增任务，只填写标题和备注；edit 用于编辑已有任务，
 * 会额外展示状态选择器并在提交时更新当前任务。
 */
export type TodoModalMode = "create" | "edit";

/**
 * TodoFilters 描述主窗口任务列表的筛选条件。
 *
 * keyword 作用于任务标题的本地模糊搜索，statuses 表示允许展示的
 * 任务状态集合；statuses 为空时代表不过滤状态。
 */
export type TodoFilters = {
  keyword: string;
  statuses: TodoStatus[];
};

/**
 * TodoCounts 是任务状态统计结果，用于主窗口任务卡片顶部的数据标签。
 */
export type TodoCounts = {
  completed: number;
  inProgress: number;
  notStarted: number;
};
