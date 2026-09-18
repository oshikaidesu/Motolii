// Words on ropes. One hand: the title is keyed left and right. Each string is a Rope — a cubic whose belly is
// a spring on the GPU — so it lags, swings and settles. The words stay; the relation moves.
comp({ width: 1440, height: 1080, fps: 30, seconds: 5, background: "#F0ECE3" });
const INK = "#14141A", RED = "#E8442E", BLUE = "#2B4C8C";
const title = text("MOTLEY", { name: "Title", Position: [720, 240] }).fill(INK).font("Helvetica Neue").set("Scale", [1.4, 1.4]);
title.keys("Position", [[0, [720, 240], "Hold"], [0.5, [720, 240], "Bezier"], [1.4, [1040, 220]], [2.2, [1040, 220], "Bezier"], [3.2, [400, 260]], [3.9, [400, 260], "Bezier"], [4.8, [720, 240]]]);
const words = ["patchwork", "collage", "jet set", "flipnote", "aviutl", "one hand", "many things", "relations"];
words.forEach((w, k) => {
  const x = 180 + k * 155, y = 720 + (k % 2) * 110;
  const word = text(w, { name: `W${k}`, Position: [x, y] }).fill(k % 3 === 0 ? RED : k % 3 === 1 ? BLUE : INK).font("Helvetica Neue").set("Scale", [0.36, 0.36]);
  line({ name: `Rope ${k}` }).fill(INK).set("Connect From", title).set("Connect To", word)
    .set("Line Path", "Rope").set("Slack", 18 + k * 4).set("Stroke Width", 2);
});
text("one hand on the title · eight ropes: cubic bellies on springs, solved on the GPU", { Position: [720, 1010] }).fill(INK).font("Helvetica Neue").set("Scale", [0.2, 0.2]);
