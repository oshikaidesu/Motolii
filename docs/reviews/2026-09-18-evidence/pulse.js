// One wave, forty-eight things. Nothing is keyed: the wave breathes position, size and opacity together.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#F0ECE3" });
const D = 270, C = ["#E8442E", "#2B4C8C", "#E9C46A", "#14141A"];
const things = [];
for (let k = 0; k < 48; k++) {
  const col = k % 12, row = Math.floor(k / 12);
  things.push(ellipse({ name: `T${k}`, Position: [300 + col * 120, 360 + row * 120] })
    .fill(C[k % C.length]).set("Scale", [70 / D, 70 / D]));
}
// The only hand on the motion: one wave that runs along the things in order.
things.forEach((t) => t.effect("Wave", { "Amplitude": 40, "Frequency": 0.75, "Wavelength": 12, "Scale": 0.45, "Opacity": 0.85 }));
text("one wave · position · size · opacity", { Position: [960, 960] }).fill("#14141A").font("Helvetica Neue").set("Scale", [0.28, 0.28]);
