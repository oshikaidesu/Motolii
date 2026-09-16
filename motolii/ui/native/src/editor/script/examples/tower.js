// A tower and a heavy ball: stack the blocks in the layout, drop one heavy thing, let it fall over.
// The only thing said about the ball is that it weighs more.
comp({ width: 1080, height: 1080, fps: 30, seconds: 7, background: "#11131A" });
const D = 270;
const INK = "#F2EFE6", RED = "#FF4D2E", BLUE = "#4C6FFF", SAND = "#FFC53D";
const C = [INK, RED, BLUE, SAND];
const blocks = [];
for (let k = 0; k < 10; k++) {
  const wide = k % 2 === 0;
  blocks.push(rectangle({ name: `Block ${k}` }).fill(C[k % C.length])
    .set("Scale", [(wide ? 250 : 80) / D, (wide ? 56 : 110) / D]));
}
const ball = ellipse({ name: "Ball" }).fill(RED).set("Scale", [150 / D, 150 / D]);
const g = ellipse({ name: "Gravity" }).fill("#00000000").set("Scale", [6 / D, 6 / D]);
const room = group(...blocks, ball, g).name("Room");
room.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed")
  .set("Width", 880).set("Height", 880).set("Background", "#171A22").set("Border Radius", 16).set("Position", [100, 100]);
// Stack them: wide slab, two legs, wide slab, two legs …
blocks.forEach((b, k) => {
  const level = Math.floor(k / 2);
  const wide = k % 2 === 0;
  const y = 830 - level * 160 - (wide ? 28 : 105);
  const x = wide ? 440 : 440 + (k % 4 === 1 ? -80 : 80);
  b.set("Position Type", "Absolute").set("Margin", 1).set("Hardness", 0.55).set("Position", [x, y]);
});
ball.set("Position Type", "Absolute").set("Hardness", 0.25).set("Heaviness", 14).set("Position", [500, -500]);
g.set("Position Type", "Absolute").set("Position", [440, 440]);
g.effect("Field", { "Spread": 1, "Angle": 90, "Strength": 1600 });
text("one heavy thing", { Position: [540, 1010] }).fill(INK).font("Helvetica Neue").set("Scale", [0.26, 0.26]);
