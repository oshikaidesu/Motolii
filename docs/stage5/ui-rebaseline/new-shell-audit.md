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
| Workspace | detach a panel into its own OS window (SH-29/30, WS-13) | A | tab menu Detach; a panel-only window; panels return when it closes; test |
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
| Inspector | Esc during a drag restores the value (IN-023) | A | Toys, the Transform gizmo and the Layout diagram; `new_transform_test`, `new_layout_test`, `new_effect_test` |
| Browser | six shelves as dock tabs on the finished hf bodies (Create, Effects, Media, Colors, Fonts, Files) over the production shelves: items, apply, menu, drag, preview, tools, path header, editor slot (colour wheel, font scope), tooltip, tile size from Settings, reveal after import | A | `ShelfPanel` (`app/new/browser/`), `new_browser_shelves_test` |
| Browser | keyboard: find, clear, arrows, Enter, Delete, digits file the pick in a collection (BR-005/006/044/045/048) | A | through the seat; workflow and shelf tests |
| Browser | favorites, collections 2 to 7, recent, saved searches | A | same desk settings rows as Classic (`collections`, `collectionNames`); `recent` and `searches` are new rows beside them; menu, digits, header overflow |
| Browser | class column, structured search over the shelf's tags, multi-select and range select, count label | A | hf chassis over `tagsOf`/`classification` |
| Browser | the user's own tags: Tag… and Remove tag in the item menu, found by `tag:` and by plain words; the shelf's value groups become searchable tags | A | `tags` desk rows shared with Classic; `new_browser_shelves_test` |
| Browser | range cuts on continuous facts (duration): seeds and the user's own cuts from the header menu, matched as `duration:5-30` words of the structured search; cuts kept in the `ranges` desk row Classic uses | A | `new_browser_shelves_test` |
| Browser | Filter View chips with counts, per-shelf list and thumbnail view modes | E | superseded by the class column, structured search and the bodies that fold by width; the finished design has no chip panel |
| Desk | History, Blend, Depth, Ease, Notes on the finished desks over the session | A | Desk face hook (`app/new/desk`); History goes to a head, Blend previews on Stage and applies as one step, Ease is one desk logic with two skins (`EaseView`): curves and handles from the runtime model, parameters, Sequence ghosts, keep and apply; Notes likewise (`NotesView`): pages, cards, paste, image, link, file drop; Depth is the top floor plan (no front or side view: the document has no height plan); `new_desk_test` |
| Desk | Web | A | same production panel |
| Document | create, edit, key, scrub, undo, redo, save, reopen, export | A | Flutter: `new_shell_workflow_test`; real engine: `editor::workflow` (run with `--ignored`, about five minutes in a debug build) |
| Document | export in the real-engine workflow | F | the workflow passes through make, move, key, scrub, undo, redo, save and reopen. The export step hangs in the cargo test harness: the worker sits in `re_renderer` file server `watch`, in `notify` `FSEventStreamStart`, waiting on fseventsd (sampled 2026-09-26). This is the known dev shader watcher cold start, not a New shell fault; Flutter's export path is covered by the workflow test with the channel double |
