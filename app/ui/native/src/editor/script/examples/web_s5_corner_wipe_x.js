// SANKOU! #5 化合 — the same four-edge wipe on pictures instead of words (the Web applies it to text only). @keyframes textClip-left:
// 0% polygon collapsed on the right edge → 20% the full box → 80% full → 100% collapsed on the left edge; -right / -top /
// -bottom are the same law with another edge. One box per word, clip-path: inset(): the near edge closes first
// (Clip Left W → 0 for -left), then the far edge closes (Clip Right 0 → W). The times are not in the CSS; 2.0 s here.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#091E2D" });
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/";
const SIZE = 150, W = 1000, H = SIZE * 1.2, LEN = 2.0;
const OPPOSITE = { left: "right", right: "left", top: "bottom", bottom: "top" };
const edge = (side) => `Clip ${side[0].toUpperCase()}${side.slice(1)}`;
const wipe = (word, x, y, side, at) => {
  const t = media(`${MAT}photo${1 + (word.length % 3)}.png`, { name: word }).set("Scale", [W / 1920, W / 1920]);
  const box = group(t).name(`${word} clip`);
  box.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H).set("Overflow", "Clip").set("Position", [x, y]);
  t.set("Position", [0, 0]);
  const full = side === "left" || side === "right" ? W : H;
  const [k0, k1, k2, k3] = [at, at + LEN * 0.2, at + LEN * 0.8, at + LEN];
  box.keys(edge(side), [[k0, full, "Linear"], [k1, 0]]);
  box.keys(edge(OPPOSITE[side]), [[k2, 0, "Linear"], [k3, full]]);
};
wipe("LEFT", 160, 140, "left", 0.2);
wipe("RIGHT", 760, 340, "right", 0.6);
wipe("TOP", 160, 540, "top", 1.0);
wipe("BOTTOM", 760, 740, "bottom", 1.4);
