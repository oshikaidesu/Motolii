// Physics Trace: the same shot twice — the picture, and what the physics is doing under it.
// Boxes are where the solver put them, the lines are the things that are touching right now.
comp({ width: 1400, height: 900, fps: 30, seconds: 6, background: "#0D0E12" });
const D = 270, rng = random(8), C = ["#F25C54", "#7EBDC2", "#F7B267", "#EDEDED"];
const things = [], spots = [];
for (let k = 0; k < 16; k++) {
  const side = 90 + rng() * 80;
  things.push((k % 4 === 0 ? ellipse : rectangle)({ name: `T${k}` }).fill(C[k % C.length]).set("Scale", [side / D, side / D]));
  spots.push([180 + rng() * 560, -140 - k * 120]);
}
const g = ellipse({ name: "Gravity" }).fill("#0D0E12").set("Scale", [2 / D, 2 / D]);
// The trace layer lives in the same room, on top: it reads the boxes below it.
const room = group(...things, g).name("Room");
room.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 940).set("Height", 700).set("Background", "#151821").set("Border Radius", 18).set("Position", [230, 120]);
things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 5).set("Hardness", 0.35).set("Position", spots[k]));
g.set("Position Type", "Absolute").set("Position", [470, 350]);
g.effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1500 });
// The trace layer is comp-sized and sits on top: it reads what the solver is holding.
rectangle({ name: "Trace" }).fill("#ffffff").set("Anchor", [0, 0]).set("Position", [0, 0])
  .effect("Physics Trace", { "Contact Line Color": "#FFD166", "Field Color": "#4ECDC4" });
