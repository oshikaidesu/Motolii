// Run in an empty document. Compare the two upper rows while scrubbing.
// The lower white source should be covered by its translucent Blob group copy.
(() => {
  comp({ width: 640, height: 360, fps: 30, seconds: 2, background: "#000000" });
  const moving = (name, y, color) => rectangle({ name }).fill(color).projection("2D")
    .set("Scale", [1 / 3, 1 / 3])
    .keys("Position", [[0, [80, y]], [1, [560, y]], [2, [80, y]]]);
  moving("Motion reference", 75, "#ffffff");
  const blurred = moving("Motion sampled", 170, "#55ccff");
  blurred.effect("Motion Blur", { Tune: 3 });
  const source = moving("Blob source", 270, "#ffffff");
  const follower = rectangle({ name: "Blob material" }).fill("#ff555588").projection("2D");
  const copies = group(follower).name("Blob group").projection("2D")
    .set("Position", [0, 0]);
  follower.set("Anchor", [0, 0]).set("Position", [0, 0]);
  copies.effect("Blob Track", {
    "Track Layer": source, "Find By": "Brightness", "Min Size": 1,
    "Separation": 0, "Detail": 120, "Fit": "Box", "Material Size": [90, 90],
  });
})();
