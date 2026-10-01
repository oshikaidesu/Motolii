// Breathing Swiss grid: a paper poster whose columns and rows swell in waves, content poured in by the grid.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#F2EDE4" });

const INK = "#161616", PAPER = "#F2EDE4", RED = "#E4572E", BLUE = "#2E5EAA", YELLOW = "#F3B61F", GREEN = "#7FA37A";
const all = [];

const letter = (ch, bg, ink, radius = 0, scale = 1.9) => {
  const t = text(ch, { name: `Letter ${ch}` }).fill(ink).font("Helvetica Neue");
  t.set("Position", [0, 0]).set("Scale", [scale, scale]);
  const box = group(t).name(`Cell ${ch}`);
  box.set("Display", "Flex").set("Justify Content", "Center").set("Align Items", "Center")
    .set("Background", bg).set("Border Radius", radius).set("Overflow", "Clip");
  all.push(t);
  return box;
};
const dots = [];
const dot = (color) => { const d = ellipse({ name: "Dot" }).fill(color); dots.push(d); return d; };
const block = (color) => rectangle({ name: "Block" }).fill(color);

const items = [];
items.push(letter("M", RED, PAPER, 0, 3.1));
const singles = [
  () => dot(BLUE), () => letter("O", INK, PAPER, 400), () => block(YELLOW), () => letter("T", PAPER, INK),
  () => dot(RED), () => block(GREEN), () => letter("O", BLUE, PAPER, 400), () => block(INK),
  () => dot(YELLOW), () => letter("L", YELLOW, INK), () => block(RED), () => letter("I", GREEN, PAPER),
];
for (const make of singles) items.push(make());
const wide = letter("LAYOUT IS MOTION", INK, PAPER, 0, 0.78);
wide.set("Justify Content", "Start").set("Padding", [40, 0]);
items.push(wide);
const more = [
  () => letter("I", RED, PAPER, 400), () => dot(INK), () => block(BLUE), () => dot(GREEN),
  () => letter("+", PAPER, INK), () => letter("2", YELLOW, INK), () => dot(RED), () => letter("0", INK, PAPER, 400),
  () => block(GREEN), () => letter("2", BLUE, PAPER), () => dot(YELLOW), () => letter("6", RED, PAPER, 400),
];
for (const make of more) items.push(make());

const poster = group(...items).name("Poster");
poster.set("Display", "Grid").set("Grid Columns", 8).set("Grid Rows", 4).set("Gap", 12)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1760).set("Height", 920)
  .set("Position", [80, 80]);

items.forEach((item, i) => {
  item.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill");
  const at = 0.15 + i * 0.035;
  item.key("Position", at, [0, 90], "Bezier").key("Position", at + 0.55, [0, 0]);
  item.key("Opacity", at, 0, "Bezier").key("Opacity", at + 0.35, 1);
});
for (const d of dots) d.set("Object Fit", "Contain").set("Transition Duration", 0.3);
wide.set("Padding", [40, 24]);
const wideText = all.find((l) => l.json().name === "Letter LAYOUT IS MOTION");
wideText.set("Horizontal Sizing", "Fill").set("Alignment", "Left").set("Transition Duration", 0.5).set("Transition Easing", "Ease In Out");
items[0].set("Column Span", 2).set("Row Span", 2);
wide.set("Column Span", 4);

// The grid breathes: a wave runs across the columns, then one runs down the rows, then back.
for (let c = 1; c <= 8; c++) {
  const t0 = 1.3 + (c - 1) * 0.13;
  const t1 = 3.9 + (8 - c) * 0.13;
  poster.keys(`Column ${c}`, [[t0, 1, "Bezier"], [t0 + 0.45, 2.6, "Bezier"], [t0 + 1.0, 1, "Bezier"],
    [t1, 1, "Bezier"], [t1 + 0.45, 2.2, "Bezier"], [t1 + 1.0, 1]]);
}
for (let r = 1; r <= 4; r++) {
  const t0 = 2.7 + (r - 1) * 0.16;
  poster.keys(`Row ${r}`, [[t0, 1, "Bezier"], [t0 + 0.4, 2.4, "Bezier"], [t0 + 0.95, 1]]);
}

for (const layer of [poster, ...items, ...all]) layer.projection("2D");
