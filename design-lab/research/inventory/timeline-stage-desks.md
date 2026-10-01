> **Use of this file (owner, 2026-10-02): COUNTER-EXAMPLE ONLY.** The old Motolii is not a reference for the new UI or UX. This file is a list of what exists (a capability checklist) and of what NOT to import. Nothing here is a spec.

# Inventory: Timeline, Stage, Desks (Motolii, branch dev/ui-inspect)

Source root: `/private/tmp/wt/tok/motolii/ui/lib` (paths below are relative to it). All entries come from code read in this pass.
Explorer stories live in `/private/tmp/wt/tok/motolii/ui/explorer/lib/stories/{timeline,panels,workspace}.dart`. Story names are quoted.
Convention: "story: none" = no explorer story shows it. `[uncertain]` = inferred, not fully verified in code.

Scope note: the Timeline and Desk panels here are the "hf / New" live skins. The Stage panel in this directory (`stage/`) is the one `LiveWorkspace` mounts with NO `topBar`/`bottomBar`, so it draws its own bars (Classic-style, `EditorBar`). The `topBar`/`bottomBar` hooks (a host's own bars driven by `StageToolbarApi`) exist, but no caller was found in `lib/` (grep) [uncertain: the "tool column" of the newest UI may live outside my area, e.g. `app/new/**`; nothing named a Stage tool column was found in `stage/**`].

---------------------------------------------------------------------------------------------------------------

## A. TIMELINE  (`timeline/timeline.dart`, `rows.dart`, `semantics.dart`, `session.dart`, `geometry.dart`)

Architecture: `TimelineSession` (`timeline/session.dart:15`) holds rows/view/gesture state; `LiveTimeline` (`timeline/timeline.dart:102`) is one skin that hit-tests and paints it. Panel tab name in dock: "Timeline" (`workspace/seats.dart`).

### A1. Panel frame and strip tools
1. **Timeline panel (LiveTimeline)** · `timeline/timeline.dart:102`
   - One panel: name column (left) + ruler (top) + rows/lanes (body). Name column width = 22% of panel clamped 132..220 (`:285` `labelW`). Frame 0 sits `Surface.panelInset` right of the name column.
   - States: default; hover row (row ground lightens, switches show); selected rows; playing; scrubbing; bar held mid-drag; narrow (name column clamps to 132); empty (no rows, just ruler and grounds); drop-target for assets (DragTarget).
   - Displays: everything below. Story: "Empty", "Normal (4 layers)", "Dense (40 layers)", "Long names (English, Japanese)", "Many keys", "Properties open", "Short bars", "Narrow panel", "Playing", "Bar held mid-drag".
2. **Split tool** (seat-strip tool) · `timeline/timeline.dart:37` (`_Tool`, label 'Split', glyph split)
   - Quiet text+pictogram button at the right end of the Timeline's seat strip; runs `split` command. States: live (ink g63), hover (g95), disabled (g33, when `split` unsupported). Story: none (the strip tools are only inside the dock; "Workspace 90/100/115%" shows the whole workspace [uncertain whether tool is visible at that size]).
3. **Marker tool** (seat-strip tool) · `timeline/timeline.dart:37` (`_Tool`, 'Marker', glyph marker); runs `addMarker`. Same states as Split. Container: `LiveTimelineTools` `:20`. Story: none (workspace stories only).

### A2. Layer rows and the name (label) cell
4. **Layer row ground (banded row)** · painted in `_RowsPainter` `:589`
   - Alternating bands `#1E1E1E` / `#2A2A2A`, dark 1px line under each row; row height 23 px*scale (`Surface.px(23)`). States: normal, hover (g15), selected (g20). Region left of name column also painted by the same row ground. Stories: all Timeline stories.
5. **Layer name cell** · `_outline()` `timeline/timeline.dart:415`
   - Per row: 2px left edge in the layer's colour when selected (grey for groups), indent (9px * depth), twirl, kind chip, name, switches. Press picks the layer (and starts a row drag to reorder); right-click opens the row menu. Name styles: normal g86, selected g100 + w600, hidden g56, group w600, lane name g69.
   - States: default, hover, selected, hidden (dimmed name), group (bold), lane (property row), ellipsized long name (one line, `Overflow.ellipsis`). Story: "Long names (English, Japanese)", "Dense (40 layers)".
6. **Twirl (disclosure triangle)** · `timeline/timeline.dart:501` (`_Twirl`, `_TriPainter`)
   - 7px triangle: right = closed, down = open. On a normal layer opens/closes its property lanes (`toggleLanes`); on a group row the left twirl folds/unfolds children (`toggleFold`) and a second right-hand twirl opens the group's own lanes. `strong` variant (groups) is lighter (g69 vs g51). Story: "Properties open".
7. **Kind chip** · `timeline/timeline.dart:523` (`_Chip`)
   - Layer: 8x8 rounded colour square in the layer's family colour (grey g33 if hidden). Camera: camera glyph. Group: list glyph. Story: any Timeline story.
8. **Layer switch column: Clip, Eye, Solo, Lock** · `timeline/timeline.dart:~455-460` (via `mark`/`glyph` closures in `_outline`)
   - Four 14px slots at the row's right: clip-to-below (crop glyph; only visible when on or when row is hovered/selected), eye / eye-off (hidden), solo, lock. Lit (filled accent `H.toggleOn`) only when on; faint otherwise; brighter when row hovered or selected (`quiet`). Calls `toggleSwitch` (flags hidden/solo/locked via `toggle`, clipToBelow via `clip`). Story: visible in "Normal (4 layers)" etc.
9. **Property lane name cell** · `_outline()` lane branch
   - For an opened lane: left indent, a thin coloured line in the layer's colour (the lane belongs to that layer), the property label (or `Effect · Param` for effect params, `Content` for text content keys), and a **key-at-playhead diamond** button.
   - **Key-at-playhead diamond** · `timeline/timeline.dart:536` (`_Diamond`): hollow diamond (edge g56); filled near-white when a key exists on the current frame; click toggles a key at the playhead (`toggleKey`). Story: "Properties open".
10. **Row context menu** · `session.dart:445` (`menu`) shown with `showHfMenu`
    - Lines: Freeze/Unfreeze, Show animated properties, Show all properties, Hide properties, then Copy (⌘C), Cut (⌘X), Paste (⌘V), Duplicate (⌘D), Delete (⌫), Group (⌘G), Ungroup (⇧⌘G), Split (⌘K). Items disabled when the op is unsupported; dividers via `groupEnd`. Opens from name cell or lanes (null row = off the rows). Story: none.
11. **Group nesting / fold state** · `timeline/rows.dart:97` (`LaneLayout`), `:56` (`LaneContainer`)
    - Groups are recursive containers; children appear under the group when `groupOpen`; indentation = `depth*9`. Folded group's children omitted. Story: none (no group in the fixtures I could confirm) [uncertain].

### A3. Bars, keys, interpolation marks, lanes
12. **Layer bar** · `_RowsPainter` `:589` (bar block)
    - Rounded bar (0.87 of row, radius ~2px), colour from layer family (`_family(id)`, derived from id), calmer when not selected, group = grey, hidden = g20; faint light line on top and bottom edge; selected gets a 55%-white 1px outline. Min drawn width 3px. Hit parts: body (move), start edge (trim in), end edge (trim out); Alt+press = slip (`TlGesture.slip`). Locked layers / unsupported ops disable the gesture. Cursor: resizeLeftRight on edges, click on body. Story: "Short bars", "Normal", "Bar held mid-drag".
13. **Bar drag preview (held bar)** · `session.dart` `press`/`drag` (`TlGesture.move/trimIn/trimOut/slip`), `previewTimings`
    - While held, the host previews timings into the layer itself (the Stage shows the same); skin just redraws. Story: "Bar held mid-drag".
14. **Ghost extension (loop/hold tail)** · `_RowsPainter` ghost block
    - Layer `ghost` frames (positive: after the end, negative: before the start) drawn as a 28%-alpha copy of the bar. Timeline extent includes ghosts. Story: none [uncertain whether any fixture has ghost].
15. **Waveform on bar** · `_RowsPainter` wave block
    - If `state['waveforms']` has columns for the layer, draws min/max vertical slivers (colour `H.wave` mixed 38% to white) over the bar. Story: none.
16. **Folded-layer summary keys** · `_RowsPainter` (`!r.lanesOpen` branch), `rows.dart` `summaryFrames`
    - When lanes are closed, every key of the layer (properties, content, effect params) is drawn as a diamond on the bar, with a faint 1px line from first to last key. Each key frame is hit-testable (`TlLayerKey`). Story: "Many keys".
17. **Key diamond (key mark)** · `timeline/timeline.dart:556` (`_keyMark`)
    - Small near-white filled diamond with fine edge; picked = accent fill + white edge; key under the playhead = playhead blue, 1.2x larger, thicker white edge. Same size for layer and lane keys. Used on lanes and folded bars. Story: "Many keys", "Properties open".
18. **Key carried preview** · `_RowsPainter` `keyX`
    - Picked keys draw shifted by `deltaFrames` during a key drag (and `settlingDelta` right after release) so what is drawn is what lands. Stories: none.
19. **Lane span line with interpolation mark** · `_RowsPainter` lane branch
    - Between consecutive keys of a lane a horizontal 1px line (g51; accent 1.6px when both ends picked). **Linear** interp = dashed (2.5px dash / 5px period); every other kind = solid. No other glyph per kind (no Hold step mark, no Bezier curve glyph in the timeline). Span is clickable (`TlSpan`, picks both end keys). Story: "Properties open".
20. **Property lane (row)** · `rows.dart:9` (`TrackRow` with `property != null`), `LaneLayout`
    - One row per property that has keys (all properties when "Show all properties"), plus a `Content` row for text `contentKeys`, plus rows per effect parameter. Lanes carry keys only; no value graph/curve. Story: "Properties open".
21. **Key marquee / selection** · see A6.

### A4. Ruler, playhead, markers, work area
22. **Ruler** · `_RowsPainter` ruler block, height `Surface.ruler`
    - Dark strip (g07). Major ticks every step whose label spacing >= 70px (steps from 1/2/5/10 frames, half-second, seconds, 2/5/10/30/60/300 s); minor ticks in between when >= 8px; labels `MM:SS` (or `SS:FF` when the major step is under a second) in micro text. Press or drag on the ruler seeks (immediately on pointer-down). Wheel over ruler zooms about the pointer; trackpad gesture beginning on the ruler = scrub-zoom. Story: all Timeline stories.
23. **Time grid** · `timeline/timeline.dart:572` (`_grid`)
    - 1px light line at each major tick, fainter lines about every 30px between them; also continues below the last row. Story: all.
24. **Composition end mark** · `_RowsPainter`
    - A 1px line at `durationFrames` and a darker (22% black) shade over rows/space past the end. Story: all.
25. **Playhead** · `timeline/timeline.dart:779` (`_HeadPainter`)
    - 1px vertical line in `H.playhead` plus a pentagon flag in the ruler's lower band; own `RepaintBoundary`, repaints per frame; follows `scrub` value while scrubbing. States: playing (moves per frame), scrubbing (optimistic), hidden when left of the name column. Story: "Playing".
26. **Marker (flag + line)** · `_RowsPainter` markers, `_marker()` `timeline/timeline.dart:393`
    - Red (`H.record`) flag in the ruler plus a 22%-alpha vertical line through the lanes. Draggable horizontally in the ruler (`setMarker`), right-click shows a one-item menu "Delete marker" (`deleteMarker`). 12px target. States: default, dragging (`markerDrag`), no-edit when `setMarker` unsupported. Story: none. Markers have no name label drawn.
27. **Work area / loop range**: NOT FOUND in `timeline/**` (no code). Only `durationFrames` as an end mark (item 24). [uncertain only in the sense that it could exist elsewhere; grep of `timeline/` for work/loop found nothing]. The classic inventory doc `docs/stage5/ui-rebaseline/inventory/timeline.md` also records "no Loop or work-area control anywhere in the current UI" (TL-004).

### A5. Scrub, zoom, scroll (no visible widget besides cursor and view)
28. **View navigation** · `timeline/timeline.dart` `_wheel`, `onScale*`, `input/viewport_motion.dart:10` (`ViewportMotion`)
    - Wheel: Cmd/Ctrl or over ruler = zoom about pointer (`exp(-dy*.002)`); Shift = horizontal pan; plain = pan both axes. Trackpad: two-finger pan with inertia, pinch zoom, scrub-zoom starting over ruler (`ViewportGestureMode.scrubZoom`). No scrollbar widget in this skin, no overview strip, no zoom buttons or % field in this skin (the Classic `EditorPercentField`, overview and scrollbars are not in `timeline/timeline.dart`) [verified by reading the whole file]. Story: none (interaction only).
29. **Keyboard on Timeline** · `timeline/timeline.dart` `_key`, `session.dart` `key`
    - Left/Right: move picked keys by 1 (Shift: 10) else step playhead; Esc cancels gesture and preview. Story: none.

### A6. Marquee, drop guide, asset drop
30. **Marquee** · `_RowsPainter` (`s.marquee`)
    - Accent 12% fill + accent 1px outline, in frames x rows; selects keys inside (lane keys; folded layers' summary keys) and layers whose bar it touches; additive with Cmd/Shift. Started by a press on empty space (`TlEmpty`). Story: none.
31. **Row-drag drop guide** · `_RowsPainter` (`s.drop`)
    - Reordering by dragging a name: a 1.5px `H.guide` line at the insertion position, indented by `depth*9+45`; "inside" (onto a group) draws an outline box around the row. Story: none.
32. **Asset drop target (from Browser)** · `LiveTimeline.build` DragTarget, `session.dart` `aimAsset/leaveAsset/acceptAsset`
    - While an asset from the Browser is dragged over, the same drop guide shows where the layer would land (row, frame); accepts `placeAsset` or `placeCatalogAsset`. Story: none.

### A7. Seat panels in the Timeline seat
33. **Graph panel (placeholder)** · `workspace/seats.dart` (`'Graph'`: `ColoredBox(Surface.base)`)
    - Empty base-colour box; tab exists in the Timeline seat. Comment in code: "what they show is not built yet". Story: none.
34. **Console panel** · `app/console.dart:10` (`LiveConsole`)
    - Header row "N messages" + "Clear" (disabled/dim when empty); body: newest-first list, each line = HH:MM:SS (mono), a 4.5px square (red `H.scatter.n` for error, `H.follow.b` for notice), text. Empty: "No messages". Log is `ConsoleLog` (`session/console_log.dart`). Story: none.

---------------------------------------------------------------------------------------------------------------

## B. STAGE  (`stage/stage.dart`, `chrome.dart`, `overlay.dart`, `touch.dart`, `camera.dart`, `spatial.dart`, `session.dart`, `view.dart`, `window.dart`)

Two dock tabs share one class: **Stage** (`view: 'User'`, observer view, workbench) and **Camera** (`view: 'Camera'`, the output picture). `workspace/seats.dart` mounts both with default Classic bars.

35. **Stage panel (tab "Stage")** · `stage/stage.dart:53` (`StagePanel`, state `:77`)
    - Native texture of the composition inside a drawn frame, with everything below layered on top; background `EditorTheme.app`; area outside the frame is dimmed (55% app colour) in the Stage tab (`dimOutside`: "frame is a reference, not a crop"). States: front view vs orbited observer (`_front`), gizmos hidden while playing, "Transform gestures unavailable" notice when `stageGesture` unsupported, Esc cancels a gesture. Story: "Stage chrome" (779x520, a 2D shape selected).
36. **Camera panel (tab "Camera")** · same class, `view: 'Camera'`
    - Shows the output picture itself at comp size (not covering); cages and handles appear only while the pointer is inside (`_setInside`, comment cites 2026-09-19 user decision). Story: none.
37. **Top bar (Classic-style)** · `stage/chrome.dart` build (`EditorBar`)
    - Left: "Front" button (only Stage tab; disabled when already front; tooltip "Look straight at the composition (double-click background)"). Right: "Fit", "100%", "−", a percent field "Stage zoom" (2..1600%), "+". Story: "Stage chrome".
38. **Bottom bar (Classic-style)** · `stage/chrome.dart` build
    - Left: `W × H` text; a **transparent-ground switch** (grid glyph, tooltips "The frame has no ground; the export carries alpha" / "Drop the ground so the export carries alpha"); on Stage tab an **Extend** toggle button ("Extend" / "● Extend", "Allow the working area edges to be dragged"). Right: "Frame N" readout, and "Transform gestures unavailable" if unsupported. Story: "Stage chrome".
39. **Alpha checkerboard** · `controls/panel/scale.dart:59` (`CheckerPainter`), used in `chrome.dart` `_picture`
    - Drawn under the texture across the frame when the background alpha < 1. Story: none (needs a transparent scene).
40. **Frame outline and outside dim** · `stage/overlay.dart:30` (`_StageOverlay.paint`)
    - 1px frame polygon (`colors.line`) — a rectangle when front, a projected quad when orbited; outside area dimmed on the Stage tab. Story: "Stage chrome".
41. **Selection cage and handles** · `stage/touch.dart:48` (`_handles`), painted in `overlay.dart`
    - For a selected, hovered/touched 2D or 2.5D layer: outline polygon (accent 1px) + 4 corner handles (nw ne se sw), 4 edge-middle handles (n e s w) as 6px squares (app fill, accent edge), and a **rotation** knob (4px circle) 22px above the top edge. Shown only on the layer under the pointer, or on all selected layers while dragging. P / R / S keys held narrow the gizmo to position/rotation/scale (`touch.dart:3`). Story: "Stage chrome".
42. **3D layer gizmo mesh** · `stage/spatial.dart`, `geometry.dart:32` (`SpatialMesh`)
    - For 3D layers, native hands over the exact triangles (axes) it hit-tests; drawn via `drawVertices`; hovered part lights. Story: none [uncertain: fixture may have no 3D layer].
43. **Anchor-pad preview cross** · `overlay.dart` (`anchorPreview`)
    - Cross + small circle at where a hovered Inspector anchor-pad cell would put the pivot. Story: none.
44. **Snap guides** · `chrome.dart` `_snapGuides`, overlay lines
    - Accent lines at composition x / y while a Cmd-held move is snapping. Story: none.
45. **Selection marquee** · `overlay.dart` (`marquee`)
    - Accent 12% fill + 1px outline on the Stage when dragging on empty space. Story: none.
46. **Camera gizmos (Stage tab)** · `overlay.dart`, `camera.dart`
    - Blender-style camera object: frame box on the comp plane, eye point, frustum edges, up triangle, target cross. Colour `ink.camera`. Hidden while playing. Story: none.
47. **Camera box handles (front view)** · `camera.dart:35` (`_cameraHandles`), `touch.dart:93`
    - Boxcam: selected camera shows 4 corner handles (zoom0..3, 6px squares) + roll circle; dragging an edge moves Center, a corner Zoom, the circle Roll (`StCameraHandle`). Story: none.
48. **Working area (extent) frame** · `overlay.dart` (`extent`, `extendable`), `camera.dart:6`
    - Polygon around the working comp; thin muted line normally, 2px ink when "Extend" is on and edges (top/right/bottom/left) can be dragged (`StExtentEdge`). Toggling Extend creates a Stage layer if none. Story: none.
49. **Observer target mark** · `overlay.dart` (`observerTarget`)
    - Circle + cross at the orbit target when the Stage is orbited (not front). Story: none.
50. **Stage gestures with no visible widget** · `stage/touch.dart` `_target`, `session.dart` (`StGesture`)
    - Primary press on layer = pick/move; on handle = scale/rotate; empty = marquee; right-button drag on Stage tab = orbit (`StOrbit`); middle-button or Space-drag = pan; double-click on a layer = look at it, on ground = back to front (`StFocus`); eyedropper armed (`c.eyedropper`) = pick colour (`StPick`, `pickColor`). Wheel zooms about the pointer (`exp(-dy*.0015)`); trackpad pan/pinch. Story: none.
51. **Corner labels / zoom HUD / rulers / grid / safe area**: NOT FOUND in `stage/**`. No ruler, no grid overlay, no safe-area guides, no on-canvas zoom HUD, no corner label. Only the bottom-bar `W × H`, `Frame N` text and the zoom percent field. [uncertain: could exist in a host bar outside my area]
52. **Stage tool column**: NOT FOUND in `stage/**` (no vertical tool strip; tools are P/R/S keys and gesture targets).

---------------------------------------------------------------------------------------------------------------

## C. DESKS  (`desks/**`)

Dock seat "Desk" (`workspace/seats.dart`): tabs Ease, Depth, Blend, History, Notes (weight .19, square-based). Relations and Web open on demand. Common frame below.

### C0. Shared desk parts (`desks/parts.dart`)
53. **Desk shell (header + 3 morphologies)** · `desks/parts.dart:62` (`DeskShell`)
    - Header row: when docked (in a dock tab) shows only the subtitle line (tab already names the desk); undocked shows desk icon + title + subtitle. Body switches by size: **strip** (height < 200), **tall** (width < 230), **full**. Optional trailing widget (hidden unless header wide, unless `alwaysShowTrailing`). Story: "Depth desk", "Ease desk", "Notes desk" (420x420 full).
54. **Desk icon** · `desks/parts.dart:19` (`DeskIcon`): per-kind line pictogram (ease curve, depth concentric circles, blend two circles, history clock, notes page). Shown only when undocked. Story: none.
55. **NumBox** · `desks/parts.dart:101`: small label + large value box (Depth readouts). **Segmented** · `desks/parts.dart:120`: equal-width pill row, active = yellow-dim fill + yellow edge (used by the unhosted Depth view switch). Story: "Depth desk" shows NumBox.
56. **Desk palette**: flat colours `kYellow` (current/touched), `kBlue`, `kPink`, `kMint`, `kViolet` (`parts.dart:9-15`).

### C1. Ease desk  (`desks/ease/face.dart:152` `EaseDesk`; live host `desks/ease/desk.dart:290` `LiveEase`)
57. **Ease desk panel** · header title "Ease", subtitle = target text (e.g. "Layer · property · 12–40 · 2 intervals · Mixed · Read only", "Sequence · N layers · ghosts", "No interval · Workspace"; unhosted fallback "KEYFRAMES · MOTION"); trailing **"Over OK" switch** (check-box glyph; allows values outside 0–1 for Y). States: full, short strip (scrolls), narrow (blocks stack), read-only/locked, mixed intervals (sel = -1), sequence (several layers, no keys), workspace (no interval). Story: "Ease desk".
58. **Curve plot** · `desks/ease/face.dart:669` (`_PlotP`), box via `easePlotBox`
    - Dark rounded plot with two rails (0 and 1), gradient fill (pink→violet→blue) under the curve, white curve line, endpoint dots, **yellow Bezier handles** with halo (draggable, Esc puts curve back), non-Bezier kinds show their own handle points; **playhead** (white line + flag) that follows the document or a local 0→1 preview; mint current-value dot and a **motion strip** at the right edge (nine evenly timed dots + mint marker); mixed mode draws several coloured curves; sequence mode draws a dot per layer; ghost mode draws 3 offset ghost curves; peek (preset hover/keys) shows the preset's shape with name "· peek". Labels (when large enough): kind name top-left (or "Mixed"), caption/frames top-right ("12 f", "3 layers", "Workspace"), the kind's one-line meaning (`meaning.dart`), `f<start>` and `f<end>` at the bottom. Story: "Ease desk".
59. **Preset row** · `face.dart:400`
    - Play-preview button (`ease-play`, blue when running, runs a 1.4s 0→1 sweep) + horizontal scroll of preset tiles (`ease-preset-i`): small curve icon in a colour from the 5-colour cycle; chosen = tinted fill + coloured edge; peek = lighter. Arrow/Home/End keys peek, Enter/click applies. Kinds from host `easeKinds`: Linear, Bezier, Hold, Bounce, Elastic, Cyclic, Random, Steps, ElasticSteps (see `ease/meaning.dart`); unhosted: Linear, Ease, Bezier, Spring, Bounce. Story: "Ease desk".
60. **VALUES section** · `face.dart:327`: blocks of two number cells (X1, Y1, X2, Y2 for Bezier; the kind's own parameters otherwise; "This curve has no numbers." for Hold/Linear). Cell = label + big value, click to type (`EditableText`), yellow block (mixed = grey "—"). 2-column grid at width >= 230. Story: "Ease desk".
61. **SAVED section** · `face.dart:477/492`: saved-curve thumbnails (mint curve in a 30px tile), chips "Copy curve", "Save preset", "Use for new keys", "Clear" (only when something to clear). Unhosted: demo saved curves + "Copy curve" + "Clear". Story: "Ease desk" (scrolled area [uncertain visible at 420x420]).
62. **OPTIONS section** · `face.dart:518`: "Sequence ghosts" switch (read-only reflecting multi-layer selection when hosted; toy toggle when unhosted). Story: "Ease desk" (if scrolled).
63. **Section labels** "VALUES", "SAVED", "OPTIONS": micro caps, letter-spaced.

### C2. Depth desk  (`desks/depth/face.dart:117` `DepthDesk`; wrapper `desks/depth/desk.dart:9` `NewDepth`; host `depth/host.dart:11`)
64. **Depth desk panel** · title "Depth", subtitle "STAGE VIEW"; unhosted shows a **Top / Front / Side segmented** view switch in the header (hosted: top view only). States: full, strip (wide "topWide" depth ruler), tall (top view + Dist/FOV boxes), hosted vs fixture. Story: "Depth desk".
65. **Depth diagram (floor plan)** · `face.dart:273` (`DepthPainter`), `_diagram` `:205`
    - 2D projection (not perspective): ground cross lines, camera (white dot with cone, edges, dashed centre), layers as thick lines/rects numbered 1..n (selected = yellow, others blue, hot = outlined), a `TARGET` cross at origin, corner view label "TOP / FRONT / SIDE". Dragging a layer or the camera streams `stageGesture` depth mode (host), Esc/cancel puts it back; press selects. Locked layers can't move. Story: "Depth desk".
66. **Readout row** (full): NumBox "Camera" (distance), "FOV" (degrees), selected layer name + distance; tall: "Dist", "FOV". Story: "Depth desk".
67. **Colour key row**: dots "Camera" (white), "Selected" (yellow), "Other layers" (blue). Story: "Depth desk".
68. **Hint text**: "Looking at <target>. Drag layers or the camera to move them. Camera settings live in Inspector." Story: "Depth desk".

### C3. Blend desk  (`desks/blend/face.dart:85` `BlendDesk`; wrapper `blend/desk.dart:9` `NewBlend`; host `blend/host.dart:14`)
69. **Blend desk panel** · title "Blend", subtitle "LAYER · COMPOSITE". 19 modes (Normal, Darken, Multiply, Color Burn, Lighten, Screen, Color Dodge, Add, Overlay, Soft Light, Hard Light, Difference, Exclusion, Hue, Saturation, Color, Luminosity, Stencil, Silhouette) each drawn as the same two overlapping discs (A yellow base, B blue top) through that operator. States: full, strip (horizontal marks), tall (wrapped marks), hosted (dims to 45% with no target layer), mixed targets ("Targets differ"), none ("Nothing"/"No layer"), hover (aims on Stage), keyboard (arrows walk, Enter applies, Esc leaves). Story: none.
70. **Equation / result face** · `face.dart:211`: A and B small discs with letters, an arrow "→", a large result specimen (`ResultPainter`), optional strip of the runtime's specimen "beds" colours, and the mode name. Story: none.
71. **Targets row** · `face.dart:240`: "TARGETS" label + yellow pill chips of target layer names (first 2, then "+N"); unhosted chips "Layer 2/3" are toggles. Story: none.
72. **Mode tile grid** · `face.dart:280` (`_tile`, `_mark` `:258`): 4-column wrap of 40px marks with name beneath; selected = hover fill + yellow outline + bold name. Hover previews on Stage when "Preview on Stage" is on; click applies one step. Story: none.
73. **"Preview on Stage" checkbox** · `face.dart` (`blend-stage`): blue check box + label. Story: none.

### C4. History desk  (`desks/history/face.dart:45` `HistoryDesk`; wrapper `history/desk.dart:9` `NewHistory`)
74. **History desk panel** · title "History", subtitle "UNDO · REDO". States: full (list + buttons), strip (horizontal rail, click to go), tall (dots only), empty ("No history" via `emptyBody`), read-only when `historyGoto` unsupported. Story: none.
75. **History rail list (vertical)** · `face.dart:152` (`_row`), painter `_Node` `:215`
    - 38px rows: a rail with node per entry — reached steps = solid blue line + blue dot; current = large yellow ring with halo and a faint yellow row tint; redo tail = dashed dim line + hollow dot; marks: save (mint square), open (violet folder), warning (amber triangle), error (red X circle), end (blue square outline). Label (bold at current, quiet otherwise) + HH:MM:SS mono time. Click a row = `historyGoto`. Auto-scrolls the current step to the middle. Story: none. It is a linear list/rail, not a branching graph.
76. **Horizontal history rail (strip)** · `face.dart:232` (`_HRail`): same nodes along a horizontal line. Story: none.
77. **Undo / Redo buttons** · `face.dart:141` (`_btn`): two wells with label and shortcut ("Undo ⌘Z", "Redo ⇧⌘Z"); dimmed when nothing to undo/redo. Story: none.

### C5. Notes desk  (`desks/notes/face.dart:53` `NotesDesk`; live `notes/desk.dart:170` `LiveNotes`)
78. **Notes desk panel** · title "Notes", subtitle "PAGES · REFERENCES". Free 2D canvas of blocks with pages. States: full (toolbar + canvas + zoom bar), strip / tall (same canvas fitted small), editing a note, selected block, empty notebook (a page chip is still shown). Story: "Notes desk".
79. **Notes toolbar** · `face.dart:150`: 5 tool buttons (`notes-tool-i`: select arrow, new note, new image, new reference, hand-line/curve pictogram; tools 1,2,3 add a block immediately) + page chips 1..N (current = yellow edge; right-click = "New page" / "Delete page"). Story: "Notes desk".
80. **Notes canvas** · `face.dart:239` (`_canvas`), dot grid painter `_DotsP`: pannable/zoomable dotted surface; Cmd+V pastes clipboard (picture, else text) as a block; Delete/Backspace removes selected; drag-drop pictures land on the shown page. Story: "Notes desk".
81. **Note block** (`kind note`): tinted card with editable text (double-click to edit, `EditableText`). **Image block** (`kind image`): picture (png) or placeholder. **Reference block** (`kind ref`): small card with a dot and label like "Layer 2 · 0–120"; double-click follows to the layer and first frame. **Hand block** (`kind hand`): handwriting-font line (Snell Roundhand), unhosted fixture only (live host holds note/image/ref). Selected = yellow outline + bottom-right resize square (`resize`). Drag moves; commits `x,y` on release. Story: "Notes desk" (fixture-like content from `night-sky.rrd`) [uncertain whether blocks present].
82. **Notes zoom bar** · `face.dart:225`: "Page N · M blocks" left; "−", zoom % (mono), "+", "Fit" (yellow) right. Story: "Notes desk".

### C6. Relations desk  (`desks/relations/desk.dart:15` `RelationsPanel`; model `model.dart`; session `session.dart:9`)
Opened on demand (Inspector property link / right-click "Relation…"); closable tab.
83. **Relations panel frame** · header (`DockedPanel`, chrome row): title switches "RELATIONS" / "CHOOSE THE THINGS IT DRIVES" (draft) / "EDIT THE MEMBERS"; right "N things now" (mono). Wide (>=520px): graph left, 250px side card right; narrow: graph over a 165px side. Story: none.
84. **Relation graph (dot map)** · `desk.dart:110` `_graph`, painter `:393` `_GraphPainter`
    - Layers present at the current frame as dots placed where they sit on the Stage (camera/stage layers excluded), name beneath. Source = red dot with halo; members = ink dot with red ring; in picking mode non-members get a faint ring. A red line from source to the members' centroid, thin lines to each member, label "N mappings". **Click / Shift-click** picks members; **lasso** (drag on empty space in picking mode; Shift = add) selects dots inside the polygon; outside picking mode a click selects the layer and focuses its relation. Esc cancels. 32px grid. Story: none.
85. **Draft card ("Set" a new relation)** · `desk.dart:213`: source name (red dot), "SOURCE RANGE" two scrubbable ends (`_End`: e.g. `0px`, drag to change) with a range bar and a red dot for the live source value plus "set current as min/max" arrows (⇤ ⇥); "MEMBERS" count ("Click or lasso things in the graph." / "N things selected"); "DESTINATION" chips Scale / Rotation / Opacity / Position (disabled until members exist); "<DEST> RANGE" ends in the destination unit (%, °, px); "Cancel" and red "Create" (enabled only with members and a destination). Writes one `relate`. Story: none.
86. **Relation card (focused relation)** · `desk.dart:258`: source name + "✕" delete; source range scrubber (preview while scrubbed, one commit on release); "MEMBERS" count + "Edit in graph" / "Done"; "MAPPINGS" rows (property label, "✕" remove = `unrelate`, its range scrubber); "+ Scale/Rotation/…" chips to add a mapping. Story: none.
87. **Relation list (nothing focused)** · `desk.dart:183`: rows "Source → N things · Scale, Rotation" with a red dot; empty text "No relation yet. In the Inspector, right-click a value and choose Relation…". Story: none.
   - Note on the brief's wording "Set → normalize → 1:N": in code the flow is draft (source + input range) → members → destination + output range → Create; normalisation is the `motolii.link.remap` (in range to out range) evaluated by the host (`docs/stage5/relations-v0.md`). Lasso, members, 1:N are as described; there is no widget called "Set" or "normalize" other than the ⇤ ⇥ "set current as min/max" arrows [uncertain mapping of the user's words].

### C7. Web desk  (`desks/web/desk.dart:12` `NewWeb`)
88. **Web desk panel** · closable, opened from the Dock menu: search glyph, label "WEBSITE", a URL text field (`web-url`, default `https://www.pinterest.com/`, saved with `storeDesk('webUrl')`), and an "Open in browser" button (`web-open`; asks the host to open the page, no embedded web view). No DeskShell header. Story: none.

### C8. Console (see A7 item 34) and Graph placeholder (item 33) are Timeline-seat panels, not Desk-seat panels.

---------------------------------------------------------------------------------------------------------------

## D. WORKSPACE / INPUT / SESSION (only what the user sees)

89. **Dock seats and seat strips** · `workspace/dock_workspace.dart:28` (`DockWorkspace`), `workspace/seats.dart:27` (`LiveWorkspace`)
    - Panels grouped in seats; each seat draws the Browser's seat strip (`browser/parts.dart` `Leaf`): front tab shows its name, others fold to a glyph; a tab is draggable to another seat's edge (split) or strip (join; a translucent drop target appears); right-click tab menu: Detach, Close (only on-demand panels: Relations, Web), Reset Layout. Dividers are 3px, highlight on hover (`workspace/dock_theme.dart`). Default arrangement: top row = Browser tabs (Create, Effects, Colors, Fonts, Media) .22 | Stage + Camera .50 | Inspector .28 (row weight .68); bottom row = Timeline + Graph + Console .81 | Desk tabs (Ease, Depth, Blend, History, Notes) .19 (weight .32). Panel min sizes 180..320. Story: "Workspace 90% / 100% / 115%" (full workspace at 1440x900).
90. **Window keys** · `input/window_keys.dart:11` (`LiveKeys`): no widget; key map: Space play, ⌘Z/⇧⌘Z undo/redo, ⌘C/X/V/D copy cut paste duplicate, ⌘G / ⇧⌘G group/ungroup, ⌘A select all, ⌘K split, ⌘N/O/S/I file ops, ⌘±/0/1 and ⌘⌥±/0 zoom/UI scale [uncertain which], Delete/Backspace delete, Home/End, arrows (move keys or step), ↑/↓ select step, M add marker, F9 easy-ease (Shift/Cmd+Shift variants), A/P/R/S etc. reveal properties. Story: none.
91. **Viewport motion** · `input/viewport_motion.dart:10` (`ViewportMotion`): pan inertia and scrub-zoom shared by Timeline (and Stage `view.dart`). No visuals. Story: none.
92. **Session data the user sees**: `ConsoleLog` (Console), `status_notice.dart`, `color_edit.dart`/`swatches.dart`/`color_palette.dart` (not read: belong to Inspector/Browser areas, out of scope) [uncertain].

---------------------------------------------------------------------------------------------------------------

## (a) Count of entries
92 numbered entries (some are sub-items grouped under one number; the distinct on-screen widgets/panels/parts counted separately is ~80). Numbers 27, 51, 52 are "NOT FOUND" records, kept so absences are explicit.

## (b) Things I could not classify
- Where the "newest UI" Stage tool column, zoom HUD, rulers, grid and safe-area live: not in `stage/**`. `StagePanel` has `topBar`/`bottomBar` hooks with a `StageToolbarApi`, but no caller found under `lib/` (grep for `topBar:`). They may be in a part of the worktree outside my area, or not built.
- Timeline has no work area/loop range, scrollbar, overview strip, zoom buttons or time readout in `timeline/timeline.dart` (the Classic panel `panels/timeline*` has them per `docs/stage5/ui-rebaseline/inventory/timeline.md`; that is a different file set).
- "Graph" tab in the Timeline seat is an empty placeholder.
- Notes block fixtures: whether "night-sky.rrd" shows any blocks in the explorer story is unknown.
- Whether any explorer fixture contains a group layer, a ghost, a waveform, markers or a transparent ground (no story targets them explicitly).
- Window-key mappings for zoom/UI scale keys were skimmed, not fully verified.
