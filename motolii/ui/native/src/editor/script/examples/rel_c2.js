// Codrops #2 を関係で書く: 帯は語を抱く箱(Hug)で、語に付いて置かれる(Position Anchor)。抱く写しが at に来ると
// 箱はその幅へ広がり、移り方(Transition)が 0.8 s 掛けて広げる。Scale / Opacity の鍵は 0。
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#1a1720" });
const SIZE = 120, BAND = "#6a5ace", HI = "#e1def4", INK = "#7a7580";
const line = (y, words, at) => {
  const made = words.map(([word, hot]) => {
    let band = null;
    if (hot) {
      const ghost = text(word, { name: `${word} ghost` }).fill(BAND).font("Helvetica Neue").set("Size", SIZE);
      band = group(ghost).name(`${word} band`);
      band.set("Display", "Flex").set("Padding", [0, SIZE * 0.05]).set("Background", BAND).set("Border Radius", 8).set("Overflow", "Clip").set("Transform Origin", "Center").set("Transition Duration", 0.8);
      ghost.set("Position", [0, 0]).time(at, 4 - at);
    }
    return [text(word, { name: word }).fill(hot ? HI : INK).font("Helvetica Neue").set("Size", SIZE), band];
  });
  const row = group(...made.map(([t]) => t)).name(`Line ${y}`);
  row.set("Display", "Flex").set("Gap", SIZE * 0.5).set("Align Items", "Center").set("Position", [160, y]);
  for (const [t, band] of made) { t.set("Position", [0, 0]); if (band) band.set("Position Anchor", t).set("Position Area", "Center"); }
};
line(360, [["MARK", true], ["WHAT", false], ["MATTERS", true]], 0.4);
line(620, [["THEN", false], ["READ", true], ["ON", false]], 1.8);
