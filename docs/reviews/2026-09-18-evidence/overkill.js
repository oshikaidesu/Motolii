// Overkill. Overload (block side) inside a depth stack that tilts, extruded bars and glass rings (3D), then on the whole
// stage: Turbulent Displace (field), Radiance (light), RGB Trail (memory pass), Overkill (pixel side). No keys but the tilt.
comp({ width: 1920, height: 1080, fps: 30, seconds: 15, background: "#07080A" });
const INK = "#F0ECE3", RED = "#FF3B1F", ACID = "#C8FF00";
const words = ("MOTLEY PATCHWORK JETSET FLIPNOTE AVIUTL FLASH PROCESSING CAVALRY OVERLOAD MAXIMAL TOO MUCH ON PURPOSE INDEX TIME LAW SHELF BEAT BAND DRIFT TUMBLE STAND GRID CIRCLE SPIRAL LINE").split(" ");
const things = [];
for (let k = 0; k < 180; k++) {
  const kind = k % 3, ink = k % 7 === 0 ? RED : k % 5 === 0 ? ACID : INK;
  let t;
  if (kind === 0) t = text(words[k % words.length], { name: `W${k}` }).fill(ink).font("Helvetica Neue").set("Scale", [0.28, 0.28]);
  else if (kind === 1) { t = rectangle({ name: `B${k}` }).fill(ink).set("Scale", [(40 + (k % 4) * 30) / 270, 8 / 270]); t.effect("Extrude", { "Depth": 26 }); }
  else { t = ellipse({ name: `R${k}` }).fill(ink).set("Scale", [(16 + (k % 5) * 6) / 270, (16 + (k % 5) * 6) / 270]); t.effect("Glass", { "Refraction": 1.6, "Dispersion": 0.8, "Roughness": 0.08 }); }
  t.set("Position", [0, 0]);
  things.push(t);
}
const stage = group(...things).name("Stage");
stage.set("Display", "Flex").set("Flex Direction", "Depth").set("Align Items", "Center").set("Justify Content", "Center")
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1920).set("Height", 1080).set("Position", [0, 0]);
things.forEach((t) => t.set("Position Type", "Absolute").set("Position", [960, 540]));
things.forEach((t) => t.effect("Overload", { "BPM": 128, "Kick": 0.4, "Hold": 1.875, "Cell": 96, "Radius": 430, "Drift": 26, "Tumble": 40, "Sweep": 3.75 }));
stage.keys("Gap", [[0, 4, "Bezier"], [7, 40, "Bezier"], [15, 6]]);
stage.keys("Tilt Y", [[0, -18, "Bezier"], [7.5, 22, "Bezier"], [15, -18]]);
stage.keys("Tilt X", [[0, 6, "Bezier"], [15, -10]]);
stage.effect("Turbulent Displace", { "Amount": 18, "Size": 260, "Complexity": 3, "Evolution": 0 }).whole(true);
stage.effect("Radiance", { "Threshold": 0.55, "Intensity": 3.5, "Reach": 90, "Air": 0.8 }).whole(true);
stage.effect("RGB Trail", { "Reach": 120 }).whole(true);
stage.effect("Overkill", { "Curve": 0.16, "Split": 0.012, "Scanlines": 0.3, "Ripple": 0.015, "Grain": 0.1, "Vignette": 0.6 }).whole(true);
text("OVERKILL · overload × depth stack × extrude × glass × turbulence × radiance × rgb trail × overkill pass", { Position: [960, 1046] }).fill(INK).font("Helvetica Neue").set("Scale", [0.18, 0.18]);
