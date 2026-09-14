// A grid that ripples out from the middle — the kind of sketch you would write in a draw loop,
// written once: the loop places layers, the delay is part of each key's time.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#111111" });

const columns = 16;
const rows = 9;
const gap = 110;
const colors = ["#f72585", "#7209b7", "#3a0ca3", "#4361ee", "#4cc9f0"];

for (let row = 0; row < rows; row++) {
  for (let column = 0; column < columns; column++) {
    const x = 960 + (column - (columns - 1) / 2) * gap;
    const y = 540 + (row - (rows - 1) / 2) * gap;
    const distance = Math.hypot(column - (columns - 1) / 2, row - (rows - 1) / 2);
    const delay = distance * 0.08;
    roundedRectangle({ name: `Cell ${row}-${column}`, Position: [x, y] })
      .fill(colors[Math.floor(distance) % colors.length])
      .key("Scale", delay, [0, 0], "Elastic")
      .key("Scale", delay + 0.8, [0.1, 0.1])
      .key("Rotation", delay + 1.5, 0, "Bezier")
      .key("Rotation", delay + 2.3, 90)
      .key("Opacity", 3.8 + delay * 0.5, 1, "Bezier")
      .key("Opacity", 4.6 + delay * 0.5, 0);
  }
}

text("ripple", { Position: [960, 540] })
  .fill("#ffffff")
  .key("Opacity", 1.8, 0, "Bezier")
  .key("Opacity", 2.4, 1)
  .key("Tracking", 1.8, 200, "Bezier")
  .key("Tracking", 3, 20);
