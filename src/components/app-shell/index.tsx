import type { ReactNode } from "react";
import { classNames } from "@/utils/class-name";
import styles from "./index.module.css";

type AppShellProps = {
  children: ReactNode;
  isFloating: boolean;
  isFloatingCollapsed: boolean;
  titlebar: ReactNode;
};

/**
 * AppShell 渲染应用最外层窗口布局。
 *
 * 该组件集中管理普通窗口、悬浮窗和收起态的根布局样式，业务内容通过 children 注入。
 */
export function AppShell({
  children,
  isFloating,
  isFloatingCollapsed,
  titlebar,
}: AppShellProps) {
  return (
    <main
      className={classNames(
        styles.app,
        isFloating && styles.floating,
        isFloatingCollapsed && styles.collapsed,
      )}
    >
      <section className={styles.panel}>
        {titlebar}

        {isFloatingCollapsed ? null : (
          <div className={styles.content}>
            <div className={styles.contentInner}>{children}</div>
          </div>
        )}
      </section>
    </main>
  );
}
