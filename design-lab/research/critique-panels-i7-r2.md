# Critique round 2: Inspector I7 (Parts @200% sheet, I7-a Advanced fold)

Judge: independent critic, no code touched. Both files are 1411x840 (full capture, verified with `file`). Measured on 3x-6x Lanczos enlargements of crops; JPEG blur limits 1px judgements to "about". Sheet 1 is drawn at 200%, so the 10px / g56 floor cannot be read from it at 1x and is `unverified` there.

Files: vb8wxp.jpg (Parts @200% Folds: group heading states), wei9de.jpg (I7-a Advanced fold).

## A. Parts @200% sheet (group heading states)

| # | item | result | observation |
|---|---|---|---|
| 1 | hierarchy | Y | Sheet title (bold) > caption > "GROUP HEADING" label > state boxes; TRANSFORM caps clearly above the dim count. |
| 2 | colour discipline | Y | Grey only; focus ring is light grey, no hue. Accent: OPEN (none shown). |
| 3 | depth | Y | Closed/open fill #1b-ish, hover about one step lighter, pressed about one more; boxes sit on a darker ground with a 1px border. |
| 4 | consistency | N | The count "14" sits about 2 screen px (1 design px) above the TRANSFORM baseline in every row; the open chevron (down) spans x 325-343 against x 330-342 for the closed one, so its left edge sticks out ~5 screen px. |
| 5 | spacing | Y | Box pitch is a constant 85 px, caption-to-box gap constant, label start x=355 in all foldable rows. |
| 6 | legibility | unverified | Sheet is 200%; 10px/g56 floor at 1x not measurable. At 200% all text is readable; "scaled by layer distan..." sub reads dimmer than the count. |
| 7 | states | N | closed/open/hover/pressed/focused shown, but hover and pressed differ by about one grey step (hard to tell apart), focused changes only the border, no selected state and no disabled state. The "fold row" and "property rows" in the sheet title are not visible. |
| 8 | finish | N | The last box ("dock 200, long, ellipsis") is cut by the window's bottom edge with no scroll cue; the fold row and property-row sections promised in the title are below the cut. Ellipsis cases themselves are intentional and clean. |
| 9 | product-grade | Y | Crisp, even, calm at component level. |

OPEN: accent hue (none on this sheet).

### Round-1 defects against this sheet
Round-1 file covered only I7-a; none of its 8 defects applies here (hover vs selected, defect 7: still not answerable, see A7).

### New fine-detail defects (sheet)
1. Count baseline ~1 design px high vs the heading text (all rows).
2. Open chevron wider than and offset left of the closed one.
3. Hover vs pressed fill difference about one step.
4. Bottom of sheet cut by the window edge; two of the three promised sections not on screen.
5. No selected state for the heading.

Verdict A: FIX-THEN-RELEASE (baseline of count; chevron box; separate hover and pressed; show selected; capture the fold row and property rows; sheet must not be cut).

## B. I7-a Advanced fold panel

| # | item | result | observation |
|---|---|---|---|
| 1 | hierarchy | Y | Layer name > "Fractal Noise" > labels > "Advanced 21"; the fold label is now bold white and readable as a row. |
| 2 | colour discipline | Y | Grey only except the toggle (blue-violet). Accent hue: OPEN. |
| 3 | depth | Y | Effect row one step lighter than the panel, wells darker, 1px well borders and 1px panel border visible at zoom. |
| 4 | consistency | N | Dropdown wells left-align text with a chevron while numeric wells right-align; the "deg" unit is a tiny ring glyph (about 4px) whereas "%" is a full glyph, so units are drawn at different weights. |
| 5 | spacing | Y | Row pitch 27 px constant, wells equal width and left edge, labels share one left edge; fold chevron aligns with the label column. |
| 6 | legibility | N | Unit glyphs "%" and the degree ring measure about 9px (cap height about 0.8 of the digit), below the 10px floor, and dim. Subtitles "Footage layer" / "28 properties" about 10px, borderline: unverified. |
| 7 | states | N | Only the closed fold and the toggle On are shown. Fold hover, open and focus, row hover, and the effect toggle off are not shown (the knob toggle at right is a different control). |
| 8 | finish | Y | No clipping or ellipsis in the panel; hairlines above and below the fold row are visible. |
| 9 | product-grade | Y | Reads as a usable form; fold row now findable. |

OPEN: accent hue of the toggle.

### Round-1 defects
1. Unit inconsistency (Contrast bare): FIXED (Contrast now "100 %"). Residual: the degree glyph is too small (see new 1).
2. Dropdown vs numeric wells differ in alignment: NOT FIXED.
3. Fold row least prominent: FIXED (bold white label); the count "21" is still dim and small, unverified at 10px.
4. Duplicate count in header and fold: FIXED (header "28 properties", fold "21": different facts).
5. Subtitle size/contrast: unverified (about 10px, borderline).
6. Toggle meaning unlabelled: FIXED for On ("On" label); off state not captured.
7. Hover vs selected: NOT FIXED (no state captured).
8. Hairlines not visible: FIXED (visible on zoom).

### New fine-detail defects (panel)
1. Degree ring glyph about 4px, much smaller than "%": units drawn at inconsistent size.
2. "%" and degree glyphs about 9px, below floor.
3. Fold row and open state never captured; "21" and "28" are both bare numbers with different meanings.
4. Panel runs far below the fold row with an empty body (harness height); no problem for the component but the empty region has no end hairline cue.

Verdict B: FIX-THEN-RELEASE (unit glyph size >= 10px and degree mark matching %; align or deliberately differentiate dropdown vs numeric wells; capture fold hover/open/focus, row hover, toggle off; confirm subtitle and "21" size at 1x).
