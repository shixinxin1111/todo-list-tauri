import { useEffect, useState } from "react";
import { Message } from "@arco-design/web-react";
import { getErrorMessage, getTodoStoreApi, onTodosChange } from "@/utils/api";

/**
 * useTodoStore 负责渲染进程侧的任务数据读取与写入编排。
 *
 * hook 内部统一处理 loading、错误提示和写入后的列表回填，组件层只通过语义化
 * 的 add/update/delete/setStatus 方法表达业务意图。
 */
export function useTodoStore() {
  const [todos, setTodos] = useState<TodoItem[]>([]);
  const [isTodoBusy, setIsTodoBusy] = useState(false);

  useEffect(() => {
    let isMounted = true;
    const todoStore = getTodoStoreApi();

    if (!todoStore) {
      Message.error("任务数据能力暂不可用。");

      return () => {
        isMounted = false;
      };
    }

    void todoStore
      .list()
      .then((nextTodos) => {
        if (isMounted) {
          setTodos(nextTodos);
        }
      })
      .catch((error) => {
        if (isMounted) {
          Message.error(getErrorMessage(error, "任务读取失败。"));
        }
      });

    const unsubscribe = onTodosChange((nextTodos) => {
      if (isMounted) {
        setTodos(nextTodos);
      }
    });

    return () => {
      isMounted = false;
      unsubscribe();
    };
  }, []);

  async function runTodoOperation(
    operation: (todoStore: TodoStoreApi) => Promise<TodoItem[]>,
  ) {
    const todoStore = getTodoStoreApi();

    if (!todoStore) {
      Message.error("任务数据能力暂不可用。");
      return null;
    }

    setIsTodoBusy(true);

    try {
      const nextTodos = await operation(todoStore);
      setTodos(nextTodos);
      return nextTodos;
    } catch (error) {
      Message.error(getErrorMessage(error, "任务操作失败。"));
      return null;
    } finally {
      setIsTodoBusy(false);
    }
  }

  function addTodo(input: TodoInput) {
    return runTodoOperation((todoStore) =>
      todoStore.add({
        note: input.note,
        title: input.title,
      }),
    );
  }

  function updateTodo(id: string, input: TodoInput) {
    return runTodoOperation((todoStore) => todoStore.update(id, input));
  }

  function deleteTodo(id: string) {
    return runTodoOperation((todoStore) => todoStore.remove(id));
  }

  function setTodoStatus(id: string, status: TodoStatus) {
    return runTodoOperation((todoStore) => todoStore.setStatus(id, status));
  }

  return {
    addTodo,
    deleteTodo,
    isTodoBusy,
    setTodoStatus,
    todos,
    updateTodo,
  };
}
