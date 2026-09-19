---
name: motolii-doc
description: Change the document side of Motolii (crates/motolii-doc) — layout rows, placement effects (Repeater, Mirror), connecting/tracing lines, planes and draw order, anchor/follow. Use when a new row, a new placement kind, or a new line path is wanted, or a doc test is red.
---

# Motolii document (2026-09-19)

`motolii/crates/motolii-doc` is the contract: it holds the edited state and evaluates it into `ResolvedLayer`s. It never draws.
The render crate reads `StoreView`; the UI writes through `Intent` only.

## Contract

- **Writes are Intents.** `Document::apply(Intent)` (`src/store/document.rs:425`); the enum is at `document.rs:30` (`AddLayer`,
  `SetTrack`, `SetConstant`, `SetEffects`, `SetMeta`, `SetComposition`, …). Window edits go through `Document::place` (489) /
  `place_checked` (`document/edit.rs:35`), which turn a value + time + Animate into Intents. Reads go through `doc.view()` (269).
- **Rows are tables, not code.** A layout row is `(property, "Window Label", default, range, choices)` — `layout::Row`
  (`src/store/layout.rs:145`). Tables: `GROUP_ROWS` (150), `ITEM_ROWS` (185), `SPACE_ROWS` (230), `CONNECT_ROWS` (267), `READOUT_ROWS`
  (283). `layout::row(property)` (297) chains them; `names::label` (`names.rs:87`) resolves any property to its label, fixed transform
  names at `names.rs:10-51`. Add a row = add one tuple; the Inspector and the script read the same table.
- **Placement effects** (`src/store/placement.rs`): an effect that returns placements, not pixels. `PlacementKind { plugin_id, label,
  params, shape, grid }` (22-29); `KINDS` (77-124) has `REPEAT = "motolii.repeat"` (Line / Circle / Grid, Each and Random columns,
  `Pick`, `Transform` Each/Whole) and `MIRROR = "motolii.mirror"` (Axis Horizontal / Vertical / Both / Radial, `Segments`, `Centre`).
  `placements(kind, params) -> Vec<Placement>` (185) is a pure function of `(params, seed)`; `mirrors` (250) is `x ↦ R(θ) S (x − c) + c`
  written as `offset = c − R S c`, `stretch = ±1`. `Placement` (151-163): `index`, `offset`, `rotation_degrees`, `scale`, `offset_z`
  (2.5D/3D only, via `affine3` 174), `opacity`, `time_offset` (a copy shows `t − time_offset`), `stretch` (Blob Track / Mirror).
  `picks` (281) hands a group's children out per placement (Random by `share.<id>` weight, or Iterate).
  Registration is automatic: `kind::all()` (`kind.rs:88`) chains `placement::KINDS` into the shelf; `resolve.rs:896-920` expands the
  first placement effect on a layer; `resolve.rs:1120-1128` hides a placed group's children; `ui/native/src/snapshot.rs:456` builds the
  grid rows from `PlacementKind::grid`.
- **Connecting and tracing lines** (`src/store/connect.rs`): a shape layer with `Connect From` + `Connect To` has its outline replaced by a
  route solved each frame from the two boxes (`connect_shapes` 60); `connection` (27) and `tracing` (43) are the two questions. `route_at`
  (307-345) switches on `Line Path`: 0 Straight, 1 Curved (cubic, 0.4 × distance), 2 Elbow, 3 Hang (CPU catenary, `hang` 377,
  length = distance × (1 + Slack %)), 4 Rope (one cubic, bellies from `rope_controls` 349 = 1/3, 2/3 along, dropped by Slack % × distance;
  ends are box centres, no socket, no Margin — the render side moves the bellies on the GPU). `socket` (355) picks the edge point for
  From/To Side. `trace_path` (113) draws around one thing: Outline, Handles, Diagonals, Circle (4 cubics), Guides (to comp edges), Grid,
  Push (the pushed-from box + arrow). A lit loop is cut by the thread-local `ROUTING` set (route 296-305).
- **Planes and draw order** (`src/store/view/resolve.rs`): `put_on_planes` (1356) gives every layer the plane of the outermost Display group
  it sits flat in; `put_connectors_in_front` (1402) makes a connector/trace inherit its ends' plane and draw at `max(order) + 1`, so a line
  in a 2.5D room is not swallowed by the room background; `put_backgrounds_behind` (1423).
- **Anchor / follow** (`layout.rs:479-560`): `Position Anchor` (+ `Position Anchor 2`) with `Position Area ≠ None` places a thing on a side of
  another thing's box (`anchored`, `anchored_inner`); the shift is added to the layout nudge (`nudge` 581-586). On the GPU the same pair
  becomes `follows` and a `FollowPass` adds the target's translate (render `blocks.rs:420-437, 700-702`). Loops are cut by `ANCHORING`.
- Layout pushes (`layout_frame`), transitions (`transition_samples`) and Overflow = Bounce (`bounced` 213) are all pure functions of time.

## Adding a placement kind (mirror the Mirror commit)

1. `placement.rs`: a `pub const YOURS: &str = "motolii.yours"` and choice constants; a `PlacementKind` in `KINDS` with `params` built by
   `number` / `vec2` (68-74) or an explicit `PlacementParam` for a `Choice`; `shape` lists the params that toggle with the shape; `grid`
   rows name real params (test `only_the_chosen_shape_shows_its_fields` checks this).
2. `placements()`: branch on `kind.plugin_id` before the Repeater path (like `MIRROR` at 203-205) and return `Vec<Placement>` — fill every
   field; `stretch = [1, 1]` unless you flip; `time_offset = RationalTime::ZERO` unless you delay.
3. Tests in `mod pure_function_contract` (329): copy the shape of `a_horizontal_mirror_reflects_…` (440) and
   `both_gives_the_four_quadrants_…` (453): assert copy 0 is `Affine2::IDENTITY` about a pivot, assert transformed points, assert
   `shown(mode)` for shape-only params, assert `mirror(&[]) == mirror(&[])` (pure). Use `affine2(pivot)` / `affine3` for geometry.
4. Nothing to add in render for offset/rotation/scale/stretch/opacity/offset_z — `resolve.rs` already maps them. A new `Placement` field
   would need `resolve.rs:907-960` and the render `placement.z`/flat checks (`compositor.rs:143`, `blocks.rs:523`).
5. Script: `layer.effect("Yours", { "Axis": "Radial", "Segments": 8 })` (labels). `expandEffect` (`ui/native/src/editor/placement_edit.rs`)
   does not read `stretch` — baking a flipped copy drops the flip (known, UI side).

## Gotchas

- `Value::LayerId(0)` means "no layer"; readers also accept `Value::F64(v >= 1)` (connect.rs:29-33) — keep both when adding a layer row.
- A layer counts as present only if `here(layer, t)` (`layout.rs:853`); a connector to an absent end returns `None`, not a stub line.
- Rope's ends are centres (connect.rs:338-344) while Straight/Curved/Elbow/Hang use sockets + Margin — do not "fix" one to match the other.
- Placement `scale_each` is percent compounded per copy (`(1 + s/100)^i`, 236); `opacity_each` is added per copy (239).

## Verification

```sh
cargo test -p motolii-doc                         # from the repo root (the workspace is /Users/member_ottoto/rust_ae/Motolii/Cargo.toml)
cargo test -p motolii-doc placement               # the pure-function contract
cargo test -p motolii-doc connect                 # a_line_joins_two_boxes_edge_to_edge_and_follows_them, a_hanging_rope_keeps_its_length_and_sags_down
cargo test -p motolii-doc --test edit_transactions
```
Then the render side must still agree: `cargo test -p motolii-render --lib blocks::tests::a_connector_follows_what_a_block_moved` and the
script examples `cargo test -p motolii-ui --lib every_example_builds_its_document`.
