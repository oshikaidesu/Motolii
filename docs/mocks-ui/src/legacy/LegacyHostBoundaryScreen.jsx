import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
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
  InspectorContext,
} from "@motolii/motolii-web";
import {
  ResizableLegacyApp,
  ResizableLegacyTimeline,
  ResizableLegacyWorkspace,
  ResizableTimelineSurface,
} from "../layout/ResizablePanelLayout.jsx";

const initializationKey = Symbol.for("motolii.legacyHostBoundary.cleanup");
const inspectorSinkKey = Symbol.for("motolii.legacyHostBoundary.inspector");

const EasingTriggerContext = createContext(null);

function EasingTriggerSlot({ Component }) {
  const value = useContext(EasingTriggerContext);
  return (
    <Component
      activeInterval={value?.activeInterval ?? null}
      pressed={value?.pressed ?? false}
    />
  );
}

class ContainmentInitializationError extends Error {
  constructor(message) {
    super(message);
    this.name = "ContainmentInitializationError";
  }
}

class EasingContainmentInitializationError extends Error {
  constructor(message) {
    super(message);
    this.name = "EasingContainmentInitializationError";
  }
}

const EASING_CONTAINMENT_RULES = [
  {
    anchor: 'button=$("#interval-easing")',
    replacement: 'button=document.createElement("button")',
  },
  {
    anchor: '$("#interval-easing").setAttribute("aria-pressed","false");',
    replacement:
      'document.createElement("button").setAttribute("aria-pressed","false");',
  },
  {
    anchor: '$("#interval-easing").setAttribute("aria-pressed","true");',
    replacement:
      'document.createElement("button").setAttribute("aria-pressed","true");',
  },
  {
    anchor: '$("#interval-easing").onclick=',
    replacement: 'document.createElement("button").onclick=',
  },
  {
    anchor: '$("#interval-easing").click()',
    replacement: 'document.createElement("button").click()',
  },
];

const INSPECTOR_CONTAINMENT_RULES = [
  {
    anchor:
      '$("#inspector").innerHTML=inspector(mode); clearUndo(); bindInspector();',
    replacement:
      'document.createElement("aside").innerHTML=inspector(mode); clearUndo();window[Symbol.for("motolii.legacyHostBoundary.inspector")]?.publish({mode,state,setUndo,status,projectAutomation,applyScrubValue,setSurface,syncColorBook,setStageTool,renderPluginHistory,setMode});',
  },
  {
    anchor:
      'if(chip){chip.style.setProperty("--chip",state.selectedColor);chip.dataset.label=state.selectedColor}',
    replacement:
      'if(chip)window[Symbol.for("motolii.legacyHostBoundary.inspector")]?.refresh()',
  },
];

const EMPTY_STAGE_PROJECTION_CLASSES = [
  "scene-copy",
  "rings",
  "selection-bounds",
  "motion-path",
  "stage-hud",
  "stage-badge",
];

function containAnchors(script, rules, label) {
  for (const { anchor } of rules) {
    let count = 0;
    let index = script.indexOf(anchor);
    while (index !== -1) {
      count += 1;
      index = script.indexOf(anchor, index + anchor.length);
    }
    if (count !== 1) {
      const message = `${label} containment anchor ${JSON.stringify(anchor)} expected count 1, observed ${count}`;
      if (label === "easing") {
        throw new EasingContainmentInitializationError(message);
      }
      throw new ContainmentInitializationError(message);
    }
  }
  let contained = script;
  for (const { anchor, replacement } of rules) {
    contained = contained.replace(anchor, replacement);
  }
  return contained;
}

function containEasingTrigger(script) {
  return containAnchors(script, EASING_CONTAINMENT_RULES, "easing");
}

function containInspector(script) {
  return containAnchors(script, INSPECTOR_CONTAINMENT_RULES, "inspector");
}

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
  EasingTriggerComponent = null,
  onActiveIntervalChange = null,
  emptyStageProjection = false,
) {
  const options = {
    replace(node) {
      if (
        emptyStageProjection &&
        EMPTY_STAGE_PROJECTION_CLASSES.some((className) =>
          matches(node, { className }),
        )
      ) {
        return <></>;
      }
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
          emptyProjection: emptyStageProjection,
          legacyCurveShelf: curveShelf
            ? domToReact([curveShelf], options)
            : null,
          onActiveIntervalChange,
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
        EasingTriggerComponent &&
        matches(node, { id: "interval-easing" })
      ) {
        return <EasingTriggerSlot Component={EasingTriggerComponent} />;
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

function executeTrustedFixtureScript(fixture, script = legacyScript) {
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
    Function(`"use strict";\n${script}`)();
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
  EasingTriggerComponent,
  resizableLayout,
  developmentEmptyProjection,
}) {
  const [activeInterval, setActiveInterval] = useState(null);
  const [pressed, setPressed] = useState(false);
  const [inspectorPayload, setInspectorPayload] = useState(null);
  const [inspectorRefresh, setInspectorRefresh] = useState(0);

  const containedScript = useMemo(() => {
    let script = containInspector(legacyScript);
    if (EasingTriggerComponent) {
      script = containEasingTrigger(script);
    }
    return script;
  }, [EasingTriggerComponent]);

  useEffect(() => {
    window[inspectorSinkKey] = {
      publish: (payload) => {
        setInspectorPayload(payload);
      },
      refresh: () => {
        setInspectorRefresh((value) => value + 1);
      },
    };
    return () => {
      delete window[inspectorSinkKey];
    };
  }, []);

  const parsedContent = useMemo(
    () =>
      parse(
        legacyBody,
        createParserOptions(
          BrowserComponent,
          EasingGraphComponent,
          GraphViewComponent,
          TimelineComponent,
          resizableLayout,
          EasingTriggerComponent,
          setActiveInterval,
          developmentEmptyProjection,
        ),
      ),
    [
      BrowserComponent,
      EasingGraphComponent,
      GraphViewComponent,
      TimelineComponent,
      resizableLayout,
      EasingTriggerComponent,
      developmentEmptyProjection,
    ],
  );

  useEffect(
    () => {
      if (developmentEmptyProjection) {
        return undefined;
      }
      return executeTrustedFixtureScript(fixture, containedScript);
    },
    [fixture, containedScript, developmentEmptyProjection],
  );

  useEffect(() => {
    if (!EasingTriggerComponent) {
      return undefined;
    }
    const button = document.querySelector("#interval-easing");
    if (!button) {
      return undefined;
    }
    const openPanel = () => {
      setPressed(true);
      const panel = document.querySelector("#easing-panel");
      panel?.classList.add("open");
      panel?.setAttribute("aria-hidden", "false");
    };
    button.addEventListener("click", openPanel);
    return () => {
      button.removeEventListener("click", openPanel);
    };
  }, [EasingTriggerComponent]);

  useEffect(() => {
    if (!EasingTriggerComponent) {
      return undefined;
    }
    const onCloseClick = (event) => {
      if (event.target.closest("#close-easing")) {
        setPressed(false);
      }
    };
    const onKeyDown = (event) => {
      if (event.key === "Escape") {
        setPressed((current) => {
          if (!current) {
            return current;
          }
          return false;
        });
      }
    };
    document.addEventListener("click", onCloseClick);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("click", onCloseClick);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [EasingTriggerComponent]);

  return (
    <>
      <style data-legacy-host-boundary>{legacyStyle}</style>
      <InspectorContext.Provider
        value={{
          payload: inspectorPayload,
          refresh: inspectorRefresh,
        }}
      >
        <EasingTriggerContext.Provider value={{ activeInterval, pressed }}>
          {parsedContent}
        </EasingTriggerContext.Provider>
      </InspectorContext.Provider>
    </>
  );
}

export function LegacyHostBoundaryScreen({
  fixture = "all-surfaces",
  BrowserComponent,
  EasingGraphComponent,
  GraphViewComponent,
  TimelineComponent,
  EasingTriggerComponent,
  resizableLayout = false,
  developmentEmptyProjection = false,
}) {
  return (
    <LegacyFixture
      key={fixtureHash(fixture)}
      fixture={fixture}
      BrowserComponent={BrowserComponent}
      EasingGraphComponent={EasingGraphComponent}
      GraphViewComponent={GraphViewComponent}
      TimelineComponent={TimelineComponent}
      EasingTriggerComponent={EasingTriggerComponent}
      resizableLayout={resizableLayout}
      developmentEmptyProjection={developmentEmptyProjection}
    />
  );
}
