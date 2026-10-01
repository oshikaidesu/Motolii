// Kinetic type without keys: each letter is a layer in a row, the row says where they belong,
// and Arrive says how they get there — in from the outside, spinning, landing one after another.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#0E1013" });
const word = [..."MOTOLII"];
const glyphs = word.map((ch, k) => text(ch, { name: `Glyph ${ch}${k}` }).fill(k % 2 ? "#FF5470" : "#F3F3F3").font("Helvetica Neue").set("Scale", [1.6, 1.6]));
const row = group(...glyphs).name("Word");
row.set("Display", "Flex").set("Gap", 18).set("Padding", 40).set("Align Items", "Center")
  .set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Position", [520, 470]);
glyphs.forEach((g) => g.set("Position", [0, 0])
  .effect("Arrive", { "From": -90, "Distance": 800, "Radial": 0.85, "Arrive": 0.7, "Bounce": 0.5, "Stagger": 0.07, "Spin": 40 }));
