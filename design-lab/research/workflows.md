# Workflows: Motolii's interaction language, tried in the existing Widgetbook (2026-10-02)

Direction (owner): stop adding static components. The lab becomes a prototype where the main production operations can be tried by hand. The only workbench stays the Widgetbook (`book/`). No new gallery, catalog, framework or tool. Look/tone stay as DESIGN.md says; this document is about behaviour. Everything below was checked in `book/lib` (file:line given); nothing is guessed. Not code, a plan.

## 0. What exists today (checked)
- One real model exists: `Doc extends ChangeNotifier` + `Ctx` (InheritedWidget) in `sets/inspector_parts.dart:108,324`. It is a mock of ONE selected layer (values, keys `KeyS`, multi-select "mixed", reveal, effects order). Only the Inspector family (`inspector_set`, `inspector_gui_set`) is bound to it. Everything else (Timeline demos, Stage gizmos, Relation gadgets, Browser drag) keeps its own private `setState` and a `Live<V>` scratch value (`inputs_parts.dart:16`). So the panels cannot talk: selecting a row in a Timeline demo changes nothing in the Inspector.
- Timeline: `TimelinePanel` (parts/timeline.dart) is a StatelessWidget over const `sampleLayers`, `selected` is a knob. Parts that do move: playhead scrub (`timeline_parts.dart:320,1017,1154`), marker flag drag (:411), work-area bar move/resize (:486-502), keys select+move on property lanes (:806), mini curve editor handles (:639), lane-header scrub + stopwatch + key toggle (:1107).
- Motion language: one constant, 120 ms easeOut (`kDur/kEase` inputs_parts.dart:10, `kFast` duplicated in 3 files, stage_parts.dart:21). Hover = `MouseRegion` + `Hit`/`HitState` (inputs_parts.dart:90-175) in most sets.

## 1. Workflows (ordered by dependency, then by value)
Notation: W# = id. "Missing" names grammar from section 2.

**W1 Select and inspect.** Click a layer bar/label in Timeline (or its box on Stage) -> row, bar, Stage box and Inspector all show the same subject; click empty ground -> "nothing selected"; hover a row lights the same layer's Stage box.
- Widgets: `TpLabelCell`/`LayerRow` (timeline), Selection box + Transform gizmo (stage_set), Inspector panel + Empty state + header (inspector_set), Asset tile.
- Missing: shared selection, cross-panel hover link, 100-200 ms select transition on all three surfaces.

**W2 Edit one value both ways (number <-> graphic).** Select a layer; scrub Position X in the Inspector -> Stage box moves; drag the box on Stage -> number follows; Esc during drag restores; Shift fine / Ctrl coarse.
- Widgets: Number field (inspector_parts.dart:539 `_key`, mult rule :60), `inspector-gui` pad/dial/scale/anchor (inspector_gui_set), Stage gizmo `_GizmoDemo` (stage_set.dart:328).
- Missing: gizmo and pad bound to one value store (today the gizmo has its own `setState`); one drag = one undo step.

**W3 Set keys and play.** Stopwatch on Position -> key at playhead; move playhead, change value -> second key (auto, `Doc._touch`); diamonds appear on the bar row; Space plays, Stage object moves between keys; loop; Esc/Home to start.
- Widgets: key mark / reset mark (inspector_set:80), lane header (timeline_parts:1107), Transport (:1017), Playhead, Keyframe diamonds, Scrub strip.
- Missing: keys live in shared model; playhead is shared (`frame`); Stage evaluates the value at `frame` (linear is enough).

**W4 Timeline direct manipulation.** Drag a bar body to move, drag its edges to trim, drag a key to retime (snaps to frame, to playhead with Shift off), marquee-select keys, wheel/pinch zoom, middle-drag/scroll pan, click ruler to scrub.
- Widgets: Layer bars, Key diamonds, Property lane (:806), Ruler zoom-adaptive, Time zoom slider (:873), Work area bar (:486).
- Missing: bar move/resize, marquee, key snap, wheel zoom/pan (no `onPointerSignal` anywhere), clamp feedback.

**W5 Multi-select and edit a shared value.** Shift/Cmd-click three layers (Timeline) -> Inspector says "3 layers", differing values show mixed; scrub = relative to each, typed = absolute for all; Stage shows one combined box.
- Widgets: Mixed values use case (inspector_set:163), `Doc.delta/set` (A7 rules already coded), Selection box.
- Missing: multi-selection set in shared state; additive click grammar; combined Stage box.

**W6 Browser -> Stage / Timeline (and swap).** Drag a tile onto Stage -> ghost follows cursor, drop target highlights, new layer appears at drop point, selected, bar appears on Timeline; drop on Timeline row -> layer at that time; drop an asset on an existing layer's tile -> swap, bars/keys kept.
- Widgets: Asset tile/grid (media_set), Drag ghost (`_DragScene` media_set.dart:739, ghost card :788), Import drop zone, Composition frame.
- Missing: real drag source/target (no `Draggable`/`DragTarget` anywhere; the scene uses three fake slots), accept/reject states, drop -> model `addLayer`.

**W7 Make an object, add Scatter, widen it, key it, play (headline).** Add a shape (W6 or menu) -> Inspector Relations tab -> Add relation menu -> Scatter card -> drag the Scatter gadget (density/spread handles) -> Stage shows N copies spreading live -> key Spread at 0 and at 2 s -> play.
- Widgets: Add relation menu, Card collapsed/expanded, `ScatterGadget`/`HandleField` (relation_parts.dart:196), `_Flow` (relation_set.dart:684, already add -> tune -> see for Falloff), Relation badge/chip on the layer row, Colour = family on bar.
- Missing: relation list on the layer in the model; gadget value = model param (`rel.d/s/f` already in Doc); Stage renders the result from the same param; badge appears on the bar; param keyable (stopwatch on a gadget param).

**W8 Apply a Relation to an object by connecting.** Drag from a relation chip / Relations panel onto a layer (or drag a connection line between two layers) -> line follows with 120 ms magnet, target highlights, release creates the relation, Inspector of target shows the card.
- Widgets: Connection line `_ConnectionDemo` (relation_set.dart:542), Chip, Badge, Card list.
- Missing: shared selection and relation list; drag-drop grammar (W6 provides it); cancel on Esc.

**W9 Edit an easing.** Select two keys -> Ease section shows the curve; hover a preset previews on the Stage/curve without writing; click applies; drag Bezier handles; Esc mid-drag reverts; release = one step; overshoot toggle.
- Widgets: Curve preset picker (inputs.dart:195), Mini curve editor (timeline_parts:639), EaseThumb (dose_parts:379), `CurveGadget`.
- Missing: keys carry an `ease` value; peek-without-write; keyboard on the picker (arrows/Home/End/Enter/Esc, ease.md A1).

**W10 Keyboard only.** Select with arrows in the layer list, P/S/R/T/A reveal a property (exists in Inspector only, inspector_parts:1924), Up/Down nudges, Enter edits, Esc cancels, K adds key, J/L jump to key, Space plays, Cmd-K palette opens and runs "Add Scatter".
- Widgets: Inspector `_key`, Command palette, Tooltip with shortcut, Shortcut chip, FocusRing.
- Missing: focus order and one key map across Timeline/Stage/Browser; visible focus on bars and tiles.

## 2. Gap map (which existing widgets implement each grammar item; "-" = none)
| Grammar | Implemented today | Not implemented |
|---|---|---|
| Selection (within a panel) | label cells, `_Context` row pick (timeline_parts:1183), relation list/card pick (relation_set:203), media tile/row/tree (media_set:54,148,425), property lane key pick (:806) | Timeline main panel (knob only), Stage boxes, browser->selection |
| Selection (cross-panel, multi) | - | everything; only `Doc.kind=several` fakes the result |
| Hover | `Hit`/`HitState` (inputs_parts:90), timeline_parts (12 sites), stage_set (11), media tiles, relation chips, nav items | hover link to other panels; hover on bars/keys in `TimelinePanel` |
| Focus ring + Tab order | `FocusRing` (inputs_parts:65), `Hit`, TextBox, ScrubField, Inspector number field, inspector_gui instruments | Timeline, Stage, Browser tiles, Relation cards, nav/menu/palette (no FocusNode in navigation*.dart) |
| Keyboard | Inspector P/S/R/T/A + arrows + Enter/Esc + Del reset (inspector_parts:696,1924), Esc revert in inspector_gui (:210,:353,:1875), dose_spatial arrows/Esc (:266,:313), `Hit.onKey` | Timeline (playback, key nav), Stage (nudge, tool keys), Browser (arrows), Command palette, Curve picker, Cmd-Z |
| Number <-> graphic, both ways | Inspector number <-> `inspector-gui` pad/dial/face (shared `Doc`); gizmo, pad, gadgets each self-contained | gizmo bound to Doc; relation gadget <-> number field; ease handles <-> Bezier fields |
| Drag and drop | `_DragScene` ghost + fake slots (media_set:739) | real source/target, Browser->Stage/Timeline, relation->layer, effect reorder, swap |
| Timeline interaction | scrub playhead, marker drag, work-area move/resize, key move on lanes, curve handles, zoom slider +/-, stopwatch | bar move/resize, marquee, snap, wheel zoom/pan, twirl expand all (Alt-click), ruler click-scrub in the main panel |
| Graphic choice | Segmented, ToggleGroup, Curve picker, mode chips in inspector_gui | peek-before-commit (ease A1) |
| State transitions 100-200 ms | `kFast`/`kDur` 120 ms easeOut in ~20 files (`AnimatedContainer`, hover fades, popovers) | select/deselect on bars/keys, drop-target glow, magnet snap, "value changed" settle; one shared constant (three copies of `kFast`) |
| Esc / cancel / one undo step per gesture | number field + gui instruments restore on Esc | Timeline and Stage drags; no undo list wired (feedback `Undo history` is static) |
| Cross-panel flow | `_Flow` (relation_set:684) only, three private widgets | all |

## 3. Minimum shared state
Where: ONE new file `book/lib/session.dart`, a sibling of `tokens.dart` and `kit.dart` (not under any set), plus a 15-line `SessionScope` InheritedNotifier in the same file. It wraps the existing `Doc`; it does not replace it.
- `class Session extends ChangeNotifier`: `List<LayerM> layers` (id, name, `Fam` colour, start, end, `List<int> keyFrames` per property id, `List<RelM> relations` each with family + the three params already named in Doc: `rel.d`, `rel.s`, `rel.f`), `Set<String> selected` + `String? hover`, `int frame`, `bool playing`, `double ppf` (timeline zoom), `Map<String, Doc> docs` (one existing `Doc` per layer, so every Inspector widget works unchanged), and an `active` getter that returns the `Doc` the Inspector should show (a layer's Doc, or a `Kind.several`/`Kind.none` Doc).
- Verbs only, each a plain method that mutates and calls `notifyListeners()`: `select(id, {add})`, `hover(id)`, `moveBar(id, dFrames)`, `trimBar`, `moveKey`, `addLayerFromAsset`, `swapAsset`, `addRelation(layerId, fam)`, `setRelParam`, `seek(frame)`, `play(bool)`. A drag calls `begin()` (snapshot), `update()`, `commit()` or `cancel()` (restore snapshot); the snapshot list is the only undo (also feeds the existing static "Undo history" widget).
- Used by: a use case reads `SessionScope.of(context)`; a part takes values + callbacks as it does now, so parts stay stateless-by-constructor and `Widgetbook` knobs still work.
- MUST NOT become: a store/reducer/event bus/DI/Riverpod/Provider/Bloc dependency, a persistence or serialisation layer, a model of the real Motolii (no FFI, no revision, no cache), a place for editor-only chrome state that parts own (hover of a button, open popovers, scroll), or a second place for values already in `Doc` (copy nothing; delegate). No new package. No generic "command" classes. If a verb needs more than ~10 lines it belongs in the part that owns the gesture.
- Stage evaluation (W3, W7) is one pure function in the same file (`valueAt(layer, prop, frame)`, linear plus a named ease curve); not an animation engine.

## 4. Acceptance (critic, in the real window; each check Y/N; "photo" = screenshot pair before/after)
- **W1** (1) Click bar of layer 3 in Timeline: the Inspector header changes to layer 3's name within the same frame (photo). (2) The same layer's box on Stage shows the selection outline (photo). (3) Click empty ground: Inspector reads "nothing selected" and no outline remains. (4) Hover another row without clicking: its Stage box lightens, the Inspector does not change. (5) Tab/arrow moves the selection and the focus ring is visible on the row.
- **W2** (1) Scrub Position X +50 in the Inspector: the Stage box moves right. (2) Drag the box 40 px left: the X readout decreases to match. (3) Press Esc during the drag: box and number return. (4) Shift-scrub moves 10x slower than plain. (5) After a drag, Cmd-Z restores in one step.
- **W3** (1) Click stopwatch: a diamond appears at the playhead on that layer's row. (2) Move playhead to 2 s, change value: a second diamond and a link line appear. (3) Space: the Stage object travels between the two positions and the playhead advances. (4) Space again stops; the object stays at the current frame. (5) Delete on a selected diamond removes it and the line.
- **W4** (1) Drag a bar body: start and end move equally and the Inspector/readout updates. (2) Drag its right edge: only the end moves, min width 3 px holds. (3) Drag a key 10 frames: it lands on a whole frame, the lane row link line follows. (4) Marquee over keys: all inside become accent-filled. (5) Wheel with Cmd: ruler labels change step, playhead stays under the cursor.
- **W5** (1) Shift-click two more layers: Inspector header reads "3 layers". (2) A differing value shows the mixed mark (not one layer's number). (3) Scrub Opacity -10: every layer drops by 10 (values still differ). (4) Type 50: all three read 50. (5) Stage shows one combined box around the three.
- **W6** (1) Press on an asset tile and move: a ghost follows the cursor (shadow, tilt) and the tile dims. (2) Over Stage the drop area highlights; over a locked layer it says it is locked and refuses. (3) Release on Stage: a new layer appears at the drop point, selected, its bar appears on the Timeline. (4) Drag onto an existing layer: the layer's picture changes, its bar and keys do not. (5) Esc during the drag leaves nothing added.
- **W7** (1) New object -> Relations tab -> Scatter: a Scatter card and a pink badge on the bar appear. (2) Drag the card's gadget handle outward: copies on Stage spread and the Spread number follows. (3) Stopwatch on Spread at 0 s and again at 2 s with another value: two diamonds on that row. (4) Space: spread animates. (5) Remove the card: copies, badge and diamonds are gone.
- **W8** (1) Press on a relation chip and move: a line (or ghost) follows the cursor with a 120 ms ease. (2) Layers under the cursor highlight one at a time. (3) Release on a layer: its bar gets the family-colour badge and the Inspector (when selected) shows the card. (4) Release on empty ground: nothing is created, the line retracts. (5) Esc cancels mid-drag.
- **W9** (1) With two keys selected the Ease section shows their curve. (2) Hover a preset: the curve and Stage object preview the new ease; moving off restores it. (3) Click applies: the diamond link changes and one undo step exists. (4) Drag a Bezier handle: the curve and Stage motion change live; Esc reverts. (5) Arrow keys + Enter pick a preset without the mouse.
- **W10** (1) With no mouse: arrows select layer 2, P reveals Position and focuses X. (2) Up changes X by 1 (Shift 10); Enter types; Esc cancels. (3) K adds a key at the playhead; J/L jump keys. (4) Space plays/stops. (5) Cmd-K, type "scatter", Enter: Scatter is added to the selection; every focused thing showed a visible ring.

## 5. Precedent look needed (one question each) vs decidable from DESIGN.md
Decidable now from DESIGN.md/existing code: W1 (selection by component class is already `[decided]`: row = g20 + tick, tile = accent outline, key = accent), W2, W3 (key and playhead looks are `[decided]`), W5 (A7 rules in inspector.md and coded in `Doc`), W7 (family colours, badge, card all exist), W8 visuals (connection line exists).
Precedent look (name the question only):
- W4: Where does a bar edge grab zone start (px) and which key snaps to what (frame, playhead, other keys) in AE, Premiere, Ableton? And does wheel = zoom or scroll?
- W6: What does a drop target look like in a dense dark tool while the ghost is over it (outline vs fill vs insertion line) in AE project->comp, Figma, Ableton browser->track? Swap: Alt-drop (AE) vs plain drop?
- W9: Is peek-without-write (ease.md A1) used in a shipped tool, and does Esc revert or commit there (Figma/Cavalry/AE graph editor)?
- W10: Which key set is the minimum AE/Blender/Ableton users share (J/K/L, Space, P/S/R/T/A, U)? Do not invent: take AE's.

## 6. Build order and ownership (two authors in parallel; one owner of `session.dart` per round)
New use cases go in two NEW files, not into existing 1,000-2,000 line sets; `main.dart` gets one line per round (owner: Author A, appended at the end of the round to avoid conflicts).
- **Round 1 (foundation + the first value loop): W1 + W2.**
  - Author A: `book/lib/session.dart` (Session, SessionScope, snapshots), new `book/lib/sets/workflow_set.dart` (the "Workflow" component: one use case "Select and inspect" laying out Timeline strip + Stage + Inspector in one window), small additive edits to `sets/timeline_parts.dart`/`parts/timeline.dart` for click select and hover.
  - Author B: `sets/stage_parts.dart` + `sets/stage_set.dart` (gizmo and boxes read/write Session via callbacks), `sets/inspector_parts.dart` only to expose `Doc` creation from Session (`Doc` constructor/adapter; one method).
  - Shared, read-only for both: `tokens.dart`, `kit.dart`, DESIGN.md. If either needs a token, it goes to A in the round notes.
- **Round 2: W3 + W4.** Author A: `sets/timeline_parts.dart`, `parts/timeline.dart` (bar/key drag, snap, marquee, zoom/pan), `timeline_parts_parts.dart`. Author B: playback and key evaluation in `session.dart` (`valueAt`, `play`) plus Stage rendering, `sets/transport.dart`/`transport_parts.dart` for Space/loop. Boundary: Timeline calls `session.moveBar/moveKey`; B owns `session.dart` this round, A files requests.
- **Round 3: W5 + W6.** Author A: `sets/media_set.dart`/`media_parts.dart` (real `Draggable` source, drop ghost, accept/reject). Author B: Stage drop target in `stage_set.dart`, multi-select in Inspector adapter (`inspector_parts.dart` `Doc` for `several`), `session.addLayerFromAsset/swapAsset`.
- **Round 4: W7 + W8** (the headline). Author A: `sets/relation_set.dart`/`relation_parts.dart` (gadget <-> param, connect drag). Author B: Stage result renderer + Inspector Relations section (`inspector_parts.dart`, `parts/inspector.dart`), badge on bars (`timeline_parts.dart` row only through a one-line hook A already added in round 2).
- **Round 5: W9 + W10.** Author A: `sets/inputs.dart`/`inputs_parts.dart` (Curve picker keys + peek), `dose_parts.dart` EaseThumb reuse. Author B: key map + focus order (`sets/navigation_parts.dart` Command palette, `FocusRing` usage in timeline/stage/browser) after A's file list is frozen.
- Rules for every round: one file has one author; cross-file requests are written as one line in the round notes, not edited; each author runs `flutter analyze` only; the critic then runs the acceptance of section 4 in the real window (Widgetbook, path `workflow/...`), not on screenshots alone, and reports Y/N per check. Done = the owner can perform the workflow by hand, not that the parts exist (memory: Done is the real thing).
