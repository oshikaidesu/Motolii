const golden = Math.PI * (3 - Math.sqrt(5));
function each(i, n, t, k) {
  const r = k.spread * Math.sqrt((i + 0.5) / n), a = i * golden + t * 0.6;
  const [x, y] = [r * Math.cos(a), r * Math.sin(a)];
  return { x, y, size: k.size * (1 + 0.5 * Math.sin(t * 3 - r * 4)) * (0.4 + r), rotate: a };
}
const k = { spread: 2.2, size: 0.04 };
const clock = typeof performance !== 'undefined' ? () => performance.now() : () => Date.now();
let sum = 0, worst = 0, sink = 0;
for (let p = 0; p < 120; p++) {
  const a = clock();
  for (let i = 0; i < 2000; i++) sink += each(i, 2000, p / 60, k).x;
  const d = clock() - a;
  if (p >= 60) { sum += d; worst = Math.max(worst, d); }
}
print(`${(sum / 60).toFixed(3)} ms a picture (worst ${worst.toFixed(3)}) for 2000 copies; ${sink.toFixed(1)}`);
