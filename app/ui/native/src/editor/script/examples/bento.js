// A bento grid: 4 columns x 3 rows, the hero spans 2 x 2, cells rise in one after another.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#0e0f12" });

const cells = [
  { name: "Hero", color: "#534AB7", label: "New", span: [2, 2] },
  { name: "Stat", color: "#1D9E75", label: "4.8x" },
  { name: "Logo", color: "#D85A30", label: "M" },
  { name: "Title", color: "#26215C", label: "Motion, laid out", span: [2, 1] },
  { name: "Photo A", color: "#378ADD", label: "A" },
  { name: "Photo B", color: "#BA7517", label: "B" },
  { name: "Quote", color: "#993556", label: "“Yes”" },
  { name: "CTA", color: "#F1EFE8", label: "Try", ink: "#2C2C2A" },
];

const boxes = cells.map((c, i) => {
  const word = text(c.label, { name: `${c.name} text` }).fill(c.ink ?? "#ffffff");
  word.set("Position", [0, 0]);
  const box = group(word).name(c.name);
  box.set("Display", "Flex").set("Flex Direction", "Column").set("Justify Content", "End")
    .set("Padding", [36, 28]).set("Background", c.color).set("Border Radius", 28);
  return box;
});
const bento = group(...boxes).name("Bento");
bento.set("Display", "Grid").set("Grid Columns", 4).set("Grid Rows", 3).set("Gap", 16)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1824).set("Height", 984)
  .set("Position", [48, 48]);
boxes.forEach((box, i) => {
  const c = cells[i];
  box.set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill");
  if (c.span) box.set("Column Span", c.span[0]).set("Row Span", c.span[1]);
  box.key("Position", 0.2 + i * 0.08, [0, 60], "Bezier").key("Position", 0.7 + i * 0.08, [0, 0]);
  box.key("Opacity", 0.2 + i * 0.08, 0, "Bezier").key("Opacity", 0.6 + i * 0.08, 1);
});
for (const layer of [bento, ...boxes]) layer.projection("2D");
