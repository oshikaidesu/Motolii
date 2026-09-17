// SANKOU! #2 化合 — the band opens on a picture, not on ink text: the content strip is a photo (the Web keeps this to text). On hover the grey label is clipped away from its
// bottom (inset(0 0 100% 0)) while the ink content is revealed from its bottom (inset(100% 0 0 0) → 0), both .4s
// cubic-bezier(.35,.75,.1,1), with a .2em push; hover out plays it back. Three links, hovered one after another.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#F4F1EA" });
const IO = { kind: "Bezier", x1: 0.35, y1: 0.75, x2: 0.1, y2: 1 };
const PHOTO = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/photo2.png";
const SIZE = 140, H = SIZE * 1.2, W = 1400, PUSH = SIZE * 0.2, T = 0.4;
const link = (word, y, hoverIn, hoverOut) => {
  // Two masks over the same spot. A box keeps its top edge, so the label just shrinks; the content box is moved up as it
  // grows and its text moved down by the same amount, which pins the content to the bottom edge (inset from the top).
  const strip = (name, color) => {
    const t = name === "content" ? media(PHOTO, { name: `${word} photo` }).set("Scale", [W / 1920, W / 1920])
      : text(word, { name: `${word} ${name}` }).fill(color).font("Helvetica Neue").set("Size", SIZE);
    const mask = group(t).name(`${word} ${name} mask`);
    mask.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H)
      .set("Overflow", "Clip").set("Position", [200, y]);
    t.set("Position", [0, 0]);
    return [mask, t];
  };
  const [label, labelText] = strip("label", "#9A958C");
  const [content, contentText] = strip("content", "#161616");
  for (const [at, on] of [[hoverIn, 1], [hoverOut, 0]]) {
    const [a, b] = on ? [0, 1] : [1, 0];
    label.key("Height", at, H * (1 - a), IO).key("Height", at + T, H * (1 - b));
    labelText.key("Position", at, [0, PUSH * a], IO).key("Position", at + T, [0, PUSH * b]);
    content.key("Height", at, H * a, IO).key("Height", at + T, H * b);
    content.key("Position", at, [200, y + H * (1 - a)], IO).key("Position", at + T, [200, y + H * (1 - b)]);
    contentText.key("Position", at, [0, -H * (1 - a) - PUSH * (1 - a)], IO).key("Position", at + T, [0, -H * (1 - b) - PUSH * (1 - b)]);
  }
};
link("WORKS", 240, 0.8, 2.0);
link("ABOUT", 240 + H + 30, 1.6, 3.0);
link("CONTACT", 240 + (H + 30) * 2, 2.6, 4.2);
