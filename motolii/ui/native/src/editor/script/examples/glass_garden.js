// GLASS GARDEN / LIVE 001 — a glass flower among a glass disc, a glass slab and an orbit of glass.
// Run it live: `scripts/motolii-ui.sh dev <this file>`; saving the file redraws the window.
//
// Everything is a relation, nothing is placed one by one:
//   light   — a studio light map with rainbow strips (an external image as the environment): every
//             glass surface reflects it, so the spectrum runs along the edges as they turn
//   water   — a dark caustic mesh gradient (WGSL on the shelf: Caustic Light) moving behind
//   flower  — one petal of dark glass, bent on the GPU every frame (Petal Wave) and repeated round
//             the centre by a Repeater, twice: an outer ring and an inner one. The Glow after each
//             Repeater reads its whole ring once
//   glass   — the disc, the slab and the orbiting spheres are clear glass (transmissive): they
//             refract, split (dispersion) and blur what is behind them
// The piece is one 8-second cycle: every motion closes on itself. The loop is asked for explicitly
// (`loop: true`) — playback never repeats unless a work says so.

const SECONDS = 8;
comp({ width: 1920, height: 1080, fps: 60, seconds: SECONDS, background: "#030306", loop: true });

// The knobs a live session turns.
const PETALS = 32;        // inner ring
const REFRACTION = 1.5;   // clear glass
const GLOW = 0.55;        // after each ring's Repeater
const WAVE = 10;          // Petal Wave, px

const cycle = (effect) => effect.keys("Phase", [[0, 0, "Linear"], [SECONDS, 1]]);
const breathe = (layer, property, a, b) =>
  layer.keys(property, [[0, a, "sine.inOut"], [SECONDS / 2, b, "sine.inOut"], [SECONDS, a]]);
const clear = (layer, values = {}) =>
  layer.effect("Glass", { "Refraction": REFRACTION, "Roughness": 0.04, "Transmission": 1, "Dispersion": 1.2, ...values });
const dark = (layer) => layer.effect("Glass", { "Refraction": 1.5, "Roughness": 0.02, "Transmission": 0, "Metallic": 0 });

// ── light ──────────────────────────────────────────────────────────────────────────────────
// A 4K studio HDRI when it has been fetched (Poly Haven, CC0):
//   curl -L -o glass_garden/studio_4k.hdr https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/4k/studio_small_09_4k.hdr
// otherwise the small generated light map next to this script.
const studio = (() => {
  try { return media("glass_garden/studio_4k.hdr", { name: "Studio Light" }); }
  catch { return media("glass_garden/studio.png", { name: "Studio Light" }); }
})();
studio.environment();

// One orbit, written once: the shader draws its line, the glass spheres ride it.
const ORBIT = { cx: 960, cy: 500, rx: 660, ry: 150, angle: -14 };
const orbitLight = (part) => {
  const layer = rectangle({ name: `Orbit ${part}` }).set("Position", [960, 540]).set("Scale", [9.6, 5.4]).projection("2D");
  cycle(layer.effect("Prism Orbit", {
    "Part": part, "Center": [ORBIT.cx / 19.2, ORBIT.cy / 10.8], "Radius X": ORBIT.rx, "Radius Y": ORBIT.ry,
    "Angle": ORBIT.angle, "Line Width": 1.3, "Glow": 0.7, "Light Leaks": 0.35,
  }));
  return layer;
};
orbitLight("Stage and Orbit");

// A warm light low behind the flower: the glass has something bright to bend.
const ember = ellipse({ name: "Ember" })
  .fill("radial-gradient(#FFD6EC, #8A5CFF40 45%, #00000000 70%)")
  .set("Position", [960, 520]).set("Scale", [3.4, 3.4]).projection("2D");
ember.effect("Blur", { "Radius": 60 });
breathe(ember, "Opacity", 70, 100);

// ── glass disc and slab ────────────────────────────────────────────────────────────────────
const disc = ellipse({ name: "Glass Disc" }).fill("#EAF2FF").set("Position", [610, 470]).set("Scale", [3.3, 3.3]);
disc.effect("Extrude", { "Depth": 16 });
disc.effect("Bevel", { "Radius": 12 });
clear(disc, { "Roughness": 0.12 });
breathe(disc, "Tilt Y", -14, 8);

const slab = roundedRectangle({ name: "Glass Slab" }).fill("#EAF2FF").set("Position", [1300, 470]).set("Scale", [1.3, 4.2]);
slab.effect("Extrude", { "Depth": 44 });
slab.effect("Bevel", { "Radius": 14 });
clear(slab);
breathe(slab, "Tilt Y", 10, 4);

// ── flower ─────────────────────────────────────────────────────────────────────────────────
const bloom = (name, count, scale, reach, turn) => {
  const petal = ellipse({ name: `${name} Petal` }).fill("#0B0B16").set("Scale", scale).set("Position", [reach, 0]);
  petal.effect("Extrude", { "Depth": 8 });
  petal.effect("Bevel", { "Radius": 6 });
  cycle(petal.effect("Petal Wave", { "Wave": WAVE, "Wavelength": reach * 1.4, "Twist": 26, "Lift": reach * 0.35 }));
  clear(petal, { "Roughness": 0.02, "Dispersion": 1.6 });
  const ring = group(petal).name(name).set("Position", [960, 500]);
  ring.effect("Repeater", { "Count": count, "Position X Each": 0, "Position Y Each": 0, "Rotation Each": 360 / count }).whole();
  // One petal step per cycle: the turn closes on itself.
  ring.keys("Rotation", [[0, 0, "Linear"], [SECONDS, turn * 360 / count]]);
  ring.effect("Glow", { "Threshold": 0.6, "Intensity": GLOW, "Radius": 48 });
  return ring;
};
const outer = bloom("Outer Bloom", 8, [1.55, 0.6], 175, 1);
breathe(outer, "Scale", [1, 1], [1.05, 1.05]);
bloom("Inner Bloom", PETALS, [0.55, 0.12], 62, -1);

// ── orbit ──────────────────────────────────────────────────────────────────────────────────
// Two clear spheres ride the orbit once per cycle; the lower half of the ellipse is nearer.
const onOrbit = (turn) => {
  const a = ORBIT.angle * Math.PI / 180, q = [ORBIT.rx * Math.cos(turn), ORBIT.ry * Math.sin(turn)];
  return [ORBIT.cx + Math.cos(a) * q[0] + Math.sin(a) * q[1], ORBIT.cy - Math.sin(a) * q[0] + Math.cos(a) * q[1]];
};
for (const start of [0.35, 0.35 + Math.PI]) {
  const rider = media("glass_garden/orb.obj", { name: "Orbit Sphere" }).set("Scale", [42, 42]);
  clear(rider, { "Roughness": 0.01 });
  const steps = 64, keys = [], depth = [];
  for (let i = 0; i <= steps; i++) {
    const turn = start + 2 * Math.PI * i / steps, at = SECONDS * i / steps;
    keys.push([at, onOrbit(turn), "Linear"]);
    depth.push([at, -40 * Math.sin(turn), "Linear"]);
  }
  rider.keys("Position", keys).keys("Position Z", depth);
}
orbitLight("Orbit Front");

// ── type ───────────────────────────────────────────────────────────────────────────────────
const label = (lines, [x, y], size, color = "#D8DCE8", gap = size * 1.45) =>
  lines.forEach((line, i) =>
    text(line, { name: line }).fill(color).set("Position", [x, y + i * gap]).set("Size", size).projection("2D"));
label(["MOTOLII"], [215, 100], 84, "#F4F2FA");
label(["L I V E   0 0 1"], [150, 160], 17, "#8A8FA3");
label(["G L A S S", "G A R D E N"], [470, 455], 30, "#E8ECF8", 52);
label(["0:00 ──────────── 8:00"], [1690, 70], 14, "#8A8FA3");
label(["L O O P   B Y   Y O U"], [1720, 104], 12, "#8A8FA3");
label(["R E F L E C T", "R E F R A C T", "R E P E A T", "B R E A T H E"], [1700, 540], 13, "#B8BCCB", 24);
label(["M O T O L I I", "R E A L T I M E", "V I S U A L   E N V I R O N M E N T"], [1720, 960], 11, "#8A8FA3", 22);
label(["I D E A S", "F L O W", "R E N D E R", "E V O L V E"], [110, 250], 12, "#8A8FA3", 22);
