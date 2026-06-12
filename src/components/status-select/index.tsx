import { Select } from "@arco-design/web-react";
import { todoStatusOptions } from "@/constants/todo";
import { classNames } from "@/utils/class-name";
import styles from "./index.module.css";

type StatusSelectProps = {
  disabled: boolean;
  extraClassName?: string;
  onChange(status: TodoStatus): void;
  status: TodoStatus;
};

/**
 * getStatusOptionClass 返回下拉选项文案对应的状态色样式。
 */
function getStatusOptionClass(status: TodoStatus) {
  if (status === "notStarted") {
    return styles.optionNotStarted;
  }

  if (status === "inProgress") {
    return styles.optionInProgress;
  }

  return styles.optionCompleted;
}

/**
 * StatusSelect 渲染可直接切换任务状态的 mini 下拉框。
 *
 * 组件内部统一维护三种状态的视觉颜色，保证主窗口表格和悬浮窗列表的状态入口一致。
 */
export function StatusSelect({
  disabled,
  extraClassName,
  onChange,
  status,
}: StatusSelectProps) {
  const coloredStatusOptions = todoStatusOptions.map((option) => ({
    ...option,
    label: (
      <span
        className={classNames(
          styles.optionLabel,
          getStatusOptionClass(option.value),
        )}
      >
        {option.label}
      </span>
    ),
  }));

  return (
    <Select
      className={classNames(
        styles.select,
        status === "notStarted" && styles.notStarted,
        status === "inProgress" && styles.inProgress,
        status === "completed" && styles.completed,
        extraClassName,
      )}
      disabled={disabled}
      dropdownMenuClassName={styles.dropdown}
      options={coloredStatusOptions}
      size="mini"
      value={status}
      onChange={(nextStatus) => onChange(nextStatus as TodoStatus)}
    />
  );
}
