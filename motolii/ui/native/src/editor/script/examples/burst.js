// A burst that ends as a grid: the same Arrive, with the distance on the other side of zero, so the
// tiles start on top of each other in the middle and fly out to where the layout wants them.
comp({ width: 1080, height: 1080, fps: 30, seconds: 4, background: "#F5F2EC" });
const C = ["#1B1B1B", "#E5533C", "#2C6E8F", "#E8B93B", "#7B61FF"];
const tiles = [];
for (let k = 0; k < 36; k++) tiles.push(rectangle({ name: `Tile ${k}` }).fill(C[k % C.length]));
const board = group(...tiles).name("Board");
board.set("Display", "Grid").set("Grid Columns", 6).set("Grid Rows", 6).set("Gap", 18)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 880).set("Height", 880)
  .set("Background", "#F5F2EC").set("Position", [100, 100]);
tiles.forEach((t) => t.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill")
  .effect("Arrive", { "Radial": 1, "Distance": -520, "Arrive": 1.1, "Bounce": 0.3, "Stagger": 0.015, "Spin": 60 }));
