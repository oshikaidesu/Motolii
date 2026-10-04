# Critique: W3 "Set keys and play" (Widgetbook "Workflow" use case, real window)

Judge: independent critic, no code touched. Source: 7 real-window stills from the orchestrator's action log. Rules: workflows.md section 4 (W3) and PROCESS.md step 5. Owner exclusion applied: window arrangement is a rough harness and is not judged, only each panel's own interaction. Stills are downscaled (847 px wide); text size, contrast and 2-4 px gaps are `unverified`. "not exercised" = no evidence either way (not N).

Screenshot key (file suffix): a = ff5frx (Stagger selected, 00:00), b = 3uydd4 (after first click on key-mark area), c = 7jvszt (Inspector back at top, playhead 02:03, Stage dragged), d = 28ufa4 (after key click at 02:03), e = c7sv3v (ruler to 01:01, Stage drag), f = qoc32o (after Space), g = l9wmdh (after Space again).

Timebase read from the ruler: 00:15 / 01:00 / 01:15 ticks, so frames are 30 per second (mm:ff).

## W3 checks

| # | check | result | proof |
|---|---|---|---|
| 1 | Click stopwatch: a diamond appears at the playhead on that layer's row | Y (done at 02:03, not at 0) | c to d: Inspector row "POSITION KEY 02:03" mark was hollow, after the click a diamond appears on the Stagger Timeline row at x of the 02:03 playhead and one at the Stage box centre. Row hollow in a/c, no key on the row in a/b/c |
| 2 | Move playhead, change value: a second diamond and a link line appear | Y (playhead 01:01, not 2 s) | d to e: ruler click to 01:01 then Stage drag. Result: second diamond at 01:01 on the Stagger row, a link line between the two diamonds, a path line on the Stage, Inspector mark filled blue. X -519.7 / Y 137.2 at 01:01. Precondition seen in b/c: with no key, a value change creates none (position changed at 02:03, no diamond on the row), so the second diamond exists only because a first key existed |
| 3 | Space: the Stage object travels between the two positions and the playhead advances | Y | e to f: label "Play" becomes "Pause", playhead 01:01 to 01:05, box moved along the path, readout X -398.1 Y 86.2. Linear check from keys (01:01: -519.7, 137.2; 02:03: 453.2, -271.0; 32 frames apart): at 01:05 (t = 4/32) X = -398.1, Y = 86.2, matches to 0.1 |
| 4 | Space again stops; the object stays at the current frame | Y for stop and state agreement; "stays" over time not shown | f to g: label back to "Play", playhead 01:09, readout X -276.5 Y 35.2, which equals the linear value at t = 8/32 (X -276.5, Y 35.15). Stage box sits between the two keys. No later still proves it does not drift after the stop |
| 5 | Delete on a selected diamond removes it and the line | not exercised | no still; key pick on Timeline and Delete were not performed |

Also not exercised: scrub drag on the ruler (only clicks), key pick on the Timeline, undo of a key add.

## Visible defects (this window)

1. Inspector row hidden after the first click (a to b). In a the header "LAYER / Stagger" is followed by the row "POSITION KEY 00:00 (hollow diamond)" at y 59, then SPACE at y 76. In b the header is unchanged, the POSITION KEY row is gone, SPACE is at y 64 and every group below moved up about 12 px (POSITION 106 vs 119, ANCHOR 283 vs 295, footer text 355 vs 367). The Timeline row has no diamond in b, so the click added no key. What is observable: the click was followed by the key row leaving the Inspector (a scroll of the Inspector content under a fixed header, or removal of the row; the stills do not tell which) and the pointer, at (665, 67), now sits over empty space where the row was. The row reappeared in c when the Inspector was back at top. A key mark that moves away from under the pointer after a click is a state defect: the user sees no key added and no row to click again.
2. No key on a value change when none exists (b/c): the Stage drag at 02:03 changed X/Y (453.2 / -271.0) and no diamond appeared; the Inspector mark stayed hollow ("POSITION KEY 02:03" in c). Observable consequence: the value at 02:03 is edited but not stored as a key; nothing in the still tells the user so except the hollow mark.
3. Footer step counter missing after edits: b shows "0 steps / Undo" at the foot of the Inspector. In c, d, e, f, g (after a Stage drag, a key add and a second Stage drag) the line ends at "Undo button" with no step count and no Undo control. The counter that W2 stills showed is not visible here (a also has none). Whether the counter is hidden, scrolled out or not drawn is not decidable from the stills.
4. Footer wording "0 steps" (b): plural wrong for 0/1 in W2 as well; here only "0 steps" seen, which is acceptable; "1 steps" not seen in this run.
5. Pointer over the key mark: in d the pointer sits on the Inspector diamond (665, 63) and hides its fill state; the filled state is readable only in e. Pointer in the still, not an app fault (unverified).
6. Path line on the Stage (e/f/g): a thin white line from the 01:01 position through the box centre toward the 02:03 key at the right edge of the "Follow" box. The key at 02:03 lies on the edge of the "Follow" box and the pointer sits on it, so the second diamond cannot be told apart from the Follow box's top-left corner in e. Overlap by data, a legibility problem.
7. The Stagger box and its handles overlap the Attach box in e/f/g (selected box over a neighbour's edge); label "Stagger" is clear of the neighbour in e/f/g. No text clipping.
8. Hover fill on "Follow" (e/f/g) is lighter than the selected Stagger box fill; hover reads stronger than selection (same as W1/W2 finding). Contrast `unverified`.
9. Timeline link line (e/f/g) is a thin white line with end diamonds on the blue bar; it is readable. The playhead label (01:01, 01:05, 01:09) sits over the ruler and hides the 01:00/01:15 tick labels while it covers them (g: playhead over 01:00 to 01:15 labels). The ruler tick labels beneath the playhead are not readable in g.
10. Inspector key mark in f/g is a diamond with side ticks (between keys), in e a filled diamond (on a key), in a/c a hollow diamond (no key). The three states are distinct in the stills; the mark is small (about 8 px) and its exact shape in f/g is `unverified` at this scale.
11. Stage readout and Inspector always agreed (c X 453.2 Y -271.0; e -519.7 / 137.2; f -398.1 / 86.2; g -276.5 / 35.2); the playhead label "00:00 .. 01:09" in the readout and the ruler agreed in every still. No wrong state found there.
12. Discrepancy with the action log: b is described as "after a ruler click to 02:03 and a Stage drag", but the still shows playhead 00:00, X -300 / Y 200 and "0 steps". The still shows only the effect of the first click. Judged as shown.

## Summary counts

W3: 4 Y (check 4 with the qualifier "stays" not shown over time), 1 not exercised (Delete). No N. N-worthy items above are defects (1 and 3 first), not failed checks. Not exercised: Delete key, ruler scrub drag, key pick on the Timeline, undo of a key add.
