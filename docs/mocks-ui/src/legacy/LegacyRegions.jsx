import { createElement } from "react";
import {
  attributesToProps,
  domToReact,
} from "html-react-parser";

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
  return <OriginalElement {...props} />;
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

