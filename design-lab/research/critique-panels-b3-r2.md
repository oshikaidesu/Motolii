# Critique round 2: Browser B3 Find (parts sheet top + unified query panel)

Judge: independent critic, no code touched. Both files are 1411x840 (checked with `file`), real-window captures. Method: crops zoomed 4x and pixel samples (grey values 0-255 of the JPEG; JPEG noise of a few levels, so anything within about 5 levels is "unverified"). Scale note: panel B measures 311 px for a nominal 320, field 26-27 px for a nominal 28, row pitch 33 for 34, so the window is rendered at about 0.97; sizes below are given as captured, nominal in brackets. Sheet A is @200%, so real size = captured/2.

## A. Parts @200%: field states (6 of 7 states visible; "long text" row is cut by the window bottom)

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | caption (small, dim) -> field on plate; typed text "bl" and hint are the brightest things in each field. |
| colour discipline | Y | greys only; edges rest/hover/focus differ by grey level only (edge peak about 50 / 67 / 160). Only colour is the sidebar selection (Widgetbook chrome). |
| depth | Y | plate (25) one step above field (19), 1 px (2 captured px) edge, no shadow. |
| consistency | Y | all plates same width (298-912 = 614) and all fields same width (315-896), plate padding 17 left / 16 right / 16 top, in every row; field height equal in all 6; "x" and "/" are the same size and centre line (x at 863, chip at 861). |
| spacing | Y | like elements measured equal (above). Gap between field and its mark (chip right inset 17, x right inset 33): the two marks are not on the same right inset (x sits 16 captured px further in than the chip). Minor N-candidate, see defect 2. |
| legibility | Y | hint and typed text well above floor at 200% (real about 11px). Sheet captions about 9 captured px high, annotations not product, not judged. |
| states | Y/partial | rest, hover, focus, typed rest, typed hover, typed focus all shown and distinguishable. Not shown: hover of the clear mark; the disabled state is stated "not designed: find is always on" in the sheet header (OK). |
| finish | unverified | "long text, resting: ellipsis" caption is visible but the field itself is below the window edge; the ellipsis fix cannot be seen. |
| product-grade | Y for the visible five states | |

Hover vs focus: hover edge (peak about 67) is only about 17 levels above rest (about 50), focus about 160. Hover is readable but weak; typed hover (67) vs typed rest (54) only 13 levels apart: near the JPEG noise, `unverified` whether it is visible at 100%.

## B. B3-a Find unified query (320, query "bl")

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | tabs, field, count line, rows (bold name with bold match, dim second line, right kind). |
| colour discipline | OPEN | media thumbnails carry hue (pink VHS Wobble, teal/yellow lens_bloom.mp4, warm grey Bloom). Content colour, owner to decide. UI chrome itself is grey. |
| depth | Y | panel 25, field darker (about 18), selected row 52 = g20. No shadow. |
| consistency | N | selected row (g20 fill, 2 px tick of value 236 at x 438-440) matches the DESIGN.md rows rule. But the thumbnails: the five Blur rows have a near-black blurred strip that reads as an empty/failed image next to Bloom, My Soft Bloom, VHS Wobble which have real tiles; thumbnails differ in brightness by an order of magnitude inside one list. |
| spacing | Y | selected row band 218-264 captured? rows pitch 33 even through the list; tab text, field, count line share one left inset (about 10 px); name and second line share x=486; kind labels right-aligned at 734 (Mine at the same right edge). |
| legibility | Y/unverified | second line peak grey 135, count line 132, kind label 144, hint 144 (thin anti-aliased glyphs peak below the true fill; consistent with g56 = 142, not below). Text height: name about 9 captured px cap/x height, second line smaller; nominal 10px floor cannot be confirmed from pixels, `unverified`. Tab labels peak 161. |
| states | N | active tab still not marked: all four tabs look identical (peak 161, no pill, no underline); the count line now says "no tab on: all places", which explains it but the line is about 130-grey text, not an indicator. Hover row and no-result are in other use cases. |
| finish | Y | the only clip is the intended ellipsis on "Directional Blur (Motion Trail, 8-samp..."; thumbnails and names aligned. |
| product-grade | N (partial) | see defects 3, 4. |

## 1. Round-1 defects

A (field):
1. Typed text vs hint clip treatment differs: unverified (long-text field not in frame).
2. Focused vs rest edge, typed rest vs typed focused: FIXED for focus (typed focused edge about 170 vs 54); typed hover vs rest weak, unverified.
3. "/" duplicates help text: the help line is no longer in the count line; FIXED. The sheet header still names the key marks.
4. Clear mark contrast: x reads about g63-like and clearly readable; FIXED/Y.
5. Plate widths differ between rows: FIXED (all 614 in the visible rows; the narrow-dock row is below the frame, unverified).
6. Hover for field not shown: FIXED (hover rows added). Hover for the clear mark: NOT FIXED (not shown).

B (panel):
1. Duplicate kind statement ("Project" right + "Video" in line 2): NOT FIXED, lens_bloom.mp4 still reads "1920x1080 - 4.2 MB - Video" with "Project" at right; "Effects" + "- Blur"/"- Glow"/"- Stylize" in line 2 is a category, not a duplicate.
2. Duplicate clear action ("Esc clears" in status line plus "x"): FIXED in the panel (status line now only "10 of 72 - no tab on: all places"). The helper sentence above the panel still carries it (harness text).
3. Active tab invisible: NOT FIXED in the visual (explained in words only); no tab has any state mark.
4. Count line "all places" ambiguous: FIXED in wording ("no tab on").
5. No cue that 62 more results exist: NOT FIXED, no scroll or "more" cue; list ends with Tilt Shift then an empty panel area (about 230 px blank) with no "62 more" hint; the count line is the only cue.
6. 10px and g56 verification: partly verified, values consistent with g56 (see legibility); 10px unverified.
7. Media thumbnail colour: OPEN.

## 2. New fine-detail defects

1. Blank space: panel body runs to y=778 while the last row ends at 545, so about 230 px of empty panel under the list; hint "Type to filter..." sits outside the panel below that. (Harness size, but visible state "few results" has a dead area.)
2. Marks in A: "/" chip right inset 17 vs "x" right inset 33 captured: the two marks do not share an alignment; when typing starts, the right mark jumps in by about 16 px.
3. Tick in B: selected tick at x 438-440, row thumbnail starts about x 445: only about 4 captured px between tick and thumbnail, vs 10 px inset of tabs/field; tick sits closer to the panel edge (4 px) than anything else, so the left edge rhythm differs (10 px for tabs/field/count, 4 px for tick, 11 px for thumb).
4. Thumbnails: five of ten rows show a near-black blurred smear (values about 20-40 vs 100+ in neighbours); reads as loading/failed state; hue of VHS tile (pink) is the loudest object in the panel and draws the eye away from the selected row.
5. Row 1 selected fill spans full width (434-745) while the tabs/field are inset 10 px: fill edge and field edges do not align (by design of rows, noted only).
6. Sheet A is cut by the window bottom mid-row ("long text, resting: ellipsis" caption, field not visible): the part under review is not in the frame.
7. Japanese second lines and Latin second lines have visibly different weight/size ("ガウスぼかし" vs "Blur"); one line mixes both with a "-" separator: acceptable, `unverified` at real size.

## 3. Verdicts

- Search field (A): FIX-THEN-RELEASE. Fixes: show hover of the clear mark; verify (re-capture lower part) the long-text ellipsis and the narrow-dock plate; align the clear mark right inset with the "/" chip inset (or state intent).
- Unified query panel (B): FIX-THEN-RELEASE. Fixes: visible active/none-active tab state (currently no state mark on any tab); remove the repeated kind ("Project" + "Video") per row; fix the near-black blank thumbnails; add a cue for the 62 unshown results; tick/thumb/inset alignment (4/11/10 px); confirm media hue with the owner (OPEN).
- OPEN (owner): accent hue for selection, tab hue, media thumbnail colour.
