// Codrops #6 化合 — the same lagging columns, but the middle row of every column is a video clip (the Web scrolls still pictures; the one law drags moving ones).
// ElasticGridScroll (js/demo1/index.js): the grid's columns follow the scroll late, the farther from the centre the later.
// Source: ScrollSmoother.create({smooth: 1, effects: true}); per column smoother.effects(column, {speed: 1, lag}) with
// lag = baseLag + |i − mid| · lagScale, baseLag = 0.5, lagScale = 0.1. A lag is a first-order follower (the previous frame's memory),
// so as in SANKOU! #8 it is written as the closed form of the scroll's keyed runs — a pure function of time — one key per frame per column.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#EDEAE3" });
const FPS = 30, SEC = 6, COLS = 5, ROWS = 4, CW = 340, CH = 420, GAP = 30, TOP = 80, mid = (COLS - 1) / 2;
const C = ["#E4572E", "#2E5EAA", "#F3B61F", "#534AB7", "#7FA37A", "#D9553A", "#1C1C1E"];
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/";
// The scroll: runs of the page's Y (a scroll down, a stop, further, then back up).
const K = [[0, 0], [1.0, -700], [1.6, -700], [2.8, -1300], [3.4, -1300], [4.6, -300], [SEC, -300]];
const g = (t) => { let i = 0; while (i + 2 < K.length && t >= K[i + 1][0]) i++; const [t0, p0] = K[i], [t1, p1] = K[i + 1];
  return p0 + (p1 - p0) * Math.min(1, (t - t0) / (t1 - t0)); };
const slope = (i) => (i < 0 || i + 1 >= K.length) ? 0 : (K[i + 1][1] - K[i][1]) / (K[i + 1][0] - K[i][0]);
const lag = (t, tau) => g(t) - tau * K.reduce((s, [tk], k) => tk <= t ? s + (slope(k) - slope(k - 1)) * (1 - Math.exp(-(t - tk) / tau)) : s, 0);
for (let i = 0; i < COLS; i++) {
  const cells = [];
  for (let r = 0; r < ROWS; r++) cells.push((r === 1 ? media(MAT + `clip${(i % 3) + 1}.mp4`, { name: `Clip ${i}` }) : rectangle({ name: `Pic ${i}.${r}` }).fill(C[(i * 3 + r * 5) % C.length])));
  const col = group(...cells).name(`Column ${i}`);
  col.set("Display", "Flex").set("Flex Direction", "Column").set("Gap", GAP).set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug");
  cells.forEach((c) => c.set("Position", [0, 0]).set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", CW).set("Height", CH));
  cells[1].set("Object Fit", "Contain");
  // lag = baseLag + distance · lagScale — the column's Y is the scroll seen through a lag of that many seconds.
  const tau = 0.5 + Math.abs(i - mid) * 0.1, x = 960 - (COLS * CW + (COLS - 1) * GAP) / 2 + i * (CW + GAP);
  for (let f = 0; f <= SEC * FPS; f++) col.key("Position", f / FPS, [x, TOP + lag(f / FPS, tau)], "Linear");
}
