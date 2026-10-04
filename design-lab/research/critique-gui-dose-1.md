# Critique: inspector-gui and dose (6 real-window screenshots)

Judge: independent critic, no code touched. Source: Widgetbook (lab_book), macOS, 1411x840 frame.
Method: PROCESS.md step 5 (9 items, Y/N, one concrete observation per N). Text sizes, contrast and 2-4 px gaps were NOT measured (no zoom was possible on these stills), so they are `unverified`. Axis colours (decision B), accent, dose level and direction are `OPEN`, never N.

Legend: Y / N / OPEN / unverified. Items: 1 hierarchy, 2 colour discipline, 3 depth, 4 consistency, 5 spacing, 6 legibility, 7 states, 8 finish, 9 product-grade.

## Shot 1: inspector-gui / Transform gui

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | Y | SPACE / POSITION / Rotation-Scale-Anchor read in order; pad is the largest element. |
| 2 colour | OPEN | Axis bars (red/blue/yellow ticks before X/Y/Z) depend on decision B. Rest is grey. |
| 3 depth | Y | Grey levels only; selected "2.5D" plate is the lighter one. |
| 4 consistency | N | Z field reads "0.0" under the pad but "30." with a clipped value in the Rotation row; units appear as "px", "px", "°", "%" with different gaps. |
| 5 spacing | unverified | Gaps look even; 2-4 px not measurable. |
| 6 legibility | N | The Y field shows "60." (digit cut off), the Rotation Z field shows "30." (cut off), Anchor X/Y show "50" with the unit missing (right edge, rows y~504/526). The description block under the panel is a ~10-line paragraph in dim grey. Size/contrast unverified. |
| 7 states | unverified | Only one state shown (2.5D selected, Link chip, Z chip selected). Hover/drag/disabled not visible. |
| 8 finish | N | Two overflow stripe blocks ("RIGHT OVERFLOWED BY 2.0 PIXELS") at the right of the pad and of the Rotation/Scale/Anchor row. |
| 9 product-grade | N | Debug stripes plus cut-off numbers. |

## Shot 2: inspector-gui / Camera face

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | Y | The orbit diagram is the clear focus; four field groups sit below it. |
| 2 colour | OPEN | Yellow target, pink/red ring, blue sphere and purple camera body are axis/family choices (decision B). |
| 3 depth | Y | Flat, grey-level only. |
| 4 consistency | N | Framing label is lowercase "x" while all other axis labels are uppercase; Roll shows the "°" glyph twice (label and unit) at y~499. |
| 5 spacing | unverified | |
| 6 legibility | unverified | Diagram marks (ticks, small camera glyph) are small; exact size not measured. Help paragraph is dim. |
| 7 states | unverified | "Target is a layer" toggle is off; the disabled state of the point fields is not shown. |
| 8 finish | Y | No overflow, no clipping. |
| 9 product-grade | Y | The only one of the three gui screens with no defect. |

## Shot 3: inspector-gui / Layout mini-diagram

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | Y | Diagram then three number fields then W/H rows. |
| 2 colour | OPEN | Small blue dot and green tick on the diagram are axis colours. |
| 3 depth | Y | |
| 4 consistency | N | The W and H rows show "Hu" at the left of the field and "Hug" at the right of the same field (y~453, y~476): the same word twice in different forms. |
| 5 spacing | unverified | |
| 6 legibility | N | The icon buttons in the W/H rows (arrows, box) are tiny and low contrast; a faint red mark at the right and bottom edge of the diagram (~x726 y318, ~x657 y355) cannot be identified at this resolution. |
| 7 states | unverified | Grid is on; the "Grid off dims the diagram" state is not shown. |
| 8 finish | N | The right ~40% of the diagram canvas is empty, so the mini-diagram looks unfinished (frame holds one 3x2 block at the left only). |
| 9 product-grade | N | See 4 and 8. |

## Shot 4: inspector-gui / Assembled (a)(b)(c)

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | N | Column (c) is covered by a vertical stripe block (x~1025-1105) so its structure cannot be read; the page cannot be compared as three columns. |
| 2 colour | OPEN | |
| 3 depth | Y | |
| 4 consistency | N | The same value is formatted differently between columns: (a) "180.0 px", "30.0 °"; (b) "180 px", "60." , "30. °". |
| 5 spacing | unverified | |
| 6 legibility | N | (b) Y field "60." and Anchor "50" clipped, as in shot 1. (c) is hidden behind stripes. |
| 7 states | unverified | |
| 8 finish | N | Five overflow areas: (b) pad "RIGHT OVERFLOWED BY 2.0 PIXELS", (b) Rotation row same, (c) vertical "RIGHT OVERFLOWED BY 50 PIXELS", horizontal bottom stripe across (a)/(b)/(c) at y~740-815, and "BOTTOM OVERFLOWED BY 115 PIXELS" at y~812. Column (a) is cut at the Layout W row (y~738). |
| 9 product-grade | N | |

## Shot 5: dose / All doses side by side

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | Y | Section number + name, then Dose 0-3 columns, then a caption line. |
| 2 colour | OPEN | Pink kicker "01 SCATTER", orange/green/purple badges are dose-level choices. |
| 3 depth | Y | |
| 4 consistency | N | Dose 1-3 of "3 Easing thumbnails" look identical to each other (white line, same outline), so the doses do not read as steps there; sections 1, 2, 4 do change per dose. |
| 5 spacing | unverified | |
| 6 legibility | N | Section 4 Dose 2/3: the NEW/GPU badges overlap the crosshair glyph at the tile centre (~x366/479/565, y~688). Dose 3: the third (orange) badge reads "3l" with the right part cut by the tile edge (Blur, Light, Warp tiles, ~x760, 880, 997). |
| 7 states | Y | Selected vs rest is visible in every section (accent outline, filled plate). |
| 8 finish | N | Clipped orange badge in all three Dose 3 tiles; the right panel label "Badges requested (the dose cap..." is truncated. |
| 9 product-grade | N | See 6 and 8. |

## Shot 6: dose / 6 Spatial pad

| item | verdict | observation |
|---|---|---|
| 1 hierarchy | N | The pad, dial and scale box have different sizes and different top edges (pad top ~y362, dial ~y367, scale box ~y388); the three do not read as one control group. |
| 2 colour | Y | Grey and white only at dose 1. |
| 3 depth | Y | |
| 4 consistency | N | The readout "X +0.30 Y -0.20 Z 0.40 R 45deg S 1.00 x 1.00" mixes signed (+0.30, -0.20) and unsigned (0.40) numbers, and no units, unlike the Transform gui shown in shot 1 (px, %). |
| 5 spacing | unverified | |
| 6 legibility | unverified | Dial labels (0, 90, 180, 270) and the corner labels X/Y/Z are very small; size not measured. |
| 7 states | unverified | One state only (dose 1). |
| 8 finish | Y | No overflow or clipping. |
| 9 product-grade | N | The pad has a Z rail with a diamond at the right, but in shot 1 the Position pad in 2.5D has none; the two "pads" are different designs. |

## (a) Hard defects

- Shot 1: overflow stripes ("RIGHT OVERFLOWED BY 2.0 PIXELS") at the right of the Position pad and of the Rotation/Scale/Anchor row; truncated numbers "60." (Y), "30." (Rotation Z), Anchor "50" without the unit.
- Shot 3: no overflow; "Hu"/"Hug" duplicated in W and H fields (a content defect, not clipping).
- Shot 4: five overflow marks (2 px twice, 50 px right, bottom stripe, 115 px bottom); column (c) is unreadable; column (a) is cut at the Layout row; truncated "60." and "50" in (b).
- Shot 5: orange badge "3l" clipped at the tile edge in Dose 3 (3 tiles); NEW/GPU badges overlap the crosshair glyph in Dose 2 and 3 (6 tiles); right panel label truncated.
- Shot 2 and shot 6: none.

## (b) Does inspector-gui read as a good abstraction of numbers into GUI (pad, dial, handles, numbers secondary)?

N, as rendered. Reasons:
1. In shot 1 the numbers are NOT secondary: each instrument is followed by its field strip at the same width and the same visual weight, and those fields are the places that are truncated. The two small instruments (dial, scale box, anchor) are thumbnail-sized next to the fields (exact size unverified).
2. The overflow stripes sit exactly on the instruments, so the abstraction is the part that is broken.
3. The Camera face (shot 2) is the one that does read as a graphic with numbers underneath (orbit rings, target dot, camera glyph). That is Y for that screen only.
4. Drag, hover and handle affordance cannot be judged from stills: unverified.

## (c) Are (a) fields / (b) instruments / (c) both distinguishable?

(a) vs (b): yes, clearly (a vertical list of fields vs graphic panels with a few fields). (c) is not distinguishable: it is covered by the 50 px overflow stripe and the bottom stripe, so only its fields column and parts of the instruments are visible. Problems specific to each:
- (a) Fields only: the longest column; it runs off the bottom at the Layout W row (y~738). Values use one decimal ("180.0 px"), the cleanest formatting of the three.
- (b) Instruments: right overflow by 2 px on the pad and the instrument row, bottom overflow by 115 px; values clipped ("60.", "50"); format differs from (a) ("180 px").
- (c) Both: widest column (overflows right by 50 px per the stripe label); fields and instruments repeat the same numbers (X 180, Y 60 in both), which cannot be judged as good or bad from the still, but it is the only column that forces a width the panel does not have.

## Not judged (OPEN, owner's)

Axis colours (decision B, "1 Fam hues (now)"), accent colour, dose level (the slider shows 3 in shot 5 and 1 in shot 6), the Persona/game direction.
