import { Button, Message } from "@arco-design/web-react";
import { IconDesktop, IconPalette } from "@arco-design/web-react/icon";
import { getErrorMessage, getTodoWindowApi } from "@/utils/api";
import { classNames } from "@/utils/class-name";
import styles from "./index.module.css";

type TitlebarProps = {
  unfinishedCount: number;
  view: "main" | "tray";
};

/**
 * Titlebar 渲染桌面窗口顶部拖拽栏。
 */
export function Titlebar({ unfinishedCount, view }: TitlebarProps) {
  const isTrayView = view === "tray";

  async function expandToMainWindow() {
    const todoWindow = getTodoWindowApi();

    if (!todoWindow) {
      Message.error("窗口控制能力暂不可用，无法打开主窗口。");
      return;
    }

    try {
      await todoWindow.showMainWindow();
    } catch (error) {
      Message.error(getErrorMessage(error, "主窗口打开失败。"));
    }
  }

  return (
    <div
      className={classNames(styles.titlebar, isTrayView && styles.compact)}
      data-tauri-drag-region
    >
      <span className={styles.brand} data-tauri-drag-region>
        <IconPalette className={styles.brandIcon} aria-hidden="true" />
        Todo List
        {isTrayView ? (
          <span className={styles.metric}>未完成 {unfinishedCount}</span>
        ) : null}
      </span>

      {isTrayView ? (
        <div className={styles.actions}>
          <Button
            aria-label="打开主窗口"
            className={styles.actionButton}
            htmlType="button"
            icon={<IconDesktop />}
            size="mini"
            title="打开主窗口"
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => void expandToMainWindow()}
          />
        </div>
      ) : null}
    </div>
  );
}
