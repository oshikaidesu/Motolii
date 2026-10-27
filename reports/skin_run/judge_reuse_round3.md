# Judge of re-makes, round 3, the last (item G only)

Read: `sheet.md` (G), `judge_reuse_round1.md`, `judge_reuse_round2.md`, `ASSETS.md`, `~/rust_ae/HANDOFF.md`, the past repository (read only), `git log 05c2005..HEAD` (24 commits), the diff since round 2 (`e0f654c..HEAD`) file by file, and `filament/doc.h`, `motolii.h`, `motolii.mm`, `skin/book/lib/motolii.dart`, `workspace.dart` whole as they are at `86e5315`. Nothing was built, run or changed. I have not seen this project before.

## G = 0

Counted re-makes in the code as it is now: **5** (2 carried from round 2 and kept on purpose, 1 carried from round 1 that round 2 left at "half" and that has grown since, 2 new since round 2). The sheet's line for 0 is "three or more". None of the five is glass or light.

Round 2's five: **3 taken back, 2 not** (the two the builders say they kept).

What changed and what did not. The three gestures are now one form and the reckoning is inside: that take-back is real and whole. But the two things written after it went the old way again. One (a stage-sized Solid) is the shell working out what the document is to hold, a fourth time. The other (flat things lying by order against solid ones) is a law the past repository wrote down with a date, with two mechanisms for it, in a crate the ledger marks "未確認": nobody looked there before writing it a third way.

## A. Round 2's five, as they are now

1. **B1, "show a value without writing it", with no keep: taken back.** `doc_hold` / `doc_carry` / `doc_let_go` (`filament/doc.h:686-731`): what is carried is kept inside (`Doc::Held::tried`, `doc.h:50`, read by `doc_number` at `:126-127`), let go is one act. `motolii_try`, `motolii_try_drop`, `motolii_trim` are in no file.
2. **B2, clip to below as an upright rectangle: not taken back (kept on purpose).** Still Filament's scissor on every kind's looks (`doc.h:230-245`). The box is better: `doc_covers` (`doc.h:659-672`) goes through the thing's own turns. Counted again, B1 below.
3. **B3, bars held, the timing rules in the shell: taken back.** The shell says which part and how many seconds (`skin/book/lib/workspace/timeline.dart:200-225`); limits, "on for good" and a film's head are inside (`doc.h:706-714`); the bar is drawn from what the inside says back (`ws.dart:134-140`). No slip, as before.
4. **B4, a thing carried on the stage, the drag in the shell: taken back.** The shell says the hand's travel in picture points (`skin/book/lib/workspace/stage_view.dart:105-128`); the inside puts it at the thing's own depth (`doc.h:702-705`, `doc_point` `:643-646`).
5. **B5, a revision counted by the shell: not taken back (kept on purpose), and there is a second one now.** `Ws.rev`, `shown()`, `reread()` (`skin/book/lib/workspace/ws.dart:128`, `:143-153`), and `dockRead` (`skin/book/lib/workspace/dock_files.dart:148`, `:177`). Counted again, B2 below.

Round 1's leftovers, for the record: several changes as one step is whole now (one session across the act, all or nothing, nested: `doc.h:364-405`); the value names are one table (`workspace.dart:314-325`); the reader and teller of a way in Dart still stand (B5 below).

## B. Counted re-makes, as the code is now

### B1. Clip to below, by a rectangle of the screen, for every kind (carried; likely)
- Today: `filament/doc.h:230-245` (`setScissor` on every look of the thing, through `doc_looks` `:165-175`), the box `doc_covers` `:659-672`, the seat `skin/book/lib/workspace/workspace.dart:260-272`.
- Existing here: the kit's matte, `filament/kit.h:634` (`matte`) and `:692` (`matted`: seen only inside it, any shape, soft, four on a thing); `filament/flat.mat:12-14`, `:54` reads it. So do films, pictures, texts, flat shapes and glass today.
- Existing in the past: clip to below IS a matte by the layer below (`crates/motolii-render/src/picture/resolve.rs:135-139`; the command, `ui/native/src/port.rs:275`).
- What stands: a circle clips as its box, a turned rectangle as the upright box round it. The reason given (the scissor is the only way to cut a 3D file) is a reason for the looks that cannot read a matte, not for the five kinds that can. In the target it is a 3D file cut by an upright rectangle, the one case where the two give the same picture; turn the rectangle and they do not.

### B2. A revision counted by the shell, twice (carried; likely, small)
- Today: `Ws.rev` (`ws.dart:128`), bumped through `reread()` / `shown()` at 17 call sites in 7 files; read by `dock_files.dart:183-185` and `stage_art.dart:46`, `:69`. New since round 2: a second counter for another thing that changed, `dockRead` (`dock_files.dart:148`, `:177`, `:308`).
- Existing: the document's own `revision()` and `display_revision()` (`crates/motolii-edit/src/document.rs:392-404`: the second is exactly `shown()`, "what is shown changed and the document did not"), handed out with every reading.
- A call site that forgets it shows a stale window; the inside knows when it changed and is still not asked. See C3 for what SQLite already counts.

### B3. Flat things lie by the order, solid things by depth: the law and its mechanism (new; sure that it existed, likely as a re-make)
- Today: `filament/doc.h:184-223`. Every solid thing is given a nearest and furthest reach (a ball as wide as its widest: `:194-196`); every flat thing is moved along the eye's line to before the solids that lie before it and behind those after it, a hair (`1e-4`) nearer per place, and scaled by as much so the picture does not move (`:217-222`).
- Existing: the past wrote this law down, dated: "2D は世界に居ない(法 2026-09-12): 深度に参加せず、積み順だけで重なる … AE の「2D 層は 3D 世界の仕切り」" (`crates/motolii-render/src/compositor/sequential/accumulate.rs:172-180`: a run of flat layers is one plane, the solid group between resolves depth among itself, runs go over each other in stacking order). And the hair: "積み順ぶんだけ面に垂直にカメラ側へずらして … 1 段 0.02 px、視線に平行なので絵は動かない" (`crates/motolii-render/src/compositor/surface_scene.rs:760-779`, `two_d_stack_bias`, `draw_order`). And whether a thing lies by order or by depth is the document's (`LayerProjection` 2D / 2.5D / 3D, `crates/motolii-doc/src/store/attrs.rs:133-141`).
- Ledger: `ASSETS.md` §5 has `motolii-render` as "未確認". It was not read before this was written.
- Today's is weaker than the past's: a 3D file's reach is guessed from its width and height (`doc_half` knows no depth, `doc.h:620-628`), so a long or leaning solid is cut by a sheet that should lie behind it; two solids whose reaches overlap leave a flat thing between them nowhere to go (`need` beats `allow`, `:220`); a flat thing after a solid near the eye is pulled to within half a unit of the eye and drawn tiny there. The past's form (runs) needs no reach at all. Filament's own way to draw runs is C1.

### B4. What a new thing is and how large: reckoned in the shell again (new; likely, small)
- Today: `skin/book/lib/workspace/dock_make.dart:368`, `:382`: a Solid is a rect whose `size` is `m.size.$2 / m.point` and whose `wide` is `m.size.$1 / m.size.$2`, worked out in Dart from the inside's stage size and the inside's eye. The same reckoning for a shape's scale: `workspace.dart:352` and `:371-379` (scale x and y to `size` and `wide`, and back).
- Existing here: `doc_add` gives a new thing what it begins with, inside: the text's word and size were moved there one commit earlier, after the window judge (`doc.h:480-483`). The stage's size is the document's own (`doc.h:322-341`).
- Existing in the past: `new_layer_intents(layer, order, playhead, duration, fps, comp, kind, …)` (`ui/native/src/editor/create.rs:236-262`, called from `port.rs:103-109`): the inside is handed the composition's size and gives each kind its beginning, as one step.
- This is round 1's item 7 and round 2's B3 / B4 a fourth time, written in `d5b0290`, two commits after the form that ends it was taken. The next shell types the same division.

### B5. A second reader and teller of a way, in Dart (round 1's item 2, "half" in round 2; grown since; likely)
- Today: `skin/book/lib/workspace/desk_ease.dart:107-122` (`_said`, `_heard`). Since round 2: which kinds have a seat is typed a third time (`:142`, beside `:55` and `:115-119`); a way with no seat is drawn by two roads in one file (`:142`: 33 calls of `motolii_way_at` per tile; `:169-177`: 49 calls of `motolii_value`); and the inside lists its ways twice (`doc_ways`, `doc.h:566-571`; `motolii_way_names`, `filament/motolii.mm:159-166`, in the file whose first line says nothing is worked out there).
- Existing here: `interp_said` and `interp_words` (`filament/interp.h:168`, `:158`).
- Existing in the past: one reading hands the desk a kind, its numbers, its handles and its curve as 64 samples (`ui/native/src/snapshot.rs:22-33`); the desk's tiles are the host's kinds, "each drawn from its own samples" (`ui/lib/desks/ease/desk.dart:9`, `:30-48`); the desk never parsed words.
- What is true now: the second row of tiles does take the past's idea (the curve is the inside's), and no kind's formula was typed in Dart. But `motolii_way_at` knows only the nine, so a way defined in the document is drawn as a straight line on its tile (`motolii.mm:167-170`) although the comment at `desk_ease.dart:134-135` says its curve is the inside's own working; the tiles are remembered for the life of the handle (`:140`), so a way defined later has no tile; a tile picked tells the kind's name alone, so the stretch's own numbers are thrown away (as `Bounce` was in round 1).

## Seen, not counted

- **`motolii_keying`** (`doc.h:45`, `:519`; `motolii.mm:141`). This is the past's form (`Animate`, `crates/motolii-edit/src/document/edit.rs:15-19`; the command `animate`, `port.rs:263`; said back in every reading, `snapshot.rs:319`). Taken, under another name. Left over: two holders of the one truth, the inspector's `animateAll` and the inside's `keying`, kept in step by a callback (`skin/book/lib/sets/inspector/inspector_parts.dart:231`, `:242-244`); the inside is never asked. The past's `From` (keys from where the hand began) is not there.
- **Render** (`skin/book/lib/motolii.dart:136-145`, `workspace.dart:66-73`). The film is the kit's own, the other thread is Dart's `Isolate.run`. Not a re-make. The past's form was a job (start, progress, cancel, on a flattened copy of the document: `crates/motolii-render/src/export.rs:49-70`, `port.rs:293-294`); none of it was taken. Read from the code, not run: `motolii_write` goes through the one hand (`motolii.mm:229-237`), so while a film is written every other call waits, and the first click that makes a seat ask the inside (`wsTransform`, `stage_art.dart:47-52`) holds the window until the film is done. "The window does not wait" is true only while nobody touches it. Opening the library in another isolate and looking up one call is typed twice (`motolii.dart:27-35`, `:138-144`).
- **The stage's size on thing 0** (`doc.h:322-341`). As round 2: the past had one record (width, height, rate, length, ground), changed as a step (`port.rs:282`). Unsure. Still true: the rate is in no document; the shell counts 30 a second (`ws.dart:98`, used 15 times) and the inside draws and writes 60 (`motolii.mm:80`, `:233`), so a bar carried "by whole frames" (`timeline.dart:214`) lands on thirtieths.
- **`visible`** (`doc.h:22`, `:192`). A fact through the existing `doc_set` and the kit's `seen`. The past's was one switch call for hidden, solo and locked (`port.rs:239`); today solo and lock are still the shell's own sets and reach nothing (`timeline.dart:29`, `:374-375`).
- **`order`, `motolii_raise`**: as round 2, the past's form and word.
- **Thumbnails remembered in Dart** (`motolii.dart:20-37`): now named by the film's place and its last change, in the temp folder. The past remembered inside (`ui/native/src/editor/thumbnail.rs:7-19`). C8.
- **`solid_mesh`** (`filament/solid.h:66-115`): unchanged, glass-side, not counted. C7.
- **Small doubles inside today's own code**: where a point is seen, three times (`doc.h:637-641`; `motolii.mm:199-205`; Dart, right only at z 0, `workspace.dart:326-331`); a thing's placing, twice (`doc.h:224-226` and `:662-664`); `doc_frame` still does not know `rotate_x` / `rotate_y` (`doc.h:649-657`) though `doc_covers` beside it does, so a leaning thing is picked by a frame it does not have; undo in two places (`top_bar.dart:161`, `workspace.dart:105-106`), remove in two (`timeline.dart:154`, `workspace.dart:109-114`).

## C. A standard part would do it (not counted in G)

1. **Runs that go over each other in order** (B3). Filament draws views in the order they are given, into the same target, without clearing between: one `View` for each run of flat or of solid things, told apart by `View::setVisibleLayers` (the kit already sets layers, `kit.h:177`, `:221`). Or channels (`RenderableManager::setChannel`, `RenderableManager.h:726`: eight of them, each sorted by itself and drawn in turn), with `setBlendOrderAt` / `setGlobalBlendOrderEnabledAt` (`:1018`) for the flat things inside a run. No reach, no ball, no hair, nothing moved or scaled.
2. **The menu, and the same act from a button, a key and a menu** (`workspace.dart:77-119`, `top_bar.dart:161`, `timeline.dart:154`). Flutter's own `Actions` / `Intent` (one `UndoIntent`, one remove) called by all three. As it stands, by reading: the menu replaces the runner's own Edit menu (`macos/Runner/Base.lproj/MainMenu.xib` had Cut, Copy, Paste, Select All), and ⌘Z belongs to the menu, so in the inspector's text box it undoes the document, not the typing. Open and New: the system's own panel (`file_selector`: `openFile`, `getSaveLocation`) in place of a list of the `.motolii` files beside the open one, read with `listSync` on every build of the window (`workspace.dart:53-56`, from `:190`), the very wait that `d5b0290` took out of the dock.
3. **Has it changed** (B2). SQLite counts: `sqlite3_total_changes`, `PRAGMA data_version`, `sqlite3_update_hook`. One number handed out as `motolii_frames` is. For acts that nest: `SAVEPOINT` / `RELEASE` / `ROLLBACK TO` nest by themselves; today's is a counter over one `begin` (`doc.h:364-391`), which works.
4. **Folders read without waiting** (`dock_files.dart:144-179`, `:308`: a cache, a set of what is being read, a counter, a builder on the counter). Flutter's `FutureBuilder` / `StreamBuilder` over `Directory.list()`; `Directory.watch()` for a folder that changes. Or no lister: the system's chooser and a drop (`file_selector`, `desktop_drop`), which also never ask leave.
5. **A colour behind everything**: Filament's clear colour (`kit.h:134-136`) or ThorVG's ground (`filament/drawn.h:67-72`), as a fact of thing 0 beside `wide` and `high`. A Solid can stay a thing; a background need not be one that the shell sizes.
6. **A thing's box on the picture, and what is under the hand** (rounds 1 and 2, unchanged): `View::pick` (`View.h:895`), `RenderableManager::getAxisAlignedBoundingBox`. Three users lean on hand-made boxes now: picking, clip, and the ball of B3.
7. **A 3D file's triangles**: `FilamentAsset::getSourceAsset()` (`FilamentAsset.h:242`) hands out the cgltf data gltfio has already read, until `releaseSourceData`. One reader, not two.
8. **Thumbnails**: macOS's `QLThumbnailGenerator` (every kind of file; a per-OS road), Flutter's image cache for the remembering.
9. **The binding typed by hand** (now 42 lines, `motolii.dart:69-110`): `package:ffigen`. Not bound: `motolii_keys`, `motolii_define`, `motolii_ways`, `motolii_still`; `motolii_write` is looked up apart, in the isolate.
10. **Unchanged from round 2**: the one hand (GLib's main loop); a file's kind by the end of its name (`doc.h:464-474`, and the regex at `dock_files.dart:567`); the inspector's host finding changes by comparing whole maps (five mirrors now, with `_animating`).

## D. Non-findings: looked for a re-make, found a borrow

- **Hold, carry, let go**: the past's begin / update / commit / cancel (`port.rs:372`, `:235`, `verb.rs:19`), under the skin's words. All three shells' drags go through it; nothing is said twice; no shell works out a limit.
- **Acts**: SQLite's session and its own transaction; no column, no loop, no blob read back.
- **The inspector's pairing**: one table (`workspace.dart:314-325`).
- **`visible`, `order`**: facts through the existing `doc_set` / one change; shown by the kit's `seen` and `solid_seen`.
- **Screen-space refraction off, ambient occlusion on**: Filament's own switches, a line each (`motolii.mm:46`, `:50`).
- **The menu bar**: Flutter's `PlatformMenuBar`, `PlatformProvidedMenuItem`; no menu drawn by hand, though the skin has a drawn one on its shelf (`sets/panels/transport.dart:226`).
- **Render**: the kit's `film`, Dart's `Isolate.run`; what it says goes in the top bar's existing seat (`top_bar.dart:270`), no new widget, though `ToastCard` and `StatusBar` are on the shelf (`sets/foundations/feedback_parts.dart:305`, `:348`).
- **The Solid tile, Rectangle, Ellipse**: ThorVG's `Shape` through the existing `drawn`; the existing `_Make` tile.
- **The second row on the Ease desk**: the existing `_Presets` and `_PresetTile`; the curve by `interp_ease` inside; no formula in Dart.
- **A new text's word and size**: moved from the shell to `doc_add`.
- **Folders**: Dart's `Directory.list()`, the existing `FileNode` tree and tiles.
- **No new widget class** in the workspace since round 2. No undo stack in Dart. The GSAP table: not brought. One `LiveTexture`.
