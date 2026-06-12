/**
 * getStatusTagColor 把任务状态映射到 Arco Tag 的内建颜色。
 *
 * Tag 的文本色和背景色完全交给 Arco 的 color 属性控制，CSS Modules 只负责布局。
 */
export function getStatusTagColor(status: TodoStatus) {
  if (status === "completed") {
    return "green";
  }

  if (status === "inProgress") {
    return "arcoblue";
  }

  return "orange";
}
