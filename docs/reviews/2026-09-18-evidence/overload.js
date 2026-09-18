// Overload. Three materials (words, bars, rings), three inks on black, 240 things at one point, one block that stacks
// every law, ropes on every eighth pair. No keys. Excess by multiplication.
comp({ width: 1920, height: 1080, fps: 30, seconds: 15, background: "#07080A" });
const INK = "#F0ECE3", RED = "#FF3B1F", ACID = "#C8FF00";
const words = ("MOTLEY PATCHWORK JETSET FLIPNOTE AVIUTL FLASH PROCESSING CAVALRY OVERLOAD MAXIMAL TOO MUCH ON PURPOSE INDEX TIME LAW SHELF BEAT BAND DRIFT TUMBLE STAND GRID CIRCLE SPIRAL LINE").split(" ");
const things = [];
for (let k = 0; k < 240; k++) {
  const kind = k % 3, ink = k % 7 === 0 ? RED : k % 5 === 0 ? ACID : INK;
  if (kind === 0) things.push(text(words[k % words.length], { name: `W${k}`, Position: [960, 540] }).fill(ink).font("Helvetica Neue").set("Scale", [0.26, 0.26]));
  else if (kind === 1) things.push(rectangle({ name: `B${k}`, Position: [960, 540] }).fill(ink).set("Scale", [(40 + (k % 4) * 30) / 270, 6 / 270]));
  else things.push(ellipse({ name: `R${k}`, Position: [960, 540] }).fill(ink).set("Scale", [(14 + (k % 5) * 6) / 270, (14 + (k % 5) * 6) / 270]));
}
things.forEach((t) => t.effect("Overload", { "BPM": 128, "Kick": 0.4, "Hold": 1.875, "Cell": 96, "Radius": 430, "Drift": 26, "Tumble": 40, "Sweep": 3.75 }));
for (let k = 0; k + 8 < things.length; k += 8) {
  line({ name: `Rope ${k}` }).fill(INK).set("Connect From", things[k]).set("Connect To", things[k + 8])
    .set("Line Path", "Rope").set("Slack", 10).set("Stroke Width", 1).set("Dash", 2).set("Dash Gap", 6);
}
text("OVERLOAD · 240 things · one block · every law multiplied · no keys", { Position: [960, 1046] }).fill(INK).font("Helvetica Neue").set("Scale", [0.2, 0.2]);
