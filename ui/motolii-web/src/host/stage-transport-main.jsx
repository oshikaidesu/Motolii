import { createRoot } from "react-dom/client";
import { StageTransportCandidate } from "../index.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./stage-host-screen.css";
import { readStageHostSnapshot } from "./stageHostBridge.js";

const snapshot = readStageHostSnapshot(window.__MOTOLII_STAGE_HOST__);
const container = document.querySelector("#motolii-stage-transport-root");
if (!container) {
  throw new TypeError("Motolii Stage transport mount is unavailable");
}

createRoot(container).render(
  <main className="stage-standalone-screen stage-transport-screen">
    <StageTransportCandidate
      timecode={snapshot.timecode}
      barPosition={snapshot.barPosition}
      tempoStatus={snapshot.tempoStatus}
      qualityStatus={snapshot.qualityStatus}
    />
  </main>,
);
