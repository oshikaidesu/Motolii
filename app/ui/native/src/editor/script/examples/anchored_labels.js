// Labels that belong to things (CSS anchor positioning): each card declares which thing it sits on and on which side.
// The things wander; the cards follow at their own distance, and when a card changes sides it slides around.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#0e0f12" });

const PAPER = "#F2EDE4", INK = "#161616";
const card = (words, color, ink = INK) => {
  const t = text(words, { name: `${words} text` }).fill(ink).font("Helvetica Neue");
  t.set("Position", [0, 0]).set("Scale", [0.36, 0.36]);
  const g = group(t).name(words);
  g.set("Display", "Flex").set("Padding", [18, 10]).set("Background", color).set("Border Radius", 10);
  t.set("Position", [0, 0]);
  return g;
};

const circle = ellipse({ name: "Circle" }).fill("#E4572E").set("Scale", [0.8, 0.8]);
circle.keys("Position", [[0, [360, 420], "Bezier"], [2, [900, 300], "Bezier"], [4, [1300, 700], "Bezier"], [6, [620, 760]]]);
const square = rectangle({ name: "Square" }).fill("#2E5EAA").set("Scale", [0.6, 0.6]);
square.keys("Position", [[0, [1500, 250], "Bezier"], [3, [1100, 520], "Bezier"], [6, [1560, 820]]]);
square.keys("Rotation", [[0, 0, "Bezier"], [6, 180]]);
const star = globalThis.star({ name: "Star" }).fill("#F3B61F").set("Scale", [0.7, 0.7]);
star.keys("Position", [[0, [700, 850], "Bezier"], [3, [420, 640], "Bezier"], [6, [960, 520]]]);

const labels = [
  [card("CIRCLE  01", PAPER), circle, [[0, "Top", "Hold"], [2.2, "Right", "Hold"], [4.2, "Bottom", "Hold"]]],
  [card("SQUARE  02", "#2E5EAA", PAPER), square, [[0, "Left", "Hold"], [3.0, "Top Left", "Hold"]]],
  [card("STAR  03", "#F3B61F"), star, [[0, "Bottom Right", "Hold"], [1.6, "Top", "Hold"], [4.6, "Right", "Hold"]]],
];
for (const [label, thing, sides] of labels) {
  label.set("Position Anchor", thing).set("Margin", 14).set("Transition Duration", 0.45).set("Transition Easing", "Ease In Out");
  label.keys("Position Area", sides);
}
