// anchor / parent: one keyed Hub (in its own group) names nothing; 98 tiles in a Flex group name it with Position Anchor.
// Their Effector takes its centre from the Hub's current centre (anchor_centre) instead of the sweep — one hand, 98 things.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#101014" });
const N = 14 * 7, tiles = [];
for (let k = 0; k < N; k++) tiles.push(rectangle({ name: `T${k}` }).fill("#5C7CFA").set("Scale", [90 / 270, 90 / 270]));
const room = group(...tiles).name("Room");
room.set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", 28)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1624).set("Height", 800).set("Position", [148, 120]);
tiles.forEach((t) => t.set("Position", [0, 0]));
const hub = ellipse({ name: "Hub" }).fill("#F2E6C8").set("Scale", [0.18, 0.18]);
group(hub).name("Hubs");
hub.keys("Position", [[0, [120, 900]], [3, [1800, 220]], [6, [120, 900]]]);
tiles.forEach((t) => {
  t.set("Position Anchor", hub);
  t.effect("Effector", { "Shape": "Sphere", "Sweep": 2.5, "Size": 260, "Soft": 140, "Strength": 1, "Lift": -70 });
});
text("anchor · 98 tiles name one keyed hub (Position Anchor) · the effector's centre is the hub's now-centre, not a sweep", { Position: [960, 1020] }).fill("#D0D0D8").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
