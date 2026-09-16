// The layout is the ground: the grid stays home, and the field only bulges it as it passes.
// Hold 0.8 keeps every tile on its leash; nothing is keyed but the little white mover.
comp({ width: 1080, height: 1080, fps: 30, seconds: 6, background: "#0D0D10" });
const tiles = [];
for (let k = 0; k < 144; k++) tiles.push(rectangle({ name: `Tile ${k}` }).fill(k % 3 ? "#EDEDED" : "#FF5470"));
const mover = ellipse({ name: "Mover" }).fill("#4ECDC4");
const grid = group(mover, ...tiles).name("Grid");
grid.set("Display", "Grid").set("Grid Columns", 12).set("Grid Rows", 12).set("Gap", 14)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 880).set("Height", 880)
  .set("Background", "#131318").set("Position", [100, 100]);
tiles.forEach((t) => t.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill"));
mover.set("Position Type", "Absolute").set("Scale", [0.09, 0.09])
  .keys("Position", [[0, [140, 140], "Bezier"], [2, [700, 260], "Bezier"], [4, [260, 700], "Bezier"], [6, [140, 140]]]);
mover.effect("Field", { "Spread": 0, "Turn": 180, "Strength": 260, "Reach": 420, "Hold": 0.8, "Tumble": 0.5 });
