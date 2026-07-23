import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App.jsx";
import { DiscoveryBrowserCandidate } from "./candidates/DiscoveryBrowserCandidate.jsx";
import { EasingGraphCandidate } from "./candidates/EasingGraphCandidate.jsx";
import { GraphViewCandidate } from "./candidates/GraphViewCandidate.jsx";
import { TimelineCandidate } from "./candidates/TimelineCandidate.jsx";
import { LegacyHostBoundaryScreen } from "./legacy/index.js";
import { AllSurfacesScreen } from "./screens/AllSurfacesScreen.jsx";

// 各画面担当はこのregistryへfixtureを足し、Appの経路解決を共有する。
export const screenRegistry = {
  "archive/all-surfaces": {
    title: "Archive / host boundary / all surfaces",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "all-surfaces" },
    catalogKind: "archive",
    archive: true,
  },
  "archive/asset-explorer": {
    title: "Archive / host boundary / asset explorer",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "asset-explorer" },
    catalogKind: "archive",
    archive: true,
  },
  "archive/inbox-empty": {
    title: "Archive / host boundary / empty inbox",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "inbox-empty" },
    catalogKind: "archive",
    archive: true,
  },
  "archive/color-book": {
    title: "Archive / host boundary / color book",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "color-book" },
    catalogKind: "archive",
    archive: true,
  },
  "archive/z-rail": {
    title: "Archive / host boundary / depth rail",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "z-rail" },
    catalogKind: "archive",
    archive: true,
  },
  "archive/easing-interval": {
    title: "Archive / host boundary / easing interval",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "easing-interval" },
    catalogKind: "archive",
    archive: true,
  },
  "archive/settings": {
    title: "Archive / host boundary / settings",
    Component: LegacyHostBoundaryScreen,
    props: { fixture: "settings" },
    catalogKind: "archive",
    archive: true,
  },
  "plugin-browser-candidate": {
    title: "Plugin discovery / browser candidate",
    Component: LegacyHostBoundaryScreen,
    props: {
      fixture: "plugin-browser-candidate",
      BrowserComponent: DiscoveryBrowserCandidate,
      EasingGraphComponent: EasingGraphCandidate,
      GraphViewComponent: GraphViewCandidate,
      TimelineComponent: TimelineCandidate,
      resizableLayout: true,
    },
    catalogKind: "candidate",
  },
  "graph-view-candidate": {
    title: "Multi-key Graph View candidate",
    Component: GraphViewCandidate,
    catalogKind: "candidate",
  },
  skeleton: {
    title: "Component boundary skeleton",
    Component: AllSurfacesScreen,
    catalogKind: "diagnostic",
  },
};

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <App registry={screenRegistry} />
  </StrictMode>,
);
