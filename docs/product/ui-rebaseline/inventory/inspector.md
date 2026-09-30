# Capability inventory — Inspector (+ Blend desk, Gradient bar, Rich text box, shared numeric controls)

Audited read-only at `claude/ui-rebaseline` (db7d68118). Paths are repo-relative. Abbreviations:
`I/` = `motolii/ui/lib/panels/inspector/`, `P/` = `motolii/ui/lib/panels/`, `F/` = `motolii/ui/lib/foundation/`,
`N/` = `motolii/ui/native/src/`. "port.rs:N" = the op's handler in `N/port.rs`.

Global conditions used below:
- **shown layer** = last of `selectedIds`, else `activeLayer` (`I/reading.dart:62-67`). Nothing selected → "Select a layer" placeholder (`P/inspector.dart:252-264`).
- **canEdit** = layer not locked AND caps `previewProperties`+`commitPreview` (`I/reading.dart:207-210`). Locked → every write control disabled.
- **multi** = >1 selected. Multi hides Layout/Text/Fill/Matte/effects; Transform/World stay (`P/inspector.dart:307-341`).
- **Write route** for every numeric/choice/pad/dial: drag → `previewProperties` (latest-value queue) → `commitPreview`; Esc / pointer-cancel / window blur / app background → `cancelPreview` (`I/writing.dart:58-141`, `F/panel_controls/drag.dart:14-80`). Multi-target: drag is **relative** (each layer keeps its offset), typed value is **absolute** to all selected unlocked layers (`I/writing.dart:44-94`).

Disposition key: PRESERVE / MOVE / MERGE / CONTEXTUALIZE / VISUALIZE (+gadget type). No DELETE.

---

### A. Panel frame, header, scroll, reveal

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-001 | Empty state "Select a layer" | none selected | no shown layer | P/inspector.dart:252-264 | UI-local | PRESERVE | Tells user why panel is blank |
| IN-002 | Header shows layer colour block, kind glyph (Text/Shape/Camera/Video/Audio/Group/Image), name (ellipsis + full-name tooltip) | always | layer shown | P/inspector.dart:174-214 | UI-local (reads layer) | PRESERVE | Identity of what is edited; ties to Timeline colour |
| IN-003 | Header says "N layers" under multi-select | select >1 | multi | P/inspector.dart:204 | UI-local | PRESERVE | Signals relative/absolute multi-edit mode |
| IN-004 | Expand/collapse ALL effect cards of the layer | click unfold_more/unfold_less glyph in header | !multi && layer has ≥1 effect | P/inspector.dart:215-233 | UI-local (`_closed`) | PRESERVE + MOVE | Bulk fold; could sit on an Effects section head |
| IN-005 | Animate (auto-key) toggle; tooltip varies with Settings "animateFrom" (also keys the frame Animate was turned on) | click diamond switch in header; also `A` key globally | cap `animate` | P/inspector.dart:234-244; session_commands.dart:16-20; input/editor_shortcuts.dart:163-166 | `command('animate',{enabled,from,shape})` → port.rs:222 | PRESERVE + MOVE | Global record mode; belongs with transport/Timeline too, keep in Inspector reach |
| IN-006 | Scroll the card stack; scroll jumps to top when shown layer changes | wheel / scrollbar; selection change | — | P/inspector.dart:278-367; I/reading.dart:45-49 | UI-local | PRESERVE | Stable seat for Transform |
| IN-007 | Reveal a property: opens its card (Transform/Camera/Stage or effect card + effect Advanced fold), scrolls it into view, focuses its first well (Enter then types) | `P`/`S`/`R`/`T`/`Shift+A` shortcuts (show Inspector + focusProperty) | row exists | P/inspector.dart:103-131; input/editor_shortcuts.dart:167-179 | UI-local (`focusProperty`) | PRESERVE | Keyboard-first property access (AE P/S/R/T) |
| IN-008 | Card order is fixed: Camera\|Stage\|Transform, World, Layout, Text, Fill, Matte, effects | — | — | P/inspector.dart:283-321 | UI-local | PRESERVE (principle) / MOVE allowed | "Selection change never pushes Position" — spatial stability |
| IN-009 | Inspector cell-width zoom: `−` / `+` buttons (±1%) | click | always (panel foot) | F/panel_controls/scale.dart:130-136,160,176; P/inspector.dart:369-376 | `storeDesk('inspectorCell', px)` (persisted desk pref) | PRESERVE + MOVE | Density control; could move to panel menu/settings |
| IN-010 | Inspector zoom slider (tap to jump, horizontal drag) | click/drag slider | bar width ≥ sliderRoom | F/panel_controls/scale.dart:162-170; F/leaves/track.dart:107-186 | same as IN-009 | PRESERVE + MOVE | Same |
| IN-011 | Inspector zoom percent field (drag-scrub / double-click type / Esc restores start value, integers, `%`) | drag or type | always | F/panel_controls/scale.dart:50-100,150-157 | same as IN-009 | PRESERVE + MOVE | Exact density |
| IN-012 | Zoom range 88–200 px per cell drives how many columns the card grids (_cells) pack | via IN-009..011 | — | I/grid.dart:22-42 | UI-local layout | PRESERVE | Density |
| IN-013 | Name column shows words when wide, glyph + tooltip when narrow | resize panel | panel width | I/wells.dart:175-199; I/grid.dart:47-54 | UI-local | PRESERVE | Readability at any width |

### B. Card folds (section collapse)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-014 | Collapse/expand any card (Transform, Camera, Stage, World, Layout, Text, Fill, Matte, each effect) | click card head (chevron + title) | always | I/folds.dart:11-33; F/panel_controls/frames.dart:165-291 | UI-local `_closed` (per title; effects per layer+effect id); not persisted | PRESERVE | Focus on what matters |
| IN-015 | "Advanced" fold per effect and per Layout card | click Advanced line | card has advanced rows | I/parts.dart:125-152; F/panel_controls/frames.dart:74-105 | UI-local `_advancedOpen` | PRESERVE + CONTEXTUALIZE | OP-1 "shift" layer; hides seldom rows |

### C. Numeric well (shared by every number in Inspector) — `EditorNumericField` via `_wellBody`

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-016 | Drag-scrub value (3 px dead-zone, horizontal) | press + drag horizontally | enabled | F/panel_controls/numeric.dart:228-339 | previewProperties → commitPreview on release | PRESERVE | Core AE/Figma gesture |
| IN-017 | Precision ladder during drag: move pointer >32 px up = ×10, level ×1, 32–70 px down ×0.1, further ×0.01; rung pill shown over well; tooltip "label ×N" | vertical offset while dragging | dragging | F/panel_controls/numeric.dart:303-339,101-108,463-481 | UI-local modifier | PRESERVE | Fine/coarse control without keys |
| IN-018 | Trackpad two-finger horizontal pan scrubs (Shift ×10); settles on release | two-finger swipe sideways | trackpad | F/panel_controls/numeric.dart:246-268,450-454 | preview → commit | PRESERVE | Laptop-native scrub |
| IN-019 | Horizontal scroll wheel nudges (no button), auto-commit after 300 ms idle; Shift ×10 | scroll sideways | — | F/panel_controls/numeric.dart:270-301 | preview → commit | PRESERVE | Mouse tilt wheel |
| IN-020 | Vertical wheel while left button held steps ±1 unit (Shift ×10) | hold LMB + wheel | — | F/panel_controls/numeric.dart:270-284 | preview → commit | PRESERVE | Mouse precision stepping |
| IN-021 | Type a number: double-click or Enter opens in-place editor (text preselected; mixed opens empty) | double-click / Enter on focused well | enabled | F/panel_controls/numeric.dart:195-208,379-382,449 | UI-local until commit | PRESERVE | Exact numeric entry |
| IN-022 | Commit typed number: Enter or blur; clamps to min/max; non-number → red frame "Number required" and stays open | Enter / click away | editing | F/panel_controls/numeric.dart:150-152,210-224,422 | onCommit → previewProperties + commitPreview (I/writing.dart:92-93) | PRESERVE | Validation |
| IN-023 | Esc cancels typing (reverts), or cancels a running drag (restores) | Esc | editing / dragging | F/panel_controls/numeric.dart:136-147,365-377 | cancelPreview | PRESERVE | Safe abort |
| IN-024 | Arrow Up/Down on focused well steps by `speed` (Shift ×10) and commits | ↑/↓ when well focused (not editing) | enabled | F/panel_controls/numeric.dart:383-394 | previewProperties + commitPreview | PRESERVE | Keyboard nudging |
| IN-025 | Drag speed derived from range: range/300 per px; scale 0.01; angle 0.5; unbounded 1 | implicit | — | I/wells.dart:82-90 | UI-local | PRESERVE | Consistent feel |
| IN-026 | Unit rider inside well: px (position/anchor/camera center/place/size/soft params), ° (angles), % (scale, opacity, 0..1 or 0..100 amounts), declared `unit`, or line-supplied word (cols, rows, px, s) | display | — | I/property_style.dart:163-182; I/wells.dart:76-80; F/panel_controls/numeric.dart:52-55 | UI-local | PRESERVE | Unit handling (display only; typing ignores unit text) |
| IN-027 | Percent presentation: scale and 0..1 `%` rows shown ×100, typed in percent, sent /100 | display/typing | scale, opacity, %-with-max-1 | I/wells.dart:79,91,110-143 | UI-local conversion | PRESERVE | Human units |
| IN-028 | Decimals: 0 for percent/narrow wells/counts, else 2 | display | — | I/wells.dart:124-126 | UI-local | PRESERVE | Digits stay whole |
| IN-029 | zeroWord: value 0 reads as word (grid rows 0 = "auto") while still scrubbable | display | Layout rows | F/panel_controls/numeric.dart:61-65,189-193 | UI-local | PRESERVE | Meaningful zero |
| IN-030 | Mixed value across selection shows "—" | display | multi & values differ | I/wells.dart:61-70; F/panel_controls/numeric.dart:543 | UI-local | PRESERVE | Multi-select honesty |
| IN-031 | Family tint: coloured left rule + flood colour while dragging (place/size=spatial, amount/opacity/level=amount, time, count, seed, angle) | display | declared character | I/property_style.dart:124-139; F/panel_controls/numeric.dart:166-172,487-514 | UI-local | PRESERVE | Glanceable families |
| IN-032 | Track inside well for tight bounded ranges: fill / threshold(level) / steps(count ≤12) / ruler(size,soft), with default tick | display | bounded & `_tight` (or `fill:true`, e.g. Opacity) | I/property_style.dart:141-161; F/panel_controls/numeric_paint.dart; F/panel_controls/numeric.dart:518-533 | UI-local | PRESERVE + VISUALIZE (distribution/curve later) | Already a mini-gadget |
| IN-033 | Value at its default reads quiet (tab ink), moved value reads ink; dotted underline = draggable | display | defaultValue known | F/panel_controls/numeric.dart:553-581 | UI-local | PRESERVE | "Changed from default" cue |
| IN-034 | Hover tooltip "label · drag to adjust · double-click or Enter to type" | hover | — | F/panel_controls/numeric.dart:462-467 | UI-local | PRESERVE | Discoverability |
| IN-035 | Draft typed for one layer is dropped (not committed) if the well is handed another layer | selection change mid-type | — | F/panel_controls/numeric.dart:155-164 | UI-local | PRESERVE | Prevents wrong-layer write |

### D. Keyframe lamp, cell context menu, reset

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-036 | Key lamp in well's corner shows none/keyed/key-at-this-frame (hollow ring on hover when unkeyed) | display | every numeric well | F/panel_controls/toggles.dart:60-150; I/wells.dart:97-104 | reads `keys`,`keyedNow` | PRESERVE | Ableton-style time state |
| IN-037 | Toggle key at current frame (add if absent using current/default value, remove if present) | click lamp | cap `toggleKey` && canEdit | I/wells.dart:99-104 | `toggleKey{layer,property}` → port.rs:208-214 | PRESERVE | Per-property keyframe set/remove |
| IN-038 | Right-click cell menu: "Key this frame" / "Remove key" | right-click on any numeric well, and on any `_control` cell (choice, text, color, layer picker, dial+well, seed) | cap `toggleKey` && canEdit | I/wells.dart:95-96,351-378; I/controls.dart:206-209 | `toggleKey` | PRESERVE + MERGE(with IN-037) | Same action, second route; reaches non-numeric rows the lamp cannot |
| IN-039 | Right-click cell menu: "Reset" to declared default | right-click → Reset | row has numeric `default` (only **effect params** carry one; Transform/Text/Layout rows and vec2 params have Reset disabled) | I/wells.dart:357-372; N/snapshot.rs:488 | previewProperties + commitPreview | PRESERVE (+ fix gap) | Only reset path per property; note limited reach |
| IN-040 | Menu keyboard nav (↑/↓, Esc closes) | keys in open menu | menu open | F/leaves/floating.dart:95-118 | UI-local | PRESERVE | Accessibility |
| IN-041 | Content (text) key toggle exists natively (`toggleKey` property `content`) but Inspector's text box has no lamp/menu for it | — | — | N/port.rs:209; P/rich_text_editor.dart (none) | not reachable here | MOVE (verify Timeline) | Flag: text content keys not keyable from Inspector |

### E. Transform card (non-Camera, non-Stage layers)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-042 | Position X / Y wells | scrub/type/keys (C) | row `position` | I/transform_card.dart:12-21 | previewProperties `position` [x,y] | PRESERVE + VISUALIZE (Stage handle; field gadget) | Brief example |
| IN-043 | Position Z well | same | row `position.z` (native always emits it) | I/transform_card.dart:16-20 | `position.z` | PRESERVE + CONTEXTUALIZE (show when 2.5D/3D) | Z meaningless in 2D but shown |
| IN-044 | Scale X (or single "Scale" when linked & X==Y) | scrub/type | row `scale` | I/transform_card.dart:22-44 | `scale` [x,y] (%) | PRESERVE + VISUALIZE (Stage handle) | Core |
| IN-045 | Scale Y well (hidden when linked and even) | scrub/type | !(linked && even) | I/transform_card.dart:34-38 | `scale` | PRESERVE | — |
| IN-046 | Scale Z well | scrub/type | row `scale.z` | I/transform_card.dart:39-43 | `scale.z` | PRESERVE + CONTEXTUALIZE (3D) | — |
| IN-047 | Scale link toggle (keep proportions; 2:3 at X=4 → 4:6; zero base axis writes value to both; unlinked writes touched axis only). Default ON, not saved | click link glyph | canEdit | I/transform_card.dart:45-53; I/writing.dart:6,24-29 | UI-local state; affects writes | PRESERVE | Only "link" in Inspector; UI-local |
| IN-048 | Rotation (Z) well, ° | scrub/type | row `rotation` | I/transform_card.dart:55-57 | `rotation` | PRESERVE + VISUALIZE (dial/Stage) | — |
| IN-049 | Rotation X and Y wells (3D tilt) | scrub/type | rows `rotation.x`,`rotation.y` (always emitted) | I/transform_card.dart:58-63 | `rotation.x`, `rotation.y` | PRESERVE + CONTEXTUALIZE (3D) | — |
| IN-050 | Rotation dial (drag needle; wraps across ±180; Esc cancels) | drag dial | row `rotation` | I/transform_card.dart:64; I/wells.dart:302-316; F/panel_controls/dials.dart:16-119 | previewProperties `rotation` | PRESERVE + VISUALIZE | Existing visual control |
| IN-051 | Depth (extrusion) well | scrub/type | row `depth` (shape/text/media) && projection ≠ 2D | I/transform_card.dart:65-72 | `depth` | PRESERVE + CONTEXTUALIZE | Already contextual |
| IN-052 | Opacity well with fill track, 0–100 % | scrub/type | row `opacity` | I/transform_card.dart:73-87; I/wells.dart:71-74 | `opacity` | PRESERVE | — |
| IN-053 | Anchor 9-point picker (click a cell sets pivot to that bounds fraction, compensating position) | click cell | cap `anchor` && !locked (needs rendered bounds) | I/transform_card.dart:88-107; F/panel_controls/frames.dart:108-163 | `anchor{layer,xFraction,yFraction}` → port.rs:261 | PRESERVE + VISUALIZE (Stage pivot handle) | Only anchor edit in Inspector |
| IN-054 | Anchor hover preview on Stage (where the pivot would land) | hover a cell | — | I/transform_card.dart:94-95 | `anchorPreview` ValueNotifier (UI-local → Stage) | PRESERVE | Preview before commit |
| IN-055 | Current anchor cell lit (within 5 %) | display | `anchorFraction` | F/panel_controls/frames.dart:141-151 | reads snapshot | PRESERVE | State readout |
| IN-056 | Arbitrary numeric anchor NOT editable here (no `anchor` well; Shift+A only opens Transform) | — | — | I/transform_card.dart:88-107 | — | flag | Gap: exact anchor only via Stage (verify) |

### F. Camera card (replaces Transform when kind = Camera)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-057 | Camera Center X/Y | scrub/type | row `camera.center` | I/transform_card.dart:114-119 | `camera.center` | PRESERVE + VISUALIZE (Stage camera gizmo exists) | — |
| IN-058 | Camera Target Z | scrub/type | row | I/transform_card.dart:120-125 | `camera.target.z` | PRESERVE | — |
| IN-059 | Camera Target layer picker ("None" + other layers) | choose | row | I/transform_card.dart:126-129; I/wells.dart:329-348 | `setProperty{layer,property:'camera.target',value:id}` → port.rs:170 | PRESERVE | Look-at relation |
| IN-060 | Camera Orbit Pitch / Yaw | scrub/type | row | I/transform_card.dart:130-135 | `camera.orbit` | PRESERVE + VISUALIZE (orbit pad; Depth desk has one) | — |
| IN-061 | Camera Distance (labelled "Scale") | scrub/type | row | I/transform_card.dart:136-141 | `camera.distance` | PRESERVE | Label mismatch worth noting |
| IN-062 | Camera Zoom | scrub/type | row | I/transform_card.dart:142-147 | `camera.zoom` | PRESERVE | — |
| IN-063 | Camera Roll well + dial | scrub/type/dial | row | I/transform_card.dart:148-157 | `camera.roll` | PRESERVE + VISUALIZE | — |
| IN-064 | Blend button focuses Depth desk instead of Blend for cameras (World hidden for Camera anyway) | — | kind Camera | P/desk.dart:112-114; P/inspector.dart:307 | — | note | Camera has no World card |

### G. Stage card (kind = Stage)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-065 | Stage margins Left/Top/Right/Bottom as cells (well each, keyable) | scrub/type/menu | kind Stage | P/inspector.dart:290-301; N/snapshot.rs:437-439 | `stage.left/top/right/bottom` | PRESERVE + VISUALIZE (box/margin gadget, Stage handles) | Box model |

### H. World card (all kinds except Camera)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-066 | Space: 2D / 2.5D / 3D segmented buttons | click | cap `setAttrs` && !locked | I/transform_card.dart:170-189; I/parts.dart:173-191 | `setAttrs{layers: ALL selected, patch:{projection}}` → port.rs:181 | PRESERVE | Applies to whole selection |
| IN-067 | Parent picker ("None" + every other layer) | choose | same | I/transform_card.dart:190-210 | `setAttrs{layers:[shown], patch:{parent}}` | PRESERVE + VISUALIZE (relation wire later) | Shown layer only (not selection) |
| IN-068 | Blend button (shows current mode) → opens Blend desk | click | always | I/transform_card.dart:211-221; session_commands.dart:12-14; P/desk.dart:99-117 | `focusEditing(layer,'blendMode')` → Desk shows "Blend" | PRESERVE + MERGE (with Blend desk) | Entry to picker |
| IN-069 | Environment toggle (image lights & surrounds scene) | click switch | kind Image && canSetAttrs | I/transform_card.dart:223-240 | `setAttrs{layers:[shown],patch:{environment}}` | PRESERVE + CONTEXTUALIZE | Image-only |
| IN-070 | Ghost toggle (same layer seen later by a delay; default delay) | click switch | `ghostable` && cap `ghost` && !locked | I/transform_card.dart:241-255 | `ghost{enabled}` → port.rs:228 (applies to all selected ghostable) | PRESERVE + CONTEXTUALIZE | Delay value itself not editable here |
| IN-071 | Clip to layer below toggle | click switch | cap `clip` && !locked | I/transform_card.dart:256-267 | `clip{layer}` (toggles) → port.rs:230 | PRESERVE | — |
| IN-072 | Flag switches compact to glyph-only when narrow | resize | width | I/transform_card.dart:226-259 | UI-local | PRESERVE | Density |

### I. Blend desk (`P/blend_panel.dart`, opened from IN-068)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-073 | 19 blend-mode tiles in W3C family order incl. StencilAlpha / SilhouetteAlpha | view | — | P/blend_panel.dart:30-53 | — | PRESERVE + VISUALIZE (already specimen) | Choose by look |
| IN-074 | Each tile paints the selected layer's colour blended over beds (black→white ramp, blue, orange) | display | selection has `blendPreviews` | P/blend_panel.dart:104-130; N/editor/blend_preview.rs:9-207; N/editor/visual_samples.rs:12 | snapshot `blendPreviews` | PRESERVE | Visual specimen |
| IN-075 | Hover a tile (90 ms debounce, latest wins) previews that mode on Stage | hover / keyboard focus | targets non-empty, cap `previewBlend`, not applying | P/blend_panel.dart:145-189,332-351 | `previewBlend{layer: last target, mode}` → port.rs:231 | PRESERVE | Try before commit |
| IN-076 | Leave tile / Esc / blur / selection change cancels preview | pointer exit, Esc, focus loss | previewing | P/blend_panel.dart:134-143,191-198,239-251 | `cancelPreview` | PRESERVE | — |
| IN-077 | Click tile applies mode to all selected unlocked non-camera layers (one undo step; skipped if already that mode) | click / Enter | cap `setAttrs`, live | P/blend_panel.dart:202-236,344-345,413-421 | `setAttrs{layers,patch:{blendMode}}` | PRESERVE | Multi-layer apply |
| IN-078 | Current mode tile outlined accent; mixed selection shows none | display | — | P/blend_panel.dart:111-112,380-397 | UI-local | PRESERVE | — |
| IN-079 | Grid dims when nothing blendable selected; columns adapt to width (≥70 px tiles) | display | — | P/blend_panel.dart:252-283 | UI-local | PRESERVE | — |

### J. Layout card (Group, or child of a laid-out Group)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-080 | Grid on/off (display grid=2 / none=0; Flex documents read as on) | click grid switch | kind Group && canEdit | I/layout_card.dart:99-114 | previewProperties+commit `layout.display` | PRESERVE | — |
| IN-081 | Columns well ("cols", integer) | scrub/type/key | Group | I/layout_card.dart:116-125 | `layout.grid_columns` | PRESERVE + VISUALIZE (grid gadget) | — |
| IN-082 | Rows well ("rows", 0 = "auto") | scrub/type/key | Group | I/layout_card.dart:126-136 | `layout.grid_rows` | PRESERVE + VISUALIZE (grid) | — |
| IN-083 | Gap well (px) | scrub/type | Group && display≠0 | I/layout_card.dart:141-157 | `layout.gap` | PRESERVE + VISUALIZE (grid) | — |
| IN-084 | Padding X / Y wells (px) | scrub/type | Group && display≠0 | I/layout_card.dart:158-177 | `layout.padding` [x,y] | PRESERVE + VISUALIZE (box) | — |
| IN-085 | Alignment 3×3 pad: justify-content × align-items; snaps to nearest of 9 (Shift holds dot on snaps); Esc cancels | drag pad | Group && display≠0 | I/layout_card.dart:214-265; F/panel_controls/dials.dart:166-315 | previewProperties `layout.justify_content`,`layout.align_items` (written even if row absent) | PRESERVE + VISUALIZE | Already a 2D gadget |
| IN-086 | Width sizing choice (Hug/Fill/Fixed) + width px well (greyed unless Fixed) | choose / scrub | Group or child, row present | I/layout_card.dart:269-294 | `layout.horizontal_sizing`, `layout.width` | PRESERVE | — |
| IN-087 | Height sizing choice + height px well | same | same | I/layout_card.dart:269-294 | `layout.vertical_sizing`, `layout.height` | PRESERVE | — |
| IN-088 | Transition duration (s) + easing choice | scrub / choose | Group && row `layout.transition_duration` present | I/layout_card.dart:188-207 | `layout.transition_duration`, `layout.transition_easing` | PRESERVE + VISUALIZE (curve) | Easing = curve gadget |
| IN-089 | Child: "Ignore layout" (position: absolute) switch | click | laid-out child | I/layout_card.dart:298-323 | `layout.position_type` 0/1 | PRESERVE | — |
| IN-090 | Child grid cell: column start, row start, column span, row span | scrub/type | laid-out child in grid parent | I/layout_card.dart:326-349 | `layout.column_start/row_start/column_span/row_span` | PRESERVE + VISUALIZE (grid cell picker) | — |
| IN-091 | Advanced fold: every other `layout.*` row as generic controls (flex direction/wrap, depth alignment, exclusions, column/row track sizes, margin, hardness, heaviness, flex shrink, transition delay, loop duration/direction, Field, Field Falloff/Scale/Opacity, snap to grid, constraints, snap size, align self…) | open fold | Group or laid-out child | I/layout_card.dart:69-86 | generic `_control` (see K) | PRESERVE + VISUALIZE (Falloff → field; tracks → grid) | Brief's Falloff example lives here |
| IN-092 | Layout choices in lines (`_pick`) have no right-click/lamp (not keyable from here) | — | — | I/wells.dart:249-269 | — | flag | Key gap for sizing/easing choices |

### K. Generic declared-param controls (`_control` — used by Stage, Text rows, Fill rows, Layout Advanced, effects)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-093 | Choice menu (enum / `choices`) | click → pick; menu keys | `_Kind.choice` | I/controls.dart:92-107; F/leaves/choice.dart:12-76 | previewProperties+commit (index) | PRESERVE | — |
| IN-094 | Layer picker param ("None" + other layers) | choose | row `layer:true` | I/controls.dart:87-91; I/wells.dart:329-348 | `setProperty` | PRESERVE + VISUALIZE (relation wire) | Relations future |
| IN-095 | Vec2 param as two wells "Label X/Y" | scrub/type | `vec2` or list value | I/controls.dart:108-116 | previewProperties list | PRESERVE | — |
| IN-096 | Bounded param well (track when range is tight) | scrub/type | min&max | I/controls.dart:117-118 | previewProperties | PRESERVE + VISUALIZE | — |
| IN-097 | Angle param = dial + well (°) | drag dial / scrub | id rotation*/camera.roll, or `.param.` with angle character | I/controls.dart:119-140 | previewProperties | PRESERVE + VISUALIZE | — |
| IN-098 | Text param field (Enter/blur commits, Esc reverts) | type | kind text | I/controls.dart:141-151; F/panel_controls/fields.dart:69-165 | previewProperties+commit | PRESERVE | — |
| IN-099 | Colour param swatch (checker for alpha) → focuses Browser Colors wheel on that property | click swatch | kind color && cap `focusColor` && canEdit | I/controls.dart:152-173; F/color_field.dart:11-35; session_commands.dart:37-41 | `focusColor{layer,property}` → port.rs:218 + show Colors shelf | PRESERVE + MOVE/CONTEXTUALIZE | Picker is Colors shelf |
| IN-100 | Scalar / scale well | scrub/type | fallback | I/controls.dart:174-187 | previewProperties | PRESERVE | — |
| IN-101 | Seed well + die: roll a fresh seed | click die | id ends `.seed` && canEdit | I/controls.dart:176-203; I/parts.dart:155-171 | previewProperties+commit | PRESERVE | Variation |
| IN-102 | Word label over each cell (hero = ink) | display | — | I/parts.dart:78-97 | UI-local | PRESERVE | — |

### L. Text card (kind Text, single selection)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-103 | Multi-line text box edits content in place (IME-safe; composing held) | type | Text && !locked | P/inspector/content_cards.dart:23-28; P/rich_text_editor.dart:334-372 | `previewText{layer,content}` coalesced (P/rich_text_editor.dart:148-154) → port.rs:176 | PRESERVE (MOVE: also on-Stage) | Primary text entry |
| IN-104 | Commit text on blur, or when Save/close flushes pending editors | blur / Save | dirty | P/rich_text_editor.dart:228-269; session_commands.dart:6-10 | `commitPreview` (fallback `setText` → port.rs:180) | PRESERVE | One undo step |
| IN-105 | Esc reverts to text before editing | Esc | dirty && !composing | P/rich_text_editor.dart:190-198,271-283 | `cancelPreview` | PRESERVE | — |
| IN-106 | Box renders runs in proportion (per-style size ratio, known font families) | display | — | P/rich_text_editor.dart:40-96 | UI-local | PRESERVE | WYSIWYG-ish |
| IN-107 | Highlight characters in the Fonts shelf's current scope (all / hiragana / katakana / han / latin / upper-lower…) | display (driven by Fonts shelf) | textStyleTarget.layer == this | P/rich_text_editor.dart:123-130,324-339 | UI-local; publishes `textStyleTarget{layer,scope,text}` on focus/edit | PRESERVE + MERGE (with Fonts shelf) | Rich-text formatting itself (`styleText` font/size per scope) lives on Fonts shelf, not here |
| IN-108 | Font row shows family; click opens Fonts shelf aimed at this layer | click | canEdit && cap `setFont` | P/inspector/content_cards.dart:30-63; session_commands.dart:24-35 | `focusFont` → Browser tab Fonts (setFont / styleText there, port.rs:177-179) | PRESERVE + MOVE/CONTEXTUALIZE | Brief: Fonts contextual on Text |
| IN-109 | Split choice (None / Chars / Words / Lines) | choose / right-click key | row `text_split` | P/inspector/content_cards.dart:15-20,64-67; N/snapshot.rs:449-458 | previewProperties `text_split` | PRESERVE | Per-unit animation source |
| IN-110 | Text Autospace choice | choose | row `text_autospace` | same | `text_autospace` | PRESERVE + CONTEXTUALIZE (CJK) | Typesetting law |
| IN-111 | Text Spacing Trim choice | choose | row `text_spacing_trim` | same | `text_spacing_trim` | PRESERVE + CONTEXTUALIZE (CJK) | Typesetting law |
| IN-112 | Line Height well (0–1000) | scrub/type/key | row `text_style.<id>.line_height` | same; N/editor/functions/read.rs:251-274 | previewProperties | PRESERVE | — |
| IN-113 | Tracking well (−200..200, track) | scrub/type/key | row `text_style.<id>.tracking` | same | previewProperties | PRESERVE + VISUALIZE (spacing ruler) | — |
| IN-114 | Excluded on purpose from Text card: alignment (`text_justify`), size (`*.size`), fill colour — edited on Fonts / Colors shelves | — | — | P/inspector/content_cards.dart:15-20 | — | note (MOVE already) | Must stay reachable via shelves |

### M. Fill card (kind Shape with a fill)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-115 | Fill bar shows solid colour or native-rendered gradient sample (with blend) | display | Shape && fill map | P/gradient_inspector.dart:201-251 | `visualSample{kind:gradient}` (NativeVisualSample) | PRESERVE + VISUALIZE (gradient gadget — already) | Existing gradient gadget |
| IN-116 | Solid fill: click bar → edit colour in Colors shelf | click bar | solid | P/gradient_inspector.dart:215-218,100-109 | `focusColor{layer,slot}` + show Colors | PRESERVE | — |
| IN-117 | Gradient: click empty bar adds a stop at that offset (colour sampled there; ≤32) and selects it | click bar | gradient && `setGradient` && !locked | P/gradient_inspector.dart:111-122,219-224 | `setGradient{slot,addStop}` → port.rs:279; N/editor/gradient.rs:31-86 | PRESERVE | — |
| IN-118 | Click a stop handle: select it and focus its colour in Colors shelf | click stop | enabled | P/gradient_inspector.dart:273-278,100-109 | `focusColor{layer,slot}` | PRESERVE | — |
| IN-119 | Drag a stop along bar (clamped between neighbours) | drag handle | gradient | P/gradient_inspector.dart:124-148,279-301 | previewProperties `fill.stop.<id>.offset` → commitPreview | PRESERVE + VISUALIZE | — |
| IN-120 | Drag a stop off the bar (>32 px vertical) removes it on release | drag away | >2 stops | P/gradient_inspector.dart:127,48-62 | cancelPreview then `setGradient{removeStop}` | PRESERVE | — |
| IN-121 | Delete / Backspace removes selected stop | key | bar focused, >2 stops | P/gradient_inspector.dart:181-190 | `setGradient{removeStop}` | PRESERVE | — |
| IN-122 | Esc cancels stop drag | Esc | dragging | P/gradient_inspector.dart:176-180 | cancelPreview | PRESERVE | — |
| IN-123 | Fill Angle (plain scalar well, no dial) | scrub/type/key | gradient fill | P/inspector/content_cards.dart:74-97; crates/motolii-render/src/picture/shape_props.rs:98-108 | previewProperties `fill.angle` | PRESERVE + VISUALIZE (angle dial / on-Stage axis) | Inconsistent: not rendered as angle |
| IN-124 | Fill Center X/Y | scrub/type | gradient | same | `fill.center` | PRESERVE + VISUALIZE (Stage handle) | — |
| IN-125 | Fill Spread (0–1000) | scrub/type | gradient | same | `fill.spread` | PRESERVE | — |
| IN-126 | Gradient kind (linear/radial/angular/diamond), blend, solid↔gradient mode: NOT in Inspector — on Colors shelf | — | — | P/gradient_inspector.dart:10-13 | `setGradient{kind,blend}`, `setFillMode` (port.rs:279-280) | note | Keep reachable via Colors |

### N. Matte card (legacy)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-127 | Matte Source layer choice | choose | layer has matte && !clipToBelow && !multi | P/inspector/content_cards.dart:101-121 | `setMatte` — **not in native CAPABILITIES → control always disabled** | PRESERVE (read-only) | Legacy data still visible |
| IN-128 | Matte Mode choice (Alpha / Alpha inverted / Luma / Luma inverted) | choose | same | P/inspector/content_cards.dart:122-142 | `setMatte` — same, inert | PRESERVE (read-only) | Same |

### O. Effects cards (one per effect, single selection)

| ID | Capability | Trigger | Condition | Code | Effect path | Disposition | Reason |
|---|---|---|---|---|---|---|---|
| IN-129 | Reorder effect by dragging grip | drag grip | cap `moveEffect`, !frozen | P/inspector/effects_card.dart:165; I/parts.dart:99-121; P/inspector.dart:346-360 | `moveEffect{layer,id,to}` → port.rs:226 (rejects warp after spatial) | PRESERVE | Pipeline order |
| IN-130 | Enable/bypass effect (eye glyph; card dims when off) | click eye | cap `enableEffect` | P/inspector/effects_card.dart:171-185,162 | `enableEffect{layer,id,enabled}` → port.rs:225 | PRESERVE | — |
| IN-131 | Effect menu: Apply earlier / Apply later (one step) | "…" → item | cap `moveEffect`, not at end | P/inspector/effects_card.dart:21-31,55-60 | `moveEffect` | PRESERVE + MERGE (with IN-129) | Keyboard/precise alt |
| IN-132 | Effect menu: "Throw every number within its reach" (randomise bounded params ±20 %, seeds fully, counts whole; one undo) | menu | canEdit | P/inspector/effects_card.dart:32-36,61-62; I/writing.dart:146-171 | previewProperties (all params) + commitPreview | PRESERVE | Ableton dice |
| IN-133 | Effect menu: "Back to where the numbers rest" (reset all numeric params to defaults; one undo) | menu | canEdit | P/inspector/effects_card.dart:37-41,63-64; I/writing.dart:174-186 | previewProperties + commit | PRESERVE | Bulk reset |
| IN-134 | Effect menu: "Expand copies into layers" | menu | placement effect && cap `expandEffect` | P/inspector/effects_card.dart:42-47,65-69 | `expandEffect` → port.rs:223 (selects copies) | PRESERVE | Bake placement to layers |
| IN-135 | Effect menu: Remove effect | menu | cap `removeEffect` | P/inspector/effects_card.dart:48-52,70-74 | `removeEffect` → port.rs:227 | PRESERVE | — |
| IN-136 | Heroes row: declared heroes, else first 4 non-advanced (if >4), label in ink, rule under | display | — | P/inspector/effects_card.dart:96-110,196-203 | UI-local | PRESERVE | OP-1 four knobs |
| IN-137 | Section labels inside effect (declared `section`) | display | — | P/inspector/effects_card.dart:126-130; I/parts.dart:19-38 | UI-local | PRESERVE | — |
| IN-138 | Point pad for paired params (`group` or `name_x`+`name_y`): drag dot (Esc cancels) + two wells | drag / scrub | pair both numeric | P/inspector/effects_card.dart:131-148; I/controls.dart:7-65 | previewProperties both | PRESERVE + VISUALIZE (field/point gadget) | Existing 2D gadget |
| IN-139 | Every declared param via `_control` (choice, layer, vec2, bounded, angle, text, colour, scalar, seed) incl. right-click Reset/Key | see K | — | P/inspector/effects_card.dart:150-155 | see K | PRESERVE + VISUALIZE per character | Declared sheet for free |
| IN-140 | Advanced params behind fold (declared `advanced` or placement grid rows marked advanced) | open fold | has advanced | P/inspector/effects_card.dart:85-95,205-210 | UI-local | PRESERVE | — |
| IN-141 | Frozen layer: effects greyed, not interactive, message "Frozen — effects are baked. Unfreeze to edit." | display | layer frozen | P/inspector.dart:323-345 | UI-local (Freeze entry is Timeline right-click) | PRESERVE | State explanation |
| IN-142 | Adding effects is NOT in Inspector (Browser Effects shelf → `applyEffect`); effect `whole` scope (`scopeEffect`) not shown | — | — | N/port.rs:8 caps | — | note | Keep reachable elsewhere |

---

## Properties that are good VISUALIZE candidates (future 2D gadget)

| Property / group | Gadget type | Existing seed |
|---|---|---|
| Position / Scale / Rotation / Anchor / Camera center | Stage handles (spatial) + field | Stage gizmo, rotation dial, anchor 9-grid |
| Opacity, amounts, levels (track wells) | distribution / curve over time | fill / threshold / steps / ruler tracks (IN-032) |
| `layout.field`, `layout.field_falloff`, `field_scale`, `field_opacity` (Layout Advanced) | **field** | none — rows only |
| Grid columns/rows/gap/padding, grid cell, column/row track sizes | grid / box gadget | alignment 3×3 pad (IN-085) |
| Placement effects (count, along, pick, seed, Each/Random rows) | **point distribution** / along-path | point pad (IN-138) |
| Transition easing, loop direction, split + stagger | **curve** / space-time slope | none here (Ease desk has curves) |
| Gradient stops, fill angle/center/spread | **gradient** | gradient bar (IN-115..122) |
| Blend mode | specimen grid | Blend desk tiles (IN-073) |
| Audio-reactive effect params (if declared) | **spectrum** | none |
| Camera orbit | orbit pad | Depth desk pad |
| Tracking / line height | spacing ruler | ruler track |

---

## Declared rows NOT surfaced by the Inspector (flag — verify Stage/Timeline/Depth before migration)

The snapshot sends these properties but no Inspector card renders them (cards filter by id: Transform ids, `text*`, `fill.*`, `layout.*` only when Layout card shows):
- **Shape** `shape.points`, `shape.outer_radius`, `shape.inner_radius`, `shape.size`, `shape.length`, `shape.stroke_width`, `shape.fill_color` (content_cards comment says they "sit with the shape's other values" — no card does).
- **Shape connect** `connect.from/to`, from/to side, line path, slack, dash, dash gap/offset, trace, handle size (N/snapshot.rs:413-414).
- **Text** `readout.kind`, `readout.of` (N/snapshot.rs:409), `layout.stagger`, `layout.stagger_from`, `layout.from_end` (split schedule), and `hanging_punctuation` (id does not start with `text`, so the Text-card filter drops it — likely a bug).
- **SPACE_ROWS on free layers** (`layout.margin`, hardness, heaviness, flex shrink, transition duration/easing/delay, loop, **Field / Field Falloff / Field Scale / Field Opacity**) — only reachable when the layer is a Group or a laid-out child (Layout card gate `I/layout_card.dart:15-20`).
- **Camera** `camera.framing` (Framing Size), `camera.near_fade`, camera transition rows.
- **Particles** layer: all `particles.*` rows (Transform card has no position etc. for it; no card lists the rest).
- Ghost delay value (only on/off), numeric anchor, text content keys.

## UI-local state to preserve

- `_closed` card folds (per card title; effects per layer+effect) — `I/folds.dart:6`; not persisted.
- `_advancedOpen` Advanced folds (effect id / `layout:<layer>`) — `I/folds.dart:38`.
- `_scaleLocked` scale link, default ON — `I/writing.dart:6`.
- Inspector cell width `deskWork['inspectorCell']` (persisted desk pref) — `I/grid.dart:30-33`.
- `animateFrom` (Settings) and `newKeyShape` (Ease desk) feed the Animate switch — session_core.dart:17; session_commands.dart:16-20.
- `focusProperty` reveal target; `anchorPreview` hover pivot; `textStyleTarget` {layer, scope, text} shared with Fonts shelf; `editingFocus` (Blend desk routing); `pendingEditors` flush-on-save.
- Numeric well transient: typing draft, error, rung, shown value, idle focus node per `id:axis` (`I/wells.dart:6,111-114`).
- Gradient: selected stop index, drag draft, dropping flag.
- Rich text: baseline, dirty, composing, preview queue.
- Blend desk: hover, previewing, cached samples.

## Suspected duplicates / merge candidates

1. **Key toggle**: lamp click (IN-037) vs right-click "Key this frame/Remove key" (IN-038) — same op; merging changes nothing if the menu route is kept for non-numeric rows (the lamp exists only on numeric wells).
2. **Effect reorder**: grip drag (IN-129) vs menu Apply earlier/later (IN-131) — same `moveEffect`; menu is the keyboard/one-step route, keep both reachable.
3. **Reset**: per-cell Reset (IN-039) vs effect-wide "Back to where the numbers rest" (IN-133) — different scope; merge would change behaviour. Note per-cell Reset is disabled for every non-effect row (no `default` sent), while wells know rest values by meaning (`_restOf` scale 1/opacity 1/rotation 0) — unifying would **add** reset to Transform (behaviour change).
4. **Blend**: World "Blend" button (IN-068) is only a router to the Blend desk (IN-073..079); merging into an inline contextual picker would not change document behaviour but removes the separate desk step.
5. **Colour entry**: swatch (IN-099), solid bar (IN-116), stop tap (IN-118) all route to `focusColor` + Colors shelf — one mechanism, three surfaces; safe to unify visually.
6. **Font**: Font row (IN-108) and text-box scope highlight (IN-107) are two halves of the Fonts shelf workflow; merging Fonts contextually into the Text card changes no semantics.
7. **Rotation**: well (IN-048) + dial (IN-050) on the same row; Camera Roll likewise; generic angle params also pair dial+well — keep as one control with two affordances.
8. **Layer pickers**: Parent (IN-067, `setAttrs`), Camera Target (IN-059, `setProperty`), layer params (IN-094, `setProperty`), Matte source (IN-127, `setMatte`) — same UI, different ops; merging the widget is fine, merging the op is not.
9. **Space** writes to all selected layers, but **Parent / Environment / Clip** write only the shown layer and **Ghost** writes all selected ghostable — inconsistent multi-select semantics; unifying would change behaviour (flag for user).
10. **Matte card** (IN-127/128) overlaps Clip-to-below (IN-071) and Stencil/Silhouette blend modes; it is inert (no `setMatte` capability) — candidate to MERGE into a read-only legacy note.
11. **Stage camera gizmo / Depth desk** also write `camera.center/zoom/roll/orbit/distance`, `position(.z)` — Inspector wells are the precision half of those; keep both.

## Other observations (for the owner)

- Expression entry does not exist: typed text is `double.tryParse` only (no math, no units like "10px") — `F/panel_controls/numeric.dart:212`.
- `_restOf('scale')` returns 1.0 while the well displays ×100, so Scale never reads "at rest" quiet ink — `I/wells.dart:162` vs `:91`.
- Z axes and Rotation X/Y are always emitted and shown even in 2D projection.
- Camera Distance is labelled "Scale" in the Camera card (`I/transform_card.dart:138`).
