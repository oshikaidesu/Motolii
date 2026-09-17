// Codrops #10 化合 — the same opening face, but what it reveals is a physics room: the menu items are bodies dropping under a gravity field, piling on the floor (walls belong to the box).
// EaseReverseClipMenu (js/index.js, css/base.css): a face opens from one point to the four corners; closing runs the
// eases the other way. Source: menu clip-path polygon(50% 50% ×4) → polygon(0 0, 100% 0, 100% 100%, 0 100%), 0.8 s 'expo',
// easeReverse 'expo', at start + 0.3; cover items go 600 px out from the viewport centre (getRadialPosition), rotation random(−30, 30),
// opacity 0, 0.7 s 'expo', delay 0.3 · (1 − distance / maxDistance) (near ones wait), easeReverse 'elastic.out(0.3)' on a full close.
// The four corners move together from the centre, so the face = a full-screen box with clip-path: inset(50% 50% 50% 50%) → inset(0):
// the four Clip edges are keyed, the content does not move. Close: timeline reversed with the reverse eases.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#F4F1EA" });
const EXPO = { kind: "Bezier", x1: 0.19, y1: 1, x2: 0.22, y2: 1 }, ELASTIC = { kind: "Elastic", limit: 1.1, period: 0.3, damp: 0.35 };
const OPEN = 0.3, CLOSE = 2.3, TL = 1.1, rnd = random(3);
// Cover items: labels scattered over the page.
[["STUDIO", 260, 180], ["WORK", 1500, 220], ["JOURNAL", 380, 880], ["CONTACT", 1560, 860], ["ABOUT", 960, 120], ["2026", 960, 960], ["EN", 140, 540], ["JP", 1780, 540]]
  .forEach(([s, x, y]) => {
    const d = Math.hypot(x - 960, y - 540), progress = Math.min(1, d / Math.hypot(960, 540)), delay = 0.3 * (1 - progress);
    const out = [x + (x - 960) / d * 600, y + (y - 540) / d * 600], rot = -30 + rnd() * 60;
    const t = text(s, { name: s }).fill("#1A1A1E").font("Helvetica Neue").set("Size", 64);
    const a = OPEN + delay, b = CLOSE + (TL - delay - 0.7);
    t.keys("Position", [[a, [x, y], EXPO], [a + 0.7, out, "Hold"], [b, out, ELASTIC], [b + 0.7, [x, y]]]);
    t.keys("Rotation", [[a, 0, EXPO], [a + 0.7, rot, "Hold"], [b, rot, ELASTIC], [b + 0.7, 0]]);
    t.keys("Opacity", [[a, 1, EXPO], [a + 0.7, 0, "Hold"], [b, 0, ELASTIC], [b + 0.7, 1]]);
  });
const items = ["HOME", "PROJECTS", "TEAM", "CONTACT"].map((s) => text(s, { name: `Menu ${s}` }).fill("#F4F1EA").font("Helvetica Neue").set("Size", 130));
const gravity = ellipse({ name: "Gravity" }).fill("#E8442E00").set("Scale", [2 / 270, 2 / 270]);
const content = group(...items, gravity).name("Menu content").set("Display", "Flex").set("Flex Direction", "Column").set("Align Items", "Center")
  .set("Justify Content", "Center").set("Gap", 16).set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1920).set("Height", 1080);
items.forEach((i, k) => i.set("Position Type", "Absolute").set("Hardness", 0.4).set("Position", [420 + k * 360, 80 + (k % 2) * 140]));
gravity.set("Position Type", "Absolute").set("Position", [960, 1060]).effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1500 });
const face = group(content).name("Menu").set("Display", "Flex").set("Background", "#E8442E")
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1920).set("Height", 1080).set("Position", [0, 0]);
const m = OPEN + 0.3, c = CLOSE + (TL - 0.3 - 0.8);
// polygon from the centre point to the corners = every edge from half the box to 0, and back.
for (const [edge, half] of [["Clip Top", 540], ["Clip Bottom", 540], ["Clip Left", 960], ["Clip Right", 960]])
  face.keys(edge, [[m, half, EXPO], [m + 0.8, 0, "Hold"], [c, 0, EXPO], [c + 0.8, half]]);
for (const l of [face, content, ...items]) l.projection("2D");
