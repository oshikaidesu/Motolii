import React, { useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { revision as hmrRevision } from "virtual:g0-9-hmr-probe";
import { browserItem, createTimelineFixture, fixture } from "./fixture.js";
import { TimelineRenderer } from "./timeline-renderer.js";
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
  const rendererRef = useRef(null);
  const model = useMemo(() => createTimelineFixture(), []);
  const [gpuStatus, setGpuStatus] = useState("pending");

  useEffect(() => {
    const renderer = new TimelineRenderer(canvasRef.current, model);
    rendererRef.current = renderer;
    renderer.drawCanvas(0, 48);
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
    };
    setGpuStatus(navigator.gpu ? "not initialized" : "navigator.gpu unavailable");
  }, [model]);

  return (
    <section>
      <h2>Timeline — one dense surface</h2>
      <canvas ref={canvasRef} width="1200" height="512" data-testid="timeline" />
      <p>1,000 clips / 100,000 keys; DOM children: canvas only</p>
      <output data-testid="webgpu-status">{gpuStatus}</output>
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
    </main>
  );
}

createRoot(document.getElementById("root")).render(<App />);
