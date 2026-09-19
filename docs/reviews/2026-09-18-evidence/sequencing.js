// Sequencing (kynd's MotionToolKit) as one block: every tile runs the same windows — back in, hold, elastic quarter turn, beat pulses, cubic out — staggered from the centre.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#101014" });
const N = 14 * 7, tiles = [];
for (let k = 0; k < N; k++) tiles.push(rectangle({ name: `T${k}` }).fill("#5C7CFA").set("Scale", [90 / 270, 90 / 270]));
const room = group(...tiles).name("Room");
room.set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", 28)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1624).set("Height", 800).set("Position", [148, 120]);
tiles.forEach((t) => t.set("Position", [0, 0]));
tiles.forEach((t) => {
  t.effect("Sequencing", { "Stagger": 0.03, "Enter": 0.6, "Hold": 0.8, "Turn": 0.6, "Pulses": 2, "BPM": 120, "Leave": 0.5, "Drop": 240 });
});
text("sequencing · one block · kynd's time windows: back in → hold → elastic quarter turn → beat pulses → cubic out, staggered from the centre", { Position: [960, 1020] }).fill("#D0D0D8").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
