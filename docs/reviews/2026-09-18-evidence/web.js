// Twelve things on a ring, one in the middle, and the lines that say who is next to whom and who belongs to the centre.
// One wave moves the ring; the lines are not keyed — they follow the relation.
comp({ width: 1080, height: 1080, fps: 30, seconds: 4, background: "#14141A" });
const D = 270, C = ["#E8442E", "#E9C46A", "#F0ECE3", "#2B4C8C"], LINE = "#F0ECE3";
const N = 12, things = [];
for (let k = 0; k < N; k++) {
  const a = (k / N) * Math.PI * 2;
  things.push(ellipse({ name: `T${k}`, Position: [540 + Math.cos(a) * 340, 540 + Math.sin(a) * 340] })
    .fill(C[k % C.length]).set("Scale", [64 / D, 64 / D]));
}
const hub = ellipse({ name: "Hub", Position: [540, 540] }).fill(LINE).set("Scale", [40 / D, 40 / D]);
// Relations drawn as lines: neighbours around the ring (curved), and each thing to the hub (straight, thin, dashed).
for (let k = 0; k < N; k++) {
  line({ name: `Next ${k}` }).fill(LINE).set("Connect From", things[k]).set("Connect To", things[(k + 1) % N])
    .set("Line Path", "Curved").set("Stroke Width", 3).set("Margin", 6);
  line({ name: `Spoke ${k}` }).fill(LINE).set("Connect From", things[k]).set("Connect To", hub)
    .set("Stroke Width", 1.5).set("Dash", 6).set("Dash Gap", 8).set("Margin", 6);
}
// The only hand on the motion.
things.forEach((t) => t.effect("Wave", { "Amplitude": 60, "Frequency": 0.5, "Wavelength": 4, "Scale": 0.4, "Opacity": 0.5 }));
text("12 things · 24 relations · one wave", { Position: [540, 1010] }).fill(LINE).font("Helvetica Neue").set("Scale", [0.22, 0.22]);
