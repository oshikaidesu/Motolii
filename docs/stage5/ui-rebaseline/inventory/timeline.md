# Capability inventory — Timeline / Transport / Desk tools / Export

Audited from code (read-only) at `db7d68118` (branch `claude/ui-rebaseline`). Paths are repo-relative.
Abbreviations: `TL = motolii/ui/lib/panels/timeline`, `P = motolii/ui/native/src/port.rs`, `SC = motolii/ui/lib/input/editor_shortcuts.dart`.
"has(op)" = `EditorSession.supports(op)` (the native capability list at `P:8`); a control whose op is missing is disabled or does nothing.
Primary modifier = Cmd **or** Ctrl (`TL/frame.dart:58`).

## Table

### Transport and playback

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-001 | Play or pause | Timeline bar button `▶` / `Ⅱ` (highlighted while playing, tooltip "Play / Pause · Space") | Needs `play`/`pause`, a texture, duration ≥ 2 frames (`session_render.dart:44-45`) | `motolii/ui/lib/panels/timeline.dart:253-261` | `EditorSession.togglePlayback` → native `play` / `pause` (`P:257-258`) | PRESERVE | Primary transport. It must stay reachable with one click |
| TL-002 | Play or pause from the keyboard | `Space` (global) | Not while a text field has focus | `SC:44-47` | `togglePlayback` | PRESERVE | Standard NLE key |
| TL-003 | Keep playing while scrubbing | A seek during playback does not stop playback (Ableton-style) | Playing | `TL/frame.dart:38-43` | native `seek` (`P:260`) | PRESERVE | Behaviour contract, not a control |
| TL-004 | Play past the duration (no wall, no loop) | Playback continues past `durationFrames`. The duration is only a mark | — | `motolii/crates/motolii-render/src/playback.rs:415-417` | native clock | PRESERVE (note) | **There is no Loop or work-area control anywhere in the current UI.** Record this as absent, not as lost |
| TL-005 | Jump to start | `Home` (global) | Not typing | `SC:120-123` | `seek(0)` | PRESERVE | Standard key |
| TL-006 | Jump to last frame | `End` (global) | Not typing | `SC:124-127` | `seek(durationFrames-1)` | PRESERVE | Standard key |
| TL-007 | Step the playhead by ±1 / ±10 frames | `←`/`→`, with `Shift` for ×10 (global) | Not typing and `Alt` not held | `SC:128-139` | `seek(frame±d)` | PRESERVE | Frame stepping |
| TL-008 | Step the playhead, or move selected keys, when Timeline has focus | `←`/`→` (`Shift` ×10) while Timeline has focus | Moves keys if any are selected and `moveKeys` is supported; otherwise seeks (clamped ≥ 0) | `TL/view.dart:29-39` | `moveKeys{deltaFrames}` or `seek` | PRESERVE + flag | Same keys mean something different when Timeline has focus (see duplicates D-3) |
| TL-009 | Current frame and duration readout | Painted in the name column of the ruler: `frame / duration` | Always | `TL/paint.dart:437-444` | UI-local | PRESERVE / VISUALIZE | Time readout. It could become an editable timecode |
| TL-010 | "Layers" header label | Painted in the ruler's name column | Always | `TL/paint.dart:428-436` | UI-local | PRESERVE | Column header |

### Playhead and ruler

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-020 | Click the ruler to seek | Tap-down on the ruler, right of the name column | x ≥ labelWidth | `motolii/ui/lib/panels/timeline.dart:52-55` | `requestSeek` → serialized native `seek` (latest request wins) | PRESERVE | Core scrub |
| TL-021 | Drag the ruler to scrub | Horizontal drag on the ruler | x ≥ labelWidth | `timeline.dart:56-59`, `TL/frame.dart:38-56` | `seek` pump. The head is drawn from `scrubFrame` before native replies | PRESERVE | Core scrub with optimistic draw |
| TL-022 | Ruler time labels | Seconds (`1s` / `0.50s`) and frames (`30f`) at grid steps, with sub-ticks | Always | `TL/paint.dart:129-183` | UI-local | PRESERVE | Dual time units |
| TL-023 | Playhead line and head across the ruler, lanes and overview | Painted | Always | `TL/paint.dart:385-399`, `TL/overview.dart:68-73` | UI-local | PRESERVE | Now-indicator |
| TL-024 | Marker ticks | Vertical line in lanes and a dot on the ruler for each marker | Markers exist | `TL/paint.dart:375-384` | read `state.markers` | PRESERVE / VISUALIZE | Markers can be seen but not edited (TL-031) |
| TL-025 | Auto-extend the timeline when the playhead passes the content end | Automatic. Grows by 2× the visible width in one step | frame > content extent | `TL/frame.dart:96-106` | UI-local | PRESERVE | "Duration is a mark, not a wall" |

### Markers

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-030 | Add a marker at the playhead | Timeline bar button `Marker` | `has('addMarker')`, otherwise disabled | `motolii/ui/lib/panels/timeline.dart:371-379` | native `addMarker` → `edit_marker` (named by frame number, no duplicate at the same time) `P:344-349` | PRESERVE / MERGE with TL-032 | Only marker authoring control |
| TL-031 | Rename, edit the body of, delete, or move a marker | **Not reachable in UI.** Native `setMarker` / `deleteMarker` exist (`P:232, 347`) and appear in `protocol.dart:50-51`, but no Dart caller | — | — | native only | (gap) PRESERVE-in-native, expose in new UI | Capability exists underneath but no surface reaches it. Do not count it as a Classic UI capability |
| TL-032 | Add a marker from the keyboard | `M` (global) | Not typing and no modifier | `SC:158` | `addMarker` | PRESERVE | Shortcut |

### Time zoom and horizontal navigation

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-040 | Zoom out one step | Bar button `−`: −1 % around the view centre (range 3–1000 %; 100 % = 4 px/frame) | — | `motolii/ui/lib/panels/timeline.dart:262-269`, `TL/view.dart:88-91` | UI-local `zoom()` | MERGE | Three controls for one zoom value (D-1) |
| TL-041 | Type or drag the zoom % | `EditorPercentField` "Timeline zoom": drag-scrub (speed 1) or type. `Esc` restores the value from before the drag | — | `timeline.dart:270-276`, `motolii/ui/lib/foundation/panel_controls/scale.dart:50-97` | UI-local | MERGE / VISUALIZE | Numeric zoom |
| TL-042 | Zoom in one step | Bar button `+`: +1 % | — | `timeline.dart:277-282` | UI-local | MERGE | See TL-040 |
| TL-043 | Fit everything | Bar button `Fit`: fits content extent (layers + ghosts + duration) to the view width | — | `timeline.dart:283-291` | UI-local | MERGE with TL-046 | Same action as double-clicking the overview |
| TL-044 | Overview lens: jump the view | Primary press on the overview strip centres the view on that x | Scroll clients exist | `timeline.dart:299-321` | UI-local | PRESERVE / VISUALIZE | Mini-map navigation |
| TL-045 | Overview lens: drag to pan | Horizontal drag on the overview | — | `timeline.dart:328-329` | UI-local | PRESERVE | Mini-map pan |
| TL-046 | Overview lens: double-click to Fit | Double-tap on the overview | — | `timeline.dart:330-338` | UI-local | MERGE with TL-043 | Duplicate of Fit |
| TL-047 | Overview contents | Every layer bar (hidden layers greyed), the visible window as a rounded box, a duration mark when content is longer, the playhead | Always | `TL/overview.dart:27-75` | UI-local | PRESERVE / VISUALIZE | Arrangement map |
| TL-048 | Scroll-wheel pan | Mouse or trackpad scroll over the Timeline pans both axes | No modifier and not over the ruler | `TL/view.dart:147-152` | UI-local | PRESERVE | Navigation |
| TL-049 | Shift+scroll horizontal pan | `Shift` + wheel moves time (dy + dx) | — | `TL/view.dart:141-146` | UI-local | PRESERVE | Mouse-wheel users |
| TL-050 | Zoom at the pointer with the wheel | Primary + wheel anywhere, **or** plain wheel over the ruler: zoom anchored at the pointer x | — | `TL/view.dart:136-140` | UI-local | PRESERVE | Contract: "scroll on the ruler zooms around the time under the pointer" |
| TL-051 | Trackpad pan with inertia | Two-finger pan in the lanes. Momentum stops on the next pointer down | Trackpad | `TL/view.dart:103-131`, `motolii/ui/lib/input/viewport_motion.dart` | UI-local | PRESERVE | Contract (input and inertia) |
| TL-052 | Trackpad pinch zoom | Pinch in the lanes | Trackpad | `TL/view.dart:105-124` | UI-local | PRESERVE | Contract |
| TL-053 | Trackpad scrub-zoom on the ruler | Two-finger gesture that starts over the ruler: vertical motion zooms around the time under the pointer | Trackpad, starts on ruler | `TL/view.dart:111-113`, `viewport_motion.dart:62` | UI-local | PRESERVE | Contract |
| TL-054 | Horizontal scrollbar | Scrollbar strip under the lanes (always shows its thumb) | — | `motolii/ui/lib/panels/timeline.dart:385-428` | UI-local | PRESERVE | Mouse users |
| TL-055 | Vertical scrollbar for rows | `EditorScrollbar` on the rows | Rows taller than the view | `timeline.dart:99-105` | UI-local | PRESERVE | Mouse users |
| TL-056 | Tell the session how many frames are visible | Automatic. `visibleFrames` is passed to `create` / `placeAsset` so new layers are sized to what is visible | Always | `TL/frame.dart:183-190`, `motolii/ui/lib/session/session_commands.dart:66-69` | session `visibleFrames` → native args | PRESERVE (hidden coupling) | **The new UI must still publish this or new layer lengths change** |

### Name column and row layout

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-060 | Resize the name column | Drag the grip between names and lanes (resize cursor). Clamp 114…162 + nesting indentation | — | `motolii/ui/lib/panels/timeline.dart:217-236, 430-466` | UI-local | PRESERVE | Contract (deep hierarchies) |
| TL-061 | Reset the name column width | Double-click the grip → 138 | — | `timeline.dart:232-235` | UI-local | PRESERVE | — |
| TL-062 | Cancel a column resize | Drag cancel restores the start width | — | `timeline.dart:228-231` | UI-local | PRESERVE | — |
| TL-063 | Nested group containers | Groups draw as recursive boxes. Children are indented 8 px per level. The column widens with depth | Group layers | `TL/layout.dart:55-167`, `TL/paint.dart:414-426` | UI-local | PRESERVE / VISUALIZE | Contract: "group is a recursive container" |
| TL-064 | Row colour | Each layer's name-cell background and bar use `timelineColor(id)`, which is **derived from the id** | Always | `TL/paint.dart:220, 417-420` | UI-local | PRESERVE (note) | **No user-editable colour label exists.** Record as absent |
| TL-065 | Rename a layer | **Not reachable from Timeline**: no double-click or edit field on the name. Check the Inspector inventory | — | — | — | (n/a here) | Record as absent from Timeline |

### Layer rows: selection

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-070 | Select one layer | Click the name cell of a layer row (not on a toggle) | — | `TL/grip.dart:176-186, 84-102` | native `select{ids:[id], keys:[]}` | PRESERVE | Core |
| TL-071 | Toggle a layer in or out of the selection | Primary + click the name | — | `TL/grip.dart:95-96` | `select` | PRESERVE | Standard multi-select |
| TL-072 | Range-select layers | `Shift` + click selects the range from the last clicked anchor, in document layer order | Anchor set | `TL/grip.dart:86-94` | `select` | PRESERVE | Standard |
| TL-073 | Add a range to the selection | Primary + `Shift` + click unions the range with the current selection | Anchor set | `TL/grip.dart:93` | `select` | PRESERVE | Additive range |
| TL-074 | Click an already-selected row to make it the only selection | Press on a selected name without dragging (< 4 px). The selection collapses to that row on release | Row already selected | `TL/grip.dart:177-181, 447-448` | `select` | PRESERVE | Keeps the multi-select for dragging |
| TL-075 | Select a layer from its property row | Click the name of a property sub-row (not its ◆) | Lanes open | `TL/grip.dart:182-185` | `select` layer | PRESERVE | — |
| TL-076 | Select all layers | `Cmd/Ctrl+A` (global) | Not typing | `SC:75-78` | `select{ids: all}` | PRESERVE | Standard |
| TL-077 | Select the previous or next layer | `↑`/`↓` (global, no Alt). With `Shift` it steps the index ×10 but stops at the ends | Layers exist | `SC:141-157` | `select` single | PRESERVE | Keyboard navigation |
| TL-078 | Deselect all | `Esc` (global) when no sheet and no manually opened Desk | — | `SC:34-43` | `select{ids:[],keys:[]}` | PRESERVE | Esc also cancels previews first |
| TL-079 | Click empty lane space to deselect | Click (< 3 px) in empty lane space or empty clip area without a modifier | — | `TL/grip.dart:288, 470-526` | `select{ids:[], keys:[]}` | PRESERVE | Standard |
| TL-080 | Selection highlight | Selected rows get a lifted surface from the name cell to the time field, and a lighter bar. The focused lane (`activeLane`) gets a raised background | — | `TL/paint.dart:264-270, 445-464`, `TL/grip.dart:122` | UI-local | PRESERVE / VISUALIZE | Contract: "the selection is a region" |

### Layer rows: reorder and nesting

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-085 | Reorder layers by dragging the name | Drag a name ≥ 4 px. Moves the whole selection when the row is selected, otherwise just this row. Drop line before/after the target row | `has('moveLayers')`. Otherwise shows the error "Layer move is waiting for the native update" | `TL/grip.dart:294-303, 334-372, 440-451` | native `moveLayers{layers,target,placement:'before'/'after'}` (`P:202`) | PRESERVE | Contract: commits only on release, keeps order and descendants |
| TL-086 | Put layers into a group | Drag onto the middle of a group row (more than 5 px from its top and bottom). The drop outline boxes the group | Target is a Group | `TL/grip.dart:349-363` | `moveLayers{placement:'inside'}` | PRESERVE | Parent/child by drag |
| TL-087 | Move layers to the root end | Drop below the last row | — | `TL/grip.dart:338-344` | `moveLayers{target:null, placement:'rootEnd'}` | PRESERVE | Take a layer out of its parent |
| TL-088 | Refuse a drop into self or a descendant | Silently no drop target | Target is a moved row or its descendant | `TL/grip.dart:347-348` | UI-local guard (native also validates) | PRESERVE | Contract |
| TL-089 | Group the selection | Context menu "Group" (⌘G), `Cmd+G`, Edit ▸ Group | `has('group')` | `TL/menu.dart:29`, `SC:74`, `motolii/ui/lib/app/editor_window.dart:487-498` | native `group` (`P:200`) | PRESERVE / MERGE (menu ↔ shortcut) | Nesting creation |
| TL-090 | Ungroup | Context menu "Ungroup" (⇧⌘G), `Cmd+Shift+G`, Edit ▸ Ungroup | `has('ungroup')`. Errors "Select a Group" otherwise | `TL/menu.dart:30`, `SC:74` | native `ungroup` (`P:201`) | PRESERVE | Children reparent (contract) |

### Layer rows: expand, collapse and lanes

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-095 | Show or hide a group's children | Click the `⊟`/`⊞` disclosure (left 20 px of a group row) | Group row | `TL/grip.dart:138-149`, `TL/paint.dart:465-472` | UI-local `collapsedGroups` | PRESERVE | Contract: independent of the key lanes |
| TL-096 | Open or close keyframe lanes (animated properties only) | Click the `◇`/`◆` box left of M (x = labelWidth−81…−65). Filled accent when open. This also clears "all properties" mode | Layer row | `TL/grip.dart:125-137`, `TL/paint.dart:482-496` | UI-local `expanded` | PRESERVE | Contract: "the ◇ left of M opens and closes the keyframe lanes" |
| TL-097 | Show animated properties | Right-click row ▸ "Show animated properties" | Right-click on a row | `TL/menu.dart:47-50, 87-97` | UI-local | PRESERVE / MERGE with TL-096 | Same as the ◇ open |
| TL-098 | Show all properties (including unkeyed) | Right-click row ▸ "Show all properties" | Row | `TL/menu.dart:51-54, 94` | UI-local `allProperties` | PRESERVE | Only route to unkeyed property lanes |
| TL-099 | Hide properties | Right-click row ▸ "Hide properties" | Row | `TL/menu.dart:55-58, 90-91` | UI-local | MERGE with TL-096 | Same as the ◆ close |
| TL-100 | Content (text) key lane | An extra "Content" property row appears when a layer has `contentKeys` | Lanes open | `TL/layout.dart:121-128` | read-only layout | PRESERVE | Text-content keys are a separate target (contract) |
| TL-101 | Collapsed-group summary | A closed group paints each child's colour and time range as thin stripes. When open, the stripes are grey | Group row | `TL/paint.dart:187-212` | UI-local | PRESERVE / VISUALIZE | Contract |

### Layer rows: per-row switches (M / S / L / ↳)

The switch cluster is four 14 px boxes at x = labelWidth−65 + 16·i (`TL/grip.dart:150-167`, painted `TL/paint.dart:497-522`). A box is filled accent when the switch is on.

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-105 | **M = Mute (hidden)** | Click `M`. Toggles `layer.hidden`. A hidden layer's bar is drawn as `raised` grey, also in the overview | `has('setAttrs')` | `TL/grip.dart:152-164` | native `setAttrs{layers:[id], patch:{hidden:!v}}` (`P:181-188`) | PRESERVE | Contract: M = mute, S = solo, L = lock. Keep the letters |
| TL-106 | **S = Solo** | Click `S`. Toggles `layer.solo` | `has('setAttrs')` | same | `setAttrs{solo}` | PRESERVE | Contract |
| TL-107 | **L = Lock** | Click `L`. Toggles `layer.locked`. A locked layer cannot be timing-dragged (TL-116). In Depth it can be selected but not dragged | `has('setAttrs')` | same | `setAttrs{locked}` | PRESERVE | Contract |
| TL-108 | **↳ = Clip to below (clipping mask)** | Click `↳` (the "arrow/link" button). Toggles `clipToBelow`: this layer is clipped by the layer below (Photoshop / CSP style) | `has('clip')` | `TL/grip.dart:158-159` | native `clip{layer}` → `Intent::SetAttrs{clip_to_below:!v}` (`P:230`) | PRESERVE | Contract: the Timeline clipping mask is the mask entry point |
| TL-109 | Switches act on one row only | Clicking a switch patches that row even when several rows are selected | — | `TL/grip.dart:161-163` (`layers:[row.id]`) | — | PRESERVE (note) | Behaviour to keep, or deliberately extend later |
| TL-110 | Freeze a layer or group | Right-click row ▸ "Freeze" (Ableton Freeze Track): renders its in–out range to a cache in the background | Row targeted and `has('freeze')` | `TL/menu.dart:39-46, 98-104` | native `freeze{layer,enabled:true}` → `Intent::Freeze` + freezer job (`P:262-278`) | PRESERVE / CONTEXTUALIZE | Performance feature. Progress is shown elsewhere (`editor_window.dart:24`) |
| TL-111 | Unfreeze | Right-click a frozen row ▸ "Unfreeze". Cancels a running bake | `layer.frozen == true` | same | `freeze{enabled:false}` → `Intent::Unfreeze` | PRESERVE | Label changes with the state |

### Clips (layer bars): move, trim, slip, split

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-115 | Select a layer by clicking its bar | Press on the bar (±4 px) | Not already selected | `TL/grip.dart:261-262` | `select` | PRESERVE | — |
| TL-116 | Move a clip in time | Drag the bar body. Start clamps ≥ 0. Live preview while dragging, one commit on release | Unlocked, and `setTiming` or `setTimings` | `TL/grip.dart:253-286, 292-331, 401-436, 438-469` | during drag: `previewTimings{changes}`, on release: `commitPreview`. Fallback `setTimings` / `setTiming` (`P:205, 207`). A pure move retimes keys too | PRESERVE | Core NLE |
| TL-117 | Move several clips together | Drag a bar that is part of a multi-selection. Every selected unlocked layer moves | `has('setTimings')` | `TL/grip.dart:263-276` | `previewTimings` / `setTimings` | PRESERVE | Batch timing |
| TL-118 | Trim the in point | Drag within 6 px of the bar's left edge. Changes start, duration and sourceIn together (clamped) | Unlocked | `TL/grip.dart:279-280, 406-413` | same timing path | PRESERVE | Standard trim |
| TL-119 | Trim the out point | Drag within 6 px of the right edge. Duration ≥ 1 | Unlocked | `TL/grip.dart:281-282, 414-420` | same | PRESERVE | Standard trim |
| TL-120 | Slip the source | `Alt` + drag anywhere on the bar. Changes sourceIn ≥ 0; start and duration stay | Unlocked | `TL/grip.dart:277-278, 421-427` | same | PRESERVE / CONTEXTUALIZE | Hidden modifier. Worth a visible affordance |
| TL-121 | Cancel a timing drag | `Esc` while dragging (Timeline focus), pointer cancel, or dispose | Preview active | `TL/view.dart:24-28`, `TL/grip.dart:535-542` | `cancelPreview` | PRESERVE | Undo-free abort |
| TL-122 | Live bar drawing while dragging | Bars follow the pointer from their press-time timing + delta, before native replies | During a drag | `motolii/ui/lib/panels/timeline.dart:177-194` | UI-local | PRESERVE | Feel |
| TL-123 | Split at the playhead | Context menu "Split" (⌘K), `Cmd+K`, Edit ▸ Split | `has('split')`. Playhead must be strictly inside the layer, otherwise the error "Playhead must be inside a layer" | `TL/menu.dart:31`, `SC:91-96` | native `split` → `split_layers` (`P:204`) | PRESERVE | Standard. Selects the new copies |
| TL-124 | Ghost extension display | The grey bar extension for a layer's ghost delay (drawn outside the real bar). A notch at frame 0 when it starts earlier. The ghost itself cannot be grabbed | `layer.ghost` set | `TL/paint.dart:222-261` | UI-local (edited in Inspector / Ease Sequence) | PRESERVE / VISUALIZE | Shows the result of DK-Ease sequence |
| TL-125 | Group bar drag | A group row's bar area is hit-tested against the group layer's own start and duration, so move and trim apply to the group | Group row | `TL/grip.dart:253-286` | timing path | PRESERVE (verify) | Behaviour exists implicitly. Confirm in the window |
| TL-126 | Drop a Media asset onto the Timeline | Drag an asset from Media over the rows. The drop line or box shows before/after/inside/rootEnd. Dropping over the lanes also sets the start frame from x | `data.asset` present and `has('placeAsset')` | `motolii/ui/lib/panels/timeline.dart:106-116`, `TL/grip.dart:374-399` | native `placeAsset{id,target,placement,start?}` (+ `visibleFrames`) | PRESERVE | Cross-panel drop |

### Keyframes

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-130 | Add or remove a key at the playhead | Click `◆`/`◇` at the right end of a property row's name (x ≥ labelWidth−25). `◆` = keyed at the current frame | `has('toggleKey')`, property row. Content rows use a special path | `TL/grip.dart:168-175`, `TL/paint.dart:531-537` | native `toggleKey{layer,property}` (`P:208-214`): deletes a key at this frame, or inserts the current value (Linear) | PRESERVE | AE stopwatch or "add key" |
| TL-131 | Select one key | Click a key diamond (±7 px) on a property row | — | `TL/grip.dart:218-252` | `select{ids, keys:[…]}` | PRESERVE | Core |
| TL-132 | Toggle keys in or out of the selection | Primary or `Shift` + click a key | — | `TL/grip.dart:112, 236-243` | `select` | PRESERVE | Standard additive |
| TL-133 | Select an ease span (both ends) | Click the line between two keys (more than 7 px from either key). Selects both ends. No drag | Property row | `TL/grip.dart:62-73, 231-249` | `select` keys | PRESERVE / VISUALIZE | Span = the ease lives here (feeds DK-Ease) |
| TL-134 | Ease span look | Dashed while Linear, solid once shaped, accent and 2 px thick when both ends are selected | Property row | `TL/paint.dart:311-339` | UI-local | PRESERVE / VISUALIZE | Tells the interpolation state at a glance |
| TL-135 | Select every key at one frame from a folded layer | Click a summary diamond on a folded layer bar. Selects every key of the layer at that frame (properties, content, effect params). Primary or `Shift` toggles | Lanes closed | `TL/grip.dart:188-217`, `TL/layout.dart:30-52` | `select` keys | PRESERVE | AE-style folded keys |
| TL-136 | Move keys by dragging | Drag a selected key or summary diamond. Clamped so no key goes below frame 0. Drawn with an offset until native replies | `has('moveKeys')`, not a span | `TL/grip.dart:213, 249, 318-322, 452-465` | native `moveKeys{deltaFrames}` (`P:215`) | PRESERVE | Core |
| TL-137 | Marquee-select keys and layers | Drag in empty lane space or outside rows. Selects keys inside (property rows and folded summaries) and layers whose bars overlap | Drag > 3 px | `TL/grip.dart:470-526`, `TL/paint.dart:400-407` | `select{ids,keys}` | PRESERVE | Core |
| TL-138 | Add a marquee to the selection | Primary or `Shift` + marquee keeps the previous keys and ids | — | `TL/grip.dart:112, 472-473` | `select` | PRESERVE | Standard |
| TL-139 | Nudge keys by a frame or 10 frames | `Alt`+`←`/`→` (global, `Shift` ×10) when keys are selected. With no keys, it nudges the layer on the Stage instead | Keys selected | `SC:132-137` | `moveKeys` / `stageGesture` | PRESERVE / MERGE with TL-008 | Keyboard key nudge |
| TL-140 | Delete selected keys | `Delete`/`Backspace`, context menu Delete, Edit ▸ Delete. With keys selected this removes keys, not layers | Keys selected | `SC:118-119`, `TL/menu.dart:28` | native `delete` → `delete_key_selection_intents` (`P:116-118`) | PRESERVE | Context-dependent delete |
| TL-141 | Copy keys | `Cmd+C`, context menu, Edit menu | Keys selected. With no keys it copies layers | `SC:63`, `TL/menu.dart:24` | native `copy` → `copy_keys` (`P:190-193`) | PRESERVE | — |
| TL-142 | Cut keys | `Cmd+X` / menu | Keys selected | `SC:64` | `cut` = copy + delete | PRESERVE | — |
| TL-143 | Paste keys at the playhead | `Cmd+V` / menu. Keys land at the current frame; the fallback target is the selected layer (cross-document is allowed) | Clipboard holds keys | `SC:65` | native `paste` → `Clipboard::paste(…, frame)` (`editor/clipboard.rs:137-160`) | PRESERVE | — |
| TL-144 | Duplicate keys | `Cmd+D` / menu with keys selected: pastes a copy one frame after the latest selected key | Keys selected | `SC:66-69`, `P:195-198` | native `duplicate` | PRESERVE | — |
| TL-145 | Restore the previous key selection | `Cmd+Shift+D` | `previousKeys` recorded | `SC:66-72`, `motolii/ui/lib/session/session_commands.dart:43-47` | `select(previousKeys)` | PRESERVE / CONTEXTUALIZE | Hidden. Needs discoverability |
| TL-146 | Easy Ease on the selected keys | `F9` = Easy Ease, `Shift+F9` = Easy Ease In, `Cmd+Shift+F9` = Easy Ease Out. Also opens Ease in the Desk (focus "keyframes") | Keys selected | `SC:48-60`, `P:336-337`, `motolii/ui/native/src/editor/ease.rs:43-51` | native `ease{kind:EasyEase*}` | PRESERVE | AE F9 family. The comment in `ease.rs` says ⌘F9 = Out, but the code needs ⌘⇧F9 (flag) |

### Layer edit commands (context menu, shortcuts, Edit menu)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| TL-150 | Timeline right-click menu | Secondary tap in the rows area. First selects the row under the pointer if it is not already selected | — | `TL/menu.dart:11-107`, `motolii/ui/lib/panels/timeline.dart:117` | — | PRESERVE / CONTEXTUALIZE | Row items appear only over a row. Commands are disabled per capability |
| TL-151 | Copy layers | Menu "Copy ⌘C", `Cmd+C`, Edit ▸ Copy | No keys selected, layers selected | `TL/menu.dart:24`, `SC:63` | native `copy` → `copy_layers` (includes nested grandchildren) | PRESERVE / MERGE | — |
| TL-152 | Cut layers | Menu "Cut ⌘X", `Cmd+X`, Edit | — | `TL/menu.dart:25` | `cut` | PRESERVE / MERGE | — |
| TL-153 | Paste layers | Menu "Paste ⌘V", `Cmd+V`, Edit | Layer payload | `TL/menu.dart:26` | `paste` → `paste_layers` | PRESERVE / MERGE | Also works over empty space (menu without row items) |
| TL-154 | Duplicate layers | Menu "Duplicate ⌘D", `Cmd+D`, Edit | Layers, no keys | `TL/menu.dart:27`, `P:195-196` | native `duplicate` → `duplicate_layers` | PRESERVE / MERGE | — |
| TL-155 | Delete layers | Menu "Delete ⌫", `Delete`/`Backspace`, Edit. Deleting a parent removes its descendants | No keys selected | `TL/menu.dart:28`, `P:118` | native `delete` → `RemoveLayer` | PRESERVE / MERGE | Contract: parent delete takes children, undone as one step |
| TL-156 | Undo / redo | `Cmd+Z` / `Cmd+Shift+Z`, Edit ▸ Undo/Redo | — | `SC:62` | native `undo` / `redo` (`P:247-248`) | PRESERVE | Also see DK-History |

### Desk (auxiliary tool drawer)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| DK-001 | Open the Desk tools catalog | Desk header button (inbox icon, "Desk tools"). Lists the drawer tools (Depth, Ease, Blend, History) that live in the drawer | — | `motolii/ui/lib/panels/desk.dart:183-190, 129-175` | session `deskDrawer='Tools'` | PRESERVE / MOVE | Contract: Desk holds Depth, Ease, Blend and History |
| DK-002 | Open a tool from the catalog | Click a tool row. A tool promoted to a pane opens with `placePanel(name,'show')` instead | — | `desk.dart:120-126, 136-137` | `deskDrawer=name` or `placePanel` | PRESERVE | — |
| DK-003 | Set the idle-default tool | Star button per row ("Use X when idle"). Toggles between this tool and Tools | — | `desk.dart:151-167` | session `deskDefault` (persisted) | PRESERVE / CONTEXTUALIZE | Personalization |
| DK-004 | Desk follows the selection | Automatic: selected keys → Ease; ≥ 2 selected layers → Ease (Sequence); active Camera → Depth. Frozen while the user is interacting inside the Desk | — | `desk.dart:49-57, 88-98` | UI-local | PRESERVE | Contextual tool (core UX of Desk) |
| DK-005 | Desk follows an editing focus | `editingFocus` keyframes → Ease, Camera → Depth, blendMode → Blend (for example F9 sets keyframes) | Layer selected | `desk.dart:100-118` | UI-local | PRESERVE | Cross-panel handoff |
| DK-006 | Close a manually opened tool | `Esc` (global) clears `deskDrawer` when no sheet is open | — | `SC:38-39` | UI-local | PRESERVE | — |
| DK-007 | Desk header shows the current tool icon and name | Header row. For Ease, the Tools button is inlined into the Ease header instead | — | `desk.dart:182-222` | UI-local | PRESERVE | — |
| DK-008 | Promote a Desk tool to a permanent pane | Settings ▸ panel placement (outside this area). The Desk then skips tools placed elsewhere | — | `desk.dart:47-48`, `motolii/ui/lib/foundation/panel_catalog.dart:89-130` | `panePlaces` | PRESERVE | Contract (panel-placement) |

### Ease desk

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| DK-010 | Target line | Shows `layer · property · from–to f` (+ interval count, Mixed, Read only), or "Sequence · N layers" or "Workspace". The current frame shows `· before`/`· after` when outside the interval | — | `motolii/ui/lib/panels/ease_desk.dart:51-55, 398-407`, `ease_desk/parts.dart:265-322` | UI-local | PRESERVE | docs/stage5/ease.md layout |
| DK-011 | Interval rail | Strip of the selected key intervals on the time axis. The active one is filled; the playhead is pinned on it. The rail stays empty (not hidden) with no target | — | `ease_desk.dart:409-420`, `parts.dart:325-363` | UI-local | PRESERVE / VISUALIZE | ease.md |
| DK-012 | Active interval follows the playhead | The interval under or before the playhead becomes the shown shape | Several intervals selected | `ease_desk/state.dart:233-243, 247-281` | UI-local | PRESERVE | — |
| DK-013 | Drag curve handles | Pointer-down within 12 px of a handle on the square graph. Handles per kind: Bezier 2, Bounce 1, Elastic 2, Cyclic 4, Random 4, Steps 2, ElasticSteps 2, none for Hold and Linear | Primary button | `ease_desk.dart:135-170`, `motolii/ui/native/src/editor/ease_kinds.rs:50-262` | each move: native `easeModel{shape,overshoot,handle,point}` (latest wins). Release: `_commit` → `ease{…, selection}` or `sequence` | PRESERVE / VISUALIZE | Graph first, numbers second |
| DK-014 | Cancel a handle drag | `Esc` while dragging, pointer cancel, window focus loss, or app backgrounded | Drag in progress | `ease_desk.dart:68-78`, `state.dart:283-304` | UI-local restore (+ `cancelPreview` in Sequence mode) | PRESERVE | ease.md |
| DK-015 | Playhead in the graph | Vertical line at the playhead's position within the interval | Interval exists | `ease_desk.dart:56-59, 171-175` | UI-local | PRESERVE | — |
| DK-016 | Curve name and one-line meaning | For example "Bounce — Reach the end, then rebound." While auditioning it reads "PreviewKind · meaning" | — | `parts.dart:184-221`, `ease_desk/values.dart:65-76` | UI-local | PRESERVE | — |
| DK-017 | Preview motion | Play button: runs a single dot along the curve once (UI only). Reduce Motion jumps to the end | — | `parts.dart:223-252`, `state.dart:388-394` | UI-local | PRESERVE | ease.md: writes nothing |
| DK-018 | Numeric curve parameters | One field per numeric param (Bezier ordered x1,y1,x2,y2). Drag-scrub (speed .005), type, `Esc` cancels. Below the preset list when the Desk is short | Shape has numeric params | `ease_desk.dart:104-111, 289-312`, `parts.dart:100-157` | preview: `easeModel`. Commit: `ease` / `sequence` | PRESERVE (VISUALIZE secondary) | Exact values |
| DK-019 | Preset grid | 9 standard kinds (Hold, Linear, Bezier, Bounce, Elastic, Cyclic, Random, Steps, ElasticSteps), then the copied curve, then saved presets. 1–3 columns. The selected tile is marked | — | `ease_desk.dart:39-45, 180-287`, `parts.dart:6-98`, `ease_kinds.rs:4-14` | `state.easeKinds` + `deskWork` | PRESERVE / VISUALIZE | ease.md |
| DK-020 | Apply a preset | Click a tile | Target applies | `ease_desk.dart:276-280`, `state.dart:380-386` | `easeModel` → `ease` / `sequence`, then plays the motion preview | PRESERVE | One-click apply |
| DK-021 | Audition a preset on hover | Hover a tile: the preview and meaning show it, nothing is written. Ends on exit or focus loss | Not mid-drag | `parts.dart:29-31`, `state.dart:396-412` | UI-local | PRESERVE | ease.md: hover writes 0 |
| DK-022 | Keyboard navigation of presets | With the grid focused: arrows (←/→ ±1, ↑/↓ ± a row), `Home`/`End` move and audition; `Enter` applies | Grid focused | `ease_desk.dart:219-257` | as DK-020/021 | PRESERVE | Accessibility |
| DK-023 | Copy the curve | Footer "Copy curve". The curve becomes an extra preset tile (`curveClip`) | — | `ease_desk.dart:325-328` | `storeDesk('curveClip')` (settings file) | PRESERVE | — |
| DK-024 | Save a preset | Footer "Save preset" appends to the saved presets | — | `ease_desk.dart:329-332` | `storeDesk('easePresets')` | PRESERVE | — |
| DK-025 | Use this curve for new keys | Footer "Use for new keys (now X)". Re-arms Animate mode with the new shape if it is on | — | `ease_desk.dart:333-345` | `storeDesk('newKeyShape')` + `animate{enabled,from,shape}` | PRESERVE / MOVE | Could also live beside the Animate toggle |
| DK-026 | Clear saved presets | Footer bin button | Saved presets exist | `ease_desk.dart:346-355` | `storeDesk('easePresets', [])` | PRESERVE | Destructive, no confirm |
| DK-027 | Status and notice text | "N keys selected" / "N layers" / "Workspace", or the last notice ("Curve copied"). Tooltip shows the full target | — | `ease_desk.dart:356-374` | UI-local | PRESERVE | — |
| DK-028 | Overshoot toggle | Checkbox "Overshoot" (icon only when narrow). Lets handles leave 0–1; turned on automatically by presets that overshoot | — | `parts.dart:365-404`, `state.dart:278, 382` | UI flag passed to `easeModel{overshoot}` | PRESERVE | — |
| DK-029 | Apply | "Apply" button: commits the current shape to the selected intervals or the Sequence. Disabled when there is no target, a layer is locked, or the op is unsupported (tooltip says why) | `_canApply` | `parts.dart:406-434`, `state.dart:153-157, 347-378` | `ease{kind,…,selection}` / `sequence` | PRESERVE | — |
| DK-030 | Workspace curve (no target) | With no key interval selected, the Desk edits a stored working curve that is not applied | No keys and < 2 ghostable layers | `state.dart:273-277, 363` | `storeDesk('ease')` | PRESERVE | ease.md |
| DK-031 | Sequence (stagger) mode | Select ≥ 2 ghostable layers and no keys. The curve spreads ghost delays in selection order: total = existing max, or 6 f × (N−1); the first layer gets 0. Marks on the graph show each layer | ≥ 2 ghostable layers | `state.dart:135-210`, `ease_desk.dart:50-52, 128-133` | drag: `previewSequence` (Stage ghosts move live). Commit: `sequence{layers,ghosts}` (one undo) | PRESERVE / CONTEXTUALIZE | AE Sequence Layers without moving in-points |
| DK-032 | Mixed or Read-only flags | The target reads "Mixed" when intervals differ and "Read only" when locked | — | `ease_desk.dart:51-55`, `state.dart:82-87` | UI-local | PRESERVE | — |

### Depth desk

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| DK-040 | Top-down depth map | Plan view (x across, z up) centred on the camera's look-at target. Rings at ¼, ½, ¾ and 1 of range, axes, the camera eye with FOV lines and a direction glyph. Auto-fits so every layer and the camera fit | `depthLayout` from native | `motolii/ui/lib/panels/depth_desk.dart:20-24, 62-135, 310-386` | read `state.depthLayout` | PRESERVE / VISUALIZE | Only overhead view of the 2.5D scene |
| DK-041 | Header: camera and target | Camera icon, then a target icon and the look-at layer's name | Target set | `depth_desk.dart:86-117` | UI-local | PRESERVE | — |
| DK-042 | Layer dots | One coloured dot per layer (`layerColor`). Tooltip shows the name. Selected dots get a heavier ring and a name label | — | `depth_desk.dart:242-296` | UI-local | PRESERVE | — |
| DK-043 | Select a layer in the depth map | Click a dot (13 px) | — | `depth_desk.dart:150-163` | native `select{ids:[id]}` | PRESERVE | — |
| DK-044 | Deselect | Click empty map space | — | `depth_desk.dart:157-160` | `select{ids:[]}` | PRESERVE | — |
| DK-045 | Move a layer in depth | Drag an unlocked dot: moves the layer in the world ground plane | Not locked | `depth_desk.dart:164-166, 200-223` | `previewProperties{position, position.z}` → `commitPreview` on release, `cancelPreview` on pointer cancel | PRESERVE | Z placement is otherwise numeric only |
| DK-046 | Orbit the camera | Click the camera eye (selects the camera), then drag: yaw from the direction, distance = planar distance ÷ cos(pitch) | Camera layer present | `depth_desk.dart:141-149, 171-199` | `previewProperties{camera.orbit, camera.distance}` → `commitPreview` | PRESERVE | — |
| DK-047 | Dispose cancels a pending grab | Closing the desk mid-drag cancels the preview | — | `depth_desk.dart:55-59` | `cancelPreview` | PRESERVE | — |

### Notes desk (Notes panel)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| DK-050 | Page tabs | Horizontal list of page titles. Click switches page and resets selection and view. The current page is accent | Pages exist | `motolii/ui/lib/panels/notes_desk.dart:245-272` | UI-local `_pageId` (flushes editors first) | PRESERVE | — |
| DK-051 | New page | `+` "New page" | — | `notes_desk.dart:72-82, 273-280` | native `notes{action:addPage}` (`motolii/ui/native/src/editor/notes.rs:12-14`) | PRESERVE | — |
| DK-052 | Rename the page | "Page title" draft field (commits on submit or blur) | Page exists | `notes_desk.dart:284-297` | `notes{renamePage}` | PRESERVE | — |
| DK-053 | Delete the page | "Delete page" button (no confirm; undoable through Document) | Page exists | `notes_desk.dart:345-360` | `notes{deletePage}` | PRESERVE | — |
| DK-054 | Click the canvas to write | Tap empty canvas: creates a 220×120 text card there (and a page if none) and focuses it | — | `notes_desk.dart:402-409, 91-102` | `notes{putBlock kind:text}` | PRESERVE | "Click anywhere to write" |
| DK-055 | Edit card text | Type in a text card. Saves 400 ms after the last keystroke and on blur | Text card | `notes_desk.dart:663-682, 515-520` | `notes{patchBlock{text}}` | PRESERVE | — |
| DK-056 | Move a card | Drag the card's top handle bar | — | `notes_desk.dart:598-613, 533-561` | `notes{patchBlock{x,y}}` on release | PRESERVE | — |
| DK-057 | Resize a card | Drag the ↘ corner (min 80×60) | — | `notes_desk.dart:686-704` | `patchBlock{width,height}` | PRESERVE | — |
| DK-058 | Select a card | Tap the handle bar or the text | — | `notes_desk.dart:605, 673` | UI-local `_selection` | PRESERVE | — |
| DK-059 | Delete a card | Card `×` button, **or** `Delete`/`Backspace` with a card selected while the Notes panel has focus and no text field is active | — | `notes_desk.dart:615-629, 230-236` | `notes{deleteBlock}` | PRESERVE | Local key handler |
| DK-060 | Paste into notes | "Paste" button, or `Cmd/Ctrl+V` while Notes has focus (not typing): an image makes an image card, text makes a text card at the last click point | Clipboard holds an image or text | `notes_desk.dart:116-123, 226-229, 303-310` | native `noteClipboard` → `notes{image png}` / `putBlock` | PRESERVE | Overrides the global paste inside Notes (D-5) |
| DK-061 | Insert an image | "Insert image" button → file picker, several files staggered 24 px apart | — | `notes_desk.dart:311-327` | native `pickImport` → `notes{image path}` (re-encoded to PNG, `notes.rs:24-31`) | PRESERVE | — |
| DK-062 | Drop OS files as images | Drag files from Finder onto the Notes canvas | Notes mounted | `notes_desk.dart:42, 125-148` | `c.fileDropTarget` → `notes{image path}` | PRESERVE | — |
| DK-063 | Link the selection | "Link selection" button: a reference card labelled `layer · start–end`, taken from the selected keys or the playhead | — | `notes_desk.dart:150-170, 328-335` | `notes{putBlock kind:reference}` | PRESERVE | Notes ↔ timeline bridge |
| DK-064 | Follow a reference card | Click the reference card label: selects its layer and seeks to its start | Reference card | `notes_desk.dart:648-662` | `select` + `seek` | PRESERVE | — |
| DK-065 | Pan and zoom the note canvas | `InteractiveViewer`: drag to pan, pinch or scroll to zoom 0.25–2× | — | `notes_desk.dart:391-398` | UI-local | PRESERVE | — |
| DK-066 | Reset the note view | "Reset view" button | — | `notes_desk.dart:336-344` | UI-local | PRESERVE | — |
| DK-067 | Import legacy notes and references | "Import previous text / references" button: turns old `deskWork.note(s)` and reference assets into cards | No pages, and legacy data exists | `notes_desk.dart:172-187, 364-386` | `notes{putBlock}` / `notes{image}` | PRESERVE / CONTEXTUALIZE | Migration path. Keep while old data can exist |

### History records

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| DK-070 | History timeline list | Vertical rail with every edit and record, oldest first. Filled dots are reached, outlined dots are the redo tail, the current point is ringed | History exists; otherwise "No history" | `motolii/ui/lib/panels/history_records.dart:68-122, 204-277` | read `state.history` (`motolii/ui/native/src/editor/history.rs`, cap 200) | PRESERVE / VISUALIZE | — |
| DK-071 | Jump to a history point | Tap an entry: undoes or redoes up to that step | `has('historyGoto')`, the entry has a `head` (not from a previous run), not the current entry | `history_records.dart:93, 104, 115-117` | native `historyGoto{head}` (`P:250-256`), errors if unreachable | PRESERVE | Multi-step undo |
| DK-072 | Record markers | Save, Open, End (crash/quit), Warning and Error entries show an icon and a square point. Entries from earlier runs cannot be jumped to | — | `history_records.dart:39-46, 161-170` | read-only | PRESERVE | Session log |
| DK-073 | Timestamps | HH:MM:SS per entry | Panel width ≥ 160 | `history_records.dart:30-37, 186-196` | UI-local | PRESERVE / CONTEXTUALIZE | — |
| DK-074 | Auto-scroll to the current point | The list centres on the current entry when it changes | — | `history_records.dart:48-57` | UI-local | PRESERVE | — |

### Export

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| EX-001 | Open the Export sheet | Top bar button "Export" (toggle). Tap outside closes it; `Esc` closes the sheet | — | `motolii/ui/lib/app/editor_window.dart:500-510, 557, 578-580`, `SC:36-37` | UI-local `sheet` | PRESERVE / MOVE | — |
| EX-002 | Range: All | "All" button: 0 → durationFrames | Default | `motolii/ui/lib/panels/export_controls.dart:58-62` | UI-local (not persisted) | PRESERVE | — |
| EX-003 | Range: marker to marker | "Marker to marker": from the last marker at or before the playhead to the next marker after it (0 or duration when missing) | Markers exist (works without them too) | `export_controls.dart:33-46, 63-67` | UI-local | PRESERVE / VISUALIZE | Only way to export part of the timeline. Could be drawn on the Timeline |
| EX-004 | Summary line | `W × H · fps` and `start–end · MP4 H.264 + AAC` | — | `export_controls.dart:70-72` | read state | PRESERVE | **The format is fixed. There is no format, codec or quality choice** |
| EX-005 | Export… | Button → native save dialog (default `Untitled.mp4`), then starts the export | Range not empty, otherwise the error "Choose a non-empty range" | `export_controls.dart:75-101` | native `pickExport{name}` → `export{path,start,end}` (`P:238`) | PRESERVE | — |
| EX-006 | Progress | Shows `done / total` while running. Polls `exportStatus` every 300 ms | phase running or cancelling | `export_controls.dart:73, 92-100` | `exportStatus` | PRESERVE / VISUALIZE | Could become a progress bar |
| EX-007 | Cancel an export | The same button reads "Cancel" while running | Running | `export_controls.dart:76-77` | `cancelExport` (`P:239`) | PRESERVE | — |
| EX-008 | Export error | Shows `export.error` text | Error | `export_controls.dart:74` | read state | PRESERVE | — |

## UI-local state to preserve

These live in Flutter only (not in the Document). A new projection must re-create them, or hoist them into shared session state, to keep the same behaviour.

- **Timeline view:**
  - `pixelsPerFrame` (zoom, 0.1–40 px/frame, default 4) and the horizontal and vertical scroll offsets (`TL/frame.dart:18-20`).
  - Trackpad momentum (`ViewportMotion`).
  - `grownFrames`, the auto-extension beyond the content (`TL/frame.dart:138`).
- **Timeline rows:**
  - `expanded` (key lanes open per layer), `allProperties` ("show all properties" per layer), `collapsedGroups` (group children hidden).
  - All three are pruned to live ids on every relayout (`TL/frame.dart:15-17, 162-181`). They are not persisted across restarts.
- **Name column width:** `baseNameWidth` (default 138, `TL/frame.dart:24`). Not persisted.
- **Range-select anchor:** `anchor` (last clicked layer) (`TL/grip.dart:14`). **Focused lane:** `activeLane`, cleared when focus leaves (`TL/frame.dart:27, 124-126`).
- **In-flight gestures:**
  - Scrub: `scrubFrame` (optimistic playhead).
  - Timing drag: `queuedTimings` and `previewUsed` (the preview/commit handshake).
  - Key drag: `settlingKeys` / `settlingDelta` (keys stay drawn at their target until native replies).
  - These are behaviour contracts (no snap-back flicker), not just state.
- **`visibleFrames`:** published to the session and consumed by `create` / `placeAsset` for default layer length (TL-056).
- **`previousKeys`:** the prior key selection for `Cmd+Shift+D` (`session_snapshot.dart:39`).
- **Desk:**
  - Shared session notifiers: `deskDrawer` (manual tool), `deskDefault` (starred idle tool, persisted), `panePlaces`.
  - Desk-local: `_automatic` (selection-derived tool) and `_inside` (freeze auto-follow while interacting).
- **Ease desk:**
  - Persisted through `storeDesk`: `deskWork.ease` (workspace curve), `curveClip`, `easePresets`, `newKeyShape` (settings file, `session_files.dart:134-140`).
  - Transient: `_free` (overshoot), `_audition` / `_hover` (peek), `_focused` (grid keyboard index), `_original` (cancel snapshot), `_notice`.
- **Depth desk:** `_grab`, `_start`, `_pending` (preview queue). The auto-fit range is derived.
- **Notes:**
  - `_pageId` (current page), `_selection` (selected card), `_insertion` (last click point, used as the paste and link target), `_transform` (canvas pan and zoom).
  - Per-card dirty text with a 400 ms debounce, registered in `pendingEditors` so save and page switch flush it.
- **History:** `_followed` (auto-scroll bookkeeping).
- **Export:**
  - `markers` range-mode toggle, which is **not persisted** and resets when the sheet reopens.
  - The status poll timer.

## Suspected duplicates / merge candidates (flag only)

- **D-1 Timeline zoom:** `−` / `+` buttons (TL-040/042), the % field (TL-041), Fit (TL-043), overview double-click (TL-046), and wheel, pinch and ruler zoom (TL-050–053) all write `pixelsPerFrame`. Fit and overview double-click are the **same** computation (no behaviour change if merged). The `−` / `+` buttons step by just 1 %, which barely moves at high zoom. Merging them into a slider or log-scale control **changes the step behaviour**.
- **D-2 Lane expansion:** the ◇/◆ toggle (TL-096) = "Show animated properties" / "Hide properties" (TL-097/099). Merging changes nothing. "Show all properties" (TL-098) is only in the menu and must survive the merge.
- **D-3 Arrow keys:**
  - Global `←`/`→` seek and `Alt+←/→` moves keys (TL-007, TL-139).
  - With **Timeline focus**, plain `←`/`→` move keys when any are selected (TL-008).
  - The same key does different things depending on focus. The product contract warns against "the same operation meaning something different in an invisible state". Merging to one rule **changes behaviour** for one of the two paths.
  - Also, the Timeline's `Esc` (cancel gesture and preview) swallows the event, so the global `Esc` (deselect, close Desk) does not run while Timeline has focus. Verify this in the window.
- **D-4 Edit commands, three entry points each:** context menu (TL-150), keyboard (`SC`), and top Edit menu (`editor_window.dart:487-498`) for Copy, Cut, Paste, Duplicate, Delete, Group, Ungroup and Split. Identical ops, so merging the surfaces does not change behaviour. The Timeline menu uses `has()` to disable items; the Edit menu does not (it fires and relies on native errors).
- **D-5 Paste:**
  - Global `Cmd+V` → native `paste` (layers or keys).
  - Inside Notes, `Cmd+V` → `noteClipboard` (OS image or text) instead.
  - These are different clipboards (the editor's internal one vs the OS one). They are not duplicates, so do not merge them blindly.
- **D-6 Delete key:** global `Delete` → layers or keys (TL-140/155). In Notes with a card selected → delete the card (DK-059). The action depends on context; keep the scoping rule.
- **D-7 Key selection:** single click, folded summary click and span click (TL-131/133/135) all go to `select{keys}`. They are distinct gestures, so do not merge the gestures, only the visual language.
- **D-8 Ease entry points:**
  - F9 family (TL-146) and the Ease desk presets and Apply (DK-020/029) both reach native `ease`.
  - Desk auto-follow (DK-004/005) opens Ease from key selection or F9.
  - Candidate: a contextual ease strip on the selected span (TL-133/134) that opens the Desk. This is behaviour-preserving if it still calls `_model → _commit`.
- **D-9 Ghost / Sequence:** DK-031 (Ease Sequence) writes ghost delays that the Timeline only visualizes (TL-124). Per-layer ghost editing lives in the Inspector (`inspector/transform_card.dart:165, 250`, outside this area). Candidate for MERGE into one "stagger" surface. Keep the native ops `sequence`, `previewSequence` and `ghost` separate.
- **D-10 Selecting a layer:** Timeline name, Timeline bar and Depth dot (TL-070/115, DK-043), plus reference cards in Notes (DK-064), all issue `select`. The Depth dot and the reference card pass no `keys`; native clears keys either way, so the behaviour is equal.
- **D-11 Markers:** the Marker button (TL-030) and the `M` key (TL-032) are identical. Export "Marker to marker" (EX-003) is the only other marker consumer. Rename, delete and move are **native-only** (TL-031).
- **D-12 Undo:** `Cmd+Z` / `Cmd+Shift+Z` / Edit menu (TL-156) step one at a time; History (DK-071) jumps many steps. These are complementary, so do not merge them. History could host Undo and Redo buttons without changing behaviour.

## Absent (searched for, not found)

Recorded so the new UI does not assume these existed:

- Loop or work area
- Layer rename in the Timeline
- User colour labels
- Marker edit, delete or drag
- Export format, codec or resolution choice
- Timeline snapping
- Per-key right-click menu (right-click always shows the layer/edit menu)
- Keyframe interpolation menu in the Timeline (it goes through the Ease desk and F9)
- Native `reorder{delta}` op (`P:203`): no caller in the Timeline
