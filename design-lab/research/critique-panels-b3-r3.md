# Critique round 3: Browser B3 Find (parts sheet top + unified query panel)

Judge: independent critic, no code touched. Both files 1411x840 real-window captures. Method: pixel samples of grey values 0-255 (JPEG noise about 5 levels; within that = unverified). I did not re-zoom visually beyond pixel probes; text height (10px floor) is `unverified` throughout. Sheet A is @200%.

## A. Parts sheet top: field states (6 of 7 visible; "long text" row again below the frame)

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | caption small/dim, then field; typed "bl" and hint are the brightest in each field. |
| colour discipline | Y | greys only; edge peaks rest 51 / hover 101 / focus 173 (typed: 50 / 105 / 169). Sidebar selection purple is Widgetbook chrome. |
| depth | Y | plate 25, field 19, 2 captured px edge, no shadow. |
| consistency | N | the right-hand marks do not share an alignment: "/" chip spans about x 845-879 (glyph 860-861), the clear "x" glyph spans 868-872 (centre about 870 vs chip centre about 861). Mark centre moves about 9 captured px (about 4-5 real px) when typing starts. Plate widths and field heights equal in all six rows. |
| spacing | Y | plate padding and field widths equal across rows (same as r2). |
| legibility | Y/unverified | hint and typed text large at 200%; 10px floor of real size not measurable here. |
| states | N | hover of the clear mark is still not shown anywhere in the frame; rest/hover/focus, typed rest/hover/focus all present and distinguishable (hover 101 vs rest 51 = clearly visible now). |
| finish | unverified | the "long text, resting: ellipsis" caption is cut at the window bottom (y about 838); the ellipsis field and the narrow-dock row are not in frame. |
| product-grade | Y (visible six) | |

## B. B3-a Find unified query (320, query "bl")

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | tabs, field, count line, rows with bold name and bold match, dim second line, right kind. |
| colour discipline | OPEN | media thumbnail hue (VHS Wobble tile) is owner-undecided; chrome is grey. |
| depth | Y | panel 25, field 19, selected row 52 (g20), no shadow. |
| consistency | N | selected-row tick (x 437-440, peak 236) to thumbnail start (x about 443) is about 3 px; tick sits about 3 px from the panel edge while tabs/field/count are inset about 10 px. Left rhythm 3 / 10 / 10 / 11 px not unified. |
| spacing | Y | row pitch 33 equal; name/second line share x=484; kind labels right edge 734. |
| legibility | Y/unverified | second line peak 128, count line 128, kind 132 (thin glyph peaks; r2 had 135/132/144): about 7-12 levels lower than r2 and below g56=142 expected peak. Cannot separate JPEG/anti-alias from a real token drop: `unverified`, flag for owner re-measure. Tab labels 160. |
| states | N | no tab shows any state: all four tabs are identical (peak 160, no pill, no underline); only the count line says "no tab on: all places". Hover row / no-result are in other use cases. |
| finish | N | panel body runs to y about 778 and the last row ends at y about 545: about 230 px of dead area, and no cue that 62 more results exist (only "10 of 72"). |
| product-grade | N | defects 2, 3, 4 below. |

## 1. Previous (r2) defects

A (field):
1. Long-text ellipsis vs hint clip: NOT FIXED / unverified (row out of frame again).
2. Typed hover vs rest weak: FIXED (105 vs 50).
3. Clear mark hover not shown: NOT FIXED.
4. Marks right-inset mismatch ("/" 17 vs "x" 33): NOT FIXED (centres still about 9 captured px apart).
5. Narrow-dock plate: unverified (out of frame).
6. Plate widths: FIXED (earlier), still equal.

B (panel):
1. Duplicate kind ("Project" right + "Video" in line 2 of lens_bloom.mp4): NOT FIXED.
2. Active tab invisible: NOT FIXED.
3. Near-black blurred thumbnails: FIXED for the five Blur rows (mean about 135, equal to neighbours). lens_bloom.mp4 tile is darker (mean 85) and VHS Wobble 100 vs 135-162: smaller spread, acceptable, minor.
4. No cue for 62 unshown results: NOT FIXED.
5. Dead panel area below list: NOT FIXED.
6. Tick/thumb/inset alignment (4/11/10): NOT FIXED (3/10/11).
7. Media hue: OPEN.
8. "no tab on: all places" wording: FIXED earlier, unchanged.

## 2. New fine-detail defects
1. Second-line/count/kind grey peaks 128-132 vs 135-144 in r2 (see legibility); token may have dropped below g56. Verify the token value.
2. Selected row fill spans full panel width (x 434-745) while tabs and field are inset about 10 px: edges do not align (by design of rows; noted).
3. Mixed JP/Latin second lines have different visual weight; one row mixes with " - " (unverified at real size).
4. Helper text above panel still says "Esc clears the text, a second Esc leaves the field" while the panel itself has no Esc hint (harness text, not product).

## 3. Verdicts
- Search field (A): FIX-THEN-RELEASE. Fixes: align clear-mark centre with "/" chip; show clear-mark hover; re-capture the lower rows (long text ellipsis, narrow dock).
- Unified query panel (B): FIX-THEN-RELEASE. Fixes: visible tab state (including none-on); drop the repeated kind ("Project" + "Video"); add a "62 more" / scroll cue; unify left insets (tick 3 / tabs 10 / thumb 11); verify secondary text is still >= g56 and >= 10px.
- OPEN (owner): accent hue, tab hue, media thumbnail colour.
