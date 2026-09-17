// SANKOU! #5 化合 — the same four-edge wipe on pictures instead of words (the Web applies it to text only). @keyframes textClip-left:
// 0% polygon collapsed on the right edge → 20% the full box → 80% full → 100% collapsed on the left edge; -right / -top /
// -bottom are the same law with another edge. One Clip box per word; the pinned edge is the box's own edge, the free
// edge is the keyed Width (or Height) plus the same key on Position and on the text so the far edge stays put.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#091E2D" });
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/";
const SIZE = 150, W = 1000, H = SIZE * 1.2, LEN = 2.0;
const wipe = (word, x, y, side, at) => {
  const t = media(`${MAT}photo${1 + (word.length % 3)}.png`, { name: word }).set("Scale", [W / 1920, W / 1920]);
  const box = group(t).name(`${word} clip`);
  box.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H)
    .set("Overflow", "Clip").set("Position", [x, y]);
  t.set("Position", [0, 0]);
  const axis = side === "left" || side === "right" ? "Width" : "Height", full = axis === "Width" ? W : H;
  const [k0, k1, k2, k3] = [at, at + LEN * 0.2, at + LEN * 0.8, at + LEN];
  // Growing from the far edge (right / bottom): the box and the text are keyed together so that edge stays still.
  const grow = side === "right" || side === "bottom" ? 1 : 0, shrink = 1 - grow;
  const shift = (amount) => axis === "Width" ? [x + amount, y] : [x, y + amount];
  const inner = (amount) => axis === "Width" ? [-amount, 0] : [0, -amount];
  box.keys(axis, [[k0, 0, "Linear"], [k1, full, "Hold"], [k2, full, "Linear"], [k3, 0]]);
  box.keys("Position", [[k0, shift(full * shrink), "Linear"], [k1, shift(0), "Hold"], [k2, shift(0), "Linear"], [k3, shift(full * grow)]]);
  t.keys("Position", [[k0, inner(full * shrink), "Linear"], [k1, inner(0), "Hold"], [k2, inner(0), "Linear"], [k3, inner(full * grow)]]);
};
wipe("LEFT", 160, 140, "left", 0.2);
wipe("RIGHT", 760, 340, "right", 0.6);
wipe("TOP", 160, 540, "top", 1.0);
wipe("BOTTOM", 760, 740, "bottom", 1.4);
