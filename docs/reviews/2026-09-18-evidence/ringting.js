// Ring Ting (Cavalry example), packaged: 72 dots at one point, one block puts them on a ring and lets a noise spot walk round.
comp({ width: 1080, height: 1080, fps: 30, seconds: 10, background: "#14141A" });
const dots = [];
for (let k = 0; k < 72; k++) dots.push(ellipse({ name: `Dot ${k}`, Position: [540, 540] }).fill(k % 6 ? "#F0ECE3" : "#E9C46A").set("Scale", [14 / 270, 14 / 270]));
dots.forEach((d) => d.effect("Ring Ting", { "Radius": 360, "Amount": 110, "Frequency": 0.006, "Spot": 0.3, "Orbit": 8, "Ease": "In Out" }));
text("ring ting · one block · circle · noise · a falloff spot walks round", { Position: [540, 1040] }).fill("#F0ECE3").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
