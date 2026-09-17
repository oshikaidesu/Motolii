// Codrops #2 化合 — the same marker band and letter pop, but the page stands in 2.5D (a null parent at the centre with Tilt Y / Tilt X, 3D);
// the letter rows sit 3 px nearer than their band so the band stays under them.
// Codrops #2 — OnScrollTextHighlight (js/effect-5/highlightEffect.js, css/base.css .hx-5): a marker band grows under a word, the letters pop in.
// Source: .hx-5::after{left:-2.5%;top:10%;bottom:-7.5%;width:105%;transform:scale3D(var(--after-scale)…);background:#6a5ace;border-radius:8px}
// gsap.timeline({defaults:{duration:0.4, ease:'power1'}}).fromTo(chars, {scale:1.3, opacity:0}, {stagger: pos => 0.1+0.05*pos, scale:1, opacity:1})
// .fromTo(el, {'--after-scale':0}, {duration:0.8, ease:'expo', '--after-scale':1}, 0). onEnter = the moment the line is reached (time).
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#1a1720" });
const P1 = { kind: "Bezier", x1: 0.25, y1: 0.46, x2: 0.45, y2: 0.94 }, EXPO = { kind: "Bezier", x1: 0.19, y1: 1, x2: 0.22, y2: 1 };
const SIZE = 120, W = SIZE * 0.68, D = 270, BAND = "#6a5ace", HI = "#e1def4", INK = "#7a7580";
const made = [], page = nullLayer({ name: "Page" }).set("Position", [960, 540]).set("Tilt Y", 24).set("Tilt X", 6);
const line = (y, words, at) => {
  let x = -800;
  for (const [word, hot] of words) {
    const w = word.length * W;
    if (!hot) { made.push(text(word, { name: word }).fill(INK).font("Helvetica Neue").set("Size", SIZE).set("Position", [x + w / 2, y])); x += w + W; continue; }
    // ::after — under the letters (made first), scaled from its centre with expo over 0.8 s.
    const band = rectangle({ name: `${word} band` }).fill(BAND).set("Position", [x + w / 2, y + SIZE * 0.09]);
    band.effect("Rounded Corners", { "Radius": 8 });
    band.key("Scale", at, [0, 0], EXPO).key("Scale", at + 0.8, [w * 1.05 / D, SIZE * 0.975 / D]);
    // .char — scale 1.3 → 1 and opacity 0 → 1, 0.4 s power1, the row hands out 0.1 + 0.05·pos.
    const chars = [...word].map((ch, k) => {
      const t = text(ch, { name: `${word} ${ch}${k}` }).fill(HI).font("Helvetica Neue").set("Size", SIZE);
      return t.key("Scale", at + 0.1, [1.3, 1.3], P1).key("Scale", at + 0.5, [1, 1]).key("Opacity", at + 0.1, 0, P1).key("Opacity", at + 0.5, 1);
    });
    const row = group(...chars).name(`${word} row`);
    row.set("Display", "Flex").set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Stagger", 0.05 * (chars.length - 1)).set("Position", [x, y - SIZE * 0.6]);
    chars.forEach((c) => c.set("Position", [0, 0]));
    made.push(band, row.set("Position Z", -3));
    x += w + W;
  }
};
line(-180, [["MARK", true], ["WHAT", false], ["MATTERS", true]], 0.4); line(80, [["THEN", false], ["READ", true], ["ON", false]], 1.8);
for (const layer of [page, ...made]) layer.parent(layer === page ? null : page).projection("3D");
