import {
  createContext,
  createElement,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  attributesToProps,
  domToReact,
} from "html-react-parser";
import "./resizable-panel-layout.css";

const DEFAULT_LAYOUT = {
  browser: 284,
  inspector: 326,
  timeline: 270,
};

const PANEL_SPEC = {
  browser: {
    label: "Browser",
    min: 180,
    max: 520,
    orientation: "vertical",
  },
  inspector: {
    label: "Inspector",
    min: 220,
    max: 560,
    orientation: "vertical",
  },
  timeline: {
    label: "Timeline",
    min: 160,
    max: 520,
    orientation: "horizontal",
  },
};

const PanelLayoutContext = createContext(null);

function clamp(value, min, max) {
  return Math.round(Math.max(min, Math.min(max, value)));
}

function layoutLimits(key, layout) {
  const spec = PANEL_SPEC[key];
  if (key === "timeline") {
    const chromeHeight = 34 + 32 + 24;
    const minimumWorkspaceHeight = 260;
    return {
      min: spec.min,
      max: Math.max(
        spec.min,
        Math.min(
          spec.max,
          window.innerHeight - chromeHeight - minimumWorkspaceHeight,
        ),
      ),
    };
  }

  const other =
    key === "browser" ? layout.inspector : layout.browser;
  const minimumStageWidth = 440;
  return {
    min: spec.min,
    max: Math.max(
      spec.min,
      Math.min(
        spec.max,
        window.innerWidth - other - minimumStageWidth,
      ),
    ),
  };
}

function clampLayout(layout) {
  let next = { ...layout };
  for (const key of ["browser", "inspector", "timeline"]) {
    const limits = layoutLimits(key, next);
    next[key] = clamp(next[key], limits.min, limits.max);
  }
  return next;
}

function originalElement(node, options, extraProps = {}, children = null) {
  const props = attributesToProps(node.attribs, node.name);
  return createElement(
    node.name,
    { ...props, ...extraProps },
    children ?? domToReact(node.children ?? [], options),
  );
}

export function ResizableLegacyApp({ node, options }) {
  const [layout, setLayout] = useState(DEFAULT_LAYOUT);

  const setPanelSize = useCallback((key, value) => {
    setLayout((current) => {
      const limits = layoutLimits(key, current);
      return {
        ...current,
        [key]: clamp(value, limits.min, limits.max),
      };
    });
  }, []);

  const resetPanelSize = useCallback(
    (key) => setPanelSize(key, DEFAULT_LAYOUT[key]),
    [setPanelSize],
  );

  useEffect(() => {
    const handleResize = () => {
      setLayout((current) => clampLayout(current));
    };
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  const value = useMemo(
    () => ({ layout, resetPanelSize, setPanelSize }),
    [layout, resetPanelSize, setPanelSize],
  );
  const style = {
    "--mock-browser-size": `${layout.browser}px`,
    "--mock-inspector-size": `${layout.inspector}px`,
    "--mock-timeline-size": `${layout.timeline}px`,
  };

  return (
    <PanelLayoutContext.Provider value={value}>
      {originalElement(
        node,
        options,
        {
          "data-resizable-layout": "true",
          style,
        },
      )}
    </PanelLayoutContext.Provider>
  );
}

function usePanelLayout() {
  const value = useContext(PanelLayoutContext);
  if (!value) {
    throw new Error("Resizable panel must be inside ResizableLegacyApp");
  }
  return value;
}

function PanelSeparator({ panel }) {
  const { layout, resetPanelSize, setPanelSize } = usePanelLayout();
  const drag = useRef(null);
  const [dragging, setDragging] = useState(false);
  const spec = PANEL_SPEC[panel];
  const limits = layoutLimits(panel, layout);

  const finish = useCallback(() => {
    drag.current = null;
    setDragging(false);
  }, []);

  useEffect(() => {
    if (!dragging) {
      return undefined;
    }
    const cancel = (event) => {
      if (event.key !== "Escape" || !drag.current) {
        return;
      }
      setPanelSize(panel, drag.current.startValue);
      finish();
    };
    window.addEventListener("keydown", cancel);
    return () => window.removeEventListener("keydown", cancel);
  }, [dragging, finish, panel, setPanelSize]);

  const begin = (event) => {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = {
      startX: event.clientX,
      startY: event.clientY,
      startValue: layout[panel],
    };
    setDragging(true);
  };

  const move = (event) => {
    if (!drag.current) {
      return;
    }
    const horizontalDelta = event.clientX - drag.current.startX;
    const verticalDelta = event.clientY - drag.current.startY;
    const delta =
      panel === "browser"
        ? horizontalDelta
        : panel === "inspector"
          ? -horizontalDelta
          : -verticalDelta;
    setPanelSize(panel, drag.current.startValue + delta);
  };

  const handleKeyDown = (event) => {
    if (event.key === "Home") {
      event.preventDefault();
      resetPanelSize(panel);
      return;
    }

    const step = event.shiftKey ? 48 : 16;
    const direction = {
      browser: { ArrowLeft: -1, ArrowRight: 1 },
      inspector: { ArrowLeft: 1, ArrowRight: -1 },
      timeline: { ArrowUp: 1, ArrowDown: -1 },
    }[panel][event.key];
    if (!direction) {
      return;
    }
    event.preventDefault();
    setPanelSize(panel, layout[panel] + direction * step);
  };

  return (
    <button
      type="button"
      className={`react-panel-separator react-panel-separator--${panel}${dragging ? " is-dragging" : ""}`}
      role="separator"
      aria-label={`${spec.label}のサイズを変更`}
      aria-orientation={spec.orientation}
      aria-valuemin={limits.min}
      aria-valuemax={limits.max}
      aria-valuenow={layout[panel]}
      data-panel-id={panel}
      title={`${spec.label}: drag / arrow keys / double-click to reset`}
      onDoubleClick={() => resetPanelSize(panel)}
      onKeyDown={handleKeyDown}
      onPointerCancel={() => {
        if (drag.current) {
          setPanelSize(panel, drag.current.startValue);
        }
        finish();
      }}
      onPointerDown={begin}
      onPointerMove={move}
      onPointerUp={finish}
    />
  );
}

export function ResizableLegacyWorkspace({ node, options }) {
  return originalElement(
    node,
    options,
    { "data-react-layout-surface": "workspace" },
    <>
      {domToReact(node.children ?? [], options)}
      <PanelSeparator panel="browser" />
      <PanelSeparator panel="inspector" />
    </>,
  );
}

export function ResizableLegacyTimeline({ node, options }) {
  return originalElement(
    node,
    options,
    { "data-react-layout-surface": "timeline" },
    <>
      <PanelSeparator panel="timeline" />
      {domToReact(node.children ?? [], options)}
    </>,
  );
}

export function ResizableTimelineSurface({ Component, componentProps = {} }) {
  return (
    <Component
      {...componentProps}
      resizeHandle={<PanelSeparator panel="timeline" />}
    />
  );
}
