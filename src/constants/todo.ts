import type { TodoFilters } from "@/types/app";

/**
 * initialWindowState 是渲染进程在主进程状态返回前使用的窗口默认形态。
 *
 * 这里保持普通主窗口 + 悬浮展开的组合，避免首次渲染时误进入收起态。
 */
export const initialWindowState: TodoWindowState = {
  mode: "normal",
};

/**
 * defaultDraft 是新增或编辑弹窗关闭后恢复的表单默认值。
 *
 * 新增任务时不会主动提交 status，存储层会兜底使用 notStarted。
 */
export const defaultDraft: TodoInput = {
  note: "",
  status: "notStarted",
  title: "",
};

/**
 * defaultFilters 是主窗口任务列表的默认筛选条件。
 *
 * 默认只展示未完成状态，和用户手动点击搜索栏重置按钮后的状态保持一致。
 */
export const defaultFilters: TodoFilters = {
  keyword: "",
  statuses: ["notStarted", "inProgress"],
};

/**
 * todoStatusOptions 是所有状态选择控件共享的选项源。
 *
 * 统一维护可以保证表格筛选、弹窗状态选择和状态操作按钮使用同一组状态。
 */
export const todoStatusOptions: Array<{ label: string; value: TodoStatus }> = [
  { label: "未开始", value: "notStarted" },
  { label: "进行中", value: "inProgress" },
  { label: "已完成", value: "completed" },
];
