# Judge of re-makes, round 1 (item G only)

Read: `ASSETS.md`, `~/rust_ae/HANDOFF.md`, the past repository (read only), `git diff 05c2005..HEAD` (11 commits, 54 files, +1781 / -172). Nothing was built, run or changed.

## G = 0

Counted re-makes: **8** (4 sure, 4 likely). The sheet's line for 0 is "three or more". Glass and light are not counted and nothing of them is in the list.

The pattern behind most of them: the shell (Dart) was given small reads and small writes, and then the document's rules were typed a second time on the shell's side. The past repository had those rules inside (edit crate, snapshot), and so does today's `doc.h` for some of them.

## Counted re-makes

### 1. `StageTexture` is a second copy of `GlassTexture` (sure)
- Today: `skin/book/macos/Runner/MainFlutterWindow.swift:98-155`.
- Existing: the same file, `:26-92` (`GlassTexture`). Ledger: `ASSETS.md:27`, and `ASSETS.md:34` says in so many words that the Flutter-side receiver is the same for either picture.
- Same job line for line: dlopen, dlsym of a front and a frames function, register a `FlutterTexture`, a 120-a-second `Timer` that compares a frame count, `copyPixelBuffer` handing the retained buffer on. What differs: the channel's name, the two symbol names, and that the functions take a handle. Three parameters, not a second class.

### 2. A second reader and teller of a stretch's way, in Dart (sure)
- Today: `skin/book/lib/workspace/desk_ease.dart:99-114` (`_said`, `_heard`).
- Existing: `filament/interp.h:159` (`interp_said`, the parser the document itself uses). Past form: `~/rust_ae/Motolii/motolii/ui/native/src/snapshot.rs:22-33` (`interp()` hands the desk the kind and its numbers as fields; the past desk never parsed words).
- The two already disagree. `interp_said` takes names in any case and a kind said alone (usual numbers). `_heard` matches `'Hold'`, `'Bounce'`, `'Bezier'`, `'Linear'` exactly, and `'Bezier'` only with four numbers: `bezier 0.2 0.1 0.3 1` or `Bezier` alone, both right for the document, come out on the desk as "a kind with no seat" over a straight line. `Bounce` with its numbers is heard as the preset and told back as `Bounce` alone, so touching the desk (Reverse, a preset) throws the document's numbers away.

### 3. A second material loader in `solid.h` (sure, small)
- Today: `filament/solid.h:70-76` (`static Material* lit`, `load_file("floor_lit.filamat")`, `Material::Builder`).
- Existing: `filament/kit.h:178-181` (the read-once table inside `add`), which today's own `doc.h:201` already uses for the same file (`add(s, made.shape, "floor_lit.filamat")`). The same material is now built twice by two tables.

### 4. The value names typed again in the shell, twice (sure)
- Today: `skin/book/lib/workspace/ws.dart:110` (`Ws.moving = x, y, z, rotate, size`) and `skin/book/lib/workspace/workspace.dart:217-222` (`_propNames`, which has `opacity`).
- Existing: `filament/doc.h:48` (`doc_values`: every name, which kinds have it, its range, whether it moves).
- Already out of step: `opacity` moves in `doc.h` and in `_propNames`, not in `Ws.moving`. A key on opacity shows in the inspector's mark, not on the timeline bar (`ws.dart:131`), and the Ease desk cannot reach its stretch (`desk_ease.dart:116-122` walks `Ws.moving`).

### 5. "Show a value without writing it" (`motolii_try`, `motolii_try_drop`) (likely)
- Today: `filament/doc.h:452-459`, `filament/motolii.mm:143-144`, `filament/motolii.h:47-50`.
- Existing: the past preview: `~/rust_ae/Motolii/motolii/crates/motolii-edit/src/document/edit.rs:134-230` (`begin_preview`, `preview_edits`, `clear_preview_edits`), driven by `ui/native/src/port.rs:61-67` and `:199-201` (`previewProperties`, `commitPreview`, `cancelPreview`). Named in the ledger's row `ASSETS.md:88`.
- Same job. The past form is a set of edits held under one owner, and commit applies that set as one step. Taking that form would have made item 6 unnecessary. In fairness: the words try and drop are the skin's own (`ws.dart:242-256`, `tryFx`, `keepFx`, `dropFx`); keep was not taken, a try is settled by an ordinary set.

### 6. "Several changes are one step of undo" (`motolii_together`, `step.joined`) (likely)
- Today: `filament/doc.h:36`, `:232`, `:237-238` (a new column and a migration), `:263-265`, `:281-306` (`doc_step_one` and the loop), `filament/motolii.mm:145`.
- Existing: the past `apply_all` / `apply_then`: `~/rust_ae/Motolii/motolii/crates/motolii-edit/src/document.rs:151-165` (one call carries several edits, one step; if one fails none stays). And `doc.h`'s own `doc_change` (`:247`) is already "whatever one SQLite session saw is one step": `doc_add` (`:342`) writes four rows as one step that way.
- Today's form is weaker than both: if the second change of a bracket is refused, the first stays, as a step of its own.

### 7. The document's editing rules worked out in Dart (likely)
- Today: `skin/book/lib/workspace/workspace.dart:238-253` (`_write`: a value that has keys gets a key at the playhead, else it is set), `:271-286` (`_mark`: key on, key off, no longer animated), `:257-268` (`_keyMarks`: is there a key now), with the test `(s - at).abs() < .5 / Ws.fps` typed three times (`:265`, `:276`, `:282`).
- Existing: past `crates/motolii-edit/src/document/edit.rs:38-132` (`place_checked`: the same rule, inside), `ui/native/src/port.rs:244` (`toggleKey`), `ui/native/src/snapshot.rs:46` (`keyedNow`, worked out inside). In today's inside: `motolii_keys` (`filament/motolii.h:26`) and `doc_keys(..., at)` (`filament/doc.h:363`, "whether it has one at a moment") were there before today; `motolii.dart` does not bind `motolii_keys` and asks for all the key moments instead.

### 8. Which file is which kind, as a table in the dock (likely)
- Today: `skin/book/lib/workspace/dock_files.dart:128-143` (`_glyphOf`, `_kindFor`).
- Existing: past `~/rust_ae/Motolii/motolii/crates/motolii-render/src/media.rs:25-60`, written there as the one door for it (`import_extensions`, `asset_type_for_extension`), on the inside, read by the OS chooser, by drop and by the browser alike.
- Today the table lives in one shelf of the shell; `motolii_add` still has to be told the kind by its caller, so the next caller types the table again.

## Seen, not counted (the form existed, today's is its own)

- **What a shell reads.** Today: `motolii_things`, `motolii_word`, `motolii_lasts`, `motolii_key_moments`, `motolii_way_of`, `motolii_frame`, `motolii_point` (`filament/motolii.h:33-51`), put together in Dart by `Ws.reread` (`ws.dart:116-142`, about 14 calls a thing, each one a trip to the inside's thread and back). Past form: one reading of the whole state (`ui/native/src/snapshot.rs:271-355`, with `documentRevision`, per-value `keys`, `keyedNow`, `bounds`), named in `ASSETS.md:48`. Unsure: the past one cannot be taken whole, and many small calls is `motolii.h`'s older manner.
- **A thing's frame on the picture** (`motolii.mm:127-141`, `doc.h:434-447`). Past: `snapshot.rs:206-253` (`bounds`, four corners as the eye sees them). Today's is a middle, a width, a height and one turn: right for flat sheets facing the eye, wrong for anything tilted or deep. Unsure.
- **The three pixel buffers** (`filament/live.h:26-73`). Written in `glass_door_mac.mm`'s manner, and says so; the old door hands out Metal swap chains, which Dawn cannot take, so it could not be used whole. The names were not kept (`live_open`, `live_front` against `glass_door_open`, `glass_door_hand`). Not counted.
- **Reading a folder of the disk** (`dock_files.dart:152-165`). Past: `ui/native/src/catalog/scan.rs` (a walked, indexed, watched catalogue). A different size of thing. Not counted; see the next list.

## A standard part would do it (not counted in G)

1. **Several changes as one step** (`doc.h:36`, `:232-238`, `:263-306`): SQLite's session extension does this itself. Keep the one `sqlite3_session` open from the bracket's start to its end and take one changeset, or join the changesets with `sqlite3changegroup_add` / `sqlite3changeset_concat`. No `joined` column, no migration, no loop in undo, and a bracket that fails rolls back whole.
2. **The binding typed by hand** (`skin/book/lib/motolii.dart:46-74`, 29 `lookupFunction` lines with both signatures each): Dart's own `package:ffigen` writes it from `motolii.h`. The hand list has already drifted from the header (`motolii_keys`, `motolii_define`, `motolii_ways`, `motolii_still`, `motolii_write` are not there).
3. **Picking by boxes** (`motolii_frame`, `doc_half`, `skin/book/lib/workspace/stage_art.dart:271-277`): Filament's `View::pick(x, y, ...)` says which renderable is under a point, exact for a ring's hole, a tilted solid, a keyed-out film. A thing's bounds: `RenderableManager::getAxisAlignedBoundingBox`, one way for every kind, instead of a loop over a mesh's points for some and gltfio's box for others.
4. **The one hand** (`filament/motolii.mm:30-76`: a queue, a condition variable, `packaged_task`, its own 16,667 microsecond pacing): GLib's main loop is already linked in through GStreamer (`g_main_context_invoke`, `g_timeout_add` on a loop of its own thread). Note too that the measured finding in `glass_door_mac.mm:36-37` (a thread of default priority is woken late: asked 16.7 ms, slept about 22) was not carried: `motolii_hand` sets no priority.
5. **The disk browser in the dock** (`dock_files.dart:146-212`): the OS's own chooser through Flutter's `file_selector` (or a drop through `desktop_drop`). It also works with the sandbox on; a hand-made lister does not. `_base`, `_dir`: `package:path` (already in `pubspec.lock`).
6. **A file's kind by the end of its name** (`dock_files.dart:128-143`): the owners can say. GStreamer's `GstDiscoverer` (has it a video stream, a sound stream, is it a still), ThorVG's `Picture::load` result, gltfio's; or macOS's `UTType(filenameExtension:)`.
7. **A film's corner pixels read by hand** (`filament/video.h:86-111`: CoreVideo plane pointers, macOS only, with `flat.mat`'s hue formula typed again for the CPU): GStreamer's `gst_video_frame_map` reads the same planes on every OS from the `GstSample` already held.
8. **"While the hand is on it, then once when it lets go"** (`inspector_parts.dart:175-190`, `onValues(changed, live:)`, `onDrop`): Flutter's own form is `onChangeStart` / `onChanged` / `onChangeEnd`.

## Non-findings: looked for a re-make, found a borrow

- **Number field**: the existing `NumField`; three lines added to call the inspector `Doc`'s existing `beginGesture` / `endGesture` (`inspector_number_field.dart:124`, `:161-164`, `:185`). No new field.
- **Key marks**: the existing `KeyS` and the inspector's own marks, fed from outside.
- **Undo**: no stack in Dart. The top bar's existing buttons call the document's undo (SQLite's session), `top_bar.dart:145-151`.
- **Timeline**: the existing `WsLayer`, rows, bars, key marks, playhead. Only what fills them changed.
- **Dock**: the existing `FileNode`, tree, tiles, rows, `DockFooter`, `DockSection`, `_MakeTile`. No new widget.
- **Thumbnails**: none made by hand today; a disk file wears the existing `_Art`.
- **Stage**: the existing `StageView`, `ArtBox`, selection painter (split into under and over), Flutter's own `Texture`.
- **Ease desk**: the existing desk and presets; the nine kinds' numbers stay in `interp.h` and were not typed into Dart (only the words' reader was, item 2).
- **`motolii.dart`**: in `sets/glass_live.dart`'s `dart:ffi` manner, `package:ffi` from pub.
- **`live.h`**: Dawn's shared texture memory and Filament's `Texture::Builder::import` through the door the patch already had. No new change to Filament.
- **Opacity**: ThorVG's word, one parameter in the existing `flat.mat`.
- **Keying by a film's own ground**: the existing `key` / `keySoft` parameters and `key_out`'s numbers. No new shader.
- **Fill**: the existing `thick_glass`, `floor_lit.mat`, and the kit's `gem`, `cube`, `torus`.
- **GSAP table**: not brought.
