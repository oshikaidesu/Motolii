import React, { useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import Konva from "konva";
import { revision as hmrRevision } from "virtual:g0-9-hmr-probe";
import { DynamicSceneBenchmark } from "./dynamic-scene-benchmark.js";
import { browserItem, createTimelineFixture, fixture } from "./fixture.js";
import { TimelineRenderer } from "./timeline-renderer.js";
import { ObjectHandleProbe, SpatialGizmoProbe } from "./object-handle-probes.jsx";
import "./styles.css";

function BrowserProbe() {
  const rowHeight = 28;
  const overscan = 6;
  const [scrollTop, setScrollTop] = useState(0);
  const [selectedId, setSelectedId] = useState("asset-00000");
  const visibleRows = Math.ceil(fixture.browserViewport[1] / rowHeight);
  const start = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
  const end = Math.min(fixture.browserItems, start + visibleRows + overscan * 2);
  const rows = [];
  for (let index = start; index < end; index += 1) rows.push(browserItem(index));

  return (
    <section>
      <h2>Browser — virtualized DOM</h2>
      <p data-testid="browser-selection">{selectedId}</p>
      <div
        className="browser"
        data-testid="browser"
        onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
      >
        <div style={{ height: fixture.browserItems * rowHeight, position: "relative" }}>
          {rows.map((item, offset) => (
            <button
              className="asset-row"
              data-asset-id={item.id}
              key={item.id}
              onClick={() => setSelectedId(item.id)}
              style={{ top: (start + offset) * rowHeight }}
              type="button"
            >
              {item.label}
            </button>
          ))}
        </div>
      </div>
      <output data-testid="browser-dom-count">{rows.length}</output>
    </section>
  );
}

function TimelineProbe() {
  const canvasRef = useRef(null);
  const pixiHostRef = useRef(null);
  const konvaHostRef = useRef(null);
  const rendererRef = useRef(null);
  const model = useMemo(() => createTimelineFixture(), []);
  const [gpuStatus, setGpuStatus] = useState("pending");
  const [sceneStatus, setSceneStatus] = useState("pending");

  useEffect(() => {
    const renderer = new TimelineRenderer(canvasRef.current, model);
    rendererRef.current = renderer;
    renderer.drawCanvas(0, 48);
    const dynamicScene = new DynamicSceneBenchmark();
    const dynamicSceneReady = dynamicScene.initialize(pixiHostRef.current, konvaHostRef.current)
      .then((result) => {
        setSceneStatus(`${result.pixiRenderer}; ${result.visibleKeys} visible keys`);
        return result;
      });
    window.g09 = {
      fixture,
      measureBrowser: async (steps = 120) => {
        const browser = document.querySelector('[data-testid="browser"]');
        const samples = [];
        for (let step = 0; step < steps; step += 1) {
          const started = performance.now();
          browser.scrollTop = (step / (steps - 1)) * (browser.scrollHeight - browser.clientHeight);
          browser.dispatchEvent(new Event("scroll"));
          await new Promise(requestAnimationFrame);
          samples.push(performance.now() - started);
        }
        samples.sort((a, b) => a - b);
        return {
          steps,
          medianMs: samples[Math.floor(samples.length * 0.5)],
          p95Ms: samples[Math.floor(samples.length * 0.95)],
          domRows: document.querySelectorAll("[data-asset-id]").length,
        };
      },
      measureTimeline: (mode, frames) => renderer.measure(mode, frames),
      initializeWebGpu: async () => {
        const result = await renderer.initializeWebGpu();
        setGpuStatus(result.available ? "available" : result.reason);
        return result;
      },
      initializeDynamicScenes: () => dynamicSceneReady,
      measureDynamicScenes: async (frames = 90) => {
        await dynamicSceneReady;
        return dynamicScene.measureAll(frames);
      },
    };
    setGpuStatus(navigator.gpu ? "not initialized" : "navigator.gpu unavailable");
  }, [model]);

  return (
    <section>
      <h2>Timeline — one dense surface</h2>
      <canvas ref={canvasRef} width="1200" height="512" data-testid="timeline" />
      <p>1,000 clips / 100,000 keys; DOM children: canvas only</p>
      <output data-testid="webgpu-status">{gpuStatus}</output>
      <h3>Known scene-graph drag probes</h3>
      <div className="scene-probe" ref={pixiHostRef} />
      <div className="scene-probe konva-probe" ref={konvaHostRef} />
      <output data-testid="scene-status">{sceneStatus}</output>
    </section>
  );
}

function InteractionProbe() {
  const hostRef = useRef(null);
  const [interaction, setInteraction] = useState({ commits: 0, cancels: 0, selected: [], x: 40, y: 48 });
  const [capture, setCapture] = useState({ moves: 0, cancels: 0, captured: false });
  const [ime, setIme] = useState({ composing: false, shortcutCount: 0, events: [] });

  useEffect(() => {
    const stage = new Konva.Stage({ container: hostRef.current, width: 600, height: 240 });
    const layer = new Konva.Layer();
    const marquee = new Konva.Rect({ visible: false, fill: "#59c7ff33", stroke: "#59c7ff" });
    const group = new Konva.Group({ x: 40, y: 48, draggable: true, name: "key-group" });
    let dragOrigin = { x: group.x(), y: group.y() };
    let cancelled = false;
    let selectionStart = null;

    for (let index = 0; index < 100; index += 1) {
      group.add(new Konva.Rect({
        x: (index % 20) * 24,
        y: Math.floor(index / 20) * 18,
        width: 8,
        height: 8,
        fill: index < 3 ? "#ffcd70" : "#73869c",
        name: `key-${index}`,
      }));
    }
    group.dragBoundFunc((position) => ({
      x: Math.round(position.x / 10) * 10,
      y: Math.round(position.y / 10) * 10,
    }));
    group.on("dragstart", () => {
      dragOrigin = { x: group.x(), y: group.y() };
      cancelled = false;
    });
    group.on("dragend", () => {
      if (!cancelled && (group.x() !== dragOrigin.x || group.y() !== dragOrigin.y)) {
        setInteraction((value) => ({
          ...value, commits: value.commits + 1, x: group.x(), y: group.y(),
        }));
      }
    });

    stage.on("mousedown touchstart", (event) => {
      if (event.target !== stage) return;
      selectionStart = stage.getPointerPosition();
      marquee.setAttrs({ x: selectionStart.x, y: selectionStart.y, width: 0, height: 0, visible: true });
    });
    stage.on("mousemove touchmove", () => {
      if (!selectionStart) return;
      const position = stage.getPointerPosition();
      marquee.setAttrs({
        x: Math.min(selectionStart.x, position.x),
        y: Math.min(selectionStart.y, position.y),
        width: Math.abs(position.x - selectionStart.x),
        height: Math.abs(position.y - selectionStart.y),
      });
      layer.batchDraw();
    });
    stage.on("mouseup touchend", () => {
      if (!selectionStart) return;
      const box = marquee.getClientRect();
      const selected = group.getChildren()
        .filter((node) => Konva.Util.haveIntersection(box, node.getClientRect()))
        .map((node) => node.name());
      selectionStart = null;
      marquee.visible(false);
      setInteraction((value) => ({ ...value, selected }));
    });

    const cancel = (event) => {
      if (event.key !== "Escape" || !group.isDragging()) return;
      cancelled = true;
      group.stopDrag();
      group.position(dragOrigin);
      layer.batchDraw();
      setInteraction((value) => ({
        ...value, cancels: value.cancels + 1, x: dragOrigin.x, y: dragOrigin.y,
      }));
    };
    document.addEventListener("keydown", cancel);
    layer.add(group, marquee);
    stage.add(layer);
    return () => {
      document.removeEventListener("keydown", cancel);
      stage.destroy();
    };
  }, []);

  const recordComposition = (event) => setIme((value) => ({
    ...value,
    composing: event.type !== "compositionend",
    events: [...value.events, event.type],
  }));
  const handleKeyDown = (event) => {
    if (event.isComposing || ime.composing) return;
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      setIme((value) => ({ ...value, shortcutCount: value.shortcutCount + 1 }));
    }
  };

  return (
    <section className="interaction-section">
      <h2>Actual interaction / IME / a11y proxy</h2>
      <div ref={hostRef} className="interaction-stage" data-testid="interaction-stage" role="img"
        aria-label="Timeline keyframe surface. Use the adjacent list to inspect the selection." />
      <output data-testid="interaction-state">
        {JSON.stringify(interaction)}
      </output>
      <div className="capture-probe" data-testid="capture-probe"
        onPointerDown={(event) => {
          event.currentTarget.setPointerCapture(event.pointerId);
          setCapture((value) => ({ ...value, captured: true }));
        }}
        onPointerMove={(event) => {
          if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
          setCapture((value) => ({ ...value, moves: value.moves + 1 }));
        }}
        onPointerUp={(event) => event.currentTarget.releasePointerCapture(event.pointerId)}
        onLostPointerCapture={() => setCapture((value) => ({ ...value, captured: false }))}
        onPointerCancel={() => setCapture((value) => ({ ...value, captured: false, cancels: value.cancels + 1 }))}>
        Pointer capture probe
      </div>
      <output data-testid="capture-state">{JSON.stringify(capture)}</output>
      <label>
        Keyframe name
        <input data-testid="ime-input" onCompositionStart={recordComposition}
          onCompositionUpdate={recordComposition} onCompositionEnd={recordComposition}
          onKeyDown={handleKeyDown} />
      </label>
      <output data-testid="ime-state">{JSON.stringify(ime)}</output>
      <ul aria-label="Selected keyframes" data-testid="selection-proxy">
        {interaction.selected.length === 0
          ? <li>No keyframes selected</li>
          : <li><button type="button">{interaction.selected[0]}</button>; {interaction.selected.length} selected</li>}
      </ul>
    </section>
  );
}

function CommunitySandboxProbe() {
  const frameRef = useRef(null);
  const [result, setResult] = useState({ status: "pending" });
  const [broker, setBroker] = useState({ handled: 0, allowed: [], denied: [] });
  const brokerState = useRef({ handled: 0, allowed: [], denied: [] });
  const source = `<!doctype html><meta http-equiv="Content-Security-Policy"
    content="default-src 'none'; script-src 'unsafe-inline'; connect-src 'none'; img-src 'none'; style-src 'unsafe-inline'">
    <style>body{background:#101720;color:#e7edf4;font:14px system-ui}button{color:inherit;background:#182331}</style>
    <button>Community panel fixture</button><script>
    (async () => {
      const blocked = {};
      try { void parent.document.body; blocked.parentDocument = false; }
      catch (_) { blocked.parentDocument = true; }
      try { localStorage.setItem('g09', 'x'); blocked.storage = false; }
      catch (_) { blocked.storage = true; }
      try { await fetch('/__g0_9_forbidden'); blocked.network = false; }
      catch (_) { blocked.network = true; }
      blocked.nativeBridge = typeof window.__TAURI__ === 'undefined';
      parent.postMessage({ type: 'g0-9-sandbox-result', blocked }, '*');
      parent.postMessage({ type: 'g0-9-capability-request', capability: 'theme.read', requestId: 'theme-1' }, '*');
      parent.postMessage({ type: 'g0-9-capability-request', capability: 'document.raw', requestId: 'raw-1' }, '*');
      parent.postMessage({ type: 'g0-9-capability-request', capability: 'native.invoke', requestId: 'invoke-1' }, '*');
    })();
    <\/script>`;

  useEffect(() => {
    const receive = (event) => {
      if (event.source !== frameRef.current?.contentWindow || event.origin !== "null") return;
      if (event.data?.type === "g0-9-sandbox-result") {
        setResult({ status: "received", origin: event.origin, ...event.data.blocked });
        return;
      }
      if (event.data?.type !== "g0-9-capability-request") return;
      const encoded = JSON.stringify(event.data);
      const capability = event.data.capability;
      const allowed = encoded.length <= 1024 && capability === "theme.read";
      const next = {
        handled: brokerState.current.handled + 1,
        allowed: allowed
          ? [...brokerState.current.allowed, capability]
          : brokerState.current.allowed,
        denied: allowed
          ? brokerState.current.denied
          : [...brokerState.current.denied, capability],
      };
      brokerState.current = next;
      setBroker(next);
      event.source.postMessage(
        allowed
          ? { type: "g0-9-capability-result", requestId: event.data.requestId, value: { theme: "dark" } }
          : { type: "g0-9-capability-result", requestId: event.data.requestId, error: "denied" },
        "*",
      );
    };
    window.addEventListener("message", receive);
    return () => window.removeEventListener("message", receive);
  }, []);

  return (
    <section className="sandbox-section">
      <h2>Community sandbox negative probe</h2>
      <iframe ref={frameRef} title="Community panel sandbox" sandbox="allow-scripts" srcDoc={source} />
      <output data-testid="sandbox-state">{JSON.stringify(result)}</output>
      <output data-testid="capability-broker-state">{JSON.stringify(broker)}</output>
    </section>
  );
}

function App() {
  const [revision, setRevision] = useState(hmrRevision);
  if (import.meta.hot) {
    import.meta.hot.accept("virtual:g0-9-hmr-probe", (module) => setRevision(module.revision));
  }
  return (
    <main>
      <header>
        <h1>Motolii G0-9 isolated Web UI spike</h1>
        <output data-testid="hmr-revision">{revision}</output>
      </header>
      <div className="grid">
        <BrowserProbe />
        <TimelineProbe />
      </div>
      <InteractionProbe />
      <ObjectHandleProbe />
      <SpatialGizmoProbe />
      <CommunitySandboxProbe />
    </main>
  );
}

createRoot(document.getElementById("root")).render(<App />);
