// Codrops #5 — OneElementScroll (js/index.js): one picture slides from frame to frame as the page scrolls (Flip to each [data-step]).
// Source: states = stepElements.map(el => Flip.getState(el)); tl.add(Flip.fit(oneElement, state, {duration: 1, ease: 'sine.inOut'}), '+=0.5')
// scrollTrigger {scrub: true, start: 'clamp(center center)'}; gsap.fromTo(oneElement, {filter:'brightness(80%)'}, {filter:'brightness(100%)'}).
// Scroll = time: the page (a 9-row grid) moves up; the one element's cell is switched with Hold keys and its Transition carries it (FLIP).
comp({ width: 1920, height: 1080, fps: 30, seconds: 5, background: "#101014" });
const MAT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/cb607ba7-2ce9-4b35-8d8e-46a99d824596/scratchpad/mat/";
// The frames the element visits: [column start, column span, row start, row span] in a 12 × 9 grid of 160 × 300 cells.
const steps = [[2, 4, 1, 2], [7, 5, 4, 3], [1, 12, 7, 3]];
const frames = steps.map((s, i) => rectangle({ name: `Step ${i + 1}` }).fill("#23232b"));
const one = media(MAT + "photo1.png", { name: "One" });
const page = group(...frames, one).name("Page");
page.set("Display", "Grid").set("Grid Columns", 12).set("Grid Rows", 9).set("Gap", 20).set("Padding", [40, 40])
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1920).set("Height", 2700).set("Position", [0, 0]);
const place = (l, [c, cs, r, rs]) => l.set("Column Start", c).set("Column Span", cs).set("Row Start", r).set("Row Span", rs);
frames.forEach((f, i) => place(f.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill"), steps[i]));
place(one.set("Position", [0, 0]).set("Horizontal Sizing", "Fill").set("Vertical Sizing", "Fill"), steps[0]);
one.set("Transition Duration", 1).set("Transition Easing", "Ease In Out");
// Each step is a Hold key on the cell (0.5 s gap after each 1 s flip, as in the timeline).
for (const [name, k] of [["Column Start", 0], ["Column Span", 1], ["Row Start", 2], ["Row Span", 3]])
  one.keys(name, steps.map((s, i) => [Math.max(0, i * 1.5 - 0.5), s[k], "Hold"]));
one.effect("Gain", { "Gain": 0.8 }).key("Gain", 0.8, 0.8, "Linear").key("Gain", 3.5, 1);
// Scroll: the page moves up so each frame is centred when the element lands in it.
page.keys("Position", [[0.8, [0, 0], "Linear"], [2.0, [0, -810], "Linear"], [3.5, [0, -1710]]]);
for (const layer of [page, ...frames, one]) layer.projection("2D");
