---
name: motolii-render-port
description: Open a port on the render side of Motolii — what a block can read or write, how objects/connectors/ropes reach the GPU motion buffer, the rerun fork's shaders, WESL modules, the tests that must stay green. Use when a shelf (vism/) change is not enough and Rust or the fork must move.
---

# Motolii render port (2026-09-19)

Rule of the house: the shelf (`motolii/crates/motolii-render/vism/*.wgsl`) is where laws live; Rust changes only when a new *kind* of thing
must reach the GPU. Before writing Rust, check the block contract (`docs/skills/motolii-block/SKILL.md`) covers it.

## The pipe, in order (one frame)

1. `prepare_blocks` (`src/engine/blocks.rs:226`) — from `ResolvedLayer`s decide the **objects**: a layer is wanted if it carries a block
   effect, sits in a field room, is an end of a connector/trace (`needed`, 316-324), or anchors (`follows`, 420-437). Connecting/tracing lines
   without their own block are *not* objects (337-340). Each object gets a **slot** `k` (463), a `BlockItem` (464-473: `lo hi room_lo
   room_size radius group margin weight`), an outline, a basis, and its blocks are grouped into **batches** — same `stage` (position in
   the effect list), same plugin, same params (480-489). Connectors become `connectors: (layer, a, b)` (502), `Line Path == Rope` adds
   `ropes: (index, slack, 60.0, 6.0)` (500), traces map to the target's slot (507-508).
2. `attach_block` (551) — writes the motion index into the last shader param slot: object `k + 1`, connector `objects + 2c + 1`, trace
   `target + 1` (557-575, `surface_program::PARAM_SLOTS - 1`, `PARAM_SLOTS = 24`). A rope sets `grid_hint = 48` (560).
3. `run_blocks` (661) — `BlockWorld::begin_from` (block_program.rs:580) uploads items, seeds state from Rapier offsets, builds the
   **neighbour grid** (`neighbors` 78: per group, cells of 2 × (largest box + 2·max(margin, reach)), 3×3 lookup; `reach` = the largest
   value of the `@reach` input over all batches, blocks.rs:677-681). Then stages run in order (685-696): every batch's `BlockProgram`
   dispatches `motolii_block_main`, `state_out = state_in ⊕ block(k)` (add translate/rotate, multiply scale/tint; module_source 203-209),
   `@rounds` repeats it. Then `FollowPass` (616-670: follower.translate += target.translate), `WorldPass` (672-760: state → motion), and
   `RopePass` (762-888: spring-damped bellies, ping-pong state, reset when the frame is not `last_frame + 1`).
4. `MotionBuffer::new(ctx, objects + 2 × links)` (blocks.rs:704; fork `view_builder.rs:323-335`, 64 bytes per entry, zeroed) is handed to
   the compositor; the fork reads it at `@group(0) @binding(11)` (`global_bindings.wgsl:91-92`).

## The block program (what the author's file becomes)

- `PRELUDE` (block_program.rs:114-129) is the WESL module `motolii`: `Item`, `Offset { translate, rotate, scale, tint }`, `BlockHost { time,
  members, objects, round, source }`, bindings 0-7 (`objects`, `state_in`, `host`, `block_params: array<vec4f, 6>`, `state_out`, `members`,
  `neighbor_starts`, `neighbor_list`), `NO_OFFSET`, `neighbor_count/neighbor`, `now_lo/now_hi`. Add a readable value here — and to
  `BlockItem` + `ITEM_BYTES` (56 since 2026-09-19: `parent_slot`, `anchor_slot`) if it is per object — and every block sees it.
- Manifest: JSON head `/*{ "ID", "STAGE": "block", "INPUTS": [...] }*/` with `fn block(k, p: BlockParams)` (legacy), or the override form
  read by `wgsl_manifest` (321-430): attributes `@id @label @description @rounds @scope @physics` on the file, `@label @range @options @reach`
  on `override name: f32|u32|i32|bool = init;` (max 24, `PARAM_SLOTS`), rewritten to `var<private>` and assigned each frame by
  `motolii_block_inputs()` (module_source 166-222). `catalog.rs:279-281` decides "block" by `fn block(`; a `.wgsl` with neither head nor
  `fn block(` nor `mainImage` is a **module** named by its stem (`is_module` 270), collected by catalog.rs:532 and handed to wesl's `VirtualResolver` (block_program.rs:213-220), then
  imported with `import package::<stem>::{ … };`. Modules today: `processing`, `cavalry`, `effectors` (the `_`-prefix rule from the
  9-18 note no longer exists in code). wesl is pinned `=0.4.2` (`motolii-render/Cargo.toml:45`), Rust stable 1.96 (`rust-toolchain.toml`).
- Manifest fields the engine reads: `stage`, `rounds`, `reach` (input name), `scope` (`members` / `room`), `physics` (`isf/mod.rs:196-223`).
- The shelf is read from disk when `load_shaders_from_disk` is set (`catalog.rs:187`; `.cargo/config.toml` `IS_IN_RERUN_WORKSPACE=1` +
  debug assertions), i.e. `dev`, `test` and the `watch` profile (`Cargo.toml:103-105`); `--release` bakes it.

## The fork (`~/rust_ae/rerun-s2-seam-20260818`, crates/viewer/re_renderer)

- `shader/global_bindings.wgsl:85-157`: four vec4 per entry — `(offset.xyz, turn rad)`, `(centre.xyz, scale)`, `(axis.xyz, kind)`,
  `(tint.rgb, opacity)`. `motion_offset(slot, world_pos)` (109): kind 0 = turn about axis through centre, scale, offset; kind 1 = connector
  (`offset at A`, `A`, `B − A`, `offset at B`; a vertex blends the two ends along A→B); kind 2 = rope over **two** entries, the second holding
  `c0/c1 drawn` and `c0/c1 now`, vertex moves by `live cubic − drawn cubic`. `motion_tint` (151) is read by fragments.
- `shader/rectangle_vs.wgsl:8` and `rectangle_grid_vs.wgsl` read the slot from `rect_info.surface_params[5].w` (= param 23); the grid vertex
  stage makes `field_grid²` cells × 6 vertices. `src/renderer/rectangles.rs:709-711, 888-912, 947-954`: `grid_pipelines` are chosen when
  `field_grid > 1` and no surface program — `field_grid` comes from `SurfaceShading::field_grid()` (`surface_program.rs:38-43`).
- A new motion kind = a branch in `motion_offset`, a writer pass in `block_program.rs`, and the entry count in `blocks.rs:704`.
- Pin update: commit in the fork, `git -C ~/rust_ae/rerun-s2-seam-20260818 push`, then replace the 15 `rev = "<old>"` lines in the root
  `Cargo.toml` (lines 28-76, all `file:///…/rerun-s2-seam-20260818`) with `sed -i '' 's/<old>/<new>/g' Cargo.toml`; `re_mp4` has its own
  fork line (`Cargo.toml:109`). Confirm with `git -C ~/rust_ae/rerun-s2-seam-20260818 log --oneline -1`.

## Tests that must stay green

```sh
cargo test -p motolii-render --lib block_program      # 8: bounce folds, scale/tint compose, override manifest, world pass, rope pass, shelf packages
cargo test -p motolii-render --lib catalog            # library import, override card, shadertoy paste, refresh keeps device, character oracle
cargo test -p motolii-render --lib blocks::tests      # connector follows, field moves room, chain in effect order, wave order, push_apart ≈ all pairs, reach finds neighbours
cargo run -p motolii-render --example zz_shelf        # prints `rejected: <file>: <reason>` for every card the shelf refuses
```
Known: `catalog::character_oracle` (catalog.rs:773) can fail when GPU tests run in parallel — rerun it alone (`-- --test-threads=1`).
`blocks::tests::things_stop_moving_once_they_have_settled` and the two `a_field_moves_*` have the same parallel flake (9-17 note).
`every_example_builds_its_document` (ui) fails on `web_c8_ink_settle.js` independent of render changes.

## After any Rust change

Rebuild every watch binary you use, or you measure old code: `cargo build --profile watch -p motolii-render --example zz_watch --example
zz_reach --example zz_shelf` (target dir `motolii/target/watch/examples/`). Then re-shoot one known picture (e.g.
`docs/reviews/2026-09-18-evidence/relation_chain.js`) and compare frame 90 byte-for-byte with the sheet you had before.
