# Critique round 2: inspector-gui (Transform gui, Assembled) and dose (All doses side by side)

Judge: independent critic, no code touched. Source: 3 real-window screenshots (lab_book, 1411x840), A = Transform gui, B = Assembled (a)(b)(c), C = All doses (top part only).
Method: PROCESS.md step 5, same format as critique-gui-dose-1.md. I cropped and enlarged regions (3x) of the stills, but they are JPEG stills at 1x, so font px, contrast ratios and 2-4 px gaps stay `unverified`. Axis colours (decision B), accent, dose level are `OPEN`.
Legend: Y / N / OPEN / unverified. Items: 1 hierarchy, 2 colour, 3 depth, 4 consistency, 5 spacing, 6 legibility, 7 states, 8 finish, 9 product-grade.

## A: Transform gui

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | Y | SPACE, POSITION, then Rotation/Scale/Anchor; the pad is the largest element, fields sit under each instrument. |
| 2 colour | OPEN | Axis ticks (red/green/blue bars before X/Y/Z) are decision B. |
| 3 depth | Y | Grey levels only; selected 2.5D plate is lighter with a thin accent underline. |
| 4 consistency | N | Same-kind numbers are formatted differently: Scale "100.0%" vs Anchor "50%" side by side (one decimal vs none); units are tight to the digits ("180.0px", "30.0°") here but spaced ("180.0 px", "30.0 °") in Assembled (a) and (c). |
| 5 spacing | N | Scale X/Y and Anchor X/Y fields are stacked with their borders touching (about 1 px), while the Position X/Y/Z fields have clear gaps between them (measured on a 3x crop; exact px unverified). |
| 6 legibility | unverified | All values now render in full (180.0, 60.0, 0.0, 30.0, 100.0, 50). The "°" glyph after 30.0 is very small (looks about 3 px). Help paragraph is dim grey; size/contrast not measurable. |
| 7 states | unverified | One state only (2.5D, Link chip shown, Z axis tab selected); no hover/drag/locked/mixed. |
| 8 finish | Y | No overflow stripes, no clipped values, no overlap. |
| 9 product-grade | N | An 11-line grey help paragraph sits inside the panel border, about as tall as the whole Rotation/Scale/Anchor row; the "Centre" caption hangs alone under the Anchor Y field with no counterpart under Rotation/Scale. (If the paragraph is Widgetbook story chrome and not product UI, this N drops; I cannot tell from the still.) |

## B: Assembled (a)(b)(c)

Right edge of (c) is cut by the viewport: not counted.

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | Y | Three columns with bold headers "(a) Fields only / (b) Pad, dial, handles / (c) Both"; same section order in each. |
| 2 colour | OPEN | |
| 3 depth | Y | |
| 4 consistency | N | Unit spacing differs between columns: (a) and (c) "180.0 px", (b) "180.0px". Anchor "50 %" vs Scale "100.0 %" decimals differ in (a), (c). |
| 5 spacing | unverified | Gaps look even inside each column. |
| 6 legibility | unverified | Values complete in all three columns (no "60." truncation). In (c) the Rotation dial is drawn very small (about 35 px) in a wide empty box; size not measured to the px. |
| 7 states | unverified | Only the default state. |
| 8 finish | N | Columns (a) and (b) end with a hard horizontal cut at y~728 (in (a) mid Layout row under "Gap/Padding"; in (b) mid Camera "Target/Orbit" fields, bottom of the "Y 0.0 px" fields sliced), with the caption below; there is no scroll indicator, so it looks like clipping. Whether the area scrolls vertically is unverified. No overflow stripes. |
| 9 product-grade | N | See 8. |

## C: All doses side by side (top part)

Only sections 1-3 and the top of 4 (Dose 0/1 tiles, top row of Dose 2) are visible; the lower tiles are not.

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | Y | Section number+name, DOSE n labels, caption line. |
| 2 colour | OPEN | Pink "SCATTER" kicker, badge colours are dose choices. |
| 3 depth | Y | |
| 4 consistency | N | Easing thumbnails: Dose 1 and Dose 2 differ only by a slightly bolder line (Dose 2 adds a notched corner on the selected one), which is hard to read as a step. Doses 0/1 browser tiles show "NEW" without overlap but Dose 2 uses a different header ("BLUR/LIGHT/WARP"). |
| 5 spacing | unverified | |
| 6 legibility | unverified | Captions ("selected Linear", key hints) are small and dim; size not measured. |
| 7 states | Y | Selected vs rest visible in each section (accent outline, plate fill). |
| 8 finish | Y | In the visible area: no clipping or overlap; right-panel label is now complete ("Badges (0-3)"). |
| 9 product-grade | unverified | The part where round 1 found defects (Dose 2/3 browser tiles) is not in this still. |

## (1) Round-1 hard defects

- Shot 1 overflow stripes (2 px): FIXED (none in A or in B (b)).
- Shot 1 truncated "60.", "30.", Anchor "50" without unit: FIXED (60.0px, 30.0°, 50%).
- Shot 3 "Hu"/"Hug" duplicate: unverified (Layout diagram not in these stills; the Layout block in B (a) is cut before its W/H rows).
- Shot 4 five overflow marks (2 px twice, 50 px, bottom stripe, 115 px): FIXED (no stripe or "OVERFLOWED" label in B).
- Shot 4 column (c) unreadable: FIXED for the visible part (readable up to the viewport edge).
- Shot 4 column (a) cut at the Layout row: NOT FIXED (still cut at the bottom of the column, now without a stripe, at the Layout Columns/Gap/Padding row).
- Shot 4 truncated "60."/"50" in (b): FIXED.
- Shot 5 orange badge "3l" clipped (Dose 3): unverified (Dose 3 tiles not in C).
- Shot 5 NEW/GPU badge overlapping crosshair (Dose 2/3): unverified (only Dose 0/1 tiles and the top row of Dose 2 visible; Dose 0/1 do not overlap).
- Shot 5 right-panel label truncated: FIXED ("Badges (0-3)").
- Shot 5 easing doses 1-3 identical: PARTLY FIXED (Dose 1 bolder line, Dose 2/3 notched corner; Dose 1 vs 2 still close).

## (2) New hard defects

None as stripes/clipping inside the visible area. Candidate, not hard: bottom hard cut of columns (a)/(b) in B (listed above as NOT FIXED / item 8).

## (3) Does Transform gui read as numbers abstracted into a hands-on GUI, numbers visually secondary?

N, partly. The abstraction is readable (pad with Z rail, dial, scale box with handles, 3x3 anchor), and instruments are bigger and placed above the fields. But the numbers are not visually secondary: the values are the brightest, boldest (mono, white) marks in the panel, while the instruments are thin grey strokes on near-black; each instrument is followed by a full-width field strip; the 11-line help paragraph under them takes more area than the Rotation/Scale/Anchor instruments. Dial, scale box and anchor are small thumbnails (about 78 px) next to the 215 px pad. Drag/hover/handle affordance cannot be judged from stills.

## (4) Assembled: (a)/(b)/(c)

All three are distinguishable (headers, layout: fields list / instruments / fields beside instruments).
- (a) Fields only: longest column, clipped at the bottom in the Layout row; unit spacing "180.0 px" differs from (b).
- (b) Pad, dial, handles: units tight ("180.0px"); column clipped at the bottom through the Camera fields; Camera orbit diagram is full width while the other instruments are small.
- (c) Both: widest column, right part cut by the viewport (not a defect); instruments are squeezed to the height of the 3-field stack, so the dial is tiny in a wide empty box and the Scale and Anchor instruments are mismatched in size to (b); the same numbers appear twice (field list and the instrument next to it), so the instrument has no extra field strip beneath it, unlike (b).

## Not judged (OPEN)
Axis colours (decision B), accent, dose level (slider at 3).
