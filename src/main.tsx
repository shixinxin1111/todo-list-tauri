import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@arco-design/web-react/dist/css/arco.css";
import { App } from "./app";
import "./style.css";

// 在样式加载前标记当前视图，便于 style.css 针对托盘窗口禁用根背景，
// 让 macOS transparent 窗口的圆角不被 html/body/#root 的实色填充覆盖。
const view = new URLSearchParams(window.location.search).get("view") ?? "main";
document.documentElement.dataset.view = view;
document.body.dataset.view = view;
const rootEl = document.getElementById("root");
if (rootEl) {
  rootEl.dataset.view = view;
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
