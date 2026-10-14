# Judge with a user's eyes — round 2 (2026-10-14, 09:53–10:13)

Fresh judge. I used the window only (app `lab_book`, page Workspace › Live), in the background, by clicks, drags and scrolls. No code read, no TASKS.md, no memory folder. After almost every action I compared the window with the document file (`filament/out/skin.motolii`, tables `fact` and `step`, read only). About 19 minutes of hands-on work, some 250 actions. I restarted the app twice on purpose (item D). The app is left open; the document now holds 17 things and 63 steps.

The first judge's report was read only after my own pass through A to E.

Short reading: **the inside is joined for real now. Everything I changed reached the file and came back after a restart, and undo never lost or mixed a step.** What is left is of another kind: a few controls that look alive and are not, a 3D file that arrives without its body, and an order rule (flat things against 3D things) that the timeline does not tell. No target was finished.

## Scores

| Item | Score | Why, in one line |
|---|---|---|
| A. picture | 7 | Live moving picture in the stage seat; play, stop, seek work; the clock kept real time with 30 things (4.20 s of film in 4.14 s of wall time); survives three stage sizes and two restarts. 60 a second and window resizing could not be checked; a placed rocket file is drawn without its body; selection frames do not fit turned 3D things. |
| B. placing | 8 | Film, .jpg, .json, text, two .glb, rectangle, cube, glass on a cube and on a 3D file: all on the stage and on the timeline, all in the file. Against it: the rocket file shows only its engine bells until it is given a colour; a font file placed from Media becomes its own file name across the whole stage; removing was not tried (keyboard) and the eye on a row does not hide. |
| C. values, keys, stretches | 8 | Place, size, turn about X, Y and Z, opacity, blend, Clip to below, carrying on the stage, keys at two moments that travel, eight Ease seats and a dragged handle: all reach stage and file. Against it: Elastic is drawn right but cannot be chosen (8 seats, not 9); with Scale unlinked a drag of Y changes X too; a thing off the stage cannot be carried; the strip under the stage gives other numbers than the inspector. |
| D. undo, reopening | 9 | Held everywhere I pushed: same value changed three times and taken back one by one, 18 undos in a row, bars slid and cut, rows reordered, redo after a restart, three stage sizes. The stage size itself is not a step. |
| E. breaking | 6 | No crash, no hang (20 clips placed in 7 quick bursts, play hammered, edits and Place while playing, undo at once). Refusals are mostly told ("No seat yet", "Not on this frame"). But I found one control that says a thing is hidden while it is drawn, one button that does nothing without a word, and several silent surprises (below). The Desktop / Documents / Downloads hazard is counted here as told; I did not trigger it. |
| F. targets | 3 | None finished. All three were begun and every kind of thing in targets 1 and 2 except typed words is on the stage; target 3 got a real start (blue rectangle, pale rocket cut by it, a thin bar). Pictures: `judge_user_r2_1.jpg`, `_2.jpg`, `_3.jpg`. |

Not scored: G, H.

## Worst findings first

### 1. The eye on a timeline row says "hidden"; the thing stays on the stage
Steps:
1. Any document with a 3D ring or the rocket visible at the playhead.
2. In the timeline, click the eye icon of that row (seen at about x = 363 of the 1427-wide window; rows `saturn_v.glb` and `Ring`).

Seen (three times, once after a restart): the row's name and its bar turn grey and the eye is dimmed. The thing is still drawn on the stage, unchanged. The file gets no fact and no step. After a restart the row is back to normal (the state lived only in the window). Picture: `judge_user_r2_4.jpg` (row 7 grey, rocket on the stage).
Expected: the thing disappears from the stage, or the eye is shut like the other unjoined seats. This is the one place where the window and the stage flatly disagree. It also leaves the user with no way at all to put a thing aside (removing needs the keyboard).
Check: click eye → pixel difference of the stage must not be zero, or the row must not change.

### 2. A placed rocket file arrives without its body
Steps: Media › Project › out › models › saturn_v › `saturn_v.glb`, Place (playhead at 0).
Seen: a row on the timeline and a tall narrow selection frame on the stage; inside it only five small white cones at the foot (the engine bells). I scaled it to 322 % and turned it 30° about X: the cones move and turn, the body is still missing, the background shows through. Only after Colors › a pale swatch › Fill did the whole rocket appear (white). `Fox.glb` placed later showed its own colours at once. No word on screen.
Expected: a 3D file is drawn whole as it is, or the window says what is missing.
Check: place the file, count non-background pixels inside its frame before any fill.

### 3. The row order does not hold between flat things and 3D things
The manual says the top row is the nearest thing. Steps:
1. Rocket (filled, scale 1124 %) at the centre, lower in the order.
2. Create › Rectangle (it becomes row 1, the top), make it a thin bar, leave it at the centre.

Seen: the bar is hidden wherever it overlaps the rocket; only the part over the flat blue rectangle shows. The same with the placed `.json` (invisible at the centre, visible once carried aside), with the text "Text", and with the font-file text (behind the fox and the rocket, in front of the flat rectangle). Dragging the bar's Position Z to 300 brought it in front — and made it larger and shifted on the stage (the strip then says P 1148 where the inspector says X 1120.3).
Expected: the top row is in front, as told; or the timeline shows that a 3D thing stands out of the order.
This is what a collage of flat bands over a 3D thing (target 3) and a cat passing in front of a 3D thing (target 1) both need.

### 4. A thing that lies off the stage cannot be carried back
Steps: wide stage with the ring at the lower right; press the size chip (stage becomes 1080 × 1920); the ring now lies right of the stage, its frame is drawn there. Hold the frame at its centre and carry it onto the stage.
Seen: nothing moves, the file is unchanged (`x 1.9`), no word. I had to drag the Position X number twice over 700 pixels of screen (1674 → about 600). On-stage things carry fine (photo, rocket, fox, .json: one step each, file agrees).
Around it: after a change of stage size every thing keeps its old place, so on the tall stage the title is cropped to "IOTOL" and half the things are outside. The chip itself is not an undo step, and it sends the playhead to 0 and Animate all to Off.

### 5. Scale with the link off: a drag of Y changes X as well
Steps: Create › Rectangle. Click the link mark beside Scale (it opens). Drag Scale X 400 px right (reads about 500). Drag Scale Y 85 px left.
Seen: Y reads 17.8 and X has dropped to 89.3. File: `size 0.178`, `wide 5.008`. With the link on, the X drag before that gave 400.6 in all three fields.
Expected: with the link open, a drag of one field leaves the other alone. As it is, a thin wide bar takes two passes and some arithmetic.

### 6. The strip under the stage tells other numbers than the inspector
- "S 100%" always: with the rocket at 322.4 and at 1124, the ring at 110, the cat at 190, the photo at 200.2.
- P differs for any thing with depth: ring, inspector 1674.5 / 1378, strip "P 1901, 1462"; bar at Z 300, inspector 1120.3, strip 1148.
- R shows Z only; the rocket turned 30° about X reads "R 0°".
Expected: one truth, or no strip.

### 7. Smaller things, all seen
- **Render** (top bar): a click does nothing and says nothing.
- **A font file placed from Media** (`Arial Unicode.ttf`, Place) becomes a text that reads "Arial Unicode.ttf", black, wider than the stage (only "rial Unicod" fits). Create › Text gives a modest "Text" instead. Two doors, two results.
- **Fonts shelf with a text picked**: "No seat yet". Honest, but then a text's font cannot be chosen anywhere I found; the inspector's Size reads "48 px" for a text whose file says `size 0.5` and whose Scale reads 50.
- **A change while playing stops the play** without a word (the change is saved; see the mended list).
- **Selection frame of a turned 3D file** is the unturned front box: the fox at −23.5° about Y is twice as wide as its frame; the rocket's frame does not follow its turn about X.
- **Ease desk**: eight seats (Linear, Ease, In, Out, In-out, Back, Bounce, Hold). An Elastic stretch from the document is drawn as a true ringing curve with "ELASTIC 1.5 0.3 0.35", but there is no seat to choose Elastic for a new stretch. With Bounce or Elastic picked the Bezier boxes still show numbers (0.25 0.10 0.25 1.00 / 0 0 1 1).
- **New things** land at the centre of the stage and start at the playhead, running to the end; twenty clips make one pile.
- **Media**: a newly opened folder keeps the scroll place of the one before (it opens half way down); `.glb` is filed under "Docs"; `.json` tiles show a generic mark; the tree says Project 9 where the list says All 15; once a text exists, the system folder "Supplemental" appears in the Project tree and pushed `vid` down so my click on `vid` opened `out`.
- **Animate all** switch sits at a different x for each name length (1100 for "Cube", 1125 for "happy.mp4"); it is small and easy to miss (my first click missed it and my "key" became a plain change).
- **Glass on a cube seen straight on, over a flat rectangle**, is invisible: the square just vanishes until the cube is turned.
- Inspector rounds what the file keeps (15 / 17.5 shown, 15.03 / 17.54 saved); a turn about Z shows 12 where the file says −12.03.

## The first judge's five worst findings

| # | Round 1 finding | Now | What I saw |
|---|---|---|---|
| 1 | A change made while playing is shown but never saved; the next undo takes back another thing | **Mended** | Play, drag Scale 40 px left on `happy.mp4`: play stops at once, Scale reads 59.9, file gets `size @10.33 = 0.599` and one step (43 → 44). One undo takes back that and nothing else (the count of things stays). A Position X drag while playing, earlier: same (saved, one step, play stopped). The play stopping is not told (finding 7). |
| 2 | Rotation X and Y show numbers and do nothing | **Mended** | Cube, rocket and fox turn on the stage; file has `rotate_x`, `rotate_y`; undo and redo carry them. |
| 3 | The window states sample values as the picked thing's | **Changed, mostly mended** | Text box reads the text's words, Font reads "Arial Unicode.ttf", Fill "None", EFFECTS 0, Scale X Y Z agree, "Not on this frame" is right. Still there: the strip's "S 100%" (finding 6), a text's Size "48 px". New of the same family: the eye (finding 1). |
| 4 | Effects "Add" and Fonts "Apply" say done and do nothing; Render and the size chip dead | **Changed** | Effects and Fonts now say "No seat yet" (also with a text picked). The size chip works (wide → tall → square → wide; the document survives, 24 steps kept, undo works after). Render still does nothing without a word. |
| 5 | The first key rewrites the value "before", so a keyed move does not happen | **Mended** | `happy.mp4`, Animate all on, X dragged at 8:16, Y dragged at 11:15: file `y @8.53 = 0`, `y @11.5 = −0.503`, x and z keyed at both. At 10:00 Y reads 1109 between 960 and 1260: it travels. Same with the cube (945 → 1016, seen moving while playing). |

Also checked from its lower list: the lower rows of the Blend list (now four words, Multiply chosen, on stage and in file); glass on a 3D file (now works: a glass fox, `fill glass`, one undo takes it off); dragging on the stage (now works on-stage); film tiles (now show each clip's own picture); order of things (rows can be carried up and down, file `order` follows, undo restores); the Elastic curve (now drawn true).

## What held (so it can stay a check)
- Clock: two captures 4.14 s apart with 30 things: 00:01:07 → 00:05:13 at 30 a second (4.20 s).
- Twenty presses of Place in seven quick bursts: 20 things, 20 steps. Then 18 undos in six bursts of three: 13 things left, `step` 48 with 20 undone, window and file agree.
- Same value three times: Rotation Z −10, −20, −30; three undos gave −20, −10, none.
- Bars: slide (Text 10.6 → 6.2 s), head cut (Rect → 12.37 s; vest → start 2.07), tail cut (popcat → duration 13.57), row carried to the top (`order 10`). Three undos restored all three, three redos after a restart brought them back.
- Restart twice: every fact the same (126 facts, 17 things before and after), undone steps still redoable.
- Keys and ways: Bounce, Ease, a dragged handle ("CUSTOM", `Bezier 0.73 0.03 0.25 1.0` in the file, one step each); three undos to Linear, three redos back, a fourth redo harmless.
- Clip to below: the rocket shows only inside the frame of the thing just below it (first the photo, then the blue rectangle after I carried that row under it).
- Create tiles with no inside (Shape, Confetti...) say "No seat yet" instead of a button.
- The film's length follows the bars (15:20 → 18:01 after two bars were slid later).

## Not tried
- Typing: a text's words, a number, the search boxes (keyboard does not reach a background window). Both targets with words stop here for me; not scored as broken or as working.
- Removing a thing with Delete or Backspace (same reason). The app's Edit › Delete menu item is disabled. Undo of a placement does take the thing away.
- Resizing the window (cannot be done to a background window).
- 60 pictures a second on the stage (I can read the clock, not count drawn frames; the bar says 30 fps).
- The folders Desktop, Documents, Downloads (told not to).
- Space 2.5D / 3D, Depth, Parent, Ghost, Camera, the Animate and Export tabs, the Desk tiles other than Ease.
- Ellipse, Gem and Ring tiles of Create (a ring was already in the document); Add and Screen of the Blend list; Hold.
- Picking by a click on the stage where things overlap (I picked by rows).
- Whether a film cut at its head shows the right moment of the clip (I saw the bar and the file change, not the frame).
- Two tool hiccups on my side (a batch call refused each time; two clicks refused once) are not counted against the window.

## What stopped each target

**Target 1 (`edit_v5.mp4`).** Reached, on a 1080 × 1920 stage made with the chip: the After Effects background, the title, the fox turned about Y and scaled, the glass ring brought down (it refracts the fox), the photo scaled and tilted, three keyed `happy` cats plus `popcat` and `vest`, a second text. Picture `judge_user_r2_1.jpg`. Stopped by:
1. Two typed lines (one Japanese): typing not tried.
2. After the change of size everything had to be brought back by long number drags; off-stage things cannot be carried (finding 4). I ran out of patience before the layout looked like the film: the cats ended behind the fox and the photo.
3. The cat passes in front of the photo and beside 3D things in the film; flat against 3D does not follow the rows (finding 3), so it needs Z by hand, which also changes size and place.
4. Nothing can be put aside while working (eye dead, no removing without keyboard), so the rocket from my other tries stayed in the picture.

**Target 2 (cat puppet scene).** Reached: a background; five keyed cats (`happy` × 3, `popcat`, `vest`); one `happy` travels by my two keys, `vest` keeps its Bounce and Elastic stretches, one cat placed up left, two bars slid to come on at 2.07 s and 3.7 s. Picture `judge_user_r2_2.jpg`. Stopped by: typed words not tried; no new or empty document from the window, so the scene shares the stage with the fox, rocket, ring and photo of target 1 and I could not hide them (finding 1); every cat starts at the same centre spot at full length.

**Target 3 (rocket collage).** Reached: a blue rectangle 400 % wide and high, the rocket filled pale at 1124 %, turned 30° about X and cut by the rectangle with Clip to below after I carried the rectangle's row under it, one thin pale bar brought in front with Z. Picture `judge_user_r2_3.jpg`. It reads as white clay on blue: a true start. Stopped by:
1. The rocket arrives without its body (finding 2) — found by luck that Fill cures it.
2. Flat bands over the rocket need Z by hand (finding 3); bands are fiddly to shape (finding 5).
3. The collage has several windows each cutting the rocket differently: that would be one more rocket and one more rectangle per window, each pair laid by number drags; I made one.
4. Cyrillic words: typing not tried.
5. The reference looks up the rocket from close below with strong depth; I found no camera seat to try (Camera not tried), only turns of the thing.

## Pictures
- `judge_user_r2_1.jpg` — target 1 try on the tall stage.
- `judge_user_r2_2.jpg` — target 2 try: cats apart, bars slid, keys on one cat.
- `judge_user_r2_3.jpg` — target 3 try: blue rectangle, pale rocket clipped by it, bar in front by Z, glass ring.
- `judge_user_r2_4.jpg` — the eye: row 7 `saturn_v.glb` grey, the rocket still on the stage.
