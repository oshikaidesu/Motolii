// A magnet in a box: everything gathers, packs, and holds. One dial at 0 = pull.
comp({ width: 1080, height: 1080, fps: 30, seconds: 5, background: "#F4F1EA" });
const D = 270, rng = random(9), C = ["#1B1B1B", "#E5533C", "#2C6E8F", "#E8B93B"];
const things = [], spots = [];
for (let k = 0; k < 20; k++) {
  const side = 80 + rng() * 70;
  things.push((k % 4 === 0 ? ellipse : rectangle)({ name: `T${k}` }).fill(C[k % C.length]).set("Scale", [side / D, side / D]));
  spots.push([40 + rng() * 760, 40 + rng() * 760]);
}
const m = ellipse({ name: "Magnet" }).fill("#F4F1EA").set("Scale", [2 / D, 2 / D]);
const room = group(...things, m).name("Board");
room.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 900).set("Height", 900).set("Background", "#EAE5DA").set("Border Radius", 16).set("Position", [90, 90]);
things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 8).set("Position", spots[k]));
m.set("Position Type", "Absolute").set("Position", [450, 450]);
m.effect("Field", { "Spread": 0, "Turn": 0, "Strength": 150, "Tumble": 0.8 });
