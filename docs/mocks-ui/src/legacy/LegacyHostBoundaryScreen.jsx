import { useEffect, useMemo } from "react";
import parse from "html-react-parser";
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

function createParserOptions() {
  const options = {
    replace(node) {
      const props = { node, options };
      if (matches(node, { className: "browser" })) {
        return <LegacyBrowser {...props} />;
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

function LegacyFixture({ fixture }) {
  const content = useMemo(
    () => parse(legacyBody, createParserOptions()),
    [],
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
}) {
  return <LegacyFixture key={fixtureHash(fixture)} fixture={fixture} />;
}
