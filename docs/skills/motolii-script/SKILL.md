---
name: motolii-script
description: Write a Motolii script — a JS file that builds a document with the window's own names (comp, rectangle, text, .set/.key/.effect, Connect/Trace, cuts, shuffle, media). Use when someone wants a picture from a script, or a script fails on a name.
---

# Motolii script (2026-09-19)

A script is JS run once by `run_script` (`motolii/ui/native/src/editor/script.rs:18`) on QuickJS with the prelude
(`motolii/ui/extensions/script/src/prelude.js`). Every call is a window op (`__op`), every name is a name the
window shows. A refused script leaves nothing behind (script.rs:26-28). Budget 20 s (script.rs:10), memory 512 MB.

## The whole contract

```js
comp({ width: 1080, height: 1080, fps: 30, seconds: 4, background: "#14141A" });   // first — layers take the comp's length at creation
const box = rectangle({ name: "R0", Position: [540, 540] }).fill("#E8442E").set("Scale", [56 / 270, 56 / 270]);
const word = text("motley", { name: "W" }).fill("#F0ECE3").font("Helvetica Neue").set("Size", 60);
box.key("Opacity", 0, 0, "power2.out").key("Opacity", 1, 1);                         // key(name, seconds, value, ease)
box.keys("Position", [[0, [0, 540], "Hold"], [0.5, [540, 540], "Bezier"]]);          // [[seconds, value, ease?], ...]
box.effect("Wave", { "Amplitude": 60, "Frequency": 0.5, "Wavelength": 4 });           // effect(Label, { ParamLabel: value })
const room = group(word, box).set("Display", "Flex").set("Flex Wrap", "Wrap").set("Gap", 36)
  .set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 900).set("Height", 600).set("Position", [90, 240]);
[word, box].forEach((t) => t.set("Position", [0, 0]));                               // Flex children sit at [0,0]
line({ name: "L" }).fill("#F0ECE3").set("Connect From", box).set("Connect To", word).set("Line Path", "Curved").set("Stroke Width", 3);
line({ name: "Ring" }).fill("#7FE7D8").set("Connect From", box).set("Trace", "Circle").set("Margin", 24).set("Stroke Width", 1.2);
```

- Constructors (prelude.js:194-203): `text(content, opts)`, `rectangle`, `roundedRectangle`, `ellipse`, `star`, `polygon`, `line`,
  `nullLayer`, `particles`, `camera`, each `(opts)` where `opts.name` names the layer and every other key is `.set(key, value)`.
  `group(...layers)` (205) selects and groups, returns the group. `media(absolutePath, opts)` (169-179) runs `import` + `placeAsset`
  (start 0) for images, video, audio, 3D — the asset is matched by absolute path or trailing file name.
- `.set(name, value)` / `.key(name, seconds, value, ease)` / `.keys(name, list)` (prelude.js:123-127). `name` is the Inspector label;
  a wrong name is refused with the near names (102). Choices are written by their label (`"Wrap"`, `"Fixed"`, `"Circle"`; 36-40),
  layers by the Layer object (41), colours as `"#rrggbb"` / `"#rrggbbaa"` / `[r,g,b,a]` (24-32).
- Layer labels (`motolii/crates/motolii-doc/src/store/names.rs:10-51`): `Anchor`, `Position`, `Position X/Y/Z`, `Scale` (a pair,
  fraction of the created size), `Rotation`, `Tilt X/Y`, `Opacity`, `Depth`, `Skew`, `Size` (shape: `[w, h]` px, names.rs:40 / text
  style: font px, names.rs:67), `Stroke Width`, `Fill`, `Alignment`, `Split`, `Content`.
  A new shape is a square of `round(min(comp w, h) × 0.25)` px (`ui/native/src/editor/create.rs:199-201`) — 270 only for a 1080 short
  side; that is where `Scale = px / 270` comes from. `.set("Size", [w, h])` on a rectangle/ellipse sets pixels directly.
- Group / layout labels (`motolii-doc/src/store/layout.rs:150-208`): `Display` (None / Flex / Grid), `Flex Direction`, `Flex Wrap`,
  `Justify Content`, `Align Items`, `Grid Columns/Rows`, `Gap`, `Padding`, `Horizontal Sizing` / `Vertical Sizing` (Hug / Fill / Fixed),
  `Width`, `Height`, `Background`, `Overflow`, `Stagger`, `Stagger From`. Children: `Position Type`, `Align Self`, `Column Start`, …
  Every thing: `Margin`, `Hardness`, `Heaviness`, `Transition Duration/Easing/Delay`, `Field`, `Position Anchor`, `Position Area` (layout.rs:230-262).
- Lines (`layout.rs:267-279`): `Connect From`, `Connect To`, `From Side`, `To Side`, `Line Path` (Straight / Curved / Elbow / Hang / Rope),
  `Slack`, `Dash`, `Dash Gap`, `Dash Offset`, `Trace` (None / Outline / Handles / Diagonals / Circle / Guides / Grid / Push), `Handle Size`.
  `Connect From` + `Trace` with no `Connect To` traces the one thing (connect.rs:43-57). `Readout` / `Readout Of` put a number in a text.
- `.fill(color)` (143) = select + `applyPalette`; `.font(family)` (144); `.text(content)`, `.name`, `.parent(layer)`, `.blend(mode)`,
  `.clip(on)`, `.projection(kind)`, `.time(start, duration)` seconds.
- `.effect(label, values)` (145-156) matches the catalog label exactly (`__effects` = `known_effects()`, script.rs:54-57), then `.set`s each
  param by its label. Returns an Effect: `.set`, `.key`, `.names()`, `.enabled(on)`, `.whole(true)` (= op `scopeEffect`: the effect lands
  on the group as one picture, 82). Placement effects show grid rows as `"Position X Each"`, `"Rotation Random"` (66-80).
- Ease (44-45, `motolii-doc/src/eval/gsap.rs:8-10,31-111`): the window's kinds (`"Linear"` default, `"Bezier"`, `"Hold"`, `"Elastic"`,
  `"Bounce"`, `"Steps"`) or GSAP strings `power1..4.in/out/inOut`, `sine`, `expo`, `circ`, `back.out(1.7)`, `elastic.out(1, 0.3)`,
  `bounce.out`, `steps(5)`, `none`. Unmapped: `elastic.in/inOut`, `bounce.in/inOut`, `steps(n, true)`, `slow`, `rough`, `CustomEase`.
- `random(seed)()` (48-57) is the only random; `Math.random`, `Date`, `setTimeout`, `requestAnimationFrame` throw (6-10).
- `cuts(bpm, [[layer | [layers], beats], ...])` (213-234): back-to-back `setTiming` on the beat, one slot per layer, returns total seconds.
- `shuffle(textLayer, { seconds, rate, charset, seed, settle })` (241-269): seeded content keys every 1/rate s, settle left to right.

## Gotchas (seen 2026-09-18)

- Text size for layout is `Size` (font px), not `Scale`. `Scale` on a text scales the picture, the Flex slot keeps the font size.
- Children of a Flex/Grid group need `.set("Position", [0, 0])` — their authored Position is added to the slot.
- `comp()` before creating anything: a layer's timing is `comp.duration_frames` at creation (`port.rs:96`, `create.rs:246`). A comp
  shorter than the frames you render (`MOTOLII_LAST`) gives blank frames past the end; growing the comp later does not stretch old layers.
- Rings / strokes: there is no stroke API on filled shapes — a ring is `line()` + `Connect From` + `Trace: Circle` (connect.rs:132-143).
- A traced or connecting line without its own block is not an object: it reads its ends' motion (`render/src/engine/blocks.rs:337-340`).
  Give it a block and it becomes an object that the block moves.
- Effect param labels are the block's `@label` (see `docs/skills/motolii-block/SKILL.md`); dial names cannot be WGSL reserved words
  (`from`, `to`, `catch`) or shelf function names — the shelf rejects the file, `zz_shelf` prints why.
- Rope's stiffness 60 / damping 6 are constants (blocks.rs:500); only `Slack` is a row.
- `effect()` on a group lands on each child by default; `.whole(true)` for a pixel pass on the group as one picture (chain.js).

## Verification

```sh
# script → document (debug ui test; ~12 s)
MOTOLII_SCRIPT=/abs/piece.js MOTOLII_SAVE=/abs/out/shot.rrd cargo test -p motolii-ui --lib -- --ignored script_file
# document → frames (watch profile: release speed + disk shelf). Build once after any Rust change:
cargo build --profile watch -p motolii-render --example zz_watch
MOTOLII_LAST=120 MOTOLII_STEP=1 MOTOLII_SHRINK=2 motolii/target/watch/examples/zz_watch /abs/out/shot.rrd /abs/out/frames
```
`zz_watch` (`motolii-render/examples/zz_watch.rs`) prints `shelf: disk|baked`, writes `NNNN.png` and `sheet.png` (4 thumbs), prints
`shot: N frames in Ns`, then keeps watching the .rrd and vism/ — poll for `shot:` in its log, then `pkill -f zz_watch`.
Sheets: `montage frames/*.png -tile 6x -geometry +4+4 sheet.png` (ImageMagick) · mp4: `ffmpeg -framerate 30 -i frames/%04d.png -pix_fmt yuv420p out.mp4`.
Reach: `motolii/target/watch/examples/zz_reach shot.rrd 0 30 60` → per frame `things moved turned sized tinted values` (zz_reach.rs:12-22).
Loop without cargo: `MOTOLII_SCRIPT=… MOTOLII_OUT=… cargo test -p motolii-ui --lib watch_shot -- --ignored --nocapture` (script.rs:325).
Bundled examples must keep building: `cargo test -p motolii-ui --lib every_example_builds_its_document` (script.rs:274).
