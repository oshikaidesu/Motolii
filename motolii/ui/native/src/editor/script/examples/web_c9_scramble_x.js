// Codrops #9 化合 — the same scramble, but the random glyphs spark out of the line toward the lens (Position Z −220, Tilt Y 35, 3D) while the letters stay flat.
// LineTextHoverAnimations (js/effect-1/text-animator.js): on hover every character is replaced by random letters and symbols
// a few times, then comes back. Source: SplitType 'words, chars'; per char gsap.fromTo(char, {opacity: 0}, {opacity: 1, duration: 0.03,
// repeat: 3, repeatDelay: 0.04, repeatRefresh: true, delay: (position + 1) · 0.07, innerHTML: () => lettersAndSymbols[random]});
// onComplete: set innerHTML back to the original after 0.03. The vocabulary has no key on a glyph's content, so every char is a
// stack of layers — the letter and 4 random glyphs — whose Opacity is switched with Holds (the discrete part); the letter then fades
// back in continuously (the settle). The random stream is seeded (random(seed)), so the picture is a pure function of time.
comp({ width: 1920, height: 1080, fps: 30, seconds: 3, background: "#0C0C0E" });
const SYMBOLS = [..."abcdefghijklmnopqrstuvwxyz!@#$%^&*-_+=;:<>,"], SIZE = 120, STEP = SIZE * 0.6, FADE = 0.03, HOLD = 0.04, REPEAT = 3;
const rnd = random(7);
const line = (s, y, hover) => [...s].forEach((ch, p) => {
  if (ch === " ") return;
  const x = 960 - (s.length - 1) * STEP / 2 + p * STEP, at = hover + (p + 1) * 0.07, back = at + (REPEAT + 1) * (FADE + HOLD);
  const glyph = (g, name) => text(g, { name }).fill("#F1EDE4").font("Menlo").set("Size", SIZE).set("Position", [x, y]);
  // The letter: away while the symbols blink, then back — held off (discrete), fading in (continuous settle).
  glyph(ch, `${ch} ${p}`).keys("Opacity", [[0, 1, "Hold"], [at, 0, "Hold"], [back, 0, "Linear"], [back + 0.1, 1]]);
  for (let k = 0; k <= REPEAT; k++) {
    const t0 = at + k * (FADE + HOLD), g = SYMBOLS[Math.floor(rnd() * SYMBOLS.length)];
    glyph(g, `${ch} ${p} blink ${k}`).keys("Opacity", [[0, 0, "Hold"], [t0, 1, "Hold"], [t0 + FADE + HOLD, 0, "Hold"]])
      .fill("#F2B134").set("Position Z", -220).set("Tilt Y", 35).projection("3D");
  }
});
line("HOVER THE LINE", 400, 0.3);
line("LETTERS SCATTER", 620, 1.5);
