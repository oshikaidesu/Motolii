# Judge with a user's eyes — round 1 (2026-10-14, 08:53–09:12)

I used the window only (app `lab_book`, page Workspace › Live), in the background, by clicks, drags and scrolls. No code read. After each action I compared what the window showed with what the document file held (`skin.motolii`, tables `fact` and `step`, read only). About 19 minutes of hands-on work; the app was restarted twice by me (on purpose, for item D) and is left open.

Claim under test: "pass lines 1 to 4 of the sheet are met". My reading: **A, B and D are mostly true, C is half true, and the window still tells the user several things that are not so.** F is far away.

## Scores

| Item | Score | Why, in one line |
|---|---|---|
| A. picture | 7 | A live moving picture sits in the stage seat, play / stop / seek work and the clock keeps real time (30 things: one stumble of about 3 frames); 60 a second and resizing I could not check; one placed 3D file stayed invisible until I changed its scale. |
| B. placing | 7 | Film, photo, AE picture (.json), .svg, text, .glb, cube and glass all land on the stage and the timeline; but a new text says its font file's name at giant size, everything lands on one spot in an order I cannot change, glass on a file is refused with a cut-off reason, and removing was not tried. |
| C. values, keys, stretches | 6 | Place, size, turn about Z, opacity, keys, eight Ease seats, a dragged handle and Blend reach the stage and the file; Rotation X and Y do nothing while showing numbers, a first key rewrites the "before" value so the move I keyed did not happen, half the Blend list cannot be clicked, a 9th way has no seat. |
| D. undo, reopening | 8 | Undo and redo held through the same value changed three times, joined steps, 20 placements and two restarts; but a change made while playing never becomes a step, so the next undo takes back something else. |
| E. breaking | 4 | No crash and no hang in anything I did (20 clips at once, play hammered, place-then-undo while playing, 32 things); but I found six ways to end in a state where the window says one thing and the document another, with no word on screen. |
| F. targets | 2 | Neither target 1 nor 2 could be finished; the parts of target 1 can be placed, target 2 reached "three cats on a background, two of them moving"; target 3 cannot be begun. |

Not scored by me: G. (H was not asked of me; for what it is worth the seats and the look match `skin_window.jpg`; what changed is the content — and where the content is still the sample's, see finding 3.)

## Worst findings first

### 1. A change made while playing is shown but never saved, and the next undo takes back a different thing
Steps:
1. Pick `popcat.mp4` (click the big cat on the stage). Inspector Scale reads 190.
2. Press play.
3. While it plays, drag the Scale X number 40 px to the left.
4. Press pause.
5. Press undo once.

Seen: after 3 the cat is smaller on the stage and Scale reads 149.9. The file still has `size 1.9` for that thing and no new row in `step` (47 before, 47 after, checked twice, also after pausing). After 5 the cat jumps back to 190 **and** the last `waving.mp4` I had placed earlier disappears (layers 30 → 29, `step` undone 1 → 2).
Control: the same drag while paused is saved at once (`size 1.499…`, one new step).
A drag of Rotation Z in the same playing state left no trace at all (not on the stage, not in the file).
Expected: either the change is refused while playing, or it is a step like any other. As it is, a restart would also lose it.

### 2. Rotation X and Rotation Y take a drag, show the number and a key dot, and do nothing
Steps: pick `saturn_v.glb` (timeline row), Animate all on, drag Rotation Y 60 px right, drag Rotation X 40 px right.
Seen: inspector reads Rotation X 20, Y 30.1, each with a key dot. The rocket on the stage does not turn. The file gained only `rotate @8.8 = 0.0` (a key of the Z turn, value zero); nothing for X or Y. The timeline header still says "0 keys". Picture: `judge_user_r1_4.jpg`.
Expected: the 3D thing turns, or the two fields are shut. This alone stops target 1 (the fox turns) and target 3.

### 3. The window states sample values as if they were the picked thing's
All seen with nothing done by me to cause them:
- Text thing "Arial Unicode.ttf": the Text box is **empty** while the stage shows the words "Arial Unicode.ttf"; Font reads "Inter Display" (the file says Arial Unicode); Size reads "48 px"; Fill read "#D9D2C3" while the text was drawn black and the file had no fill. Picture: `judge_user_r1_1.jpg`.
- A photo: Fill reads "Glass" right after a glass cube was picked, and "#FF7A3C" after a restart. A photo has no fill.
- Every thing: "EFFECTS 2 — Blur On (Radius 24 px), Kaleido On". Nothing is blurred.
- Scale Z: a new cube showed 400.6 (the rocket's number); after linked Scale drags on a film it read 78.9, then 62.2. The file has one `size` only.
- The strip under the stage always says "S 100%" (rocket at 400.6, cat at 149.9).
- A thing placed at the playhead shows "Not on this frame" while it is plainly drawn (Fox.glb).
Expected: a field that is not joined is empty or greyed; a joined field shows the document.

### 4. Effects "Add" and Fonts "Apply" say done and do nothing
- Effects shelf: pick Glow, press Add with the text picked. Button turns to "Added", the row gets an "ON" badge. Stage unchanged, file unchanged, no step. Picture: `judge_user_r1_2.jpg`.
- Fonts shelf: pick Helvetica Neue with a **photo** picked, press Apply. Button turns to "In use", the row gets "Used", the count "Used 1" becomes 2. Nothing in the file.
- Top bar "Render": a click does nothing, says nothing. The "1920 × 1080" chip: same.
- Colors shelf: the count "Used here" goes up by one on every press of Fill, also when the fill was refused (10 → 12 → 13).
The Create shelf does this right: Shape and Rectangle show "No seat yet" instead of a button. The other shelves need the same.

### 5. The first key rewrites the value "before", so a keyed move does not happen
Steps: pick `happy.mp4` (at 960, 540, no keys). Animate all on. Playhead at 8:06: drag Position X to 559. Playhead at 11:07: drag Position Y to 840. Go to 8:24.
Seen: two diamonds on the row (8:06 and 11:07), header "3 keys", and a third diamond at 0 s floating left of the bar (the thing starts at 2:29). At 8:24 Y already reads 840.6: no travel. File: `y @0 = -0.895` and `y @11.23 = -0.895`; no y at 8.2; `x` the same number at 0, 8.2 and 11.23; `z` at 8.2 and 11.23.
The same happened with the cube (`x @0` took the value of the first key).
Expected: with two diamonds on the row, the thing goes from where it was at the first to where I put it at the second.

### 6. The lower rows of the Blend list cannot be chosen
Steps: pick the photo, scroll the inspector until the Blend row is about two thirds down, click the Blend row, click "Difference".
Seen (twice, once after a restart): the list closes, Blend stays as it was, and the Desk tile under that spot ("Depth") lights up. "Overlay" failed the same way once. "Multiply" (higher in the list) works: stage and file change.
Expected: every row of an open list can be clicked.

### 7. A placed 3D file was invisible until its scale changed
Steps: Media › Home › motolii › filament › out › models › saturn_v › `saturn_v.glb`, Place (playhead 2:29).
Seen: a row on the timeline and a narrow selection frame on the stage, no rocket. I moved it by X and by Y to a clear spot and moved the playhead to 5:05: still nothing, for about half a minute. On dragging Scale to 400 it appeared. `Fox.glb` placed later appeared at once. Seen once; not repeated.

### 8. A refusal whose reason cannot be read
Colors › Glass › Fill with `Fox.glb` picked: nothing changes (right), and the foot of the shelf says "the solid (~/motolii…" — the path fills the line and the reason is cut off. With nothing picked the same foot says "Nothing is picked" (good) — and keeps saying it after a thing is picked.

### 9. Smaller things
- A new text is the font file's name ("Arial Unicode.ttf"), black, wider than the stage, and drawn **behind** the films and the photo. A newer thing is sometimes in front (rocket over text), sometimes behind (photo behind the cat placed before it). I found no way to change the order.
- Everything placed lands at 960, 540 at 100 %. Twenty clips make one pile. Dragging a thing on the stage does not move it (a click picks it; a drag does nothing).
- Film tiles are sample art, not the films: nine cat clips cannot be told apart before placing.
- The Media tree shows five rows; a click on `vid` under Project opens `out` in the tree instead; the list keeps its scroll place when a folder or a filter changes; counts disagree (Project "10" in the tree, "All 16" in the list; out "482" and "500"; Create "24 kinds" and "All 27"). A .glb is filed under "Docs". An empty folder says: Nothing matches "".
- Desktop, Documents and Downloads sit in that tree one stray click from the folder I wanted (I did not click them; counted under E as told).
- Desk: a stretch whose way is Elastic is named "ELASTIC" over a straight line with handles 0, 0, 1, 1. I count eight seats (Linear, Ease, In, Out, In-out, Back, Bounce, Hold), not nine; I found no seat for Elastic.
- After choosing Bounce with the playhead still, the stage moved but Position X and the selection frame kept the old number until the playhead moved.
- New placements are written with the whole path from the disk's root; the things already in the document have paths beside the file.

## What held
- Clock against the wall: six captures 1.2 s apart, 35 or 36 frames each at 30 a second with 12 things; with 32 things one gap of 33 then 38.
- Twenty presses of Place in a row: 20 things, 20 steps, alive.
- Undo: 30 → 20 → 10, redo twice → 30; seven undos back through blend, ease (three joined values as one step), keys, fills; eight redos (one past the end, harmless) gave every fact back.
- Reopened twice: all things, keys, ways, blend, glass there; the first undo after reopening took back the last step.
- Keys: the cube moved between its keys; Bounce changed where it was at the same frame (X read 1364 at frame 168; Linear would give about 981 there, by my sum; at frame 199 Linear had read 1152); a dragged handle became "Custom" and a Bezier in the file, one step.
- Opacity and a colour on a text: on the stage and in the file. Glass on a cube: on the stage and in the file.
- Unpicking: "Nothing selected. Pick a layer on the Stage or in the Timeline."

## Not tried
- Resizing the window (cannot be done to a background window).
- Typing: the words of a text, searching in a shelf, typing a number (keyboard does not reach a background window).
- Removing with Delete or Backspace (same). Undo of a placement does take the thing away.
- 60 pictures a second on the stage (I can read the clock, not count drawn frames; the bar says 30 fps).
- "Remove animation": I did not find it in the time.
- Space 2.5D / 3D, Depth, Parent, Ghost, Clip to below, the Camera tile, the Animate and Export tabs.
- A click on the ruler while playing: lost twice, worked the third time; I cannot tell the window from my tool, so no finding.
- The folders Desktop, Documents, Downloads (told not to).

## What stopped each target
**Target 1 (`edit_v5.mp4`).** Placed with the window: AE background, keyed cats, the glass ring (was there), a glass cube, the photo (turned 20°), the fox, a text. Stopped by:
1. The film is 1080 × 1920 at 60; the window's is 1920 × 1080 at 30 and the chip does nothing.
2. The fox turns in the film; Rotation X / Y do nothing (finding 2).
3. Two typed lines, one Japanese: typing not tried; what I can say is that the Text box does not show the text's words (finding 3) and the new text arrives as a file name over the whole stage.
4. Order: the cat passes in front of the photo in the film; I cannot choose what is in front.

**Target 2 (cat puppet scene).** Reached: a background, `popcat` (still), `vest` (was keyed, I re-shaped its stretch), `happy` (my keys: no travel, finding 5), a keyed glass cube that does move, 20 `waving` in one pile. Stopped by: typed text not tried; laying out many cats is one number-drag at a time from the same centre spot, with no drag on the stage and tiles that do not show the cat; the keying trap of finding 5. (Trimming or sliding a clip on the timeline: not tried; every row I placed ran from the playhead to the end of the film, also for the 6-second clip.)

**Target 3 (rocket collage).** Missing to begin: cutting a 3D thing by a rectangular window (no mask or matte seat found; Rectangle and Shape say "No seat yet"); a camera or a turn to show the rocket from below (Rotation X / Y dead, Camera not joined that I saw); a white-clay surface (only a flat colour or glass; glass refused on a file); flat bands (no rectangle); Cyrillic words (typing not tried). A rocket file can be placed and scaled; that is all.

## Pictures
- `judge_user_r1_1.jpg` — text picked: empty Text box, "Inter Display", "48", a fill colour it does not have.
- `judge_user_r1_2.jpg` — Glow "Added" / "ON"; inspector "Blur On, Kaleido On"; nothing in the file.
- `judge_user_r1_3.jpg` — playing with 32 things.
- `judge_user_r1_4.jpg` — rocket: Rotation X 20, Y 30.1 with key dots, rocket upright.
