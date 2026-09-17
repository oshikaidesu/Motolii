// SANKOU! #2 — toshiyukihashimoto.jp: a heading opens with a band. On hover the grey label is clipped away from its
// bottom (inset(0 0 100% 0)) while the ink content is revealed from its bottom (inset(100% 0 0 0) → 0), both .4s
// cubic-bezier(.35,.75,.1,1), with a .2em push; hover out plays it back. Three links, hovered one after another.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#F4F1EA" });
const IO = { kind: "Bezier", x1: 0.35, y1: 0.75, x2: 0.1, y2: 1 };
const SIZE = 140, H = SIZE * 1.2, W = 1400, PUSH = SIZE * 0.2, T = 0.4;
const link = (word, y, hoverIn, hoverOut) => {
  // Two boxes over the same spot, each a clip-path: inset(). The label loses its bottom (Clip Bottom 0 → H), the content
  // gains its top (Clip Top H → 0); the text inside is pushed by .2em. One keyed edge per strip.
  const strip = (name, color, edge, sign) => {
    const t = text(word, { name: `${word} ${name}` }).fill(color).font("Helvetica Neue").set("Size", SIZE);
    const box = group(t).name(`${word} ${name} box`);
    box.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H).set("Position", [200, y]);
    t.set("Position", [0, 0]);
    for (const [at, on] of [[hoverIn, 1], [hoverOut, 0]]) {
      const [a, b] = (on === 1) === (sign > 0) ? [0, 1] : [1, 0];
      box.key(edge, at, H * a, IO).key(edge, at + T, H * b);
      t.key("Position", at, [0, sign * PUSH * a], IO).key("Position", at + T, [0, sign * PUSH * b]);
    }
  };
  strip("label", "#9A958C", "Clip Bottom", 1);
  strip("content", "#161616", "Clip Top", -1);
};
link("WORKS", 240, 0.8, 2.0);
link("ABOUT", 240 + H + 30, 1.6, 3.0);
link("CONTACT", 240 + (H + 30) * 2, 2.6, 4.2);
