// Codrops #3 化合 — the same rise, but no Opacity keys: a camera with Near Fade thins whatever comes too close to the lens,
// so the letters fade in because they start near the camera (real depth, not a per-letter opacity curve).
// Codrops #3 — OnScrollTextHighlight (js/effect-1/highlightEffect.js): the letters stand up out of depth, one after another.
// Source: gsap.set(el, {perspective: 500}); gsap.timeline({defaults:{duration: 0.8, ease: 'power2'}})
// .fromTo(chars, {opacity: 0, z: 300, rotationX: -45}, {stagger: 0.04, opacity: 1, z: 0, rotationX: 0}, 0). onEnter = the line's moment.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#f2efe9" });
const P2 = { kind: "Bezier", x1: 0.215, y1: 0.61, x2: 0.355, y2: 1 };
const SIZE = 150, EACH = 0.04, RISE = 0.8, INK = "#1c1a22";
const lines = ["LETTERS RISE", "OUT OF THE DEPTH"];
camera({ name: "Camera" }).set("Near Fade", 1000);
lines.forEach((line, row) => {
  const at = 0.3 + row * 0.9;
  const chars = [...line].map((ch, k) => {
    if (ch === " ") return rectangle({ name: `Space ${row}.${k}` }).set("Opacity", 0).set("Scale", [50 / 270, 1 / 270]);
    const t = text(ch, { name: `${ch} ${row}.${k}` }).fill(INK).font("Helvetica Neue").set("Size", SIZE);
    // opacity 0 → 1, z 300 → 0 (towards the viewer is −Z here), rotationX −45 → 0, all 0.8 s power2.
    return t.key("Position Z", at, -700, P2).key("Position Z", at + RISE, 0)
      .key("Tilt X", at, -45, P2).key("Tilt X", at + RISE, 0);
  });
  const rowBox = group(...chars).name(`Line ${row}`);
  rowBox.set("Display", "Flex").set("Align Items", "Center").set("Gap", 4).set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug")
    .set("Stagger", EACH * (chars.length - 1)).set("Position", [960 - line.length * SIZE * 0.36, 400 + row * (SIZE + 90)]);
  chars.forEach((c) => c.set("Position", [0, 0]));
  for (const layer of [rowBox, ...chars]) layer.projection("3D");
});
