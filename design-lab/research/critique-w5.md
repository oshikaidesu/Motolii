# Critique: W5 "Multi-select with a shared value" (Widgetbook "Workflow" W4 use case, real window)

Judge: independent critic, no code touched. Source: 2 stills from the orchestrator's action log. Rules: workflows.md section 4 (W5) and PROCESS.md step 5. Window arrangement is a rough harness and is not judged. Stills are downscaled (847 px wide); text size, contrast and 2-4 px gaps are `unverified`. "not exercised" = no evidence either way (not N).

Screenshot key (file suffix): a = iwz4de (marquee over Stagger + Along Path, 0 steps), b = o4ta9f (after a ~60 px sideways drag on Position X, then a click on the field; "1 step").

## W5 checks

| # | check | result | proof |
|---|---|---|---|
| 1 | Select several: Inspector header reads the count ("3 layers" with Shift-click) | Y for the count, via marquee with 2 layers (not Shift-click, not 3) | a: header "SEVERAL SELECTED / 2 layers", chips "Stagger" and "Along Path", a sentence "= the layers differ. A scrub moves each by the same amount; a typed value sets them all." Left list shows both rows highlighted. Shift-click itself: not exercised |
| 2 | A differing value shows the mixed mark, not one layer's number | Y | a: Position X, Position Y and Rotation Z show "—" (the layers differ in the Stage: Stagger and Along Path sit at different places and angles); Scale X/Y show 100.0 (the same). Same/different split is consistent with the Stage; actual per-layer values not shown, so correctness of the split `unverified` |
| 3 | Scrub Opacity -10: every layer drops by 10, values still differ | Partly Y, on Position X not Opacity | a to b: after the drag the counter is "1 step" (one step for the whole drag). Stage: Stagger box moved right about 7-8 px, Along Path box about 5 px at this scale (about equal; exact deltas `unverified`). Opacity was not scrubbed. Not decidable from the stills that each layer moved by the same amount; and see defect 1 (the field shows one number where the values should still differ) |
| 4 | Type 50: all read 50 | not exercised | no typed absolute value was performed |
| 5 | Stage shows one combined box around the layers | Y | a and b: a single outline rectangle encloses both the Stagger and Along Path boxes (a: about x 333-428, y 168-240 at still scale), with handles in b; Along Path labelled above it. In b the combined outline resized after the move (right edge 428 in a, 427 in b, boxes shifted), so it follows the selection |

Also not exercised: Esc during scrub, Cmd-Z restore (counter reads "1 step / Undo", Undo not pressed), axis lock, Stage gizmo moving the group, Shift/Cmd-click select, mixed Scale scrub.

## Visible defects (this window)

1. Readout mismatch with check 2/3 (b): Position X shows a single number "121.x" with the pointer on it after the drag and click, while Position Y beside it is hidden by the pointer. If the layers still differ (the Stage shows they sit apart and Rotation Z still reads "—"), a single value on X is not the layers' value; it could be an edit-mode field showing one layer (the leader). The stills do not tell which; the field is not marked as editing and not marked as one layer's number. Unverified.
2. Label "=X" / "=Y" / "=Z" (a, b): the axis tabs of the mixed fields read "=X", "=Y", "=Z" while the unmixed Scale fields read "| X", "| Y". The "=" prefix is explained only by the sentence in the header ("= the layers differ"); at this size it reads as "equals". The two marks differ by one thin stroke in the still, so their separation at real size is `unverified`. Position X before drag is "=X — px" (a); after b the "=" remains on a field that now shows a number.
3. "—" values carry a unit "px" / "°" beside them ("— px", "— °"); a unit next to a placeholder reads as a number that is missing. Observed in a.
4. Header sentence is long for the column (two lines, small type); text size `unverified`. The chips "Stagger" / "Along Path" are in layer colours (blue, green) and match the Timeline row dots.
5. Footer "2 layers (Along Path leads)" (a, b): "leads" is not explained anywhere in the still; which layer's value fills a field when the layers agree is not stated. Observable: the word is the only mention.
6. Counter wording: a "0 steps", b "1 step" with Undo link: singular is correct in b.
7. Position pad ghost boxes (a, b): the pad at the right of Position shows a small rectangle with a smaller inner box and an extra dash "-" left of the pad (b, at about x 590) and a third box edge at the right in a. Whether these are the two layers' ghosts or leftover marks cannot be told at this scale; unverified. The pad does not change between a and b while the Stage boxes moved, observable only as a difference between the Stage and the pad.
8. Stage: the combined outline and the 6 other boxes overlap (Face/Follow/Attach boxes lie behind the selection; the outer large rectangle at x 300-482 is the composition frame or a second outline, not told apart in the still). A thin crosshair guide passes through the Along Path box centre (a, b).
9. Timeline: with both layers selected the Stagger and Along Path bars show no distinct selected look beyond the left-list highlight; the bars' own selection mark is `unverified` at this scale. Keys on Scatter, Stagger and Face rows are visible; the playhead label "00:00" at the left.
10. Pointer in b sits on the Position X value and hides its right edge and the "px" unit; not an app fault (unverified).

## Summary counts

W5: 3 Y (1 with count 2 via marquee, 2, 5), 1 partly Y (3, Opacity not done, deltas unverified), 1 not exercised (4). No N. Open item to confirm by hand: defect 1 (what the X field shows after a scrub while the layers differ).
