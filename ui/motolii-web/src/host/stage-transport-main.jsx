import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { EasingTriggerCandidate, StageTransportCandidate } from "../index.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./stage-host-screen.css";
import { readStageHostSnapshot } from "./stageHostBridge.js";

const bridge = window.__MOTOLII_STAGE_HOST__;
const initialSnapshot = readStageHostSnapshot(bridge);
const container = document.querySelector("#motolii-stage-transport-root");
if (!container) {
  throw new TypeError("Motolii Stage transport mount is unavailable");
}

function StageTransportScreen() {
  const [snapshot, setSnapshot] = useState(initialSnapshot);

  useEffect(() => bridge.subscribe((next) => {
    setSnapshot(readStageHostSnapshot({ snapshot: next }));
  }), []);

  const openEasing = (anchor) => {
    if (snapshot.activeInterval) {
      bridge.openEasing(snapshot.activeInterval, anchor);
    }
  };

  const togglePlay = () => bridge.togglePlayback();
  const saveProject = () => bridge.saveProject();
  const exportProject = () => bridge.exportProject();

  return (
    <main className="stage-standalone-screen stage-transport-screen">
      <StageTransportCandidate
        timecode={snapshot.timecode}
        barPosition={snapshot.barPosition}
        tempoStatus={snapshot.tempoStatus}
        qualityStatus={snapshot.qualityStatus}
        playing={snapshot.playing}
        togglePlay={togglePlay}
        projectStatus={snapshot.projectStatus}
        exportStatus={snapshot.exportStatus}
        saveProject={saveProject}
        exportProject={exportProject}
        easingTrigger={(
          <span
            className="easing-trigger-seat"
            onClick={(event) => openEasing(event.currentTarget.getBoundingClientRect())}
          >
            <EasingTriggerCandidate
              activeInterval={snapshot.activeInterval}
              pressed={snapshot.easingPressed}
            />
          </span>
        )}
      />
    </main>
  );
}

createRoot(container).render(<StageTransportScreen />);
