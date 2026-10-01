// Words flow around things (CSS shape-outside): the circle and the star declare their shape, the column of type
// parts around them on both sides as they drift through, and each word slides to its new line instead of snapping.
comp({ width: 1080, height: 1350, fps: 30, seconds: 6, background: "#F2EDE4" });

const INK = "#161616", RED = "#E4572E", BLUE = "#2E5EAA";

const words = text(
  "A BOX IS A PROMISE ABOUT DISTANCE. EVERY THING ON THE PAGE SAYS HOW MUCH ROOM IT WANTS, AND THE WORDS LISTEN: " +
  "THEY PART WHERE A SHAPE STANDS AND CLOSE AGAIN WHEN IT MOVES ON. NOTHING IS PLACED BY HAND. THE COLUMN IS A RIVER, " +
  "THE CIRCLE AND THE STAR ARE STONES, AND THE LINES FIND THEIR WAY AROUND THEM FRAME AFTER FRAME.",
  { name: "Column" }).fill(INK).font("Helvetica Neue").set("Alignment", "Left");
words.set("Position", [0, 0]).set("Scale", [0.5, 0.5]).set("Transition Duration", 0.5).set("Transition Easing", "Ease In Out");

const circle = ellipse({ name: "Circle" }).fill(RED);
circle.set("Scale", [1.3, 1.3]).set("Shape Outside", "Content").set("Shape Margin", 16);
// The circle travels, rests, travels again: the words ripple while it moves and settle while it rests.
circle.keys("Position", [[0, [220, 300], "Hold"], [1, [220, 300], "Bezier"], [2.2, [760, 640], "Hold"], [3.4, [760, 640], "Bezier"], [4.6, [300, 1010], "Hold"]]);

const star = globalThis.star({ name: "Star" }).fill(BLUE);
star.set("Scale", [0.9, 0.9]).set("Shape Outside", "Content").set("Shape Margin", 12).set("Position", [800, 1060]);
star.key("Rotation", 0, 0, "Linear").key("Rotation", 6, 72);

const page = group(words, circle, star).name("Page");
page.set("Display", "Flex").set("Flex Direction", "Column").set("Justify Content", "Center").set("Padding", [72, 90])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1080).set("Height", 1350).set("Position", [0, 0]);
words.set("Horizontal Sizing", "Fill");
circle.set("Position Type", "Absolute");
star.set("Position Type", "Absolute");
