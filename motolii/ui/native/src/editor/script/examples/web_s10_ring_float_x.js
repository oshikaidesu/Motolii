// SANKOU! #10 化合 — the same turning stage is a physics room: a gravity field at its centre, the things drop off the ring and
// tumble along the walls as the room keeps turning (the walls belong to the box; nothing leaves it).
// mamamamamamama.com / recruit.toyox.co.jp: a ring of things keeps turning, and things beside it float.
// Source (mama): .home-objects__stage{animation:home-objects-spin 360s linear infinite} @keyframes home-objects-spin{to{rotate(360deg)}}
// — the pictures ride the ring and turn with it (no counter-rotation). Here the period is the comp: 360° over 6 s, linear.
// Source (toyox): @keyframes floating-y{0%{translateY(-10%)}100%{translateY(10%)}} `1.8s ease-in-out infinite alternate`
// (= cubic-bezier(.42,0,.58,1)); `--reverse` starts at the other end. infinite alternate = Loop Duration 1.8, Loop Direction Alternate.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#F5F1E8" });
const IO = { kind: "Bezier", x1: 0.42, y1: 0, x2: 0.58, y2: 1 }, D = 270, SEC = 6, R = 360;
const C = ["#E8442E", "#2B4C8C", "#E9C46A", "#14141A", "#7EBDC2", "#C8412B", "#8E7CC3", "#3A8F5A"];
const orbit = [["Orbit", "#14141A", 2 * R + 4], ["Orbit hole", "#F5F1E8", 2 * R - 4]].map(([n, ink, d]) => ellipse({ name: n }).fill(ink).set("Scale", [d / D, d / D]));
const things = C.map((ink, i) => (i % 2 ? ellipse : rectangle)({ name: `Thing ${i}` }).fill(ink).set("Scale", [110 / D, 110 / D]));
const g = ellipse({ name: "Gravity" }).fill("#F5F1E800").set("Scale", [2 / D, 2 / D]);
const stage = group(...things, g).name("Stage");
stage.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 900).set("Height", 900)
  .set("Transform Origin", "Center").set("Position", [960, 540]);
orbit.forEach((o) => o.set("Position", [960, 540]));
g.set("Position Type", "Absolute").set("Position", [450, 450]).effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1500 });
things.forEach((t, i) => t.set("Position Type", "Absolute").set("Hardness", 0.3)
  .set("Position", [450 + Math.cos((i / C.length) * Math.PI * 2) * R, 450 + Math.sin((i / C.length) * Math.PI * 2) * R]));
stage.key("Rotation", 0, 0, "Linear").key("Rotation", SEC, 360);
for (const layer of [stage, ...orbit, ...things]) layer.projection("2D");
// The floating badges (c-floating --Y), the middle one --reverse.
[["SAUNA", 0], ["RIVER", 1], ["FOREST", 0]].forEach(([word, reverse], k) => {
  const y = 300 + k * 240, amp = 12 * (reverse ? -1 : 1);
  const badge = text(word, { name: word }).fill("#FFFFFF").font("Helvetica Neue").set("Size", 44).set("Position", [0, 0]);
  const pill = group(badge).name(`${word} pill`);
  pill.set("Display", "Flex").set("Padding", [24, 40]).set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Background", "#14141A").set("Border Radius", 60);
  badge.set("Position", [0, 0]);
  pill.set("Loop Duration", 1.8).set("Loop Direction", "Alternate").key("Position", 0, [1500, y - amp], IO).key("Position", 1.8, [1500, y + amp]);
});
