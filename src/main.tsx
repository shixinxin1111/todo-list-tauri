import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@arco-design/web-react/dist/css/arco.css";
import { App } from "./app";
import "./style.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
