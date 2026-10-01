// A Swiss grid that changes its mind: the column count jumps (6 → 4 → 8 → 6), and every cell slides to its new place,
// the ones near the centre first and the corners last (CSS transition-delay, handed out by the grid like GSAP's stagger).
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#F2EDE4" });

const INK = "#161616", PAPER = "#F2EDE4", RED = "#E4572E", BLUE = "#2E5EAA", YELLOW = "#F3B61F";
const palette = [RED, INK, YELLOW, BLUE, INK, RED, PAPER, BLUE];
const letters = "MOTOLII LAYOUT".replace(" ", "");
const cells = [];
for (let i = 0; i < 24; i++) {
  const color = palette[i % palette.length];
  if (i % 4 === 0) {
    const ch = letters[(i / 4) % letters.length];
    const t = text(ch, { name: `Letter ${ch}` }).fill(color === YELLOW || color === PAPER ? INK : PAPER).font("Helvetica Neue");
    t.set("Position", [0, 0]).set("Scale", [1.2, 1.2]);
    const cell = group(t).name(`Cell ${ch}`);
    cell.set("Display", "Flex").set("Justify Content", "Center").set("Align Items", "Center").set("Background", color).set("Border Radius", 12).set("Overflow", "Clip");
    cells.push(cell);
  } else if (i % 3 === 1) {
    cells.push(ellipse({ name: "Dot" }).fill(color));
  } else {
    cells.push(rectangle({ name: "Block" }).fill(color));
  }
}

const grid = group(...cells).name("Grid");
grid.set("Display", "Grid").set("Grid Columns", 6).set("Gap", 12).set("Padding", [12, 12])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1840).set("Height", 1000).set("Position", [40, 40])
  .set("Stagger", 0.7).set("Stagger From", "Center");
grid.keys("Grid Columns", [[0, 6, "Hold"], [1.2, 4, "Hold"], [2.7, 8, "Hold"], [4.2, 6, "Hold"]]);
cells.forEach((cell) => {
  cell.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill").set("Transition Duration", 0.6).set("Transition Easing", "Ease In Out");
});
cells.filter((c) => c.json().name === "Dot").forEach((d) => d.set("Object Fit", "Contain"));
for (const layer of [grid, ...cells]) layer.projection("2D");
