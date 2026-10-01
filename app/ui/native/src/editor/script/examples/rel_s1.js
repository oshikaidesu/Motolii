// SANKOU! #1 を関係で書く: 行は 1 行分の窓(Fixed・Overflow Clip・Flex Wrap)。中は [詰め物, 語, 語…] で、詰め物が幅いっぱいなので
// 語は 2 行目(窓の下)に折り返している。詰め物が居なくなる時刻(time)に語は 1 行目へ折り返し直す — 動かすのは移り方(Transition)、
// 語ごとの遅れは行の Stagger が配る。Position / Opacity の鍵は 0。
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#F4F1EA" });
const SIZE = 150, H = SIZE * 1.25, W = 1900;
["TOSHIYUKI HASHIMOTO", "DESIGN & DIRECTION", "TOKYO — 2026"].forEach((line, row) => {
  const pad = rectangle({ name: "pad" }).fill("#F4F1EA00").time(0, 0.3 + row * 0.35);
  const words = line.split(" ").map((w) => text(w, { name: w }).fill(row === 1 ? "#C8412B" : "#161616").font("Helvetica Neue").set("Size", SIZE));
  const win = group(pad, ...words).name(`Line ${row}`);
  win.set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", SIZE * 0.28).set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H)
    .set("Overflow", "Clip").set("Stagger", 0.5).set("Position", [160, 300 + row * (H + 40)]);
  pad.set("Position", [0, 0]).set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", W).set("Height", H);
  words.forEach((w) => w.set("Position", [0, 0]).set("Transition Duration", 1.2));
});
