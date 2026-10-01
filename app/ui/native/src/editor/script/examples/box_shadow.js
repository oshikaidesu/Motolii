// Shadows that belong to boxes (CSS box-shadow): cards on a paper table lift one after another —
// the shadow drops farther and softens as a card rises, and settles back as it lands.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#E9E4DA" });

const tints = ["#FFFFFF", "#F3B61F", "#FFFFFF", "#2E5EAA", "#FFFFFF", "#E4572E"];
const cards = tints.map((color, i) => {
  const label = text(`0${i + 1}`, { name: `Label ${i + 1}` }).fill(color === "#FFFFFF" ? "#161616" : "#FFFFFF").font("Helvetica Neue");
  label.set("Position", [0, 0]).set("Scale", [0.9, 0.9]);
  const card = group(label).name(`Card ${i + 1}`);
  card.set("Display", "Flex").set("Justify Content", "End").set("Padding", [36, 28]).set("Background", color).set("Border Radius", 28)
    .set("Shadow Color", "#0000004D").set("Shadow Spread", -6);
  const lift = 0.5 + i * 0.7;
  card.keys("Shadow Offset", [[0, [0, 6], "Bezier"], [lift, [0, 6], "Bezier"], [lift + 0.5, [0, 42], "Bezier"], [lift + 1.2, [0, 42], "Bezier"], [lift + 1.7, [0, 6]]]);
  card.keys("Shadow Blur", [[0, 10, "Bezier"], [lift, 10, "Bezier"], [lift + 0.5, 60, "Bezier"], [lift + 1.2, 60, "Bezier"], [lift + 1.7, 10]]);
  card.keys("Position", [[0, [0, 0], "Bezier"], [lift, [0, 0], "Bezier"], [lift + 0.5, [0, -26], "Bezier"], [lift + 1.2, [0, -26], "Bezier"], [lift + 1.7, [0, 0]]]);
  return card;
});
const table = group(...cards).name("Table");
table.set("Display", "Grid").set("Grid Columns", 3).set("Grid Rows", 2).set("Gap", 60).set("Padding", [120, 90])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1920).set("Height", 1080).set("Position", [0, 0]);
for (const card of cards) card.set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill");
for (const layer of [table, ...cards]) layer.projection("2D");
