// Tome Hame. 144 things at one point, one block with four dials, excessive filters on the whole stage. No keys.
comp({ width: 1920, height: 1080, fps: 30, seconds: 10, background: "#0A0A0C" });
const INK = "#F0ECE3", RED = "#FF3B1F", ACID = "#C8FF00";
const words = ("TOME HAME STOP SNAP HOLD HIT BEAT BAND JET SET RADIO MOTLEY CAVALRY WGSL 140BPM").split(" ");
const things = [];
for (let k = 0; k < 144; k++) {
  const kind = k % 3, ink = k % 7 === 0 ? RED : k % 4 === 0 ? ACID : INK;
  let t;
  if (kind === 0) t = text(words[k % words.length], { name: `W${k}` }).fill(ink).font("Helvetica Neue").set("Scale", [0.3, 0.3]);
  else if (kind === 1) t = rectangle({ name: `B${k}` }).fill(ink).set("Scale", [(50 + (k % 4) * 30) / 270, 10 / 270]);
  else t = ellipse({ name: `R${k}` }).fill(ink).set("Scale", [(18 + (k % 5) * 6) / 270, (18 + (k % 5) * 6) / 270]);
  t.set("Position", [0, 0]);
  things.push(t);
}
const stage = group(...things).name("Stage");
stage.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1920).set("Height", 1080).set("Position", [0, 0]);
things.forEach((t) => t.set("Position Type", "Absolute").set("Position", [960, 540]));
things.forEach((t) => t.effect("Tome Hame", { "BPM": 140, "Hold": 0.62, "Snap": 0.4, "Wander": 70, "Cell": 100, "Radius": 400 }));
stage.effect("Radiance", { "Threshold": 0.5, "Intensity": 3.0, "Reach": 70, "Air": 0.6 }).whole(true);
stage.effect("RGB Trail", { "Reach": 60 }).whole(true);
stage.effect("Overkill", { "Curve": 0.14, "Split": 0.014, "Scanlines": 0.35, "Ripple": 0.01, "Grain": 0.12, "Vignette": 0.7 }).whole(true);
text("TOME HAME · 144 things · one block · four dials · stop, then snap", { Position: [960, 1046] }).fill(INK).font("Helvetica Neue").set("Scale", [0.18, 0.18]);
