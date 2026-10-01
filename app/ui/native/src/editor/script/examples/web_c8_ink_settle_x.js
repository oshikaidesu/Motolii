// Codrops #8 化合 — the same ink law on a video clip under the words (the Web filters text only; here the moving picture bleeds and sets too).
// OnScrollSVGFilterText (js/filter2.js, index.html #goo-2): a heading bleeds like ink and warps, then comes into focus.
// Source: <filter> = feGaussianBlur (stdDeviation) → feColorMatrix (alpha ×12 −4 = the gooey threshold) → feTurbulence (baseFrequency)
// → feDisplacementMap (scale) → feComposite atop; gsap tweens {stdDeviation: 20 → 0, scale: 100 → 0, baseFrequency: 0.1 → 0.05} and
// opacity 0 → 1, duration 2, ease 'expo', on scroll enter. Here: Blur (Radius) + Turbulent Displace (Amount, Size = 1 / baseFrequency,
// Direction XY) + Opacity, all expo over 2 s; the three lines enter one after another (scroll = time). The gooey colour matrix is not here.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#F2EFE8" });
const EXPO = "expo", SIZE = 210, DUR = 2;
const clip = media("/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/clip3.mp4", { name: "Clip" }).set("Position", [960, 540]).set("Opacity", 0.35);
clip.effect("Blur", { "Radius": 20 }).key("Radius", 0.2, 20, EXPO).key("Radius", 2.2, 0);
clip.effect("Turbulent Displace", { "Amount": 100, "Size": 10, "Complexity": 1, "Direction": "XY", "Seed": 2 }).key("Amount", 0.2, 100, EXPO).key("Amount", 2.2, 0);
["INK BLEEDS", "THEN THE", "WORDS SET"].forEach((line, i) => {
  const at = 0.2 + i * 0.7;
  const t = text(line, { name: line }).fill("#14121A").font("Helvetica Neue").set("Size", SIZE).set("Position", [960, 260 + i * 280]);
  t.key("Opacity", at, 0, EXPO).key("Opacity", at + DUR, 1);
  t.effect("Blur", { "Radius": 20 }).key("Radius", at, 20, EXPO).key("Radius", at + DUR, 0);
  t.effect("Turbulent Displace", { "Amount": 100, "Size": 10, "Complexity": 1, "Direction": "XY", "Seed": 2 })
    .key("Amount", at, 100, EXPO).key("Amount", at + DUR, 0).key("Size", at, 10, EXPO).key("Size", at + DUR, 20);
});
