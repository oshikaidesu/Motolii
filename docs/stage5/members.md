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

## The gap that stops the grammar (stop condition B)

The live renderer (frame graph, which also makes the pixels) evaluates every track at the frame time. It gives copies their member time because the copy path samples a layer's properties at each copy's time. It has no member time for children or split units:
- a plain group's keyed children under Stagger read 0.5 / 0.4 / 0.3 in the document and 0.5 / 0.5 / 0.5 on the live scene;
- a text with Split Words is 3 units in the document's resolve path and 1 layer on the live scene.
Tests: `the_live_scene_gives_children_their_member_time`, `the_live_scene_cuts_a_split_text_into_units` (ignored, failing).

Missing host primitive: member time in the frame graph, i.e. sampling a layer (or a unit cut of it) at the time its order gives it. The property program is designed not to read the document at evaluation, so this is an architecture decision, not a patch: either the graph samples children like copies (dynamic inputs at a member time) or a layer clock becomes a graph node.

## Fragments

Split units already are the mechanism a Fragment needs: resolve the whole layer at a member time, cut it by a region, turn it about the region's centre. What is missing is only a region source for non-text layers (grid cells, Voronoi cells, masks), and the same frame-graph member time as above. No production change was made.

## Cassette / Vism reuse

Kinetic type, shatter, trail and orbit recipes reduce to: a member source (units, cells, copies) + `order_weight` + a delay + Each / Random variation + WGSL effects on the result. The pieces exist; the frame-graph member time is the one thing a recipe would need that the live renderer lacks.
