import { createRoot } from "react-dom/client";
import { InspectorCandidate } from "../index.js";
import { decodeInspectorReadModel } from "../read-model/inspectorReadModelDecoder.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./inspector-host-screen.css";

const bridge = window.__MOTOLII_INSPECTOR_HOST__;
if (
  bridge === null
  || typeof bridge !== "object"
  || Array.isArray(bridge)
  || Object.keys(bridge).length !== 3
  || !Object.hasOwn(bridge, "snapshot")
  || !Object.hasOwn(bridge, "subscribe")
  || !Object.hasOwn(bridge, "publish")
) {
  throw new TypeError("Motolii Inspector Host bridge is unavailable");
}

const container = document.querySelector("#motolii-inspector-root");
if (!container) {
  throw new TypeError("Motolii Inspector mount is unavailable");
}

const root = createRoot(container);
const renderSnapshot = (raw) => {
  if (raw === null) {
    root.render(null);
    return;
  }
  const inspectorReadModel = decodeInspectorReadModel(raw);
  root.render(
    <main className="inspector-standalone-screen">
      <InspectorCandidate inspectorReadModel={inspectorReadModel} />
    </main>,
  );
};

bridge.subscribe(renderSnapshot);
