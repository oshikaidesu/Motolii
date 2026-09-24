// PRISM GARDEN — glass panes floating in front of a world; every pane shows that same world from the
// garden's own centre. Run it live: `scripts/motolii-ui.sh dev <this file>`; saving redraws the window.
//
// One world, few eyes, many windows:
//   world   — a caustic sea far behind, a luminous heart of two counter-turning petal rings
//   garden  — ONE glass pane repeated into a cloud (Repeater). Its Vism (Prism View, on the shelf)
//             asks for three Views from the garden's centre; every copy reads those same three
//             pictures its own way — turned, folded like a kaleidoscope, bent by how it faces — and
//             reads red, green and blue from three slightly different Views (a many-eyed dispersion)
//   orbs    — glass spheres (an imported OBJ, no texture coordinates) orbiting in front of the garden.
//             The same Vism: their Views look past the words on the camera's plane into the garden,
//             whose panes show their own Views. Read along the ray each sphere refracts, not across
//             a picture it does not have. Main → an orb's View → a pane's View: a world read inside
//             a world that was read.
//   words   — 2D type on the camera's plane: the output shows it, and so do the orbs' Views
// The piece is one 16-second cycle: every motion closes on itself.

const SECONDS = 16;
comp({ width: 1920, height: 1080, fps: 60, seconds: SECONDS, background: "#03030a", loop: true });

// The knobs a live session turns.
const PANES = 96;
const ORBS = 7;
const RING = 660;             // the garden's radius round the heart, px
const SPREAD = [420, 240];    // how far a pane strays from the ring, px
const DEPTH = 360;            // how deep the garden is, px

const cycle = (effect, turns = 1) => effect.keys("Phase", [[0, 0, "Linear"], [SECONDS, turns]]);
const breathe = (layer, property, a, b) =>
  layer.keys(property, [[0, a, "sine.inOut"], [SECONDS / 2, b, "sine.inOut"], [SECONDS, a]]);

// ── light ──────────────────────────────────────────────────────────────────────────────────
const studio = media("glass_garden/studio.png", { name: "Studio Light" });
studio.environment().set("Opacity", 0);

// ── the world, behind the garden ───────────────────────────────────────────────────────────
// The sea stands in the 3D world behind the heart, so the glass refracts it. The Views draw no
// pass effects: they see its own dark paint, the ground every window's picture sits on.
const sea = rectangle({ name: "Sea" }).fill("radial-gradient(#1C1448, #0A0720 55%, #030308)").set("Position", [960, 540]).set("Scale", [20, 11.5]).set("Position Z", 1500).projection("3D");
cycle(sea.effect("Caustic Light", { "Caustics": 1.1, "Scale": 900 }));

const heart = (name, color, count, scale, reach, z, turn) => {
  const petal = ellipse({ name: `${name} Petal` }).fill(color).set("Scale", scale).set("Position", [reach, 0]);
  const ring = group(petal).name(name).set("Position", [960, 540]).set("Position Z", z).projection("3D");
  ring.effect("Repeater", { "Along": "Circle", "Count": count, "Radius": 0, "Rotation Each": 360 / count }).whole();
  ring.keys("Rotation", [[0, 0, "Linear"], [SECONDS, turn * 360 / count]]);
  ring.effect("Glow", { "Threshold": 0.5, "Intensity": 0.7, "Radius": 60 });
  return ring;
};
heart("Heart Outer", "linear-gradient(90deg, #FF2E7E, #FFB23D)", 12, [2.4, 0.5], 300, 1100, 2);
heart("Heart Inner", "linear-gradient(90deg, #2EE6FF, #7A4DFF)", 18, [1.2, 0.26], 140, 1000, -4);
// Specks of light between the heart and the sea: every window's picture catches them too.
const speck = ellipse({ name: "Speck" }).fill("radial-gradient(#FFFFFF, #CFE8FF 40%, #CFE8FF00)").set("Scale", [0.07, 0.07]).set("Position", [0, 0]);
const specks = group(speck).name("Specks").set("Position", [960, 540]).set("Position Z", 1250).projection("3D");
specks.effect("Repeater", { "Count": 240, "Along": "Circle", "Radius": 0, "Position X Random": 2200, "Position Y Random": 1300, "Position Z Random": 300, "Scale Random": 80, "Opacity Random": 0.8, "Seed Random": 3 });
specks.keys("Rotation", [[0, 0, "Linear"], [SECONDS, -360]]);

const core = ellipse({ name: "Core" }).fill("radial-gradient(#FFFFFF, #FFE6F4 30%, #FF6FB000 70%)")
  .set("Position", [960, 540]).set("Scale", [1.6, 1.6]).set("Position Z", 950).projection("3D");
breathe(core, "Scale", [1.4, 1.4], [1.9, 1.9]);

// ── the garden ─────────────────────────────────────────────────────────────────────────────
const pane = roundedRectangle({ name: "Pane" }).fill("#F2F6FF").set("Scale", [0.72, 1.0]).set("Position", [0, 0]);
pane.effect("Extrude", { "Depth": 6 });
const eyes = pane.effect("Prism View", { "View": 0.92, "Eyes": 1, "Fold": 3, "Zoom": 0.9, "Bend": 0.35, "Clear": 0.28, "Hue": 0.35, "Rim": 0.8, "Roughness": 0.05, "Dispersion": 1.4 });
cycle(eyes, 2);
// Every pane turns once round its own axis: edge-on, its rim flashes through the spectrum.
pane.set("Tilt X", 38).keys("Tilt Y", [[0, -22, "Linear"], [SECONDS, 338]]);
const garden = group(pane).name("Garden").set("Position", [960, 540]).set("Position Z", 420).projection("3D");
garden.effect("Repeater", {
  "Count": PANES, "Along": "Circle", "Radius": RING,
  "Rotation Each": 360 / PANES * 7,
  "Position X Random": SPREAD[0], "Position Y Random": SPREAD[1], "Position Z Random": DEPTH, "Rotation Random": 360, "Scale Random": 45, "Seed Random": 7,
});
breathe(garden, "Tilt Y", -10, 10);
garden.keys("Rotation", [[0, 0, "Linear"], [SECONDS, 360]]);

// ── orbs ───────────────────────────────────────────────────────────────────────────────────
const orb = media("glass_garden/orb.obj", { name: "Orb" }).set("Scale", [95, 95]).set("Position", [0, 0]);
cycle(orb.effect("Prism View", { "View": 0.95, "Eyes": 1, "Fold": 0, "Zoom": 0.42, "Scatter": 0, "Lens": 1, "Rim": 0, "Roughness": 0.02, "Dispersion": 1.6 }), -1);
const orbit = group(orb).name("Orbit").set("Position", [960, 880]).set("Position Z", -250).projection("3D");
orbit.effect("Repeater", { "Along": "Circle", "Count": ORBS, "Radius": 560, "Rotation Each": 0 }).whole();
orbit.set("Tilt X", 58);
orbit.keys("Rotation", [[0, 0, "Linear"], [SECONDS, -360 / ORBS]]);

// ── words on the camera's plane ────────────────────────────────────────────────────────────
text("W O R L D   I N   T H E   G L A S S", { name: "Words" }).fill("#F4F2FA").set("Position", [960, 930]).set("Size", 64).set("Opacity", 70).projection("2D");

// ── type ───────────────────────────────────────────────────────────────────────────────────
const label = (lines, [x, y], size, color = "#D8DCE8", gap = size * 1.45) =>
  lines.forEach((line, i) =>
    text(line, { name: line }).fill(color).set("Position", [x, y + i * gap]).set("Size", size).projection("2D"));
label(["P R I S M   G A R D E N"], [150, 96], 26, "#F4F2FA");
label(["1   W O R L D", "3   E Y E S", `${PANES + ORBS}   W I N D O W S`], [150, 150], 12, "#8A8FA3", 22);
label(["M O T O L I I   ·   V I S M   V I E W S"], [1600, 1010], 11, "#8A8FA3");
