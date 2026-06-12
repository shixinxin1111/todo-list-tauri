import { Card, Empty, Table, Tag } from "@arco-design/web-react";
import type { TableColumnProps } from "@arco-design/web-react";
import { useEffect, useRef, useState } from "react";
import { defaultFilters } from "@/constants/todo";
import { StatusIcon } from "@/components/status-icon";
import { StatusSelect } from "@/components/status-select";
import { TableActions } from "@/components/table-actions";
import { TodoContent } from "@/components/todo-content";
import { Toolbar } from "@/components/toolbar";
import { classNames } from "@/utils/class-name";
import { getStatusTagColor } from "@/utils/status";
import type { TodoCounts, TodoFilters } from "@/types/app";
import styles from "./index.module.css";

type TodoWorkspaceProps = {
  counts: TodoCounts;
  filters: TodoFilters;
  isTodoBusy: boolean;
  onCreate(): void;
  onDelete(id: string): void;
  onEdit(todo: TodoItem): void;
  onFiltersChange(filters: TodoFilters): void;
  onStatusChange(id: string, status: TodoStatus): void;
  todos: TodoItem[];
  totalTodoCount: number;
};

/**
 * TodoWorkspace 渲染主窗口的完整任务工作台。
 *
 * 工作台包含筛选工具栏、状态统计和任务表格；数据读写动作通过父组件传入，
 * 组件自身只处理展示结构和局部表格列配置。
 */
export function TodoWorkspace({
  counts,
  filters,
  isTodoBusy,
  onCreate,
  onDelete,
  onEdit,
  onFiltersChange,
  onStatusChange,
  todos,
  totalTodoCount,
}: TodoWorkspaceProps) {
  const tableFrameRef = useRef<HTMLDivElement>(null);
  const [hasTableScrollbar, setHasTableScrollbar] = useState(false);

  useEffect(() => {
    const tableFrame = tableFrameRef.current;

    if (!tableFrame) {
      return;
    }

    const observedTableFrame = tableFrame;

    function updateScrollbarState() {
      setHasTableScrollbar(
        observedTableFrame.scrollHeight > observedTableFrame.clientHeight,
      );
    }

    updateScrollbarState();

    const resizeObserver = new ResizeObserver(updateScrollbarState);
    resizeObserver.observe(observedTableFrame);

    return () => {
      resizeObserver.disconnect();
    };
  }, [todos]);

  /**
   * todoTableColumns 定义主窗口 Arco Table 的列结构。
   *
   * 列配置依赖当前 loading 和操作回调，因此保留在组件内部，避免额外传参扩散。
   */
  const todoTableColumns: TableColumnProps<TodoItem>[] = [
    {
      title: "",
      key: "statusIcon",
      render: (_, todo) => <StatusIcon status={todo.status} />,
      width: 32,
    },
    {
      title: "任务",
      dataIndex: "title",
      render: (_, todo) => (
        <TodoContent
          isDone={todo.status === "completed"}
          showNote
          todo={todo}
        />
      ),
    },
    {
      title: "状态",
      render: (_, todo) => (
        <StatusSelect
          disabled={isTodoBusy}
          status={todo.status}
          onChange={(status) => onStatusChange(todo.id, status)}
        />
      ),
      width: 120,
    },
    {
      title: "操作",
      render: (_, todo) => (
        <TableActions
          disabled={isTodoBusy}
          todo={todo}
          onDelete={onDelete}
          onEdit={onEdit}
        />
      ),
      width: 96,
    },
  ];

  return (
    <section className={styles.workspace} aria-label="任务工作台">
      <Toolbar
        keyword={filters.keyword}
        statuses={filters.statuses}
        onCreate={onCreate}
        onKeywordChange={(keyword) =>
          onFiltersChange({
            ...filters,
            keyword,
          })
        }
        onResetFilters={() => onFiltersChange(defaultFilters)}
        onStatusesChange={(statuses) =>
          onFiltersChange({
            ...filters,
            statuses,
          })
        }
      />

      <Card className={styles.card} bordered={false} aria-label="任务列表">
        <div className={styles.heading}>
          <div className={styles.headingTitle}>
            <span>任务清单</span>
            <strong>{todos.length} 项</strong>
          </div>
          <div className={styles.metrics}>
            <Tag
              className={styles.metric}
              color={getStatusTagColor("notStarted")}
              size="small"
            >
              {counts.notStarted} 项未开始
            </Tag>
            <Tag
              className={styles.metric}
              color={getStatusTagColor("inProgress")}
              size="small"
            >
              {counts.inProgress} 项进行中
            </Tag>
            <Tag
              className={styles.metric}
              color={getStatusTagColor("completed")}
              size="small"
            >
              {counts.completed} 项已完成
            </Tag>
          </div>
        </div>

        <div
          className={classNames(
            styles.tableHeader,
            hasTableScrollbar && styles.tableHeaderWithScrollbar,
          )}
          role="row"
          aria-hidden="true"
        >
          <span className={styles.tableHeaderCell} />
          <span className={styles.tableHeaderCell}>任务</span>
          <span className={styles.tableHeaderCell}>状态</span>
          <span className={styles.tableHeaderCell}>操作</span>
        </div>

        <div className={styles.tableFrame} ref={tableFrameRef}>
          <Table
            border={false}
            className={styles.table}
            columns={todoTableColumns}
            data={todos}
            loading={isTodoBusy}
            noDataElement={
              <Empty
                className={styles.empty}
                description={
                  totalTodoCount === 0
                    ? "还没有任务，先添加一项。"
                    : "没有符合条件的任务。"
                }
              />
            }
            pagination={false}
            rowKey="id"
            showHeader={false}
            tableLayoutFixed
          />
        </div>
      </Card>
    </section>
  );
}
