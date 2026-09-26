# Members: Host grammar probe (2026-09-27, working note)

Hypothesis: Create = what the host must understand (members, order, identity, time); Effects = picture in, picture out (WGSL).
Grammar under test: Source → Members → Select → Relate/Place → Vary → Time.

## What already existed

| Concept | Where | Members it serves |
|---|---|---|
| Copies | `extensions/placement.rs` (Repeater, Mirror): a pure function of params to `Placement` records (index, offset, rotation, scale, Z, opacity, time_offset) | copies of a layer; on a group, one child picked per copy (Pick: Random by `share.<id>`, Iterate) |
| Split units | `resolve/copies.rs` `push_split`: the whole text resolved at a shifted time and cut by the unit's box | a text's chars, words, lines |
| Children | real layers; `layout/time.rs` `layer_time` shifts a child's keys by the parent's order rows | a group's children |
| Order to time | `schedule_shift` in the document: "the same law for a container's children and a text's split units" (GSAP stagger amount + from) | children, units |
| Place | Repeater Line / Circle / Grid (index arithmetic); container Layout flex / grid (box solving); Offset Path | copies; children |
| Vary | Repeater Each (per index) and Random (per seed) for position, Z, rotation, scale, opacity, delay | copies only |
| Spatial weight | `layout.field`: a 0 to 1 strength from a box, scaling, fading and pushing each copy | copies (and layers) |
| Timeline | a Repeater is an effect row on its layer; no member appears as a layer | all |

None of these makes a layer. Members are found, never stored.

## Changed (production)

1. Owner of shelf entries is the host's: `owner_of(stage)` names the placement seat `host`. Status `hostCapabilities`; Create lists Repeater and Mirror under Copies and applies them with `applyEffect`, as before. Effects lists only `effect` entries. Document, render and Script (`layer.effect("Repeater")`) unchanged.
2. One order law, three sources: `schedule_delay` = Select (`order_weight`, 0 to 1 by Start / Center / End / Edges) × Stagger, used by `schedule_shift` (children, units) and by copies in both the resolve path and the frame graph. A layer with a Repeater is offered the same three order rows; so is a plain group (its children were already shifted, the rows were hidden). Default 0: documents render as before.
3. Status `members` per layer: `{kind: copies | children | chars | words | lines, count, by?}`. Script: `layer.members()` and `layer.stagger(seconds, from)`.

## Verdicts

| Step | Shared across sources? |
|---|---|
| Members | yes, as a projection: copies, children, units are all "the layer, addressed by index, resolved at a member time"; status names them |
| Select | yes for order: `order_weight` is a Range Selector over index, already weights, not booleans. `layout.field` is a second, spatial selector over copies. No boolean Pick over members exists (Repeater Pick chooses a source child per copy, it selects sources, not members) |
| Relate / Place | not shared: Repeater places by index arithmetic, Layout by solving boxes. Both produce per-member offsets; there is no second example of the same boundary yet, so no abstraction |
| Vary | copies only. Each / Random are pure index and seed functions and could serve units and children; no second source uses them yet |
| Time | yes in the document: one law for three sources |

## Time (2026-09-28): evaluation time is an input to value evaluation

The gap found on 09-27 is closed without teaching the renderer about Stagger.

Time paths found:
| Path | Global time read | Mapped to local | Sampled |
|---|---|---|---|
| `value_at` (document, resolve path) | caller's t | `layer_time` (ancestors' order shifts) then `looped_time` (Loop) | the base track, then modulators at mapped time + offset (source layer maps its own) |
| copies (`push_placements`, graph placement set) | frame t | t − copy offset (Delay Each, Random, order delay) | the layer at that time, through `value_at` / the graph |
| text units (`push_split`) | frame t | `schedule_shift` of the text's own order rows | the whole text at the unit time, cut to the unit box |
| frame graph property nodes | frame t | none (before) | `track.eval(context.time)` |
| physics blocks | frame t | `layer_time` | own solver |

Boundary: `LayerClock` (document, `layout/clock.rs`) is a layer's time mapping as data: the ancestors' order steps (outermost first) and the layer's Loop rows, each row a constant or a track. `value_at`, `layer_time` and `looped_time` read it; the graph's `PropertyClock` node samples a property's value node at `clock.for_row(row, t)` through the graph's existing dynamic-input path. Proof: the clock reads the old mapping's times frame by frame (`tests/layer_clock.rs`).

Composition order, as the code defines it:
1. global frame time
2. less the member's own delay (a copy's offset, a text unit's order delay, clamped at 0)
3. each ancestor's order shift, outermost first (each clamped at 0)
4. the layer's Loop fold (Loop rows are read at the ordered time shifted once more, as `looped_time` read them)
5. the value, and any modulator at that time plus its offset (the source layer maps its own)

A group's own Loop does not fold its children (existing: `layer_time` walks order shifts only).

Live parity (runtime tests on the live semantic scene, the one the pixels are drawn from): copies, children of a plain group, words of a split text, Loop, and a nested case (a staggered group holding a split staggered text and a shape with a staggered Repeater and Delay Each), member by member, with Undo.

One difference left: a Stagger or Loop row driven by a link. The document keeps reading such rows through `value_at` (links included); the graph's clock leaves links out. Links on these rows are only reachable by pasting a layer that already has one.

Performance: a layer with no order rows above it and no Loop gets no new node. A clocked property adds one node, sampled once more when its time differs. Units are sampled like copies, O(units). Clocks are built once per view and revision.

## Fragments

Split units now exist in both render paths as "the whole layer at a member time, cut to a region, turned about the region's centre". A Fragment needs only a region source for non-text layers (grid cells, Voronoi cells, masks) feeding the same member set; the member time is in place. No production change was made.

## Cassette / Vism reuse

Kinetic type, shatter, trail and orbit recipes reduce to: a member source (units, cells, copies) + `order_weight` + a delay + Each / Random variation + WGSL effects on the result. The pieces exist; the frame-graph member time is the one thing a recipe would need that the live renderer lacks.
