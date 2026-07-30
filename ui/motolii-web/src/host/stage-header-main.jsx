import { createRoot } from "react-dom/client";
import { StageHeaderCandidate } from "../index.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./stage-host-screen.css";
import { readStageHostSnapshot } from "./stageHostBridge.js";

const snapshot = readStageHostSnapshot(window.__MOTOLII_STAGE_HOST__);
const container = document.querySelector("#motolii-stage-header-root");
if (!container) {
  throw new TypeError("Motolii Stage header mount is unavailable");
}

createRoot(container).render(
  <main className="stage-standalone-screen stage-header-screen">
    <StageHeaderCandidate mode={snapshot.mode} />
  </main>,
);
