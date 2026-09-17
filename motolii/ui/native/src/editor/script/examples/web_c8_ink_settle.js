// Codrops #8 — OnScrollSVGFilterText (js/filter2.js, index.html #goo-2): a heading bleeds like ink and warps, then comes into focus.
// Source: <filter> = feGaussianBlur (stdDeviation) → feColorMatrix (alpha ×12 −4 = the gooey threshold) → feTurbulence (baseFrequency)
// → feDisplacementMap (scale) → feComposite atop; gsap tweens {stdDeviation: 20 → 0, scale: 100 → 0, baseFrequency: 0.1 → 0.05} and
// opacity 0 → 1, duration 2, ease 'expo', on scroll enter. Here: Blur (Radius) + Turbulent Displace (Amount, Size = 1 / baseFrequency,
// Direction XY) + Opacity, all expo over 2 s; the three lines enter one after another (scroll = time). The gooey colour matrix is not here.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#F2EFE8" });
const EXPO = "expo", SIZE = 210, DUR = 2;
["INK BLEEDS", "THEN THE", "WORDS SET"].forEach((line, i) => {
  const at = 0.2 + i * 0.7;
  const t = text(line, { name: line }).fill("#14121A").font("Helvetica Neue").set("Size", SIZE).set("Position", [960, 260 + i * 280]);
  t.key("Opacity", at, 0, EXPO).key("Opacity", at + DUR, 1);
  t.effect("Blur", { "Radius": 20 }).key("Radius", at, 20, EXPO).key("Radius", at + DUR, 0);
  t.effect("Turbulent Displace", { "Amount": 100, "Size": 10, "Complexity": 1, "Direction": "XY", "Seed": 2 })
    .key("Amount", at, 100, EXPO).key("Amount", at + DUR, 0).key("Size", at, 10, EXPO).key("Size", at + DUR, 20);
});
