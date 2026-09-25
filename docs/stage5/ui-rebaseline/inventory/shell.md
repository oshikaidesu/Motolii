# Inventory — window shell, menus, shortcuts, workspace, settings (Phase A, read-only)

Scope: `motolii/ui/lib/{main.dart, app/*, workspace/*, foundation/panel_catalog.dart, panels/registry.dart, panels/panel_settings.dart, input/editor_shortcuts.dart, session/*, bridge/*}`, `ui/native/src/{lib.rs, port.rs, viewer.rs, editor/keymap.rs, editor/clipboard.rs, editor/history.rs}`, `ui/macos/Runner/*`.
All paths below are relative to `motolii/ui/`. "cmd" in Dart = Meta **or** Control (`editor_shortcuts.dart:29`).

Disposition vocabulary: PRESERVE / MOVE / MERGE / CONTEXTUALIZE / VISUALIZE (no DELETE).

## 1. Capability inventory

### SH — window shell and menus

| ID | Capability | Trigger | Condition | Code (file:line) | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| SH-01 | New document | File ▸ New; Cmd+N | Save-changes dialog first when dirty (SH-26) | `lib/app/editor_window.dart:382-384`, `:479`; `lib/input/editor_shortcuts.dart:87-90` | `c.command('new')` → Rust `port.rs:246` "new" (requiresPause) | PRESERVE + MOVE | Core file verb. Can live in a compact File menu / app menu; must keep the confirm step |
| SH-02 | Open document (.rrd) | File ▸ Open; Cmd+O | Confirm dirty first | `editor_window.dart:385-387`; `session_files.dart:62-65` | `native('pickOpen')` (NSOpenPanel, .rrd) → `native('open',{path})` → Swift `runtime.open` → broadcast to all windows | PRESERVE + MOVE | Same |
| SH-03 | Save | File ▸ Save; Cmd+S | Untitled → falls to Save panel | `editor_window.dart:388-390`; `session_files.dart:67-71`; `session_commands.dart:57-63` | `native('flushEditors')` then `command('save',{path})` → `port.rs:242` | PRESERVE | Flushes pending field edits in all windows before saving — New shell's fields must register in `pendingEditors` too |
| SH-04 | Save as | File ▸ Save as; Cmd+Shift+S | — | `editor_window.dart:391-393`; `editor_shortcuts.dart:79-82` | `native('pickSave',{name:'Untitled.rrd'})` → `save` | PRESERVE | |
| SH-05 | Import files/folders | File ▸ Import | Extensions from Rust `state.importExtensions`; unsupported names reported in status line | `editor_window.dart:394-396`; `session_files.dart:85-120` | `native('pickImport',{extensions})` → `command('import',{paths})` → `port.rs:234`; new asset ids → `importedAssets` (Media shelf selects them) | PRESERVE + MOVE | Natural home is the Browser/Media area; keep a menu route too |
| SH-06 | Run Script… (.js) | File ▸ Run Script… | Opens picker (js) | `editor_window.dart:397-399`; `session_files.dart:74-81` | `command('runScript',{path})` → `port.rs:240` → `editor/script.rs:50` | PRESERVE | Product thesis ("CLI JS") — must stay 1 click away |
| SH-07 | Rerun Script | File ▸ Rerun Script | Refuses if doc changed after script (Rust) | `editor_window.dart:400-402`; `session_files.dart:83` | `command('rerunScript')` → `port.rs:241` | PRESERVE | Candidate for a keyboard shortcut in New UI (none today) |
| SH-08 | Undo | Edit ▸ Undo; Cmd+Z | — | `editor_window.dart:418`; `editor_shortcuts.dart:62` | `command('undo')` → `port.rs:247` (`doc.undo()`, clears key selection) | PRESERVE | |
| SH-09 | Redo | Edit ▸ Redo; Cmd+Shift+Z | — | `editor_window.dart:419`; `editor_shortcuts.dart:62` | `command('redo')` → `port.rs:248` | PRESERVE | |
| SH-10 | Cut | Edit ▸ Cut; Cmd+X; Timeline context menu | — | `editor_window.dart:420`; `editor_shortcuts.dart:64`; `panels/timeline/menu.dart:25` | `command('cut')` → `port.rs:190` (Rust process-local clipboard, layers or keys) | PRESERVE | |
| SH-11 | Copy | Edit ▸ Copy; Cmd+C; Timeline menu | — | `editor_window.dart:421`; `editor_shortcuts.dart:63` | `command('copy')` → `port.rs:190` | PRESERVE | |
| SH-12 | Paste | Edit ▸ Paste; Cmd+V; Timeline menu | — | `editor_window.dart:422`; `editor_shortcuts.dart:65` | `command('paste')` → `port.rs:194` | PRESERVE | |
| SH-13 | Duplicate | Edit ▸ Duplicate; Cmd+D; Timeline menu | — | `editor_window.dart:423`; `editor_shortcuts.dart:66-68` | `command('duplicate')` → `port.rs:195` | PRESERVE | |
| SH-14 | Split at playhead | Edit ▸ Split; Cmd+K; Timeline menu | — | `editor_window.dart:427`; `editor_shortcuts.dart:91-96` | `command('split')` → `port.rs:204` | PRESERVE + CONTEXTUALIZE | Timeline-centric verb |
| SH-15 | Delete (layers, or selected keys when keys are selected) | Edit ▸ Delete; Delete/Backspace; Timeline menu | Keys take priority over layers (Rust `delete_selection` `port.rs:115-119`) | `editor_window.dart:424`; `editor_shortcuts.dart:118-119` | `command('delete')` → `port.rs:199` | PRESERVE | |
| SH-16 | Group | Edit ▸ Group; Cmd+G; Timeline menu | — | `editor_window.dart:425`; `editor_shortcuts.dart:74` | `command('group')` → `port.rs:200` | PRESERVE | |
| SH-17 | Ungroup | Edit ▸ Ungroup; Cmd+Shift+G; Timeline menu | — | `editor_window.dart:426`; `editor_shortcuts.dart:74` | `command('ungroup')` → `port.rs:201` | PRESERVE | |
| SH-18 | Show a panel by name (17 entries: Desk, Stage, Camera, Timeline, Inspector, Create, Media, Effects, Fonts, Colors, Files, Depth, Ease, Blend, Notes, Web, History) | View ▸ <name> | Detached → focuses its window; drawer panel → opens in Desk | `editor_window.dart:499`, `:414-415`, `:294-310`; `workspace/panel_ids.dart:3` | `c.placePanel(name,'show')` → `native('placePanel')` → Swift routes to main window → `panelPlacementRequested` → `_placePanel` (UI-local) | PRESERVE + MOVE | In New UI this becomes "reach every surface"; the list is the authoritative surface roster |
| SH-19 | Reset layout | View ▸ Reset layout | Closes detached windows' panels from dock | `editor_window.dart:403-412`; `workspace/layout.dart:146-166` | UI-local dock reset + `setPaneState` + `writeSettings` | PRESERVE (Classic) / MOVE (New: "Reset New layout") | Each shell needs its own reset |
| SH-20 | Composition sheet: aspect presets (16:9, 9:16, …), width/height/durationFrames fields, background presets Black/Dark/Grey/White + swatch → Colors shelf, fps presets | Menu-bar button "Composition"; Cmd+Alt+K | Sheet toggles; tap outside closes | `editor_window.dart:500-510`, `:577-578`; `editor_shortcuts.dart:91-93`; `panels/composition_controls.dart:31-123` | `command('composition',{...})` → `port.rs:233`; swatch → `focusColor({'slot':'Background'})` → `port.rs:218` + show Colors | PRESERVE + MOVE + CONTEXTUALIZE | Composition settings fit "nothing selected" Inspector state or a Stage header. Presets are hard-coded in the widget (see domain-logic list) |
| SH-21 | Export sheet: All / Marker-to-marker range, summary (size·fps·range·MP4 H.264+AAC), Export… (save panel .mp4), progress done/total, error, Cancel | Menu-bar button "Export" | Range computed from markers around playhead; progress polled every 300 ms while sheet is open | `editor_window.dart:579-580`; `panels/export_controls.dart:17-101` | `native('pickExport')` → `command('export',{path,start,end})` `port.rs:238`; `exportStatus` (early-return status); `cancelExport` `port.rs:239` | PRESERVE + MOVE | Keep as a dedicated export surface; note polling dies when the sheet closes (`export_controls.dart:21,93-100`) — progress then only updates via other status replies |
| SH-22 | Settings sheet (see ST8-*) | Menu-bar button "Settings" | — | `editor_window.dart:581-643` | — | PRESERVE + MOVE | |
| SH-23 | Document title with dirty marker ("• name.rrd" / "Untitled") | Always, main window | — | `editor_window.dart:512-520` | reads slice `title` [dirty,path] | PRESERVE | Could move to native window title |
| SH-24 | Status line: last operation error → freeze progress ("Freezing X n/N", "Freeze failed") → effect catalog errors ("Effects: …") | Always | — | `editor_window.dart:23-38`, `:40-52`, `:539-550`; `session_values.dart:3-9` | reads `c.error`, slice `notice` | PRESERVE + VISUALIZE | Freeze progress could become a track badge; text must remain |
| SH-25 | Sheets close on outside tap / Esc | Tap scrim; Esc | — | `editor_window.dart:553-558`; `editor_shortcuts.dart:36-37` | UI-local | PRESERVE | |
| SH-26 | "Save changes?" dialog (Cancel / Don't Save / Save) | New, Open, main-window close, app quit | Only when `state.dirty` | `editor_window.dart:347-377`; host `confirmClose` `session_native.dart:126-129`; Swift `MainFlutterWindow.swift` `confirmTermination` | UI dialog → `save()` | PRESERVE | Installed via single-slot callback `c.confirmClose` (`editor_window.dart:100`) |
| SH-27 | Drop files anywhere on window → import (unless a panel claims the drop via `fileDropTarget`); drag-hover signal for shelves | OS drag-and-drop | — | `editor_window.dart:112-114`; `session_native.dart:188-195`; Swift `MainFlutterWindow.swift:28-43` | `filesDropped` → `importPaths` → `import`; `dragHover` → `c.dragging` | PRESERVE | |
| SH-28 | Launch with a document or a `.js` script | `--dart-define=MOTOLII_DOCUMENT=<path>` (`motolii-ui.sh dev <doc>`) | `.js` opens blank then `runScript` | `session_files.dart:6`, `:42-51`, `:31-35` | `open` / `runScript` | PRESERVE | Same mechanism is the natural shell-selection seam (Part 3) |
| SH-29 | Detached panel windows (native NSWindow per panel, title "Motolii · <panels>", own Flutter engine, same document/texture) | Tab ▸ Detach; Settings ▸ Panels ▸ Window; restore on launch | Main window keeps the registry `detached` | `editor_window.dart:330-345`, `:130-136`; Swift `openPanelWindow` + `PanelFlutterWindow` | `native('openPanelWindow',{panels})`; second engine attaches to the same runtime | PRESERVE | Multi-monitor workflow; New UI must keep a "pop out" route |
| SH-30 | First launch opens Notes as a detached window | Launch with no saved placements | — | `editor_window.dart:164-172` | `_placePanel('Notes','window')` | PRESERVE (Classic) / decide per shell | Default-placement policy, not a capability; New shell may place Notes differently |
| SH-31 | Effect shelf hot reload (vism/ file watch → reload catalog → redraw) | File change in vism/ | Main window only | `session_native.dart:161-175`; Swift `effectsChanged` | `command('reloadEffects')` (`port.rs:129` early return) + `refreshPreview` | PRESERVE | Background capability; must remain wired to whichever shell is mounted |
| SH-32 | Theme/pane state mirrored to every window | Any theme/placement change | — | `session_native.dart:142-153`; `theme_settings.dart:23-30` | `native('setPaneState')` → Swift broadcasts `paneState` | PRESERVE | |
| SH-33 | Global "no hover tooltips" | Always | `EditorApp.noHover` wraps the app | `app/editor_app.dart:12-14`, `:59` | UI-local (`EditorLook(tooltips:false)`) | PRESERVE | Current policy; New UI shares the wrapper if mounted under EditorApp |
| SH-34 | Desktop scroll behaviour (all pointer kinds drag, clamp, editor scrollbar) | Always | — | `editor_app.dart:78-104` | UI-local | PRESERVE | Shared by both shells |
| SH-35 | Native macOS menu bar | App menu (About, Preferences… Cmd+,, Services, Hide, Quit), Edit (Undo/Redo/Cut/Copy/Paste/Paste and Match Style/Delete/Select All/Find…/Spelling/Substitutions/Transformations/Speech), View (Enter Full Screen), Window (Minimize, Zoom, Bring All to Front), Help | Standard Flutter template; selectors go to the responder chain, **not** to Motolii intents (only text inputs react) | `macos/Runner/Base.lproj/MainMenu.xib` | AppKit responder chain | PRESERVE + (future) MERGE | Not a Motolii capability today; a New shell could route these to intents via `PlatformMenuBar`, but that would change Classic's behaviour — user decision |
| SH-36 | Window close / quit flushes all windows' editors, confirms, shuts runtime down | Close main window; Cmd+Q | — | `macos/Runner/AppDelegate.swift:9-25`; `MainFlutterWindow.swift:44-60` | Swift `confirmTermination` → `flushEditors` everywhere → `confirmClose` on main → `shutdown` | PRESERVE | Host-level, shell-agnostic |
| SH-37 | Recent files | — | **Does not exist** (no "recent" anywhere in `lib/`) | — | — | n/a | Recorded so nobody counts it as lost |

### KB — keyboard shortcuts (global handler)

All handled in `EditorShortcuts.handle` (`lib/input/editor_shortcuts.dart:26-186`), attached to the root `Focus(autofocus:true)` (`editor_window.dart:461-463`). Ignored while an `EditableText` has focus (`:19-27`). Only KeyDown. `ui/native/src/editor/keymap.rs` contains no keymap (just `enum EaseSide {Both,In,Out}`); **all bindings live in Dart**.

| ID | Capability | Trigger | Condition | Code (file:line) | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| KB-01 | Escape ladder: cancel preview → close sheet → close Desk drawer → clear selection | Esc | — | `editor_shortcuts.dart:34-43` | `cancelPreview` (`port.rs:173`); UI-local; `select {ids:[],keys:[]}` | PRESERVE | The ladder refers to Classic-only notions (sheet, deskDrawer) — New shell passes its own `hasSheet/closeSheet` |
| KB-02 | Play / pause | Space | — | `:44-47` | `togglePlayback` → `play`/`pause` (`port.rs:257-258`) + Swift playback timer | PRESERVE | |
| KB-03 | Easy Ease / Ease In / Ease Out on selected keys, then focus keyframes | F9 / Shift+F9 / Cmd+Shift+F9 | — | `:48-60` | `ease {kind}` (`port.rs:216`); `focusEditing(layer,'keyframes')` | PRESERVE | AE convention |
| KB-04 | Undo / Redo | Cmd+Z / Cmd+Shift+Z | — | `:62` | `undo`/`redo` | PRESERVE | |
| KB-05 | Copy / Cut / Paste | Cmd+C / Cmd+X / Cmd+V | — | `:63-65` | `copy`/`cut`/`paste` | PRESERVE | |
| KB-06 | Duplicate | Cmd+D | — | `:66-68` | `duplicate` | PRESERVE | |
| KB-07 | Reselect previous key selection | Cmd+Shift+D | `previousKeys` recorded in `absorb` | `:69-71`; `session_commands.dart:43-47`; `session_snapshot.dart:34-42` | `select previousKeys` | PRESERVE | Dart-held memory of the prior key selection |
| KB-08 | Group / Ungroup | Cmd+G / Cmd+Shift+G | — | `:74` | `group`/`ungroup` | PRESERVE | |
| KB-09 | Select all layers | Cmd+A | — | `:75-78` | `select {ids: all layer ids}` (ids enumerated in Dart) | PRESERVE | |
| KB-10 | Save / Save as | Cmd+S / Cmd+Shift+S | — | `:79-82` | via `onMenu` | PRESERVE | |
| KB-11 | Open | Cmd+O | — | `:83-86` | via `onMenu` | PRESERVE | |
| KB-12 | New | Cmd+N | — | `:87-90` | via `onMenu` | PRESERVE | |
| KB-13 | Split | Cmd+K | — | `:91-96` | `split` | PRESERVE | |
| KB-14 | Composition settings | Cmd+Alt+K | — | `:91-93` | `showComposition` (shell callback) | PRESERVE + MOVE | Target surface changes in New UI |
| KB-15 | Stage view Fit / Actual / Zoom in / Zoom out | Cmd+0 / Cmd+1 / Cmd+= / Cmd+- | Applies to the visible Stage (`Visibility.of`) | `:97-116`; `panels/stage/view.dart:118-` | `c.viewCommand` (UI-local); Fit on User stage also sends `stageView {fit:true}` (`stage/view.dart:98-100`) | PRESERVE | |
| KB-16 | Delete | Delete / Backspace | — | `:118-119` | `delete` | PRESERVE | |
| KB-17 | Go to start / end | Home / End | — | `:120-127` | `seek 0` / `seek durationFrames-1` | PRESERVE | |
| KB-18 | Step playhead ±1 / ±10 frames | ← / → (Shift ×10) | — | `:128-139` | `seek` (`port.rs:260`) | PRESERVE | |
| KB-19 | Move selected keys ±1/±10 frames, else nudge layer X | Alt+← / Alt+→ (Shift ×10) | keys selected → moveKeys | `:132-137`, `:188-207` | `moveKeys {deltaFrames}` (`port.rs:215`) or synthesized `stageGesture` begin/update/commit | PRESERVE | Nudge is synthesized in Dart (see domain logic) |
| KB-20 | Select previous / next layer | ↑ / ↓ | — | `:141-156` | `select {ids:[neighbor]}` computed in Dart | PRESERVE | |
| KB-21 | Nudge layer Y ±1/±10 | Alt+↑ / Alt+↓ | — | `:144-145` | synthesized `stageGesture` | PRESERVE | |
| KB-22 | Add marker at playhead | M | — | `:158` | `addMarker` (`port.rs:232`) | PRESERVE | |
| KB-23 | Toggle "keyed only" | U | — | `:159-162` | sets `c.keyedOnly` — **nothing reads it** (only writer in `lib/`) | PRESERVE (flag as inert) | AE's U; wiring is missing, not a design choice. Record, do not drop |
| KB-24 | Toggle Animate (record keys) | A | not with Shift | `:163-166`; `session_commands.dart:16-20` | `animate {enabled, from, shape}` (`port.rs:222`) | PRESERVE | |
| KB-25 | Reveal property in Inspector: Position / Scale / Rotation / Opacity (T) / Anchor (Shift+A) | P / S / R / T / Shift+A | Shows Inspector first | `:167-179` | `showInspector` + `c.focusProperty` (UI-local) | PRESERVE + CONTEXTUALIZE | AE letters |

Panel-local shortcuts (owned by other audit areas, listed so the shell inventory is complete):

| ID | Capability | Trigger | Code |
|---|---|---|---|
| KB-P1 | Hold P/R/S while dragging on Stage = move/rotate/scale mode | hold P/R/S | `panels/stage/touch.dart:12-16` |
| KB-P2 | Space+drag / middle-drag pans Stage | Space held | `panels/stage/touch.dart:288-292` |
| KB-P3 | Esc cancels camera drag | Esc | `panels/stage/chrome.dart:172-180` |
| KB-P4 | Timeline focus: Esc cancel gesture; ←/→ move keys or seek | Esc, ←/→ | `panels/timeline/view.dart:20-37` |
| KB-P5 | Browser shelves: Cmd+F search, Esc clear, Cmd+E quick-add, 1..N add to collection, Cmd+A select all, Enter apply, arrows/Home/End move, Delete remove | various | `panels/browser/frame_keys.dart:12-73` |
| KB-P6 | Notes: Cmd+V paste (system pasteboard via `noteClipboard`), Delete block | Cmd+V, Delete | `panels/notes_desk.dart:222-235` |
| KB-P7 | Ease desk: Esc cancel; arrows/Enter/Home/End on preset grid | various | `panels/ease_desk.dart:68-76`, `:225-255` |
| KB-P8 | Gradient: Esc cancel drag; Delete removes stop | Esc, Delete | `panels/gradient_inspector.dart:172-186` |
| KB-P9 | Rich text / Blend: Esc cancels | Esc | `panels/rich_text_editor.dart:192`; `panels/blend_panel.dart` |
| KB-P10 | Generic numeric fields/dials keyboard | arrows etc. | `foundation/panel_controls/{fields,numeric,dials}.dart` |

### WS — workspace / panels

| ID | Capability | Trigger | Condition | Code (file:line) | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| WS-01 | Default 4-zone layout: Browser shelves (Create/Media/Effects/Colors/Files) left · Stage+Camera centre · Inspector over Desk right · Timeline bottom | First run / Reset | — | `workspace/layout.dart:146-166` | UI-local | MOVE | Layout is not the oracle; concept art keeps the same four responsibilities |
| WS-02 | Tabs per dock leaf; click to bring to front; hidden tabs stay mounted (IndexedStack) | Click tab | — | `workspace/workspace_view.dart:170-173`, `:234-235`, `:378-384` | UI-local | PRESERVE (behaviour) / MOVE (look) | Keeping panels mounted preserves their state (scroll, drafts) |
| WS-03 | Drag a tab to another leaf (centre) or to an edge (left/right/top/bottom zones 18 %/22 %) to split | Drag tab | — | `workspace_view.dart:207-220`, `:286-303`; `layout.dart:204-235` | `onMove` → `workspace.move` + publish + persist | PRESERVE (Classic) / MOVE (New) | New shell may offer fewer docking freedoms but must keep "put panel X next to Y" reachable or document the difference |
| WS-04 | Tab context menu: Detach / Close | Right-click tab | — | `workspace_view.dart:222-233` | `onDetach` → `placePanel(name,'window')`; `onClose` | PRESERVE | |
| WS-05 | Close active tab (×) | × button in tab strip | Non-Desk → becomes drawer (if drawer-able) or hidden; Desk in detached window closes window | `workspace_view.dart:359-364`; `editor_window.dart:220-232` | `c.placePanel(name,'drawer'|'hidden')` | PRESERVE | |
| WS-06 | Resize splits (ratio for elastic sides; pixel offset for fixed-extent sides); persisted on release | Drag divider | Min 40 px per side | `workspace_view.dart:88-133`; `layout.dart:57-78` | UI-local + `persist()` | PRESERVE | |
| WS-07 | Tab strip compaction: tabs that do not fit show icon only (name on hover), strip scrolls | Narrow leaf | — | `workspace_view.dart:175-205`, `:337-357` | UI-local | PRESERVE + VISUALIZE | |
| WS-08 | Focus ring on the panel last clicked/focused; drop-target highlight | Pointer down / focus / drag over | — | `workspace_view.dart:304-329` | UI-local | PRESERVE | |
| WS-09 | Empty leaf placeholder "Drop a panel here"; empty subtrees collapse | Leaf emptied | — | `workspace_view.dart:51-55`, `:369-377` | UI-local | PRESERVE | |
| WS-10 | Panel placement model: every panel is `tab` / `window` / `drawer` (Desk) / `hidden` | View menu, tab menu, Settings ▸ Panels, `focusFont`/`focusColor`, Desk | Drawer only for Depth/Ease/Blend/History | `editor_window.dart:240-260`, `:273-328`; `session_files.dart:142-144` | `native('placePanel')` → Swift → main window `panelPlacementRequested` → `_placePanel`; publishes `panePlaces` via `setPaneState` | PRESERVE (semantic) / MOVE | The vocabulary is Classic's; New shell must honour or map it because Settings, Desk and restore read `panePlaces` |
| WS-11 | Desk host with drawers (Depth, Ease, Blend, History) + Tools view | Desk panel | Drawer chosen by: manual pick → selection (keys or >1 layer → Ease; Camera layer → Depth) → idle default | `panels/desk.dart:30-60`, `:96-125`, `:150-190` | `c.deskDrawer`, `c.deskDefault` (UI-local, persisted) | PRESERVE + CONTEXTUALIZE | Already contextual; the selection→drawer rule is UI policy living in a widget (Part 2) |
| WS-12 | "Use X when idle" (Desk default drawer) | Desk header buttons | — | `panels/desk.dart:150-170` | `c.deskDefault` → `persist()` via listener `editor_window.dart:98` | PRESERVE | |
| WS-13 | Detach panel to its own window; re-focus if already detached; closing the window returns Desk to the dock | Tab ▸ Detach, Settings ▸ Window | — | `editor_window.dart:283-287`, `:330-345`, `:102-111` | `openPanelWindow`, `focusWindow`, `closeWindow`, host `windowClosed` | PRESERVE | |
| WS-14 | Layout persistence: dock tree, UI scale, dim, deskDefault, deskWork, panelPlacements → `~/Library/Application Support/MotoliiStage5/layout.json` (debounced 300 ms); restored at launch incl. windows | Any layout change | Detached windows never write | `editor_window.dart:126-174`, `:204-217`; Swift `readSettings/writeSettings` (`MainFlutterWindow.swift`, handler) | `native('writeSettings', {...})` — **whole-file overwrite** | PRESERVE | Both shells share this file (risk, Part 3) |
| WS-15 | Legacy layout migration (Test→Inspector, add Files to browser leaf, add Camera after Stage, dedupe panes) | Restore | — | `workspace/layout.dart:98-143` | UI-local | PRESERVE | |
| WS-16 | Panel catalog: name, category (Work/Browse/Adjust/Note/Session), icon, min size, dock extents, drawer flag | — | — | `foundation/panel_catalog.dart:17-132` | UI-local metadata | PRESERVE + MERGE | New shell should reuse the same catalog (the roster), add its own placement metadata beside it |
| WS-17 | Panel factory: name → widget (Browser shelves share `BrowserPanel(fixedTab)`; Stage/Camera share `StagePanel(view)`); drawer panels get min-size scroll wrapper | — | — | `panels/registry.dart:20-68` | UI-local | PRESERVE (reuse from New shell) | This is the seam that lets New shell host existing panels unchanged |
| WS-18 | Per-panel GlobalKey so a panel keeps state when moved between leaves | — | — | `editor_window.dart:82`, `:458-459` | UI-local | PRESERVE | New shell must use its own key map (Part 3) |
| WS-19 | Cross-panel navigation: font name in Inspector → Fonts shelf targeted to the text layer; colour swatch → Colors shelf targeted at slot | Click font/colour in Inspector, Composition, Gradient | — | `session_commands.dart:24-41`; callers `inspector/content_cards.dart:35`, `inspector/controls.dart:168`, `gradient_inspector.dart:103`, `composition_controls.dart:98` | `textStyleTarget` (UI-local) / `focusColor` (Rust colour target `port.rs:218`) + `placePanel(show)` | PRESERVE + CONTEXTUALIZE | Exactly the brief's "Fonts contextual on Text" — already a route; New shell may inline it |
| WS-20 | Reveal a property row in Inspector from shortcuts | P/S/R/T/Shift+A | — | `editor_window.dart:68`; `panels/inspector.dart:68-110` | `placePanel('Inspector','show')` + `focusProperty` | PRESERVE | |
| WS-21 | Panel roster reachable from View menu (17 names incl. Desk) | — | — | `workspace/panel_ids.dart:3` | — | PRESERVE | New UI must give each a route (inventory cross-check list) |

### ST8 — settings

| ID | Capability | Trigger | Condition | Code (file:line) | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| ST8-01 | Browser tile size (zoom bar) | Settings ▸ Browser ▸ Tile size | also on the shelf itself (`browser/frame_grid.dart:89`) | `panels/panel_settings.dart:28-45` | `storeDesk('browserTile')` → deskWork in layout.json | PRESERVE + MERGE | Duplicate route with the shelf's own zoom |
| ST8-02 | Animate: "Key the start too" | Settings ▸ Animate | — | `panel_settings.dart:54-72`; used `session_core.dart:17`, `session_commands.dart:16-20` | `storeDesk('animateFrom')`; sent as `animate.from` | PRESERVE + CONTEXTUALIZE | Could sit beside the Animate toggle |
| ST8-03 | New layers projection 2.5D / 3D | Settings ▸ Layers | — | `panel_settings.dart:81-99`; `session_files.dart:128-132` | `storeDesk('flatProjection')` → `command('preferences',{flatProjection})` (`port.rs:221`) | PRESERVE | |
| ST8-04 | Per-panel placement: Desk / Tab / Window / Hidden for all 16 catalog panels | Settings ▸ Panels | Desk only for drawer panels | `panel_settings.dart:101-163` | `c.placePanel(name, place)` | PRESERVE (Classic) / MOVE (New) | Classic-vocabulary control (WS-10) |
| ST8-05 | Colour theme: current name, Load JSON… (≤128 KB), Reload, Copy JSON, Default; notices | Settings ▸ Color theme | — | `app/theme_settings.dart:11-144`; applied `editor_window.dart:190-202` | `pickImport(json)`, `storeDesk('theme')`, `setPaneState` broadcast; Flutter `Clipboard` for Copy | PRESERVE | Theme JSON is the Classic token set; New UI tokens should be able to load the same file or a sibling |
| ST8-06 | "Outside dim" −/+ (5 % steps) | Settings ▸ View | — | `editor_window.dart:84`, `:589-608`, `:211` | Held in `_EditorWindowState.dim`; written in `persist()` but **never restored, never read by Stage, and ± does not call persist** | PRESERVE (flag as inert) | Dead control today; recording so the capability oracle stays honest |
| ST8-07 | UI scale −/+ and percent field (50–200 %) | Settings ▸ View | — | `editor_window.dart:609-641`; `editor_app.dart:53-62` | `EditorScale` notifier + `persist()` | PRESERVE | Whole-app scale; shared by both shells |
| ST8-08 | Other persisted per-user prefs written by panels into `deskWork` (inspectorCell, browserRail, browserView, browserWheel, swatches, tags, collections, collectionNames, ranges, labels, folds, webUrl, curveClip, easePresets, newKeyShape) | Various panels | — | `storeDesk` callers: `inspector.dart:375`, `browser.dart:431-449`, `browser/filter_library.dart:60-150`, `browser/colors_shelf.dart:227-310`, `browser/parts.dart:105`, `web_panel.dart:22`, `ease_desk.dart:326-351` | `storeDesk` read-merge-write of layout.json | PRESERVE | Listed for ownership; these belong to panels and survive a shell swap because they live in the session |

## 2. Ownership map

| Layer | Owns | Readers | Writers |
|---|---|---|---|
| Rust `EditorRuntime` (`native/src/lib.rs:25-58`), one per process, held by Swift `ProbeSession.shared.runtime` | Document (`doc`), undo/redo stack (Document edit head), history ledger (`editor/history.rs`, persisted to `~/Library/Application Support/MotoliiStage5/history.jsonl`, 200 entries), **selection** (`viewer.selected_ids`, `selected_keys`, `color_target` — `viewer.rs:30-48`), **playhead** (`viewer.frame`, `viewer.clock`), playing state, Animate mode (`viewer.animate`), **User-stage camera** (`user_camera` orbit/fit/focus), stage window size/roi (`stage_window` — one), stage snap/pointer/view scale, preview (transient edits), clipboard (process-local typed, `editor/clipboard.rs`), export and freeze jobs, flat-projection preference (pushed from Dart), path/dirty | Status JSON built in `snapshot.rs:330` | JSON ops through `motolii_probe_request` (`lib.rs:274`) → `port.rs:121 request` |
| Swift host (`macos/Runner/MainFlutterWindow.swift`) | Playback cadence timer, IOSurfaces (2 per view, shared by all windows), Flutter textures (1 per view **per window**), attachments (1 per window), windows registry, `paneState` cache, settings file IO (`layout.json`), last status cache | — | MethodChannel `motolii/probe` handler; broadcasts `documentChanged`/`playbackFrame`/`paneState` to every window |
| `bridge/protocol.dart`, `bridge/native_bridge.dart` | Wire vocabulary (`DocumentOperation` enum, `requiresPause`, `requiresRender`); the single MethodChannel and **static** listener owner (`native_bridge.dart:7`) | — | `invoke`, `request` |
| `session/*` (`EditorSession` = SessionCore + mixins) | Mirror of Rust status (`document` ValueNotifier + named `slice`s), `frame`, `rendered`, `playing`, `textureIds`, `error`, `busy`, command serial queue, playback generation. UI-local cross-panel state: `editingFocus`, `focusProperty`, `textStyleTarget`, `anchorPreview`, `keyedOnly`, `viewCommand`, `eyedropper`, `browserTab`, `importedAssets`, `dragging`, `visibleFrames`, `previousKeys`. Per-user prefs: `deskWork` (persisted), `panePlaces`, `deskDefault`, `deskDrawer`. Host callback slots: `confirmClose`, `windowClosed`, `filesDropped`, `fileDropTarget`, `panelPlacementRequested` | Panels read `c.state` / `c.slice(name, keys)` / `c.liveLayers()` / `c.activeLayer` / `c.selectedIds` (read model = Rust snapshot, not Dart objects); `session/read_model.dart` helpers | `c.command(op,args)` (document intents), `c.native(method)` (host), `c.storeDesk` (prefs), ValueNotifier writes for UI-local focus |
| `workspace/*` | Dock tree (`DockNode`: splits, ratios, offsets, tabs, active tab), focused panel ring | `WorkspaceView` | `WorkspaceLayout.show/close/move`, divider drag |
| `app/editor_window.dart` (`_EditorWindowState`) | **Creates and disposes the EditorSession** (`:61`, `:184`), menu bar, sheets (`sheet`), status line, `detached` windows map, `hiddenPanels`, `dim`, UI scale persistence, the whole placement algorithm `_placePanel`, settings restore/persist | — | — |
| `app/editor_app.dart` | Theme (`appearance`) and UI scale notifiers, WidgetsApp root | — | — |
| `panels/*` | Widget-local drafts (text being typed, drag gestures, Stage 2D zoom/pan `_zoom/_pan` in `stage/view.dart`, Timeline lane expansion `expanded/allProperties`, Browser selection/search), preview flights | session | session |
| `foundation/*` | Visual tokens (`EditorTheme`, `EditorMetrics`), controls, panel catalog metadata; static `EditorTheme.animating` (`foundation/theme.dart:337`) | — | — |

Direct answers:

- **Selection**: owned in Rust (`viewer.rs:31-32`, `lib.rs:220` uses it for the outline; `port.rs` `select` op). Dart only mirrors `selectedIds`/`selectedKeys` from the snapshot and remembers `previousKeys` (`session_snapshot.dart:34-42`) for Cmd+Shift+D. Browser shelf item selection and Timeline row UI state are widget-local, not document selection.
- **Undo/redo**: Rust (`port.rs:247-257`, Document history; ledger in `editor/history.rs`). Dart holds no stack; the History panel reads `state.undo/redo` and sends `historyGoto`.
- **Playhead**: Rust (`viewer.frame` + `Clock`); playback cadence in Swift (16 ms `DispatchSourceTimer`); Dart mirrors `c.frame` from `playbackFrame` and replies.
- **Viewport/camera**: 3D/User stage camera in Rust (`user_camera`, `stageView` op); Stage 2D pan/zoom is widget-local in `panels/stage/view.dart`; the Stage's pixel window/roi is pushed to Rust as `stageWindow` (one per runtime).
- **Panel layout persistence**: `layout.json` in `~/Library/Application Support/MotoliiStage5/`, written by Swift on `writeSettings` with the exact map Dart passes (whole-file replace). Keys: `dock`, `scale`, `dim`, `deskDefault`, `deskWork`, `panelPlacements` (`editor_window.dart:208-215`). `storeDesk` does read→merge→write for `deskWork` only (`session_files.dart:134-140`).

Domain / product logic currently in Dart (a second shell would have to share, not copy):

1. `input/editor_shortcuts.dart:188-207` — keyboard nudge synthesizes a 3-step `stageGesture` from the first cage corner. Rust has no "nudge" op.
2. `input/editor_shortcuts.dart:141-156` — previous/next layer selection order computed from the `layers` list.
3. `input/editor_shortcuts.dart:75-78` — select-all enumerates layer ids in Dart.
4. `input/editor_shortcuts.dart:124-127` — End = `durationFrames-1`.
5. `input/editor_shortcuts.dart:48-60` — F9 variant mapping + focus-keyframes follow-up.
6. `app/editor_window.dart:379-432` — menu action → op mapping, `confirmReplacement` ordering (stop playback → dirty check → dialog → save) at `:347-377`. Lives inside the Classic widget state.
7. `app/editor_window.dart:23-38` — freeze notice text/progress derivation.
8. `app/editor_window.dart:273-328` — the entire panel placement state machine (tab/window/drawer/hidden) is a widget method.
9. `panels/export_controls.dart:32-46` — marker-to-marker export range derived from markers and playhead; `:93-100` export progress polling owned by the sheet widget.
10. `panels/composition_controls.dart:35-40`, `~:110-120` — composition size and fps presets hard-coded in the widget.
11. `panels/timeline/grip.dart:400-435` — trim-in / trim-out / slip / move timing arithmetic (`start`, `duration`, `sourceIn`) computed in Dart before `setTiming(s)`/`previewTimings`.
12. `panels/desk.dart:48-56` — selection → drawer rule (keys or multi-layer → Ease; Camera → Depth).
13. `session/editor_session.dart:36-42` + `session_core.dart:25-28` — default new-key shape (Easy Ease bezier) lives in Dart.
14. `session/session_commands.dart:65-69` — `visibleFrames` injected into `create`/`placeAsset` (default duration input).
15. `session/session_files.dart:94-120` — import extension gate duplicated in Dart (Rust also gates).
(1–5, 13–15 are in shared non-widget classes, so reusable; 6–12 are in widgets and would be duplicated by a second shell unless extracted.)

Findings worth surfacing:
- `setMatte` is sent by `panels/inspector/content_cards.dart:112,133` but is **not** in `DocumentOperation`, not in Rust `CAPABILITIES` (`port.rs:8`), and has no Rust handler — the control is permanently disabled (gated by `panelCan`).
- `U` (keyedOnly) and "Outside dim" are inert (KB-23, ST8-06).
- `setMarker`, `deleteMarker`: Rust handles them, no Dart sender — markers can be added (M, Timeline "Marker" button) but not moved or deleted from the UI.

## 3. Minimal boundary for Classic + New

Root today: `main.dart:5 runApp(EditorApp)` → `EditorApp` (theme + scale) → `WidgetsApp(home: const EditorWindow())` (`app/editor_app.dart:65`). `EditorWindow` both **creates the session** and **is the Classic layout**. There is no layout choice point.

Is `EditorSession` shareable by two shells? Yes as an object (plain ValueNotifiers, named slices, serial command queue), and it **must** be one instance per Flutter engine:
- `NativeBridge` keeps a **static** `_listenerOwner` and there is one `MethodChannel('motolii/probe')` handler per engine (`bridge/native_bridge.dart:6-29`). A second session's `listen` replaces the first; the first stops receiving `documentChanged`, `playbackFrame`, `confirmClose`, `filesDropped`.
- Swift `ProbeHost` has **one** `WindowAttachment` per engine (`WindowAttachment.swift`; handler `attach` in `MainFlutterWindow.swift`). A second `attach` without `attachmentId` bumps the generation and makes the first session "Stale UI attachment".
- Single-slot callbacks on the session: `confirmClose`, `windowClosed`, `filesDropped`, `fileDropTarget`, `panelPlacementRequested` (`session_core.dart:41-44`, `:75`). Whoever is mounted must install them.

Native assumptions:
- One `EditorRuntime` per process (`ProbeSession.shared.runtime`), one `ViewerState` → one selection, one playhead, one User camera, **one `stage_window`** (`viewer.rs:44`), one `stage_view` (which view owns the gizmo).
- Exactly two views, `User` and `Camera` (`viewer.rs:8-27`). Surfaces: 2 IOSurfaces per view shared by all windows (`ProbeSession.surfaces`). Textures: 1 Flutter texture per view per window (`ProbeHost.textures`). `frameReady` publishes only to main hosts; others get pictures via `broadcast`.
- Multiple windows are already supported (detached panel windows = extra engines with `main:false`), so "one window" is not assumed. "One Stage size" is assumed.

Smallest seam:
1. Add a `SessionHost` StatefulWidget as `home:` in `editor_app.dart:65`. It owns `final c = EditorSession()`, calls `c.initialize()`, and disposes it (moves `editor_window.dart:61`, `:127`, `:184` out of the shell). It installs the shell-agnostic host callbacks (`filesDropped → importPaths`, `windowClosed` bookkeeping).
2. `ClassicShell` = today's `EditorWindow` taking `EditorSession c` as a constructor argument; everything else stays byte-for-byte (dock, menus, sheets, settings, `_placePanel`).
3. Shell choice: `const String.fromEnvironment('MOTOLII_SHELL')` (same mechanism as `MOTOLII_DOCUMENT`, `session_files.dart:6`), default `classic`; optionally a persisted `shell` key plus a View-menu item "New UI / Classic UI" that swaps the `SessionHost` child. Only **one shell mounted at a time**; the session survives the swap.
4. Extract the domain-ish widget code both shells need into non-widget helpers without changing behaviour: `EditorActions` (menu ops + `confirmReplacement` flow, currently `editor_window.dart:347-432`) and the placement state machine (`:240-345`) or an adapter so New shell answers `placePanel` requests. `EditorShortcuts` is already a separate class parameterised by shell callbacks and can be reused directly.
5. New shell hosts existing panels via `buildPanel(name, c, key)` (`panels/registry.dart:20`) with its own GlobalKey map.
6. Detached panel windows (`windowInfo.main == false`) keep using Classic's detached leaf regardless of shell choice (lowest risk).

Code that moves vs stays:
- Moves (to SessionHost / shared helpers): session construction/initialize/dispose; `filesDropped` wiring; menu action dispatch and confirm-save flow; placement state machine (or a thin interface over it); settings restore of session-level prefs (`restoreDeskWork`, `deskDefault`, `scale`).
- Stays in Classic: menu bar widgets, sheets, status line widget, `WorkspaceView`, `DockNode`, `initialDock`, Classic's layout persistence keys, Settings sheet contents.

Risks:
- **Settings file whole-overwrite**: `persist()` writes a fixed key set (`editor_window.dart:208-215`) and Swift writes it verbatim, so any New-shell key (e.g. `newDock`) is dropped on the next Classic persist, and vice versa. Either store New layout inside `deskWork` via `storeDesk` (read-merge-write) or make both persists merge. Also `persist()` and `storeDesk` race (timer vs chained future).
- **`stageWindow` single slot**: Stage dispose sends `stageWindow {0,0}` (`panels/stage.dart:75-77`, `panels/stage/window.dart:46-50`). If both shells mount a User Stage (or the old one disposes after the new one mounts during a swap), the survivor's picture blanks until it re-sends. Two Stages of different sizes would fight. Keep one User Stage mounted.
- **`_shownViews` / `attachView`** are a set, not ref-counted (`session_render.dart:22-31`): one Stage detaching removes the view for another showing the same view.
- **GlobalKeys**: `paneKeys` (`editor_window.dart:82`) — mounting the same key in two shells at once throws; New shell must not reuse Classic's map, and both shells must not be mounted together.
- **Statics**: `NativeBridge._listenerOwner`; `EditorTheme.animating` (`foundation/theme.dart:337`, written by `_syncAnimationAppearance` `editor_window.dart:188`); measurement caches (`workspace_view.dart:177`, `browser/parts.dart:189`) are harmless.
- **`panePlaces` vocabulary** (tab/window/drawer/hidden) is read by Settings ▸ Panels, the Desk, and launch restore; New shell must publish something coherent or Classic Settings will lie while New is mounted.
- **Host routing to "main"**: `placePanel`, `effectsChanged`, `confirmClose` go to the first `isMain` host (Swift). Only the main window's mounted shell answers; that is fine with one shell at a time.
- **Esc ladder** and `showComposition`/`showInspector` are shell callbacks; New shell must supply its own equivalents.
- **Export progress polling** dies with the sheet widget; a New export surface must own polling or it inherits the same gap.

## Appendix — bridge command surface (Domain Intent + host)

### A. Document ops: Dart `c.command(op)` → `native('request', {command: {"op":…}})` → Swift `request` → Rust `motolii_probe_request` (`lib.rs:274`) → `EditorRuntime::request` (`port.rs:121`)

Rust handler lines are in `native/src/port.rs` unless noted. "Dart sender" lists representative call sites.

| op | Rust handler | Dart sender(s) | Note |
|---|---|---|---|
| status | `port.rs:129` (early return) | none via `command`; Swift calls `runtime.status()` internally | Rust-only / host-only |
| exportStatus | `port.rs:129` | `panels/export_controls.dart:94` | |
| reloadEffects | `port.rs:129` | `session_native.dart:174`; `browser/effects_shelf.dart:65` | |
| stageView | `port.rs:130-148` | `stage/view.dart:99`; `stage/camera.dart:17`; `stage/chrome.dart:120` | orbit/focus/reset/fit |
| stageWindow | `lib.rs:305` | `stage.dart:77`; `stage/window.dart:50,138` | single slot |
| renderInfo | `lib.rs:295` | none (Swift `render` only) | host-only |
| visualSample | `lib.rs:309` | `panels/native_visual_sample.dart:68` (raw `native('request')`) | bypasses `DocumentOperation` |
| fontFacts | `lib.rs:314` | `browser/fonts_shelf.dart:32` (raw `native('request')`) | bypasses enum |
| easeModel | `lib.rs:318` | `native('easeModel')` (ease desk) → Swift wraps as op | host method |
| tick | `port.rs:151` | none (Swift `render` sends quiet tick) | host-only |
| notes | `port.rs:161` | `notes_desk.dart:58,509,621` | |
| select | `port.rs:162` | `editor_shortcuts.dart:41,76,152`; timeline, stage, … (16 sites) | |
| setProperty | `port.rs:170` | `inspector/wells.dart:341`; `browser/fonts_shelf.dart:387` | |
| previewProperties | `port.rs:171` | `depth_desk.dart:38`; `gradient_inspector.dart:42`; `inspector/writing.dart:92` | |
| commitPreview / commitX | `port.rs:172` | commitPreview: 7 sites; **commitX: no Dart sender** | |
| cancelPreview | `port.rs:173` | `session_commands.dart:111`; 7 sites | |
| setX | `port.rs:174` | **no Dart sender**, not in `DocumentOperation` | Rust-only (legacy X path) |
| previewX | `port.rs:175` | **no Dart sender**, not in enum | Rust-only |
| previewText | `port.rs:176` | `rich_text_editor.dart:149` | |
| styleText | `port.rs:177` | `browser/fonts_shelf.dart:336` | |
| setFont | `port.rs:179` | `browser/fonts_shelf.dart:298` | |
| setText | `port.rs:180` | `rich_text_editor.dart:256` | |
| setAttrs | `port.rs:181` | `blend_panel.dart:225`; `inspector/transform_card.dart:179,202`; `timeline/grip.dart:161` | |
| create | `port.rs:189` | `browser/create_shelf.dart:124`; `fonts_shelf.dart:294`; `media_shelf.dart:131`; `stage/camera.dart:38` | |
| copy / cut | `port.rs:190` | menu, shortcuts, timeline menu | |
| paste | `port.rs:194` | same | |
| duplicate | `port.rs:195` | same | |
| delete | `port.rs:199` | same | |
| group | `port.rs:200` | same | |
| ungroup | `port.rs:201` | same | |
| moveLayers | `port.rs:202` | `timeline/grip.dart:443` | |
| reorder | `port.rs:203` | **no Dart sender** (in enum and CAPABILITIES) | Rust-only |
| split | `port.rs:204` | menu, Cmd+K, timeline menu | |
| setTimings / previewTimings | `port.rs:205` | `timeline/grip.dart:49` / `:34` | |
| stageGesture | `port.rs:206` (phases hover/configure/begin/update/commit/cancel; hover reply `lib.rs:324-329`) | `editor_shortcuts.dart:194-206`; `stage/touch.dart` | |
| setTiming | `port.rs:207` | `timeline/grip.dart:51` | |
| toggleKey | `port.rs:208` | `inspector/wells.dart:100,373`; `timeline/grip.dart:169` | |
| moveKeys | `port.rs:215` | `editor_shortcuts.dart:134`; `timeline/view.dart:35`; `timeline/grip.dart:458` | |
| ease | `port.rs:216` | `editor_shortcuts.dart:49`; `ease_desk/state.dart:369` | |
| setColor / previewColor | `port.rs:217` | `browser/color_picker.dart:149,263` / `:49` | |
| focusColor | `port.rs:218` | `session_commands.dart:38` | |
| applyPalette | `port.rs:219` | `browser/colors_shelf.dart:150`; `stage/touch.dart:222` | |
| applyEffect | `port.rs:220` | `browser/effects_shelf.dart:203` | |
| preferences | `port.rs:221` | `session_files.dart:131` | |
| animate | `port.rs:222` | `session_commands.dart:16` | |
| expandEffect | `port.rs:223` | `inspector/effects_card.dart:66` | |
| scopeEffect | `port.rs:224` | **no Dart sender** (in enum and CAPABILITIES) | Rust-only |
| enableEffect | `port.rs:225` | `inspector/effects_card.dart:179` | |
| moveEffect | `port.rs:226` | `inspector.dart:355`; `inspector/effects_card.dart:56` | |
| removeEffect | `port.rs:227` | `inspector/effects_card.dart:71` | |
| ghost | `port.rs:228` | `inspector/transform_card.dart:251` | |
| sequence / previewSequence | `port.rs:229` | `ease_desk/state.dart:185` / `:205` | |
| clip | `port.rs:230` | `inspector/transform_card.dart:264`; `timeline/grip.dart:159` | |
| previewBlend | `port.rs:231` | `blend_panel.dart:165` | |
| addMarker | `port.rs:232` | `editor_shortcuts.dart:158`; `timeline.dart:376` | |
| setMarker | `port.rs:232` | **no Dart sender** | Rust-only — marker move unreachable from UI |
| deleteMarker | `port.rs:232` | **no Dart sender** | Rust-only — marker delete unreachable from UI |
| composition | `port.rs:233` | `composition_controls.dart:42,61,83,117`; `stage/chrome.dart:29` | |
| import | `port.rs:234` | `session_files.dart:113` | |
| placeAsset / replaceAsset | `port.rs:235` | `media_shelf.dart:133`; `timeline/grip.dart:384` / `media_shelf.dart:222` | |
| relinkAsset | `port.rs:236` | `media_shelf.dart:328` | |
| removeAsset | `port.rs:237` | `media_shelf.dart:234,241` | |
| export | `port.rs:238` | `export_controls.dart:91` | |
| cancelExport | `port.rs:239` | `export_controls.dart:77` | |
| runScript | `port.rs:240` | `session_files.dart:47,79` | |
| rerunScript | `port.rs:241` | `session_files.dart:83` | |
| save | `port.rs:242` | `session_files.dart:70` | `copy:true` variant has no Dart sender |
| new | `port.rs:246` | `editor_window.dart:383` | |
| undo / redo | `port.rs:247-248` | menu, Cmd+Z / Cmd+Shift+Z | |
| historyGoto | `port.rs:250` | `history_records.dart:116` | |
| play / pause | `port.rs:257-258` | `session_render.dart:52,78` (`_request(DocumentOperation.play/pause)`) | |
| pickColor | `port.rs:259` | `stage/touch.dart:219` | |
| seek | `port.rs:260` | `session_commands.dart:108`; `timeline/frame.dart:50`; Swift quiet seek | |
| anchor | `port.rs:261` | `inspector/transform_card.dart:97` | |
| freeze | `port.rs:262` | `timeline/menu.dart:101` | |
| setGradient | `port.rs:279` | `gradient_inspector.dart:93`; `browser/fill_definitions.dart:121,144` | |
| setFillMode | `port.rs:280` | `browser/fill_definitions.dart:117` | |
| **setMatte** | **none** (falls to "Unsupported operation") | `inspector/content_cards.dart:112,133` | Dart-only; not in `DocumentOperation` (would throw) nor CAPABILITIES → control always disabled |

### B. Host methods: Dart `c.native(method)` → Swift `ProbeHost.handle` (`MainFlutterWindow.swift`)

| method | Swift behaviour | Dart sender |
|---|---|---|
| windowInfo | id/panels/main/paneState/panelWindows | `session_files.dart:10` |
| attach (+view) | status + attachment id + texture for view | `session_files.dart:20,23`; `session_render.dart:27` |
| detach | release attachment + textures | `editor_session.dart:121`; `session_native.dart:20` (via `_bridge.invoke`) |
| open | open runtime, bump epoch, broadcast | `session_files.dart:57` |
| request | forward op JSON to Rust, start/stop playback timer, broadcast | `native_bridge.dart:19`; raw at `fonts_shelf.dart:32`, `native_visual_sample.dart:68` |
| render | quiet tick/seek + renderInfo + ensureSurfaces + renderInto + broadcast | `session_render.dart:9` |
| easeModel | wraps as op `easeModel` | ease desk |
| readSettings / writeSettings | layout.json IO (write = whole file) | `editor_window.dart:131,139,208`; `session_files.dart:137-138` |
| setPaneState | cache + broadcast `paneState` | `editor_window.dart:255`; `theme_settings.dart:25` |
| placePanel | forwarded to main window's Dart | `session_files.dart:143` |
| flushEditors | asks every window to flush | `editor_window.dart:311`; `session_commands.dart:60` |
| openPanelWindow | new NSWindow + engine | `editor_window.dart:335` |
| focusWindow / closeWindow | window ops | `editor_window.dart:226,267,285,296,314` |
| pickOpen / pickImport / pickSave / pickExport | NSOpen/SavePanel | `session_files.dart:63,75,86,69`; `theme_settings.dart:47`; `export_controls.dart:89` |
| reveal / openFile | Finder / default app | `browser/files_shelf.dart:151,153` |
| openWeb | open http(s) URL in browser | `web_panel.dart:43` |
| noteClipboard | read system pasteboard (png or text) | `notes_desk.dart:117` |
| ensureSurfaces | allocate IOSurfaces | **no Dart sender** (Swift `render` calls it internally) |
| broadcast | re-broadcast a status | **no Dart sender** |
| close | close runtime, notify `documentClosed` | **no Dart sender** |

### C. Host → Dart callbacks (`session_native.dart:123-198`)

`confirmClose`, `flushEditors`, `placePanel`, `paneState`, `documentChanged`, `playbackFrame`, `effectsChanged`, `documentClosed`, `windowClosed`, `dragHover`, `filesDropped`. Swift also sends `nativeError`, which **no Dart handler consumes** (falls through, returns null).
