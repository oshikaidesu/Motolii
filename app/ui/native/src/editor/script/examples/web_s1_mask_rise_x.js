// SANKOU! #1 化合 — the same rise, but each line is a card lying flat in 2.5D that stands up as its words come out (Tilt X, z-space).
// Source: .a-up{overflow:hidden} .a-up__in{opacity:0;translate:0 100%} → translate:0 0, 1.2s cubic-bezier(.35,.75,.1,1), data-stagger.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#F4F1EA" });
const IO = { kind: "Bezier", x1: 0.35, y1: 0.75, x2: 0.1, y2: 1 };
const SPEED = 1.2, STAGGER = 0.12, SIZE = 150;
const lines = ["TOSHIYUKI HASHIMOTO", "DESIGN & DIRECTION", "TOKYO — 2026"];
lines.forEach((line, row) => {
  const at = 0.3 + row * 0.35;
  // One text layer per line, Split: Words (SplitText 'words'); the line's own Stagger hands each word its clock (data-stagger).
  const t = text(line, { name: `Line ${row}` }).fill(row === 1 ? "#C8412B" : "#161616").font("Helvetica Neue").set("Size", SIZE)
    .set("Split", "Words").set("Stagger", STAGGER * (line.split(" ").length - 1));
  // .a-up: the line's own box is the mask.
  const mask = group(t).name(`Mask ${row}`);
  mask.set("Display", "Flex").set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Overflow", "Clip").set("Position", [160, 300 + row * (SIZE + 40)]);
  t.set("Position", [0, 0]).set("Opacity", 0)
    .key("Position", at, [0, SIZE], IO).key("Position", at + SPEED, [0, 0])
    .key("Opacity", at, 0, IO).key("Opacity", at + SPEED, 1);
  mask.key("Tilt X", at, -70, IO).key("Tilt X", at + SPEED, 0).key("Position Z", at, 400, IO).key("Position Z", at + SPEED, 0);
  for (const layer of [mask, t]) layer.projection("3D");
});
