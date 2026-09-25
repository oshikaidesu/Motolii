// BB theater: two PNG standees on a small stage, a poster on the wall, a rug (shape) on the floor,
// a red subtitle floating in the room that glows (text in 3D), and a flat 2D caption bar.
// World: x right, y down, z into the screen, comp px. Floor at y = 900.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#000000", loop: true });

const room = media("room.hdr", { name: "Room Light" });
room.environment().set("Opacity", 0);

const FLOOR = 900;
const R = 270; // a rectangle's own size
const flat = (layer, hex) => layer.fill(`linear-gradient(90deg, ${hex}, ${hex})`);

// floor and back wall
flat(rectangle({ name: "Floor" }), "#c9b7a0")
  .set("Scale", [5200 / R, 2600 / R]).set("Position", [960, FLOOR]).set("Position Z", 400).set("Tilt X", 90).projection("3D");
flat(rectangle({ name: "Wall" }), "#e9e4da")
  .set("Scale", [3600 / R, 1500 / R]).set("Position", [960, FLOOR - 750]).set("Position Z", 760).projection("3D");
// a skirting board along the wall
flat(rectangle({ name: "Skirting" }), "#8a6f58")
  .set("Scale", [3600 / R, 40 / R]).set("Position", [960, FLOOR - 20]).set("Position Z", 755).projection("3D");

// the left wall: a picture with a window hole (the light comes in through it)
const leftWall = media("left_wall.png", { name: "Left Wall" })
  .set("Scale", [1.3, 1.3]).set("Position", [120, FLOOR - 1100 * 1.3 / 2]).set("Position Z", -150).set("Tilt Y", 90).projection("3D");
leftWall.effect("Cast Shadow", { "Strength": 1 });
// outside, seen through the window: daylight
const outside = media("outside.png", { name: "Outside" })
  .set("Scale", [2.4, 2.4]).set("Position", [-520, 250]).set("Position Z", -150).set("Tilt Y", 90).projection("3D");
outside.effect("Glow", { "Threshold": 0.9, "Intensity": 0.6, "Radius": 12 });
// the ceiling (out of the frame; it keeps the sky out of the room)
flat(rectangle({ name: "Ceiling" }), "#efe9df")
  .set("Scale", [2640 / R, 2400 / R]).set("Position", [1440, FLOOR - 1430]).set("Position Z", -300).set("Tilt X", 90).projection("3D")
  .effect("Cast Shadow", { "Strength": 1 });

// poster on the wall
media("poster.png", { name: "Poster" })
  .set("Scale", [0.9, 0.9]).set("Position", [1010, 360]).set("Position Z", 750).projection("3D");

// rug on the floor (a shape)
ellipse({ name: "Rug" }).fill("radial-gradient(#b8454a, #8f2f3a 70%, #7a2632)")
  .set("Scale", [1500 / 270, 700 / 270]).set("Position", [960, FLOOR - 2]).set("Position Z", 260).set("Tilt X", 90).projection("3D");

// the standees (PNG), standing on the floor
const girl = media("standee_a.png", { name: "Girl" })
  .set("Scale", [0.56, 0.56]).set("Position", [700, FLOOR - 1200 * 0.56 / 2]).set("Position Z", 300).projection("3D");
const boy = media("standee_b.png", { name: "Boy" })
  .set("Scale", [0.56, 0.56]).set("Position", [1230, FLOOR - 1260 * 0.56 / 2]).set("Position Z", 220).projection("3D");
girl.effect("Cast Shadow", { "Strength": 1 });
boy.effect("Cast Shadow", { "Strength": 1 });

// the red subtitle (a Group: a solid fill does not reach 3D text; the text sits left-anchored in it)
// the red subtitle, floating in the room between the two, glowing
const words = text("ほんとうに?", { name: "Red Line Text" }).fill("#ff2a2a").font("Hiragino Sans").set("Size", 96);
const line = group(words).name("Red Line").set("Position", [776, 200]).set("Position Z", 60);
for (const l of [words, line]) l.projection("3D");
line.keys("Position", [[0, [776, 200], "sine.inOut"], [2, [776, 184], "sine.inOut"], [4, [776, 200]]]);
line.effect("Glow", { "Threshold": 0.2, "Intensity": 1.2, "Radius": 30 });
line.effect("Cast Shadow", { "Strength": 1 });

// the caption bar (flat 2D, over the picture)
flat(roundedRectangle({ name: "Caption Bar" }), "#10131a").set("Opacity", 80)
  .set("Scale", [1500 / 270, 130 / 270]).set("Position", [960, 985]).projection("2D");
text("第1話  ひみつの部屋", { name: "Caption" }).fill("#f4f1ea").font("Hiragino Sans")
  .set("Size", 56).set("Position", [960, 985]).projection("2D");

// camera: a slow drift across the room
const BASE = 540 / Math.tan((55 / 2) * Math.PI / 180);
const cam = camera({ name: "Camera" });
cam.set("Center", [0, 0]).set("Target Z", 300).set("Distance", 1.0);
cam.keys("Orbit", [[0, [-6, -9], "sine.inOut"], [4, [-6, 9]]]);
