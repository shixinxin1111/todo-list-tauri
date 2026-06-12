import { Button, Input, Select } from "@arco-design/web-react";
import { IconPlus, IconRefresh } from "@arco-design/web-react/icon";
import { todoStatusOptions } from "@/constants/todo";
import styles from "./index.module.css";

type ToolbarProps = {
  keyword: string;
  onCreate(): void;
  onKeywordChange(keyword: string): void;
  onResetFilters(): void;
  onStatusesChange(statuses: TodoStatus[]): void;
  statuses: TodoStatus[];
};

/**
 * Toolbar 渲染主窗口任务列表上方的搜索、状态筛选和新增入口。
 *
 * 所有筛选状态由父组件持有，Toolbar 只负责把用户输入转换为回调事件。
 */
export function Toolbar({
  keyword,
  onCreate,
  onKeywordChange,
  onResetFilters,
  onStatusesChange,
  statuses,
}: ToolbarProps) {
  return (
    <div className={styles.toolbar}>
      <div className={styles.filters}>
        <Input.Search
          allowClear
          aria-label="搜索任务"
          className={styles.search}
          placeholder="搜索任务"
          value={keyword}
          onChange={onKeywordChange}
        />
        <Select
          allowClear
          className={styles.statusSelect}
          mode="multiple"
          options={todoStatusOptions}
          placeholder="全部状态"
          value={statuses}
          onChange={(nextStatuses) =>
            onStatusesChange(nextStatuses as TodoStatus[])
          }
        />
        <Button
          aria-label="重置筛选"
          htmlType="button"
          icon={<IconRefresh />}
          iconOnly
          title="重置筛选"
          onClick={onResetFilters}
        />
      </div>
      <div className={styles.actions}>
        <Button
          htmlType="button"
          icon={<IconPlus />}
          type="primary"
          onClick={onCreate}
        >
          添加任务
        </Button>
      </div>
    </div>
  );
}
