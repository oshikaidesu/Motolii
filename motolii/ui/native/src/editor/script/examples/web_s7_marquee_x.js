// SANKOU! #7 化合 — the same two bands on one sheet leaning into the room (2.5D: Tilt Y / Tilt X on the group, projection 3D).
// recruit.toyox.co.jp / musu-sauna.com: a band of words runs across, two bands half a period apart.
// Source (toyox): @keyframes loop { 0% translateX(100%) → translateX(-100%) } `loop 120s -60s linear infinite` on span 1
// and `loop2` (0 → -200%) on span 2 — the negative delay is the half-period phase. musu: `width:max-content`,
// `weather-scroll 10s linear infinite` to translate(-50%). The band is a Hug row of copies; time is taken mod the period:
// the wrap lands on a frame boundary, so the last frame runs into the first.
comp({ width: 1920, height: 1080, fps: 30, seconds: 10, background: "#091E2D" });
const T = 10, FPS = 30, SEC = 10, COPY = 1150, GAP = 120, W = COPY + GAP;
const band = (words, y, ink, phase, size) => {
  const copies = [0, 1, 2, 3].map((i) => {
    const t = text(words, { name: `${words} ${i}` }).fill(ink).font("Helvetica Neue").set("Size", size);
    const cell = group(t).name(`Cell ${words} ${i}`);
    cell.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Hug").set("Width", COPY);
    t.set("Position", [0, 0]);
    return cell;
  });
  const row = group(...copies).name(`Band ${words}`);
  row.set("Display", "Flex").set("Gap", GAP).set("Horizontal Sizing", "Hug").set("Vertical Sizing", "Hug");
  copies.forEach((c) => c.set("Position", [0, 0]));
  // x = -W · frac(t / T + phase): linear runs, and a wrap between two neighbouring frames.
  const keys = [[0, [-W * phase, y], "Linear"]];
  for (let at = (1 - phase) * T; at <= SEC; at += T) keys.push([at - 1 / FPS, [-W + W / (T * FPS), y], "Linear"], [at, [0, y], "Linear"]);
  keys.push([SEC, [-W * ((SEC / T + phase) % 1), y]]);
  row.keys("Position", keys);
  for (const layer of [row, ...copies]) layer.projection("3D");
  return row;
};
const sheet = group(band("TOYOX RECRUIT", 300, "#FFFFFF", 0, 130), band("WE ARE HIRING", 560, "#006FBF", 0.5, 130)).name("Sheet");
sheet.set("Tilt Y", 18).set("Tilt X", 5).projection("3D");
