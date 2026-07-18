import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App.jsx";
import { DiscoveryBrowserCandidate } from "./candidates/DiscoveryBrowserCandidate.jsx";
import { LegacyHostBoundaryScreen } from "./legacy/index.js";
import { AllSurfacesScreen } from "./screens/AllSurfacesScreen.jsx";

// 各画面担当はこのregistryへfixtureを足し、Appの経路解決を共有する。
const screenRegistry = {
  "all-surfaces": {
    title: "Host boundary / all surfaces",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "all-surfaces" },
  },
  "asset-explorer": {
    title: "Host boundary / asset explorer",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "asset-explorer" },
  },
  "inbox-empty": {
    title: "Host boundary / empty inbox",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "inbox-empty" },
  },
  "color-book": {
    title: "Host boundary / color book",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "color-book" },
  },
  "z-rail": {
    title: "Host boundary / depth rail",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "z-rail" },
  },
  "easing-interval": {
    title: "Host boundary / easing interval",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "easing-interval" },
  },
  settings: {
    title: "Host boundary / settings",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "settings" },
  },
  "plugin-browser-candidate": {
    title: "Plugin discovery / browser candidate",
    Component: LegacyHostBoundaryScreen,
    props: {
      fixture: "plugin-browser-candidate",
      BrowserComponent: DiscoveryBrowserCandidate,
    },
  },
  skeleton: {
    title: "Component boundary skeleton",
    Component: AllSurfacesScreen,
  },
};

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <App registry={screenRegistry} />
  </StrictMode>,
);
