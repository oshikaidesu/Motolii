// Probe B: a "Shatter Repeater" recipe made only of existing Host vocabulary (the Repeater effect and keyed rows).
// Exposed: pieces, amount, spread, seed. Everything else is internal to the recipe.
function shatter(layer, { pieces = 12, amount = 1, spread = 300, seed = 1, seconds = 1.5, delay = 0.03 } = {}) {
  const draw = random(seed);
  const r = layer.effect("Repeater", { Count: pieces, Along: "Grid", Pick: "Random", Transform: "Each", "Seed Random": Math.round(draw() * 1000), "Delay Each": delay });
  // Every scatter row travels from rest (0) to its full value: the whole burst is keys on Repeater rows.
  const burst = { "Position X Random": spread * amount, "Position Y Random": spread * amount, "Rotation Random": 180 * amount, "Scale Random": -0.6 * amount, "Opacity Each": -1 / pieces };
  for (const [row, value] of Object.entries(burst)) {
    r.key(row, 0, 0, "power2.out").key(row, seconds, value);
  }
  return r;
}

comp({ seconds: 3, background: "#101018" });
const glass = rectangle({ name: "Glass" });
shatter(glass, { pieces: 16, amount: 1, spread: 400, seed: 3 });
