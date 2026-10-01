// Cards stacked into depth: one Flex Direction = Depth, a key on Gap fans them out, the stack turns in space.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#0c0d12" });

const deck = [
  { word: "01  Layout", color: "#E4572E" },
  { word: "02  Depth", color: "#F3B61F" },
  { word: "03  Rotation", color: "#7FA37A" },
  { word: "04  Fill", color: "#2E5EAA" },
  { word: "05  Clip", color: "#534AB7" },
  { word: "06  Motion", color: "#F2EDE4", ink: "#161616" },
];
const texts = [];
const cards = deck.map((c) => {
  const t = text(c.word, { name: `${c.word} text` }).fill(c.ink ?? "#ffffff").font("Helvetica Neue");
  t.set("Position", [0, 0]).set("Scale", [0.9, 0.9]);
  texts.push(t);
  const card = group(t).name(c.word);
  card.set("Display", "Flex").set("Flex Direction", "Column").set("Justify Content", "End")
    .set("Padding", [40, 34]).set("Background", c.color).set("Border Radius", 28)
    .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 560).set("Height", 340);
  return card;
});
const stack = group(...cards).name("Stack");
stack.set("Display", "Flex").set("Flex Direction", "Depth").set("Align Items", "Center").set("Position", [680, 370]);
for (const card of cards) card.set("Position", [0, 0]);

stack.keys("Gap", [[0.3, 0, "Bezier"], [1.8, 170, "Bezier"], [3.2, 60, "Bezier"], [4.4, 190]]);
stack.keys("Tilt Y", [[0, 20, "Bezier"], [2.5, 48, "Bezier"], [5, 30]]);
stack.keys("Tilt X", [[0, -8, "Bezier"], [5, -18]]);

for (const layer of [stack, ...cards]) layer.projection("3D");
