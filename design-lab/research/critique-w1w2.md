# Critique: W1 + W2 (Widgetbook "Workflow" use case, real window)

Judge: independent critic, no code touched. Source: 9 real-window stills from the orchestrator's action log (letters a-i below). Rules: workflows.md section 4 and PROCESS.md step 5. Owner exclusions applied: drag-time Esc (W2-3) is not judged; the combined window arrangement is a rough harness and is not judged, only each panel's own interaction. Stills are downscaled (1411 px and 847 px wide); text size, contrast and 2-4 px gaps are `unverified`. "not exercised" = no evidence either way (not N).

Screenshot key: a = ...7zhvaq (initial), b = ...g3t5je (Stagger selected via Timeline), c = ...8bo69o (after Stage drag), d = ...m1egss (after Tab), e = ...k2u2ao (after Esc), f = ...1lezeq ('1 steps' + Undo), g = ...7md3kv (after Undo), h = ...73tdyx (Stagger from list), i = ...5hp2aw (after Inspector scrub).

## W1 Select and inspect

| # | check | result | proof |
|---|---|---|---|
| 1 | Click a bar in Timeline: Inspector header changes to that layer's name | Y (layer 2 "Stagger" instead of layer 3; same behaviour) | a to b: header "NOTHING SELECTED" becomes "LAYER / Stagger", values X -300 / Y 200 / Z 15.0 shown |
| 2 | Same layer's Stage box shows the selection outline | Y | b: Stagger box has a light outline with 8 handles, centre dot and the "Stagger" label; the other boxes have none |
| 3 | Click empty ground: "nothing selected", no outline remains | Y by Esc only; click-on-empty-ground not exercised | e: Inspector reads "NOTHING SELECTED", no box outline, Timeline rows untinted except hover |
| 4 | Hover another row: its Stage box lightens, Inspector does not change | Y for hover on a Stage box; hover on a Timeline/list row not exercised | c, d, e, f: the Attach box under the pointer is filled lighter than its neighbours while the Inspector keeps its state (Stagger / Along Path / nothing). In e the "Attach" list row also has a grey fill |
| 5 | Tab/arrow moves selection, focus ring visible on the row | Tab: Y. Arrow keys: not exercised | d: Along Path selected in all panels (list, Stage, Inspector, Timeline); a thin outline frames the Along Path Timeline row and its list row. Ring width and contrast `unverified` |
| (plan) | 100-200 ms select transition on all surfaces | not exercised (stills only) | - |

## W2 Edit one value both ways

| # | check | result | proof |
|---|---|---|---|
| 1 | Scrub Position X in the Inspector: Stage box moves right | Y (scrub amount was about +77, not exactly +50; +50 not exercised) | h to i: X -300.0 to -222.7, the Stagger box moved right and the Stage readout shows X -222.7, the Position pad box also moved |
| 2 | Drag the box on Stage: the X readout follows | Y for direction and coupling; the "40 px left" amount not measured | b to c: box moved right and up, Inspector X -300 to 484.6 and Y 200 to -271.0, Stage readout "X 484.6 Y -271.0" matches, the pad in the Position row moved |
| 3 | Esc during drag restores | not judged (owner exclusion) | - |
| 4 | Shift-scrub is 10x slower | not exercised | - |
| 5 | After a drag, Cmd-Z restores in one step | Undo button: Y (one step). Cmd-Z keystroke: not exercised | f: footer "1 steps" with Undo; g: after Undo "0 steps", Stagger back at its original place (compare b/h), X/Y no longer edited. One drag counted as one step in c ("1 step") |

Also seen, not in the checklist: Inspector pad, readout line under the Stage, and Timeline all show the dragged value (c: pad box at upper right). Typed number, rotation/scale handles, Shift/Cmd multi-select: not exercised.

## Visible defects (this window)

1. Footer counter reads "1 steps" / "0 steps" (c uses "1 step", f and i use "1 steps"): wording is inconsistent between stills (wrong plural in f, i).
2. Hint text says "Cmd-Z undo" but the keystroke did not undo in the run; the hint promises an action not confirmed (state claim unverified for a real user's keyboard).
3. The hover fill on a Stage box (Attach in c/d/e/f) is brighter and more solid than the selected box's fill in b/c, so hover reads stronger than selection on the Stage. Contrast numbers `unverified`.
4. Stage boxes overlap by data (Stagger over Attach in b, g, h, i); the selected box's label "Stagger" sits on top of the neighbour's edge in b/h. Handles over another box's outline are harder to tell apart (h). Not a layout defect, a legibility one.
5. In i the pointer sits on the X value so the digits "-222.7" are partly covered; this is the pointer in the still, not an app fault (unverified).
6. Empty-state Inspector (a, e, g): the help text is small, dim grey and left at the top of a mostly empty column; size/contrast `unverified`. No clipping seen.
7. Selected Timeline bar has no outline of its own; selection is carried only by the row tint and the left tick of the label (b, h). Whether this meets "bar selected" is a judgment left to the owner.
8. Timeline focus ring (d) is a thin line around the row; on the Along Path row it merges with the bar's right edge. Visibility `unverified` at this scale.
9. No clipping, overlap of text, or wrong state found in the Inspector values, labels (X/Y/Z, px, %, deg all complete) or the Timeline labels in any still. The Stage readout and the Inspector always agreed (b, c, h, i).

## Summary counts

W1: 4 Y (one with a qualifier), 1 partial split (hover row not exercised), arrows and transition not exercised. W2: 3 Y (1, 2, 5 via button), 3 not exercised or excluded. No N recorded: every exercised check passed; the N-worthy items above are defects, not failed checks.
