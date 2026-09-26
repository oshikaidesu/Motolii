# New shell — capability audit against Classic (working checklist, 2026-09-26)

Classic is read only as a source of what a person could do. Classes: **A** works in New, **B** built but not wired, **C** finished in `lib/hf` but not on real data, **D** Classic only and still to port, **E** superseded by the current design (not ported), **F** needs a product decision.
Sources: the Phase A inventory in `inventory/`, `routing.md`, and the code. Evidence is a test named in the last column where there is one.

| Area | Capability | Class | Where / evidence |
|---|---|---|---|
| Workspace | split, resize, tab stack, move, close, reopen | A | `docking`; `proto_hf_dock_test`, `new_shell_workspace_test` |
| Workspace | show a panel by name without rearranging the rest (SH-18, WS-19/20/21) | A | `DockWorkspace.activate`; workspace test |
| Workspace | View menu lists every panel; Reset layout (SH-19) | A | `new_shell_workspace_test` |
| Workspace | save and restore layout, selected tab, sizes (WS-14) | A | settings key `newWorkspace` v1; workspace test |
| Workspace | old dock JSON shape, migration, panel placement table (ST8-04) | E | dock replaces the placement table; reset instead of migration |
| Workspace | Desk drawer as a dock placement | E | not ported; the Desk panel keeps its own tool drawer |
| Workspace | detach a panel into its own OS window (SH-29/30, WS-13) | D | native `openPanelWindow` exists; no New route yet |
| Shell | File, Edit, shortcuts, confirm-close, file drop, doc name, status line (SH-01..17, 23..27, 36, KB-*) | A | shared functions; `new_shell_workflow_test` |
| Shell | Composition, Export, Settings sheets (SH-20/21, ST8-01..03) | A | workflow test |
| Shell | UI scale (ST8-07) | A | read, applied, set in Settings, saved beside Classic's keys; workflow test |
| Shell | theme JSON (ST8-05), Outside dim (ST8-06) | E | New has fixed tokens; Outside dim never took effect in Classic |
| Console | log of errors and notices with search and Clear | A | `NewConsole`; no Classic Console existed, so it shows what the status line already said |
| Stage | all Stage and Camera view capabilities (ST-*) | A | same production panel; `stage_in_dock_test` for pointer mapping through dock moves; `new_shell_workspace_test` for the surface protocol |
| Stage | surface withdrawn when hidden, re-asked on resize and move | A | dock supplies `Visibility` |
| Timeline | all Timeline capabilities (TL-*), export | A | same production panel, Classic Group Layer semantics, new visual skin only |
| Inspector | header, cards, effect stack (grip, eye, menu, throw, rest, expand, remove), text, fill, matte, World, Camera card | A | same production panel |
| Inspector | Transform card | A / C | promoted: Transform Instrument on the session; `new_transform_test` |
| Inspector | Layout card | A / C | promoted: Layout Instrument on a layer's `layout.*` rows; `new_layout_test` |
| Inspector | effect parameters | A / C | promoted: generic Toys with keys, routes, relative drag; `new_effect_test`. Effects that lay out copies (grid of Each / Random) keep the Inspector body until an Instrument owns that grid |
| Inspector | Camera card | A | Classic card kept. The Camera Instrument in `lib/hf` is finished but not connected (C) |
| Inspector | Esc during a drag restores the value (IN-023) | D | the Toys cancel typing but not a running drag |
| Browser | shelves as their own panels, search per shelf, shelf tools, apply, context menu, tooltip, tile size, reveal after import, Fonts/Colors routes | A | `NewBrowser` over the shared shelves |
| Browser | keyboard: find, clear, arrows, Enter, Delete (BR-005/006/044/045/048) | A | workflow test |
| Browser | category rail, collections, saved filters, tags, filter groups, view modes, multi-select and range select, quick tags, count label (BR-007/008/020..039/041..043/049..050/060..066) | D / E | The finished hf Browser (class column, structured search, favorites, saved views) is the design that replaces them, and it is still on fixtures (C). Until it is on the real shelves these are missing in New |
| Desk | Ease, Depth, Blend, History, Notes, Web | A | same production panels. The hf desks are finished on fixtures (C) |
| Document | create, edit, key, scrub, undo, redo, save, reopen, export | A | Flutter: `new_shell_workflow_test`; real engine: `editor::workflow` (run with `--ignored`, about five minutes in a debug build) |
