// SANKOU! #6 を関係で書く: 線は 2 つの端を結ぶ(Connect)、描くのは Trim Paths の End(鍵 2 つ = 線の描き終わり)。
// 文字は線が引き終わる時刻に来る(time)。文字を抱く箱(Hug・Overflow Clip)がその時に広がり、移り方(Transition)が 0.4 s で開く。Opacity の鍵は 0。
comp({ width: 1920, height: 1080, fps: 30, seconds: 3, background: "#FFFFFF" });
const OUT = { kind: "Bezier", x1: 0, y1: 0, x2: 0.58, y2: 1 };
const BLUE = "#006FBF", INK = "#091E2D", D = 270;
const end = (name, x, y) => ellipse({ name }).fill("#FFFFFF00").set("Scale", [4 / D, 4 / D]).set("Position", [x, y]);
const a = end("Line start", -40, 760), b = end("Line end", 1960, 380);
const stroke = line({ name: "Line" }).fill(BLUE).set("Connect From", a).set("Connect To", b)
  .set("From Side", "Right").set("To Side", "Left").set("Line Path", "Curved").set("Stroke Width", 6);
stroke.effect("Trim Paths", { "Trim": "Simultaneously" }).key("End", 0.2, 0, OUT).key("End", 1.4, 100);
const copy = [["TOYOX", 190, INK], ["RECRUIT", 130, BLUE], ["Make the future with us.", 48, INK]]
  .map(([words, size, ink]) => text(words, { name: words }).fill(ink).font("Helvetica Neue").set("Size", size).set("Alignment", "Left"));
const box = group(...copy).name("Copy");
box.set("Display", "Flex").set("Flex Direction", "Column").set("Align Items", "Start").set("Gap", 40).set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug")
  .set("Overflow", "Clip").set("Transition Duration", 0.4).set("Position", [420, 230]);
copy.forEach((t) => t.set("Position", [0, 0]).time(1.6, 1.4));
