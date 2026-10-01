// A grid that lands: the layout already says where every card belongs, so all this says is how they
// arrive. No keys, no positions — one effect per card.
comp({ width: 1080, height: 1080, fps: 30, seconds: 4, background: "#0F1115" });
const C = ["#E7E7E7", "#FF5470", "#4ECDC4", "#FFD166"];
const cards = [];
for (let k = 0; k < 16; k++) cards.push(rectangle({ name: `Card ${k}` }).fill(C[k % C.length]));
const board = group(...cards).name("Board");
board.set("Display", "Grid").set("Grid Columns", 4).set("Grid Rows", 4).set("Gap", 26)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 860).set("Height", 860)
  .set("Background", "#171A21").set("Position", [110, 110]);
cards.forEach((c) => c.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill")
  .effect("Arrive", { "From": -90, "Distance": 950, "Arrive": 0.75, "Bounce": 0.45, "Stagger": 0.05, "Spin": 24 }));
