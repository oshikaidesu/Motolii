// Mazin (Cavalry example), packaged: 20 × 20 short strokes at 45°, one block rewrites the maze where a band sweeps.
comp({ width: 1080, height: 1080, fps: 30, seconds: 9, background: "#F0ECE3" });
const strokes = [];
for (let k = 0; k < 400; k++) strokes.push(rectangle({ name: `S${k}`, Position: [90 + (k % 20) * 47.4, 90 + Math.floor(k / 20) * 47.4], Rotation: 45 }).fill("#14141A").set("Scale", [56 / 270, 5 / 270]));
strokes.forEach((s) => s.effect("Mazin", { "Sweep": 4, "Band": 240, "Seed": 7, "Ease": "In Out" }));
text("mazin · one block · 10 PRINT · a band sweeps and the maze rewrites", { Position: [540, 1040] }).fill("#14141A").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
