// Pinterest's "objects tipped out of frame": a field with Spread 1 pours them down, Turn tips them
// sideways as they go, and they settle on each other. One field, one moving source.
comp({ width: 1440, height: 1080, fps: 30, seconds: 6, background: "#101014" });
const D = 270, rng = random(12), C = ["#F7B267", "#F4845F", "#F25C54", "#7EBDC2", "#EDEDED"];
const things = [], spots = [];
for (let k = 0; k < 18; k++) {
  const w = 90 + rng() * 140, h = 60 + rng() * 120;
  things.push((k % 5 === 0 ? ellipse : rectangle)({ name: `T${k}` }).fill(C[k % C.length]).set("Scale", [w / D, h / D]));
  spots.push([120 + rng() * 900, -160 - rng() * 900]);
}
const g = ellipse({ name: "Gravity" }).fill("#101014").set("Scale", [2 / D, 2 / D]);
const room = group(...things, g).name("Shelf");
room.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 1160).set("Height", 800).set("Background", "#17171D").set("Border Radius", 20).set("Position", [140, 180]);
things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 6).set("Position", spots[k]));
g.set("Position Type", "Absolute").set("Position", [580, 400]);
g.effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1100, "Tumble": 0.75 });
