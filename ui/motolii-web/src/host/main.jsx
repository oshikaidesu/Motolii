import { createRoot } from "react-dom/client";
import { DiscoveryBrowserCandidate } from "../index.js";
import {
  createBrowserHostSender,
  decodeBrowserHostSnapshot,
} from "./browserHostCodec.js";

const bridge = window.__MOTOLII_BUILTIN_HOST__;
if (
  bridge === null
  || typeof bridge !== "object"
  || Array.isArray(bridge)
  || Object.keys(bridge).length !== 2
  || !Object.hasOwn(bridge, "snapshot")
  || !Object.hasOwn(bridge, "postMessage")
) {
  throw new TypeError("Motolii built-in Host bridge is unavailable");
}

const snapshot = decodeBrowserHostSnapshot(bridge.snapshot);
const onPlaceIntent = createBrowserHostSender(snapshot, bridge.postMessage);
const container = document.querySelector("#motolii-browser-root");
if (!container) {
  throw new TypeError("Motolii Browser mount is unavailable");
}

createRoot(container).render(
  <DiscoveryBrowserCandidate
    rectangleIdentity={snapshot.rectangleIdentity}
    onPlaceIntent={onPlaceIntent}
  />,
);
