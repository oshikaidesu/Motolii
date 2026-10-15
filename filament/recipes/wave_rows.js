// Wave rows: lines of one length one above another, each row waving more times across than the one above, flowing sideways
// make shape line
// say fill ffffff
// set copies 1000
// set tall 6
// say copy_fill ffd166 ff3366 3a86ff
// --track@rows:Rows,1,40,10,1
// --track@waves:Waves,0,20,1,0.5
// --track@more:More a row,0,10,1,0.5
// --track@high:Height,0,2,0.1,0.01
// --track@wide:Width,0.5,12,5.4,0.01
// --track@gap:Gap,0,2,0.26,0.01
// --track@speed:Speed,-4,4,0.5,0.01

// Each row the same length (Width) made of short pieces of line, each piece where the row's wave is at it and turned
// along it, so the pieces join into one curve; the first row waves Waves times across, each one after More times more.
function each(i, n, t, k) {
  const rows = Math.max(1, Math.round(k.rows)), per = Math.floor(n / rows);
  if (per < 2 || i >= rows * per) return { size: 0 };
  const r = Math.floor(i / per), j = i % per, step = k.wide / per;
  const waves = k.waves + r * k.more, w = (2 * Math.PI * waves) / k.wide, phase = t * k.speed * 2 * Math.PI;
  const x = (j + 0.5) * step - k.wide / 2;
  const y = k.high * Math.sin(w * x - phase), slope = k.high * w * Math.cos(w * x - phase);
  return { x: x, y: ((rows - 1) / 2 - r) * k.gap + y, size: step * Math.sqrt(1 + slope * slope) * 1.05, rotate: (Math.atan(slope) * 180) / Math.PI };
}
