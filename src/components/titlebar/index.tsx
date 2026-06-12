import { Button } from "@arco-design/web-react";
import {
  IconDesktop,
  IconMinus,
  IconPalette,
  IconPushpin,
} from "@arco-design/web-react/icon";
import { classNames } from "@/utils/class-name";
import styles from "./index.module.css";

type TitlebarProps = {
  isBusy: boolean;
  isFloating: boolean;
  isFloatingCollapsed: boolean;
  onWindowModeChange(mode: TodoWindowMode): void;
  unfinishedCount: number;
  windowMode: TodoWindowMode;
};

/**
 * Titlebar 渲染桌面窗口顶部拖拽栏和窗口形态切换按钮。
 *
 * 组件只接收窗口状态和事件回调，真正的 Electron 窗口控制逻辑由 useWindowState 处理。
 */
export function Titlebar({
  isBusy,
  isFloating,
  isFloatingCollapsed,
  onWindowModeChange,
  unfinishedCount,
  windowMode,
}: TitlebarProps) {
  const windowModeActions = [
    {
      icon: <IconMinus />,
      label: "迷你悬浮窗",
      mode: "miniFloating",
    },
    {
      icon: <IconPushpin />,
      label: "悬浮窗",
      mode: "floating",
    },
    {
      icon: <IconDesktop />,
      label: "主窗",
      mode: "normal",
    },
  ] as const;

  return (
    <div
      className={classNames(styles.titlebar, isFloating && styles.floating)}
      data-tauri-drag-region
    >
      <span className={styles.brand} data-tauri-drag-region>
        <IconPalette className={styles.brandIcon} aria-hidden="true" />
        Todo List
        {isFloatingCollapsed ? (
          <span className={styles.metric}>未完成: {unfinishedCount}</span>
        ) : null}
      </span>

      <div className={styles.actions}>
        {windowModeActions
          .filter((action) => action.mode !== windowMode)
          .map((action) => (
            <Button
              aria-label={`进入${action.label}`}
              disabled={isBusy}
              htmlType="button"
              icon={action.icon}
              key={action.mode}
              size="mini"
              title={`进入${action.label}`}
              onClick={() => onWindowModeChange(action.mode)}
            />
          ))}
      </div>
    </div>
  );
}
