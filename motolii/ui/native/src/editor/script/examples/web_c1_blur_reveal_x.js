// Codrops #1 化合 — the same blur(10px) brightness(0%) → clear, but on a video under the letters too (the Web only filters text; the one law clears a moving picture). ScrollBlurTypography (js/effect-1/blurScrollEffect.js): every character starts blurred and dark and clears one after another.
// Source: SplitType 'words, chars'; gsap.fromTo(chars, {filter:'blur(10px) brightness(0%)'}, {filter:'blur(0px) brightness(100%)', ease:'none',
// stagger: 0.05, scrollTrigger:{scrub:true, start:'top bottom-=15%', end:'bottom center+=15%'}}). Scroll scrub = time: the line's clearing
// takes 1.0 s per char (ease none), the line's own Stagger hands each char its 0.05 s delay (Stagger = 0.05 × (chars − 1)).
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#0E0E10" });
const SIZE = 132, EACH = 0.05, CLEAR = 1.0, INK = "#F1EDE4";
const lines = ["EVERY LETTER", "COMES OUT OF", "THE DARK"];
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/";
const clip = media(MAT + "clip2.mp4", { name: "Clip" }).set("Position", [960, 540]).set("Opacity", 0.55);
clip.effect("Blur", { "Radius": 10 }).key("Radius", 0.2, 10, "Linear").key("Radius", 2.2, 0);
clip.effect("Gain", { "Gain": 0 }).key("Gain", 0.2, 0, "Linear").key("Gain", 2.2, 1);
lines.forEach((line, row) => {
  const at = 0.2 + row * 0.5, chars = line.replace(/ /g, "").length;
  // One text layer per line, Split: Chars (SplitType 'chars'; a space is no unit).
  const t = text(line, { name: `Line ${row}` }).fill(INK).font("Helvetica Neue").set("Size", SIZE).set("Split", "Chars").set("Stagger", EACH * (chars - 1));
  // filter: blur(10px) brightness(0%) → blur(0) brightness(100%): one Blur and one Gain, both linear (ease none).
  t.effect("Blur", { "Radius": 10 }).key("Radius", at, 10, "Linear").key("Radius", at + CLEAR, 0);
  t.effect("Gain", { "Gain": 0 }).key("Gain", at, 0, "Linear").key("Gain", at + CLEAR, 1);
  const rowBox = group(t).name(`Row ${row}`);
  rowBox.set("Display", "Flex").set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Position", [160, 260 + row * (SIZE + 60)]);
  t.set("Position", [0, 0]);
});
