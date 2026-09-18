// Falloff entrance (Cavalry docs), packaged: a grid of words in a room, one block, a circle sweeps left to right.
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#F0ECE3" });
const INK = "#14141A", RED = "#E8442E", BLUE = "#2B4C8C";
const words = "motley patchwork collage jet set flipnote aviutl one hand many things relations ropes formations zero g falloff stagger oscillator noise duplicator shelf text law".split(" ");
const things = words.map((w, k) => text(w, { name: `W${k}` }).fill([INK, RED, BLUE][k % 3]).font("Helvetica Neue").set("Size", 60));
const room = group(...things).name("Room");
room.set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", 36)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1700).set("Height", 800).set("Position", [110, 100]);
things.forEach((t) => t.set("Position", [0, 0]));
things.forEach((t) => t.effect("Falloff Reveal", { "Sweep": 3.2, "Radius": 360, "Lift": 70, "Shrink": 0.7, "Ease": "Out" }));
text("falloff reveal · one block · a circle sweeps, words rise where it passes", { Position: [960, 1020] }).fill(INK).font("Helvetica Neue").set("Scale", [0.2, 0.2]);
