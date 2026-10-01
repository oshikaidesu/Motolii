// SANKOU! #4 化合 — the same hover line, but the whole menu stands in 2.5D (Tilt Y) so the redrawn line runs into depth.
// Source: @keyframes textLine { 0% size 100% at right; 30% size 0 at right; 70% size 0 at left; 100% size 100% at left },
// .6s cubic-bezier(.455,.03,.515,.955). The length is Scale X; the origin (right / left) is the Anchor, switched once,
// with a Hold — and after that discrete switch the length settles continuously.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#FFFFFF" });
const E = { kind: "Bezier", x1: 0.455, y1: 0.03, x2: 0.515, y2: 0.955 };
const INK = "#091E2D", SIZE = 96, THICK = 4, D = 270, T = 0.6;
const link = (word, x, y, hover) => {
  const len = word.length * SIZE * 0.62;
  const t = text(word, { name: word }).fill(INK).font("Helvetica Neue").set("Size", SIZE).set("Position", [x + len / 2, y]);
  const line = rectangle({ name: `${word} line` }).fill(INK).set("Scale", [len / D, THICK / D]);
  // The line is at rest drawn from the left: anchor on its left end, position at the word's left.
  line.set("Anchor", [0, D / 2]).set("Position", [x, y + SIZE * 0.62]);
  const RIGHT = [[D, D / 2], [x + len, y + SIZE * 0.62]], LEFT = [[0, D / 2], [x, y + SIZE * 0.62]];
  line.keys("Anchor", [[0, LEFT[0], "Hold"], [hover, RIGHT[0], "Hold"], [hover + T * 0.3, LEFT[0], "Hold"]]);
  line.keys("Position", [[0, LEFT[1], "Hold"], [hover, RIGHT[1], "Hold"], [hover + T * 0.3, LEFT[1], "Hold"]]);
  line.keys("Scale", [[hover, [len / D, THICK / D], E], [hover + T * 0.3, [0, THICK / D], "Hold"], [hover + T * 0.7, [0, THICK / D], E], [hover + T, [len / D, THICK / D]]]);
  return [t, line];
};
const menu = nullLayer({ name: "Menu" }).set("Position", [960, 540]).set("Tilt Y", 42).set("Tilt X", 8);
const parts = [link("RECRUIT", -700, -200, 0.5), link("MESSAGE", -700, 0, 1.4), link("ENTRY", -700, 200, 2.3)].flat();
for (const layer of [menu, ...parts]) layer.parent(layer === menu ? null : menu).projection("3D");
