# Capability inventory — Stage / viewport

Audited from code (read-only) on branch `claude/ui-rebaseline`, 2026-09-25.
Scope: `motolii/ui/lib/panels/stage.dart`, `panels/stage/*`, `panels/composition_controls.dart`,
`panels/native_visual_sample.dart`, `panels/web_panel.dart`, `input/viewport_motion.dart`, the
Stage-related global shortcuts in `input/editor_shortcuts.dart`, and the native side
(`ui/native/src/port.rs` `stageView`/`stageGesture`/`pickColor`, `editor/stage.rs`, `editor/gizmo3d.rs`,
`editor/create.rs`, `editor/placement_edit.rs`, `viewer.rs`, `snapshot.rs`).

Paths below are relative to `motolii/ui/` (for example `lib/panels/stage/touch.dart:226` means
`motolii/ui/lib/panels/stage/touch.dart`). Native paths are under `native/src/`.

**Two views of one panel.** `StagePanel(view:'User')` is the **Stage** tab (the observer, also called "User
Stage"). `StagePanel(view:'Camera')` is the **Camera** tab (the document camera, which is also the output).
They are registered at `lib/panels/registry.dart:50-51`. Most rows apply to both. Where a row applies to
only one, the Condition column says "Stage only" or "Camera only". Native `View::parse` accepts only
`User`/`Stage`/`Camera` (`native/src/viewer.rs:14-19`). **There are no Front / Top / Left / Right ortho
views.** "Front" is a button that resets the observer (ST-010).

Disposition key: PRESERVE / MOVE / MERGE / CONTEXTUALIZE / VISUALIZE (combos allowed; nothing is DELETE).

---

## Inventory

### A. View tabs, attachment and picture

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-001 | Open the Stage view (observer, "User Stage") as a panel | Show the "Stage" pane (View menu / dock tab) | — | `lib/panels/registry.dart:50` | `StagePanel(view:'User')` | PRESERVE | Primary workbench view |
| ST-002 | Open the Camera view (what the document camera outputs) as a panel | Show the "Camera" pane | — | `lib/panels/registry.dart:51` | `StagePanel(view:'Camera')` | PRESERVE+MERGE | Could become a mode of one viewport. Merging only changes layout if both can still be shown at once |
| ST-003 | Only visible views render: a tab coming into view attaches its texture, and a hidden one detaches. The Stage also withdraws its window (`stageWindow 0×0`) | Switch dock tab / hide pane | Implicit | `lib/panels/stage/window.dart:35-52`, `lib/panels/stage.dart:66-79` | `c.attachView` / `c.detachView`; native `stageWindow` | PRESERVE | Perf contract, not UI. The new shell must keep calling it |
| ST-004 | The Stage asks native to render at the tab's pixel size and zoom/pan ROI. Resizing does not stretch the image | Resize pane / zoom / pan | Stage only; `supports('stageWindow')` | `lib/panels/stage/window.dart:60-139`, `lib/panels/stage/chrome.dart:159-172` | native `stageWindow {width,height,roi}` (FFI same-frame via `commandNow`) | PRESERVE | Required for the "same frame" viewport bridge (memory: viewport FFI) |
| ST-005 | Placeholder text "No rendered texture" when a view has no texture | Automatic | No texture id for the view | `lib/panels/stage/overlay.dart:13-22` | UI-local | PRESERVE | Empty-state feedback |
| ST-006 | Stage dims the world outside the comp frame (frame is a reference, not a crop, "Boxcam"). Fixed 55 % | Automatic | Stage only | `lib/panels/stage/overlay.dart:94-103`, `lib/panels/stage/chrome.dart:383` | UI-local paint | PRESERVE+VISUALIZE | Core Boxcam reading. See ST-073 (the dim setting is not wired) |
| ST-007 | Comp frame outline. When the Stage is orbited it is drawn as the observer-projected quad | Automatic | Always. Projected form when not front | `lib/panels/stage/overlay.dart:87-103`, `lib/panels/stage/chrome.dart:423-428` | status `observer.frame` | PRESERVE | Orientation cue |
| ST-008 | Transparent-ground checkerboard drawn under the frame | Automatic | Background alpha < 1 | `lib/panels/stage/chrome.dart:17-20,355-366` | UI-local (`CheckerPainter`) | PRESERVE | Shows the export will carry alpha |
| ST-009 | Gizmos and overlays hidden during playback (cages, handles, camera boxes, 3D mesh) | Automatic | `c.playing` true | `lib/panels/stage/chrome.dart:324,334-336,385-410` | UI-local | PRESERVE | Clean playback picture. Note: selection and drag input are **not** blocked while playing |

### B. Observer camera (Stage only): Front, orbit, focus, fit

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-010 | **Front**: reset the observer to home (orbit 0, centre 0, distance 1) | Top-bar button "Front" (tooltip "Look straight at the composition (double-click background)") | Stage only. Disabled when already home (`observer.home`) | `lib/panels/stage/chrome.dart:115-121,69` | native `stageView {reset:true}` → `viewer.user_camera = Default` (`native/src/port.rs:141`) | PRESERVE+CONTEXTUALIZE | Useful only after orbit/focus. Could appear only when not home |
| ST-011 | Reset the observer by double-clicking the background | Primary double-click (<`kDoubleTapTimeout`, <6 px) on empty picture | Stage only. No hittable layer under pointer | `lib/panels/stage/touch.dart:40-53,246-256` | native `stageView {reset:true}` | PRESERVE+MERGE(with ST-010) | Same effect as Front. Keep both triggers |
| ST-012 | Focus the observer on a layer (look at its bounds sphere) | Primary double-click on a layer | Stage only. Layer is hittable (visible, not locked, not Group) | `lib/panels/stage/touch.dart:246-256` | native `stageView {focus:id}` → `focus_sphere` + `looking_at` (`native/src/port.rs:136-140`) | PRESERVE | rerun-style 3D navigation |
| ST-013 | Orbit the observer (yaw from horizontal drag, pitch from vertical drag, 0.3°/px, pitch clamped ±85°) | Right-button (secondary) drag on the picture | Stage only. No gesture in progress | `lib/panels/stage/touch.dart:238-245,426-435`, `lib/panels/stage/camera.dart:7-22` | native `stageView {orbit:[pitch,yaw]}` (coalesced) | PRESERVE | Only way to see the 3D world off-axis. Not undoable (view state), and Esc does not revert it |
| ST-014 | Observer target cross drawn when orbited | Automatic | Stage only, not front | `lib/panels/stage/overlay.dart:128-131`, `lib/panels/stage/chrome.dart:407` | status `observer.target` | PRESERVE | Shows the orbit pivot |
| ST-015 | Fit also frames the **working area** (Stage layer extent) in native and resets orbit | Fit button / Cmd+0 | Stage only; `supports('stageView')` | `lib/panels/stage/view.dart:93-101`, `native/src/port.rs:142-146` | native `stageView {fit:true}` → user_camera = extent-fitting camera (orbit 0) | PRESERVE | "Fit the working area" differs from a 2D fit: it also exits orbit |

### C. Zoom and pan (UI-local view transform, both views)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-016 | Fit the frame to the tab (zoom=auto, pan=0) | Top-bar "Fit" | — | `lib/panels/stage/chrome.dart:123`, `lib/panels/stage/view.dart:93-101` | UI-local (+ ST-015 on Stage) | PRESERVE | Basic viewport op |
| ST-017 | Fit via keyboard | Cmd/Ctrl+0 | Not typing. Every **visible** Stage/Camera view responds | `lib/input/editor_shortcuts.dart:97-101`, `lib/panels/stage/view.dart:118-123` | `c.viewCommand='Fit'` → UI-local | PRESERVE | AE shortcut |
| ST-018 | Actual size 100 % (zoom=1, pan=0) | Top-bar "100%" | — | `lib/panels/stage/chrome.dart:124-130` | UI-local | PRESERVE | — |
| ST-019 | Actual size via keyboard | Cmd/Ctrl+1 | Not typing. All visible views | `lib/input/editor_shortcuts.dart:102-106`, `lib/panels/stage/view.dart:124-128` | `viewCommand='Actual'` | PRESERVE | — |
| ST-020 | Zoom out 1 percentage point about the tab centre | Top-bar "−" | — | `lib/panels/stage/chrome.dart:131-137` | UI-local `_zoomAt` | PRESERVE+MERGE(ST-022) | Step differs from the shortcut (×1.2) |
| ST-021 | Zoom in 1 percentage point about the tab centre | Top-bar "+" | — | `lib/panels/stage/chrome.dart:146-152` | UI-local | PRESERVE+MERGE(ST-023) | Same |
| ST-022 | Zoom out ×1/1.2 about centre | Cmd/Ctrl+− | Not typing. All visible views | `lib/input/editor_shortcuts.dart:112-116`, `lib/panels/stage/view.dart:131-132` | `viewCommand='Out'` | PRESERVE | — |
| ST-023 | Zoom in ×1.2 about centre | Cmd/Ctrl+= | Not typing. All visible views | `lib/input/editor_shortcuts.dart:107-111`, `lib/panels/stage/view.dart:129-130` | `viewCommand='In'` | PRESERVE | — |
| ST-024 | Type an exact zoom % (2–1600) | Double-click the "Stage zoom" field → type → Enter | — | `lib/panels/stage/chrome.dart:138-145`, `lib/foundation/panel_controls/scale.dart:50-99`, `lib/foundation/panel_controls/numeric.dart:449` | UI-local | PRESERVE+VISUALIZE | Numeric readout of zoom |
| ST-025 | Scrub the zoom % (horizontal drag. Figma precision ladder: move up/down while dragging for ×10 / ×0.1 / ×0.01) | Drag on the zoom field | — | `lib/foundation/panel_controls/numeric.dart:303-305,452-454` | UI-local | PRESERVE | Inherited from `EditorNumericField` |
| ST-026 | Step zoom % with the wheel while holding the button (Shift ×10), or with a horizontal trackpad scroll | Wheel / horizontal scroll on the zoom field | — | `lib/foundation/panel_controls/numeric.dart:246-280` | UI-local | PRESERVE | Inherited |
| ST-027 | Arrow Up/Down steps zoom % while typing (Shift ×10). Esc restores the pre-edit zoom | Keys in the zoom field | Field is being edited | `lib/foundation/panel_controls/numeric.dart:365-390`, `lib/foundation/panel_controls/scale.dart:92-96` | UI-local | PRESERVE | Inherited |
| ST-028 | Zoom about the cursor with the mouse wheel (exp(−dy·0.0015), clamp 2 %–1600 %) | Scroll wheel over the picture | `PointerScrollEvent` only | `lib/panels/stage/chrome.dart:197-211`, `lib/panels/stage/view.dart:103-116` | UI-local | PRESERVE | Standard. **The macOS trackpad two-finger pan/pinch (`PointerPanZoom*`) has no handler on the Stage.** Verify in a real window. The new UI should add it |
| ST-029 | Pan with the middle mouse button | Middle-button drag | — | `lib/panels/stage/touch.dart:288-293,436-439` | UI-local `_pan` | PRESERVE | — |
| ST-030 | Pan with Space held + drag | Hold Space, primary drag | Space pressed at pointer-down | `lib/panels/stage/touch.dart:288-293,436-439` | UI-local | PRESERVE | AE / Photoshop hand tool. **Conflict:** the root shortcut toggles playback on Space key-down (`lib/input/editor_shortcuts.dart:43-46`), so Space-panning also starts or stops playback |
| ST-031 | Readout: current zoom % | Always (zoom field) | — | `lib/panels/stage/chrome.dart:138-145` | UI-local | VISUALIZE | — |

### D. Selection on the picture

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-032 | Click a layer to select it alone. If it is already in the selection, the multi-selection is kept so it can be dragged as a group | Primary click on a layer | Layer is visible, not locked, not a Group. Hit = convex hull of corners. Topmost first in list order | `lib/panels/stage/touch.dart:6-7,75-83,311-329` | native `select {ids}` (also clears selected keyframes, `native/src/port.rs:160-166`) | PRESERVE | Core |
| ST-033 | Toggle a layer in or out of the selection | Shift-, Cmd- or Ctrl-click on a layer | Same hit rules | `lib/panels/stage/view.dart:88-91`, `lib/panels/stage/touch.dart:315-320` | native `select` | PRESERVE | Standard additive modifier |
| ST-034 | Groups cannot be picked on the Stage. They are reached only from Timeline or Inspector (user ruling 2026-09-19) | Click on a Group's area | `kind=='Group'` | `lib/panels/stage/touch.dart:3-7` | — | PRESERVE | A deliberate ruling, not a gap |
| ST-035 | Click empty space to deselect all | Primary click on empty space, no modifier | — | `lib/panels/stage/touch.dart:330-333,467-478`, `lib/panels/stage/geometry.dart:128` | native `select {ids:[]}` | PRESERVE | — |
| ST-036 | Marquee (rubber-band) select: replaces the selection with every intersecting layer | Primary drag from empty space | `supports('select')`. Excludes hidden and locked layers. **Includes Group layers** (unlike click) | `lib/panels/stage/touch.dart:330-333,441-444,467-477` | native `select` | PRESERVE | Standard. See the duplicates note on the Group inconsistency |
| ST-037 | Additive marquee: adds intersecting layers to the current selection | Shift/Cmd/Ctrl + drag from empty space | Same | `lib/panels/stage/touch.dart:331,474-476` | native `select` | PRESERVE | Additive marquee never removes a layer (no XOR) |
| ST-038 | Marquee rectangle drawn (accent fill 12 %, outline) | During ST-036/037 | — | `lib/panels/stage/overlay.dart:204-210` | UI-local | PRESERVE | Feedback |
| ST-039 | Hover shows the cage (outline) only on the hovered **selected** layer. Handles appear only when a selected layer is under the pointer (user ruling 2026-09-19: "gizmos answer the intent to touch") | Pointer hover | Not playing. The hover cage is cleared when the pointer leaves the tab | `lib/panels/stage/touch.dart:344-356`, `lib/panels/stage/chrome.dart:325-339,409`, `lib/panels/stage/touch.dart:32-38` | UI-local | PRESERVE+CONTEXTUALIZE | Already contextual. The new UI must keep "selection alone draws nothing" |
| ST-040 | While dragging, every selected layer shows its cage | During a gesture | — | `lib/panels/stage/chrome.dart:327-331` | UI-local | PRESERVE | Multi-object feedback |
| ST-041 | Clicking the Stage gives it keyboard focus (so Esc and held keys reach it) | Any pointer-down | — | `lib/panels/stage/touch.dart:228` | UI-local focus | PRESERVE | Keyboard routing |
| ST-042 | Select previous or next layer (list order) | ↑ / ↓ (no Alt) | Not typing. Global shortcut | `lib/input/editor_shortcuts.dart:139-151` | native `select` | MOVE? | Global, not Stage-bound. Listed because it drives Stage selection |
| ST-043 | Select all layers | Cmd/Ctrl+A | Not typing. Global | `lib/input/editor_shortcuts.dart:74-77` | native `select` | PRESERVE | Global |
| ST-044 | Deselect all and cancel any preview (first closes an open sheet or drawer, if any) | Esc | Not typing. Global, and no Stage drag in progress | `lib/input/editor_shortcuts.dart:34-42` | `c.cancelPreview` + native `select {ids:[],keys:[]}` | PRESERVE | — |

### E. 2D / 2.5D transform cage (planar handles)

All cage gestures stream `stageGesture` begin → update… → commit. The whole drag is one preview and one undo. Native side: `native/src/port.rs:299-322`, `native/src/editor/stage.rs:568-712`. The view that was grabbed matters: in the Camera tab the cage uses the document camera's projection (`bounds`). In the Stage tab it uses the observer's (`stageBounds`).

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-045 | Move the selection by dragging a layer's body (starts after 3 px mouse slop, or touch slop) | Primary drag on a layer | `supports('stageGesture')`. Layer selected by ST-032 first | `lib/panels/stage/touch.dart:311-329,445-452,130-144` | native `stageGesture mode:'move' handle:'body'` → POSITION of every selected layer (`native/src/editor/stage.rs:689-706`) | PRESERVE | Core |
| ST-046 | Move keeps the grabbed point under the cursor on tilted, parented or 2.5D planes (homography mapping) | Implicit in ST-045 | — | `native/src/editor/stage.rs:541-555,650-653` | native | PRESERVE | Must not regress |
| ST-047 | Constrain the move to the dominant axis | Hold Shift during a move | — | `native/src/editor/stage.rs:693-695` | native `shift:true` | PRESERVE | AE convention |
| ST-048 | **Snap** the move to other layers' edges and centres and to the comp frame (left/centre/right, top/middle/bottom; threshold 6 screen px) | Hold **Cmd** during a move | Mode is move. The moved layer has rotation_x = rotation_y = 0. Targets exclude Camera, Stage and Null layers, invisible (opacity 0) layers, rotated layers, and ancestors or descendants of the moved layer | `lib/panels/stage/touch.dart:157-158`, `native/src/editor/stage.rs:596-624,655-681,683-688`, `native/src/port.rs:316` | native `stageGesture snap:true` → `viewer.stage_snap` | PRESERVE+VISUALIZE | The only snapping in the app. Hold-only, with no toggle |
| ST-049 | Snap guide lines drawn at the matched x / y (carried through the projected frame) | During ST-048 | A snap matched | `lib/panels/stage/chrome.dart:34-57`, `lib/panels/stage/overlay.dart:201-203` | status `observer.snapGuides` | PRESERVE | Feedback. These are the only "guides" in the Stage |
| ST-050 | Scale from a corner handle (nw/ne/se/sw) | Drag a corner square | Active (last-selected) layer is 2D or 2.5D, not locked, not a Group. Handles show on hover (ST-039). Grab radius ≤7 px (≤25 % of box size) | `lib/panels/stage/touch.dart:85-128,298-305`, `native/src/editor/stage.rs:165-263,632-633` | native `stageGesture mode:'scale' handle:'nw'…` → SCALE+POSITION | PRESERVE | Core |
| ST-051 | Proportional corner scale | Shift + corner drag | — | `native/src/editor/stage.rs:196-223` | native | PRESERVE | — |
| ST-052 | Corner scale from the centre (symmetric) | Alt/Option + corner drag | — | `native/src/editor/stage.rs:190-195` | native | PRESERVE | — |
| ST-053 | Proportional scale from the centre | Shift+Alt + corner drag | — | `native/src/editor/stage.rs:196-211` | native | PRESERVE | — |
| ST-054 | Scale one axis from an edge-midpoint handle (n/e/s/w) | Drag an edge square | Same as ST-050 | `lib/panels/stage/touch.dart:99-103`, `native/src/editor/stage.rs:225-249` | native `mode:'scale' handle:'n'…` | PRESERVE | Shift has no effect on edges |
| ST-055 | Edge scale from the centre | Alt + edge drag | — | `native/src/editor/stage.rs:232-247` | native | PRESERVE | — |
| ST-056 | Rotate with the rotation handle (circle 22 px above the top edge; rotates about the layer position, which is the anchor) | Drag the circle handle | Same as ST-050. Grab radius 7 px | `lib/panels/stage/touch.dart:104-111,114-115`, `native/src/editor/stage.rs:412-427,501-521` | native `mode:'rotate' handle:'rotation'` → ROTATION | PRESERVE | Core |
| ST-057 | Rotate in 15° steps | Shift + rotate | — | `native/src/editor/stage.rs:423-425` | native | PRESERVE | — |
| ST-058 | Multi-selection transforms together: other selected layers get the same move, scale ratio or rotation delta. Locked or rejected layers are skipped | Any cage drag with several layers selected | Handles belong to the **last-selected** layer only | `native/src/editor/stage.rs:338-386,640` | native | PRESERVE | Group-edit semantics |
| ST-059 | Transform writes keyframes when Animate is on (`place_checked(..., animate)`). Untouched values are never written | Any cage or 3D drag | `viewer.animate` | `native/src/editor/stage.rs:395-402,703`, `native/src/editor/gizmo3d.rs:462-472` | native | PRESERVE | Animate semantics live in native; the UI only follows |
| ST-060 | Cancel an in-flight drag (camera, working-area or layer): the preview is dropped and nothing is written | Esc while dragging (Stage focused), or pointer cancel | `_pointer != null` or dragging | `lib/panels/stage/chrome.dart:173-183,196`, `lib/panels/stage/touch.dart:180-212` | native `stageGesture phase:'cancel'` / `cancelPreview` | PRESERVE | Undo-free abort |
| ST-061 | Gesture auto-cancels if the document changes mid-drag | Implicit | revision changed | `native/src/editor/stage.rs:690`, `native/src/editor/gizmo3d.rs:425-427` | native error "Gesture canceled because document changed" | PRESERVE | Safety |
| ST-062 | Nudge the selection by 1 comp px (Shift: 10) | Alt + ←/→/↑/↓ | Not typing. Global. ←/→ nudge only when no keyframes are selected (otherwise they move keys) | `lib/input/editor_shortcuts.dart:124-146,188-207` | native `stageGesture move` begin/update/commit | PRESERVE+MOVE? | Stage transform with no pointer. Keep |
| ST-063 | Status "Transform gestures unavailable". Cage and handles are hidden and no drag is possible | Automatic | `!supports('stageGesture')` | `lib/panels/stage/chrome.dart:288-298`, `lib/panels/stage/touch.dart:87` | UI-local | PRESERVE | Capability-degraded state |

### F. 3D layer gizmo (three-axis, `transform-gizmo`)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-064 | Draw the 3-axis gizmo for the active 3D layer (mesh sent by native, drawn exactly where it hit-tests) | Automatic | Active layer projection is 3D and not locked. Not a Camera or Stage layer. Not playing. Stage uses `stageSpatialGizmo`, Camera uses `spatialGizmo` | `lib/panels/stage/spatial.dart:12-47`, `lib/panels/stage/geometry.dart:32-87`, `native/src/snapshot.rs:157-167`, `native/src/editor/gizmo3d.rs:204-275,313-331` | status mesh | PRESERVE | Core 3D manipulation |
| ST-065 | Hover highlights the gizmo part under the pointer (accent); at rest it is muted | Hover over the mesh (6 px slack) | `supports('stageGesture')`. No button pressed | `lib/panels/stage/touch.dart:339-362,395-416`, `lib/panels/stage/geometry.dart:52-61` | native `stageGesture phase:'hover'` (not a preview) | PRESERVE | Shows which axis will move |
| ST-066 | Default gizmo = **translate + rotate** (per axis and plane, local orientation). Drag a part to move or rotate | Primary drag on a mesh triangle | As ST-064 | `lib/panels/stage/touch.dart:306-309`, `native/src/editor/gizmo3d.rs:44-51,350-416,418-479` | native `stageGesture mode:'spatial'` → POSITION, POSITION_Z, ROTATION_X/Y, ROTATION, SCALE, SCALE_Z | PRESERVE | — |
| ST-067 | **Hold P** to narrow the gizmo to translate only | Hold P (no Cmd/Ctrl/Alt) | 3D active and not typing. Key repeat is swallowed. P **also** reveals Position in the Inspector (global) | `lib/panels/stage/touch.dart:9-16,364-393`, `native/src/editor/gizmo3d.rs:46`, `lib/input/editor_shortcuts.dart:163-176` | native `held:'position'` via hover or gesture | PRESERVE | AE's P |
| ST-068 | **Hold R** to narrow the gizmo to rotate only | Hold R | Same | same, `native/src/editor/gizmo3d.rs:47` | native `held:'rotation'` | PRESERVE | AE's R |
| ST-069 | **Hold S** for scale-only gizmo. This is the **only** way to scale a 3D layer on the Stage | Hold S | Same | same, `native/src/editor/gizmo3d.rs:48` | native `held:'scale'` | PRESERVE+VISUALIZE | Hidden capability. Consider a visible mode affordance, but keep the held key |
| ST-070 | Snap the 3D gizmo drag (transform-gizmo snapping: distance 10 screen px, scale step 0.1, crate-default angle step) | Hold Shift during a 3D drag | — | `native/src/editor/gizmo3d.rs:69-70,430` | native `shift:true` | PRESERVE | — |
| ST-071 | Multi-select 3D: every selected 3D layer moves with the gizmo | 3D drag with several 3D layers selected | — | `native/src/editor/gizmo3d.rs:204-275,447-477` | native | PRESERVE | — |
| ST-072 | Refusals, shown as errors: parent shears; skew ≠ 0 ("Clear Skew before using the 3D gizmo"); split Position X/Y ("Rejoin the separate Position axes…"); flattened axis; not round-trippable | Attempt a 3D grab | Layer state | `native/src/editor/gizmo3d.rs:229-247,452-455` | native error → `c.error` | PRESERVE | These tell the user why the gizmo will not work |

### G. Camera layers on the Stage (Boxcam)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-074 | Draw every visible camera layer: box on the comp plane (open where unseen), Blender-style eye plus frustum pyramid plus up-triangle, and target cross | Automatic | Stage only. Camera layer not hidden, in its time range. Not playing | `lib/panels/stage/camera.dart:42-60`, `lib/panels/stage/overlay.dart:121-174`, `native/src/snapshot.rs:169-200` | status `cameraGizmos` | PRESERVE+VISUALIZE | 3D camera legibility |
| ST-075 | Select a camera by clicking its box edge (±6 px) and immediately drag to move its **Center** | Primary press on the camera box edge, then drag | Stage only, front (orbit 0). Camera `authorable` (orbit 0, no target layer, all 4 corners visible) | `lib/panels/stage/touch.dart:276-286`, `lib/panels/stage/camera.dart:87-94,161-168` | native `select` + `previewProperties camera.center` → `commitPreview` on release | PRESERVE | Boxcam authoring |
| ST-076 | Zoom the camera by dragging a corner handle (ratio of distance from box centre) | Drag one of 4 corner squares | Stage only, front. The camera is selected and authorable | `lib/panels/stage/camera.dart:73-85,169-176`, `lib/panels/stage/touch.dart:265-275` | `previewProperties camera.zoom` → `commitPreview` | PRESERVE | — |
| ST-077 | Roll the camera by dragging the roll handle (circle 22 px above the top edge) | Drag the roll circle | Same | `lib/panels/stage/camera.dart:80-84,150-160` | `previewProperties camera.roll` → `commitPreview` | PRESERVE | — |
| ST-078 | Camera handles drawn only for the selected authorable camera | Automatic | Stage only, front, not playing | `lib/panels/stage/chrome.dart:401-403`, `lib/panels/stage/overlay.dart:132-145` | UI-local | PRESERVE+CONTEXTUALIZE | Already contextual |
| ST-079 | Esc or pointer-cancel during a camera drag cancels it (no undo entry) | Esc / cancel | During ST-075 to ST-077 | `lib/panels/stage/touch.dart:180-183`, `lib/panels/stage/camera.dart:182-194` | `cancelPreview` | PRESERVE | — |

### H. Working area (Stage layer extent, "Extend")

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-080 | Toggle **Extend** (arm the working-area edges for dragging). Label reads "● Extend" while on | Bottom-bar button "Extend" / "● Extend" | Stage only | `lib/panels/stage/chrome.dart:266-276,64-65`, `lib/panels/stage/camera.dart:34-40` | `c.storeDesk('stageExtend', on)` (persisted in deskWork settings) | PRESERVE+CONTEXTUALIZE | Mode toggle |
| ST-081 | Turning Extend on when no Stage layer exists **creates** a Stage (working-area) layer | Same button | `_extent == null` and `supports('create')` | `lib/panels/stage/camera.dart:37-39` | native `create {kind:'stage'}` (`native/src/port.rs:189`, `native/src/editor/create.rs`) | PRESERVE | Side effect that must survive: a document edit made from a view toggle |
| ST-082 | Drag a working-area edge outward or inward to set that side's margin (top, right, bottom, left; ≥0; scaled by observer distance) | Primary drag within 6 px of an extent edge | Stage only, front, Extend on, Stage layer exists | `lib/panels/stage/camera.dart:24-32,115-139`, `lib/panels/stage/touch.dart:257-264` | `previewProperties stage.left/top/right/bottom` → `commitPreview` | PRESERVE | Boxcam working comp |
| ST-083 | Working-area outline drawn (bold when armed, muted otherwise) | Automatic | Stage only. A Stage layer exists | `lib/panels/stage/overlay.dart:112-120`, `lib/panels/stage/chrome.dart:405-406` | status `observer.extent.points` | PRESERVE | — |

### I. Anchor, colour pick, composition ground (from the Stage)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-084 | Anchor preview: while hovering the Inspector's anchor pad, a cross and circle mark on the Stage shows where the pivot would land (bilinear in corners) | Hover a cell of the Inspector anchor pad | A selected, grabbable layer. Not playing | `lib/panels/inspector/transform_card.dart:95`, `lib/panels/stage/chrome.dart:340-349`, `lib/panels/stage/overlay.dart:108-111` | `c.anchorPreview` (UI-local) | PRESERVE+VISUALIZE | **The only anchor affordance on the Stage.** No direct anchor drag exists (the edit goes through Inspector `anchor`) |
| ST-085 | Eyedropper: the next click on the Stage reads the rendered pixel and applies it to the colour target (else the selection) | Arm the eyedropper in the Browser colour picker, then click the Stage | `c.eyedropper` true; `supports('pickColor')`. `applyPalette` if supported. One-shot (disarms on click) | `lib/panels/stage/touch.dart:214-232`, `lib/panels/browser/color_picker.dart:274-282`, `native/src/port.rs:259,219` | native `pickColor {x,y}` → `applyPalette {rgba}` | PRESERVE | Cross-panel tool that ends on the Stage. Note: native samples the **output (Camera) render** at comp x,y, so on an orbited Stage the pixel read may not be the one under the cursor |
| ST-086 | Toggle transparent ground (background alpha 0 ↔ 1, RGB kept). Tooltip explains that the export carries alpha. One undo | Bottom-bar switch with grid glyph | `supports('composition')` | `lib/panels/stage/chrome.dart:22-32,251-265` | native `composition {background:[r,g,b,a]}` | PRESERVE+MERGE(ST-094) | The code comment says "back to black" but the code keeps RGB |
| ST-087 | Readout: comp size "W × H" | Always (bottom bar) | — | `lib/panels/stage/chrome.dart:244-250` | status `width/height` | VISUALIZE+MOVE | Could open Composition settings on click (currently not clickable) |
| ST-088 | Readout: "Frame N" (current frame) | Always (bottom bar) | — | `lib/panels/stage/chrome.dart:278-287` | `c.frame` | PRESERVE+MOVE | Duplicates the Timeline/Transport readout |

### J. Composition settings sheet (`CompositionControls`)

Opened from the top-bar button "Composition" (toggles the sheet; tap outside to close) or with Cmd/Ctrl+Alt+K. See `lib/app/editor_window.dart:500-509,561-578` and `lib/input/editor_shortcuts.dart:90-95`.

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-089 | Open or close the Composition sheet | Top-bar "Composition" / Cmd+Alt+K / click outside / Esc | Not typing (shortcut) | `lib/app/editor_window.dart:500-509,555-558,67`, `lib/input/editor_shortcuts.dart:37,90-95` | UI-local `sheet` | MOVE+CONTEXTUALIZE | Could live near the Stage size readout (ST-087) |
| ST-090 | Size presets 16:9 (1920×1080), 9:16 (1080×1920), 1:1 (1080×1080), 4K (3840×2160) | Click a preset button | — | `lib/panels/composition_controls.dart:32-48` | native `composition {width,height}` | PRESERVE+VISUALIZE | Could show aspect thumbnails |
| ST-091 | Type width (px) | Field "width" → Enter or blur. Esc cancels | Positive integer (validator "Enter a positive integer") | `lib/panels/composition_controls.dart:49-67`, `lib/foundation/panel_controls/fields.dart:69-160` | native `composition {width}` | PRESERVE | — |
| ST-092 | Type height (px) | Field "height" | Same | same | native `composition {height}` | PRESERVE | — |
| ST-093 | Type duration (frames) | Field "durationFrames" | Same | same | native `composition {durationFrames}` | PRESERVE+MOVE | Also a Timeline concern |
| ST-094 | Background grey presets Black (0) / Dark (0.12) / Grey (0.5) / White (1). The current one is highlighted. These always set alpha 1 | Click a preset | — | `lib/panels/composition_controls.dart:68-87,134-135` | native `composition {background:[g,g,g,1]}` | PRESERVE+MERGE(ST-086) | A preset silently turns transparency off |
| ST-095 | Pick an arbitrary background colour: the swatch routes the Browser colour picker to the Background slot | Click the background swatch (tooltip "Pick the background in the Browser") | `supports('focusColor')`; otherwise the swatch is read-only | `lib/panels/composition_controls.dart:88-101`, `lib/foundation/color_field.dart:26-34` | `c.focusColor({slot:'Background'})` → native `focusColor` | PRESERVE+VISUALIZE | Cross-panel colour routing |
| ST-096 | Frame-rate presets 23.976 / 24 / 25 / 29.97 / 30 / 59.94 / 60. The current one is highlighted | Click a preset | — | `lib/panels/composition_controls.dart:104-125` | native `composition {fpsNum,fpsDen}` | PRESERVE | No custom fps entry exists (note only) |

### K. Settings touching the Stage, and adjacent panels in scope

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST-073 | "Outside dim" −/+ (5 % steps, shows %) in Settings ▸ View | Settings sheet buttons | — | `lib/app/editor_window.dart:589-607,84,211` | UI-local `dim`, persisted via `writeSettings` | PRESERVE (fix wiring) | **Not connected**: the Stage overlay uses a hard-coded 0.55 (`lib/panels/stage/overlay.dart:100`). The control exists but has no visible effect |
| ST-097 | Web panel: edit the reference URL (default pinterest.com). Commits on Enter or blur | Field "Website" | — | `lib/panels/web_panel.dart:18-23` | `c.storeDesk('webUrl')` | PRESERVE+MOVE | Reference-board utility, not Stage |
| ST-098 | Web panel: open the URL in the system browser (flushes editors first) | Button "Open in browser" | — | `lib/panels/web_panel.dart:24-49` | native channel `openWeb {url}` (`macos/Runner/MainFlutterWindow.swift:805`) | PRESERVE+MOVE | Same |
| ST-099 | Native visual specimens (small renders in Effects, Fill, Colour and Gradient shelves), throttled and cached | Automatic when those shelves scroll into view | `state.visualSamples == true` | `lib/panels/native_visual_sample.dart:13-169`; used by `lib/panels/browser/effects_shelf.dart:179`, `fill_definitions.dart:105,134`, `color_values.dart:27`, `lib/panels/gradient_inspector.dart:238` | native `request {op:'visualSample'}` | PRESERVE | Not a Stage capability, and no user trigger of its own. Infrastructure for Browser previews |
| ST-100 | Timeline inertial pan and scrub-zoom (`ViewportMotion`) | Timeline gestures | — | `lib/input/viewport_motion.dart:1-107`, used only at `lib/panels/timeline/view.dart:18-19,120` | UI-local | PRESERVE | **Not used by the Stage.** The Stage has no inertia and no pinch. Listed so the owner (Timeline audit) picks it up |

---

## Explicitly absent in the current Stage (checked, not found)

These were in the brief's checklist. They are listed so nobody assumes they exist:

- **Orthographic or named views** (Front/Top/Left…): none. "Front" only resets the observer (ST-010).
- **Create-by-drag** on the Stage (draw a shape or text box): none. Creation happens from the Browser shelves (`create_shelf.dart:124`, `fonts_shelf.dart:294`, `media_shelf.dart:131`). There is also no drop target on the Stage: assets drop on the Timeline only.
- **Text in-place editing** on the Stage: none. Double-click on a layer = observer focus (ST-012).
- **Direct anchor-point dragging**: none (anchor preview only, ST-084).
- **User guides, rulers, grid, safe-area overlays**: none. The only lines are Cmd-snap guides (ST-049) and the transparency checker (ST-008).
- **Right-click context menu** on the Stage: none. Right-drag = orbit (ST-013).
- **Trackpad pinch / two-finger pan** on the Stage: no `onPointerPanZoom*` handler (ST-028).
- `native/src/editor/placement_edit.rs` is **not** a Stage feature: it is `expandEffect` (Inspector effect "expand", `native/src/port.rs:223`).
- `native/src/editor/create.rs` is reachable from the Stage only through ST-081 (`kind:'stage'`).

## UI-local state to preserve

| State | Where | Persisted? | Notes |
|---|---|---|---|
| `_zoom` (null = fit), `_pan` | `lib/panels/stage/view.dart:12-13` | No (per panel instance) | Each view (Stage, Camera) has its own. Cmd+0/1/=/- hits all visible ones |
| `stageExtend` (Extend armed) | `c.deskWork['stageExtend']`, `lib/panels/stage/view.dart:59` | Yes (deskWork via writeSettings) | — |
| Observer camera (orbit, centre, distance) | native `viewer.user_camera` (`native/src/viewer.rs:46`) | No (viewer state, not document) | Front, orbit, focus and fit all write it. Not undoable |
| Hovered layer `_touched`, pointer-inside `_inside` | `lib/panels/stage/touch.dart:32-38,347-351` | No | Drives the hover-cage rule (ST-039) |
| Held key P/R/S `_held` | `lib/panels/stage/touch.dart:364-393`; native `viewer.stage_held` | No | — |
| Gizmo hover point / view / view scale | native `viewer.stage_pointer`, `stage_view`, `stage_view_scale` | No | `view_scale` sets the gizmo size and snap threshold in screen px |
| Snap guides | native `viewer.stage_snap` | No | Cleared on commit |
| `c.anchorPreview` | `lib/session/session_core.dart:53` | No | Written by the Inspector, read by the Stage |
| `c.eyedropper` | `lib/session/session_core.dart:69` | No | Armed in the Browser, consumed by the Stage |
| `c.viewCommand` | `lib/session/session_core.dart:55` | No | Shortcut → Stage bus |
| Stage window (`_sentWindow`, ROI) | `lib/panels/stage/window.dart:56-139`; native `viewer.stage_window` | No | — |
| `webUrl` | `c.deskWork['webUrl']` | Yes | Web panel |
| Settings `dim` | `lib/app/editor_window.dart:84` | Yes (writeSettings `dim`) | Currently unused by the Stage (ST-073) |
| Composition `sheet` open/closed | `lib/app/editor_window.dart` | No | — |

## Suspected duplicates / merge candidates (flag only)

1. **Zoom step: "+/−" buttons (±1 percentage point) vs Cmd+=/− (×1.2)** (ST-020/021 vs ST-022/023). Same intent with different step sizes. Merging onto one step **changes behaviour** for whichever trigger moves.
2. **Front button vs double-click background** (ST-010 vs ST-011): identical `stageView {reset:true}`. Merging the affordance does not change behaviour. Keep both triggers.
3. **Fit on the Stage also resets orbit and frames the working area** (ST-015/016), while Fit on the Camera tab is purely 2D. Unifying them would change behaviour on one view.
4. **Transparent-ground switch (Stage bar) vs background presets (Composition sheet)** (ST-086 vs ST-094/095). They edit the same `composition.background`. The presets force alpha 1, which silently turns transparency off. Merging into one "Ground" control (colour + alpha) keeps behaviour if the alpha is kept explicit. The implicit reset would become visible, which is a mild behaviour change.
5. **"Frame N" readout on the Stage** (ST-088) duplicates the transport/timeline frame readout. Moving it does not change behaviour.
6. **Stage and Camera tabs** (ST-001/002) are one widget with a `view` flag. Merging them into one viewport with a view switch changes behaviour only if the user loses the ability to see both at once.
7. **P/R/S held on the Stage vs P/R/S reveal in the Inspector**: the same key does both at once by design ("one letter means one thing everywhere"). Not a duplicate. Keep them coupled.
8. **Selection hit-rule inconsistency**: click excludes Group layers (ST-034) but marquee includes them (ST-036, `lib/panels/stage/touch.dart:469-473` filters only `locked`). Aligning the two **changes behaviour** of marquee.
9. **Space is overloaded**: Space+drag pan (ST-030) and Space = play/pause (`lib/input/editor_shortcuts.dart:43-46`) fire together. Resolving this changes behaviour.

## Debug / prototype / dead affordances (still listed, nothing deleted)

- **PROBE window-lag logging** (`lib/panels/stage/window.dart:3-6,77-84,133-156`): the `kDebugMode`-only `debugPrint('PROBE room=stage-window …')`. Debug only, and not user-reachable.
- **"Transform gestures unavailable"** (ST-063): only appears against a runtime without `stageGesture`, meaning an older or partial native build. A degraded-mode affordance.
- **Dead native modes**: `GizmoMode::Orbit` / `GizmoMode::Depth` and the `Move` arm of `preview_values` in `native/src/editor/stage.rs:18-23,274-289,322-336`. No UI handle string maps to Orbit or Depth, and Move is handled in `CageDrag::edits` (`stage.rs:692-706`). Unreachable, so not a capability.
- **Settings "Outside dim"** (ST-073): a live control with no effect on the Stage (hard-coded 0.55). Looks like a prototype hook left unwired.
- **`_inside` flag** (`lib/panels/stage/touch.dart:32-38`): its comment says Camera-tab gizmos show only while the pointer is inside. In practice it only clears the hover cage on exit, which the Stage tab does too.
- **Suspected bug, not an affordance**: pressing Esc during a right-drag orbit calls `_finish(true)`, which never resets `_orbiting` (`lib/panels/stage/touch.dart:180-212` vs `426-435,458-462`). The next primary drag would then orbit instead of select or move until release. Verify in a real window.
- **Eyedropper coordinate mismatch** (ST-085): `pickColor` samples the output render at comp x,y (`native/src/port.rs:259`). On an orbited or zoomed Stage, `_toComp` gives a coordinate in the Stage's comp-image space, which is not necessarily the output pixel under the cursor.
