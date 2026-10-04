# Critique: W4 "Timeline: grab, move, resize, snap, zoom/pan, marquee" (Widgetbook use case, real window)

Judge: independent critic, no code touched. Source: 4 real-window stills from the orchestrator's action log. Rules: workflows.md section 4 (W4), PROCESS.md step 5. Window arrangement is a rough harness and is not judged; only the Timeline interaction and its knob panel. Stills a is 1411 px wide, b/c/d are 847 px wide. Text size, contrast and 2-4 px gaps are `unverified`. "not exercised" = no evidence either way (not N).

Screenshot key (suffix): a = qofuar (initial, nothing selected), b = l6e03m (after Follow bar body dragged right ~90 px), c = 5odbhb (after Along Path right edge dragged left ~74 px), d = 193wy6 (after marquee on empty lane ground).

Timebase from the ruler: 00:01 at x 591, 00:02 at 707 in a (116 px per second). b/c/d are 0.6x of a.

## W4 checks

| # | check | result | proof |
|---|---|---|---|
| 1 | Drag a bar body: start and end move equally and the Inspector/readout updates | Y for "move equally"; N for "readout updates" (no start/end readout seen) | a to b: Follow bar start x 547 to 382/0.6 = 637 and end 869 to 575/0.6 = 958; both +90 px, width 322 px unchanged. Layer Follow became selected (list row, Timeline row label, Stage box, Inspector header "Follow"). The Inspector in b shows Position X 640 / Y -220, the same kind of content as before; no bar start, end or duration number appears anywhere in b, so the "readout updates" part has no observable result. Whether the bar snapped to a target is not decidable from the stills |
| 2 | Drag its right edge: only the end moves, min width 3 px holds | Y for "only the end moves"; min width not exercised | b to c: Along Path bar start x 513 (a) = 308 (c, 0.6x), unchanged; end 913 (a) to 505/0.6 = 842 in c, 71 px shorter, which is about 19 frames at 30 fps (71/116 x 30). Whole-frame landing not measurable. Bar was not squeezed toward 3 px |
| 3 | Drag a key 10 frames: lands on a whole frame, link line follows | not exercised | no still of a key drag; no key diamonds on any of the 6 rows in a-d |
| 4 | Marquee over keys: all inside become accent-filled | not exercised for keys; a marquee over bars selected layers | c to d: drag on empty lane ground, result: Stagger and Along Path selected (layer list rows, Inspector "SEVERAL SELECTED / 2 layers", Stage shows both, Along Path with handles). The other 4 layers stayed unselected. No rubber-band rectangle visible in d (mid-drag not captured: unverified). No keys on the rows, so "accent-filled" cannot be judged |
| 5 | Wheel with Cmd: ruler labels change step, playhead stays under the cursor | not exercised | plain wheel did nothing (consistent with the knob "Zoom: wheel with Cmd / Ctrl" seen in a-d, whose value reads Cmd / Ctrl but is clipped at the panel foot); no Cmd+wheel performed; ruler labels 00:01 to 00:05 identical in a-d |

Also not exercised: key drag/retime, ruler scrub drag, zoom, pan, snap line display, snap modifier, Esc mid-drag (owner: not required), Cmd-Z of a bar move, min bar width, left-edge trim.

Side evidence that held: each bar drag counted as one step. a shows "0 steps"; d shows "2 steps" after the two bar edits (one per drag). Undo itself was not performed.

## Visible defects

1. Inspector help text and step counter clipped (b, c). With a layer selected the Inspector column ends mid-sentence at the Timeline boundary: "Click select · Shift / Cmd add · Tab next · Esc clear, or revert a drag · Shift = fine · Space play · Delete removes keys · Timeline: drag a bar to move, its ed" (b) and "...its edge to resize, a key to retime, empty ground for a" (c, a longer clip), and the "N steps / Undo" line below it is not visible in b or c. It is visible in a (nothing selected, "0 steps Undo") and d (several selected, "2 steps Undo"). So the step counter and Undo are readable in only 2 of 4 stills; whether they are scrolled out or not drawn is not decidable.
2. Selection state of the bars is hard to read. In b the Follow bar looks only slightly lighter than in a; in c no outline or tint on the selected Along Path bar is visible at this scale. In d the Stagger bar shows a light outline but the second selected bar, Along Path, shows none that I can see. The row labels (left column) carry the clearer selected mark. Exact state `unverified` (downscaled).
3. Stage readout vs multi-selection (d). Footer under the Stage reads "Along Path X 60.0 Y -160.0 0.0° 100.0%" while two layers are selected; the Inspector says "2 layers". Readout shows only one of the two. Stage selection handles appear only on Along Path; Stagger shows a label and an outline.
4. Playhead chip clipped (a-d). The playhead marker on the ruler at 00:00 sits at the left edge of the lane and its label reads "00" / ":00" cut off ("|00" in a, a blue chip with "00" in b-d).
5. Ruler markers unlabeled (a-d). Two small flags sit at about 00:01 (x 575 in a) and 00:03 (x 808 in a); no name or time is shown on them, and in d they stay the same. Their purpose cannot be read from the screen.
6. No cursor feedback seen. The pointer is an arrow in b (over the bar), c (at the bar's new end) and d. Resize/grab cursors, hover states on bars and edges were not captured (the screenshot cursor may be synthetic: unverified).
7. The bar mid-drag state (ghost, snap line, delta label) is not captured in any still, so what the user sees during a drag is unknown.
8. Bars carry no text. Names are only in the left column, 6 rows at about 22 px pitch; fine at this size. No clipping found there.
9. Lane area right of 00:05: the ruler stops at 00:05 while the grid keeps going to the panel edge; no bar reaches it. Not an error; noted for completeness.

## Knob panel (right)

Seen in a-d: "Snap: default and modifier" (dropdown, value "On, hold to bypass (A...", clipped), "Snap modifier key" (Cmd / Ctrl), toggles for playhead, markers, bar edges, keys, grid step (all on), "Zoom: wheel with" (dropdown, clipped at the bottom edge of the window in all four stills). Items below that (zoom/pan options, grab sizes) were never visible.
- Value of the first dropdown is truncated: "On, hold to bypass (A" and the rest is cut; the text in parentheses (probably Alt or an app name) cannot be read. Full option text unverified.
- Two knobs name a modifier: "Snap: default and modifier" and "Snap modifier key". From the labels alone, which one decides the key and which the default is unclear (the first contains "modifier" and its value mentions a key in parentheses while the second also sets a key). Possible conflict if the two disagree; behaviour not tested.
- "Zoom: wheel with" has its dropdown cut by the window bottom, so its value text reads "Cmd / Ctrl" only partially; the knob list continues below the fold and was not scrolled to.
- The five snap toggles have long, regular spacing and are readable; all are on, so no case where some targets are off was tried (not exercised).
- The knob panel is clipped at the bottom, so grab-size knobs are not seen at all in these stills.

## Summary counts

W4: 2 Y (check 1 move, check 2 edge, each partial: readout part N for check 1, min width not exercised for check 2), 1 partial N (readout), 3 not exercised (3, 4 keys, 5). Defects 1 and 3 are the most visible: the clipped step counter/help text and the single-layer Stage readout under a two-layer selection. Much of W4 (keys, snap, zoom, pan, scrub) has no evidence yet.
