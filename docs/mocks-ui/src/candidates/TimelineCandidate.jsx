import { useEffect, useMemo, useRef, useState } from "react";
import "./timeline-candidate.css";

const OBJECTS = [
  {
    id: "song",
    band: 0,
    kind: "♪",
    name: "night_drive.wav",
    colorSlot: 1,
    left: 1,
    width: 98,
    automation: [
      { channel: "Volume", keys: [18, 43, 71], easing: "Smooth" },
    ],
    content: (
      <>
        <b>night_drive.wav</b>
        <span className="candidate-song-wave">
          ╱╲╱▁╲╱╲▁╱╲╱╲▁╱╲╱╲▁╱╲╱╲▁╱╲
        </span>
      </>
    ),
  },
  {
    id: "pulse-rings",
    band: 1,
    kind: "G",
    name: "Pulse rings",
    depthCapable: true,
    depthLabel: "Pulse",
    depth: 0,
    isGroup: true,
    children: ["city-grid"],
    colorSlot: 2,
    left: 4,
    width: 88,
    selected: true,
    automation: [
      { channel: "Intensity", keys: [28, 54, 76], easing: "Smooth" },
      { channel: "Spread", keys: [34, 68], easing: "Ease Out" },
      { channel: "Depth", keys: [22, 61], easing: "Linear" },
    ],
    content: (
      <>
        <b className="group-label">Pulse rings</b>
        <span className="fx-flow">
          <b>IN</b><span className="arrow">→</span>
          <i id="plugin-flow-name">Echo Bloom</i>
          <span className="arrow">→</span><b>OUT</b>
        </span>
        <span
          className="readiness"
          id="plugin-readiness"
          aria-label="ready, rendering, stale"
        >
          <i className="ready" style={{ width: "46%" }} />
          <i className="rendering" style={{ width: "21%" }} />
          <i className="stale" style={{ width: "33%" }} />
        </span>
      </>
    ),
  },
  {
    id: "night-drive",
    band: 3,
    kind: "T",
    name: "NIGHT DRIVE",
    depthCapable: true,
    depthLabel: "Text",
    depth: 0,
    colorSlot: 3,
    left: 27,
    width: 38,
    automation: [
      { channel: "Opacity", keys: [32, 78], easing: "Ease In" },
    ],
    content: (
      <>
        <b>NIGHT DRIVE</b>
      </>
    ),
  },
  {
    id: "city-grid",
    band: 2,
    kind: "S",
    name: "City grid",
    depthCapable: true,
    depthLabel: "Grid",
    depth: 0,
    parentId: "pulse-rings",
    colorSlot: 4,
    left: 8,
    width: 79,
    automation: [
      { channel: "Depth", keys: [24, 66], easing: "Smooth" },
    ],
    content: (
      <>
        <span className="parent-label">↳ Pulse rings</span>
        <b>City grid</b>
      </>
    ),
  },
  {
    id: "city-loop",
    band: 4,
    kind: "V",
    name: "neon_reflection.mp4",
    depthCapable: true,
    depthLabel: "Reflection",
    depth: 0,
    colorSlot: 5,
    left: 16,
    width: 31,
    automation: [],
    content: <b>▧ city_loop.mp4</b>,
  },
  {
    id: "traffic-pass",
    band: 4,
    kind: "V",
    name: "traffic_pass.mp4",
    depthCapable: true,
    depthLabel: "Traffic",
    depth: 0,
    colorSlot: 6,
    left: 51,
    width: 37,
    automation: [],
    content: <b>▧ traffic_pass.mp4</b>,
  },
];

const BAND_COUNT =
  Math.max(...OBJECTS.map((object) => object.band)) + 1;

const ALL_AUTOMATION_CHANNELS = [
  "Opacity",
  "Position X",
  "Position Y",
  "Scale",
  "Rotation",
  "Depth",
  "Volume",
  "Intensity",
  "Spread",
];

const TICKS = [
  ["52", 0, true],
  ["52.2", 12.5, false],
  ["53", 25, true],
  ["53.2", 37.5, false],
  ["54", 50, true],
  ["54.2", 62.5, false],
  ["55", 75, true],
  ["55.2", 87.5, false],
  ["56", 100, true],
];

const DEPTH_MIN = -0.5;
const DEPTH_MAX = 0.5;

function formatDepth(value) {
  if (Math.abs(value) < 0.005) return "0";
  const fixed = Math.abs(value).toFixed(2).replace(/^0/, "");
  return `${value > 0 ? "+" : "−"}${fixed}`;
}

function depthToPercent(value) {
  return (
    ((Math.max(DEPTH_MIN, Math.min(DEPTH_MAX, value)) - DEPTH_MIN) /
      (DEPTH_MAX - DEPTH_MIN)) *
    100
  );
}

function distributeDepth(objects, farDepth, nearDepth, reversed) {
  const ordered = reversed ? [...objects].reverse() : objects;
  return new Map(
    ordered.map((object, index) => {
      const ratio =
        ordered.length <= 1 ? 0.5 : index / (ordered.length - 1);
      return [
        object.id,
        nearDepth + (farDepth - nearDepth) * ratio,
      ];
    }),
  );
}

function ObjectStateControls({ object, muted, soloed, onMute, onSolo }) {
  return (
    <span className="candidate-object-state" aria-label={`${object.name} controls`}>
      <button
        type="button"
        className="candidate-sm candidate-solo"
        aria-label={`${object.name}をSolo`}
        aria-pressed={soloed}
        onClick={onSolo}
      >
        S
      </button>
      <button
        type="button"
        className="candidate-sm candidate-mute"
        aria-label={`${object.name}をMute`}
        aria-pressed={muted}
        onClick={onMute}
      >
        M
      </button>
    </span>
  );
}

function GroupFoldButton({ object, expanded, onClick }) {
  return (
    <button
      type="button"
      className="candidate-group-fold"
      aria-label={`${object.name}を${expanded ? "折り畳む" : "展開する"}`}
      aria-expanded={expanded}
      title={expanded ? "Groupを折り畳む" : "Groupをその場で展開"}
      onClick={onClick}
    >
      <span aria-hidden="true">{expanded ? "▾" : "▸"}</span>
      <small aria-hidden="true">{object.children.length}</small>
    </button>
  );
}

function BandStateButton({ band, kind, state, onClick }) {
  const label = kind === "solo" ? "Solo" : "Mute";
  return (
    <button
      type="button"
      className={`candidate-band-sm candidate-band-${kind}`}
      aria-label={`帯${band + 1}上の全Objectを${label}`}
      aria-pressed={state === "mixed" ? "mixed" : state === "on"}
      data-state={state}
      title={`帯${band + 1}上の全Objectへ${label}を一括適用`}
      onClick={onClick}
    >
      {kind === "solo" ? "S" : "M"}
    </button>
  );
}

function AutomationTrigger({ object, count, expanded, active, onClick }) {
  return (
    <button
      type="button"
      className={`candidate-automation-trigger${active ? " is-active" : ""}`}
      aria-label={`${object.name}のAutomationを開く · ${count} channel`}
      aria-expanded={expanded}
      title={
        count > 0
          ? `Automation済みchannelを表示 · ${count}`
          : "Automationを追加"
      }
      onClick={onClick}
    >
      {count > 0 ? `◆ ${count}` : "◇＋"}
    </button>
  );
}

function DepthRail({
  depthByObject,
  depthOpen,
  depthScope,
  depthToolOpen,
  farDepth,
  focusedDepthObject,
  nearDepth,
  reversedDepthOrder,
  selectedObjects,
  onApplyDistribution,
  onCancelDistribution,
  onFarDepthChange,
  onNearDepthChange,
  onOpenDistribution,
  onReverseDepthOrder,
  onSelectObject,
  onSetScope,
}) {
  const scopeObjects = OBJECTS.filter(
    (object) =>
      object.depthCapable &&
      (depthScope === "root"
        ? !object.parentId
        : object.parentId === depthScope),
  );
  const allSelectedDepthObjects = OBJECTS.filter(
    (object) => object.depthCapable && selectedObjects.has(object.id),
  );
  const selectedScopeObjects = scopeObjects.filter((object) =>
    selectedObjects.has(object.id),
  );
  const selectedParents = new Set(
    allSelectedDepthObjects.map((object) => object.parentId ?? "root"),
  );
  const hasMixedParents = selectedParents.size > 1;
  const canDistribute =
    selectedScopeObjects.length > 1 && !hasMixedParents;
  const previewDepths =
    depthToolOpen && canDistribute
      ? distributeDepth(
          selectedScopeObjects,
          farDepth,
          nearDepth,
          reversedDepthOrder,
        )
      : new Map();
  const visibleDepth = (object) =>
    previewDepths.get(object.id) ?? depthByObject[object.id] ?? 0;
  const depthGroups = new Map();
  scopeObjects.forEach((object) => {
    const value = visibleDepth(object);
    const key = value.toFixed(4);
    depthGroups.set(key, [
      ...(depthGroups.get(key) ?? []),
      { object, value },
    ]);
  });
  const scopeGroup =
    depthScope === "root"
      ? null
      : OBJECTS.find((object) => object.id === depthScope);
  const focusedObject = OBJECTS.find(
    (object) => object.id === focusedDepthObject,
  );

  return (
    <div
      className="z-rail"
      id="depth-rail"
      aria-label="Depth Rail · Edit-Space Z"
      aria-hidden={!depthOpen}
    >
      <div className="z-head">
        <b>DEPTH</b>
        <button
          type="button"
          className="candidate-depth-scope"
          aria-label={`Depth scope: ${
            scopeGroup ? `ROOT / ${scopeGroup.name}` : "ROOT"
          }`}
          title="同じparentのEdit-Space Zだけを表示"
          onClick={() => onSetScope("root")}
        >
          {scopeGroup ? `ROOT / ${scopeGroup.name}` : "ROOT"}
        </button>
        <button
          type="button"
          className="depth-key"
          id="depth-key"
          aria-pressed="false"
          aria-label="現在時刻にDepth keyを追加"
        >
          ◇
        </button>
        <small id="z-readout">
          {focusedObject
            ? `${focusedObject.name} ${formatDepth(
                depthByObject[focusedObject.id] ?? 0,
              )}`
            : "Z / EDIT SPACE"}
        </small>
      </div>
      <div className="z-plot">
        <div className="z-scale">
          <span>−.50</span><span>−.25</span><span>0</span><span>+.25</span><span>+.50</span>
        </div>
        <div className="z-depth-lane candidate-depth-scope-lane">
          <span className="z-lane-name">
            {scopeGroup ? "CHILD" : "ROOT"}
          </span>
          <div className="z-axis" id="z-axis" />
          {[...depthGroups.values()].map((entries) => {
            const { value } = entries[0];
            const selectedEntries = entries.filter(({ object }) =>
              selectedObjects.has(object.id),
            );
            const focusedEntry =
              entries.find(
                ({ object }) => object.id === focusedDepthObject,
              ) ??
              selectedEntries[0] ??
              entries[0];
            const isStack = entries.length > 1;
            return (
              <button
                type="button"
                className={`z-marker candidate-depth-marker${
                  selectedEntries.length > 0 ? " selected" : ""
                }${isStack ? " is-stack" : ""}${
                  depthToolOpen && previewDepths.has(focusedEntry.object.id)
                    ? " is-preview"
                    : ""
                }`}
                aria-label={
                  isStack
                    ? `Depth ${formatDepth(value)} に${entries.length} Object。focus: ${focusedEntry.object.name}`
                    : `${focusedEntry.object.name} · Depth ${formatDepth(value)}`
                }
                data-depth-count={entries.length}
                data-object-id={focusedEntry.object.id}
                data-z={value.toFixed(3)}
                key={`${depthScope}-${value.toFixed(4)}`}
                style={{ left: `${depthToPercent(value)}%` }}
                onClick={() => onSelectObject(focusedEntry.object.id)}
              >
                {isStack
                  ? `${formatDepth(value)} × ${entries.length}`
                  : `${focusedEntry.object.depthLabel} ${formatDepth(value)}`}
              </button>
            );
          })}
          {depthScope === "root" ? (
            <span
              className="z-marker camera-marker"
              data-z="0.42"
              style={{ left: "92%" }}
            >
              CAM +.42 →
            </span>
          ) : null}
          {depthToolOpen && canDistribute ? (
            <div
              className="candidate-depth-range-band"
              aria-label={`配布区間 ${formatDepth(farDepth)} から ${formatDepth(nearDepth)}`}
              style={{
                left: `${depthToPercent(farDepth)}%`,
                width: `${Math.max(
                  0,
                  depthToPercent(nearDepth) -
                    depthToPercent(farDepth),
                )}%`,
              }}
            />
          ) : null}
        </div>
        {depthToolOpen ? (
          <div className="candidate-depth-range-controls">
            <label>
              奥
              <input
                type="range"
                aria-label="Depth配布の奥端"
                min={DEPTH_MIN}
                max={nearDepth - 0.05}
                step="0.05"
                value={farDepth}
                onChange={(event) =>
                  onFarDepthChange(Number(event.target.value))
                }
              />
              <output>{formatDepth(farDepth)}</output>
            </label>
            <label>
              手前
              <input
                type="range"
                aria-label="Depth配布の手前端"
                min={farDepth + 0.05}
                max={DEPTH_MAX}
                step="0.05"
                value={nearDepth}
                onChange={(event) =>
                  onNearDepthChange(Number(event.target.value))
                }
              />
              <output>{formatDepth(nearDepth)}</output>
            </label>
          </div>
        ) : null}
      </div>
      <div className="z-tools candidate-depth-tools">
        <button
          type="button"
          aria-label="Layer Order Distributeを開く"
          aria-expanded={depthToolOpen}
          disabled={!canDistribute}
          title={
            hasMixedParents
              ? "同じparentのObjectだけを選択してください"
              : "選択Objectを奥端・手前端へLayer Orderで配布"
          }
          onClick={onOpenDistribution}
        >
          ⇥≋⇤
        </button>
        {depthToolOpen ? (
          <>
            <button
              type="button"
              aria-label="Depth配布順を反転"
              aria-pressed={reversedDepthOrder}
              onClick={onReverseDepthOrder}
            >
              ⇄
            </button>
            <button
              type="button"
              aria-label="Depth配布を適用"
              onClick={() => onApplyDistribution(previewDepths)}
            >
              ✓
            </button>
            <button
              type="button"
              aria-label="Depth配布をキャンセル"
              onClick={onCancelDistribution}
            >
              ×
            </button>
          </>
        ) : (
          <span>FIT</span>
        )}
      </div>
    </div>
  );
}

export function TimelineCandidate({
  EasingGraphComponent,
  GraphViewComponent,
  legacyCurveShelf,
  resizeHandle,
}) {
  const [packHeights, setPackHeights] = useState(() =>
    Array.from({ length: BAND_COUNT }, () => 34),
  );
  const [muted, setMuted] = useState(() => new Set());
  const [soloed, setSoloed] = useState(() => new Set());
  const [automationByObject, setAutomationByObject] = useState(() =>
    Object.fromEntries(
      OBJECTS.map((object) => [
        object.id,
        object.automation.map((automation) => ({ ...automation })),
      ]),
    ),
  );
  const [automationMenu, setAutomationMenu] = useState(null);
  const [automationQuery, setAutomationQuery] = useState("");
  const [expandedAutomation, setExpandedAutomation] = useState(
    () => new Set(),
  );
  const [focusedAutomation, setFocusedAutomation] = useState({
    objectId: "pulse-rings",
    channel: "Intensity",
  });
  const [expandedGroups, setExpandedGroups] = useState(
    () => new Set(["pulse-rings"]),
  );
  const [selectedKeys, setSelectedKeys] = useState(() => new Set());
  const [selectedObjects, setSelectedObjects] = useState(
    () => new Set(["pulse-rings"]),
  );
  const [focusedDepthObject, setFocusedDepthObject] =
    useState("pulse-rings");
  const [depthByObject, setDepthByObject] = useState(() =>
    Object.fromEntries(
      OBJECTS.filter((object) => object.depthCapable).map((object) => [
        object.id,
        object.depth,
      ]),
    ),
  );
  const [depthScope, setDepthScope] = useState("root");
  const [depthOpen, setDepthOpen] = useState(false);
  const [depthToolOpen, setDepthToolOpen] = useState(false);
  const [farDepth, setFarDepth] = useState(-0.25);
  const [nearDepth, setNearDepth] = useState(0.25);
  const [reversedDepthOrder, setReversedDepthOrder] =
    useState(false);
  const [objectOffsets, setObjectOffsets] = useState(() =>
    Object.fromEntries(OBJECTS.map((object) => [object.id, 0])),
  );
  const [keyToolsMode, setKeyToolsMode] = useState("keys");
  const [keyToolsOpen, setKeyToolsOpen] = useState(true);
  const [keyScope, setKeyScope] = useState("object");
  const [keySection, setKeySection] = useState("align");
  const [layerSection, setLayerSection] = useState("align");
  const [timelineView, setTimelineView] = useState("timeline");
  const playheadLeft = 62.5;
  const packDrag = useRef(null);
  const [draggingBand, setDraggingBand] = useState(null);
  const [hoveredBand, setHoveredBand] = useState(null);
  const hasSolo = soloed.size > 0;
  const visibleObjects = useMemo(
    () =>
      OBJECTS.filter(
        (object) =>
          !object.parentId || expandedGroups.has(object.parentId),
      ),
    [expandedGroups],
  );
  const packLayout = useMemo(() => {
    let top = 0;
    const visibleBandIds = [
      ...new Set(visibleObjects.map((object) => object.band)),
    ].sort((a, b) => a - b);
    return visibleBandIds.map((bandId) => {
      const baseHeight = packHeights[bandId];
      const expandedRows = visibleObjects
        .filter(
          (object) =>
            object.band === bandId &&
            expandedAutomation.has(object.id),
        )
        .map(
          (object) =>
            (automationByObject[object.id]?.length ?? 0) + 1,
        );
      const expansion =
        expandedRows.length > 0
          ? Math.max(...expandedRows) * 20 + 4
          : 0;
      const height = baseHeight + expansion;
      const entry = {
        bandId,
        top,
        height,
        baseHeight,
        expansion,
        bottom: top + height,
      };
      top += height;
      return entry;
    });
  }, [
    automationByObject,
    expandedAutomation,
    packHeights,
    visibleObjects,
  ]);
  const packLayoutByBand = useMemo(
    () => new Map(packLayout.map((band) => [band.bandId, band])),
    [packLayout],
  );
  const packContentHeight =
    packLayout[packLayout.length - 1]?.bottom ?? 0;
  const activeInterval = useMemo(() => {
    const object = OBJECTS.find(
      (entry) => entry.id === focusedAutomation.objectId,
    );
    const channel = (
      automationByObject[focusedAutomation.objectId] ?? []
    ).find((entry) => entry.channel === focusedAutomation.channel);
    if (!object || !channel || channel.keys.length < 2) return null;
    const offset = objectOffsets[object.id] ?? 0;
    const localPlayhead =
      ((playheadLeft - object.left - offset) / object.width) * 100;
    const sortedKeys = [...channel.keys].sort((left, right) => left - right);
    const startIndex = sortedKeys.findIndex(
      (position, index) =>
        index < sortedKeys.length - 1 &&
        localPlayhead > position &&
        localPlayhead < sortedKeys[index + 1],
    );
    if (startIndex < 0) return null;
    return {
      objectId: object.id,
      objectName: object.name,
      channel: channel.channel,
      startIndex,
      keyCount: sortedKeys.length,
    };
  }, [automationByObject, focusedAutomation, objectOffsets]);

  // Preview transportはまだlegacy fixtureが所有しているため、React Timelineの
  // focus区間をGraph入口へ一方向に投影する。旧inline-key探索は使わない。
  useEffect(() => {
    const button = document.querySelector("#interval-easing");
    if (!button) return undefined;
    const syncButton = () => {
      button.disabled = !activeInterval;
      button.classList.toggle("on", Boolean(activeInterval));
      button.setAttribute(
        "aria-label",
        activeInterval
          ? `${activeInterval.objectName} · ${activeInterval.channel}のInterval Easing Editorを開く`
          : "key間へ移動するとInterval Easing Editorを開けます",
      );
    };
    syncButton();
    // 親のlegacy初期化effectが初回だけ旧Timelineを読んだ後にも再投影する。
    const frame = window.requestAnimationFrame(syncButton);
    const openGraph = (event) => {
      if (!activeInterval) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      const panel = document.querySelector("#easing-panel");
      panel?.classList.add("open");
      panel?.setAttribute("aria-hidden", "false");
      button.setAttribute("aria-pressed", "true");
    };
    button.addEventListener("click", openGraph, true);
    return () => {
      window.cancelAnimationFrame(frame);
      button.removeEventListener("click", openGraph, true);
    };
  }, [activeInterval]);

  function toggle(setter, id) {
    setter((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function bandState(set, index) {
    const ids = visibleObjects
      .filter((object) => object.band === index)
      .map((object) => object.id);
    const selected = ids.filter((id) => set.has(id)).length;
    if (selected === 0) return "off";
    if (selected === ids.length) return "on";
    return "mixed";
  }

  function toggleBand(setter, index) {
    const ids = visibleObjects
      .filter((object) => object.band === index)
      .map((object) => object.id);
    setter((current) => {
      const next = new Set(current);
      const turnOff = ids.every((id) => next.has(id));
      ids.forEach((id) => {
        if (turnOff) next.delete(id);
        else next.add(id);
      });
      return next;
    });
  }

  function openAutomation(object) {
    const channels = automationByObject[object.id] ?? [];
    if (channels.length === 0) {
      setAutomationMenu(object.id);
      setAutomationQuery("");
      return;
    }
    setExpandedAutomation((current) => {
      const next = new Set(current);
      if (next.has(object.id)) next.delete(object.id);
      else next.add(object.id);
      return next;
    });
    setAutomationMenu(null);
    setAutomationQuery("");
  }

  function toggleGroup(objectId) {
    setExpandedGroups((current) => {
      const next = new Set(current);
      if (next.has(objectId)) next.delete(objectId);
      else next.add(objectId);
      return next;
    });
  }

  function openAutomationAdd(objectId) {
    setAutomationMenu(objectId);
    setAutomationQuery("");
  }

  function addAutomation(objectId, channel) {
    setAutomationByObject((current) => ({
      ...current,
      [objectId]: [
        ...(current[objectId] ?? []),
        {
          channel,
          keys: [62.5],
          easing: "Smooth",
        },
      ],
    }));
    setExpandedAutomation((current) => new Set(current).add(objectId));
    setAutomationMenu(null);
    setAutomationQuery("");
  }

  function toggleKey(objectId, channel, keyIndex) {
    const id = `${objectId}|${channel}|${keyIndex}`;
    setSelectedKeys((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function applyKeyOperation(operation) {
    const entries = [];
    OBJECTS.forEach((object) => {
      (automationByObject[object.id] ?? []).forEach((channel) => {
        channel.keys.forEach((localTime, keyIndex) => {
          const id = `${object.id}|${channel.channel}|${keyIndex}`;
          if (!selectedKeys.has(id)) return;
          entries.push({
            id,
            object,
            channel: channel.channel,
            globalTime:
              object.left +
              (objectOffsets[object.id] ?? 0) +
              object.width * (localTime / 100),
          });
        });
      });
    });
    const groups = new Map();
    entries.forEach((entry) => {
      const groupId =
        keyScope === "global"
          ? "global"
          : keyScope === "channel"
            ? entry.channel
            : entry.object.id;
      groups.set(groupId, [...(groups.get(groupId) ?? []), entry]);
    });
    const nextGlobalTimes = new Map();
    groups.forEach((group) => {
      const sorted = [...group].sort(
        (a, b) => a.globalTime - b.globalTime,
      );
      const start = sorted[0]?.globalTime ?? 0;
      const end = sorted[sorted.length - 1]?.globalTime ?? start;
      sorted.forEach((entry, index) => {
        let next = entry.globalTime;
        if (operation === "align-start") next = start;
        if (operation === "align-end") next = end;
        if (operation === "align-playhead") next = playheadLeft;
        if (operation === "stagger" && sorted.length > 1) {
          next = start + ((end - start) * index) / (sorted.length - 1);
        }
        if (operation === "reverse") {
          next = sorted[sorted.length - 1 - index].globalTime;
        }
        if (operation === "stretch-80") {
          next = start + (entry.globalTime - start) * 0.8;
        }
        if (operation === "stretch-120") {
          next = start + (entry.globalTime - start) * 1.2;
        }
        nextGlobalTimes.set(entry.id, next);
      });
    });
    setAutomationByObject((current) =>
      Object.fromEntries(
        OBJECTS.map((object) => [
          object.id,
          (current[object.id] ?? []).map((channel) => ({
            ...channel,
            keys: channel.keys.map((time, keyIndex) => {
              const id = `${object.id}|${channel.channel}|${keyIndex}`;
              const globalTime = nextGlobalTimes.get(id);
              if (globalTime === undefined) return time;
              return Math.max(
                0,
                Math.min(
                  100,
                  ((
                    globalTime -
                    object.left -
                    (objectOffsets[object.id] ?? 0)
                  ) /
                    object.width) *
                    100,
                ),
              );
            }),
          })),
        ]),
      ),
    );
  }

  function selectObject(event, objectId) {
    if (event.target.closest("button")) return;
    const object = OBJECTS.find((entry) => entry.id === objectId);
    setFocusedDepthObject(objectId);
    if (document.querySelector("#timeline")?.classList.contains("depth-open")) {
      setDepthScope(object?.parentId ?? "root");
    }
    setSelectedObjects((current) => {
      if (event.shiftKey || event.metaKey || event.ctrlKey) {
        const next = new Set(current);
        if (next.has(objectId)) next.delete(objectId);
        else next.add(objectId);
        return next;
      }
      return new Set([objectId]);
    });
  }

  function selectDepthObject(objectId) {
    const object = OBJECTS.find((entry) => entry.id === objectId);
    setSelectedObjects(new Set([objectId]));
    setFocusedDepthObject(objectId);
    setDepthScope(object?.parentId ?? "root");
    setDepthToolOpen(false);
  }

  function openDepthForObject(objectId) {
    const object = OBJECTS.find((entry) => entry.id === objectId);
    setSelectedObjects(new Set([objectId]));
    setFocusedDepthObject(objectId);
    setDepthScope(object?.parentId ?? "root");
    setDepthToolOpen(false);
    const toggle = document.querySelector("#depth-toggle");
    if (
      toggle &&
      !document.querySelector("#timeline")?.classList.contains("depth-open")
    ) {
      toggle.click();
    }
  }

  function applyDepthDistribution(previewDepths) {
    if (previewDepths.size === 0) return;
    setDepthByObject((current) => ({
      ...current,
      ...Object.fromEntries(previewDepths),
    }));
    setDepthToolOpen(false);
  }

  function applyLayerOperation(operation) {
    const selected = OBJECTS.filter((object) =>
      selectedObjects.has(object.id),
    ).map((object) => ({
      object,
      left: object.left + (objectOffsets[object.id] ?? 0),
    }));
    if (selected.length === 0) return;
    const sorted = [...selected].sort((a, b) => a.left - b.left);
    const start = sorted[0].left;
    const end = sorted[sorted.length - 1].left;
    const nextLeft = new Map();
    sorted.forEach((entry, index) => {
      let left = entry.left;
      if (operation === "align-start") left = start;
      if (operation === "align-end") left = end;
      if (operation === "stagger" && sorted.length > 1) {
        left = start + ((end - start) * index) / (sorted.length - 1);
      }
      if (operation === "reverse") {
        left = sorted[sorted.length - 1 - index].left;
      }
      if (operation === "shift-left") left -= 2;
      if (operation === "shift-right") left += 2;
      nextLeft.set(entry.object.id, left);
    });
    setObjectOffsets((current) => ({
      ...current,
      ...Object.fromEntries(
        selected.map(({ object }) => [
          object.id,
          Math.max(
            -object.left,
            Math.min(
              100 - object.left - object.width,
              (nextLeft.get(object.id) ?? object.left) - object.left,
            ),
          ),
        ]),
      ),
    }));
  }

  function clampPackStep(value) {
    return Math.max(30, Math.min(46, Math.round(value / 2) * 2));
  }

  function setPackHeight(index, value) {
    setPackHeights((current) =>
      current.map((height, itemIndex) =>
        itemIndex === index ? clampPackStep(value) : height,
      ),
    );
  }

  function beginPackDrag(event, index) {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    packDrag.current = {
      index,
      startY: event.clientY,
      startHeight: packHeights[index],
    };
    setDraggingBand(index);
  }

  function movePackDrag(event) {
    if (!packDrag.current) return;
    setPackHeight(
      packDrag.current.index,
      packDrag.current.startHeight +
        event.clientY -
        packDrag.current.startY,
    );
  }

  function finishPackDrag() {
    packDrag.current = null;
    setDraggingBand(null);
  }

  function cancelPackDrag() {
    if (packDrag.current) {
      setPackHeight(
        packDrag.current.index,
        packDrag.current.startHeight,
      );
    }
    finishPackDrag();
  }

  return (
    <section
      className={`timeline candidate-timeline${
        depthOpen ? " depth-open" : ""
      }${depthToolOpen ? " depth-tool-active" : ""}`}
      id="timeline"
      aria-label="譜面"
      data-react-surface="timeline"
    >
      {resizeHandle}
      <div
        className="timeline-head candidate-timeline-head"
        data-info="譜面 / Timeline|一枚の時間面へObject barをpackingして読む"
      >
        <button
          className="depth-toggle"
          id="depth-toggle"
          aria-expanded={depthOpen}
          aria-controls="depth-rail"
          aria-label={
            depthOpen ? "Depth Railを閉じる" : "Depth Railを開く"
          }
          onClick={() => setDepthOpen((current) => !current)}
        >
          ≋
        </button>
        <b>譜面 / Timeline</b>
        {GraphViewComponent ? (
          <div
            className="candidate-timeline-view-switch"
            role="group"
            aria-label="Timeline view"
          >
            <button
              type="button"
              aria-label="Open Timeline"
              aria-pressed={timelineView === "timeline"}
              onClick={() => setTimelineView("timeline")}
            >
              ▤
            </button>
            <button
              type="button"
              aria-label="Open Graph View"
              aria-pressed={timelineView === "graph"}
              onClick={() => setTimelineView("graph")}
            >
              ⌁
            </button>
          </div>
        ) : null}
      </div>

      <DepthRail
        depthByObject={depthByObject}
        depthOpen={depthOpen}
        depthScope={depthScope}
        depthToolOpen={depthToolOpen}
        farDepth={farDepth}
        focusedDepthObject={focusedDepthObject}
        nearDepth={nearDepth}
        reversedDepthOrder={reversedDepthOrder}
        selectedObjects={selectedObjects}
        onApplyDistribution={applyDepthDistribution}
        onCancelDistribution={() => setDepthToolOpen(false)}
        onFarDepthChange={setFarDepth}
        onNearDepthChange={setNearDepth}
        onOpenDistribution={() =>
          setDepthToolOpen((current) => !current)
        }
        onReverseDepthOrder={() =>
          setReversedDepthOrder((current) => !current)
        }
        onSelectObject={selectDepthObject}
        onSetScope={(scope) => {
          setDepthScope(scope);
          setDepthToolOpen(false);
        }}
      />

      {/* legacy bridgeの初期化だけを満たし、製品面へInboxを再表示しない。 */}
      <div hidden aria-hidden="true">
        <aside className="inbox" id="inbox">
          <span id="inbox-count">0</span>
          <div className="inbox-list" id="inbox-list" />
          <div className="inbox-tip">
            <button id="dismiss-tip" type="button" tabIndex="-1" />
          </div>
        </aside>
      </div>

      {timelineView === "timeline" ? <div
        className={`timeline-body candidate-timeline-body${
          keyToolsOpen ? " has-key-tools" : ""
        }`}
      >
        <div
          className="candidate-band-action-rail"
          aria-label="帯上のObjectを一括操作"
        >
          <div className="candidate-band-action-head" aria-hidden="true">
            <span>S</span><span>M</span>
          </div>
          <div className="candidate-band-action-rows">
            {packLayout.map((band, index) => (
              <div
                className="candidate-band-action-row"
                data-band={index + 1}
                key={`band-action-${index}`}
                style={{ height: `${band.height}px` }}
              >
                <BandStateButton
                  band={index}
                  kind="solo"
                  state={bandState(soloed, band.bandId)}
                  onClick={() => toggleBand(setSoloed, band.bandId)}
                />
                <BandStateButton
                  band={index}
                  kind="mute"
                  state={bandState(muted, band.bandId)}
                  onClick={() => toggleBand(setMuted, band.bandId)}
                />
                <button
                  type="button"
                  className={`candidate-pack-resize-zone${
                    draggingBand === band.bandId ? " is-dragging" : ""
                  }`}
                  role="slider"
                  aria-label={`packingレーン${index + 1}の高さを調整`}
                  aria-orientation="vertical"
                  aria-valuemin="30"
                  aria-valuemax="46"
                  aria-valuenow={band.baseHeight}
                  title={`この境界を上下dragして帯${index + 1}だけ変更`}
                  onBlur={() => setHoveredBand(null)}
                  onFocus={() => setHoveredBand(index)}
                  onKeyDown={(event) => {
                    const delta = {
                      ArrowUp: -2,
                      ArrowDown: 2,
                      PageUp: -6,
                      PageDown: 6,
                    }[event.key];
                    if (!delta) return;
                    event.preventDefault();
                    setPackHeight(band.bandId, band.baseHeight + delta);
                  }}
                  onPointerEnter={() => setHoveredBand(band.bandId)}
                  onPointerLeave={() => {
                    if (draggingBand !== band.bandId) setHoveredBand(null);
                  }}
                  onPointerCancel={cancelPackDrag}
                  onPointerDown={(event) =>
                    beginPackDrag(event, band.bandId)
                  }
                  onPointerMove={movePackDrag}
                  onPointerUp={finishPackDrag}
                />
              </div>
            ))}
          </div>
        </div>
        {keyToolsOpen ? (
          <aside className="candidate-key-tools" aria-label="Key Tools">
            <div className="candidate-key-mode">
              <button
                type="button"
                aria-pressed={keyToolsMode === "keys"}
                onClick={() => setKeyToolsMode("keys")}
              >
                KEYS
              </button>
              <button
                type="button"
                aria-pressed={keyToolsMode === "layers"}
                onClick={() => setKeyToolsMode("layers")}
              >
                LAYERS
              </button>
              <button
                type="button"
                aria-label="Key Toolsを閉じる"
                title="閉じる"
                onClick={() => setKeyToolsOpen(false)}
              >
                ×
              </button>
            </div>
            {keyToolsMode === "keys" ? (
              <>
                <div className="candidate-key-tools-head">
                  <b>◆ {selectedKeys.size}</b>
                  <div className="candidate-key-scope" aria-label="適用単位">
                    {[
                      ["object", "▤", "Object別"],
                      ["channel", "⋮", "Channel別"],
                      ["global", "◎", "全選択"],
                    ].map(([value, icon, label]) => (
                      <button
                        type="button"
                        aria-label={label}
                        aria-pressed={keyScope === value}
                        key={value}
                        title={label}
                        onClick={() => setKeyScope(value)}
                      >
                        {icon}
                      </button>
                    ))}
                  </div>
                </div>
                <div className="candidate-key-sections">
                  {[
                    ["align", "┆◆┆", "Align"],
                    ["stagger", "◆⋰◆", "Stagger"],
                    ["stretch", "←◆→", "Stretch"],
                  ].map(([section, icon, label]) => (
                    <button
                      type="button"
                      aria-label={label}
                      aria-expanded={keySection === section}
                      key={section}
                      title={label}
                      onClick={() =>
                        setKeySection((current) =>
                          current === section ? null : section,
                        )
                      }
                    >
                      {icon}
                    </button>
                  ))}
                </div>
                <div className="candidate-key-actions">
                  {keySection === "align" ? (
                    <>
                      <small>ALIGN</small>
                      <button type="button" aria-label="開始へ整列" title="開始へ整列" onClick={() => applyKeyOperation("align-start")}>│◆</button>
                      <button type="button" aria-label="Playheadへ整列" title="Playheadへ整列" onClick={() => applyKeyOperation("align-playhead")}>◆┆◆</button>
                      <button type="button" aria-label="終了へ整列" title="終了へ整列" onClick={() => applyKeyOperation("align-end")}>◆│</button>
                    </>
                  ) : null}
                  {keySection === "stagger" ? (
                    <>
                      <small>STAGGER</small>
                      <svg viewBox="0 0 96 38" aria-hidden="true">
                        <path d="M4 4 C28 4 64 34 92 34" />
                        <circle cx="4" cy="4" r="2" />
                        <circle cx="92" cy="34" r="2" />
                      </svg>
                      <button type="button" aria-label="等間隔に分布" title="等間隔に分布" onClick={() => applyKeyOperation("stagger")}>◆··◆</button>
                      <button type="button" aria-label="順序を反転" title="順序を反転" onClick={() => applyKeyOperation("reverse")}>⇄</button>
                    </>
                  ) : null}
                  {keySection === "stretch" ? (
                    <>
                      <small>STRETCH</small>
                      <button type="button" onClick={() => applyKeyOperation("stretch-80")}>80%</button>
                      <button type="button" onClick={() => applyKeyOperation("stretch-120")}>120%</button>
                    </>
                  ) : null}
                </div>
              </>
            ) : (
              <>
                <div className="candidate-key-tools-head">
                  <b>▤ {selectedObjects.size}</b>
                </div>
                <div className="candidate-key-sections">
                  {[
                    ["align", "┆▤┆", "Layer Align"],
                    ["stagger", "▤⋰▤", "Layer Stagger"],
                    ["shift", "←▤→", "Layer Shift"],
                  ].map(([section, icon, label]) => (
                    <button
                      type="button"
                      aria-label={label}
                      aria-expanded={layerSection === section}
                      key={section}
                      title={label}
                      onClick={() =>
                        setLayerSection((current) =>
                          current === section ? null : section,
                        )
                      }
                    >
                      {icon}
                    </button>
                  ))}
                </div>
                <div className="candidate-key-actions">
                  {layerSection === "align" ? (
                    <>
                      <small>ALIGN</small>
                      <button type="button" aria-label="Layerを開始へ整列" onClick={() => applyLayerOperation("align-start")}>│▤</button>
                      <button type="button" aria-label="Layerを終了へ整列" onClick={() => applyLayerOperation("align-end")}>▤│</button>
                    </>
                  ) : null}
                  {layerSection === "stagger" ? (
                    <>
                      <small>STAGGER</small>
                      <button type="button" aria-label="Layerを等間隔に分布" onClick={() => applyLayerOperation("stagger")}>▤··▤</button>
                      <button type="button" aria-label="Layer順序を反転" onClick={() => applyLayerOperation("reverse")}>⇄</button>
                    </>
                  ) : null}
                  {layerSection === "shift" ? (
                    <>
                      <small>SHIFT</small>
                      <button type="button" aria-label="Layerを左へ移動" onClick={() => applyLayerOperation("shift-left")}>≪</button>
                      <button type="button" aria-label="Layerを右へ移動" onClick={() => applyLayerOperation("shift-right")}>≫</button>
                    </>
                  ) : null}
                </div>
              </>
            )}
          </aside>
        ) : (
          <button
            type="button"
            className="candidate-key-tools-open"
            aria-label="Key Toolsを開く"
            onClick={() => setKeyToolsOpen(true)}
          >
            ◆
          </button>
        )}
        <div
          className="time-plane candidate-time-viewport"
          aria-label="Object barをpackingした一枚の時間面"
        >
          <div className="candidate-time-content">
            <div className="beat-ruler candidate-beat-ruler" aria-label="Beat ruler">
              <span className="candidate-axis-title">TIME / BEAT</span>
              {TICKS.map(([label, left, major]) => (
                <span
                  key={label}
                  className={major ? "major" : "minor"}
                  style={{ left: `${left}%` }}
                >
                  {label}
                </span>
              ))}
            </div>
            <div
              className="candidate-pack-plane"
              style={{ minHeight: `${packContentHeight + 8}px` }}
            >
              <div className="candidate-pack-guides" aria-hidden="true">
                {packLayout.map((band) => (
                  <i
                    key={`pack-guide-${band.bandId}`}
                    data-active={
                      hoveredBand === band.bandId ||
                      draggingBand === band.bandId
                        ? "true"
                        : "false"
                    }
                    style={{ top: `${band.bottom - 1}px` }}
                  />
                ))}
              </div>
              {OBJECTS.filter(
                (object) =>
                  object.isGroup && expandedGroups.has(object.id),
              ).map((group) => {
                const child = OBJECTS.find(
                  (object) => object.parentId === group.id,
                );
                const groupBand = packLayoutByBand.get(group.band);
                const childBand = packLayoutByBand.get(child?.band);
                if (!child || !groupBand || !childBand) return null;
                return (
                  <div key={`group-projection-${group.id}`}>
                    <i
                      className="candidate-group-lane-bg"
                      data-color-slot={group.colorSlot}
                      aria-hidden="true"
                      style={{
                        top: `${childBand.top}px`,
                        height: `${childBand.height}px`,
                      }}
                    />
                    <i
                      className="candidate-group-guide"
                      aria-hidden="true"
                      style={{
                        left: `${group.left + 1}%`,
                        top: `${
                          groupBand.top + groupBand.baseHeight - 5
                        }px`,
                        width: `${Math.max(2, child.left - group.left)}%`,
                        height: `${
                          childBand.top -
                          groupBand.top -
                          groupBand.baseHeight / 2
                        }px`,
                      }}
                    />
                  </div>
                );
              })}
              {visibleObjects.map((object) => {
                const isMuted = muted.has(object.id);
                const isSoloed = soloed.has(object.id);
                const parentSoloed =
                  object.parentId && soloed.has(object.parentId);
                const childSoloed =
                  object.isGroup &&
                  object.children.some((childId) =>
                    soloed.has(childId),
                  );
                const audible =
                  !hasSolo || isSoloed || parentSoloed || childSoloed;
                const band = packLayoutByBand.get(object.band);
                const automation = automationByObject[object.id] ?? [];
                return (
                  <div
                    className={`clip time-bar candidate-time-bar${
                      selectedObjects.has(object.id) ? " on selected" : ""
                    }${object.isGroup ? " is-group" : ""}${
                      object.parentId ? " is-group-child" : ""
                    }`}
                    key={object.id}
                    id={object.id === "pulse-rings" ? "vism-clip" : undefined}
                    data-object-id={object.id}
                    data-selected={
                      selectedObjects.has(object.id) ? "true" : "false"
                    }
                    data-muted={isMuted ? "true" : "false"}
                    data-audible={audible ? "true" : "false"}
                    data-color-slot={object.colorSlot}
                    data-own-color-slot={object.colorSlot}
                    data-kind={object.kind}
                    data-name={object.name}
                    data-path={`PROJECT / Main / ${object.name}`}
                    data-flow-state="ready"
                    data-group-expanded={
                      object.isGroup
                        ? expandedGroups.has(object.id)
                          ? "true"
                          : "false"
                        : undefined
                    }
                    data-parent-id={object.parentId}
                    style={{
                      left: `${
                        object.left + (objectOffsets[object.id] ?? 0)
                      }%`,
                      width: `${object.width}%`,
                      top: `${4 + band.top}px`,
                      height: `${band.baseHeight - 6}px`,
                    }}
                    onClick={(event) => selectObject(event, object.id)}
                    onDoubleClick={(event) => {
                      if (!object.isGroup || event.target.closest("button")) {
                        return;
                      }
                      toggleGroup(object.id);
                    }}
                  >
                    <span className="candidate-kind" aria-hidden="true">{object.kind}</span>
                    {object.isGroup ? (
                      <GroupFoldButton
                        object={object}
                        expanded={expandedGroups.has(object.id)}
                        onClick={() => toggleGroup(object.id)}
                      />
                    ) : null}
                    <ObjectStateControls
                      object={object}
                      muted={isMuted}
                      soloed={isSoloed}
                      onMute={() => toggle(setMuted, object.id)}
                      onSolo={() => toggle(setSoloed, object.id)}
                    />
                    <AutomationTrigger
                      object={object}
                      count={automation.length}
                      expanded={expandedAutomation.has(object.id)}
                      active={expandedAutomation.has(object.id)}
                      onClick={() => openAutomation(object)}
                    />
                    {object.content}
                    {object.depthCapable ? (
                      <>
                        <button
                          type="button"
                          className="candidate-depth-open"
                          aria-label={`${object.name}のDepth Railを開く`}
                          title="Depth RailでこのObjectを表示"
                          onClick={() => openDepthForObject(object.id)}
                        >
                          ≋
                        </button>
                        <span
                          className="candidate-depth-value"
                          aria-label={`${object.name}のDepth ${formatDepth(
                            depthByObject[object.id] ?? 0,
                          )}`}
                        >
                          z {formatDepth(depthByObject[object.id] ?? 0)}
                        </span>
                      </>
                    ) : null}
                  </div>
                );
              })}
              {visibleObjects.filter((object) =>
                expandedAutomation.has(object.id),
              ).map((object) => {
                const band = packLayoutByBand.get(object.band);
                const automation = automationByObject[object.id] ?? [];
                return (
                  <div
                    className="candidate-automation-stack"
                    data-object-id={object.id}
                    key={`automation-stack-${object.id}`}
                    style={{
                      left: `${
                        object.left + (objectOffsets[object.id] ?? 0)
                      }%`,
                      top: `${band.top + band.baseHeight}px`,
                      width: `${object.width}%`,
                    }}
                  >
                    {automation.map((channel) => (
                      <div
                        className="candidate-automation-row"
                        data-channel={channel.channel}
                        key={channel.channel}
                      >
                        <span>{channel.channel}</span>
                        {channel.keys.map((position, keyIndex) => {
                          const keyId =
                            `${object.id}|${channel.channel}|${keyIndex}`;
                          return (
                            <button
                              type="button"
                              className="candidate-automation-key"
                              aria-label={`${object.name} · ${channel.channel} · Key ${keyIndex + 1}`}
                              aria-pressed={selectedKeys.has(keyId)}
                              data-easing={channel.easing}
                              key={keyId}
                              style={{ left: `${position}%` }}
                              onClick={() => {
                                setFocusedAutomation({
                                  objectId: object.id,
                                  channel: channel.channel,
                                });
                                toggleKey(
                                  object.id,
                                  channel.channel,
                                  keyIndex,
                                );
                              }}
                            />
                          );
                        })}
                      </div>
                    ))}
                    <button
                      type="button"
                      className="candidate-automation-add-row"
                      aria-label={`${object.name}へAutomationを追加`}
                      title="Automationを追加"
                      onClick={() => openAutomationAdd(object.id)}
                    >
                      ＋
                    </button>
                  </div>
                );
              })}
              {automationMenu
                ? (() => {
                    const object = OBJECTS.find(
                      (entry) => entry.id === automationMenu,
                    );
                    const automation =
                      automationByObject[automationMenu] ?? [];
                    const existing = new Set(
                      automation.map((entry) => entry.channel),
                    );
                    const normalizedQuery =
                      automationQuery.trim().toLocaleLowerCase();
                    const matches = normalizedQuery
                      ? ALL_AUTOMATION_CHANNELS.filter(
                          (channel) =>
                            !existing.has(channel) &&
                            channel
                              .toLocaleLowerCase()
                              .includes(normalizedQuery),
                        )
                      : [];
                    const band = packLayoutByBand.get(object.band);
                    const menuTop = Math.min(
                      band.bottom + 3,
                      Math.max(2, packContentHeight - 122),
                    );
                    return (
                      <div
                        className="candidate-automation-menu"
                        role="dialog"
                        aria-label={`${object.name}のAutomation`}
                        style={{
                          left: `${Math.min(
                            object.left + (objectOffsets[object.id] ?? 0),
                            83,
                          )}%`,
                          top: `${menuTop}px`,
                        }}
                      >
                        <div className="candidate-automation-menu-head">
                          <b>{object.name}</b>
                          <button
                            type="button"
                            aria-label="Automationメニューを閉じる"
                            onClick={() => setAutomationMenu(null)}
                          >
                            ×
                          </button>
                        </div>
                        <input
                          type="search"
                          aria-label="Automation channelを検索"
                          autoFocus
                          placeholder="⌕"
                          value={automationQuery}
                          onChange={(event) =>
                            setAutomationQuery(event.target.value)
                          }
                        />
                        <div className="candidate-automation-search-results">
                          {normalizedQuery && matches.length === 0 ? (
                            <span>該当なし</span>
                          ) : null}
                          {matches.map((channel) => (
                            <button
                              type="button"
                              key={channel}
                              onClick={() =>
                                addAutomation(object.id, channel)
                              }
                            >
                              <span>{channel}</span><i>＋</i>
                            </button>
                          ))}
                        </div>
                      </div>
                    );
                  })()
                : null}
            </div>
            <div
              className="playhead candidate-playhead"
              id="playhead"
              style={{ left: `${playheadLeft}%` }}
            />
          </div>
        </div>
      </div> : GraphViewComponent ? (
        <GraphViewComponent docked />
      ) : null}

      {EasingGraphComponent ? (
        <EasingGraphComponent intervalContext={activeInterval} />
      ) : null}
      {legacyCurveShelf}
    </section>
  );
}
