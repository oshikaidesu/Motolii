// The beat is the clock. 120 bpm: every thing lands on the same instant, size and opacity together,
// and the wave's length is 1 so nothing is staggered — being in time is the picture. Two hands: the grid, the beat.
comp({ width: 1080, height: 1080, fps: 30, seconds: 4, background: "#F0ECE3" });
const D = 270, C = ["#14141A", "#E8442E", "#2B4C8C"];
const things = [];
for (let k = 0; k < 25; k++) {
  const col = k % 5, row = Math.floor(k / 5);
  things.push(rectangle({ name: `B${k}`, Position: [220 + col * 160, 220 + row * 160] })
    .fill(C[(col + row) % C.length]).set("Scale", [110 / D, 110 / D]));
}
// 120 bpm = 2 per second. Wavelength 1: every k has the same phase.
things.forEach((t) => t.effect("Wave", { "Amplitude": 0, "Frequency": 2, "Wavelength": 1, "Scale": 0.35, "Opacity": 0.6 }));
text("120 bpm · one wave · wavelength 1 · everything on the beat", { Position: [540, 1010] }).fill("#14141A").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
