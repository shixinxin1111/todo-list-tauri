import { Button, Popconfirm, Space } from "@arco-design/web-react";
import styles from "./index.module.css";

type TableActionsProps = {
  disabled: boolean;
  onDelete(id: string): void;
  onEdit(todo: TodoItem): void;
  todo: TodoItem;
};

/**
 * TableActions 渲染主窗口表格中的行级操作。
 *
 * 状态切换已经下沉到状态列 Select 中，这里只保留任务内容编辑和删除动作。
 */
export function TableActions({
  disabled,
  onDelete,
  onEdit,
  todo,
}: TableActionsProps) {
  return (
    <Space className={styles.actions} size={8}>
      <Button
        aria-label={`编辑 ${todo.title}`}
        className={styles.button}
        disabled={disabled}
        htmlType="button"
        size="mini"
        type="text"
        onClick={() => onEdit(todo)}
      >
        编辑
      </Button>
      <Popconfirm
        cancelText="取消"
        okText="删除"
        okButtonProps={{ status: "danger" }}
        position="left"
        title="确定删除这项任务吗？"
        onOk={() => onDelete(todo.id)}
      >
        <Button
          aria-label={`删除 ${todo.title}`}
          className={styles.button}
          disabled={disabled}
          htmlType="button"
          size="mini"
          status="danger"
          type="text"
        >
          删除
        </Button>
      </Popconfirm>
    </Space>
  );
}
