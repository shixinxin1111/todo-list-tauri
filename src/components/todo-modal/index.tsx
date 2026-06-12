import { useEffect } from "react";
import { Form, Input, Modal } from "@arco-design/web-react";
import { classNames } from "@/utils/class-name";
import type { TodoModalMode } from "@/types/app";
import styles from "./index.module.css";

type TodoModalFormValues = {
  note: string;
  title: string;
};

type TodoModalProps = {
  draft: TodoInput;
  isBusy: boolean;
  isFloating: boolean;
  mode: TodoModalMode | null;
  onCancel(): void;
  onSubmit(draft: TodoInput): void;
};

/**
 * TodoModal 渲染新增和编辑任务共用的弹窗表单。
 *
 * 弹窗只暴露可编辑的标题和备注字段，字段状态和校验统一交给 Arco Form 管理。
 */
export function TodoModal({
  draft,
  isBusy,
  isFloating,
  mode,
  onCancel,
  onSubmit,
}: TodoModalProps) {
  const [form] = Form.useForm<TodoModalFormValues>();
  const isOpen = mode !== null;

  useEffect(() => {
    if (!isOpen) {
      form.resetFields();
      return;
    }

    form.setFieldsValue({
      note: draft.note,
      title: draft.title,
    });
  }, [draft.note, draft.title, form, isOpen]);

  async function submitForm() {
    const values = await form.validate();

    onSubmit({
      note: values.note ?? "",
      title: values.title.trim(),
    });
  }

  return (
    <Modal
      className={classNames(styles.modal, isFloating && styles.floating)}
      cancelText="取消"
      confirmLoading={isBusy}
      maskClosable={false}
      okText={mode === "create" ? "添加任务" : "保存修改"}
      title={mode === "create" ? "新增任务" : "编辑任务"}
      visible={isOpen}
      onCancel={onCancel}
      onOk={() => void submitForm()}
    >
      <Form className={styles.form} form={form} layout="vertical">
        <Form.Item
          field="title"
          label="标题"
          rules={[
            { required: true, message: "请输入任务标题" },
            {
              validator(value, callback) {
                if (!value?.trim()) {
                  callback("请输入任务标题");
                }
              },
            },
          ]}
        >
          <Input autoFocus placeholder="例如：整理需求" />
        </Form.Item>
        <Form.Item field="note" label="备注">
          <Input.TextArea
            autoSize={{ minRows: 3, maxRows: 5 }}
            placeholder="补充细节，可留空"
          />
        </Form.Item>
      </Form>
    </Modal>
  );
}
