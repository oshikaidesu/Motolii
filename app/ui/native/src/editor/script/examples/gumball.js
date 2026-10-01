// A jar filling with beads: 60 circles, one field, no keys. Real circles collide as circles.
comp({ width: 1080, height: 1350, fps: 30, seconds: 7, background: "#EFE9DD" });
const D = 270, rng = random(31);
const C = ["#E8442E", "#2B4C8C", "#E9C46A", "#2A9D8F", "#1A1A1F", "#FFFFFF"];
const beads = [], spots = [];
for (let k = 0; k < 60; k++) {
  const d = 70 + rng() * 60;
  beads.push(ellipse({ name: `B${k}` }).fill(C[k % C.length]).set("Scale", [d / D, d / D]));
  spots.push([300 + rng() * 240, -80 - k * 90]);
}
const g = ellipse({ name: "Gravity" }).fill("#00000000").set("Scale", [6 / D, 6 / D]);
const jar = group(...beads, g).name("Jar");
jar.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 840).set("Height", 1000).set("Background", "#E3DBCB").set("Border Radius", 60).set("Position", [120, 200]);
beads.forEach((b, k) => b.set("Position Type", "Absolute").set("Margin", 2).set("Hardness", 0.12).set("Position", spots[k]));
g.set("Position Type", "Absolute").set("Position", [420, 500]);
g.effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1800 });
text("sixty beads", { Position: [540, 130] }).fill("#1A1A1F").font("Helvetica Neue").set("Scale", [0.34, 0.34]);
