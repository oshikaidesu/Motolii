// Effector (Notch / Unreal / MoGraph), packaged: a grid of tiles, one block, a box of influence sweeps back and forth; inside it tiles lift, turn, grow, warm up.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#101014" });
const N = 14 * 7, tiles = [];
for (let k = 0; k < N; k++) tiles.push(rectangle({ name: `T${k}` }).fill("#5C7CFA").set("Scale", [90 / 270, 90 / 270]));
const room = group(...tiles).name("Room");
room.set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", 28)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1624).set("Height", 800).set("Position", [148, 120]);
tiles.forEach((t) => t.set("Position", [0, 0]));
tiles.forEach((t) => t.effect("Effector", { "Shape": "Box", "Sweep": 2.5, "Size": 300, "Soft": 160, "Strength": 1, "Lift": -90 }));
text("effector · one block · law × shape: things inside the box lift, turn, grow and warm", { Position: [960, 1020] }).fill("#D0D0D8").font("Helvetica Neue").set("Scale", [0.2, 0.2]);
