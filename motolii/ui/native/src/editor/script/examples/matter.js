// One dial, three materials: Hardness 0 bounces, 0.5 absorbs, 1 drags and piles.
// Nothing else differs — same shapes, same drop, same field.
comp({ width: 1600, height: 1000, fps: 30, seconds: 6, background: "#F2EEE6" });
const D = 270, rng = random(21);
const WELLS = [
  { name: "BOUNCE", hardness: 0, ink: "#E8442E" },
  { name: "ABSORB", hardness: 0.5, ink: "#1B4B8F" },
  { name: "DRAG", hardness: 1, ink: "#111111" },
];
WELLS.forEach((wl, i) => {
  const x = 170 + i * 430;
  const things = [], spots = [];
  for (let k = 0; k < 9; k++) {
    const side = 70 + rng() * 40;
    things.push((k % 3 === 0 ? ellipse : rectangle)({ name: `${wl.name} ${k}` }).fill(wl.ink).set("Scale", [side / D, side / D]));
    spots.push([120 + rng() * 80, -120 - k * 180]);
  }
  const g = ellipse({ name: `G${i}` }).fill("#F2EEE6").set("Scale", [2 / D, 2 / D]);
  const well = group(...things, g).name(wl.name);
  well.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
    .set("Width", 340).set("Height", 620).set("Background", "#E4DED2").set("Border Radius", 24).set("Position", [x, 150]);
  things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 4).set("Hardness", wl.hardness).set("Position", spots[k]));
  g.set("Position Type", "Absolute").set("Position", [170, 310]);
  g.effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1400 });
  text(wl.name, { Position: [x + 170, 860] }).fill("#111111").font("Helvetica Neue").set("Scale", [0.34, 0.34]);
});
text("hardness", { Position: [800, 90] }).fill("#111111").font("Helvetica Neue").set("Scale", [0.5, 0.5]);
