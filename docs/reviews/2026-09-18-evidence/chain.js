// Chain: the same tiles carry three blocks in a row (Effector → Wave → Zero Gravity), then the stage carries two pixel passes (Radiance → Overkill).
// Blocks compose like pixel effects compose: each returns an Offset, the shelf adds/multiplies them in order. Rust 0.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#101014" });
const N = 14 * 7, tiles = [];
for (let k = 0; k < N; k++) tiles.push(rectangle({ name: `T${k}` }).fill(k % 5 ? "#5C7CFA" : "#F2C14E").set("Scale", [90 / 270, 90 / 270]));
const room = group(...tiles).name("Room");
room.set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", 28)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1624).set("Height", 800).set("Position", [148, 120]);
tiles.forEach((t) => t.set("Position", [0, 0]));
tiles.forEach((t) => {
  t.effect("Effector", { "Shape": "Sphere", "Sweep": 2.5, "Size": 420, "Soft": 200, "Strength": 1, "Lift": -70 });
  t.effect("Wave", { "Amplitude": 22, "Frequency": 0.8, "Wavelength": 6, "Scale": 0.25, "Opacity": 0.3 });
  t.effect("Zero Gravity", { "Drift": 18, "Tumble": 8, "Breath": 0.05, "Slowness": 5 });
});
const stage = group(room).name("Stage");
stage.effect("Radiance", { "Threshold": 0.55, "Intensity": 2.5, "Reach": 70, "Air": 0.5 }).whole(true);
stage.effect("Overkill", { "Curve": 0.10, "Split": 0.010, "Scanlines": 0.25, "Ripple": 0.008, "Grain": 0.10, "Vignette": 0.6 }).whole(true);
text("chain · three blocks on every tile (effector → wave → zero gravity) · two passes on the stage (radiance → overkill)", { Position: [960, 1020] }).fill("#D0D0D8").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
