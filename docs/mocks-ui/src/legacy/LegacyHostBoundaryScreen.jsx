import { useEffect, useMemo } from "react";
import parse, { domToReact } from "html-react-parser";
import {
  LegacyBrowser,
  LegacyColorBook,
  LegacyInspector,
  LegacyOriginalElement,
  LegacyRecovery,
  LegacySettings,
  LegacyStageShell,
  LegacyTimeline,
} from "./LegacyRegions";
import {
  legacyBody,
  legacyScript,
  legacyStyle,
} from "./legacySource";
import {
  ResizableLegacyApp,
  ResizableLegacyTimeline,
  ResizableLegacyWorkspace,
  ResizableTimelineSurface,
} from "../layout/ResizablePanelLayout.jsx";

const initializationKey = Symbol.for("motolii.legacyHostBoundary.cleanup");

function matches(node, { id, className }) {
  if (node.type !== "tag") {
    return false;
  }
  if (id && node.attribs?.id === id) {
    return true;
  }
  return Boolean(
    className &&
      node.attribs?.class?.split(/\s+/).includes(className),
  );
}

function findDescendant(node, target) {
  if (matches(node, target)) {
    return node;
  }
  for (const child of node.children ?? []) {
    const match = findDescendant(child, target);
    if (match) {
      return match;
    }
  }
  return null;
}

function createParserOptions(
  BrowserComponent = LegacyBrowser,
  EasingGraphComponent = null,
  GraphViewComponent = null,
  TimelineComponent = null,
  resizableLayout = false,
) {
  const options = {
    replace(node) {
      const props = { node, options };
      if (matches(node, { className: "browser" })) {
        return <BrowserComponent {...props} />;
      }
      if (matches(node, { id: "color-book-drawer" })) {
        return <LegacyColorBook {...props} />;
      }
      if (matches(node, { className: "stage-shell" })) {
        return <LegacyStageShell {...props} />;
      }
      if (matches(node, { id: "inspector" })) {
        return <LegacyInspector {...props} />;
      }
      if (
        TimelineComponent &&
        matches(node, { id: "timeline" })
      ) {
        const curveShelf = findDescendant(node, { id: "curve-shelf" });
        const componentProps = {
          EasingGraphComponent,
          GraphViewComponent,
          legacyCurveShelf: curveShelf
            ? domToReact([curveShelf], options)
            : null,
        };
        if (resizableLayout) {
          return (
            <ResizableTimelineSurface
              Component={TimelineComponent}
              componentProps={componentProps}
            />
          );
        }
        return <TimelineComponent {...componentProps} />;
      }
      if (
        EasingGraphComponent &&
        matches(node, { id: "easing-panel" })
      ) {
        return <EasingGraphComponent />;
      }
      if (
        resizableLayout &&
        matches(node, { className: "workspace" })
      ) {
        return <ResizableLegacyWorkspace {...props} />;
      }
      if (
        resizableLayout &&
        matches(node, { id: "timeline" })
      ) {
        return <ResizableLegacyTimeline {...props} />;
      }
      if (matches(node, { id: "timeline" })) {
        return <LegacyTimeline {...props} />;
      }
      if (matches(node, { id: "recovery" })) {
        return <LegacyRecovery {...props} />;
      }
      if (matches(node, { id: "settings-sheet" })) {
        return <LegacySettings {...props} />;
      }
      if (matches(node, { className: "app" })) {
        if (resizableLayout) {
          return <ResizableLegacyApp {...props} />;
        }
        return <LegacyOriginalElement {...props} />;
      }
      return undefined;
    },
  };
  return options;
}

function fixtureHash(fixture) {
  const normalized = String(fixture).replace(/^#/, "");
  return normalized ? `#${normalized}` : "";
}

function executeTrustedFixtureScript(fixture) {
  const host = document.querySelector(".app");
  if (!host || host[initializationKey]) {
    return () => {};
  }

  const listeners = [];
  const originalAdd = EventTarget.prototype.addEventListener;
  EventTarget.prototype.addEventListener = function addTrackedListener(
    type,
    listener,
    options,
  ) {
    const target = this instanceof EventTarget ? this : window;
    listeners.push([target, type, listener, options]);
    return originalAdd.call(target, type, listener, options);
  };

  const originalUrl = window.location.href;
  const nextUrl = new URL(originalUrl);
  nextUrl.hash = fixtureHash(fixture);

  let initialized = false;
  try {
    window.history.replaceState(
      window.history.state,
      "",
      `${nextUrl.pathname}${nextUrl.search}${nextUrl.hash}`,
    );
    // sourceは上の静的importだけであり、外部HTMLや入力文字列は評価しない。
    Function(`"use strict";\n${legacyScript}`)();
    host.dataset.parityReady = "true";
    initialized = true;
  } catch (error) {
    listeners.forEach(([target, type, listener, options]) => {
      target.removeEventListener(type, listener, options);
    });
    throw error;
  } finally {
    EventTarget.prototype.addEventListener = originalAdd;
    const previousUrl = new URL(originalUrl);
    window.history.replaceState(
      window.history.state,
      "",
      `${previousUrl.pathname}${previousUrl.search}${previousUrl.hash}`,
    );
  }

  const cleanup = () => {
    delete host.dataset.parityReady;
    listeners.forEach(([target, type, listener, options]) => {
      target.removeEventListener(type, listener, options);
    });
    delete host[initializationKey];
  };

  if (initialized) {
    host[initializationKey] = cleanup;
  }
  return cleanup;
}

function LegacyFixture({
  fixture,
  BrowserComponent,
  EasingGraphComponent,
  GraphViewComponent,
  TimelineComponent,
  resizableLayout,
}) {
  const content = useMemo(
    () =>
      parse(
        legacyBody,
        createParserOptions(
          BrowserComponent,
          EasingGraphComponent,
          GraphViewComponent,
          TimelineComponent,
          resizableLayout,
        ),
      ),
    [
      BrowserComponent,
      EasingGraphComponent,
      GraphViewComponent,
      TimelineComponent,
      resizableLayout,
    ],
  );

  useEffect(
    () => executeTrustedFixtureScript(fixture),
    [fixture],
  );

  return (
    <>
      <style data-legacy-host-boundary>{legacyStyle}</style>
      {content}
    </>
  );
}

export function LegacyHostBoundaryScreen({
  fixture = "all-surfaces",
  BrowserComponent,
  EasingGraphComponent,
  GraphViewComponent,
  TimelineComponent,
  resizableLayout = false,
}) {
  return (
    <LegacyFixture
      key={fixtureHash(fixture)}
      fixture={fixture}
      BrowserComponent={BrowserComponent}
      EasingGraphComponent={EasingGraphComponent}
      GraphViewComponent={GraphViewComponent}
      TimelineComponent={TimelineComponent}
      resizableLayout={resizableLayout}
    />
  );
}
