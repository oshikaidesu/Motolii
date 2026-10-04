# Critique: Browser B2 Tag bands (chips and group rows; Ableton type panel at 320)

Judge: independent critic, no code touched. First review of B2 (no previous critique). Both captures are 1411x840 real-window screenshots. Method: pixel sampling and bounding boxes with PIL on the full-resolution files (grey 0-255 of JPEG, noise about 5 levels; thin glyphs peak below the true fill). Sheet A is @200% (real = captured / 2). Panel B is rendered at about 0.97 of nominal.

## A. Parts @200% Chips and groups (use case "Parts @200% Chips and groups")

Measured: plate 25, rest chip 32, hover chip 52, on chip 66, mode line 2 captured px (= 1 px real) at about 132 under the on chip; rest text 208, count 150; zero text 150 and zero count 149; on text 245.

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | name is bright (208), count dimmer (150); on chip is the brightest object in a row. |
| colour discipline | OPEN | mode line is lavender (accent hue). Chips themselves are grey. Accent hue is owner-undecided. |
| depth | Y | steps plate 25 < rest 32 < hover 52 < on 66; no shadow. |
| consistency | N | plates differ in width by cell (233 wide for rows 1-2 and the first cell of row 3, 264 for the two-digit cells, 296 for "long word"); all other metrics (inset 16, chip height) are equal. Reads as intentional "fit to content", but the sheet header says nothing about it. |
| spacing | Y | chip left inset in plate 16 in every cell; group rows: chip gap about 10 captured (5 real); wrapped second line aligns to the first chip column (Stylize x458 = Blur x458). |
| legibility | Y (chips) / N (sheet captions) | chip text and counts well over floor at 200% (real about 10-11 px, g58+). Sheet captions peak 126 (about g49) and group label 133: below g56, but they are sheet annotations, not product, so not judged. |
| states | N | rest, hover, on, zero, zero+hover, no-count, no-count on, 2-digit, long word are shown. Missing: pressed, keyboard focus, and the hover of an ON chip (on 66 vs hover 52 differ by only 14 levels; whether an on chip under the pointer stays distinguishable from a hovered rest chip is not shown). |
| finish | unverified | "long word" Directional 112 is shown at full width (178 captured); how a chip behaves when the group is narrower than the chip (ellipsis or wrap) is not shown. The wrap-to-two-lines case is cut by the window bottom (third line "Generate" and anything below not in frame). |
| product-grade | Y for what is visible | |

Fine-detail defects (A):
1. Rest chip (32) on plate (25) is only 7 levels apart; chip boundary is nearly invisible at rest, the shape reads only through its text. Hover (52) jumps by 20. The 7-level gap is at the edge of JPEG noise: `unverified` whether it is perceptible at 100%.
2. Disabled (zero) chip is the same fill as a rest chip (32); only the text drops (208 to 150). The sheet header says "g56 word", measured 150 (about g59): acceptable, but disabled vs rest differs by text alone and the chip still has a hover-looking plate.
3. Mode line sits inside the chip bottom (2 captured px at 191-192) and its colour is the accent hue, while the same selection class in DESIGN.md says g20 pill + 1 px accent underline: here the fill is g26-equivalent (66) and not the g20 the rule names: deviation `[decided]` vs measured, to confirm with the owner (may be intended: "g26 + line").
4. Sheet is cut at the bottom (third line of the wrapped Category row); part under review not entirely in frame.
5. Count digit and name share a baseline but the count is at lower contrast and Menlo: fine; on-chip count (about 150) is not raised to match the on-chip name (245): the count looks dim on an on chip (contrast to chip fill 66: about 84 levels, OK).

## B. B2-a Tag bands, Ableton type, Browser 320 (2 filters, 8 of 72)

Measured: chip text cap height 7 captured px (about 9.7 px nominal font, `unverified` against the 10 px floor), counts 8 px digit height; on chip fill 65 vs rest 32; on text 238, rest text 188; zero chips (Media 0, Time 0 ...) text peak about 126-136; group labels (Kind, Category, ...) peak 135; second line of rows 137; status line "2 filters" 242; row selected 52.

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | search field > chip groups > status line "2 filters . 8 of 72 / Clear" (bold, 242) > rows. The two on chips (Effect, Soft) are the brightest in the band. |
| colour discipline | OPEN | chip band grey; thumbnails are grey glyph tiles (no hue). Right-hand knob panel lavender is Widgetbook chrome. Accent hue for on-chip underline OPEN. |
| depth | Y | band and list on the same plate (25), selected row 52; no shadow. |
| consistency | N | selected row (g20 fill, left tick) matches the rows rule. But the same band shows two different selected-state treatments: chips use fill + underline (choice control), rows use fill + tick, as DESIGN.md decides; fine. The N: kind label "Effect" at the right of every row repeats the active filter chip "Effect" above, and the second line ". Blur" repeats the category: every one of the 8 rows carries information already fixed by the filters (carry-over of B3 defect B1). |
| spacing | N | (1) selected tick at the very left of the panel (x about 437, 4 px from the edge) vs chip/field/label inset about 10 px and thumbnail inset about 9 px: three left edges (4 / 10 / 9). (2) Row groups: gap between group rows about 6 px (between Kind and Category, 4-5 px between wrapped lines): tighter than the 10 px inset, reads dense but even. |
| legibility | N / unverified | (1) zero chips: "Time 0", "Media 0" text peaks about 126-136 (about g49-g53) with thin 7 px glyphs: probably below the g56 (142) floor; `unverified` exactly because anti-aliased thin glyphs under-read, but it is the weakest text in the panel. (2) Group labels "Kind / Category / Character / Source" peak 135 and sit at the same grey as disabled chips: label and disabled chip text cannot be told apart by grey level. (3) chip font about 9.7 px nominal at this scale: borderline vs 10 px floor, `unverified`. |
| states | N | on chips (Effect, Soft) vs rest are clear (fill 65 vs 32, text 238 vs 188). Mode line under on chips is faint at 100% (2 px at about 96-103, 1 px real): the fill carries the state, the line is almost invisible. Hover and disabled-hover not shown in this use case (shown in A). The state "no filter on" and "one group has several on" are in other use cases (groups, zero result). |
| finish | N | (1) about 230 captured px of empty panel below "Ripple" (y 641-778) with the footer hint outside the panel: dead area. (2) "Directional Blur (Motion Trail, 8-sample..." ellipsis intentional. (3) panel frame bottom edge and the hint line below are different grey from the knob column; harness only. |
| product-grade | N (partial) | see defects below; chip band itself is close to product level. |

Fine-detail defects (B):
1. Redundant right-hand kind "Effect" on all 8 rows when Kind=Effect is the only on chip (and ". Blur"/". Glow" duplicating category chips). Same defect as B3 round 1 B1, still present in this component.
2. Disabled chips ("Media 0", "Time 0", "Title 0" ...): 8 of 24 chips in the band are disabled; they take the same size and position as live chips, so the band shows 8 dead chips at similar visual weight (rest vs disabled differ by about 60 levels in text only).
3. Left-edge rhythm 4 / 9 / 10 px (tick / thumbnail / chips).
4. Faint mode line (about 100 grey, 1 px) under on chips at 320.
5. Dead area (about 230 px) under the list, no "n more" cue (8 of 8 shown here, so no cue needed; the blank is the harness size).
6. Row thumbnails: the Blur family tiles are all the same grey smear icon (five identical tiles), Glow and Bloom identical tiles: rows are not distinguishable by thumbnail; name carries all the information.
7. Clear action "Clear" is right-aligned text at 242 grey, brighter than any chip but not a button-looking object; whether it is clickable is not signalled (no hover shown).
8. Count line "2 filters" says how many are on but not which (the chips above show which); fine.

## C. Verdicts

- Chips and group rows (A): FIX-THEN-RELEASE. Fixes: show pressed and keyboard-focus states and the hover of an on chip; show the narrow-group behaviour of a long chip (ellipsis or wrap) and re-capture the wrapped row uncut; confirm with the owner that the on-chip fill is g26 (66) not the g20 the selected-state rule names; state in the sheet that plates fit their content.
- Tag-band panel at 320 (B): FIX-THEN-RELEASE. Fixes: drop the per-row kind and category text that the active filters already state; raise the zero-chip and group-label grey to at least g56 (measure by zoom, then confirm) and separate the two by some means other than grey; make the mode line readable at 1 px (or rely on fill only and say so); align tick, thumbnail and chip left inset; give the Blur/Glow tiles distinguishable thumbnails or drop thumbnails for effects.
- OPEN (owner): accent hue of the on-chip underline; tab/chip selected fill g20 vs g26.
