// Codrops #3 — OnScrollTextHighlight (js/effect-1/highlightEffect.js): the letters stand up out of depth, one after another.
// Source: gsap.set(el, {perspective: 500}); gsap.timeline({defaults:{duration: 0.8, ease: 'power2'}})
// .fromTo(chars, {opacity: 0, z: 300, rotationX: -45}, {stagger: 0.04, opacity: 1, z: 0, rotationX: 0}, 0). onEnter = the line's moment.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#f2efe9" });
const P2 = "power2";
const SIZE = 150, EACH = 0.04, RISE = 0.8, INK = "#1c1a22";
const lines = ["LETTERS RISE", "OUT OF THE DEPTH"];
lines.forEach((line, row) => {
  const at = 0.3 + row * 0.9;
  // One text per line, Split: Chars; each char tilts about its own centre (SplitText's char), the line's Stagger hands out 0.04 s per char.
  const chars = text(line, { name: `Line ${row}` }).fill(INK).font("Helvetica Neue").set("Size", SIZE).set("Split", "Chars").set("Stagger", EACH * (line.replace(/ /g, "").length - 1));
  // opacity 0 → 1, z 300 → 0 (towards the viewer is −Z here), rotationX −45 → 0, all 0.8 s power2.
  chars.key("Opacity", at, 0, P2).key("Opacity", at + RISE, 1)
    .key("Position Z", at, -300, P2).key("Position Z", at + RISE, 0)
    .key("Tilt X", at, -45, P2).key("Tilt X", at + RISE, 0);
  const rowBox = group(chars).name(`Row ${row}`);
  rowBox.set("Display", "Flex").set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Position", [960 - line.length * SIZE * 0.36, 400 + row * (SIZE + 90)]);
  chars.set("Position", [0, 0]);
  for (const layer of [rowBox, chars]) layer.projection("3D");
});
