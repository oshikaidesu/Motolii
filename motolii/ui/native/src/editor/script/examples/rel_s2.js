// SANKOU! #2 を関係で書く: 見出しは 1 行分の窓(Fixed・Overflow Clip)、中は縦に 2 行(content・label)。窓がどちらを見せるかは
// 札 1 つ(Justify Content: End = label / Start = content)。hover の時刻に札を切り替えると、移り方(Transition)が 0.4 s で滑らせる。
// 鍵は hover(入力の所作)の Hold だけ — Clip 辺・Position の鍵は 0。
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#F4F1EA" });
const SIZE = 140, H = SIZE * 1.2, W = 1400;
const link = (word, y, hoverIn, hoverOut) => {
  const content = text(word, { name: `${word} content` }).fill("#161616").font("Helvetica Neue").set("Size", SIZE);
  const label = text(word, { name: `${word} label` }).fill("#9A958C").font("Helvetica Neue").set("Size", SIZE);
  const win = group(content, label).name(`${word} window`);
  win.set("Display", "Flex").set("Flex Direction", "Column").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H)
    .set("Overflow", "Clip").set("Justify Content", "End").set("Position", [200, y]);
  for (const t of [content, label]) t.set("Position", [0, 0]).set("Transition Duration", 0.4);
  win.key("Justify Content", 0, "End", "Hold").key("Justify Content", hoverIn, "Start", "Hold").key("Justify Content", hoverOut, "End", "Hold");
};
link("WORKS", 240, 0.8, 2.0);
link("ABOUT", 240 + H + 30, 1.6, 3.0);
link("CONTACT", 240 + (H + 30) * 2, 2.6, 4.2);
