import { createRoot } from "react-dom/client";
import { InspectorCandidate } from "../index.js";
import { decodeInspectorReadModel } from "../read-model/inspectorReadModelDecoder.js";
import { createInspectorHostSender } from "./inspectorHostCodec.js";
import "../../../motolii-tokens/generated/tokens.css";
import "./inspector-host-screen.css";

const bridge = window.__MOTOLII_INSPECTOR_HOST__;
if (
  bridge === null
  || typeof bridge !== "object"
  || Array.isArray(bridge)
  || Object.keys(bridge).length !== 4
  || !Object.hasOwn(bridge, "snapshot")
  || !Object.hasOwn(bridge, "subscribe")
  || !Object.hasOwn(bridge, "publish")
  || !Object.hasOwn(bridge, "postMessage")
) {
  throw new TypeError("Motolii Inspector Host bridge is unavailable");
}

const container = document.querySelector("#motolii-inspector-root");
if (!container) {
  throw new TypeError("Motolii Inspector mount is unavailable");
}

const root = createRoot(container);
const inspectorHostSender = createInspectorHostSender(bridge.postMessage);
function projectRustPosition(readModel, rawPosition) {
  if (rawPosition === undefined) return readModel;
  if (
    rawPosition === null
    || typeof rawPosition !== "object"
    || Array.isArray(rawPosition)
    || !["const", "key", "animated"].includes(rawPosition.kind)
  ) {
    throw new TypeError("Inspector Position projection is invalid");
  }
  if (rawPosition.kind === "animated") {
    return { ...readModel, position: { kind: "animated" } };
  }
  if (!Number.isFinite(rawPosition.x) || !Number.isFinite(rawPosition.y)) {
    throw new TypeError("Inspector Position coordinates are invalid");
  }
  return {
    ...readModel,
    position: {
      kind: rawPosition.kind,
      x: rawPosition.x,
      y: rawPosition.y,
    },
  };
}
const renderSnapshot = (raw) => {
  if (raw === null) {
    inspectorHostSender.project(null);
    root.render(null);
    return;
  }
  const { position: rawPosition, ...decoderSnapshot } = raw;
  const inspectorReadModel = projectRustPosition(
    decodeInspectorReadModel(decoderSnapshot),
    rawPosition,
  );
  inspectorHostSender.project(inspectorReadModel);
  root.render(
    <main className="inspector-standalone-screen">
      <InspectorCandidate
        inspectorReadModel={inspectorReadModel}
        onEffectParamGesture={inspectorHostSender.send}
        onPositionKeyGesture={inspectorHostSender.sendPositionKeyGesture}
        onAddPositionKey={inspectorHostSender.sendAddPositionKey}
      />
    </main>,
  );
};

bridge.subscribe(renderSnapshot);
