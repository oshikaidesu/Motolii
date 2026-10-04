# Critique r2: Browser B4 Results band (panel 3 filters; Band states @200% sheet)

Judge: independent critic, no code touched. Captures 1411x840 real window. Method: PIL pixel probes on the full file (grey 0-255, JPEG noise about 5; thin glyphs peak below true fill). No zoom to nominal font, so the 10 px floor stays `unverified`. Previous: critique-panels-b4.md.

Measured (panel): plate 25, band 32-33, count "3 filters" 236, "2 of 72" 187, "Clear" 187, chip text 212, row name 223, JP line 137 (thin), empty-state hint 138 (thin), footer 156, harness line 161. Chips now have a fill body (probe mean 42 vs plate 25). Sheet @200%: count 255 / "of" 173 / Clear 212 (on) vs Clear 156 (dim, 0 on); hover fill about 52 vs rest about 30.

## A. 9 items (panel capture + states sheet)

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | field, band (count brightest), chips, rows, footer; Clear and "of" sit below the count. |
| colour discipline | Y | greys only inside panel and sheet; accent / tab / thumbnail hue: OPEN. |
| depth | Y | plate 25, band 32, chip body above plate; no shadow. |
| consistency | Y | active filters are now filled chips with an x, as B2 draws them; Clear is a bordered button in both frames. |
| spacing | Y | left edges 443-444 for count, chips, thumbnail, footer; row pitch 33. |
| legibility | Y / unverified | footer 156, count, Clear all >= g56 (142). Empty-state hint peak 138 and JP line peak 137 are thin glyphs: `unverified` against g56 and the 10 px floor (not zoomed). |
| states | N | sheet shows dim, dim+hover (no change), 1, 3, 3+hover, zero, note, similarity: good. Still not in frame: Clear pressed and keyboard focus ring; the sheet's "search text on" and "narrow 232" states are below the fold (cut at y 834), so unverified. |
| finish | N | (1) about 230 px blank between row 2 (y 291) and the empty-state hint (y 512) and then footer (y 766): the footer cue is about 470 px from the list it describes. (2) Glow and Bloom tiles are still the same radial image (means 122 vs 141, same layout). |
| product-grade | N | carried by states (focus / pressed absent) and finish (dead area, twin thumbnails). |

## B. Previous defects

- Active filters not the B2 chip object: FIXED (filled chips with x).
- "(click a word to drop it)" always on screen: FIXED (removed).
- Footer repeats band count: FIXED (footer now only "70 more hidden by the filters"; band says "2 of 72").
- Clear not button-shaped: FIXED (bordered button; sheet shows rest, hover fill, dim).
- Clear hover and dim state not in frame: FIXED (sheet rows 1-5).
- Sheet re-captured at 200% with zero / one / many: FIXED. Search-text-on state: not visible, unverified.
- Dead area / footer far from list: NOT FIXED (an empty-state hint with a target glyph now occupies the middle; the hint repeats nothing but the footer is still 470 px away).
- Glow and Bloom identical tile: NOT FIXED.
- JP second line and chip line measured by zoom: NOT FIXED (still unverified).
- Harness text under the panel repeating the count: NOT FIXED (not product; lab chrome).

## C. Fine-detail defects (facts)

1. Empty-state hint "Drag an effect onto a layer, or double-click to use" appears while 2 results are listed; it describes the list's use but is placed 220 px below the rows, with a crosshair glyph (a new symbol; DESIGN.md: symbols few, needs reason).
2. Hint peak 138 and JP line 137 are under g56 (142) by peak; thin glyphs, `unverified`.
3. Band fill 32-33 vs plate 25: 7-8 levels, lower edge faint.
4. Glow and Bloom thumbnails are the same image.
5. Clear in the panel peaks 187 but in the sheet at "3 filters" 212: panel Clear is dimmer than the "at rest" spec in the sheet (sheet says g76 at rest); capture is 1:1 so thin-glyph peak may explain; `unverified`.
6. Pressed and focus states of Clear and chip x absent from both frames.

## D. Verdict

FIX-THEN-RELEASE. Fixes: close the dead area (anchor "more hidden" to the list end or fill it), give Bloom a different tile, show pressed and focus, zoom-measure hint / JP / chip text for >= g56 and >= 10 px, scroll the sheet to show search-text-on and narrow 232. OPEN (owner): accent hue, tab hue, thumbnail hue.
