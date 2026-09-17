// SANKOU! #1 — toshiyukihashimoto.jp: words rise out of a mask, one after another.
// Source: .a-up{overflow:hidden} .a-up__in{opacity:0;translate:0 100%} → translate:0 0, 1.2s cubic-bezier(.35,.75,.1,1), data-stagger.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#F4F1EA" });
const IO = { kind: "Bezier", x1: 0.35, y1: 0.75, x2: 0.1, y2: 1 };
const SPEED = 1.2, STAGGER = 0.12, SIZE = 150;
const lines = ["TOSHIYUKI HASHIMOTO", "DESIGN & DIRECTION", "TOKYO — 2026"];
lines.forEach((line, row) => {
  const at = 0.3 + row * 0.35;
  const words = line.split(" ").map((w) => {
    const t = text(w, { name: `${w} ${row}` }).fill(row === 1 ? "#C8412B" : "#161616").font("Helvetica Neue").set("Size", SIZE);
    // .a-up: the word's own box is the mask.
    const mask = group(t).name(`Mask ${w} ${row}`);
    mask.set("Display", "Flex").set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Overflow", "Clip");
    t.set("Position", [0, 0]).set("Opacity", 0)
      .key("Position", at, [0, SIZE], IO).key("Position", at + SPEED, [0, 0])
      .key("Opacity", at, 0, IO).key("Opacity", at + SPEED, 1);
    return mask;
  });
  // The line hands out the delay: every word has the same keys, the row shifts each word's clock (data-stagger).
  const rowBox = group(...words).name(`Line ${row}`);
  rowBox.set("Display", "Flex").set("Gap", 44).set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug")
    .set("Stagger", STAGGER * (words.length - 1)).set("Position", [160, 300 + row * (SIZE + 40)]);
  words.forEach((m) => m.set("Position", [0, 0]));
});
