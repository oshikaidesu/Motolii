// kkmfd「言葉」型の骨格を今の口で: 芯 1 つ(黒い輪 + 青い星)、文字の格子は同じ平面(3D、z = 0)、カメラを傾けて見下ろす、芯の近くに注記、縁に HUD(2D = 画面に固定)。
// 手 = 群の Flex、Zero Gravity、星の鍵、カメラの Orbit、HUD。CSS の grid + perspective + rotateX と同じ作り。
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#EEEEF0" });
const INK = "#111111", BLUE = "#1A1AFF", GREY = "#777777";
// 1. 平面に敷いた文字の格子(群 + Flex Wrap)
const words = [];
for (let k = 0; k < 7 * 4; k++) words.push(text("言葉", { name: `W${k}` }).fill(INK).font("Hiragino Mincho ProN").set("Size", 150));
const sheet = group(...words).name("Sheet");
sheet.set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", 120)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 2600).set("Height", 1500).set("Position", [-340, -300]);
words.forEach((t) => t.set("Position", [0, 0]));
words.forEach((t) => t.effect("Zero Gravity", { "Drift": 6, "Tumble": 4, "Breath": 0.02, "Slowness": 9 }));
// 2. 芯: 黒い輪(道)と、輪の上を走る青い星
const ring = ellipse({ name: "Ring", Position: [880, 620] }).fill(INK).set("Scale", [2.6, 1.3]);
const hole = ellipse({ name: "Hole", Position: [880, 620] }).fill("#EEEEF0").set("Scale", [1.6, 0.7]);
const spark = star({ name: "Spark", Position: [560, 380] }).fill(BLUE).set("Scale", [0.5, 0.5]);
spark.key("Position", 0, [560, 380], "power2.inOut").key("Position", 3, [1240, 560]).key("Position", 6, [560, 380]);
spark.key("Rotation", 0, 0).key("Rotation", 6, 720);
// 3. 注記: 芯の穴の中に小さな字と線(同じ平面の上)
const note = text("station", { name: "Label", Position: [800, 640] }).fill(INK).font("Helvetica Neue").set("Size", 34).set("Rotation", -70);
const note2 = text("流失的 / 22.7 s", { name: "Label2", Position: [960, 700] }).fill(GREY).font("Helvetica Neue").set("Size", 18);
const rule = line({ name: "Rule", Position: [960, 660] }).fill(INK).set("Scale", [1.2, 0.02]).set("Rotation", -18);
// 世界(z = 0 の平面)を見下ろすカメラ。2D の HUD はカメラに従わない。
for (const l of [...words, sheet, ring, hole, spark, note, note2, rule]) l.projection("3D");
camera({ name: "Camera" }).set("Orbit", [-50, 12]);
// 4. HUD: 画面の縁に固定(2D)
const hud = [];
hud.push(text("Goodbye Twilight Train", { Position: [340, 54] }).fill(GREY).font("Helvetica Neue").set("Size", 26));
for (const [x, y] of [[60, 34], [1860, 34], [60, 1046], [1860, 1046]]) hud.push(rectangle({ Position: [x, y] }).fill(GREY).set("Scale", [0.2, 0.03]));
for (const [x, y] of [[150, 34], [1770, 34], [150, 1046], [1770, 1046]]) for (let i = 0; i < 9; i++) hud.push(ellipse({ Position: [x + (i % 3) * 12 - 12, y + Math.floor(i / 3) * 12 - 12] }).fill(GREY).set("Scale", [0.012, 0.012]));
hud.push(line({ Position: [470, 30] }).fill(INK).set("Scale", [1.7, 0.01]));
hud.push(line({ Position: [1450, 30] }).fill(INK).set("Scale", [1.7, 0.01]));
hud.forEach((l) => l.projection("2D"));
