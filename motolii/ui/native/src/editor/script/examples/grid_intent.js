// Chance, then intent: blocks and type drift where a Repeater's dice put them. The grid is declared,
// its red lines draw on, and everything snaps to the fields at once — the same random layout now reads as a design.
// (Müller-Brockmann's modular grid; the snap is C4D's Quantize for layers.)
comp({ width: 1080, height: 1350, fps: 30, seconds: 6, background: "#F1EFEA" });

const RED = "#D9553A", INK = "#151515", STONE = "#D3CEC4";
const made = [];
const keep = (l) => { made.push(l.id); return l; };

const snapKeys = (layer, on, off) => layer.keys("Snap to Grid", [[0, 0, "Hold"], [on, 0, "Bezier"], [on + 0.7, 1, "Hold"], [off, 1, "Bezier"], [off + 0.6, 0]]);

// The grid: 4 columns, 8 rows, gutters — nothing in its flow, only free children that may snap to it.
const lines = keep(line({ name: "Grid lines" }).fill(RED).set("Stroke Width", 2));
const blocks = keep(rectangle({ name: "Blocks" }).fill(STONE));
const reds = keep(rectangle({ name: "Red blocks" }).fill(RED));
const dots = keep(ellipse({ name: "Dots" }).fill(RED));
const title = keep(text("Grid systems", { name: "Title" }).fill(INK).font("Helvetica Neue"));
const kana = keep(text("グリッドシステム", { name: "Kana" }).fill(INK).font("Hiragino Sans"));
const sub = keep(text("in graphic design", { name: "Sub" }).fill(INK).font("Helvetica Neue"));
const note = keep(text("A visual communication manual\nfor graphic designers,\ntypographers and\nthree dimensional designers", { name: "Note" }).fill(INK).font("Helvetica Neue"));
const page = keep(group(lines, blocks, reds, dots, title, kana, sub, note).name("Page"));
page.set("Display", "Grid").set("Grid Columns", 4).set("Grid Rows", 8).set("Gap", 16).set("Padding", [0, 0])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 960).set("Height", 1230).set("Position", [60, 60]);

// The grid's own lines, drawn on as the order arrives.
lines.set("Position Type", "Absolute").set("Connect From", page).set("Trace", "Grid");
lines.effect("Trim Paths", {}).key("End", 1.2, 0, "Bezier").key("End", 2.0, 100, "Hold").key("End", 4.6, 100, "Bezier").key("End", 5.4, 0);

// Chance: repeated blocks thrown across the page.
const scatter = (layer, count, seed, size) => {
  layer.set("Position Type", "Absolute").set("Position", [480, 615]).set("Scale", size);
  layer.effect("Repeater", { "Count": count, "Position X Each": 0, "Position Y Each": 0, "Position X Random": 400, "Position Y Random": 540, "Scale Random": 45, "Seed Random": seed });
  layer.set("Snap Size", "Fields");
  snapKeys(layer, 2.0, 4.4);
};
scatter(blocks, 4, 11, [0.7, 0.5]);
scatter(reds, 2, 29, [0.6, 0.8]);
dots.set("Position Type", "Absolute").set("Position", [480, 615]).set("Scale", [0.5, 0.5]);
dots.effect("Repeater", { "Count": 3, "Position X Each": 0, "Position Y Each": 0, "Position X Random": 400, "Position Y Random": 540, "Seed Random": 5 });
snapKeys(dots, 2.1, 4.4);

// Type thrown at slight angles; it straightens as it snaps.
const toss = (layer, at, scale, angle, on) => {
  layer.set("Position Type", "Absolute").set("Position", at).set("Scale", [scale, scale]);
  layer.keys("Rotation", [[0, angle, "Hold"], [on, angle, "Bezier"], [on + 0.7, 0, "Hold"], [4.4, 0, "Bezier"], [5.0, angle]]);
  snapKeys(layer, on, 4.4);
};
toss(title, [520, 430], 1.25, -6, 2.2);
toss(kana, [560, 800], 1.05, 4, 2.3);
toss(sub, [300, 600], 0.3, 9, 2.35);
toss(note, [700, 620], 0.3, -7, 2.4);

op("setAttrs", { layers: made, patch: { projection: "2D" } });
