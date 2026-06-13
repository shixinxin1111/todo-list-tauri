# AGENTS.md

## 项目概览

这是一个基于 `Tauri v2` 的桌面 Todo 应用，主要技术栈如下：

- 前端：`Vite + React + TypeScript`
- 桌面壳：`Tauri v2`
- 原生层：`Rust`
- UI 组件库：`@arco-design/web-react`

前端负责渲染 Todo 界面，并通过 Tauri 的 `invoke` 调用 Rust 命令。Rust 侧负责本地数据持久化、窗口模式切换和系统托盘行为。

## 常用命令

安装依赖：

```bash
npm install
```

仅启动前端开发服务器：

```bash
npm run dev
```

仅构建前端资源：

```bash
npm run build
```

本地桌面调试：

```bash
npm run tauri dev
```

桌面应用打包：

```bash
npm run tauri build
```

说明：

- `npm run tauri dev` 会先自动执行 `npm run dev`，因为 `src-tauri/tauri.conf.json` 中配置了 `beforeDevCommand`
- `npm run tauri build` 会先自动执行 `npm run build`，因为 `src-tauri/tauri.conf.json` 中配置了 `beforeBuildCommand`

## 目录结构

- `src/`：React 渲染层代码
- `src/app/`：应用顶层组合入口
- `src/components/`：界面组件
- `src/hooks/`：渲染层状态与副作用逻辑
- `src/utils/`：Tauri API 封装与业务工具函数
- `src-tauri/`：Rust / Tauri 原生工程
- `src-tauri/src/lib.rs`：原生命令、托盘逻辑、窗口状态、Todo 持久化
- `src-tauri/tauri.conf.json`：Tauri 配置、前端联调配置、打包配置

## 架构说明

- 渲染层通过 `@tauri-apps/api/core` 的 `invoke` 与 Rust 通信
- Todo 的增删改查能力由 Rust 暴露，并在 `src/utils/api.ts` 中封装
- 渲染层的 Todo 数据加载、写入编排和错误处理主要位于 `src/hooks/use-todo-store.tsx`
- 窗口模式切换、悬浮态和迷你悬浮态由 Tauri 事件与原生命令共同协调
- Todo 数据保存在本地 JSON 文件中，不依赖远程后端

## 优先阅读文件

在回答问题、排查问题或修改代码前，优先阅读以下文件：

- `package.json`
- `src-tauri/tauri.conf.json`
- `src/app/index.tsx`
- `src/utils/api.ts`
- `src/hooks/use-todo-store.tsx`
- `src-tauri/src/lib.rs`

## AI 工作规则

- 先理解当前实现，再提出结构性调整建议
- 将本项目视为 Tauri 桌面应用，而不是普通 Web 页面
- 如果任务涉及启动、打包、托盘、窗口模式或本地持久化，优先检查 `src-tauri/src/lib.rs`
- 如果任务涉及界面行为，优先检查 `src/app`、`src/components` 和 `src/hooks`
- 如果任务涉及数据写入或状态同步，同时检查 `src/utils/api.ts` 和 `src-tauri/src/lib.rs`
- 不要默认认为 `README.md` 是权威信息源；当前更应以配置文件和源码为准
- 优先做最小、定向、符合当前架构的修改，避免无必要重构

## AI 回答前的项目上下文

在后续回答中，默认记住以下事实：

- 本项目使用的是 `npm`，不是 `pnpm` 或 `yarn`
- 本项目标准的本地桌面调试命令是 `npm run tauri dev`
- 本项目标准的桌面打包命令是 `npm run tauri build`
- `npm run dev` 只会启动前端开发服务器，不能完整覆盖桌面壳行为
- `npm run build` 只会构建前端资源；完整桌面产物应通过 `npm run tauri build` 生成
