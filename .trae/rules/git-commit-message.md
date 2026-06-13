---
alwaysApply: true
description: 统一 Git 提交信息的格式与语言风格，遵循 Conventional Commits 规范，确保提交历史清晰、可检索、可生成变更日志
scene: git_message
---

# Git 提交信息生成规则

## 生效时机

- 当用户请求生成 / 撰写 / 优化 commit message 时
- 执行 `git commit` 前由 AI 自动起草提交信息时
- 对暂存区（staged）变更进行总结、补充说明时
- 修改、合并、整理已有提交（amend / squash / reword）时

> 仅作用于 Git 提交信息文本本身，不影响代码内容、PR 描述、注释或文档。

## 总体要求

1. **语言**：统一使用简体中文撰写主题与正文。专有名词（如 React、Tauri、macOS、API 名、文件名、命令）保持英文原样。
2. **规范**：遵循 [Conventional Commits 1.0.0](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 规范。
3. **聚焦“为什么”**：主题描述变更的目的或效果，正文解释动机与背景，避免逐行复述代码改动。
4. **原子性**：一次提交只对应一个独立、自洽的变更。混合改动应拆分成多个 commit，对应多条 message。

## 标题（Header）

格式：

```
<type>(<scope>): <subject>
```

- `type`：必填，小写，从下表中选择
- `scope`：可选，使用小写英文 / 短横线，标识受影响的模块或目录（如 `titlebar`、`window-state`、`tauri`、`deps`）
- `subject`：必填，简体中文，**不超过 50 个字符**，使用动宾结构，结尾**不加句号**

### 允许的 type

| type     | 适用场景                                         |
| -------- | ------------------------------------------------ |
| feat     | 新增用户可感知的功能                             |
| fix      | 修复 Bug 或异常行为                              |
| perf     | 优化性能、响应速度、资源占用                     |
| refactor | 重构代码，不改变外部行为                         |
| style    | 仅调整代码格式（空白、缩进、引号等），不影响逻辑 |
| ui       | 调整界面视觉、布局、交互（不涉及功能逻辑）       |
| docs     | 仅修改文档、注释、README                         |
| test     | 新增或调整测试用例                               |
| build    | 构建系统、打包脚本、依赖版本变更                 |
| ci       | CI / CD 配置变更                                 |
| chore    | 杂项维护（清理无用文件、升级工具链等）           |
| revert   | 回滚先前提交，正文需注明被回滚的 commit hash     |

## 正文（Body）

- 与标题之间空一行
- 每行**不超过 72 个字符**，必要时手动换行
- 说明：**为何改**、**改了什么**、**对外影响**、**关键决策**
- 涉及多点变更时使用无序列表 `- `
- 引用代码、路径、命令时使用反引号

## 页脚（Footer）

按以下顺序、按需出现：

1. **BREAKING CHANGE**：以 `BREAKING CHANGE:` 开头单独成段，描述破坏性变更与迁移指南
2. **关联 Issue / PR**：`Closes #123`、`Refs #456`
3. **共同作者**：`Co-authored-by: Name <email>`

## 严格禁止

- 使用表情、emoji、装饰字符
- 主题以大写字母开头或带句号
- 使用模糊词（如 `update`、`修改若干`、`一些改动`）
- 在主题中堆砌多个变更（如 `feat: 新增 A 并修复 B`）
- 自动生成无意义的 message（例如直接复制 diff）

## 示例

良好示例：

```
fix(window-state): 修正 Retina 屏窗口模式切换右上角错位

apply_window_mode 之前用物理像素计算锚点位置，但写回时用逻辑像素，
在 scale_factor=2 的设备上导致每次切换偏移半个屏幕。

- 统一通过 scale_factor 换算到逻辑单位再做边界裁剪
- 调整顺序为 set_position → set_size → set_position，避免 macOS
  动画过程中触发屏幕边界夹取
```

```
ui(titlebar): 主窗交通灯按钮与品牌标题垂直对齐

trafficLightPosition 由 (16,13) 调整为 (18,18)，配合 titlebar 左
内边距 86px，使红黄绿按钮与右侧 Todo List 标题在同一基线。
```

不良示例：

```
update titlebar    ← 缺少 type、信息含糊
feat: 修改了一些样式并修了bug ← 混合改动 + 模糊描述
```
