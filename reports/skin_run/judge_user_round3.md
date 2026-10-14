# Judge with a user's eyes — round 3 (2026-10-14, 10:44–11:10)

Fresh judge, window only (app `lab_book`, page Workspace › Live, in the background: clicks, drags, scrolls). No code, no TASKS.md, no memory folder. After almost every act I compared the window with the file (`filament/out/skin.motolii`, tables `fact`, `step`, read only). About 25 minutes of hands-on work, some 300 acts, two restarts.

**One thing shaped the whole round: the menu bar could not be used.** From the background every menu item reports itself disabled — File › New Document, Open › any name, Render, Edit › Undo, Redo, Remove, and even `lab_book › Quit lab_book` (tried on three launches, after clicks, after drags, after setting focus). Since Quit is never really disabled, I take this as a property of a window that has never been in front, not as the app's own refusal; I do not score it as broken or as working. The consequence is real though: **I could not make a new document, open another one, or remove a thing.** All work was done in `skin.motolii` (a copy of it as it was at 10:44 is `judge_user_r3_skin_before.motolii` beside this report; the document now holds 20 things and 85 steps). Round 2's report was opened once early, for the word "menu" only (one grep), and read in full after my own pass through A to E.

Short reading: **what round 2 found dead is alive (the eye, the rocket's body, carrying from off the picture, unlinked Scale, Render), and undo is exact everywhere I pushed.** What is left: things that are put aside still catch the hand; the nearer / further rule between flat things and 3D things still surprises, in a new way; the stage size is still outside undo.

## Scores

| Item | Score | Why, in one line |
|---|---|---|
| A. picture | 7 | Live moving picture in the stage seat; play, stop, seek work; the clock kept real time with 12 and with 33 things (18 frames of a 30-a-second film in 0.56 s); survives three stage sizes and two restarts. 60 a second on the stage and window resizing could not be checked. The frame drawn round a turned or enlarged 3D file does not fit it (fox at 250 %, −38.6° about Y: body about three times as wide as its frame). |
| B. placing | 8 | Film, .jpg, three .glb (rocket twice, fox), text, Solid, rectangle, cube; glass on a cube and on the rocket file: all on the stage, on the timeline and in the file. The rocket now arrives whole. Glass on a photo is refused with a reason. Against it: removing was not reachable (menu), a text's words not typed; I did not place a .json myself (the one in the document plays). |
| C. values, keys, stretches | 8 | Place, size, turn about Z and Y, opacity, blend (four ways), Clip to below, carrying, keys by Animate all at two moments by number and by carrying, Bounce / Elastic / Steps chosen and seen moving (Steps: 1453 → 1297 in jumps), bars slid and cut, rows carried, the eye: all reach stage and file. Against it: nearer / further between flat and 3D things (finding 2); Animate all does not key where the thing was (finding 5); opacity on a glass cube does nothing without a word. |
| D. undo, reopening | 8 | Exact: 12 undos then 14 redos gave the same 82 facts; 22 undos while playing gave the same 82 facts; a restart kept every fact and the next undo and redo worked across it. Against it: the stage size is not a step, so undo after the chip takes back another act (finding 3); "a new document and back" not tried (menu). |
| E. breaking | 6 | No crash, no hang: 21 quick presses of Place, undo hammered while playing, edits and Render while playing, the Desktop folder opened. Refusals are told ("No seat yet", "a picture has no fill…", "Not on this frame"). But twice the file changed for a thing I could not see (finding 1), a rocket vanished without a word (finding 2), and undo once took back the wrong act (finding 3). |
| F. targets | 4 | Target 2 made in substance and written out (1920 × 1080, 60 a second, 15.7 s, sound track; frames checked) — but inside `skin.motolii`, and without typed words. Target 1 half laid on the tall stage. Target 3: one window. None in its own new document (menu). |

Not scored: G, H.

## Worst findings first

### 1. A thing put aside by its eye still catches the hand on the stage — and is carried, unseen
Steps (a): two rectangles at the centre; put both aside with their eyes (rows grey, not drawn, file `visible 0`). Pick the photo by its row; press on the photo at the centre of the stage and carry.
Seen: the inspector jumps to "Rect"; the invisible rectangle is carried (file: thing 17 gets `x 2.20, y −0.84`); the photo stays.
Steps (b): tall stage; `happy.mp4` put aside; pick the Ring by its row (its frame is drawn right of the stage); press inside that frame and carry it onto the stage.
Seen: the ring does not move. The file shows the step went to the invisible `happy.mp4` (`x 2.34 → 0.34`), which lay under the pointer. No word on screen either time. I had to drag the ring's Position X number over 1100 pixels of screen instead.
Expected: a thing that is not drawn is not picked by a press on the stage; and a press inside the frame of the picked thing carries that thing.
Check: eye off on thing A lying over thing B; press-carry at that spot; A's `x`, `y` must not change.

### 2. Nearer / further between flat things and 3D things: a rocket in the top row vanishes behind the rectangle below it
Steps: the stock document (its glass Ring has Z = 1.0, shown as 335.9). Playhead at 6 s. Create › Rectangle, Scale 400, Fill blue. Media › saturn_v.glb, Place (it is row 1, the rectangle row 2), Scale 400 to 870, Fill pale. Clip to below on or off.
Seen: only blue. The rocket is not drawn anywhere (picture `judge_user_r3_4.jpg`). Put the rectangle aside: the rocket is there. Bring it back, put the **Ring** aside (twelve rows lower, elsewhere on the stage): the rocket appears over the rectangle (picture `judge_user_r3_5.jpg`). Putting the fox aside changed nothing.
Same family, seen earlier in the round:
- "Text" (row 5) is drawn over the uncoloured rocket in row 1, while the title (row 9) is drawn under it.
- A stage-sized Solid in the bottom row covers the After Effects background (Z −1) — until an uncoloured rocket file is placed: then the background is in front of the Solid. Undo: Solid in front again. Redo: background again. Twice (picture `judge_user_r3_1.jpg`). Nothing in the timeline tells any of this.
Expected: the top row is the nearest, as the manual says; or the row of a thing that stands out of the order says so.
Check: ring with Z 1 + rectangle + 3D file placed last; count the file's pixels inside its frame: not zero.

### 3. The stage size is not an undo step: the next Undo takes back something else
Steps: cut a bar's tail (file: `duration 11.24`). Press the size chip (stage 1080 × 1920; `step` stays at 32). Press Undo.
Seen: the stage stays tall; the bar's cut is gone (`duration` removed). No word.
Expected: undo takes back the last thing I did, or the chip is plainly outside undo. Also still true: the chip sends the playhead to 0.

### 4. The menu bar from the background
All items of File and Edit, and Quit, report disabled (see top). Not scored. If a user at the keyboard sees them alive, this is only a testing trap — but then the round's instructions ("the menu bar works in the background") did not hold here, and New Document, Open and Remove have no other door in the window: no button, no shelf entry. A window whose only way to a new document is a menu I could not reach left F and half of D untested.

### 5. Animate all forgets where the thing was
Steps: cube carried to (1609, 223) with Animate all off. Animate all on at 0 s. Ruler to 4 s. Carry the cube to (735, 821).
Seen: file has `x`, `y` only at 4.03 s; the value at 0 is gone. The cube does not travel: it sits at the new place for the whole film. I had to go back to 0 and carry it again.
Expected: the first key keeps the old value as the start (or switching Animate all on keys what is there), so "on, move the time, move the thing" gives a move.
Beside it: turning Z with Animate all on also keyed `rotate_x` and `rotate_y` at 0; with Animate all **off**, carrying a keyed thing still writes a key (0.0, Linear) 0.07 s before the next one.

### 6. Smaller things, all seen
- **Opacity on the glass cube**: dragging the number changes nothing, in the window or the file, without a word. On the photo it works (85 %).
- **Blend on the cube** offers one way only ("Normal"); on the photo four.
- **Desktop folder**: opening it kept the window answering (play started right after). The list says "Nothing matches ''", which is not the reason.
- **Folders open on one click**: a double click falls two levels deep (third_party → libassimp).
- **The Scale link, once opened, stays open for the next thing made**: my next rectangle came out 400 × 100.
- **With the link open, Scale Z follows Y.**
- **New files are saved with whole disk paths** (`/Users/…/out/vid/happy.mp4`), the stock ones with paths from the document (`../vid/popcat.mp4`).
- **A new Solid is named "Rect"**, lands on top and covers everything until its row is carried down; a new rectangle has the same pale colour as a new Solid (invisible on it).
- **Render** says nothing while it writes ("Saved"), then "Written: skin.mp4"; pressing it while playing stops the play without a word.
- **Ease tiles, second row** (Elastic, Random, Steps): single clicks lit the tile and chose nothing, seven times; a click straight after another click took. Random never took. Possibly my tool; not counted.
- **Glass cube seen straight on over a pale rectangle** is invisible until carried or turned (as in round 2).
- Clip to below on a thing whose window is put aside: the thing vanishes (reasonable, untold).

## Round 2's five worst findings

| # | Round 2 finding | Now | What I saw |
|---|---|---|---|
| 1 | The eye says hidden, the thing stays on the stage | **Mended** | Ring, rocket, rectangles, clips, fox: each leaves the stage, the row greys, file `visible 0`; undo and redo carry it. New of the same family: the unseen thing is still picked and carried (finding 1). |
| 2 | A placed rocket file arrives without its body | **Mended** | Placed twice with no fill: whole, black and white, at 316 %. |
| 3 | Row order does not hold between flat and 3D things | **Changed, not mended** | A flat rectangle above the rocket or the cube now covers it. But with a 3D thing that has its own Z in the document (the stock ring), a rectangle in row 2 hides a rocket in row 1, a text in row 5 is over a rocket in row 1, and a Solid and a background swap when a rocket is placed (finding 2). |
| 4 | A thing off the stage cannot be carried back | **Mended, with a hole** | Tall stage, cube off to the right: press its frame, carry, it lands on the stage, file follows. Fails when an unseen thing lies under the pointer (finding 1b). |
| 5 | Scale with the link off: Y drags X | **Mended** | X 197.2 stayed, Y went to 49.9; file `wide 3.95`, `size 0.499`. |

Also from its list: Render now writes the film and says so; Elastic has a seat (13 tiles now); the strip under the stage says the true size (S 170 %, 322 %, 779 %); a glass refusal is told. Still there: the stage size is not a step and sends the playhead to 0; the frame of a turned 3D file; the Animate all switch sits at another x for each name.

## What held (so it can stay a check)
- Restart: 66 facts the same before and after; then Undo took back exactly the last step, three more in one burst, five redos brought all back (fifth harmless).
- 12 undos / 14 redos over eye, opacity, blend, clip, carries, number drags, bar slide, bar cut, ways: file identical (82 facts).
- 21 presses of Place in seven bursts: 21 things, 21 steps; 22 undos while playing: back to the 82 facts, still playing.
- Carrying a thing back to the same spot gave the same number to 14 places (`1.93090751340281`).
- Steps way: `x`, `y` at the first key say `Steps`; on the stage the cube jumps.
- Bar: slide → `start 2.07`; tail cut → `duration 11.24`; the rendered film shows the photo only between.
- Render with 33 things: `skin.mp4`, h264 1920 × 1080, 60 a second, 940 frames, AAC track; stills at 3 s and 9 s match the stage.

## Not tried
- File › New Document, Open, Render; Edit › Undo, Redo, Remove (every menu item reports disabled from the background, Quit too).
- Therefore: acts in a new empty document; a new document and back; removing a thing.
- Typing: a text's words, numbers, search (keyboard does not reach a background window).
- Resizing the window; 60 pictures a second on the stage.
- Placing a .json myself; Ellipse, Ring, Gem tiles; Add and Screen; the Ease handles; Hold, Cyclic, ElasticSteps; Rotation X; Ghost, Depth, Parent, Space 2.5D / 3D, Camera, Animate and Export tabs.
- Documents and Downloads folders (one of the three was allowed; I opened Desktop).
- Batch calls of my tool were refused every time ("user interrupt"); single calls were used. Not counted.

## What stopped each target
All three were tried inside `skin.motolii`, putting other things aside with the eyes, because no new document could be made.

**Target 1 (`edit_v5.mp4`)** — picture `judge_user_r3_3.jpg`. Reached on the tall stage: the After Effects background, the title, a keyed cat, the fox placed from eight folders down, enlarged to 250 % and turned −38.6° about Y, the glass ring brought onto the stage (it refracts the fox), a "Text". Stopped by: (1) the ring could not be carried from off the stage because an unseen clip lay under the pointer (finding 1); number drags instead. (2) After the size change the title is cropped ("IOTOL") and every thing has to be brought back one by one. (3) The fox's frame does not show its real size, so laying it out is guesswork. (4) The cat and the text lie in front of the fox whatever the rows say (finding 2). (5) Two typed lines: not tried. The photo with the cat on it was not laid.

**Target 2 (cat puppet scene)** — picture `judge_user_r3_2.jpg` (three stills of the written film, 1 s, 4 s, 7 s). Made: a Solid behind, title, `popcat`, `vest` (its own Bounce / Elastic keys), `happy` carried right, `dog` placed at 2.07 s and travelling by my two keys (2.07 s → 6.13 s). All keyed by themselves. Render pressed: `skin.mp4` written, frames agree with the stage. Missing: typed words; its own document.

**Target 3 (rocket collage)** — pictures `judge_user_r3_4.jpg` (stopped) and `judge_user_r3_5.jpg` (after the ring was put aside). Reached: a blue rectangle 400 %, the rocket filled pale at 779 %, tilted 20°, Clip to below on: white clay on blue, one window. Stopped by: (1) the rocket was invisible behind its own window until I found, by trying eyes one by one, that the ring elsewhere in the document was the cause (finding 2). (2) Time: each further window is one more rectangle and one more rocket laid by number drags. (3) Cyrillic words: not tried. No grey rectangle, no thin bars over the rocket, no second window.

## Pictures
- `judge_user_r3_1.jpg` — an uncoloured rocket placed: the background is in front of the stage-sized Solid.
- `judge_user_r3_2.jpg` — target 2: three stills of the written film.
- `judge_user_r3_3.jpg` — target 1 try on the tall stage.
- `judge_user_r3_4.jpg` — rocket in row 1, blue rectangle in row 2: only blue.
- `judge_user_r3_5.jpg` — the same with the ring put aside: target 3 start.
