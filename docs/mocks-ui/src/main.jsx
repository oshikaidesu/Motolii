import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App.jsx";

// 各画面担当はこのregistryへfixtureを足し、Appの経路解決を共有する。
const screenRegistry = {};

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <App registry={screenRegistry} />
  </StrictMode>,
);
