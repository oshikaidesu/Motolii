// A ring of thirty-six. Two hands on the motion: they arrive from the centre (Arrive, Radial), then one wave breathes them.
comp({ width: 1080, height: 1080, fps: 30, seconds: 4, background: "#14141A" });
const D = 270, C = ["#E8442E", "#E9C46A", "#F0ECE3", "#2B4C8C"];
const things = [];
for (let k = 0; k < 36; k++) {
  const a = (k / 36) * Math.PI * 2;
  things.push(rectangle({ name: `R${k}`, Position: [540 + Math.cos(a) * 360, 540 + Math.sin(a) * 360] })
    .fill(C[k % C.length]).set("Scale", [56 / D, 56 / D]).set("Rotation", (k / 36) * 360));
}
things.forEach((t) => t.effect("Arrive", { "From": 0, "Distance": 300, "Arrive": 1.2, "Bounce": 0.3, "Stagger": 0.03, "Spin": 180, "Radial": 1 }));
things.forEach((t) => t.effect("Wave", { "Amplitude": 0, "Frequency": 0.5, "Wavelength": 9, "Scale": 0.5, "Opacity": 0.7 }));
text("36 things · arrive from the centre · one wave", { Position: [540, 990] }).fill("#F0ECE3").font("Helvetica Neue").set("Scale", [0.24, 0.24]);
