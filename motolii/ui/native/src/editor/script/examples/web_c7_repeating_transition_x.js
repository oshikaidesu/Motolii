// Codrops #7 化合 — the same trail of frames, but the movers come in tilted (Tilt Y, 2.5D) and flatten as they land, and the panel opens on a video clip (the Web only moves a still).
// RepeatingImageTransition (js/index.js): a clicked picture leaves copies of its frame along the way to the panel.
// Source: generateMotionPath(startRect, endRect, steps) lerps centre and size over steps + 2 points and drops the ends (6 movers);
// mover i: delay i · stepInterval (0.05), fromTo {opacity .4, clip hide} → {opacity 1, clip reveal} stepDuration .35 'sine.in',
// then to clip from, .35 'sine' after moverPauseBeforeExit .14; top-bottom: from = inset(0 0 100% 0) (pinned top), hide = inset(100% 0 0 0)
// (pinned bottom). Panel: fromTo hide → reveal, .7 'sine.inOut', delay steps · .05. Clicked item: opacity 0 + clip from, .7 'sine'; others .8 scale.
comp({ width: 1920, height: 1080, fps: 30, seconds: 3, background: "#111114" });
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/"; const PHOTO = MAT + "photo2.png";
const SIN = { kind: "Bezier", x1: 0.47, y1: 0, x2: 0.745, y2: 0.715 }, SOUT = { kind: "Bezier", x1: 0.39, y1: 0.575, x2: 0.565, y2: 1 }, SIO = { kind: "Bezier", x1: 0.445, y1: 0.05, x2: 0.55, y2: 0.95 };
const STEPS = 6, DT = 0.05, DUR = 0.35, PAUSE = 0.14, CLICK = 0.3;
// A picture in a box with clip-path: inset(). edge(t, state, ease): "bottom" = collapsed on the bottom edge (Clip Top = h),
// "full" (inset 0), "top" = collapsed on the top edge (Clip Bottom = h) — the pinned edge is the box's own edge, nothing else moves.
const pic = (name, x, y, w, h) => {
  const img = media(name === "Panel" ? MAT + "clip1.mp4" : PHOTO, { name: `${name} img` }).set("Scale", [Math.max(w / 1920, h / 1080), Math.max(w / 1920, h / 1080)]);
  const box = group(img).name(name).set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
    .set("Width", w).set("Height", h).set("Overflow", "Clip").set("Position", [x, y]);
  img.set("Position", [0, 0]);
  const edge = (t, state, ease) => { box.key("Clip Top", t, state === "bottom" ? h : 0, ease).key("Clip Bottom", t, state === "top" ? h : 0, ease); return o; };
  const o = { box, img, edge }; return o;
};
const start = [200, 380, 320, 240], end = [900, 140, 880, 800];
const grid = [[200, 100], [560, 100], [200, 380], [560, 380], [200, 660], [560, 660]].map(([x, y], i) => pic(`Item ${i}`, x, y, 320, 240));
grid.forEach((g, i) => { if (i === 2) g.edge(CLICK, "full", SOUT).edge(CLICK + 0.7, "top").img.key("Opacity", CLICK, 1, SOUT).key("Opacity", CLICK + 0.7, 0);
  else g.img.key("Opacity", CLICK + 0.1 * Math.abs(i - 2), 1, SOUT).key("Opacity", CLICK + 0.3 + 0.1 * Math.abs(i - 2), 0); });
for (let i = 1; i <= STEPS; i++) {
  const u = i / (STEPS + 1), lerp = (a, b) => a + (b - a) * u, w = lerp(start[2], end[2]), h = lerp(start[3], end[3]);
  const m = pic(`Mover ${i}`, lerp(start[0] + start[2] / 2, end[0] + end[2] / 2) - w / 2, lerp(start[1] + start[3] / 2, end[1] + end[3] / 2) - h / 2, w, h);
  const at = CLICK + (i - 1) * DT;
  m.edge(at, "bottom", SIN).edge(at + DUR, "full", "Hold").edge(at + DUR + PAUSE, "full", SOUT).edge(at + 2 * DUR + PAUSE, "top");
  m.img.key("Opacity", at, 0.4, SIN).key("Opacity", at + DUR, 1);
  m.box.key("Tilt Y", at, 35 * (1 - u), SIN).key("Tilt Y", at + DUR, 0).projection("3D");
}
pic("Panel", ...end).edge(0, "bottom", "Hold").edge(CLICK + STEPS * DT, "bottom", SIO).edge(CLICK + STEPS * DT + 0.7, "full");
