import { createRoot } from "react-dom/client";
import { StageTransportCandidate } from "../index.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./stage-host-screen.css";
import {
  readStageHostSnapshot,
  subscribeStageTransportSnapshot,
} from "./stageHostBridge.js";

const snapshot = readStageHostSnapshot(window.__MOTOLII_STAGE_HOST__);
const container = document.querySelector("#motolii-stage-transport-root");
if (!container) {
  throw new TypeError("Motolii Stage transport mount is unavailable");
}

const root = createRoot(container);
function render(nextSnapshot) {
  root.render(
    <main className="stage-standalone-screen stage-transport-screen">
      <StageTransportCandidate
        timecode={nextSnapshot.timecode}
        barPosition={nextSnapshot.barPosition}
        tempoStatus={nextSnapshot.tempoStatus}
        qualityStatus={nextSnapshot.qualityStatus}
        activeInterval={nextSnapshot.activeInterval}
      />
    </main>,
  );
}

render(snapshot);
subscribeStageTransportSnapshot(window.__MOTOLII_STAGE_HOST__, render);
