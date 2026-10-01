// SANKOU! #9 化合 — the sticky picture is a video clip (the Web has a still); the same one value darkens a moving picture.
// musu-sauna.com: the sticky picture darkens as you scroll. Source: .p-tsunagaru__mvImg{--scrub:0;position:sticky}
// :before{background-color:#00000080;opacity:var(--scrub)}; gsap.fromTo(n,{"--scrub":0},{"--scrub":1,ease:"none",
// scrollTrigger:{start:"top 80%",end:"top 20%",scrub:true}}). Scroll is time: the page moves up at a steady 400 px/s, the
// picture stays (sticky), and one value 0 → 1 (linear, `ease:"none"`) is the overlay's opacity while the trigger's top crosses
// from 80% to 20% of the viewport. The overlay is a black box at 50% (#00000080) times the value.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#FFFFFF" });
const SPEED = 400, TOP0 = 1400, D = 270;
media("/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/clip3.mp4", { name: "MV (sticky)" });
const t0 = (TOP0 - 0.8 * 1080) / SPEED, t1 = (TOP0 - 0.2 * 1080) / SPEED;
rectangle({ name: "Scrub overlay" }).fill("#000000").set("Anchor", [0, 0]).set("Position", [0, 0]).set("Scale", [1920 / D, 1080 / D])
  .set("Opacity", 0).key("Opacity", t0, 0, "Linear").key("Opacity", t1, 0.5);
// The scrolling page over it: a heading (the trigger) and a paragraph.
const page = [["TSUNAGARU", 160, 0], ["Sauna, river, forest —", 64, 220], ["everything is connected here.", 64, 300]].map(([s, size, dy]) =>
  text(s, { name: s }).fill("#FFFFFF").font("Helvetica Neue").set("Size", size).set("Position", [960, TOP0 + dy + size / 2]));
page.forEach((l) => { const [x, y] = l.property("Position").value; l.key("Position", 0, [x, y], "Linear").key("Position", 4, [x, y - SPEED * 4]); });
