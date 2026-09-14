// A six-second title: dots rain in, a ring turns, the word lands with a spring and glows.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#0b0b12" });

const rand = random(7);
const palette = ["#ff4d6d", "#ffd166", "#06d6a0", "#4cc9f0"];

// Dots: made in a loop, each with its own delay — the loop runs once, the keys do the moving.
for (let i = 0; i < 24; i++) {
  const x = 160 + rand() * 1600;
  const y = 120 + rand() * 840;
  const delay = rand() * 0.8;
  ellipse({ name: `Dot ${i + 1}` })
    .fill(palette[i % palette.length])
    .key("Position", delay, [x, -80], "Bezier")
    .key("Position", delay + 0.9, [x, y], "Bounce")
    .key("Scale", 0, [0.12, 0.12])
    .key("Opacity", 4.2, 1, "Bezier")
    .key("Opacity", 5, 0);
}

// A ring of squares that turns behind the word.
const ring = nullLayer({ name: "Ring", Position: [960, 540] })
  .key("Rotation", 0, -90, "Bezier")
  .key("Rotation", 6, 270);
for (let i = 0; i < 12; i++) {
  const angle = (i / 12) * Math.PI * 2;
  rectangle({ name: `Tile ${i + 1}` })
    .fill(palette[(i + 1) % palette.length])
    .parent(ring)
    .set("Position", [Math.cos(angle) * 360, Math.sin(angle) * 360])
    .set("Rotation", (angle * 180) / Math.PI)
    .key("Scale", 0.6 + i * 0.05, [0, 0], "Elastic")
    .key("Scale", 1.4 + i * 0.05, [0.08, 0.08]);
}

// The word.
const title = text("MOTOLII", { name: "Title", Position: [960, 540] })
  .fill("#ffffff")
  .key("Scale", 1.2, [0, 0], "Elastic")
  .key("Scale", 2.2, [1, 1])
  .key("Tracking", 1.2, 400, "Bezier")
  .key("Tracking", 2.4, 40);
title.effect("Glow");
