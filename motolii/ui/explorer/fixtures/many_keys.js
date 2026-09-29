// Layers carrying many keys, on several properties.
comp({ width: 1920, height: 1080, fps: 30, seconds: 10, background: "#101114" });
for (let i = 0; i < 6; i++) {
  const l = rectangle().name(`Keyed ${i + 1}`);
  const keys = [];
  for (let k = 0; k < 14 + i * 3; k++) keys.push([k * 0.33, [200 + ((k * 97 + i * 31) % 1500), 200 + ((k * 53) % 700)], "Bezier"]);
  l.keys("Position", keys);
  l.keys("Opacity", [[0, 0], [0.5, 1], [3, 1], [3.5, 0.2], [6, 1]]);
  if (i % 2 === 0) l.keys("Rotation", [[0, 0, "Bezier"], [2, 90, "Bezier"], [4, 180, "Bezier"], [8, 360]]);
}
