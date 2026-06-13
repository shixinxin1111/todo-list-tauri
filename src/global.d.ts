type TodoWindowMode = "normal" | "floating" | "miniFloating";

type TodoWindowState = {
  mode: TodoWindowMode;
};

type WindowCursorPosition = {
  x: number;
  y: number;
};

type TodoWindowApi = {
  /**
   * getMode 读取主进程记录的当前窗口形态，用于渲染层初始化同步。
   */
  getMode(): Promise<TodoWindowState>;
  /**
   * getCursorPosition 读取鼠标相对窗口内容区域的位置；当鼠标不在窗口内时返回 null。
   */
  getCursorPosition(): Promise<WindowCursorPosition | null>;
  /**
   * setMode 请求主进程切换窗口形态，实际窗口尺寸由主进程裁决。
   */
  setMode(mode: TodoWindowMode): Promise<TodoWindowState>;
  /**
   * onModeChange 订阅主进程形态变化，返回取消订阅函数。
   */
  onModeChange(callback: (state: TodoWindowState) => void): () => void;
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
