// After "Sync" by ewn / eone (Scenery, made in Cavalry 2.5.2), CC BY — https://scenery.io/scenes/sync-rQysTIq1bYH/
// Sync (after ewn / eone, Scenery, CC BY): two things — black and white — change the box they live in, draw closer,
// and where they finally overlap becomes a box of its own. Built only from boxes, blocks and effects.
comp({ width: 1920, height: 1080, fps: 30, seconds: 16, background: "#E6E6E6" });

const INK = "#151515", PAPER = "#FFFFFF", LIGHT = "#E6E6E6", GREY = "#A6A6A6", PINK = "#F2A6E0", BLUE = "#8FB6F2";
const all = [];
const keep = (l) => { all.push(l.id); return l; };
const D = 270; // the side a new shape is created with
const circle = (name, color, diameter, at) => keep(ellipse({ name })).fill(color).set("Scale", [diameter / D, diameter / D]).set("Position", at);
const box = (name, color, w, h, at) => keep(rectangle({ name })).fill(color).set("Scale", [w / D, h / D]).set("Position", at);
const words = (content, size, at, color = INK, font = "Menlo") => keep(text(content, { name: content })).fill(color).font(font)
  .set("Scale", [size, size]).set("Transform Origin", "Center").set("Position", at);
const during = (layers, a, b) => { for (const l of layers) l.time(a, b - a); return layers; };

// ── 1. typed (0 – 2.4) ─────────────────────────────────────────────────────
const typed = ["<", "<Ru", "<Runn", "<Running", "<Running_co", "<Running_code", "(J", "(Just", "(Just u", "(Just us", "(Just us t", "(Just us two", "(Just us two)"];
typed.forEach((s, i) => during([words(s, i < 6 ? 0.18 : 0.34, [960, 540])], i * 0.18, i === typed.length - 1 ? 2.4 : (i + 1) * 0.18));

// ── 2. letters loose in a circle, on a dark grid (2.4 – 3.7) ─────────────────
{
  const a = 2.4, b = 3.7;
  const ground = box("Dark", INK, 1920, 1080, [960, 540]);
  const columns = box("Grid columns", "#2A2A2A", 2, 1080, [110, 540]);
  columns.effect("Repeater", { "Count": 12, "Position X Each": 155, "Position Y Each": 0 });
  const rows = box("Grid rows", "#2A2A2A", 1920, 2, [960, 70]);
  rows.effect("Repeater", { "Count": 7, "Position X Each": 0, "Position Y Each": 155 });
  const arcs = [circle("Arc left", "#8A8A8A", 900, [-160, 540]), circle("Arc left hole", INK, 890, [-160, 540]), circle("Arc right", "#8A8A8A", 900, [2080, 540]), circle("Arc right hole", INK, 890, [2080, 540])];
  const dots = circle("Dotted ring", PAPER, 7, [960, 540]);
  dots.effect("Repeater", { "Along": "Circle", "Count": 64, "Radius": 300, "Position X Each": 0, "Position Y Each": 0 });
  dots.keys("Rotation", [[a, 0, "Linear"], [b, 40]]);
  const letters = [..."Justustwo"].map((ch, i) => keep(text(ch, { name: `Letter ${ch}${i}` })).fill(INK).font("Helvetica Neue").set("Scale", [0.55, 0.55]));
  const inside = keep(group(...letters)).name("Circle of letters");
  inside.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 460).set("Height", 460)
    .set("Border Radius", 230).set("Background", PAPER).set("Position", [730, 310]);
  const rng = random(11);
  letters.forEach((l) => {
    const from = [170 + rng() * 120, 170 + rng() * 120];
    const v = [(rng() - 0.5) * 900, (rng() - 0.5) * 900];
    l.set("Position Type", "Absolute").keys("Position", [[a, from, "Linear"], [b, [from[0] + v[0], from[1] + v[1]], "Linear"]]);
    l.effect("Bounce", {});
    l.effect("Push Apart", { "Margin": 4 });
  });
  during([ground, columns, rows, ...arcs, dots, inside, ...letters], a, b);
}

// ── 3. the circle in a frame, labels going round (3.7 – 4.4) ─────────────────
{
  const a = 3.7, b = 4.4;
  const ring = circle("Orbit", "#CFCFCF", 820, [960, 540]);
  const ringHole = circle("Orbit hole", LIGHT, 816, [960, 540]);
  const core = circle("Core", PAPER, 520, [960, 540]);
  core.key("Fill", 4.15, PAPER, "Hold").key("Fill", 4.2, INK, "Hold");
  const frame = keep(line({ name: "Frame" })).fill(GREY).set("Connect From", core).set("Trace", "Outline").set("Margin", 20).set("Stroke Width", 2);
  const tags = circle("Tags", INK, 34, [960, 540]);
  tags.effect("Repeater", { "Along": "Circle", "Count": 14, "Radius": 390, "Position X Each": 0, "Position Y Each": 0 });
  tags.keys("Rotation", [[a, 0, "Linear"], [b, 25]]);
  const left = words("Together", 0.16, [240, 540]);
  const right = words("Together", 0.16, [1680, 540]);
  during([ring, ringHole, core, frame, tags, left, right], a, b);
}

// ── 4. the circle cut into a checker of halves (4.4 – 6.2) ─────────────────
{
  const a = 4.4, b = 6.2;
  const cells = [];
  const inks = [[INK, PAPER], [PAPER, INK], [PAPER, INK], [INK, PAPER]];
  for (let i = 0; i < 4; i++) {
    const back = box(`Cell ${i}`, i % 3 === 0 ? PAPER : "#C8C8C8", 260, 260, [0, 0]);
    const half = circle(`Half ${i}`, inks[i][0], 260, [i % 2 === 0 ? 260 : 0, 130]);
    half.clip();
    const cell = keep(group(back, half)).name(`Checker ${i}`);
    cell.set("Display", "Flex").set("Overflow", "Clip").set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill");
    back.set("Position Type", "Absolute").set("Position", [130, 130]);
    half.set("Position Type", "Absolute");
    cells.push(cell, back, half);
  }
  const board = keep(group(...cells.filter((_, k) => k % 3 === 0))).name("Checker");
  board.set("Display", "Grid").set("Grid Columns", 2).set("Grid Rows", 2).set("Gap", 0)
    .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Transform Origin", "Center").set("Position", [960, 540])
    .keys("Width", [[a, 760, "Bezier"], [b, 520]]).keys("Height", [[a, 760, "Bezier"], [b, 520]]).keys("Rotation", [[a, -45, "Bezier"], [a + 0.6, 0, "Hold"]]);
  const bracket = keep(line({ name: "Brackets" })).fill(INK).set("Connect From", board).set("Trace", "Handles").set("Margin", 30).set("Handle Size", 40);
  const rule = box("Rule", "#9A9A9A", 1920, 2, [960, 540]);
  during([rule, board, ...cells, bracket], a, b);
}

// ── 5. a dark pill, two ends tied by waves (6.2 – 7.2) ─────────────────
{
  const a = 6.2, b = 7.2;
  const fans = [circle("Fan left", "#D2D2D2", 900, [120, 540]), circle("Fan left 2", "#DCDCDC", 700, [60, 540]), circle("Fan right", "#EFEFEF", 900, [1800, 540]), circle("Fan right 2", "#F5F5F5", 700, [1860, 540])];
  const pill = keep(roundedRectangle({ name: "Pill" })).fill(INK).set("Scale", [1200 / D, 110 / D]).set("Position", [960, 540]);
  const left = circle("Left end", PINK, 44, [390, 540]);
  const right = circle("Right end", BLUE, 44, [1530, 540]);
  const waves = [[PINK, 2, 26, 0], [BLUE, 3, 18, 0.33], [PAPER, 1, 10, 0.66]].map(([c, f, amp, off], i) => {
    const w = keep(line({ name: `Wave ${i}` })).fill(c).set("Connect From", left).set("Connect To", right).set("Stroke Width", 2);
    w.effect("Oscillator", { "Amplitude": amp, "Frequency": f }).key("Offset", a, off, "Linear").key("Offset", b, off + 2);
    return w;
  });
  during([...fans, pill, ...waves, left, right], a, b);
}

// ── 6. the two in a card (7.2 – 9.4) ─────────────────
{
  const a = 7.2, b = 9.4;
  const halo = [];
  const black = keep(ellipse({ name: "Black" })).fill(INK).set("Scale", [110 / D, 110 / D]);
  const white = keep(ellipse({ name: "White" })).fill(PAPER).set("Scale", [110 / D, 110 / D]);
  const card = keep(group(black, white)).name("Card");
  card.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1000).set("Height", 620)
    .set("Border Radius", 44).set("Background", GREY).set("Position", [460, 230]);
  black.set("Position Type", "Absolute").keys("Position", [[a, [180, 170], "Linear"], [b, [180 + 520, 170 + 300], "Linear"]]);
  white.set("Position Type", "Absolute").keys("Position", [[a, [760, 430], "Linear"], [b, [760 - 610, 430 + 260], "Linear"]]);
  for (const o of [black, white]) { o.effect("Bounce", {}); o.effect("Push Apart", { "Margin": 10 }); }
  halo.push(keep(line({ name: "Card halo" })).fill("#C9C9C9").set("Connect From", card).set("Trace", "Circle").set("Margin", 120).set("Stroke Width", 2));
  // "Between", one letter to a layer in a row; a GPU Wave runs along them.
  const glyphs = [..."Between"].map((ch, i) => keep(text(ch, { name: `Between ${i}` })).fill("#8E6FA8").font("Menlo").set("Position", [0, 0]).set("Scale", [0.2, 0.2]));
  const between = keep(group(...glyphs)).name("Between");
  between.set("Display", "Flex").set("Gap", 2).set("Transform Origin", "Center").set("Position", [960, 540]);
  for (const g of glyphs) g.effect("Wave", { "Amplitude": 8, "Frequency": 1.2, "Wavelength": 5 });
  during([...halo, card, black, white, between, ...glyphs], a, b);
}

// ── 7. the two go round an iridescent ring (9.4 – 10.9) ─────────────────
{
  const a = 9.4, b = 10.9;
  const band = circle("Ring", PAPER, 560, [960, 540]);
  band.effect("4-Color Gradient", { "Color 1": BLUE, "Color 2": "#FFFFFF", "Color 3": "#FFFFFF", "Color 4": PINK, "Blend": 180 });
  band.keys("Scale", [[a, [560 / D, 560 / D], "Bezier"], [b, [300 / D, 300 / D]]]);
  const hole = circle("Ring hole", LIGHT, 300, [960, 540]);
  hole.keys("Scale", [[a, [300 / D, 300 / D], "Bezier"], [b, [40 / D, 40 / D]]]);
  const black = keep(ellipse({ name: "Black on ring" })).fill(INK).set("Scale", [80 / D, 80 / D]);
  const white = keep(ellipse({ name: "White on ring" })).fill(PAPER).set("Scale", [80 / D, 80 / D]);
  const track = keep(group(black, white)).name("Track");
  track.set("Display", "Flex").set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Border Radius", 1000)
    .set("Transform Origin", "Center").set("Position", [960, 540])
    .keys("Width", [[a, 440, "Bezier"], [b, 170]]).keys("Height", [[a, 440, "Bezier"], [b, 170]]);
  black.set("Position Type", "Absolute").set("Offset Path", "Border Box").keys("Offset Distance", [[a, 0, "Linear"], [b, 90]]);
  white.set("Position Type", "Absolute").set("Offset Path", "Border Box").keys("Offset Distance", [[a, 50, "Linear"], [b, 140]]);
  during([band, hole, track, black, white], a, b);
}

// ── 8. one circle, half and half, falling with a trail (10.9 – 11.9) ─────────
{
  const a = 10.9, b = 11.9;
  const disc = circle("Half disc", PAPER, 130, [960, 260]);
  const shade = box("Half shade", INK, 140, 70, [960, 260]);
  shade.set("Transform Origin", "Bottom").clip();
  for (const l of [disc, shade]) l.keys("Position", [[a, [960, 260], "Bezier"], [b, [960, 700]]]).keys("Rotation", [[a, 0, "Linear"], [b, 200]]);
  const spin = disc;
  const ghosts = [0, 1, 2, 3].map((i) => {
    const g = circle(`Trail ${i}`, "#BDBDBD", 132, [960, 260]);
    const g2 = circle(`Trail hole ${i}`, LIGHT, 128, [960, 260]);
    for (const l of [g, g2]) l.keys("Position", [[a + 0.08 * (i + 1), [960, 260], "Bezier"], [b + 0.08 * (i + 1), [960, 700]]]);
    return [g, g2];
  }).flat();
  during([...ghosts, disc, shade], a, b);
}

// ── 9 / 10. the two close in, and where they overlap becomes the box (11.9 – 16) ─────────
{
  const a = 11.9, meet = 13.6, b = 16.0;
  const rule = box("Horizon", "#CFCFCF", 1920, 2, [960, 540]);
  rule.keys("Opacity", [[a, 1, "Linear"], [meet, 0, "Hold"]]);
  const bottom = circle("White sun", PAPER, 900, [960, 1500]);
  bottom.keys("Position", [[a, [960, 1500], "Bezier"], [meet, [960, 900], "Bezier"], [b, [960, 830]]]);
  const top = circle("Black sun", INK, 900, [960, -420]);
  top.keys("Position", [[a, [960, -420], "Bezier"], [meet, [960, 180], "Bezier"], [b, [960, 250]]]);
  // The lens: the white sun again, painted, clipped to the black sun below it.
  const lens = circle("Lens", PAPER, 900, [960, 1500]);
  lens.keys("Position", [[a, [960, 1500], "Bezier"], [meet, [960, 900], "Bezier"], [b, [960, 830]]]);
  lens.effect("4-Color Gradient", { "Color 1": "#9DB8F5", "Color 2": "#F39AD9", "Color 3": "#F7B6E6", "Color 4": "#A9C4F7", "Blend": 90 });
  lens.effect("Inner Shadow", { "Opacity": 35, "Distance": 6, "Size": 30, "Color": "#FFFFFF" });
  lens.clip();
  // The overlap is a box: the word sits where the two meet, and follows them as they close in.
  const sync = words("Sync", 0.55, [0, 0], "#FFFFFF", "Snell Roundhand")
    .set("Position Anchor", top).set("Position Anchor 2", bottom).set("Position Area", "Center");
  sync.keys("Opacity", [[meet, 0, "Linear"], [meet + 0.6, 1]]);
  const together = words("T o g e t h e r", 0.12, [0, 0], "#5B4A6A")
    .set("Position Anchor", sync).set("Position Area", "Bottom").set("Margin", 6);
  together.keys("Opacity", [[meet + 0.3, 0, "Linear"], [meet + 0.9, 1]]);
  during([rule, bottom, top, lens, sync, together], a, b);
}

op("setAttrs", { layers: all, patch: { projection: "2D" } });
