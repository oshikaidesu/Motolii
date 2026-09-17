// Motolii script prelude. Every call is a window operation; every name is a name the window shows.
// Host: __op(json) -> json (throws on refusal), __layer(id) -> json, __effects() -> json, __comp() -> json, __assets() -> json.
"use strict";

const refuse = (what, instead) => () => { throw new Error(`${what} is not available in Motolii scripts. ${instead}`); };
Math.random = refuse("Math.random", "Use random(seed) — the same seed gives the same picture every time.");
globalThis.Date = refuse("Date", "Time is the composition's: use seconds you pass to key().");
globalThis.setTimeout = refuse("setTimeout", "A script builds the document once; animate with key().");
globalThis.setInterval = globalThis.setTimeout;
globalThis.requestAnimationFrame = refuse("requestAnimationFrame", "There is no draw loop: set keys with key(), the window plays them.");

const op = (name, args = {}) => JSON.parse(__op(JSON.stringify({ op: name, ...args })));
const compInfo = () => JSON.parse(__comp());
const frameOf = (seconds) => {
  if (typeof seconds !== "number" || !Number.isFinite(seconds)) throw new Error(`Time must be seconds (a number), got ${JSON.stringify(seconds)}`);
  const c = compInfo();
  return Math.max(0, Math.round(seconds * c.fps));
};
const near = (wanted, names) => {
  const lower = wanted.toLowerCase();
  const hits = names.filter((n) => n.toLowerCase().includes(lower) || lower.includes(n.toLowerCase()));
  return (hits.length ? hits : names).join(", ");
};
const colorOf = (value) => {
  if (Array.isArray(value)) return value.length === 3 ? [...value, 1] : value;
  if (typeof value === "string" && /^#?[0-9a-f]{6}([0-9a-f]{2})?$/i.test(value)) {
    const hex = value.replace("#", "");
    const channel = (i) => parseInt(hex.slice(i, i + 2), 16) / 255;
    return [channel(0), channel(2), channel(4), hex.length === 8 ? channel(6) : 1];
  }
  throw new Error(`A color is "#rrggbb", "#rrggbbaa" or [r, g, b, a] in 0..1, got ${JSON.stringify(value)}`);
};
/** What the window stores for a written value: a choice by its name, a layer by the layer, a color by hex. */
const valueFor = (row, value) => {
  if (row.kind === "color" || row.subtype === "color") return colorOf(value);
  if (row.choices && typeof value === "string") {
    const index = row.choices.indexOf(value);
    if (index < 0) throw new Error(`${row.label} is one of ${row.choices.join(", ")}, got ${JSON.stringify(value)}`);
    return index;
  }
  if (value instanceof Layer) return value.id;
  return value;
};
/** An ease is the window's kind ("Bezier", "Hold", …) or a GSAP ease string ("power2.out", "back.out(1.7)", "elastic.out(1, 0.3)", "steps(5)", "none"). */
const easeShape = (ease) => (ease === undefined ? { kind: "Linear" } : typeof ease === "string" ? { kind: ease } : ease);

/** One seeded random stream: random(seed)() -> 0..1. The same seed always gives the same numbers. */
globalThis.random = (seed = 0) => {
  let s = (Math.imul(Number(seed) | 0, 2654435761) ^ 0x9e3779b9) >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let t = s;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
};

class Effect {
  constructor(layer, id) { this.layer = layer; this.id = id; }
  row() {
    const found = this.layer.json().effects.find((e) => e.id === this.id);
    if (!found) throw new Error(`Effect ${this.id} is no longer on ${this.layer.label()}`);
    return found;
  }
  /** The window's names: a grid row shows its columns ("Position Each", "Position Random"). */
  rows() {
    const e = this.row();
    const byId = new Map(e.params.map((p) => [p.id, p]));
    const gridded = new Set();
    const grid = [];
    for (const r of e.layout?.rows ?? []) {
      for (const [column, id] of [["Each", r.each], ["Random", r.random]]) {
        if (!id || !byId.has(id)) continue;
        gridded.add(id);
        grid.push({ ...byId.get(id), label: `${r.label} ${column}`, axis: r.axis ?? undefined });
      }
    }
    return [...e.params.filter((p) => !gridded.has(p.id)), ...grid];
  }
  /** Where the effect lands on a group: each child (default) or the whole group as one picture. */
  whole(on = true) { op("scopeEffect", { layer: this.layer.id, id: this.id, whole: on }); return this; }
  property(name) {
    const rows = this.rows();
    const row = rows.find((p) => p.label === name);
    if (!row) throw new Error(`${this.row().name} has no ${JSON.stringify(name)}. Names here: ${near(name, rows.map((p) => p.label))}`);
    return row;
  }
  set(name, value) { this.layer.write(this.property(name), value, undefined); return this; }
  key(name, seconds, value, ease) { this.layer.write(this.property(name), value, { seconds, ease }); return this; }
  names() { return this.rows().map((p) => p.label); }
  enabled(on) { op("enableEffect", { layer: this.layer.id, id: this.id, enabled: on }); return this; }
}

class Layer {
  constructor(id) { this.id = id; }
  json() { return JSON.parse(__layer(this.id)); }
  label() { return `${this.json().kind} layer ${JSON.stringify(this.json().name)}`; }
  property(name) {
    const rows = this.json().properties;
    const row = rows.find((p) => p.label === name);
    if (!row) throw new Error(`${this.label()} has no ${JSON.stringify(name)}. Names here: ${near(name, rows.map((p) => p.label))}`);
    return row;
  }
  write(row, value, keyed) {
    let v = valueFor(row, value);
    if (row.axis !== undefined) {
      // One well of a pair (Position X of a Repeater row): the other half keeps its value.
      const pair = [...row.value];
      pair[row.axis] = v;
      v = pair;
    }
    const frame = keyed ? frameOf(keyed.seconds) : 0;
    op("animate", keyed ? { enabled: true, shape: easeShape(keyed.ease) } : { enabled: false });
    op("seek", { frame });
    try {
      op("setProperty", { layer: this.id, property: row.id, value: v });
    } finally {
      op("animate", { enabled: false });
    }
  }
  /** A still value: no keys. */
  set(name, value) { this.write(this.property(name), value, undefined); return this; }
  /** A key at `seconds`. `ease` is how the value travels from this key to the next one. */
  key(name, seconds, value, ease) { this.write(this.property(name), value, { seconds, ease }); return this; }
  /** Several keys: [[seconds, value, ease?], ...]. */
  keys(name, list) { for (const [seconds, value, ease] of list) this.key(name, seconds, value, ease); return this; }
  names() { return this.json().properties.map((p) => p.label); }
  name(text) { op("setAttrs", { layers: [this.id], patch: { name: text } }); return this; }
  parent(layer) { op("setAttrs", { layers: [this.id], patch: { parent: layer ? layer.id : null } }); return this; }
  blend(mode) { op("setAttrs", { layers: [this.id], patch: { blendMode: mode } }); return this; }
  /** Show this layer only where the layer just below it is (a clipping mask). */
  clip(on = true) { op("setAttrs", { layers: [this.id], patch: { clipToBelow: on } }); return this; }
  projection(kind) { op("setAttrs", { layers: [this.id], patch: { projection: kind } }); return this; }
  /** When the layer is on screen, in seconds. */
  time(start, duration) {
    const j = this.json();
    op("setTiming", { layer: this.id, start: frameOf(start), duration: Math.max(1, frameOf(duration)), sourceIn: j.sourceIn });
    return this;
  }
  text(content) { op("seek", { frame: 0 }); op("setText", { layer: this.id, content: String(content) }); return this; }
  /** The fill of a shape or the color of a text: "#rrggbb". */
  fill(color) { op("select", { id: this.id }); op("applyPalette", { rgba: colorOf(color) }); return this; }
  font(family) { op("setFont", { layer: this.id, family }); return this; }
  effect(name, values = {}) {
    const known = JSON.parse(__effects());
    const found = known.find((e) => e.name === name);
    if (!found) throw new Error(`No effect named ${JSON.stringify(name)}. Effects: ${near(name, known.map((e) => e.name))}`);
    const before = new Set(this.json().effects.map((e) => e.id));
    op("select", { id: this.id });
    op("applyEffect", { pluginId: found.pluginId });
    const added = this.json().effects.find((e) => !before.has(e.id));
    const effect = new Effect(this, added.id);
    for (const [param, value] of Object.entries(values)) effect.set(param, value);
    return effect;
  }
}

const create = (kind, extra = {}) => (options = {}) => {
  op("seek", { frame: 0 });
  const reply = op("create", { kind, ...extra });
  const layer = new Layer(reply.selected);
  if (options.name !== undefined) layer.name(options.name);
  for (const [name, value] of Object.entries(options)) if (name !== "name") layer.set(name, value);
  return layer;
};

/** A picture, video or 3D file placed as a layer. The path is absolute. */
globalThis.media = (path, options = {}) => {
  op("import", { paths: [path] });
  const name = path.split("/").pop();
  const asset = JSON.parse(__assets()).reverse().find((a) => a.path === path || (a.path ?? "").endsWith(`/${name}`));
  if (!asset) throw new Error(`${path} was not admitted as a material`);
  op("seek", { frame: 0 });
  const layer = new Layer(op("placeAsset", { id: asset.id, start: 0 }).selected);
  if (options.name !== undefined) layer.name(options.name);
  for (const [key, value] of Object.entries(options)) if (key !== "name") layer.set(key, value);
  return layer;
};

/** The composition: { width, height, fps, seconds, background }. */
globalThis.comp = ({ width, height, fps, seconds, background } = {}) => {
  const c = compInfo();
  op("composition", {
    width: width ?? c.width,
    height: height ?? c.height,
    fpsNum: fps ?? c.fps,
    fpsDen: 1,
    durationFrames: Math.round((seconds ?? c.seconds) * (fps ?? c.fps)),
    ...(background === undefined ? {} : { background: colorOf(background) }),
  });
  return compInfo();
};
globalThis.text = (content, options = {}) => create("text")(options).text(content);
globalThis.rectangle = create("rectangle");
globalThis.roundedRectangle = create("roundedRectangle");
globalThis.ellipse = create("ellipse");
globalThis.star = create("star");
globalThis.polygon = create("polygon");
globalThis.line = create("line");
globalThis.nullLayer = create("null");
globalThis.particles = create("particles");
globalThis.camera = create("camera");
/** Group layers into one; returns the group. */
globalThis.group = (...layers) => { op("select", { ids: layers.map((l) => l.id) }); return new Layer(op("group").selected); };
/** The window's effect names. */
globalThis.effects = () => JSON.parse(__effects()).map((e) => e.name);
