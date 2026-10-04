// Production UI oracle: a small work with two shapes, text, and keyed motion (integration_test bootstrap).
comp({ width: 1920, height: 1080, fps: 30, seconds: 10, background: "#101114" });
const a = rectangle().name("Shape A");
const b = rectangle().name("Shape B");
text("Motolii").name("Title");
a.keys("Position", [[0, [320, 540]], [5, [960, 540]]]);
b.keys("Position", [[0, [640, 400]], [3, [1200, 600]]]);
