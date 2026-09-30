# Relations v0 (2026-09-28): one real relationship

Probe, not a system: Null.Position.X drives four things' Scale, then Rotation, from the GUI; saved, reopened, exported.

## What was reused (nothing new evaluates)
- `PropertyLink` and `PropertySource::link_only`: the document's retained binding. A link replaces the property's own value (`SetPropertyLink` overwrites its track); cycles are refused. This is the existing ownership rule, so v0 never had to choose between replace and blend.
- `motolii.link.remap` (in_min, in_max, out_min, out_max, clamp): normalise then map, evaluated in one place (`translate_link`) by both `value_at` and the frame graph. Extended by two params: `in_component` reads one component of a vector source (Position read as X), `out_components` fills a vector destination (Scale) with the mapped number.
- The property spread, preview and commit contracts; Undo groups one `relate` into one step.

## What the host owns now
- `relate {source{layer, property, component}, inMin, inMax, members[], property, outMin, outMax, preview?}`: one link per member, as one step. A source never written is pinned to its current value first, so the renderer's graph has a node to read.
- `unrelate {layers[], property}`: the links come off and each thing keeps the value it shows as its own constant.
- Status: every property row carries `link` (source layer, property, component, kind, ranges) when it is driven.
- A relation is not stored: it is the links that share a source and an input range, seen together. Its member set is their destinations. Deleting the source layer leaves members at a default; deleting a member takes its link with it.

## What Flutter owns
- Entry: right-click a value in the Transform Instrument → Relation… (the value becomes the source; a vector's axis is the component).
- The Relations panel: the things at the current frame as dots where they are on the Stage; click, Shift-click, lasso; Esc cancels. Destination chips, two scrubbable ends per range, "set current as min/max" arrows, Create. Focused relation: source range (previewed while scrubbed, one commit on release), members with Edit in graph, mappings with ranges and remove, add mapping.
- Badges on Instrument lines: `◉ Null 1` on a driven value, `◉ 8` on a source (four things × two mappings); tap points at the relation.
- Units: Scale and Opacity shown as percent of the stored ratio, Rotation degrees, Position px.

## Acceptance
- Document (`port/relate.rs`): endpoints, clamp, reversed range, two mappings from one source, undo/redo, no drift on preview, status link, save/reopen, unrelate keeps the value, deleted member, deleted source, empty range and cycle refused.
- Live renderer (`port/members.rs`): the scene the pixels come from shows scale 1.0 at X 500, 1.25 and 15° at X 650 while dragging, one commit, three undos back to the circle's own scale. Save, reopen, scrub at three frames, export 6 frames: passed in 326 s.
- New shell (`new_shell_relations_test`): menu → panel → lasso 4 → Scale 50→150% → one `relate` with members [2,3,4,5]; add Rotation; badges; a click on a dot selects; remove a mapping is one `unrelate`.

## Next: DistanceSignal (not built)
A second link kind, `motolii.link.distance`, whose `translate_link` needs the destination layer's own position as well as the source value. Today `translate_link(plugin, params, value)` has no access to the destination; that is the one signature change the next step needs. The Relations panel, `relate`, badges and the member set are unchanged.

## Found on the way
- `position.x` / `position.y` as separate properties are a legacy split the resolver reads and the frame graph does not; the Inspector edits `position` as a vec2. v0 addresses the axis as a component of `position` instead.
- The Instrument draws Transform lines itself, so relation badges live in two places (ParamCell for sheets, `_line` for the Instrument).
