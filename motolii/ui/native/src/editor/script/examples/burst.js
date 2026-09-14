// A logo burst: a star draws itself on, copies fan out around it, sparks fly, the camera leans in.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#05060a" });

const center = [960, 540];

// Sparks behind everything.
particles({ name: "Sparks", Rate: 0, Speed: 900, Spread: 360, Life: 1.4, Size: 12, Gravity: 300, Seed: 3 })
  .set("Color", "#ffd166")
  .set("Color at Death", "#ff4d6d")
  .key("Rate", 0.9, 0, "Hold")
  .key("Rate", 1.0, 1200, "Hold")
  .key("Rate", 1.3, 0);

// The star draws itself on.
const logo = star({ name: "Logo", Position: center }).fill("#ffffff");
logo.effect("Trim Paths").key("End", 0, 0, "Bezier").key("End", 1, 100);
logo.key("Rotation", 0, -45, "Elastic").key("Rotation", 1.4, 0);
logo.effect("Motion Blur");

// Six copies around a circle (a choice is written by its name: Along "Circle").
ellipse({ name: "Petals", Position: center, Scale: [0.12, 0.12] })
  .fill("#4cc9f0")
  .key("Opacity", 0.8, 0)
  .key("Opacity", 1, 1)
  .effect("Repeater", { Count: 6, Along: "Circle", Radius: 320 });

text("BURST", { name: "Word", Position: [960, 860] })
  .fill("#ffffff")
  .key("Opacity", 1.6, 0, "Bezier")
  .key("Opacity", 2.2, 1)
  .key("Position", 1.6, [960, 900], "Bezier")
  .key("Position", 2.2, [960, 860]);
