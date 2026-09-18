// Optical Art (Cavalry example), packaged: 36 bars in a column, one block ripples them.
comp({ width: 1080, height: 1080, fps: 30, seconds: 6, background: "#F0ECE3" });
const bars = [];
for (let k = 0; k < 36; k++) bars.push(rectangle({ name: `Bar ${k}`, Position: [540, 120 + k * 24] }).fill(k % 2 ? "#14141A" : "#E8442E").set("Scale", [720 / 270, 12 / 270]));
bars.forEach((b) => b.effect("Optical Art", { "Swing": 70, "Frequency": 0.4, "Stagger": 0.06, "Divisor": 2, "Waveform": "Sine" }));
text("optical art · one block · stagger · modulate · oscillator", { Position: [540, 1040] }).fill("#14141A").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
