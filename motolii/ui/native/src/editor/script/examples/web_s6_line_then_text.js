// SANKOU! #6 — recruit.toyox.co.jp: a line draws itself across the section, then the words appear.
// Source: @keyframes line-stroke { to { stroke-dashoffset: 0 } } (dasharray = the full length), .p-top-fv__line 1.2s .2s ease-out
// (= cubic-bezier(0,0,.58,1)); the copy follows: opacity .4s 1.6s ease-in-out (= cubic-bezier(.42,0,.58,1)).
// The stroke is a Trim Paths End 0 → 100 on a connected line; no mask.
comp({ width: 1920, height: 1080, fps: 30, seconds: 3, background: "#FFFFFF" });
const OUT = { kind: "Bezier", x1: 0, y1: 0, x2: 0.58, y2: 1 }, IO = { kind: "Bezier", x1: 0.42, y1: 0, x2: 0.58, y2: 1 };
const BLUE = "#006FBF", INK = "#091E2D", D = 270;
const end = (name, x, y) => ellipse({ name }).fill("#FFFFFF00").set("Scale", [4 / D, 4 / D]).set("Position", [x, y]);
const a = end("Line start", -40, 760), b = end("Line end", 1960, 380);
const stroke = line({ name: "Line" }).fill(BLUE).set("Connect From", a).set("Connect To", b)
  .set("From Side", "Right").set("To Side", "Left").set("Line Path", "Curved").set("Stroke Width", 6);
stroke.effect("Trim Paths", { "Trim": "Simultaneously" }).key("End", 0.2, 0, OUT).key("End", 1.4, 100);
const copy = [["TOYOX", 190, 0, INK], ["RECRUIT", 130, 1, BLUE], ["Make the future with us.", 48, 2, INK]];
copy.forEach(([words, size, row, ink]) => text(words, { name: words }).fill(ink).font("Helvetica Neue").set("Size", size)
  .set("Position", [700, 330 + row * 190]).set("Opacity", 0).key("Opacity", 1.6, 0, IO).key("Opacity", 2.0, 1));
