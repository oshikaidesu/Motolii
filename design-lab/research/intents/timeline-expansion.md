# Intent cards: Timeline layer rows, expansion, clips, switches, lanes

Source: old Motolii `/private/tmp/wt/tok/motolii/ui/lib/timeline/**` (read-only), the inventory `inventory/timeline-stage-desks.md`, `old-motolii-complaints.md`, and web precedents. Only INTENT and interaction contract are recorded; nothing about how the old UI looked is a spec. `[inferred]` = my reading, not stated in code or docs. `[unverified]` = precedent fact from memory or a search snippet, not confirmed on the vendor page (Adobe help pages returned 403; Cavalry docs 404).

Old-code references are `session.dart:line` unless noted. Verdict "WELL / POORLY" is about the old app's behaviour, not looks.

Quick map of what the old app did NOT have (so you do not tick things that never existed): no value readout on lanes, no curve graph in the timeline (Graph tab is an empty placeholder), no per-interpolation glyph other than linear = dashed, no shy, no work area/loop, no scrollbar, no name labels on markers, no keyboard way to expand/collapse, no keyboard way to select layers/keys beyond arrows.

---

## A. Expand / collapse

### A1. Per-layer twirl (open a layer's parameter lanes)
- **Intent**: Open one layer to see and edit its animated parameters without leaving the timeline.
- **Contract**:
  - Clicking the layer's twirl toggles its lanes; the lanes appear directly under the layer row, in the layer's own order.
  - Default open set = only parameters that have keys (plus text content keys and effect parameters with keys).
  - Opening one layer does not change any other layer's state.
- **Came from**: `session.dart:62` `toggleLanes`; `rows.dart:~130` (lane list built only if `all.contains(id) || keys.isNotEmpty`); inventory item 6. AE precedent: https://helpx.adobe.com/after-effects/using/layer-properties.html (twirl arrow per layer, then per group, e.g. Transform > Position) [unverified, 403].
- **Old app**: WELL on the "keyed only" default (a layer with 2 keyed params shows 2 lanes, not 20). POORLY: twirl is mouse-only; open state is not remembered across sessions [inferred]; a group has TWO twirls (fold children vs open own lanes), which is confusing (item 6).
- **Candidate?** Yes. Core of "expansion", which the owner already named.

### A2. Show only keyed / show all / hide, per layer
- **Intent**: Choose between "just what I animated" and "everything the layer has".
- **Contract**:
  - One action reveals keyed parameters only; another reveals all parameters of that layer; another closes the lanes.
  - Switching between them keeps the layer open.
- **Came from**: `session.dart:67 showLanes`, `:70 hideLanes`, menu lines `:~440` ("Show animated properties / Show all properties / Hide properties"). AE: `U` = reveal properties with keyframes, `UU` = reveal modified-from-default properties (a snippet claims UU = expressions; AE's `EE` is expressions, so treat the snippet as doubtful) https://www.tutsplus.com / https://helpx.adobe.com/after-effects/desktop/get-started/keyboard-shortcuts/keyboard-shortcuts-reference.html [unverified]. Blender dope sheet: filter "Only Show Selected", "Show Hidden", "Only Show Errors" instead of per-layer modes https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html.
- **Old app**: WELL in intent. POORLY in reach: it lived only in a right-click menu (no shortcut, no modifier-click on the twirl), and there is no "modified from default but not keyed" tier (AE's UU) because the old data had none [inferred].
- **Candidate?** Yes. Cheap to carry; the AE U / UU habit is what After Effects users already expect. Decide whether "changed" (UU) is wanted separately from "keyed" (U).

### A3. Expand / collapse all (and Alt-click twirl = all)
- **Intent**: Open or close every layer at once.
- **Contract**: Not in old app. Proposed: one gesture (e.g. Alt-click on any twirl) applies to all layers; same again closes all.
- **Came from**: NOT in old code (grep of `session.dart`: only per-id sets). AE: Alt/Opt-click on a twirl expands/collapses all siblings at that level [unverified]. Ableton: Alt/Opt-click on Unfold Track, or Alt-U, unfolds all tracks; plain `U` unfolds the selected track https://www.ableton.com/en/manual/arrangement-view/. Blender: Ctrl+Numpad +/- expand/collapse channels [unverified].
- **Old app**: ABSENT. With 40 layers this is a real gap.
- **Candidate?** Yes. Strong precedent in AE and Ableton; one-line contract.

### A4. Group fold (hide children)
- **Intent**: Fold a group so its member layers disappear from the list without being deleted or hidden.
- **Contract**:
  - Folding a group removes its children's rows (and their lanes) from the list; rendering is unaffected.
  - A folded group still shows a summary of its children's keys [inferred, old code only summarises the group's own keys, `rows.dart _allKeys`].
  - Children are indented one step per depth.
- **Came from**: `session.dart:75 toggleFold`; `rows.dart:~95-130` `LaneLayout`, `groupOpen`; children only nest under `kind == 'Group'`. Ableton Group Tracks fold [unverified detail].
- **Old app**: Mixed. Nesting recursion is clean and tolerates cycles. Poorly: fold state and lane-open state are separate sets per group (two toggles); indentation widens the whole name column (`nameWidth = base + indent`, clamped) so the clip area shifts when nesting changes [inferred, `rows.dart`].
- **Candidate?** Yes (needed if groups exist at all). Ask owner: one fold control per group, not two.

### A5. Row height when expanded
- **Intent**: Expanded content must not make rows jump or make the list unreadable.
- **Contract**:
  - Every row, layer or lane, has the same pitch (one row unit); opening N lanes inserts exactly N rows and pushes layers below down.
  - Scroll position is kept so the layer you opened stays where it was.
- **Came from**: `rows.dart LaneContainer.place` (uniform `rowHeight`, children stacked); `session.dart:~100 relane` clamps `firstRow`. Ableton: unfolded track height is user-adjustable, Alt+/- https://www.ableton.com/en/manual/arrangement-view/.
- **Old app**: WELL for uniformity; POORLY in that opening near the bottom does not scroll the new lanes into view [inferred, no code found for it]; no per-lane height change.
- **Candidate?** Yes (uniform pitch + keep opened layer in view). Per-row height change: No unless the owner wants value readouts to need more space.

---

## B. The clip (bar)

### B1. Select a clip
- **Intent**: Pick one or several layers by their bars.
- **Contract**: Click = pick only that; Cmd-click toggles it; Shift-click picks the range from the anchor; clicking a bar of an already-picked layer keeps the whole selection (so a group can be carried).
- **Came from**: `session.dart:_choose` (~225), `press TlBar`.
- **Old app**: WELL; selection logic is one place, and range-by-order is standard.
- **Candidate?** Yes.

### B2. Move a clip
- **Intent**: Slide a layer in time.
- **Contract**: Dragging the bar body shifts all selected unlocked layers by the same whole-frame delta; frame 0 is a floor; Esc cancels and restores; the Stage shows the same preview while held.
- **Came from**: `session.dart` `TlGesture.move`, `previewTimings`, `commitPreview`, `key()` Esc.
- **Old app**: WELL. Live preview in the viewer plus Esc-cancel is the right contract.
- **Candidate?** Yes.

### B3. Trim in / out
- **Intent**: Change where a layer starts or ends without moving its content.
- **Contract**: Dragging either end of the bar trims that end by whole frames; a locked layer ignores the drag; the content under it does not shift.
- **Came from**: `TlGesture.trimIn/trimOut`, modes 'trimStart'/'trimEnd'.
- **Old app**: WELL for behaviour. The trim hit area is a thin edge, small on dense rows [inferred].
- **Candidate?** Yes.

### B4. Slip (Alt-drag)
- **Intent**: Change which part of the source is visible while the bar stays put.
- **Contract**: Alt-drag on the bar slides the content within a fixed bar.
- **Came from**: `TlGesture.slip` (`session.dart` press, `mods.alt`). AE: Alt-drag in layer bar [unverified].
- **Old app**: Present, undiscoverable (no cue anywhere).
- **Candidate?** Ask. Powerful but needs discoverability.

### B5. Ghost / hold tail
- **Intent**: See that a layer's effect continues (loop or hold) beyond its bar, without it being real clip length.
- **Contract**: A fainter extension beyond the end (or before the start) shows the held/looped span; it is not selectable as a clip.
- **Came from**: `session.dart:~100 extent` includes `ghost`; inventory item 14 (28% copy of bar).
- **Old app**: Implemented but no fixture exercises it [inferred, "Story: none"]; unproven.
- **Candidate?** Ask owner. Only keep if the product has loop/hold-after.

### B6. Split at playhead, markers
- **Intent**: Cut a layer where the playhead is; flag points in time.
- **Contract**: Split cuts selected layers at the playhead; a marker is added at the playhead, can be dragged in the ruler and deleted from its menu.
- **Came from**: Strip tools Split/Marker (`timeline.dart:37`), `session.dart:~160 dragMarker/deleteMarker`.
- **Old app**: Marker has no label; no work area/loop at all (TL-004).
- **Candidate?** Split: Yes (menu/shortcut ⌘K existed). Markers: Ask. Out of scope for this card set otherwise.

### B7. Summary keys on a closed layer
- **Intent**: A collapsed layer still shows that and when it is animated.
- **Contract**: With lanes closed, every key frame of every parameter shows once on the bar (merged by frame); picking one picks all keys at that frame in that layer.
- **Came from**: `rows.dart:_allKeys, summaryFrames`; `session.dart press TlLayerKey`.
- **Old app**: WELL in intent (one mark per frame, not one per parameter). Noisy with "many keys" because they sit on top of the bar [inferred].
- **Candidate?** Yes.

---

## C. Switches (eye, lock, others)

### C1. Eye (visibility)
- **Intent**: Take a layer out of the picture without deleting it.
- **Contract**:
  - Off = layer is not rendered or shown on the Stage; its timing and keys remain editable.
  - Row name dims when off.
  - One click toggles; it applies to the row you clicked (not to the selection) [inferred: `toggleSwitch(row, field)` is per row].
- **Came from**: `session.dart:80 toggleSwitch` (flag `hidden`), inventory item 8. AE video switch https://helpx.adobe.com/after-effects/using/layers.html [unverified]; Blender dope sheet/graph editor channel hide-eye (hides the channel from the editor, not from render) https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html. NOTE: in Blender the eye hides the F-curve in the editor; in AE/Motolii it hides the layer from render. Pick the AE meaning.
- **Old app**: WELL in contract; POORLY in noise: eye, lock, solo, clip all share a 4-slot column, dim until hover.
- **Candidate?** Yes (owner named it).

### C2. Lock
- **Intent**: Make a layer safe from accidental edits.
- **Contract**:
  - Locked layer cannot be moved, trimmed, or slipped from its bar.
  - It still renders and can still be seen/selected for reading [inferred; old code lets you select but skips it in `_timingRows`; AE instead refuses selection in the panels, https://helpx.adobe.com/after-effects/using/layers.html [unverified]]. Decide which.
  - Key edits on a locked layer: old code does not guard `moveKeys` explicitly (the host may) [inferred, uncertain].
- **Came from**: `session.dart:~215` (`l['locked'] != true`), `toggleSwitch`. Blender channel lock (padlock) prevents editing keyframes of that channel https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html. Ableton "Lock Envelopes" is a different thing: it ties automation to song time instead of clips https://www.ableton.com/en/manual/arrangement-view/.
- **Old app**: Only partly consistent: the guard is on bar timing, not demonstrably on keys, properties, or Stage edits.
- **Candidate?** Yes (owner named it). Needs one definition: "locked = no edit anywhere, still visible".

### C3. Solo
- **Intent**: Look at one thing alone.
- **Contract**: With any layer soloed, only soloed layers render; turning solo off restores all; solo does not change the document's visibility flags [inferred].
- **Came from**: flag `solo` in `toggleSwitch`; AE solo "includes the layer in previews and renders, ignoring layers without this switch" https://helpx.adobe.com/after-effects/using/layers.html [per search snippet].
- **Old app**: Present, behaviour not verified in the timeline code (the render side is elsewhere).
- **Candidate?** Ask (owner named only eye and lock). Candidate for "FEW symbols": could live in the row menu.

### C4. Clip-to-below
- **Intent**: Use the layer below as a matte/clip for this one.
- **Contract**: Toggles a clip relationship with the layer under it.
- **Came from**: `toggleSwitch` field `clipToBelow` calls `clip` op.
- **Old app**: A fourth always-present slot for a rare action is the clearest example of too many glyphs.
- **Candidate?** No in the row; belongs in a menu / Inspector.

### C5. Freeze, shy, mute
- **Intent**: Freeze = hold a layer on one frame; shy = hide row from the list but keep rendering; mute = silence audio.
- **Contract**: Freeze: via menu only (`freeze` op, `session.dart:~445`). Shy: absent. Mute: absent in this file.
- **Came from**: AE shy (hides the layer's row when "Hide Shy Layers" is on; still renders) https://blog.nobledesktop.com/learn/after-effects/shy-switch-timeline-tips. Old: menu only.
- **Candidate?** Shy: Ask (useful at 40 layers, but it is a second visibility concept). Freeze: No for the row. Mute: Ask if audio matters.

### C6. Switch scope on multiple selection
- **Intent**: Toggling a switch while several layers are selected.
- **Contract**: Unspecified in old app (acts on the clicked row only). [inferred] Proposed: Alt-click = applies to all.
- **Candidate?** Ask.

---

## D. Per-parameter lanes

### D1. Key lane (add, remove, select)
- **Intent**: See and edit a parameter's keys in time.
- **Contract**:
  - A lane shows one mark per key at its frame.
  - The row has a button that toggles a key at the playhead for that parameter (on = filled when a key exists on the current frame).
  - Click picks one key; Cmd/Shift adds or removes; a marquee on empty space picks keys inside it.
- **Came from**: `session.dart:~95 toggleKeyHere`, `_pickKeys`, `tlPick` (`semantics.dart:~85`), `tlMarquee`. AE stopwatch/diamond arrows [unverified].
- **Old app**: WELL; selection helpers are small and well separated. POORLY: you can only add a key through the playhead (no double-click on the lane to add) [inferred].
- **Candidate?** Yes.

### D2. Move keys
- **Intent**: Shift keys in time.
- **Contract**: Drag a picked key (or any of several picked) by whole frames; floor at frame 0; keys keep drawing where they will land, and after release stay there until the document confirms (no snap-back); arrows nudge 1 frame, Shift+arrow 10.
- **Came from**: `session.dart:drag TlGesture.keys, release, settlingKeys`, `key()`.
- **Old app**: WELL. The "settling" idea (no flicker on release) is a good contract to keep, whatever the draw.
- **Candidate?** Yes.

### D3. Interpolation display between keys
- **Intent**: Tell what happens between two keys (linear, hold/step, eased) at a glance.
- **Contract**: Old: linear is distinguished from every other kind; there is no marker for hold or bezier. Click on the span picks both end keys.
- **Came from**: inventory item 19; `semantics.dart TlSpan`. Blender dope sheet draws a hold bar between equal-valued keys and (option) shows handle/interpolation types https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html [unverified detail].
- **Old app**: POORLY. Only 2 states are distinguishable, and "everything not linear" is lumped as solid, so hold looks the same as bezier.
- **Candidate?** Ask. Owner needs to decide whether the timeline shows interpolation at all or leaves it to the Ease desk.

### D4. Value readout along a lane
- **Intent**: Know the parameter's value at the playhead (and at each key) without opening the Inspector.
- **Contract**: NONE in the old app; the lane rows show label + key button only (code grep: no value read in `timeline/`).
- **Came from**: absent. AE: the timeline's property row shows the current value at the playhead, editable by scrubbing the number (click-drag) or typing [unverified]; Ableton automation lanes draw the envelope itself in the lane, plus a value in the lane header's chooser.
- **Old app**: ABSENT. The owner named "each parameter's display" as liked; since the old timeline had no value, this may mean the display of the lane itself or the value shown in the Inspector. [inferred] Ask.
- **Candidate?** Ask. Highest-value question in this file: what is "display"? (a) value number at the playhead; (b) a mini curve in the lane; (c) the key marks only.

### D5. Scrub a value from the timeline
- **Intent**: Change the value at the playhead directly in the lane row.
- **Contract**: Not in old app (dragging a number happened in the Inspector).
- **Came from**: AE value drag in the property row [unverified].
- **Candidate?** No unless D4 is chosen with a number.

### D6. Mini curve / graph in lane
- **Intent**: See the shape of change, not only key positions.
- **Contract**: Not in old app (Graph tab was a placeholder). Ableton draws automation as an editable line in the lane https://www.ableton.com/en/manual/arrangement-view/. Blender separates it (Graph Editor) from the Dope Sheet.
- **Candidate?** Ask (relates to D3, D4).

### D7. Choose which parameter a lane shows
- **Intent**: Pick a parameter to see/edit when it has no keys yet.
- **Contract**: Old: "Show all properties" lists every parameter; no way to pick one. Ableton: lane header has two choosers (device, then parameter) https://www.ableton.com/en/manual/arrangement-view/ [partly unverified].
- **Candidate?** Ask. AE-like "show all" may be enough.

### D8. Effect parameter lanes
- **Intent**: Effects on a layer appear in the same lane list, labelled by effect + parameter.
- **Contract**: Effect parameters with keys are listed with the effect name prefix.
- **Came from**: `rows.dart:~118` (`'${effect.name} · ${param.label}'`).
- **Old app**: WELL.
- **Candidate?** Yes.

---

## E. Reorder, drop, view

### E1. Reorder by dragging a layer name; drop guide
- **Intent**: Change stacking order and group membership by dragging.
- **Contract**:
  - Dragging a name shows exactly one guide: a line between rows, or an outline on a group when dropping inside it.
  - You cannot drop a layer into itself or its descendants.
  - The middle ~54% of a group row means "inside", the outer parts mean before/after.
- **Came from**: `session.dart:_dropAt` (f > .23 && f < .77 = inside), `TlDrop`. Same code path accepts an asset dragged from the Browser (`aimAsset/acceptAsset`).
- **Old app**: WELL; the drop target logic is tidy and shared with asset drops. Rough: "inside" zone is thin and not discoverable [inferred].
- **Candidate?** Yes.

### E2. Zoom and scroll
- **Intent**: Move through time and through many layers without a scrollbar.
- **Contract**: Wheel pans; Shift-wheel pans time; Cmd/Ctrl-wheel or over the ruler zooms about the pointer; pinch zooms; two-finger pan has inertia; the view cannot leave the work (extent = longest of duration/playhead/layer end/ghost + margin).
- **Came from**: `session.dart:zoomAt, _clampView, extent`; inventory item 28.
- **Old app**: Mixed. WELL: zoom about pointer, clamped view. POORLY: no scrollbar or overview, so you cannot tell where you are in a long list; no zoom-to-fit button in this skin. Complaint #1 (1% steps came out as 10%) applies to any scroll-step contract [from complaints file].
- **Candidate?** Yes. Add the contract line: "one notch = a fixed step; Shift = coarser", and say it in the UI.

### E3. Seek by pressing the ruler
- **Intent**: Click anywhere on the ruler to put the playhead there at once; dragging scrubs.
- **Contract**: Press seeks immediately; the playhead draws where the pointer is before the host replies.
- **Came from**: `session.dart:seek, scrub`.
- **Old app**: WELL.
- **Candidate?** Yes.

### E4. Density with 40 layers
- **Intent**: 40 layers must stay legible and fast.
- **Contract**: Closed layers cost one row each; only keyed lanes open by default; the list scrolls by fractional rows; layout is recomputed only when layer shape changes (not on every value edit).
- **Came from**: `session.dart:~20` (`derived: c.layerShape`), `rows.dart`; story "Dense (40 layers)".
- **Old app**: Mixed. Performance idea WELL (layout is derived from shape, not values). UX POORLY: no way to find a layer (no filter/search, no shy, no collapse-all) [inferred]. Blender dope sheet offers name filter and "Only Show Selected" https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html [partly unverified].
- **Candidate?** Yes for "stays legible at 40"; Ask for "filter/search/only selected".

### E5. Keyboard access
- **Intent**: Do the common things without the mouse.
- **Contract (old)**: Left/Right = nudge picked keys by 1 (Shift 10), else step the playhead; Esc = cancel a gesture and restore; menu shortcuts only: ⌘C/X/V/D, ⌫, ⌘G, ⇧⌘G, ⌘K.
- **Came from**: `session.dart:key()`, `menu()`.
- **Old app**: POORLY. Nothing for expand/collapse, select next/prev layer, toggle eye/lock, or jump to next key. AE has U/UU, J/K (prev/next key), and layer switch shortcuts [unverified]; Ableton `U` unfolds a track [verified above].
- **Candidate?** Yes. Minimum proposed set: expand/collapse selected layer, reveal keyed, eye and lock on selection, prev/next key. Owner to choose keys.

---

## F. Do-not-repeat list (rough or noisy in the old app)

1. Too many glyphs per row: twirl + kind chip + name + clip + eye + solo + lock, plus a key diamond on lanes. The owner wants FEW symbols; keep at most the twirl and the switches the owner ticked (eye, lock), and move the rest to the menu/Inspector.
2. Switches that are dim until hovered: the row looks different from one moment to the next. Show a state only when it is not the default [inferred] (visible = changed).
3. Two twirls on a group (fold children vs open own lanes).
4. Name column that widens with nesting depth and so moves the whole clip area.
5. Key marks drawn on top of the bar when closed (the bar's colour and the keys fight [inferred]).
6. Features in the right-click menu only (expand modes, freeze), with no gesture or key.
7. Hidden gestures with no cue: Alt-drag slip, thin "inside group" drop zone.
8. Placeholder tabs (Graph, empty) in the same seat as the timeline.
9. Interpolation shown with a 2-state code (dashed vs not): lumps hold and ease together.
10. Colour per layer from its id (not meaningful) [inferred from `_family(id)`], so colour cannot carry information.
11. Row height 23 with 14px switch slots: small hit targets at density [inferred].
12. No scrollbar / position cue; no search at 40 layers.
13. Lock defined only on bar timing, not on keys or Stage edits (inconsistent contract).
14. Presentation ruling per complaints file: a thick line or loud element in a dock panel is too assertive. Keep timeline marks quiet [from old-motolii-complaints #3, by analogy].

---

## G. Precedent notes (what each really does)

| Tool | Behaviour | Source |
|---|---|---|
| After Effects layer twirl | Each layer has a twirl; opens groups (Transform, Effects...) then properties. Alt/Opt-click on a twirl applies to all at that level [unverified]. | https://helpx.adobe.com/after-effects/using/layer-properties.html (403) |
| AE `U` / `UU` | U = reveal properties that have keyframes on selected layers. UU = reveal properties changed from default (one search snippet says expressions; unconfirmed). | https://helpx.adobe.com/after-effects/desktop/get-started/keyboard-shortcuts/keyboard-shortcuts-reference.html [unverified] |
| AE switches | Eye (video) toggles visibility; speaker toggles audio; Solo = only soloed layers render/preview; Lock = no editing and layer cannot be selected in the panels; Shy = hides the layer's row when "Hide Shy Layers" is on, still renders. | https://helpx.adobe.com/after-effects/using/layers.html, https://blog.nobledesktop.com/learn/after-effects/shy-switch-timeline-tips |
| AE property value | A property row shows its current value at the playhead, editable by drag or typing; its stopwatch creates the first key [unverified]. | memory |
| Blender dope sheet | Channel list at left: expand/collapse hierarchy, eye/hide, padlock lock, mute checkbox; filters Only Show Selected, Show Hidden, Only Show Errors; keys select/move with box select. | https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html (page summary only) |
| Ableton Arrangement | Fold/unfold button beside track name; `U` unfolds selected track; Alt/Opt-click or Alt-U unfolds all; unfolded height adjustable (Alt/Opt +/-); Automation Mode toggle shows automation lanes, with a parameter chooser per lane; Lock Envelopes ties automation to song time, not clips; Group Tracks nest. | https://www.ableton.com/en/manual/arrangement-view/ |
| Cavalry | Not verified (docs URL redirected then 404). From memory: attributes with animation get keys on a timeline under the layer, and a separate Graph Editor holds curves [unverified]. | https://cavalry.studio/docs/ [not read] |

---

## H. One-glance tick list (for the owner)

Already named by the owner: expansion (A1), clip (B1-B3), eye (C1), lock (C2), each parameter's display (D1; D4 is the question).

Yes-by-default candidates: A1, A2, A3, A4, A5, B1, B2, B3, B7, C1, C2, D1, D2, D8, E1, E2, E3, E4, E5.
Ask: B4 slip, B5 ghost, B6 markers, C3 solo, C5 shy/mute, C6 multi-select switch scope, D3 interpolation display, D4 value readout, D6 mini curve, D7 pick a parameter, E4 filter/search.
No: C4 clip-to-below in the row, C5 freeze in the row, D5 scrubbing values in the timeline.
