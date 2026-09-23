// Motolii Live — a bloom of petals: CSS gradients, a circular Repeater, a slow turn and a glow.
// Run it with `scripts/motolii-ui.sh dev <this file>`; saving the file redraws the window.
comp({ width: 1920, height: 1080, fps: 60, seconds: 8, background: "#0B0A12", loop: true });

// One petal, squashed and set out from the centre; the group turns a copy of it every 15°.
const petal = ellipse({ name: "Petal" })
  .fill("linear-gradient(to right, #7A2E8C, #FF5FA2 45%, #FFB36B 80%, #FFF1C9)")
  .set("Scale", [1.0, 0.2])
  .set("Position", [70, 0]);
const bloom = group(petal).name("Bloom").set("Position", [960, 540]);
bloom.effect("Repeater", { "Count": 24, "Position X Each": 0, "Position Y Each": 0, "Rotation Each": 15 }).whole();
bloom.keys("Rotation", [[0, 0, "Linear"], [8, 90]]);
bloom.effect("Glow", { "Intensity": 0.35, "Radius": 28 });

const core = ellipse({ name: "Core" })
  .fill("radial-gradient(#FFF6E0, #FF7A59 55%, #7A2E8C)")
  .set("Position", [960, 540]);
core.keys("Scale", [[0, [0.32, 0.32], "Bezier"], [4, [0.42, 0.42], "Bezier"], [8, [0.32, 0.32]]]);

text("MOTOLII", { name: "Title" }).fill("#FFF1C9").set("Position", [960, 1000]).set("Size", 48);
