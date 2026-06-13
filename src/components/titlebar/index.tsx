import { useEffect, useRef, useState } from "react";
import { Button } from "@arco-design/web-react";
import {
  IconDesktop,
  IconMinus,
  IconPalette,
  IconPushpin,
} from "@arco-design/web-react/icon";
import { getTodoWindowApi } from "@/utils/api";
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
  const actionRefs = useRef<Partial<Record<TodoWindowMode, HTMLDivElement | null>>>({});
  const [manualHoveredMode, setManualHoveredMode] = useState<TodoWindowMode | null>(null);
  const [isWindowFocused, setIsWindowFocused] = useState(() => document.hasFocus());

  useEffect(() => {
    function handleFocus() {
      setIsWindowFocused(true);
      setManualHoveredMode(null);
    }

    function handleBlur() {
      setIsWindowFocused(false);
    }

    window.addEventListener("focus", handleFocus);
    window.addEventListener("blur", handleBlur);

    return () => {
      window.removeEventListener("focus", handleFocus);
      window.removeEventListener("blur", handleBlur);
    };
  }, []);

  useEffect(() => {
    if (isWindowFocused) {
      return;
    }

    const todoWindowApi = getTodoWindowApi();
    let isDisposed = false;

    async function syncInactiveHover() {
      try {
        const cursorPosition = await todoWindowApi.getCursorPosition();

        if (isDisposed || !cursorPosition) {
          if (!isDisposed) {
            setManualHoveredMode(null);
          }
          return;
        }

        const nextHoveredMode =
          windowModeActions
            .filter((action) => action.mode !== windowMode)
            .find((action) => {
              const element = actionRefs.current[action.mode];

              if (!element) {
                return false;
              }

              const rect = element.getBoundingClientRect();

              return (
                cursorPosition.x >= rect.left &&
                cursorPosition.x <= rect.right &&
                cursorPosition.y >= rect.top &&
                cursorPosition.y <= rect.bottom
              );
            })?.mode ?? null;

        if (!isDisposed) {
          setManualHoveredMode(nextHoveredMode);
        }
      } catch {
        if (!isDisposed) {
          setManualHoveredMode(null);
        }
      }
    }

    void syncInactiveHover();
    const timer = window.setInterval(() => {
      void syncInactiveHover();
    }, 80);

    return () => {
      isDisposed = true;
      window.clearInterval(timer);
    };
  }, [isWindowFocused, windowMode, windowModeActions]);

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
            <div
              className={classNames(
                styles.windowActionSlot,
                !isWindowFocused &&
                  manualHoveredMode === action.mode &&
                  styles.windowActionSlotHovered,
              )}
              key={action.mode}
              ref={(element) => {
                actionRefs.current[action.mode] = element;
              }}
            >
              <Button
                aria-label={`进入${action.label}`}
                className={styles.windowAction}
                disabled={isBusy}
                htmlType="button"
                icon={action.icon}
                size="mini"
                title={`进入${action.label}`}
                onClick={() => onWindowModeChange(action.mode)}
              />
            </div>
          ))}
      </div>
    </div>
  );
}
