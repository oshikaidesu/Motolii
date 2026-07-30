import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { KeyToolsCandidate } from "../index.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./timeline-tools-host-screen.css";

const bridge = window.__MOTOLII_TIMELINE_TOOLS_HOST__;
if (
  bridge === null
  || typeof bridge !== "object"
  || Array.isArray(bridge)
  || typeof bridge.subscribe !== "function"
) {
  throw new TypeError("Motolii Timeline Tools Host bridge is unavailable");
}

function TimelineToolsScreen() {
  const [snapshot, setSnapshot] = useState(bridge.snapshot);
  const [open, setOpen] = useState(true);
  const [mode, setMode] = useState("keys");
  const [scope, setScope] = useState("object");
  const [keySection, setKeySection] = useState(null);
  const [layerSection, setLayerSection] = useState(null);

  useEffect(() => bridge.subscribe(setSnapshot), []);

  return (
    <main className="timeline-tools-standalone-screen">
      <KeyToolsCandidate
        open={open}
        onOpen={() => setOpen(true)}
        onClose={() => setOpen(false)}
        mode={mode}
        onModeChange={setMode}
        keyCount={snapshot.keyCount}
        layerCount={snapshot.layerCount}
        scope={scope}
        onScopeChange={setScope}
        keySection={keySection}
        onKeySectionChange={setKeySection}
        layerSection={layerSection}
        onLayerSectionChange={setLayerSection}
        onKeyOperation={(operation) => bridge.reportUnavailable("key", operation)}
        onLayerOperation={(operation) => bridge.reportUnavailable("layer", operation)}
      />
    </main>
  );
}

const container = document.querySelector("#motolii-timeline-tools-root");
if (!container) {
  throw new TypeError("Motolii Timeline Tools mount is unavailable");
}
createRoot(container).render(<TimelineToolsScreen />);
