import {
  IconCheckCircleFill,
  IconClockCircle,
  IconPlayCircleFill,
} from "@arco-design/web-react/icon";
import { classNames } from "@/utils/class-name";
import styles from "./index.module.css";

type StatusIconProps = {
  isFloating?: boolean;
  status: TodoStatus;
};

/**
 * StatusIcon 展示任务状态左侧的语义图标。
 *
 * 组件只根据任务状态选择图标和颜色；悬浮窗通过 isFloating 缩小图标尺寸。
 */
export function StatusIcon({ isFloating = false, status }: StatusIconProps) {
  const className = classNames(
    styles.icon,
    isFloating && styles.floating,
    status === "notStarted" && styles.notStarted,
    status === "inProgress" && styles.inProgress,
    status === "completed" && styles.completed,
  );

  if (status === "completed") {
    return <IconCheckCircleFill className={className} aria-hidden="true" />;
  }

  if (status === "inProgress") {
    return <IconPlayCircleFill className={className} aria-hidden="true" />;
  }

  return <IconClockCircle className={className} aria-hidden="true" />;
}
