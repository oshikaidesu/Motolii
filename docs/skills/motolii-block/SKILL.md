---
name: motolii-block
description: Write a Motolii block — one WGSL text file on the shelf (vism/) that moves, sizes, tints and fades many things from one rule. No Rust, no build. Use when someone wants a per-thing law (wave, arrive, push apart, radial, pulse) that a document can dial.
---

# Motolii block (下書き 2026-09-18)

A block is one file in `motolii/crates/motolii-render/vism/*.wgsl`. The document holds only the dials; the shelf holds the rule.
Save the file and the next frame uses it (disk shelf, debug and `watch` profiles). Nothing else to touch.

## The whole contract

```wgsl
/*{
  "ID": "you.your_block", "LABEL": "Your Block", "STAGE": "block",
  "DESCRIPTION": "One sentence: what one hand does to many things.",
  "INPUTS": [ { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0 } ]
}*/
fn block(k: u32, p: BlockParams) -> Offset {
    return Offset(translate, rotate_degrees, scale, tint);   // what to ADD to thing k this frame
}
```

- `k` — the thing's index in the order the effect was applied. Pure function of `k` and `host.time`: no state, no memory.
- `p` — one `f32` per INPUT, in manifest order, named by `NAME` (only `float` / `long` / `bool`, up to 24).
- `Offset` — four components, composed with the things' current state:
  - `translate: vec2f` px, **added**
  - `rotate: f32` degrees, **added**
  - `scale: f32` factor, **multiplied** (1.0 = unchanged)
  - `tint: vec4f` rgb factor + opacity factor, **multiplied** (`vec4f(1.0)` = unchanged)
- `NO_OFFSET` — `Offset(vec2f(0.0), 0.0, 1.0, vec4f(1.0))`. Return it when the thing has settled.

## What you can read

- `host.time` seconds · `host.members` how many things carry this effect · `host.objects` all things · `host.round` · `host.source` (fields only)
- `objects[k]` — `lo`, `hi` (the thing's box, comp px), `room_lo`, `room_size` (the box it lives in), `radius`, `group`, `margin`, `weight`
- `state_in[k]` — the thing's current offset before your block; `now_lo(k)` / `now_hi(k)` — its box after earlier blocks
- `neighbor_count(k)`, `neighbor(k, i)` — nearby things of the same group (a grid, not all pairs; widen with `"REACH"`)

## Manifest keys beyond INPUTS

- `"ROUNDS": n` — run the block n times per frame, each reading the previous round (push-apart style relaxation).
- `"REACH": "<input name>"` — the input that says how far outside its box a thing looks for neighbours.
- `"SCOPE": "room"` — the thing with the effect is the *source*; everyone else in its room moves (fields: gravity, wind, wells).
- `"PHYSICS": { ... }` — solver dials for room fields (read by the physics side; not needed for a plain block).

## The function shelf (引用のルール)

Files in `vism/` whose name starts with `_` are not blocks: they are libraries, prepended to every block in name order.
The shelf is a WESL package (wesl-lang.dev; `wesl` 0.4.2 links it at load time, no mangling). A `.wgsl` file without the
`/*{ ... }*/` head is a module named by its file stem; a block pulls what it uses with WESL imports written right after the
head: `import package::cavalry::{ cv_ease, cv_stagger };`. Modules: `processing` (Processing's names — `map`, `norm`,
`random`, `noise`, `TWO_PI`; what WGSL already has is not repeated: `mix` is lerp, `clamp` is constrain, `distance` is dist),
`cavalry` (Cavalry's nodes as functions — `cv_random`, `cv_noise`, `cv_oscillator`, `cv_ease`, `cv_stagger`, `cv_falloff`,
`cv_range`, `cv_grid`, `cv_circle`, `cv_line`, `cv_spiral`), `effectors` (below), and `motolii` (the prelude: `Offset`,
`Item`, `now_lo`, `now_hi`, the bindings — a block gets it without writing the import; a module that needs it writes
`import package::motolii::{ Offset, now_lo };`). Import; do not copy. Fix a function in a module and every block that imports
it gets the fix. Nodes with memory (Lerp, Trails, Dynamics) are not here — they belong to the solver. Wildcard imports are not
in wesl 0.4; name the items. WGSL reserved words (`from`, `to`, `catch`, …) cannot be names; the shelf tells you (`zz_shelf`
prints `rejected: <file>: <reason>`).

The `effectors` module is the Effector of Notch / Unreal Motion Design / MoGraph (survey 2026-09-18): the law
says *what* happens, a shape says *where*. `ef_apply(offset, w)` pulls an Offset toward none by weight `w`;
`ef_sphere`, `ef_box`, `ef_plane`, `ef_step`, `ef_noise` give the weight from a position or index; `ef_seed` and `ef_age`
are Unity VFX's per-particle seed and age as pure functions. A block ends with `return ef_apply(law, ef_box(...) * strength);`.
Rotate in an Offset is in degrees, and tint multiplies (values above 1 brighten).

Relation chain: blocks on one layer run in list order within the frame, and each sees the previous ones' result through
`state_in[k]` / `now_lo(k)` / `now_centre(k)`. Judge shapes by `now_centre`, not `objects[k].lo`, when order should matter.
`neighbor(k, i)` lists things within the `REACH` input (built from the rest boxes); read `state_in[j]` of a neighbour to
catch what earlier blocks did to it (`neighbour_effector.wgsl`). `catch` is a reserved word.

## Rules that keep it a block

- Time is the only clock. Everything is `f(k, host.time, inputs, objects)`. If you need yesterday's value, it is not a block (that is the solver's job).
- One rule, many things. If you find yourself special-casing `k == 3`, put that in the document instead.
- Land. Motion that never settles reads as noise; return `NO_OFFSET` (or a decayed offset) once it has arrived.
- Names on the dials are the Web's and AE's words (Amplitude, Frequency, Scale, Opacity, Spread, Stagger). Do not invent.

## Examples on the shelf

Packages (one Cavalry / three.js picture each, all calling the function shelf — read these first):
- `formations.wgsl` — table → circle → helix → grid on the clock (three.js periodic table)
- `concentrick.wgsl` — rings step their radius, breathe with a stagger (Cavalry Concentrick)
- `optical_art.wgsl` — bars swing alternately, staggered, one oscillator (Cavalry Optical Art)
- `ring_ting.wgsl` — things on a ring, a noise bulge where a falloff spot walks (Cavalry Ring Ting)
- `mazin.wgsl` — 10 PRINT maze, strokes turn 90° where a band sweeps (Cavalry Mazin)
- `falloff_reveal.wgsl` — a circle sweeps, things rise where it passes (Cavalry Falloff entrance)
- `zero_gravity.wgsl` — each thing drifts on its own slow course, tumbles, breathes

Laws:
- `arrive.wgsl` — from a side, land in the layout's slot, bounce, stagger, spin
- `wave.wgsl` — one sine wave along the index; Amplitude moves, Scale breathes size, Opacity breathes alpha
- `push_apart.wgsl` — neighbours only, 32 rounds, REACH = margin
- `field.wgsl` — SCOPE room, PHYSICS dials
- `bounce.wgsl`, `hang.wgsl`

## Check it

```sh
cargo run -p motolii-render --example zz_shelf      # prints "rejected: ..." if the manifest or WGSL fails (naga)
```
A script dials it with `layer.effect("Your Block", { "Amount": 2 })` (labels, not names). Blocks compose in effect order.
Script gotchas seen today: text size for layout is `Size`, not `Scale`; children of a Flex group need `Position [0,0]`;
rings are `line()` + `Connect From` + `Trace: Circle` (there is no stroke API on filled shapes); a traced or connecting
line that carries its own block is an object (moved by the block), otherwise it follows what it connects.
