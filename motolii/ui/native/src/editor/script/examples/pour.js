// Poured in and settled: things fall, tumble, and lean on each other. Nothing but a field.
comp({ width: 1080, height: 1080, fps: 30, seconds: 5, background: "#0E0E12" });
const D = 270, rng = random(3), C = ["#FF5470", "#FDE24F", "#5BC0EB", "#F2F2F2"];
const things = [], spots = [];
for (let k = 0; k < 24; k++) {
  const side = 70 + rng() * 80;
  things.push((k % 3 === 0 ? ellipse : rectangle)({ name: `T${k}` }).fill(C[k % C.length]).set("Scale", [side / D, side / D]));
  spots.push([90 + rng() * 620, -200 - rng() * 700]);
}
const g = ellipse({ name: "Gravity" }).fill("#0E0E12").set("Scale", [2 / D, 2 / D]);
const room = group(...things, g).name("Jar");
room.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 840).set("Height", 840).set("Background", "#16161C").set("Border Radius", 420).set("Position", [120, 120]);
things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 6).set("Position", spots[k]));
g.set("Position Type", "Absolute").set("Position", [420, 420]);
g.effect("Field", { "Spread": 1, "Angle": 90, "Strength": 900, "Tumble": 0.5 });
