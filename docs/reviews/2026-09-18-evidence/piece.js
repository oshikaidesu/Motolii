// One material (the day's words), three inks, two laws stacked on the same things: Formations decides where each word
// stands and walks the shapes; Zero Gravity adds a slow float on top. Ropes tie every fourth pair. No keys.
comp({ width: 1920, height: 1080, fps: 30, seconds: 16, background: "#0B0F12" });
const INK = "#F0ECE3", RED = "#E8442E", TEAL = "#7FE7D8";
const words = ("motley patchwork collage jet set radio flipnote aviutl flash processing three cavalry after effects one hand many things relations rope zero gravity formation table circle helix grid stagger falloff noise oscillator modulate duplicator shelf text law function memory solver time index thing box field wave arrive hang bounce push apart concentrick mazin ring ting reveal optical art").split(" ");
const things = words.map((w, k) => text(w, { name: `W${k}`, Position: [960, 540] }).fill(k % 7 === 0 ? RED : k % 5 === 0 ? TEAL : INK).font("Helvetica Neue").set("Scale", [0.42, 0.42]));
things.forEach((t) => t.effect("Formations", { "Hold": 2.4, "Travel": 1.3, "Cell": 150, "Radius": 400, "Stagger": 0.006 }));
things.forEach((t) => t.effect("Zero Gravity", { "Drift": 18, "Tumble": 4, "Breath": 0.05, "Slowness": 9 }));
for (let k = 0; k + 1 < things.length; k += 4) {
  line({ name: `Rope ${k}` }).fill(INK).set("Connect From", things[k]).set("Connect To", things[k + 1])
    .set("Line Path", "Rope").set("Slack", 12).set("Stroke Width", 1.2).set("Dash", 3).set("Dash Gap", 5);
}
text("one material · three inks · two laws stacked · no keys", { Position: [960, 1040] }).fill(INK).font("Helvetica Neue").set("Scale", [0.2, 0.2]);
