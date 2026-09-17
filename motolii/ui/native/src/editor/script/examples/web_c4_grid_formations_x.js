// Codrops #4 化合 — the same three formations, but every seventh cell is a moving video clip (the Web grid is stills); the one law moves them alike.
// Codrops #4 — OnScrollLayoutFormations (js/index.js, css/base.css): a grid of pictures scatters and gathers in formations (3 of the 9 here).
// Fourth grid: calculateInitialTransform(el, 250, 300, 2000) — each cell starts pushed away from the screen centre (x, y ±250 along the angle,
// z = 2000·distanceFactor, rotateX/Y = ±300·distanceFactor, rotateX halved), autoAlpha 0, scale 0.7 → 0/1, ease 'expo', stagger {amount:0.2, from:'center', grid:[4,9]}.
// Then the grid changes its columns (the layout formation; here the cells' Transition = FLIP). Second grid, played as the exit:
// y: innerHeight, rotation ±3·|pos − middle|, ease 'power3', stagger {amount:0.3, from:'center'}. scrub → time.
comp({ width: 1920, height: 1080, fps: 30, seconds: 7, background: "#0f0f12" });
const EXPO = "expo", P3IN = "power3.in";
const COLS = 9, ROWS = 4, GAP = 12, W = 1840, H = 1000, C = ["#E4572E", "#F3B61F", "#2E5EAA", "#7FA37A", "#534AB7", "#F2EDE4", "#D9553A"];
const cw = (W - 2 * GAP - (COLS - 1) * GAP) / COLS, ch = (H - 2 * GAP - (ROWS - 1) * GAP) / ROWS, mid = (COLS * ROWS - 1) / 2;
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/", cells = [];
for (let i = 0; i < COLS * ROWS; i++) {
  const cell = i % 7 === 3 ? media(MAT + `clip${(i % 3) + 1}.mp4`, { name: `Clip ${i}` }) : rectangle({ name: `Pic ${i}` }).fill(C[(i * 5) % C.length]);
  // calculateInitialTransform: the cell's centre against the screen centre.
  const cx = 40 + GAP + (i % COLS) * (cw + GAP) + cw / 2, cy = 40 + GAP + Math.floor(i / COLS) * (ch + GAP) + ch / 2;
  const dx = cx - 960, dy = cy - 540, angle = Math.atan2(Math.abs(dy), Math.abs(dx)), df = Math.hypot(dx, dy) / Math.hypot(960, 540);
  const tx = Math.abs(Math.cos(angle)) * 250, ty = Math.abs(Math.sin(angle)) * 250;
  const from = [Math.sign(dx) * tx, Math.sign(dy) * ty], z = -2000 * df, rx = -Math.sign(dy) * (ty / 250) * 300 * df * 0.5, ry = Math.sign(dx) * (tx / 250) * 300 * df;
  cell.key("Position", 0.3, from, EXPO).key("Position", 2.3, [0, 0]).key("Position Z", 0.3, z, EXPO).key("Position Z", 2.3, 0)
    .key("Tilt X", 0.3, rx, EXPO).key("Tilt X", 2.3, 0).key("Tilt Y", 0.3, ry, EXPO).key("Tilt Y", 2.3, 0)
    .key("Opacity", 0.3, 0, EXPO).key("Opacity", 2.3, 1).key("Scale", 0.3, [0.7, 0.7], EXPO).key("Scale", 2.3, [1, 1]);
  // Exit = second grid backwards: down by the screen height, fanned ±3° per step from the middle, power3 in.
  const fan = (i < mid ? 1 : -1) * Math.abs(i - mid) * 3;
  cell.key("Position", 5.0, [0, 0], P3IN).key("Position", 6.2, [0, 1080]).key("Rotation", 5.0, 0, P3IN).key("Rotation", 6.2, fan);
  cells.push(cell);
}
const grid = group(...cells).name("Grid");
grid.set("Display", "Grid").set("Grid Columns", COLS).set("Gap", GAP).set("Padding", [GAP, GAP])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H).set("Position", [40, 40])
  .set("Stagger", 0.2).set("Stagger From", "Center");
// The layout formation: 9 columns → 6 → 12, every cell slides to its new cell (Transition).
grid.keys("Grid Columns", [[0, COLS, "Hold"], [3.0, 6, "Hold"], [4.0, 12, "Hold"]]);
cells.forEach((c, i) => c.set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill").set("Object Fit", i % 7 === 3 ? "Contain" : "Fill").set("Transition Duration", 0.7).set("Transition Easing", "Ease In Out"));
for (const layer of [grid, ...cells]) layer.projection("3D");
