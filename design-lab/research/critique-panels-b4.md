# Critique: Browser B4 Results band (n filters and Clear; Browser 320, 3 filters on, search empty)

Judge: independent critic, no code touched. First review of B4. One capture, 1411x840 real window, panel about 311 px wide at 1:1 (not @200%). Method: PIL pixel probes and bounding boxes on the full-resolution file (grey 0-255, JPEG noise about 5 levels; thin glyphs peak below the true fill). Real px = captured px. Text heights below are glyph bounding boxes (cap/x-height), so the 10 px font floor is `unverified` unless stated. Not in frame: the "Parts @200% Band states" sheet, hover, pressed, focus, zero-filter (Clear dimmed), search-text-on states, narrow dock.

Measured: panel plate 25, band 32 (7 levels above plate), search field text peak 175 (placeholder), count "3 filters" peak 236 (bold), "2 of 72" 187, "Clear" 197, chip words "Effect x / Glow x / Soft x" 229, hint "(click a word to drop it)" 150, row name 229, JP second line 131 (thin), footer "2 shown / 70 more hidden by the filters" 152. Glyph heights: count 9 px box, chips 8 px, hint 9 px, footer 9 px, JP line 9 px. Left edges: field 452 text / panel 434; count 444; chip line 444; thumbnail 443; footer 444 (all within 1 px: left rhythm unified). Row thumbnails (Glow, Bloom): mean 128.6 / 130.6, same tile.

## A. B4 Results band n filters and Clear

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | field, then band ("3 filters" brightest at 236, "2 of 72" and "Clear" at 187/197), chip line, rows, footer; the order reads. |
| colour discipline | Y | panel is greys only; no hue anywhere inside the panel (thumbnails are grey). Lavender is Widgetbook chrome. Accent hue itself: OPEN. |
| depth | Y | plate 25, band 32, rows on plate; no shadow, no border. |
| consistency | N | the three active filters are plain words with a trailing "x" and no chip body (chip line min/mean fill equals plate), while the same Browser's B2 band draws filter chips with a fill (rest 32 / on 66). One filter is two different objects in two components. |
| spacing | Y | left edges unified (field 452 text, count 444, chips 444, thumbnail 443, footer 444); row pitch 32 (Glow y 224, Bloom y 256); band to chip line to first row gaps even. |
| legibility | Y / unverified | hint 150, footer 152, "2 of 72" 187 all at or above g56 (142). JP second line peaks 131 but is thin glyph: `unverified` (could be below g56). Chip line glyph box 8 px high and hint 9 px: 10 px floor `unverified` (not zoomed to the nominal font). |
| states | N | only one state (3 on) is in this use case. Hover of "Clear", hover and pressed of the "x" words, focus ring, Clear when 0 on (dim) are not in the frame; "Clear" at 197 is bright and not button-shaped, so it does not say it is clickable. |
| finish | N | (1) the list stops at y 282 and the footer sits at y 763-772: about 480 px of empty panel between 2 rows and the footer. (2) Footer states "2 shown / 70 more hidden" and the band states "2 of 72": the same fact twice. (3) The Glow and Bloom tiles are the same image. |
| product-grade | N | the three items above (consistency, states, finish); the rest is near product level. |

## 1. Previous defects (B3 r3 and B2) that apply here

- Duplicate kind on the right of each row / ". Blur" repeating the category (B2 B1, B3 B1): FIXED. Rows show name and JP name only; no right-hand kind.
- No cue for hidden results (B3 B4): FIXED in kind. Footer "70 more hidden by the filters" now exists (but see defect 2 below: redundant with the band).
- Dead panel area below the list (B3 B5, B2 finish): NOT FIXED. About 480 px blank between the last row and the footer.
- Left inset 3 / 10 / 11 (B3 B6, B2 B3): FIXED for field, count, chips, thumbnail, footer (443-444). Selected-row tick alignment `unverified` (no row selected).
- Active tab invisible (B3 B2): not applicable here (no tabs in frame).
- Disabled chips take weight (B2): not applicable here (no zero chips in this use case).
- Identical thumbnails for Blur/Glow/Bloom (B2 B6): NOT FIXED (Glow and Bloom same tile, means 128.6 vs 130.6).
- Clear hover not shown (B2 B7, B3 A3): NOT FIXED / not in frame.
- Second-line grey below g56 (B3 new 1): `unverified` (131 thin glyph; footer/hint now 150-152).
- Media hue: OPEN. Accent hue: OPEN.

## 2. Fine-detail defects (observable)

1. Footer "2 shown / 70 more hidden by the filters" (y 763) and band "2 of 72" (y 179): the same count in two places.
2. Footer is pinned 480 px below the last row; the cue is far from the list it describes.
3. "Effect x / Glow x / Soft x" and the hint "(click a word to drop it)" share one line, one size, with the hint only dimmer (229 vs 150); the instruction takes about 107 px of a 311 px line, on screen all the time.
4. The "x" marks are the same weight as the words; no separation between filters other than a space (no chip body).
5. "Clear" (197) is brighter than "2 of 72" (187) and has no shape or underline; clickability is not signalled.
6. Band fill 32 vs plate 25: 7 levels, at the edge of JPEG noise; the band's lower edge is nearly invisible (`unverified` at 100%).
7. Glow and Bloom thumbnails are one tile; the JP second line is the only difference between the two rows.
8. Harness text under the panel ("3 on: the band counts every one...") repeats the band's own count; not product.

## 3. Verdict

- B4 Results band (n filters and Clear): FIX-THEN-RELEASE.
  Fixes: draw the active filters as the same chip object B2 uses (or say why not); remove the footer's duplicate count or merge it with the band; move the "more hidden" cue next to the list end, or fill the dead area; make "Clear" read as clickable (shape, underline or hover state) and show its hover and its dim (0 on) state in the frame; give Bloom and Glow different tiles; re-capture the "Band states" sheet at 200% for the zero, one, many and search-text-on states; measure the JP second line and chip line by zoom (>= g56, >= 10 px).
- OPEN (owner): accent hue, tab hue, media thumbnail colour.
