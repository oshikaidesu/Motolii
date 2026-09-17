// SANKOU! #3 — companycoc.com: the key visual wipes from the left. The Web clips the slide on top away
// (inset(0 0% 0 0) → inset(0 100% 0 0), 800ms `ease` = cubic-bezier(.25,.1,.25,1)) to show the next one underneath;
// the same seam is made here by the next slide's box opening from the left over the current one. Auto-advance every 2.0s.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#111111" });
const EASE = { kind: "Bezier", x1: 0.25, y1: 0.1, x2: 0.25, y2: 1 };
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/";
const slides = ["photo1.png", "photo2.png", "photo3.png"];
const D = 0.8, EVERY = 2.0;
slides.forEach((file, i) => {
  const pic = media(MAT + file, { name: `KV ${i}` });
  const slide = group(pic).name(`Slide ${i}`);
  slide.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1920).set("Height", 1080)
    .set("Overflow", "Clip").set("Position", [0, 0]);
  // The picture keeps to the left edge of its box; only the box's right side moves.
  pic.set("Position", [0, 0]).set("Scale", [1 + i * 0.01, 1 + i * 0.01]);
  for (const layer of [pic, slide]) layer.projection("2D");
  if (i > 0) slide.key("Width", 0.6 + (i - 1) * EVERY, 0, EASE).key("Width", 0.6 + (i - 1) * EVERY + D, 1920);
});
