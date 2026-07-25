import { createElement, useContext } from "react";
import {
  attributesToProps,
  domToReact,
} from "html-react-parser";
import {
  InspectorCandidate,
  InspectorContext,
} from "../candidates/InspectorCandidate.jsx";

function OriginalElement({ node, options }) {
  return createElement(
    node.name,
    attributesToProps(node.attribs, node.name),
    domToReact(node.children ?? [], options),
  );
}

export function LegacyBrowser(props) {
  return <OriginalElement {...props} />;
}

export function LegacyColorBook(props) {
  return <OriginalElement {...props} />;
}

export function LegacyStageShell(props) {
  return <OriginalElement {...props} />;
}

export function LegacyInspector(props) {
  const inspector = useContext(InspectorContext);
  if (!inspector?.payload) {
    return <OriginalElement {...props} />;
  }
  const effectFocused = document
    .querySelector("#vism-browser")
    ?.classList.contains("candidate-plugin-browser");
  const payload = inspector.payload;
  return (
    <InspectorCandidate
      mode={payload.mode}
      effectFocused={effectFocused}
      state={payload.state}
      setUndo={payload.setUndo}
      status={payload.status}
      projectAutomation={payload.projectAutomation}
      applyScrubValue={payload.applyScrubValue}
      setSurface={payload.setSurface}
      syncColorBook={payload.syncColorBook}
      setStageTool={payload.setStageTool}
      renderPluginHistory={payload.renderPluginHistory}
      setMode={payload.setMode}
    />
  );
}

export function LegacyTimeline(props) {
  return <OriginalElement {...props} />;
}

export function LegacyRecovery(props) {
  return <OriginalElement {...props} />;
}

export function LegacySettings(props) {
  return <OriginalElement {...props} />;
}

export function LegacyOriginalElement(props) {
  return <OriginalElement {...props} />;
}

