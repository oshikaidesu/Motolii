// Hanging signs. The string is a thin box hung from the same pin, so it swings with its card:
// Length sets the beat the way a real pendulum does. One swing, then rest.
comp({ width: 1440, height: 1080, fps: 30, seconds: 6, background: "#101217" });
const D = 270, C = ["#FF5470", "#FFD166", "#4ECDC4", "#E7E7E7", "#A06CD5"];
rectangle({ name: "Rail" }).fill("#3A3F4B").set("Scale", [1080 / D, 8 / D]).set("Position", [180, 200]);
for (let k = 0; k < 5; k++) {
  const x = 245 + k * 200, drop = 260 + k * 30;
  // 札と糸は 1 つの物なので、拍を揃える(長さは振り子の拍を決めるが、ここでは組み立てが先)。
  const swing = { "Swing": 15, "Settle": 3.2, "Stagger": 0.14, "Beat": 1.3 + k * 0.08 };
  rectangle({ name: `String ${k}` }).fill("#3A3F4B").set("Scale", [3 / D, drop / D]).set("Position", [x + 74, 205 + drop / 2])
    .effect("Hang", Object.assign({ "Length": drop / 2 }, swing));
  rectangle({ name: `Sign ${k}` }).fill(C[k]).set("Scale", [150 / D, 210 / D]).set("Position", [x + 74, 205 + drop + 105])
    .effect("Hang", Object.assign({ "Length": drop + 105 }, swing));
}
