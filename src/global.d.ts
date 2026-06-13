type TodoWindowApi = {
  /**
   * showMainWindow 从托盘弹出窗回到主窗口。
   */
  showMainWindow(): Promise<void>;
};

type TodoStatus = "notStarted" | "inProgress" | "completed";

type TodoItem = {
  id: string;
  note: string;
  status: TodoStatus;
  title: string;
};

type TodoInput = {
  note: string;
  status?: TodoStatus;
  title: string;
};

type TodoStoreApi = {
  /**
   * list 读取保存在本地 JSON 文件中的全部任务。
   */
  list(): Promise<TodoItem[]>;
  /**
   * add 新增任务，标题必填，状态不传时默认为未开始。
   */
  add(input: TodoInput): Promise<TodoItem[]>;
  /**
   * update 更新任务标题、备注和状态。
   */
  update(id: string, input: TodoInput): Promise<TodoItem[]>;
  /**
   * remove 删除指定任务。
   */
  remove(id: string): Promise<TodoItem[]>;
  /**
   * setStatus 只更新任务状态，供悬浮窗轻量操作使用。
   */
  setStatus(id: string, status: TodoStatus): Promise<TodoItem[]>;
};

interface Window {
  todoWindow?: TodoWindowApi;
  todoStore?: TodoStoreApi;
}
