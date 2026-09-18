// Zero gravity as a law on the shelf: one block on twelve words, ropes without slack between neighbours.
// No room, no solver. The block is one WGSL file; this script is the only other hand.
comp({ width: 1080, height: 1080, fps: 30, seconds: 8, background: "#14141A" });
const INK = "#F0ECE3", RED = "#E8442E", BLUE = "#6C8CFF", SAND = "#E9C46A";
const words = ["drift", "no gravity", "tumble", "touch", "one law", "twelve words", "eleven ropes", "zero g", "float", "breathe", "slow", "return"];
const C = [INK, RED, SAND, BLUE];
const things = words.map((w, k) => {
  const a = (k / words.length) * Math.PI * 2, r = 300 + (k % 3) * 60;
  return text(w, { name: `W${k}`, Position: [540 + Math.cos(a) * r, 520 + Math.sin(a) * r] }).fill(C[k % C.length]).font("Helvetica Neue").set("Scale", [0.4, 0.4]);
});
things.forEach((t) => t.effect("Zero Gravity", { "Drift": 70, "Tumble": 14, "Breath": 0.08, "Slowness": 8 }));
for (let k = 0; k < things.length; k++) {
  line({ name: `Rope ${k}` }).fill(INK).set("Connect From", things[k]).set("Connect To", things[(k + 1) % things.length])
    .set("Line Path", "Rope").set("Slack", 0).set("Stroke Width", 1.5).set("Dash", 4).set("Dash Gap", 6);
}
text("zero gravity · one block on the shelf · ropes without slack", { Position: [540, 1046] }).fill(INK).font("Helvetica Neue").set("Scale", [0.2, 0.2]);
