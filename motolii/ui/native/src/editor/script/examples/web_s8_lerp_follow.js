// SANKOU! #8 — companycoc.com: a thumbnail follows the cursor, late (lerp). Source: targetX = (clientX - centre) * sr,
// currentX += (targetX - currentX) * ar, per requestAnimationFrame; sr = .05, or = .1, ar = .08 (at 60 fps: λ = 4.8/s, τ = 0.21 s).
// There is no cursor in a video: the cursor is a keyed Position (linear runs), and the follower is the lerp's closed form —
// the target minus τ · Σ (slope change at each corner) · (1 − e^(−(t − t_corner)/τ)) — a pure function of time, written
// out one key per frame. No memory. (sr / or are 1 here: at .05 the lag would be under 10 px and invisible.)
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#F4F4F2" });
const FPS = 30, SEC = 4, TAU = 1 / (0.08 * 60), D = 270;
["TOPICS 01 — New showroom opens", "TOPICS 02 — Summer collection", "TOPICS 03 — Interview: the makers"].forEach((s, i) =>
  text(s, { name: `News ${i}` }).fill("#1A1A1A").font("Helvetica Neue").set("Size", 64).set("Position", [700, 300 + i * 200]));
const K = [[0, [300, 300]], [1.2, [1500, 300]], [2.0, [1500, 700]], [3.2, [400, 700]], [SEC, [400, 700]]];
const g = (t) => { let i = 0; while (i + 2 < K.length && t >= K[i + 1][0]) i++; const [t0, p0] = K[i], [t1, p1] = K[i + 1];
  const u = Math.min(1, (t - t0) / (t1 - t0)); return [0, 1].map((a) => p0[a] + (p1[a] - p0[a]) * u); };
const slope = (i) => (i < 0 || i + 1 >= K.length) ? [0, 0] : [0, 1].map((a) => (K[i + 1][1][a] - K[i][1][a]) / (K[i + 1][0] - K[i][0]));
const lag = (t) => g(t).map((x, a) => x - TAU * K.reduce((s, [tk], k) => tk <= t ? s + (slope(k)[a] - slope(k - 1)[a]) * (1 - Math.exp(-(t - tk) / TAU)) : s, 0));
const thumb = media("/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/photo3.png", { name: "Thumb" })
  .set("Scale", [0.25, 0.25]);
for (let f = 0; f < SEC * FPS; f++) thumb.key("Position", f / FPS, lag(f / FPS), "Linear");
const cursor = ellipse({ name: "Cursor" }).fill("#E8442E").set("Scale", [22 / D, 22 / D]);
cursor.keys("Position", K.map(([t, p]) => [t, p, "Linear"]));
