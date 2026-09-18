// Concentrick (Cavalry example), packaged: 24 rings traced round one hidden point, one block steps and breathes them.
comp({ width: 1080, height: 1080, fps: 30, seconds: 8, background: "#0B0F12" });
const hub = ellipse({ name: "Hub", Position: [540, 540] }).fill("#0B0F12").set("Scale", [4 / 270, 4 / 270]);
const rings = [];
for (let k = 0; k < 24; k++) rings.push(line({ name: `Ring ${k}` }).fill("#7FE7D8").set("Connect From", hub).set("Trace", "Circle").set("Margin", 24).set("Stroke Width", 1.2));
rings.forEach((r) => r.effect("Concentrick", { "Step": 0.2, "Breath": 0.04, "Frequency": 0.45, "Stagger": 0.09, "Fade": 0.7 }));
text("concentrick · one block · stagger radius · oscillator breath", { Position: [540, 1040] }).fill("#F0ECE3").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
