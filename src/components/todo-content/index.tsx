import { Typography } from "@arco-design/web-react";
import { classNames } from "@/utils/class-name";
import styles from "./index.module.css";

type TodoContentProps = {
  extraClassName?: string;
  isDone?: boolean;
  isFloating?: boolean;
  showNote: boolean;
  todo: TodoItem;
};

/**
 * TodoContent 负责展示任务标题和可选备注。
 *
 * 组件内部统一使用 Typography.Ellipsis 保证长文本溢出有 tooltip，
 * 外层只需要决定是否展示备注以及是否处于悬浮窗紧凑模式。
 */
export function TodoContent({
  extraClassName,
  isDone = false,
  isFloating = false,
  showNote,
  todo,
}: TodoContentProps) {
  return (
    <div
      className={classNames(
        styles.body,
        isFloating && styles.floating,
        extraClassName,
      )}
    >
      <Typography.Ellipsis
        className={classNames(
          styles.title,
          isFloating && styles.floatingTitle,
          isDone && styles.done,
          todo.status === "notStarted" && styles.notStarted,
          todo.status === "inProgress" && styles.inProgress,
        )}
        showTooltip
        rows={1}
      >
        {todo.title}
      </Typography.Ellipsis>
      {showNote && todo.note ? (
        <Typography.Ellipsis className={styles.note} showTooltip rows={2}>
          {todo.note}
        </Typography.Ellipsis>
      ) : null}
    </div>
  );
}
