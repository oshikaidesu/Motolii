const segmenter =
  typeof Intl !== "undefined" && Intl.Segmenter
    ? new Intl.Segmenter("ja", { granularity: "grapheme" })
    : null;

export function segmentText(text) {
  if (!segmenter) {
    return Array.from(text);
  }
  return Array.from(segmenter.segment(text), (entry) => entry.segment);
}

function cloneOverride(override) {
  return override ? structuredClone(override) : null;
}

function cloneUnit(unit) {
  return {
    id: unit.id,
    grapheme: unit.grapheme,
    override: cloneOverride(unit.override),
  };
}

export function createState(text) {
  const graphemes = segmentText(text);
  return {
    units: graphemes.map((grapheme, index) => ({
      id: `g${index + 1}`,
      grapheme,
      override: null,
    })),
    nextId: graphemes.length + 1,
    needsReview: [],
    lastEdit: "Initial text",
  };
}

export function withOverride(state, id, override) {
  return {
    ...state,
    units: state.units.map((unit) =>
      unit.id === id ? { ...cloneUnit(unit), override: structuredClone(override) } : cloneUnit(unit),
    ),
    needsReview: [...state.needsReview],
  };
}

function allocateUnits(state, text) {
  let nextId = state.nextId;
  const units = segmentText(text).map((grapheme) => {
    const unit = { id: `g${nextId}`, grapheme, override: null };
    nextId += 1;
    return unit;
  });
  return { units, nextId };
}

export function insertText(state, index, text) {
  const insertion = allocateUnits(state, text);
  return {
    units: [
      ...state.units.slice(0, index).map(cloneUnit),
      ...insertion.units,
      ...state.units.slice(index).map(cloneUnit),
    ],
    nextId: insertion.nextId,
    needsReview: [],
    lastEdit: `Insert “${text}” at ${index}`,
  };
}

export function deleteRange(state, index, count) {
  const removed = state.units.slice(index, index + count);
  return {
    units: [
      ...state.units.slice(0, index).map(cloneUnit),
      ...state.units.slice(index + count).map(cloneUnit),
    ],
    nextId: state.nextId,
    needsReview: removed
      .filter((unit) => unit.override)
      .map((unit) => ({
        oldId: unit.id,
        grapheme: unit.grapheme,
        reason: "Deleted unit had manual overrides",
      })),
    lastEdit: `Delete ${count} unit(s) at ${index}`,
  };
}

export function replaceRange(state, index, count, text) {
  const removed = state.units.slice(index, index + count);
  const insertion = allocateUnits(state, text);
  return {
    units: [
      ...state.units.slice(0, index).map(cloneUnit),
      ...insertion.units,
      ...state.units.slice(index + count).map(cloneUnit),
    ],
    nextId: insertion.nextId,
    needsReview: removed
      .filter((unit) => unit.override)
      .map((unit) => ({
        oldId: unit.id,
        grapheme: unit.grapheme,
        reason: "Replacement never inherits manual overrides automatically",
      })),
    lastEdit: `Replace ${count} unit(s) at ${index} with “${text}”`,
  };
}

export function reconcileWholeText(state, nextText) {
  const next = segmentText(nextText);
  const previous = state.units;
  let prefix = 0;
  while (
    prefix < previous.length &&
    prefix < next.length &&
    previous[prefix].grapheme === next[prefix]
  ) {
    prefix += 1;
  }

  let suffix = 0;
  while (
    suffix < previous.length - prefix &&
    suffix < next.length - prefix &&
    previous[previous.length - 1 - suffix].grapheme === next[next.length - 1 - suffix]
  ) {
    suffix += 1;
  }

  const removedMiddle = previous.slice(prefix, previous.length - suffix);
  const nextMiddle = next.slice(prefix, next.length - suffix).join("");
  const insertion = allocateUnits(state, nextMiddle);

  return {
    units: [
      ...previous.slice(0, prefix).map(cloneUnit),
      ...insertion.units,
      ...previous.slice(previous.length - suffix).map(cloneUnit),
    ],
    nextId: insertion.nextId,
    needsReview: removedMiddle
      .filter((unit) => unit.override)
      .map((unit) => ({
        oldId: unit.id,
        grapheme: unit.grapheme,
        reason: "Whole-text edit could not preserve this overridden unit safely",
      })),
    lastEdit: `Conservative whole-text reconcile → “${nextText}”`,
  };
}

export function duplicateState(state) {
  let nextId = state.nextId;
  const idMap = new Map();
  const units = state.units.map((unit) => {
    const id = `g${nextId}`;
    nextId += 1;
    idMap.set(unit.id, id);
    return {
      id,
      grapheme: unit.grapheme,
      override: cloneOverride(unit.override),
    };
  });
  return {
    state: {
      units,
      nextId,
      needsReview: [],
      lastEdit: "Independent duplicate with reminted IDs",
    },
    idMap,
  };
}

function hashString(value) {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}

function random01(seed) {
  let value = seed >>> 0;
  value += 0x6d2b79f5;
  value = Math.imul(value ^ (value >>> 15), value | 1);
  value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
  return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
}

export function randomInPose(id, seed, scatter = 0.2) {
  const base = hashString(`${seed}:${id}`);
  const angle = random01(base) * Math.PI * 2;
  const distance = 115 + random01(base + 1) * 85;
  const sideScatter = (random01(base + 2) * 2 - 1) * scatter * 180;
  return {
    x: Math.cos(angle) * distance + sideScatter,
    y: Math.sin(angle) * distance,
    rotation: (random01(base + 3) * 2 - 1) * 18,
    scale: 0.7 + random01(base + 4) * 0.25,
  };
}

export function evaluateState(
  state,
  { seed = 1842, interval = 2, duration = 8, scatter = 0.2, spacing = 76 } = {},
) {
  return state.units.map((unit, index) => {
    const override = unit.override ?? {};
    const timing = override.timing ?? { mode: "auto", value: 0 };
    const autoStart = index * interval;
    const start =
      timing.mode === "pinned"
        ? timing.value
        : timing.mode === "offset"
          ? autoStart + timing.value
          : autoStart;

    return {
      ...cloneUnit(unit),
      index,
      autoStart,
      start,
      duration: override.duration ?? duration,
      final: {
        x: index * spacing + (override.offsetX ?? 0),
        y: override.offsetY ?? 0,
        scale: override.visualScale ?? 1,
        rotation: override.rotation ?? 0,
      },
      inPose: randomInPose(unit.id, seed, scatter),
      timingMode: timing.mode,
    };
  });
}

export function stateText(state) {
  return state.units.map((unit) => unit.grapheme).join("");
}

export function validateState(state) {
  const ids = state.units.map((unit) => unit.id);
  return {
    uniqueIds: new Set(ids).size === ids.length,
    text: stateText(state),
    overriddenIds: state.units.filter((unit) => unit.override).map((unit) => unit.id),
  };
}
