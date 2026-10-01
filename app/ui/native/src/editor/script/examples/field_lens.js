// A field made of a box (C4D's Box field on a Plain effector): a lens drifts over a grid of dots.
// Dots near the lens swell and step aside; the farther away, the less — and at the falloff, nothing.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#0e0f12" });

const lens = roundedRectangle({ name: "Lens" }).fill("#F2EDE4").set("Opacity", 0.08).set("Scale", [1.1, 0.8]);
lens.keys("Position", [[0, [420, 360], "Bezier"], [2, [1260, 300], "Bezier"], [4, [1440, 760], "Bezier"], [6, [520, 700]]]);

const dots = ellipse({ name: "Dots" }).fill("#E4572E").set("Scale", [0.06, 0.06]).set("Position", [140, 110]);
dots.effect("Repeater", { "Along": "Grid", "Count": 330, "Columns": 30, "Position X Each": 57, "Position Y Each": 80 });
dots.set("Field", lens).set("Field Falloff", 260).set("Field Scale", 3.2).set("Field Push", 36);

const ink = ellipse({ name: "Ink dots" }).fill("#F2EDE4").set("Scale", [0.025, 0.025]).set("Position", [168, 150]);
ink.effect("Repeater", { "Along": "Grid", "Count": 330, "Columns": 30, "Position X Each": 57, "Position Y Each": 80 });
ink.set("Field", lens).set("Field Falloff", 260).set("Field Scale", 0.2).set("Field Push", -20);

const caption = text("a box as a field", { name: "Caption" }).fill("#F2EDE4").font("Helvetica Neue").set("Scale", [0.18, 0.18]).set("Position", [140, 1000]);

for (const layer of [lens, dots, ink, caption]) layer.projection("2D");
