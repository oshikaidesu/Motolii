// The camera frames boxes: a board of cards laid out by a grid. The camera's Target steps from card to card (Hold keys),
// Framing Size keeps each one at the same share of the screen, and the camera's Transition carries it between them.
comp({ width: 1920, height: 1080, fps: 30, seconds: 9, background: "#0e0f12" });

const cells = [
  ["01", "Layout", "#E4572E", [2, 1]], ["02", "Depth", "#F3B61F", [1, 1]], ["03", "Margin", "#2E5EAA", [1, 2]],
  ["04", "Transition", "#7FA37A", [1, 1]], ["05", "Flow", "#F2EDE4", [2, 1]], ["06", "Anchor", "#534AB7", [1, 1]],
  ["07", "Connect", "#D9553A", [1, 1]], ["08", "Stencil", "#1D9E75", [2, 1]],
];
const cards = cells.map(([n, word, color, span]) => {
  const ink = color === "#F2EDE4" || color === "#F3B61F" ? "#161616" : "#ffffff";
  const big = text(n, { name: `${word} number` }).fill(ink).font("Helvetica Neue");
  big.set("Position", [0, 0]).set("Scale", [1.4, 1.4]);
  const small = text(word, { name: `${word} word` }).fill(ink).font("Helvetica Neue");
  small.set("Position", [0, 0]).set("Scale", [0.4, 0.4]);
  const card = group(big, small).name(word);
  card.set("Display", "Flex").set("Flex Direction", "Column").set("Justify Content", "Space Between").set("Padding", [40, 32])
    .set("Background", color).set("Border Radius", 24);
  return { card, span };
});
const board = group(...cards.map((c) => c.card)).name("Board");
board.set("Display", "Grid").set("Grid Columns", 4).set("Grid Rows", 3).set("Gap", 24).set("Padding", [24, 24])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 2600).set("Height", 1500).set("Position", [-340, -210]);
for (const { card, span } of cards) {
  card.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill").set("Column Span", span[0]).set("Row Span", span[1]);
}

const cam = camera({ name: "Camera" });
cam.set("Framing Size", 0.72).set("Transition Duration", 0.9).set("Transition Easing", "Ease In Out");
const order = [0, 2, 5, 7, 4, 1];
cam.keys("Target", order.map((k, i) => [i * 1.2, cards[k].card, "Hold"]).concat([[7.2, board, "Hold"]]));
