import {
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  cloneKeys,
  constrainHandle,
  createGraphLayout,
  DEFAULT_GRAPH_LAYOUT,
  GRAPH_VIEW_RANGE,
  pathFor,
  pointFromClient,
  splitSegment,
  xOf,
  yOf,
} from "./graph-view-model.js";
import "./graph-view-candidate.css";

const VIEW = GRAPH_VIEW_RANGE;

const CHANNELS = [
  {
    id: "intensity",
    object: "Pulse rings",
    parameter: "Intensity",
    unit: "%",
    color: "primary",
    keys: [
      {
        id: "i0",
        time: 52.18,
        value: 18,
        in: null,
        out: { time: 52.56, value: 18 },
      },
      {
        id: "i1",
        time: 53.24,
        value: 82,
        in: { time: 52.86, value: 82 },
        out: { time: 53.62, value: 82 },
      },
      {
        id: "i2",
        time: 54.48,
        value: 36,
        in: { time: 54.08, value: 36 },
        out: { time: 54.86, value: 36 },
      },
      {
        id: "i3",
        time: 55.62,
        value: 78,
        in: { time: 55.22, value: 78 },
        out: null,
      },
    ],
  },
  {
    id: "spread",
    object: "Pulse rings",
    parameter: "Spread",
    unit: "%",
    color: "context-a",
    keys: [
      {
        id: "s0",
        time: 52.18,
        value: 34,
        in: null,
        out: { time: 52.72, value: 34 },
      },
      {
        id: "s1",
        time: 54.06,
        value: 64,
        in: { time: 53.52, value: 64 },
        out: { time: 54.42, value: 64 },
      },
      {
        id: "s2",
        time: 55.62,
        value: 42,
        in: { time: 55.18, value: 42 },
        out: null,
      },
    ],
  },
  {
    id: "depth",
    object: "City grid",
    parameter: "Depth",
    unit: "z",
    color: "context-b",
    keys: [
      {
        id: "d0",
        time: 52.18,
        value: 58,
        in: null,
        out: { time: 52.86, value: 58 },
      },
      {
        id: "d1",
        time: 55.62,
        value: 28,
        in: { time: 54.94, value: 28 },
        out: null,
      },
    ],
  },
];

function pointFromEvent(event, layout) {
  const svg = event.currentTarget.ownerSVGElement ?? event.currentTarget;
  const rect = svg.getBoundingClientRect();
  return pointFromClient(event.clientX, event.clientY, rect, layout, VIEW);
}

export function GraphViewCandidate({ docked = false }) {
  const [curves, setCurves] = useState(() =>
    Object.fromEntries(
      CHANNELS.map((channel) => [channel.id, cloneKeys(channel.keys)]),
    ),
  );
  const [activeId, setActiveId] = useState("intensity");
  const [selectedKeys, setSelectedKeys] = useState(() => new Set(["i1"]));
  const [snapshot, setSnapshot] = useState(null);
  const [status, setStatus] = useState("Intensity · 4 keys");
  const [drag, setDrag] = useState(null);
  const [layout, setLayout] = useState(DEFAULT_GRAPH_LAYOUT);
  const canvasWrapRef = useRef(null);
  const curvesRef = useRef(curves);
  curvesRef.current = curves;
  const activeChannel = CHANNELS.find((channel) => channel.id === activeId);
  const activeKeys = curves[activeId];
  const activePath = useMemo(
    () => pathFor(activeKeys, layout, VIEW),
    [activeKeys, layout],
  );
  const projectX = (time) => xOf(time, layout, VIEW);
  const projectY = (value) => yOf(value, layout, VIEW);
  const curvePath = (keys) => pathFor(keys, layout, VIEW);

  useLayoutEffect(() => {
    const target = canvasWrapRef.current;
    if (!target) return undefined;
    const updateLayout = () => {
      const rect = target.getBoundingClientRect();
      setLayout(createGraphLayout(rect.width, rect.height));
    };
    updateLayout();
    const observer = new ResizeObserver(updateLayout);
    observer.observe(target);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!drag) return undefined;
    const cancel = (event) => {
      if (event.key !== "Escape") return;
      setCurves(drag.startCurves);
      setDrag(null);
      setStatus("Cancel · no changes");
    };
    window.addEventListener("keydown", cancel);
    return () => window.removeEventListener("keydown", cancel);
  }, [drag]);

  function focusChannel(id) {
    setActiveId(id);
    const channel = CHANNELS.find((entry) => entry.id === id);
    setSelectedKeys(new Set([curves[id][0].id]));
    setSnapshot(null);
    setStatus(`${channel.parameter} · ${curves[id].length} keys`);
  }

  function toggleSnapshot() {
    if (snapshot) {
      setSnapshot(null);
      setStatus("Snapshot hidden");
      return;
    }
    setSnapshot({ channelId: activeId, keys: cloneKeys(activeKeys) });
    setStatus("Curve snapshot · view only");
  }

  function selectKey(event, keyId) {
    event.stopPropagation();
    setSelectedKeys((current) => {
      if (!event.shiftKey) return new Set([keyId]);
      const next = new Set(current);
      if (next.has(keyId)) next.delete(keyId);
      else next.add(keyId);
      return next;
    });
  }

  function beginDrag(event, kind, keyId, side = null) {
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrag({
      kind,
      keyId,
      side,
      startCurves: Object.fromEntries(
        Object.entries(curvesRef.current).map(([id, keys]) => [
          id,
          cloneKeys(keys),
        ]),
      ),
    });
    setStatus(
      kind === "key"
        ? "Key preview"
        : event.shiftKey
          ? "Broken tangent preview"
          : "Tangent preview",
    );
  }

  function moveDrag(event) {
    if (!drag) return;
    const point = pointFromEvent(event, layout);
    const startKeys = drag.startCurves[activeId];
    const index = startKeys.findIndex((key) => key.id === drag.keyId);
    if (index < 0) return;
    const startKey = startKeys[index];

    if (drag.kind === "key") {
      const previousTime =
        index === 0 ? VIEW.timeStart : startKeys[index - 1].time + 0.02;
      const nextTime =
        index === startKeys.length - 1
          ? VIEW.timeEnd
          : startKeys[index + 1].time - 0.02;
      const nextTimeValue = Math.max(
        previousTime,
        Math.min(nextTime, point.time),
      );
      const nextValue = Math.max(
        VIEW.valueMin,
        Math.min(VIEW.valueMax, point.value),
      );
      const deltaTime = nextTimeValue - startKey.time;
      const deltaValue = nextValue - startKey.value;
      const nextKeys = cloneKeys(startKeys);
      nextKeys[index] = {
        ...nextKeys[index],
        time: nextTimeValue,
        value: nextValue,
        in: startKey.in
          ? {
              time: startKey.in.time + deltaTime,
              value: startKey.in.value + deltaValue,
            }
          : null,
        out: startKey.out
          ? {
              time: startKey.out.time + deltaTime,
              value: startKey.out.value + deltaValue,
            }
          : null,
      };
      setCurves((current) => ({ ...current, [activeId]: nextKeys }));
      return;
    }

    const origin = { time: startKey.time, value: startKey.value };
    const original = startKey[drag.side];
    const handle = constrainHandle(point, origin, original, event);
    const nextKeys = cloneKeys(startKeys);
    nextKeys[index][drag.side] = handle;
    const oppositeSide = drag.side === "in" ? "out" : "in";
    const opposite = startKey[oppositeSide];
    if (!event.shiftKey && opposite) {
      const length = Math.hypot(
        opposite.time - origin.time,
        opposite.value - origin.value,
      );
      const angle =
        Math.atan2(handle.value - origin.value, handle.time - origin.time) +
        Math.PI;
      nextKeys[index][oppositeSide] = {
        time: origin.time + Math.cos(angle) * length,
        value: origin.value + Math.sin(angle) * length,
      };
    }
    setCurves((current) => ({ ...current, [activeId]: nextKeys }));
  }

  function commitDrag(event) {
    if (!drag) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    setDrag(null);
    setStatus(`${drag.kind === "key" ? "Key" : "Tangent"} committed · Undo 1`);
  }

  function addKeyOnCurve(event) {
    const point = pointFromEvent(event, layout);
    const keys = curvesRef.current[activeId];
    const segmentIndex = keys.findIndex(
      (key, index) =>
        index < keys.length - 1 &&
        point.time >= key.time &&
        point.time <= keys[index + 1].time,
    );
    if (segmentIndex < 0) return;
    const left = keys[segmentIndex];
    const right = keys[segmentIndex + 1];
    const amount = Math.max(
      0.02,
      Math.min(0.98, (point.time - left.time) / (right.time - left.time)),
    );
    const nextKeys = splitSegment(
      keys,
      segmentIndex,
      amount,
      `key-${Date.now()}`,
    );
    const inserted = nextKeys[segmentIndex + 1];
    setCurves((current) => ({ ...current, [activeId]: nextKeys }));
    setSelectedKeys(new Set([inserted.id]));
    setStatus("Key added · curve preserved · Undo 1");
  }

  const timeTicks = [52, 52.5, 53, 53.5, 54, 54.5, 55, 55.5, 56];
  const valueTicks = [0, 25, 50, 75, 100];

  return (
    <main
      className={`graph-view-candidate${docked ? " is-docked" : ""}`}
      data-react-surface="graph-view"
      data-docked={docked ? "true" : "false"}
      data-view-time={`${VIEW.timeStart}-${VIEW.timeEnd}`}
      data-view-value={`${VIEW.valueMin}-${VIEW.valueMax}`}
    >
      <header className="graph-view-topbar">
        <div className="graph-view-title">
          <span className="graph-view-mark" aria-hidden="true">⌁</span>
          <b>GRAPH</b>
          <span>{activeChannel.object} / {activeChannel.parameter}</span>
        </div>
        <div className="graph-view-tools" role="toolbar" aria-label="Graph View tools">
          <button type="button" aria-label="Edit keyframes" aria-pressed="true">↖</button>
          <button
            type="button"
            aria-label="Frame selected"
            onClick={() => setStatus("Frame selected · view only")}
          >
            ⛶
          </button>
          <button
            type="button"
            aria-label="Curve snapshot"
            aria-pressed={Boolean(snapshot)}
            onClick={toggleSnapshot}
          >
            ≋
          </button>
          <button
            type="button"
            aria-label="Open Interval Easing Editor"
            onClick={() => setStatus("Interval Easing Editor shortcut")}
          >
            ⤴
          </button>
        </div>
        <div className="graph-view-mode" aria-label="Graph value display">
          <button type="button" aria-pressed="true">ABS</button>
          <button
            type="button"
            aria-label="Normalize display"
            aria-pressed="false"
            onClick={() => setStatus("Normalize display · view only")}
          >
            ±1
          </button>
        </div>
      </header>

      <section className="graph-view-body">
        <aside className="graph-channel-list" aria-label="Animated parameters">
          <div className="graph-channel-heading">
            <span>PARAMETERS</span>
            <button type="button" aria-label="Filter parameters">⌕</button>
          </div>
          {CHANNELS.map((channel) => (
            <button
              type="button"
              className="graph-channel-row"
              data-channel={channel.id}
              data-color={channel.color}
              aria-pressed={activeId === channel.id}
              key={channel.id}
              onClick={() => focusChannel(channel.id)}
            >
              <i aria-hidden="true" />
              <span>
                <small>{channel.object}</small>
                <b>{channel.parameter}</b>
              </span>
              <em>{curves[channel.id].length}</em>
            </button>
          ))}
        </aside>

        <div className="graph-canvas-wrap" ref={canvasWrapRef}>
          <svg
            className="graph-canvas"
            viewBox={`0 0 ${layout.width} ${layout.height}`}
            aria-label={`${activeChannel.parameter} value over time`}
            onPointerMove={moveDrag}
          >
            <rect
              className="graph-plot-bg"
              x={layout.left}
              y={layout.top}
              width={layout.width - layout.left - layout.right}
              height={layout.height - layout.top - layout.bottom}
            />
            {valueTicks.map((value) => (
              <g key={`value-${value}`}>
                <line
                  className="graph-grid graph-grid-value"
                  x1={layout.left}
                  x2={layout.width - layout.right}
                  y1={projectY(value)}
                  y2={projectY(value)}
                />
                <text x={layout.left - 10} y={projectY(value) + 4}>
                  {value}
                </text>
              </g>
            ))}
            {timeTicks.map((time) => (
              <g key={`time-${time}`}>
                <line
                  className={`graph-grid ${
                    Number.isInteger(time) ? "graph-grid-major" : ""
                  }`}
                  x1={projectX(time)}
                  x2={projectX(time)}
                  y1={layout.top}
                  y2={layout.height - layout.bottom}
                />
                <text
                  className="graph-time-label"
                  x={projectX(time)}
                  y={layout.height - 9}
                >
                  {Number.isInteger(time) ? time : "·"}
                </text>
              </g>
            ))}

            {CHANNELS.filter((channel) => channel.id !== activeId).map(
              (channel) => (
                <path
                  key={channel.id}
                  className="graph-context-curve"
                  data-color={channel.color}
                  d={curvePath(curves[channel.id])}
                />
              ),
            )}

            {snapshot?.channelId === activeId ? (
              <path
                className="graph-snapshot-curve"
                aria-label="Curve snapshot"
                d={curvePath(snapshot.keys)}
              />
            ) : null}

            <path
              className="graph-primary-curve"
              data-testid="active-curve"
              d={activePath}
              onDoubleClick={addKeyOnCurve}
            />

            {activeKeys.map((key) => {
              const selected = selectedKeys.has(key.id);
              return (
                <g
                  key={key.id}
                  className={`graph-key-group${selected ? " selected" : ""}`}
                >
                  {selected && key.in ? (
                    <line
                      className="graph-tangent-line"
                      x1={projectX(key.time)}
                      y1={projectY(key.value)}
                      x2={projectX(key.in.time)}
                      y2={projectY(key.in.value)}
                    />
                  ) : null}
                  {selected && key.out ? (
                    <line
                      className="graph-tangent-line"
                      x1={projectX(key.time)}
                      y1={projectY(key.value)}
                      x2={projectX(key.out.time)}
                      y2={projectY(key.out.value)}
                    />
                  ) : null}
                  {selected && key.in ? (
                    <circle
                      className="graph-tangent-handle"
                      data-handle={`${key.id}-in`}
                      cx={projectX(key.in.time)}
                      cy={projectY(key.in.value)}
                      r="9"
                      role="slider"
                      tabIndex="0"
                      aria-label={`${activeChannel.parameter} key incoming tangent`}
                      onPointerDown={(event) =>
                        beginDrag(event, "handle", key.id, "in")
                      }
                      onPointerUp={commitDrag}
                      onPointerCancel={() => {
                        if (!drag) return;
                        setCurves(drag.startCurves);
                        setDrag(null);
                      }}
                    />
                  ) : null}
                  {selected && key.out ? (
                    <circle
                      className="graph-tangent-handle"
                      data-handle={`${key.id}-out`}
                      cx={projectX(key.out.time)}
                      cy={projectY(key.out.value)}
                      r="9"
                      role="slider"
                      tabIndex="0"
                      aria-label={`${activeChannel.parameter} key outgoing tangent`}
                      onPointerDown={(event) =>
                        beginDrag(event, "handle", key.id, "out")
                      }
                      onPointerUp={commitDrag}
                      onPointerCancel={() => {
                        if (!drag) return;
                        setCurves(drag.startCurves);
                        setDrag(null);
                      }}
                    />
                  ) : null}
                  <circle
                    className="graph-key"
                    data-key-id={key.id}
                    cx={projectX(key.time)}
                    cy={projectY(key.value)}
                    r="10"
                    role="button"
                    tabIndex="0"
                    aria-label={`${activeChannel.parameter} key at ${key.time.toFixed(
                      2,
                    )}, ${key.value.toFixed(1)}${activeChannel.unit}`}
                    aria-pressed={selected}
                    onClick={(event) => selectKey(event, key.id)}
                    onPointerDown={(event) =>
                      beginDrag(event, "key", key.id)
                    }
                    onPointerUp={commitDrag}
                    onPointerCancel={() => {
                      if (!drag) return;
                      setCurves(drag.startCurves);
                      setDrag(null);
                    }}
                  />
                </g>
              );
            })}

            <line
              className="graph-playhead-line"
              x1={projectX(54.2)}
              x2={projectX(54.2)}
              y1={layout.top}
              y2={layout.height - layout.bottom}
            />
            <path
              className="graph-playhead-head"
              d={`M ${projectX(54.2) - 7} ${layout.top} H ${
                projectX(54.2) + 7
              } L ${projectX(54.2)} ${layout.top + 10} Z`}
            />
          </svg>
          <div className="graph-axis-unit">{activeChannel.unit}</div>
        </div>
      </section>

      <footer className="graph-view-status">
        <span role="status" aria-live="polite">{status}</span>
        <b>00:54.2</b>
      </footer>
    </main>
  );
}
