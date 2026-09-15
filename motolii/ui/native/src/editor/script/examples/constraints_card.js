// A card that breathes: its width and height are keyed; what sits inside follows the edges it is pinned to
// (Figma's Constraints). The badge keeps to the right, the bar stretches, the mark stays centred, the title stays put.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#0e0f12" });

const card = rectangle({ name: "Card face" }).fill("#F2EDE4");
const badge = ellipse({ name: "Badge" }).fill("#E4572E");
const bar = rectangle({ name: "Bar" }).fill("#161616");
const mark = globalThis.star({ name: "Mark" }).fill("#2E5EAA");
const title = text("Constraints", { name: "Title" }).fill("#161616").font("Helvetica Neue");
const panel = group(card, badge, bar, mark, title).name("Panel");
panel.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 600).set("Height", 380)
  .set("Background", "#F2EDE4").set("Border Radius", 28).set("Position", [300, 250]);
panel.keys("Width", [[0, 600, "Bezier"], [1.5, 1300, "Bezier"], [3, 800, "Bezier"], [4.5, 1320, "Bezier"], [6, 600]]);
panel.keys("Height", [[0, 380, "Bezier"], [2, 580, "Bezier"], [4, 300, "Bezier"], [6, 380]]);
card.set("Opacity", 0);

const pin = (layer, at, h, v, extra = {}) => {
  layer.set("Position Type", "Absolute").set("Position", at).set("Horizontal Constraint", h).set("Vertical Constraint", v);
  for (const [k, val] of Object.entries(extra)) layer.set(k, val);
};
pin(badge, [540, 60], "Right", "Top", { "Scale": [0.22, 0.22] });
pin(bar, [300, 340], "Left & Right", "Bottom", { "Scale": [2.0, 0.08] });
pin(mark, [300, 190], "Center", "Center", { "Scale": [0.45, 0.45] });
pin(title, [160, 60], "Left", "Top", { "Scale": [0.45, 0.45] });
