// The same field turned 80 degrees: things spiral in and settle into a wheel.
comp({ width: 1080, height: 1080, fps: 30, seconds: 6, background: "#101014" });
const D = 270, rng = random(5), C = ["#7B61FF", "#33D9B2", "#FFC93C", "#FF6B6B"];
const things = [], spots = [];
for (let k = 0; k < 26; k++) {
  const side = 50 + rng() * 60;
  things.push(rectangle({ name: `T${k}` }).fill(C[k % C.length]).set("Scale", [side / D, (side * (0.4 + rng())) / D]));
  spots.push([40 + rng() * 800, 40 + rng() * 800]);
}
const eye = ellipse({ name: "Eye" }).fill("#FFFFFF").set("Scale", [18 / D, 18 / D]);
const room = group(...things, eye).name("Swirl");
room.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 900).set("Height", 900).set("Background", "#16161C").set("Border Radius", 450).set("Position", [90, 90]);
things.forEach((t, k) => t.set("Position Type", "Absolute").set("Margin", 5).set("Position", spots[k]));
eye.set("Position Type", "Absolute").set("Position", [450, 450]);
eye.effect("Field", { "Spread": 0, "Turn": 80, "Strength": 120, "Tumble": 1.2 });
