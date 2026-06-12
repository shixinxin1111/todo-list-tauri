import { useEffect, useState } from "react";
import { Message } from "@arco-design/web-react";
import { initialWindowState } from "@/constants/todo";
import { getErrorMessage, getTodoWindowApi } from "@/utils/api";

/**
 * useWindowState 负责同步 Electron 主进程维护的窗口形态。
 *
 * 这个 hook 封装窗口模式读取、订阅、切换以及 body data 标记，组件层只消费
 * 当前窗口状态和动作函数，不直接接触 preload API。
 */
export function useWindowState() {
  const [windowState, setWindowState] =
    useState<TodoWindowState>(initialWindowState);
  const [isBusy, setIsBusy] = useState(false);

  useEffect(() => {
    let isMounted = true;
    const todoWindow = getTodoWindowApi();

    if (!todoWindow) {
      Message.error("窗口控制能力暂不可用，基础页面已保留。");

      return () => {
        isMounted = false;
      };
    }

    void todoWindow
      .getMode()
      .then((state) => {
        if (isMounted) {
          setWindowState(state);
        }
      })
      .catch((error) => {
        if (isMounted) {
          Message.error(getErrorMessage(error, "窗口状态读取失败。"));
        }
      });

    const unsubscribe = todoWindow.onModeChange((state) => {
      if (isMounted) {
        setWindowState(state);
      }
    });

    return () => {
      isMounted = false;
      unsubscribe();
    };
  }, []);

  useEffect(() => {
    document.body.dataset.windowMode = windowState.mode;
    document.body.dataset.floatingSize =
      windowState.mode === "miniFloating" ? "collapsed" : "expanded";

    return () => {
      delete document.body.dataset.windowMode;
      delete document.body.dataset.floatingSize;
    };
  }, [windowState.mode]);

  async function changeWindowMode(mode: TodoWindowMode) {
    const todoWindow = getTodoWindowApi();

    if (!todoWindow) {
      Message.error("窗口控制能力暂不可用，无法切换形态。");
      return;
    }

    setIsBusy(true);

    try {
      const nextState = await todoWindow.setMode(mode);
      setWindowState(nextState);
    } catch (error) {
      Message.error(getErrorMessage(error, "窗口形态切换失败。"));
    } finally {
      setIsBusy(false);
    }
  }

  return {
    changeWindowMode,
    isBusy,
    windowState,
  };
}
