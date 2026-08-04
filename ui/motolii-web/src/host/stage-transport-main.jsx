import { createRoot } from "react-dom/client";
import { StageTransportCandidate } from "../index.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./stage-host-screen.css";
import {
  readStageHostSnapshot,
  subscribeStageTransportSnapshot,
} from "./stageHostBridge.js";
import { encodeStageEasingIntent } from "./stage-easing-intent-codec.js";

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
        onOpenEasing={(anchor) => {
          const easing = window.__MOTOLII_STAGE_EASING__;
          if (!easing || !Number.isSafeInteger(easing.layoutEpoch) || easing.layoutEpoch <= 0) {
            return;
          }
          try {
            easing.postMessage(encodeStageEasingIntent(anchor, easing.layoutEpoch));
          } catch {
            // 初期化直後のraceでReact event handlerを例外終了させない。
          }
        }}
      />
    </main>,
  );
}

render(snapshot);
subscribeStageTransportSnapshot(window.__MOTOLII_STAGE_HOST__, render);
