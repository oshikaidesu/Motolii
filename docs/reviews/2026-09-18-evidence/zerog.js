// Zero gravity. One room, one field with no pull and a slow turn: the words drift, touch, tumble.
// Ropes with no slack tie neighbours — in zero g a rope does not sag, it only lags.
comp({ width: 1080, height: 1080, fps: 30, seconds: 8, background: "#14141A" });
const INK = "#F0ECE3", RED = "#E8442E", BLUE = "#6C8CFF", SAND = "#E9C46A";
const words = ["drift", "no gravity", "tumble", "touch", "one field", "eight words", "seven ropes", "zero g"];
const C = [INK, RED, SAND, BLUE];
const rng = random(5);
const things = words.map((w, k) => text(w, { name: `W${k}` }).fill(C[k % C.length]).font("Helvetica Neue").set("Scale", [0.42, 0.42]));
const field = ellipse({ name: "Field" }).fill("#00000000").set("Scale", [10 / 270, 10 / 270]);
const room = group(...things, field).name("Room");
room.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 960).set("Height", 960).set("Background", "#1C1D24").set("Border Radius", 40).set("Position", [60, 60]);
things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 6).set("Hardness", 0.6).set("Heaviness", 0.2)
  .set("Position", [120 + rng() * 620, 120 + rng() * 620]).set("Rotation", rng() * 40 - 20));
field.set("Position Type", "Absolute").set("Position", [480, 480]);
// No gravity: a field that only turns, weakly. Everything in the room drifts around it.
field.effect("Field", { "Spread": 1, "Angle": 0, "Strength": 120, "Turn": 1 });
for (let k = 0; k + 1 < things.length; k++) {
  line({ name: `Rope ${k}` }).fill(INK).set("Connect From", things[k]).set("Connect To", things[k + 1])
    .set("Line Path", "Rope").set("Slack", 0).set("Stroke Width", 1.5).set("Dash", 4).set("Dash Gap", 6);
}
text("zero g · one field that only turns · ropes without slack", { Position: [540, 1046] }).fill(INK).font("Helvetica Neue").set("Scale", [0.2, 0.2]);
