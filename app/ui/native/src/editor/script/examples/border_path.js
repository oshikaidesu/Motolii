// Along the edge of the box (CSS offset-path: border-box): on a breathing bento, small pills run around each card's
// rounded border and turn with it. The cards change size; the paths change with them.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#0e0f12" });

const colors = ["#534AB7", "#1D9E75", "#D85A30", "#2E5EAA", "#F3B61F", "#993556"];
const cards = colors.map((color, i) => {
  const pill = rectangle({ name: `Pill ${i + 1}` }).fill(i === 4 ? "#161616" : "#F2EDE4");
  const tail = rectangle({ name: `Tail ${i + 1}` }).fill(i === 4 ? "#161616" : "#F2EDE4");
  const card = group(pill, tail).name(`Card ${i + 1}`);
  card.set("Display", "Flex").set("Background", color).set("Border Radius", 36);
  for (const [layer, lag, size] of [[pill, 0, [0.22, 0.05]], [tail, 3.5, [0.08, 0.05]]]) {
    layer.set("Position Type", "Absolute").set("Scale", size).set("Offset Path", "Border Box");
    layer.key("Offset Distance", 0, i * 17 - lag, "Linear").key("Offset Distance", 6, i * 17 - lag + (i % 2 ? 150 : 200));
  }
  return card;
});
const board = group(...cards).name("Bento");
board.set("Display", "Grid").set("Grid Columns", 3).set("Grid Rows", 2).set("Gap", 28).set("Padding", [0, 0])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1680).set("Height", 860).set("Position", [120, 110]);
for (const card of cards) card.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill");
board.keys("Column 1", [[0, 1, "Bezier"], [2, 2.2, "Bezier"], [4, 0.6, "Bezier"], [6, 1]]);
board.keys("Row 1", [[0, 1, "Bezier"], [3, 0.5, "Bezier"], [6, 1]]);
for (const layer of [board, ...cards]) layer.projection("2D");
