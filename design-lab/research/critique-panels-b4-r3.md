# Critique r3: Browser B4 Results band (3 filters, fit height)

Judge: independent critic, no code touched. Capture 1411x840 real window, full res. Method: PIL peak probes on the full file plus a 4x crop of the panel (433-745 x 300-610). Peaks of thin glyphs under-read true fill (JPEG noise ~5). Previous: critique-panels-b4-r2.md.

Measured: plate 25, band 31-33, count "3 filters" 239, "2 of 72" 178, Clear 182, chip text 217, row name 227/208, JP line 156/173, "70 more hidden by the filters" 190, harness line 184. Cap heights at 4x: chip/name ~8 px (11 px font), JP and footer ~7.5 px cap (about 10 px font): at the floor, not under it.

## A. 9 items

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | field, band (count brightest 239), chips, rows, footer; "of 72" and Clear sit below the count. |
| colour discipline | Y | greys only in the panel; accent / tab / thumbnail hue: OPEN. |
| depth | Y | plate 25 < band 32 < chip body 39; no shadow. |
| consistency | Y | chips filled with an x as in B2; Clear is a bordered button; left edges align at x 443-444 (count, chips, thumbnails, footer). Plural "filters" is correct for 3. |
| spacing | Y | row pitch 33, chip row to first row and row to footer gaps even. |
| legibility | Y | all text peaks >= 156 (g56 = 142); JP line 156 thin but above floor; size at the 10 px floor by crop, no zoom to nominal font so exact size `unverified`. |
| states | unverified | this frame is the "3 on" state only: Clear rest, chip rest are fine. Hover, dim, pressed, focus ring are not in this capture (earlier sheet showed hover and dim); pressed and focus never shown in any frame so far. |
| finish | N | panel is set to "Fit list (default)" but about 100 px of empty plate remains under the footer (footer y 493, panel bottom y 594); the footer itself now sits right under the list (fixed). |
| product-grade | Y | carried by the above; the finish gap is cosmetic. |

## B. Previous defects (r2)

- Footer 470 px from the list: FIXED (now 12 px under Bloom; empty-state hint and crosshair glyph gone).
- Glow and Bloom identical tile: FIXED (Glow radial, Bloom ring/ellipse).
- Hint / JP / chip text at floor: now measured, above g56 by peak.
- Footer repeating band count: stays fixed ("70 more hidden by the filters" vs "2 of 72": different, though 70 + 2 = 72 is derivable).
- Hidden-tag button: none in this panel (the "x" chips are the only tag controls, at the right of each chip).
- Pressed / focus absent: NOT FIXED, not in frame.

## C. Remaining facts

1. About 100 px blank under the footer despite Fit-list height.
2. Pressed and focus states of Clear and chip x not shown anywhere.
3. Harness line under the panel ("3 on: the band counts every one...") repeats the sheet caption; lab chrome, not product.

## D. Verdict

FIX-THEN-RELEASE (minor). Fix: make "Fit list" actually fit (drop the ~100 px trailing blank); show pressed and focus of Clear in a frame. If the owner accepts the blank as a min-height, then no N remains and this is RELEASE-READY. OPEN (owner): accent hue, tab hue, thumbnail hue.
