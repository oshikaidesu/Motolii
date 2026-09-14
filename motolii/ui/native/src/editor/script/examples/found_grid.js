// A grid found in chance: cards drift where a Repeater's dice threw them. Track Overlay reads their boxes (Detection
// Method = Layers) and raises a grid from their edges — near edges share one line — and, as Snap Strength rises, pulls the
// cards onto the lines it found. Nothing was placed on a grid; the grid is read out of the drift, and moves with it.
comp({ width: 1920, height: 1080, fps: 30, seconds: 8, background: "#F1EFEA" });

const RED = "#D9553A", INK = "#151515", STONE = "#CFC9BE";
const made = [];
const keep = (l) => { made.push(l.id); return l; };

const drift = (name, color, count, seed, size, from, to) => {
  const l = keep(rectangle({ name }).fill(color)).set("Scale", size);
  l.effect("Repeater", { "Count": count, "Position X Each": 0, "Position Y Each": 0, "Position X Random": 820, "Position Y Random": 440, "Scale Random": 40, "Seed Random": seed });
  l.keys("Position", [[0, from, "Bezier"], [8, to]]);
  return l;
};
drift("Stone cards", STONE, 5, 3, [0.9, 0.6], [880, 560], [1040, 520]);
drift("Ink cards", INK, 3, 17, [0.5, 0.75], [1060, 500], [880, 580]);
drift("Red cards", RED, 2, 41, [0.7, 0.45], [960, 470], [980, 610]);

const words = [["FOUND", 1.5, [520, 300]], ["GRID", 1.5, [1380, 760]], ["chance, then order", 0.32, [1300, 250]]];
for (const [w, sc, at] of words) {
  const t = keep(text(w, { name: w })).fill(INK).font("Helvetica Neue").set("Scale", [sc, sc]);
  t.keys("Position", [[0, at, "Bezier"], [8, [at[0] + (at[0] > 960 ? -90 : 90), at[1] + 40]]]);
}

// The reader of boxes, on top: grid lines from edges, thin corner brackets, and the pull.
const hud = keep(rectangle({ name: "Found grid" }).fill("#ffffff")).set("Anchor", [0, 0]).set("Position", [0, 0]);
const fx = hud.effect("Track Overlay", {
  "Detection Method": "Layers",
  "Grid Enabled": 1, "View Mode": "Edge", "Grid Color": RED, "Grid Opacity": 1, "Line Thickness": 1.2, "Merge Distance": 40,
  "Box Enabled": 1, "Color Box Stroke": INK, "Box Stroke": 1, "Box Gap Enabled": 1, "Box Gap Size": 0.7,
});
fx.key("Snap Strength", 0, 0, "Hold").key("Snap Strength", 2.0, 0, "Bezier").key("Snap Strength", 3.2, 1, "Hold").key("Snap Strength", 5.6, 1, "Bezier").key("Snap Strength", 6.8, 0);

op("setAttrs", { layers: made, patch: { projection: "2D" } });
