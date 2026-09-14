// Squash and stretch on the grid itself: the rows under the ball squash when it lands, the lines breathe.
comp({ width: 1920, height: 1080, fps: 30, seconds: 3, background: "#0b0b0b" });

const tiles = [];
for (let i = 0; i < 80; i++) tiles.push(rectangle({ name: `Tile ${i + 1}` }).fill("#F1EFE8"));
const ball = ellipse({ name: "Ball" }).fill("#EF9F27");
const grid = group(ball, ...tiles).name("Grid");
grid.set("Display", "Grid").set("Grid Columns", 9).set("Grid Rows", 9).set("Gap", 4)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 900).set("Height", 900)
  .set("Background", "#0b0b0b").set("Position", [510, 90]);
for (const tile of tiles) tile.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill");
ball.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill").set("Object Fit", "Contain")
  .set("Column Start", 5);

// The ball falls row by row, lands on row 9, and goes back up.
const rows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 8, 7, 6, 5, 4, 3, 2, 1];
rows.forEach((row, i) => ball.key("Row Start", i * (3 / rows.length), row, "Hold"));
const land = 8.5 * (3 / rows.length);
const at = (dt) => Math.max(0, land + dt);
grid.key("Row 9", at(-0.25), 1, "Bezier").key("Row 9", land, 0.3, "Bezier").key("Row 9", at(0.3), 1);
grid.key("Row 8", at(-0.25), 1, "Bezier").key("Row 8", land, 1.7, "Bezier").key("Row 8", at(0.3), 1);
grid.key("Column 5", at(-0.25), 1, "Bezier").key("Column 5", land, 1.9, "Bezier").key("Column 5", at(0.3), 1);
grid.key("Column 4", at(-0.25), 1, "Bezier").key("Column 4", land, 0.55, "Bezier").key("Column 4", at(0.3), 1);
grid.key("Column 6", at(-0.25), 1, "Bezier").key("Column 6", land, 0.55, "Bezier").key("Column 6", at(0.3), 1);
for (const layer of [grid, ball, ...tiles]) layer.projection("2D");
