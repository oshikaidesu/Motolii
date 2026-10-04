# Critique round 2: Inspector I2 (Parts @200% number cell sheet) and I2-b Face grab

Judge: independent critic, no code touched. Both files are 1411x840 (verified with sips), so the round-1 resolution caveat is gone. Regions were cropped and enlarged 4x (panel) / 2x (sheet) before reading. Absolute px sizes are given only where the enlargement allows a ratio; font size is otherwise `unverified`. Left Widgetbook tree and right knob panel are harness, not judged.

Q from the brief: is the sheet being cut by the right edge a layout defect? YES. The "No Knobs available" side panel does not clip anything by itself; the content area is a fixed wrap grid wider than its viewport, with no horizontal scroll cue. The third column is lost in all 4 grid rows.

---

## A. Inspector I2 "Parts @200% number cell" (sheet)

| # | item | result | observation |
|---|---|---|---|
| 1 | hierarchy | Y | story line (bold title + PARTS @200% tag) > section caps (FACE GRAMMAR, WIDTHS, GRAMMARS) > cells > captions; value text brightest |
| 2 | colour discipline | Y | greys plus two state hues only: editing = blue-violet edge, rejected = red edge. Accent hue: OPEN. Reject red: OPEN |
| 3 | depth | Y | levels of grey only: well g07 under each tile, cell fill darker than tile, edge lifts rest < hover < pressed; no shadow |
| 4 | consistency | N | tile widths and gutters differ between rows (row 1 tiles 164 wide, row 3 pair-126 tile ends at x=566 while the next starts at x=583, row 2 gutter about 15 px); wrap layout means no column aligns from row to row, so like tiles (single 164 at x=583 vs rest at x=306/656) sit on different grids |
| 5 | spacing | unverified | cell-to-caption gap (about 6-7 px) and inter-tile gap (15-17 px) differ by 2 px between rows; not measurable to the px at this scale |
| 6 | legibility | N | the story line and 4 captions are cut by the viewport edge ("focus / pressed g6", "several layers: differs read", "stepper, rest (no buttons y", "sin..."): text is clipped, not wrapped. Captions (rest, hover...) look about 10 px and dim; exact g-level `unverified`. Disabled value (960.0 px locked) looks near g40, under the g56 floor; disabled may be exempt, rule not stated: `unverified` |
| 7 | states | N | the sheet exists to show states, and 4 of 12 are not visible: focused, rejected (only its edge start), several layers, stepper rest, long-number pair 164's second cell, and one single-80 variant. Visible ones differ clearly: rest / hover (edge lighter) / pressed (fill + edge lighter) / editing (violet edge, selection chip) / disabled (value dim) / changed (value g95) |
| 8 | finish | N | right edge clipping (see Q); also "Other kinds of value" caption at the bottom is cut by the window bottom edge with nothing under it, no scroll cue on either axis |
| 9 | product-grade | N | cell design itself reads product-grade; the sheet as a deliverable does not, because a third of the states cannot be seen |

### Round-1 defects (round-1 file is the I2-b panel; this sheet is new, so only the shared items apply)
- End-of-cell marks (caret vs chevron): in the sheet, number cells carry no caret at all, unit text only. FIXED here.
- Units uniform: "px" shown on 164 single cells, dropped below 96 px by rule and labelled so ("single 80: unit dropped"); "%" kept at 164. Consistent with its stated rule. FIXED / by design.

### New fine-detail defects (sheet)
1. Sheet clipped by right viewport edge, no horizontal scroll cue, 3rd column hidden. (layout defect of the sheet, not of the cell.)
2. Text clipped mid-word at the edge (story line "g6", captions "differs read", "no buttons y").
3. Bottom "OTHER KINDS OF VALUE" caption sits on the window edge; content below unreachable by eye.
4. Hover grammar differs without a caption for the shared case: hover edge is lighter than rest by one step, pressed adds a lighter fill; "focused" shows an even brighter near-white edge (partially visible), so hover / focused / pressed edges are three close greys; separation between hover and pressed is small (about 1 step). Exact values `unverified`.
5. "single 164: unit shown" tile is wider than its neighbours and its cell has no left padding visible relative to pair cells; fine, but tile edges (x=583 vs 656) misalign by 73 px against row 1.
6. Stepper cells: "-" and "+" are glyph-weight different (thin minus, bolder plus) at 200%; the minus looks about half the weight of the plus. Also two new symbols (stepper -/+) added to a set DESIGN.md says to keep small; justified by function, flagged only.
7. Label grammar: tag X/Y at rest is dim, at hover g95 (labelled). Tag is vertically centred but its optical baseline sits about 1 px lower than the value; `unverified` at this scale.

### Verdict A: FIX-THEN-RELEASE
Fixes: make the sheet fit its viewport or wrap at its width (or give it a horizontal scroll with a visible bar); no clipped captions; align tile columns across rows.

---

## B. Inspector I2-b "Face grab" panel

| # | item | result | observation |
|---|---|---|---|
| 1 | hierarchy | Y | header (Jewel Field bold, "Shape group" dim) > hint strip > section caps (TRANSFORM etc.) > rows; value text brightest in dark cells |
| 2 | colour discipline | Y | grey only except the "Repeat Edge Pix..." switch, a muted blue-violet (single accent). Accent hue: OPEN |
| 3 | depth | Y | grey levels only: header and hint strip lighter than body, cells darker than body; scrollbar thumb grey; no shadow |
| 4 | consistency | N | (a) rows now carry 2 label inks and the hint states the rule ("A brighter label = changed from default"), but the rule is not applied evenly: Anchor Point is dim while Position/Scale/Rotation/Opacity bright; Blurriness, Blur Dimensions, Repeat Edge are bright; Glow Threshold, Glow Colors, Wave Type, Wave Width dim; Glow Radius, Glow Intensity, Wave Height bright. Plausibly right (changed rows) but Anchor Point 960/540 vs a default is not verifiable; `unverified` that it is true to data. (b) the changed-ink row Blur Dimensions "Horizontal" is bright like a default value; dropdown values are bright regardless of state, number values bright regardless, so the "changed" cue lives only in the label and not in the value, unlike the sheet (value g95 when changed) |
| 5 | spacing | Y | row pitch about 27 px every row; section gap about 25 px above caps with a hairline; cell right edges align across paired and single rows (right edge x=694 for all); label left x=446 for all. 2 px measurement `unverified` |
| 6 | legibility | N | "Repeat Edge Pix..." is ellipsized while the right half of the row is empty except On + switch; label column about 90 px. Section captions (TRANSFORM, GLOW) bright enough; the story caption above, hint strip and footer "undo 0 - nothing yet" look about 10 px and g56-ish, floor not provable: `unverified`. Dim row labels (Glow Threshold, Glow Colors, Wave Type, Wave Width) look about g56 or slightly below; `unverified` |
| 7 | states | N | only rest is shown; hover, scrub, edit, locked of rows and the panel selected state are other use cases. Inside the frame the switch shows On only |
| 8 | finish | N | (a) the last row (Wave Speed / Phase cell) is cut under the footer by about half a row; scrolling container, so the clip is conventional, and a thin scrollbar at the right now exists (round-1 defect partly fixed), but the thumb sits from caption TRANSFORM to about Repeat Edge and reads as short for 7 groups; (b) "Repeat Edge Pix..." truncation; (c) header icon is a hollow rounded square that reads as an unchecked checkbox, not a layer / shape icon |
| 9 | product-grade | N | structure and cells are product-grade; items 6 (truncation), 8 keep it from release |

### Round-1 defects
1. Label ink: two levels without meaning. PARTLY FIXED: meaning is now stated in the hint ("brighter label = changed from default"). Rule application per row: `unverified` (see 4a).
2. Mixed label languages in a group. FIXED for rows (all English now); section captions stay bilingual (TRANSFORM トランスフォーム), consistent across the 4 groups. No residual mixture in rows.
3. Different end-of-cell marks (caret vs chevron). FIXED: Rotation shows "15.0 degree-sign" (unit), dropdowns all use one chevron (Blur Dimensions, Glow Colors, Wave Type), no caret.
4. Unit suffix uniformity. FIXED in the visible rows: every length has "px", angle "deg", ratio "%", and unitless Glow Intensity 1.35 is a true unitless value. Wave Speed 1.00 and Phase not visible: `unverified`.
5. Paired cells aligned: stays good.
6. Header empty right side: NOT FIXED / unchanged (large empty area, no state mark); empty space only, not a defect by itself.
7. Hint strip low contrast: now a lighter strip than header with g60-ish text, readable; floor `unverified`.
8. Panel cut at bottom, no scroll cue: PARTLY FIXED. Scrollbar present; the partial row against the footer remains and there is no padding between last visible row and the footer hairline.
9. Story caption tiny: NOT FIXED (still about 10 px, dim, two lines).
10. No clipped text in the panel. NOT HOLDING: "Repeat Edge Pix..." is ellipsized (new).
11. Hover vs selected: `unverified` (not in this still).
Round-1 verdict was FIX-THEN-RELEASE on items 4, 8.

### New fine-detail defects (panel)
1. "Repeat Edge Pix..." truncated although the control (On + switch) takes only about 85 of the row's 250 px; the label column is fixed at about 90 px and the value area is not given back.
2. Switch row: the control is right-aligned with text "On" to its left; it is the only row whose value is not a boxed cell, so its right edge (x=694) aligns but its height (about 14 px) differs from 24 px cells; acceptable as a different control; not a defect, flagged for hover/state checks.
3. Dropdown value text ("Horizontal", "Original Colors", "Sine") is proportional Inter, number values are monospace; consistent with DESIGN (names Inter, values Menlo) but the dropdown values read about 1 px larger and brighter than the numbers (cap height compared in enlargement). `unverified` to the px.
4. Scrollbar thumb is a flat g? bar 4-5 px wide placed 8 px from the cell edge; it starts below the hint strip; thumb length (about 280 px of 565 visible) seems under-proportioned but not provable.
5. Header icon (hollow rounded square) duplicates the shape/checkbox meaning; two different glyph families (this and the stepper) vs "symbols are few".
6. Section hairline above every caps heading is 1 px g20; the first caps (TRANSFORM) has no hairline above it (the header/hint boundary replaces it); consistent by structure.
7. Unit text ("px", "%") is a dimmer grey than the digits and smaller (about 80%); on a cell with 4+ digits the gap to the unit is 6-7 px; consistent across rows.

### Verdict B: FIX-THEN-RELEASE
Fixes: (1) un-truncate "Repeat Edge Pix..." (give the label column the free width or shorten the name); (2) bottom padding so the last visible row is not cut against the footer; (3) header icon that reads as a layer, not a checkbox; (4) confirm brighter-label rule against data and check dim labels >= g56; (5) capture hover/scrub/edit/locked stills at full resolution.

---

## Summary table
| component | verdict |
|---|---|
| I2 Parts @200% number cell sheet | FIX-THEN-RELEASE (sheet clipped on the right, 4 states hidden, captions cut) |
| I2-b Face grab | FIX-THEN-RELEASE (truncated label, bottom clip, header icon) |
