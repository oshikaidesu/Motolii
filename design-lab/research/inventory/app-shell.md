> **Use of this file (owner, 2026-10-02): COUNTER-EXAMPLE ONLY.** The old Motolii is not a reference for the new UI or UX. This file is a list of what exists (a capability checklist) and of what NOT to import. Nothing here is a spec.

# App shell and global chrome inventory

Source: `/private/tmp/wt/tok/motolii/ui/lib` (branch dev/ui-inspect). Paths below are relative to that `lib/`. Read-only; everything is from code read, uncertain items marked `[uncertain]`.

Headline finding: in this tree there is NO File/Edit/View menu bar, NO Settings sheet (tile size, theme JSON, panel placement table, outside dim do not exist here), NO Composition or Export button label outside the EDIT/PLAY/EXPORT track, and NO status bar/toast. Those were Classic features (docs inventory `shell.md` SH-19..SH-24, ST8-*). The hf shell exposes: top bar, tab-menu popup, Export / Composition / Interface popovers, one Save-changes dialog, Console panel, UI Scale readout. Everything else is keyboard.

Story coverage: explorer stories are `Top bar` (otherStories, `SessionTop`) and `Workspace 90% / 100% / 115%` (`WorkspaceFace` = SessionTop + LiveWorkspace). No story exists for any popover, dialog, menu, Console, readout or Design Mode.

## 1. Window root

1. **App root (WidgetsApp)** · `main.dart:20` `_window()` — bare `WidgetsApp` (no Material), base colour `Surface.base`, text style Inter 12, wrapped by `liveEditorTheme.wrap`, then `UiScaleScope` + `UiScaleReadout` in a Stack. States: debug build wraps in `DesignMode`; release does not. Story: none directly (Workspace stories render the inside).
2. **Live shell (window)** · `app/window.dart:26` `LiveShell` — Column of `SessionTop` over the dock workspace, inside a `Focus` that routes keys to `LiveKeys`. States: not-ready (empty `Surface.base` box, no spinner/text); ready; detached-panel window (`windowInfo['main']==false` shows only that one panel full-window on base); `MOTOLII_SHOT` screenshot capture mode (writes PNG after 3 s, no visible UI). Story: Workspace stories (without the key Focus).
3. **Empty/loading/error states of the whole window** — loading = blank `Surface.base` (`window.dart:152`); init errors are swallowed (`catch (_) {}` around settings read, relink); runtime errors go to `c.error` and show only in the Console panel. No splash, no error screen, no missing-document screen [verified by reading; `c.initialize` internals not read].
4. **Files dropped on window** · `window.dart:61` — dropping files calls `importPaths`; no visible drop affordance in this file. Unsupported names produce an error string "Not supported: names" (`session/session_files.dart:118`) which lands in Console.

## 2. Top bar

All in `app/top/top_bar.dart` (`TopBar`, production face, flexible) over `TopModel` (`app/top/top.dart:14`), fed by `SessionTop` (`app/top/top_session.dart:12`). Height `Surface.topBar` (32 at 100 %), bottom hairline. Story for every entry below: **Top bar** (and Workspace stories). `app/top/top.dart:48` `top()` is the 1536-px fixed-coordinate reference face (RI list), not used by production; `app/top/ruler.dart` is its drawing primitives (Rc, Tx, Hg, Ln, Wd, RF, Pt).

5. **Brand wordmark "Motolii"** · `top_bar.dart:31` — static text. Hidden never (always shown).
6. **Tagline "Motion / for More Relations."** · `top_bar.dart:32` — static two-line label; folds away below 820 px width.
7. **Play key** · `top_bar.dart:41` `_Key` + `_Play` — green (H.play) square with triangle; click = `c.togglePlayback`. Quiet cursor if no handler.
8. **Stop key** · `top_bar.dart:42` — grey square; click stops playback and seeks to frame 0 (`top_session.dart:75`).
9. **Animate (record) key** · `top_bar.dart:43` — pink dot; `c.toggleAnimate`; null (quiet) when host lacks `animate`. Same toggle as key A.
10. **Readouts (fps · duration · clock)** · `top_bar.dart:45` — three monospace values: fps with 2 decimals, duration in frames, playhead clock `mm:ss:ff`. Duration (with its divider) folds below 680 px. Read-only, not an editable timecode field.
11. **Add-marker glyph key** · `top_bar.dart:57` `_Glyph(HG.plus)` — adds a marker at the playhead (`addMarker`); quiet if unsupported. Same as key M.
12. **Mode track EDIT · PLAY · EXPORT** · `top_bar.dart:151` `_Modes` — one segmented track; lit segment is H.mode. EDIT/PLAY set playback off/on; EXPORT opens the Export popover anchored under its own key and stays lit while the popover is open (`exporting` flag, `top_session.dart:63`). States: edit / play / export.
13. **Fit key** · `top_bar.dart:61` (HG.fit) — `stageView('Fit')`; quiet if host lacks `stageView`. Same as Cmd+0.
14. **Pin key** · `top_bar.dart:61` (HG.pin) — shown, never wired (`top_session.dart:82` comment "Pin has no operation"); drawn quiet.
15. **Open key (folder glyph)** · `top_bar.dart:61` — runs the unsaved-changes guard then the native open panel (`c.chooseOpen`).
16. **UI key ("UI")** · `top_bar.dart:103` `_TextKey` — text key that opens the Interface popover under it; present only when `m.onScale != null`.
17. **Motto "Less numbers. / More motion."** · `top_bar.dart:66` — static two-line mono label right end; first thing to fold (below 1000 px).
18. **Responsive folding** — motto at <1000, tagline at <820, duration readout at <680 (`top_bar.dart:25`).

Not present in the bar: document name / dirty marker, fps editor, Composition button, Export button (EXPORT segment is the entry), Settings button, File menu. Save is key-only (Cmd+S).

## 3. Menus

19. **Panel (tab) context menu** · `workspace/dock_workspace.dart:215` `_menu` via `showHfMenu` (`controls/menu.dart:14`) — right-click a seat tab. Lines: Detach (only if host provides onDetach), Close (only for closable panels, i.e. not the Home set), `Open <Panel>` for every panel not currently open, Reset Layout. Anchored at cursor; keys: Up/Down walk, Enter picks, Esc/outside press closes. Menu widget supports shortcuts column, dividers, info lines, check on `selected`, disabled lines, upward flip, internal scroll (capabilities, not all used here). Story: none.
20. **Other showHfMenu users (outside this area, listed for the cross-ref)**: `inspector/key_menu.dart`, `browser/browser.dart`, `browser/media/media_browser.dart`, `effects/card.dart`, `timeline/timeline.dart`, `desks/notes/face.dart`.
21. **Native macOS menu bar** — not in Dart in this tree (Classic docs mention Swift host menu, SH-35); no File/Edit/View Flutter menu exists here. [uncertain whether the Swift runner still adds one; not in lib/]
22. **Tab "Panel" ellipsis menu (docking own strip)** · `dock_workspace.dart:134` — only when `seats` is false (the legacy "New shell"); hf workspace uses `seats: true` so it is not shown. Items Detach / Close.

## 4. Sheets and popovers

All use `showHfPopover` (`controls/sheet.dart:148`): compact box anchored under its asker with right edge on the control's, no dimmed backdrop, title row (`Surface.chromeRow`) + close X, Esc/outside press closes, Enter runs the primary action. Key `ValueKey('hf-popover')`. Story: none (no popover story).

23. **Popover housing** · `controls/sheet.dart:148` — shared frame, shadow, focus capture.
24. **Export popover** · `app/sheets.dart:21` `showExportSheet`, body `_Export` `:34` — width 240. Rows: Output (fact: `W × H · fps · MP4`, fixed MP4 label), Range (Whole | Markers; Markers disabled when no markers; disabled while running), Frames (fact `start – end · seconds s`), Status row (only when job phase not idle) with text: Writing done/total, Stopping…, Written <path>, Failed: <error>, else raw phase (failed in H.record red). Buttons: Cancel (closes) / Stop (while running, only if host supports `cancelExport`), primary Export… (opens native save panel `Untitled.mp4`, then job; disabled while running or if host lacks `export`). Progress is polled every 300 ms by `ExportSession` (`:44`) and survives closing the popover. No progress bar, only text. Errors: "Choose a non-empty range" to Console.
25. **Composition popover** · `app/sheets.dart:141` `showCompositionSheet`, store `CompositionStore` `:183` — width 225, body is an Inspector `ParamSheet` with rows: Size (enum 16:9/9:16/1:1/4K), Width (u32 16–8192 px), Height, Frame rate (enum 23.976/24/25/29.970/30/50/59.940/60), Length (frames), Background (enum Black/Dark/Grey/White), Background colour (colour row that routes to the Colors panel and calls `focusColor` slot Background). Each change commits through `composition` command. Opened ONLY by Cmd+Option+K (`window_keys.dart:79`); no button opens it in this top bar.
26. **Interface (UI Scale) popover** · `app/sheets.dart:151` `showUiScaleSheet` — title "Interface", width 225. Row "UI Scale": `−` action, `EditorPercentField` (scrub or type, 50–200 %, key `ui-scale-field`), `+`; plus `Reset 100%` (disabled at 100). Opened by the top-bar UI key or Cmd+, .
27. **Form primitives used by popovers** · `controls/sheet.dart`: `HfAction` `:16` (secondary / primary (mode colour) / destructive (record red); `chosen`; quiet when no onTap; hover lift), `HfChoice<T>` `:70` (segmented, per-segment enabled), `HfFormRow` `:121` (label 64 wide + child), `HfFact` `:137` (read-only monospace value, no box). Plus `EditorPercentField` `controls/panel/scale.dart` (not read in depth).

## 5. Dialogs

28. **Question dialog** · `controls/question_dialog.dart:13` `showHfDialog` — centred 285-wide box over a `shade55` scrim; title, one body line, answer keys; destructive answers drawn red and set to the left, last answer is primary and is Enter's; Esc or scrim press answers null. Story: none.
29. **"Save changes?" guard** · `app/document_guard.dart:8` `mayReplace` — body "Save the current document before closing it."; answers Cancel / Don't Save (destructive) / Save (primary). Shown only if `state['dirty']`; stops playback first. Used by Open key, Cmd+N, Cmd+O, and the host's window-close (`c.confirmClose`, `window.dart:50`). After Save it proceeds only if the document is no longer dirty.
30. **Native file panels** · invoked from `session/session_files.dart` and `export_actions.dart` through `native('pickOpen' | 'pickSave' | 'pickImport' | 'pickExport')` — OS open/save/import panels (not Flutter). Save as default name `Untitled.rrd`; export `Untitled.mp4`; import extensions from host `importExtensions`; script `.js` import (`runScript`, no visible entry in this area).

## 6. Status, notices, errors, progress

31. **No status bar / toast / banner** in this window. All messages funnel to the Console (next section).
32. **Notice sources (text only)** · `session/status_notice.dart` — `freezeNotice`: "Freezing <layer> done/total" or "Freeze failed: <error>"; `relinkNotice`: "Found N moved file(s) through the catalog; N files have several possible matches; N files are on a source that is not connected"; effects catalogue notice (`effects/` not read). Combined in `window.dart:44`.
33. **Operation errors** — `c.error` string (e.g. "Not supported: a.xyz", "No colours found in that image", "Choose a non-empty range", native/command exceptions) are logged by `ConsoleLog` and nowhere else.
34. **Export progress** — text row inside the Export popover only (item 24); the top bar's EXPORT segment stays lit only while the popover is open, not while a background job runs [uncertain: no job indicator elsewhere found].
35. **UI Scale readout (HUD)** · `app/ui_scale.dart:124` `UiScaleReadout` — a small pill "UI 120%" top-centre (alignment 0,-.82), monospace, `veilHi` ground, fades in 120 ms and out after 1.1 s; ignores pointer; key `ui-scale-readout`. Appears on every scale change (keys, popover, field). Story: none.

## 7. Console

36. **Console panel** · `app/console.dart:10` `LiveConsole` — header row "N message(s)" (singular/plural) + `Clear` text key (key `console-clear`, greyed when empty); list newest first, each line = `hh:mm:ss` mono clock, a 4.5-px square (error = H.scatter.n pink, notice = H.follow.b orange), message text. States: empty ("No messages"), populated. Backed by `ConsoleLog` (`session/console_log.dart:15`) kept by the shell so closing the panel loses nothing. Lives as tab in the Timeline seat. Story: appears inside Workspace stories only if the Console tab is picked (default front tab is Timeline) [uncertain].

## 8. Docking / workspace

`workspace/seats.dart`, `workspace/dock_workspace.dart`, `workspace/dock_theme.dart`; docking engine = the `docking` package with its own tab strip switched off.

37. **Default layout (Product Home)** · `seats.dart:88` — top row (68 %): Browser seat [Create, Effects, Colors, Fonts, Media] 22 %, Stage seat [Stage, Camera] 50 %, Inspector alone 28 %; bottom row (32 %): [Timeline, Graph, Console] 81 %, Desk seat [Ease, Depth, Blend, History, Notes] 19 %. Opened on demand: Relations, Web. `[Graph]` body is an empty base-coloured box (placeholder, "not built yet" comment).
38. **Panel registry** · `seats.dart:27` `LiveWorkspace`, `dock_workspace.dart:15` `PanelDef` — 17 panels with id, title, family glyph, min size (180–320), optional trailing tools (only Timeline: `LiveTimelineTools`).
39. **Seat strip (tab strip)** · `browser/parts.dart:66` `Leaf` (+ `_Tab` `:157`) — strip drawn only when seat has >1 panel or the panel has tools; height `chromeRow`; selected tab raised ground with glyph + name, others muted; crowded strips fold unselected tabs to glyph only; trailing slot at the right for front panel's tools; seat frame = 1 px divider border radius 2. States: single, stacked, selected/unselected, compact/labelled, scrolled clip. Body is wrapped in `PanelSeat(stacked)` so panels drop their own title (`DockedPanel`). Story: Workspace stories.
40. **Tab drag handle** · `dock_workspace.dart:176` `_handle` — a tab is a `Draggable`: feedback 85 % opacity, 40 % at origin; drop on a seat edge splits (docking's own zones) or on another seat's strip to join as a tab.
41. **Strip drop targets** · `dock_workspace.dart:90` `_stripTargets` — while dragging, a translucent (0x33F0F0F0) highlight over each open seat's strip row.
42. **Split handles (dividers)** · `dock_theme.dart:18` `hfDockSplit` — 3 px dividers, `Surface.divider` colour, highlight `Surface.selected` on hover/drag; drag to resize; min sizes from `PanelDef`.
43. **Reset Layout** · menu line in item 19 (`dock_workspace.dart:221`) — restores the default preset; panels keep state.
44. **Open closed panel** · `Open <name>` menu lines — `activate(id, near: id)`.
45. **Detach to window** · menu line "Detach" + `window.dart:104` — opens a native panel window (`openPanelWindow`); closing it docks the panel back near its former neighbour. Only if `windowInfo['main'] != false`.
46. **Closable policy** · `seats.dart:112` — only Relations and Web can be closed; Home panels cannot (no ✕ on tabs: removed, comment `parts.dart` "no ✕ here").
47. **Layout persistence** — debounced 350 ms save of the dock snapshot (`hfWorkspace`) and restore at launch (`window.dart:120`, `dock_workspace.dart:283-318`); silent. No visible control.
48. **Panel placement requests** · `window.dart:52` — `placePanel(name,'hide')` closes; other placements open the panel beside the Inspector as its own seat (bottom). Triggered by Inspector links / keys (P, S, R, T, Shift+A reveal the Inspector). No placement table UI.

## 9. Keyboard (visible effects only)

49. **Window keys** · `input/window_keys.dart:11` `LiveKeys` — Esc (cancel preview, close Desk drawer, else clear selection), Space play/pause, F9/Shift/Cmd+Shift easing, Cmd+Z/Shift redo, C/X/V, Cmd+D duplicate, Cmd+Shift+D reselect keys, Cmd+G/Shift ungroup, Cmd+A, Cmd+S / Shift save-as, Cmd+K split, Cmd+Opt+K Composition popover, Cmd+N new, Cmd+O open, Cmd+I import, Cmd+Opt +/-/0 UI scale, Cmd+, Interface popover, Cmd+0/1/=/- Stage Fit/Actual/In/Out, Delete/Backspace, Home/End, arrows (frame step, Shift ×10, Alt nudge keys or selection), Up/Down select layer, M marker, A animate toggle, Shift+A anchor, P/S/R/T reveal property. Ignored while typing in a text field. No on-screen shortcut list.
50. **Viewport motion keys** · `input/viewport_motion.dart` (107 lines) — Stage pan/zoom input helper [uncertain: not read in detail, belongs to the Stage].

## 10. Theme and token system (what it covers)

51. **Neutral ramp N** · `theme/neutral.dart:7` — g00..g100 achromatic greys (~25 steps by lightness), plus clear/shade40/shade55/shade90/glaze*/veil/veilHi/inkSoft translucent roles. Colour is not in the ramp.
52. **Identity colours H** · `theme/identity.dart:6` — family colours `Fam` (scatter pink, stagger blue, along green, face yellow, follow orange, attach violet; some with t/n variants), relation red, warn, wave, textSelection, guide, play green, record pink, mode indigo, toggleOn, playhead; fonts Inter / Menlo; `H.s` sans and `H.m` mono style helpers.
53. **Surface Grammar and Dn text roles** · `theme/metrics.dart:95` `Surface` (topBar 32, namedHeader 28, chromeRow 24, workRow 20, control 18, controlHero 22, ruler 20, menuRow 22, hit 24 (floor 18), gaps, insets, radii, face row/tile clamped; semantic colours base/raised/hover/selected/divider/dividerFine/well/disabled/muted/ink = N steps), `Dn` (nameSize 9, labelSize 10, microSize 9.5, numericSize 11; name/value/label/micro styles), `UiScale` (50–200 %, 100 reset; derived tokens with policies scale/snap/clamp/minimum). Every dimension is a token multiplied by UiScale at build.
54. **Live palette / EditorTheme** · `theme/live_palette.dart`, `theme/editor_theme.dart` — `EditorTheme.chromatic` copy named "Live" (colour map app/panel/raised/hover/line/border/ink/muted/menu/menuEdge/disabledInk), `EditorInk` (stage drawing colours), `EditorTooltip`, `EditorButton` (selectable text button), used for foundation widgets (Stage chrome). Hover/pressed lift .08/.10, menu min width 112.
55. **Glyphs** · `theme/glyphs.dart:7` `HG` enum — 49 painted shapes (text, shape, image, camera, repeater, grid, circle, spiral, scatter, alongPath, stagger, face, follow, attach, blur, glow, color, composite, distort, stylize, arrow, move, rect, ellipse, pen, type, crop, pie, kebab, grid4, list, search, fit, corners, play, pin, folder, lock, plus, star, chevronDown, cross, triangle, headphones, diamond, power, preset, eye, eyeOff, solo); `HgPainter`. Legacy `theme/material_icons.dart` `Glyph` set still used only by the legacy docking tab button.
56. **Not covered by tokens** — Design Mode HUD colours are hard-coded (`dev/design_mode.dart:1189`), UiScaleReadout uses a literal Menlo, `Dock` drop highlight uses literal 0x33F0F0F0, `Console` square sizes are `px()` but colours are H families.
57. **Theme JSON / load-colour-theme / dark-light switch** — absent in this tree (one dark look only; `EditorInk.dark` is "the one look the window has").

## 11. Design Mode (debug builds only)

`dev/design_mode.dart` (1487 lines), `dev/design_core.dart`, `dev/design_props.dart`. Toggle logic in `DesignController`; starts on with `--dart-define=MOTOLII_DESIGN` or restored session.

58. **Design Mode wrapper** · `design_mode.dart:29` `DesignMode` — Stack over the app; when on, paints bounds + grip + HUD above everything with its own Directionality. Absent in release builds (`main.dart:19`).
59. **Selection bounds overlay** · `design_mode.dart:1104` `_BoundsPainter` — box drawn around the clicked element in the window (also padding/margin visualisation [uncertain: not read in detail]).
60. **Grip** · `design_mode.dart:1044` `_Grip` — draggable handle on the selected box to change width / height / padding of the active number.
61. **HUD** · `design_mode.dart:1184` `_Hud` — floating monospace panel (bg 0xF2101114) positioned away from the selection. Rows per property with badges DERIVED / READ-ONLY / UNSET / TOKEN n / HERE / EXCEPTION / VIA x / RAW, a scope line ("TOKEN … uses · T: this place only"), a "UI NN%" readout, hint line "Tab · wheel/↑↓ · Enter type/choices · T scope · A before · R reset · Z undo · Esc · C · [ ]", initial text "Design Mode: click a thing". Modes: idle, editing a row, `_Choices` list for enum/colour-token candidates, SESSION CHANGES list (changed lines in red/green), exit prompt (KEEP / DISCARD / CONTINUE chips: Enter keep, Esc discard). Sub-widgets `_Chip` `:1359`, `_Choices` `:1378`, `_RowView` `:1421`.
62. **Design engine** · `design_core.dart` (Token table read from metrics.dart, source rewriting, `DesignSession` undo/redo/before-after) and `design_props.dart` (Analyzer, Registry, coverage census of visual args) — non-visual; they write edits into source files for hot reload.

## 12. Bridge and session (only what surfaces)

63. **Native bridge** · `bridge/native_bridge.dart`, `bridge/protocol.dart`, `session/session_native.dart` — no widgets; errors and panel-window events come out as `c.error` (Console) and the window callbacks above. `session/latency_probe.dart`, `session/read_model.dart` [uncertain: not read; assumed non-visual].

## Counts and leftovers

(a) Entries numbered 1–63. Of these, distinct user-visible/operable widgets or parts: about 45 (the rest are non-visual engines, notes, cross-refs or absences, items 3, 20–22, 31, 33, 47, 50, 56–57, 62, 63 and similar).

(b) Things I could not classify / did not read fully:
- `EditorPercentField` internals (`controls/panel/scale.dart`) and `ParamSheet`/`ParamStore` rows (`inspector/`), which build the Composition popover body.
- `LiveTimelineTools` (Split, Marker keys at the Timeline strip's trailing end) — Timeline area, not read.
- Whether the Swift runner adds a native menu bar or window title/dirty dot; not visible in `lib/`.
- Whether `Pin` key is intended to be deleted or wired later (shown, never wired).
- Where the loading state could show an error (e.g. native library missing): window stays blank.
- `input/viewport_motion.dart`, `session/latency_probe.dart`, `session/read_model.dart` contents.
- Console default visibility in Workspace stories (front tab is Timeline).
