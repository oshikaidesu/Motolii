# Judge of re-makes, round 2 (item G only)

Read: `sheet.md` (G), `judge_reuse_round1.md`, `ASSETS.md`, `~/rust_ae/HANDOFF.md`, the past repository (read only), `git diff 8a0a35e..HEAD` (19 commits, 70 files, +2850 / -262), and the main files whole as they are at `a0038ab`. Nothing was built, run or changed. I have not seen this project before.

## G = 0

Counted re-makes in the code as it is now: **5** (1 carried over from round 1 and kept on purpose, 4 new; all "likely", none of them a line-for-line copy). The sheet's line for 0 is "three or more". Glass and light are not counted and none of the five is glass or light.

Round 1's eight: **4 taken back, 3 half, 1 not** (the builders say 7 and 1).

The pattern has not changed, it has moved. Round 1 found the document's rules typed on the shell's side for the inspector. That was taken back at 09:17. The three gestures written after it (a thing carried on the stage 09:29, bars held 09:35, clip 09:46) were written the same way again: the shell works out the change, tries it value by value, and says it a second time when the hand lets go. The past repository had one form for all of them (a gesture is begun, updated, committed or cancelled, and the inside works out the edits and their limits once).

## A. Round 1's eight, as they are now

1. **`StageTexture` a copy of `GlassTexture`: taken back.** One `LiveTexture` (`skin/book/macos/Runner/MainFlutterWindow.swift:25-59`); `GlassTexture` (`:64-95`) and `StageTexture` (`:101-124`) are two callers that differ in symbols and arguments only.
2. **A second reader and teller of a way, in Dart: half.** `_said` and `_heard` are still there (`skin/book/lib/workspace/desk_ease.dart:107-122`). What changed is around them: the inside says a way back in one form (`filament/interp.h:158-164`, `filament/doc.h:390-396`) so the Dart reader's exact match now hits, and the desk tells nothing when nothing changed (`desk_ease.dart:147`). Still true: `Bounce` with its numbers is heard as the preset and told back as `Bounce` alone, so Reverse or a preset throws the document's numbers away; `double.parse` runs on the inside's words with no guard (`:114`).
3. **Second material loader: taken back.** `material(s, file)` in `filament/kit.h:180-187`, used by `add` (`:190`) and by `solid_fill` (`filament/solid.h:124`).
4. **Value names typed again in the shell: half.** The list of names that move is asked of the inside (`doc.h:522-528`, `skin/book/lib/workspace/ws.dart:142`, `desk_ease.dart:128`), and the drift round 1 found (opacity) is gone. `_propNames` is still there, and the same names are typed three times in one file as the inspector's map: `skin/book/lib/workspace/workspace.dart:234-245`, `:255-260`, `:263-272`.
5. **`motolii_try` / `motolii_try_drop`: not taken back (kept on purpose).** `filament/doc.h:626-633`, `filament/motolii.mm:131-132`, `filament/motolii.h:54-57`. Counted again below (B1): the reason given is about the names, the finding was about the form.
6. **`motolii_together`: half.** The column, the migration and the loop in undo are gone; SQLite's own `sqlite3changeset_concat` joins the changes (`doc.h:345-357`). Still true from round 1: each change commits by itself, so if the second change of an act is refused the first stays, as a step (`doc.h:359-366`). New: the bracket is one int, so a bracket inside a bracket ends the outer one (`motolii.mm:171`; the inspector's host opens one at `skin/book/lib/sets/inspector/inspector_parts.dart:236` and `workspace.dart:203` and `:290` open another inside it). It is still not the past's form (one call carries the edits; if one fails none stays).
7. **The document's editing rules worked out in Dart: taken back, for the inspector.** `doc_put`, `doc_key_here`, `doc_stay`, `doc_keyed` (`doc.h:479-484`, `:511-514`); the shell only calls them (`workspace.dart:297`, `:309`, `:321-325`). The same pattern is back in the code written after it: B3 and B4.
8. **Which file is which kind, as a table in the dock: taken back.** `doc_kind_of` (`doc.h:426-436`), `motolii_kind_of` (`motolii.mm:145`), `motolii_add` takes the kind from the file (`doc.h:438`); the dock asks (`skin/book/lib/workspace/dock_files.dart:129-137`, `:180`). Left over: one list of photo endings as a regex in the dock (`dock_files.dart:546`).

## B. Counted re-makes, as the code is now

### B1. "Show a value without writing it", with no keep (carried from round 1; likely)
- Today: `filament/doc.h:626-633` (`doc_try`, `doc_try_drop`), `filament/motolii.h:54-57`.
- Existing: the past preview: `~/rust_ae/Motolii/motolii/crates/motolii-edit/src/document/edit.rs:134-232` (`begin_preview`, `preview_edits`, `clear_preview_edits`), driven by `ui/native/src/port.rs:199-201` (`previewProperties`, `commitPreview`, `cancelPreview`). Ledger: `ASSETS.md:88`.
- Why it is still counted: the names try and drop are the skin's, fine. What was not taken is that the inside keeps what was tried and a commit writes it as one step. Since round 1 that lack has been paid for three times: every gesture says its values twice, once tried and once more at let-go (`workspace.dart:297`; `skin/book/lib/workspace/stage_view.dart:120-121` then `:134-137`; `skin/book/lib/workspace/timeline.dart:226-227` then `:239-241`), and `motolii_together` is needed only because of it. How it is done: a made-up one-key list is written into the cache of keys (`doc.h:628-630`), so any change by anyone drops every try.

### B2. Clip to below, as an upright rectangle (new; likely)
- Today: `clip` in `filament/doc.h:22`, `:192-208` (Filament's scissor on the thing's looks, told the upright box around the frame of the thing before it), the seat in `workspace.dart:173-187`.
- Existing here: the kit's matte, `filament/kit.h:623-706` (`matte`, `matted`: "seen only inside it", any mesh or picture, soft, up to four), ledger `ASSETS.md:63`. `flat.mat` and `thick_glass.mat` read mattes, so films, pictures, texts, flat shapes and glass could be cut by the true shape today.
- Existing in the past: clip to below IS a matte by the layer below (`crates/motolii-render/src/picture/resolve.rs:135-139`; the command `clip`, `ui/native/src/port.rs:275`).
- Today's is weaker than both: a circle clips as its box; a turned thing clips as the upright box around it (`doc.h:202`); a thing leaning back (`rotate_x`, `rotate_y`) clips by its frame as if it stood straight, because `doc_frame` does not know those turns (`doc.h:609-622`). In fairness: the scissor itself is Filament's, and it is the only way here to cut a 3D file, whose looks do not read mattes. That is a reason to use it for those looks, not for every kind.

### B3. Bars held on the timeline: the timing rules, in the shell (new; likely)
- Today: `skin/book/lib/workspace/timeline.dart:197-244`. The shell decides which end is held (`:207`), clamps both ends (`:217`, `:219`), keeps "on for good" on to the end (`:222`), tries `start` and `duration` one by one (`:226-227`), and chooses which of two calls settles it (`:238-242`). Inside: `doc_trim`, `filament/doc.h:486-496`.
- Existing: one gesture form, `previewTimings` / `setTimings` with `{layer, mode: move | trimStart | trimEnd | slip, delta}` (`ui/lib/timeline/session.dart:300-320`, `ui/native/src/port.rs:235`, `:359-371`); the timings and their limits are worked out inside, once (`ui/native/src/editor/functions/verb.rs:12-66`, `timing_delta`). Its `TrimStart` moves `source_in` with the head: the same rule as `doc_trim`'s "a film cut at its head begins further into itself".
- This is round 1's item 7 again, written 18 minutes after item 7 was taken back. No slip today.

### B4. A thing carried on the stage: the drag, in the shell (new; likely)
- Today: `skin/book/lib/workspace/stage_view.dart:101-139` (`_hold`, `_carry`, `_letGo`): reads x and y, turns screen points into stage units with `motolii_point`, tries both, and puts both again at let-go inside a bracket.
- Existing: `stageGesture` begin / update / commit / cancel (`ui/native/src/port.rs:372-399`, `ui/lib/stage/session.dart:280-321`); the inside works out the edits from the pointer (`ui/native/src/editor/stage.rs:569-600`), with a test that the grabbed point stays under the pointer for tilted layers (`stage.rs:730`).
- Today's is right only at z 0 (`motolii.h:58` says so): a thing with a z of its own, or leaning, does not stay under the hand.

### B5. A revision counted by the shell (likely, small)
- Today: `Ws.rev`, `reread()`, `shown()` (`skin/book/lib/workspace/ws.dart:115-153`), bumped by hand at 16 places in 7 files; read by the dock's cache (`dock_files.dart:163-175`) and the stage's scene (`skin/book/lib/workspace/stage_art.dart:46`, `:69`).
- Existing: the document's own `revision()` and `display_revision()` (`crates/motolii-edit/src/document.rs:392-404`; the second is exactly "what is shown changed and the document did not", today's `shown()`), handed out as `documentRevision` (`ui/native/src/snapshot.rs:317`).
- `rev` was there at round 1 (56cbfe9), `shown()` came after it (e1234af). A call site that forgets it shows a stale window; the inside knows when it changed and is not asked.

## Seen, not counted

- **`motolii_raise`, `order`** (`doc.h:21`, `:83-88`, `:497-510`). Past: `reorder` with a delta on `order` (`port.rs:233`, `crates/motolii-edit/src/document/intent.rs:48`, `crates/motolii-doc/src/store.rs:503`). Same form, same word. How it is shown is in C2.
- **The stage's size on thing 0** (`doc.h:285-304`, `motolii.mm:42`, `:50`, `:135-138`). Past: one `Composition` record: width, height, fps, duration, background (`crates/motolii-doc/src/store.rs:276-283`), changed as a step (`port.rs:282`). Today's is facts, doc.h's own manner, not a step, and only at the next opening. Unsure. Two things to know: `wide` now means two things in one document (thing 0: points across; a shape: how many times as wide as high, `doc.h:23`), and the rate is not in the document at all (`Ws.fps = 30` in `ws.dart:98`; the inside draws 60 and writes 60, `motolii.mm:76`, `:208`).
- **Thumbnails.** Past: `ui/native/src/editor/thumbnail.rs` (remembered inside `:7-19`, warmed on the import thread `:22`, a film's first frame `:91-107`, a photo through the stage's own decoder `:31-38`). Today the pictures are borrowed (D); only the remembering is by hand, in Dart (`skin/book/lib/motolii.dart:20-35`). Not counted; C3.
- **`solid_mesh`** (`filament/solid.h:66-115`): a second reader of a file `solid()` already reads (`:33-64`), with the fit typed again on points (`:110-113` against `:51-52`). gltfio hands no triangles back, so some loop is needed. Unsure, glass-side. C4.
- **`flat`** (`filament/drawn.h:149-161`): `drawn` already draws a one-colour rectangle of ThorVG's as its `ground` (`drawn.h:67-72`); `flat` types the same two calls. Small.
- **Small doubles inside today's own code:** the eye's projection (`doc.h:615-618` and `motolii.mm:174-179`); the walk over a 3D file's faces (`doc.h:161-164` and `solid.h:128-132`); `flat.mat`'s hue formula typed for the CPU (`filament/video.h:96`, as round 1 said; the planes now come through `gst_video_frame_map`).
- **What a shell reads** (round 1, unsure): still many small calls, and more of them. `reread` is about 15 trips to the hand per thing; the stage asks `motolii_frame` for every thing on every scene it builds (`stage_art.dart:47-52`); the Ease desk asks `motolii_value` 49 times on every build for a way with no seat (`desk_ease.dart:155-165`), which while playing is every tick.

## C. A standard part would do it (not counted in G)

1. **Several changes as one step** (`doc.h:340-366`, `motolii.mm:171`): SQLite's `SAVEPOINT` / `RELEASE` / `ROLLBACK TO` around the act with one session open across it. Nothing stays if one change is refused, brackets nest by themselves, and the last step's blob is not read back and written again for every joined change.
2. **Which thing is in front** (`doc.h:172`, `:186`: a ten-thousandth of z per place): Filament's `RenderableManager::setBlendOrderAt` with `setGlobalBlendOrderEnabledAt` (`RenderableManager.h:1016-1018`). The z hair moves things (the eye is a perspective one), grows with the number of things, and is added to a z the user set. The kit's `add` already takes an `order` (`kit.h:189`, `:221`).
3. **Thumbnails remembered** (`motolii.dart:20-35`): the file's name is `String.hashCode` in hex, which Dart does not promise to be the same from run to run and which two films can share. Flutter's own image cache already remembers a decoded picture (`Image.file(cacheWidth:)`, used for photos at `dock_files.dart:547`); a place for files is `path_provider`'s cache folder. For the picture itself GStreamer has a one-call form (`gst_video_convert_sample`). Each OS also has its own thumbnailer for every kind of file (macOS: `QLThumbnailGenerator`), which would fill the tiles that are flat now, but it is a per-OS road.
4. **A 3D file's triangles** (`solid.h:69-115`): gltfio's `FilamentAsset::getSourceAsset()` (`FilamentAsset.h:242`) hands out the cgltf data it has already parsed, until `releaseSourceData` (`solid.h:50`). One reader of the file, not two. Today's second reader does not go through gltfio's resource loading (I did not check Draco files; read by cgltf alone they have no positions).
5. **Clip** (B2): `matted` for looks that read mattes; the scissor for those that do not.
6. **Picking by boxes** (round 1's C3, still so: `stage_art.dart:271-277`, `doc.h:591-622`): `View::pick`, `RenderableManager::getAxisAlignedBoundingBox`. Clip now leans on the same boxes.
7. **The binding typed by hand** (round 1's C2, now 39 lines: `motolii.dart:67-105`): `package:ffigen`. Not bound: `motolii_keys`, `motolii_define`, `motolii_ways`, `motolii_still`, `motolii_write`.
8. **The one hand** (round 1's C4, unchanged: `motolii.mm:30-78`): GLib's main loop, already linked in.
9. **The disk browser** (round 1's C5, unchanged: `dock_files.dart:139-161`, `:199-210`): `file_selector` / `desktop_drop`; `_base`, `_dir`: `package:path`.
10. **A file's kind by the end of its name** (`doc.h:426-436`, and the regex at `dock_files.dart:546`): the owners can say (`GstDiscoverer`, ThorVG's load, gltfio's).
11. **The inspector's host** (round 1's C8, grown from one mirror to four: `inspector_parts.dart:231-281`, `:300-318`): values, key marks, words and flags are each found by comparing a whole map with a remembered copy on every notify. Flutter's form is per field: `onChangeStart` / `onChanged` / `onChangeEnd`, or a `ValueListenable` per value. The past skin's form was three hooks per row: previewed, committed, cancelled (`ui/lib/inspector/transform/store.dart:131-165`).

## D. Non-findings: looked for a re-make, found a borrow

- **`LiveTexture`**: one class, Flutter's `FlutterTexture`.
- **Materials**: one loader, kept on the stage.
- **A way read back**: `interp_words` writes from the same table `interp_said` reads (`interp.h:158-164`); the nine kinds' numbers are still not in Dart.
- **The hand's rules**: one place (`doc.h:472-514`), built from `doc_key`, `doc_set`, `doc_keys`; no second rule set.
- **Moving names, words, choices**: read from `doc_values` (`doc.h:516-528`); the blend chooser offers the document's words (`inspector_material.dart:24`); the Chooser's list in the window's overlay is Flutter's `OverlayPortal.targetsRootOverlay`.
- **Kind of a file**: one place inside, asked by `add` and by the dock.
- **A film's thumbnail picture**: one line of GStreamer's own parts (`uridecodebin`, `videoscale`, `pngenc`: `motolii.mm:148-170`); no hand decode, scale or PNG writer. Photos: Flutter's `Image.file`. Off the window's thread: Dart's own `Isolate.run`.
- **Flat shapes**: ThorVG's `Shape` through the existing `drawn`, the existing `flat.mat`, blend and opacity. No new material, no path code.
- **Clip's mechanism**: Filament's `setScissor` / `unsetScissor`; no shader.
- **3D triangles**: cgltf as Filament ships it (`cgltf_node_transform_world`, `cgltf_accessor_read_float`); no parser written.
- **Ambient occlusion**: Filament's own option, one line (`motolii.mm:46`).
- **Order**: sorted by SQLite (`doc.h:85`), one change for the whole renumbering.
- **Undo joined**: `sqlite3changeset_concat`; no column, no loop.
- **A film's ground**: planes through `gst_video_frame_map`, the film's own colour rule, the existing `key` / `keySoft` parameters. No existing auto-key was found anywhere to have been re-made (the past's `chroma_key.fs` takes its colour from a field).
- **Gestures in the window**: Flutter's `GestureDetector`; the existing `WsLayer`, rows, bars, `KeyS` marks, `NumField`, `DockFooter`, `DockSection`, `_MakeTile`, `DockBadge`. Today's diff adds no new widget class to the workspace (one enum, `_Part`). The skin's shelf has no ready-made bar-trim or stage-carry part that was passed over.
- **Undo**: still no stack in Dart for the document. GSAP table: not brought.
