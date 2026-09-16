// One box, one field, one dial. Things fall in and settle; then the field turns from "down"
// to "here" and the same things gather. Nothing is keyed but the field's two dials.
comp({ width: 1920, height: 1080, fps: 30, seconds: 9, background: "#F0ECE3" });
const D = 270, rng = random(17);
const INK = "#14141A", RED = "#E8442E", BLUE = "#2B4C8C", SAND = "#E9C46A";
const C = [RED, BLUE, SAND, "#FFFFFF"];

const things = [], spots = [];
for (let k = 0; k < 18; k++) {
  const side = 90 + rng() * 80;
  things.push((k % 4 === 0 ? ellipse : rectangle)({ name: `T${k}` }).fill(C[k % C.length]).set("Scale", [side / D, side / D]));
  spots.push([240 + rng() * 340, -120 - k * 130]);
}
const field = ellipse({ name: "Field" }).fill("#00000000").set("Scale", [10 / D, 10 / D]);
const well = group(...things, field).name("Well");
well.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 820).set("Height", 820).set("Background", "#E4DDD0").set("Border Radius", 410)
  .set("Position", [900, 130]);
things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 5).set("Hardness", 0.3).set("Position", spots[k]));
field.set("Position Type", "Absolute").set("Position", [410, 410]);
// The only keys in the piece: the field turns from a direction into a place.
const f = field.effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1500, "Turn": 0 });
f.key("Spread", 4.2, 1, "Bezier").key("Spread", 5.6, 0);
f.key("Strength", 4.2, 1500, "Bezier").key("Strength", 5.6, 900);

text("PHYSICS", { Position: [430, 300] }).fill(INK).font("Helvetica Neue").set("Scale", [1.05, 1.05]);
text("IS A BOX", { Position: [430, 430] }).fill(RED).font("Helvetica Neue").set("Scale", [1.05, 1.05]);
text("fall · settle · gather", { Position: [300, 530] }).fill(INK).font("Helvetica Neue").set("Scale", [0.3, 0.3]);
text("one field · two dials · no keyframes on the things", { Position: [460, 980] }).fill(INK).font("Helvetica Neue").set("Scale", [0.22, 0.22]);

// The trace comes up for a beat, then goes: the picture explains itself and goes back to being a picture.
const trace = rectangle({ name: "Trace" }).fill("#ffffff").set("Anchor", [0, 0]).set("Position", [0, 0]);
trace.effect("Physics Trace", { "Contact Line Color": "#E8442E", "Field Color": "#2B4C8C", "Color Box Stroke": "#14141A" });
trace.keys("Opacity", [[0, 0, "Hold"], [2.6, 0, "Bezier"], [3.2, 1], [4.0, 1, "Bezier"], [4.6, 0]]);
