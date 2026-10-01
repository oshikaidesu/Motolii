// Codrops #3 を関係で書く: 字は行の箱に並び、行の板は奥へ寝ている(Tilt X)。Arrive が板の「手前の下」から 1 字ずつ届ける
// = 世界では奥行き(カメラの近く)から起き上がって紙に着く。近い字はカメラの Near Fade が薄くする。Position Z / Tilt / Opacity の鍵は 0。
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#f2efe9" });
const SIZE = 150, INK = "#1c1a22";
camera({ name: "Camera" }).set("Near Fade", 1000);
["LETTERS RISE", "OUT OF THE DEPTH"].forEach((line, row) => {
  const chars = [...line].map((ch, k) => text(ch, { name: `${ch}${k}` }).fill(INK).font("Helvetica Neue").set("Size", SIZE));
  const board = group(...chars).name(`Row ${row}`);
  board.set("Display", "Flex").set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug").set("Transform Origin", "Center")
    .set("Position", [960, 430 + row * (SIZE + 90)]).set("Tilt X", -25);
  chars.forEach((c) => c.set("Position", [0, 0]).effect("Arrive", { "From": 90, "Spin": 0, "Stagger": 0.04 }));
  for (const layer of [board, ...chars]) layer.projection("3D");
});
