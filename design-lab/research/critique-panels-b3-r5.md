# Critique round 5: Browser B3-a Find unified query (320, query "bl")

Judge: independent critic, no code touched. One 1411x840 real-window capture, current state. Method: 4x zoom crops (top of panel; bottom rows and footer) plus grey peaks (JPEG noise about 5). Checked against the picture, not against r4.

Peaks: line 2 248 (bolded letters), kind 191, tabs 194, count 179, footer 184, hint 201. All well above g56 (142).

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | tabs > field > count > rows (name, dim line 2, kind right) > footer > hint. |
| colour discipline | OPEN | all grey; accent hue, tab hue, thumbnail hue owner-undecided. |
| depth | Y | field darker than panel, selected row lighter, no shadow. |
| consistency | N | count line says "10 of 72" but the footer says "end of results (all places searched)": two statements of the end state that read as contradictory (10 of 72 vs nothing more). Also the kind word "Effects" repeats on 8 of 10 rows. |
| spacing | Y | equal row pitch about 33; name and line 2 share x=484; kind right edge equals field right edge. |
| legibility | Y / unverified | every text peak 179-248 (above g56). 10 px floor not measurable: cap height about 7 px in the 4x crop is borderline, `unverified`. |
| states | unverified | only the selected row is in frame; no tab on, no hover, no clear-mark hover, no hidden-tag button or plural (none present in this frame). Not an N. |
| finish | Y | footer gives an end cue; blank band under the list is not in this frame (panel ends right after the footer). Hint line sits outside the panel at x 444, aligned with content. |
| product-grade | N | follows from consistency N (count/footer mismatch). |

## Previous defects (r4 fine-detail)
1. "Tilt Shift" match reason not shown: FIXED (line 2 "Blur" now has "Bl" bold; "VHS Wobble" bolds "bl" in the name).
2. Identical thumbnails across rows: still true (five Blur rows alike), minor.
3. Directional Blur cut mid-word with ellipsis: intended.
4. Tick in gutter (x 2-4) vs content at 10: documented rule, not a defect.
5. Selected-row fill full width: by design.

## Verdict
FIX-THEN-RELEASE. One fix: make the count line and the footer agree, e.g. count "10 of 72" with footer "end of results" is contradictory; either show "10 results" or word the footer for what remains ("62 more not matching" / drop the footer when the count already says all shown). Optional: drop the kind word where one place is searched.
OPEN (owner): accent hue, tab hue, thumbnail hue.
Carry-over unverified: 10 px size floor by true-size zoom; tab-on, hover, clear-mark, no-match states in their own use cases.
