import { Card, Empty } from "@arco-design/web-react";
import { classNames } from "@/utils/class-name";
import { StatusIcon } from "@/components/status-icon";
import { StatusSelect } from "@/components/status-select";
import { TodoContent } from "@/components/todo-content";
import styles from "./index.module.css";

type FloatingListProps = {
  isTodoBusy: boolean;
  onStatusChange(id: string, status: TodoStatus): void;
  todos: TodoItem[];
};

type FloatingTodoItemProps = {
  isTodoBusy: boolean;
  onStatusChange(id: string, status: TodoStatus): void;
  todo: TodoItem;
};

/**
 * FloatingTodoItem 渲染悬浮窗中的单条未完成任务。
 *
 * 组件把状态展示和状态切换合并到固定右侧的 Select 中，避免 hover 时切换布局。
 */
function FloatingTodoItem({
  isTodoBusy,
  onStatusChange,
  todo,
}: FloatingTodoItemProps) {
  const isDone = todo.status === "completed";

  return (
    <li className={classNames(styles.item, isDone && styles.done)}>
      <StatusIcon isFloating status={todo.status} />
      <TodoContent
        extraClassName={styles.content}
        isDone={isDone}
        isFloating
        showNote={false}
        todo={todo}
      />
      <div className={styles.sideSlot}>
        <StatusSelect
          disabled={isTodoBusy}
          status={todo.status}
          onChange={(status) => onStatusChange(todo.id, status)}
        />
      </div>
    </li>
  );
}

/**
 * FloatingList 渲染悬浮窗中的未完成任务列表。
 *
 * 列表只接收父级已经筛选好的未完成任务，组件内部不再重复判断任务范围。
 */
export function FloatingList({
  isTodoBusy,
  onStatusChange,
  todos,
}: FloatingListProps) {
  return (
    <Card className={styles.card} bordered={false} aria-label="悬浮待办">
      <div className={styles.heading}>
        <div className={styles.headingTitle}>
          <span>未完成的清单</span>
          <strong>{todos.length} 项</strong>
        </div>
      </div>
      {todos.length > 0 ? (
        <ul className={styles.list}>
          {todos.map((todo) => (
            <FloatingTodoItem
              isTodoBusy={isTodoBusy}
              key={todo.id}
              todo={todo}
              onStatusChange={onStatusChange}
            />
          ))}
        </ul>
      ) : (
        <Empty className={styles.empty} description="当前没有待推进的任务。" />
      )}
    </Card>
  );
}
