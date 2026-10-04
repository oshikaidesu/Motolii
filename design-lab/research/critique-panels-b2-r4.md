# Critique round 4: Browser B2-a Tag bands, Ableton type (fit height)

Judge: independent critic, no code touched. Capture 1411x840 full resolution; 3-4x zoom crops of chip bands and list. Font px vs 10 px floor: unverified (no ruler). Selected-row tick at x 2-4 vs content at 10 is deliberate (DESIGN.md), not counted.

## 1. Nine items

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | field > chips > "2 filters" (brightest, bold) > rows; clear. |
| colour discipline | OPEN | grey only; lavender underline under Effect/Soft (accent hue owner-undecided). |
| depth | Y | plate < chip < selected chip/row by grey, no shadow. |
| consistency | Y | "+N hidden tag(s)" same text+chevron in all 4 groups, singular/plural now right ("+1 hidden tag", "+7 hidden tags"); Clear is an outlined button. |
| spacing | N | Category wraps Distort onto line 2 though line 1 ends at about x 604 and the 733 edge leaves room for it (Kind fits chip + hidden on one line): premature wrap, 3 groups now 2 lines for no space reason. |
| legibility | Y | labels and second lines read about g150+ (>= g56 floor); no empty-state hint in frame. Px size unverified. |
| states | Y | selected chips (fill + underline), Clear button, hidden-tag chevron, hovered/selected row all present in frame; chip hover/press itself not shown (unverified). |
| finish | N | footer "64 more hidden by the filters" repeats the status "8 of 72" (72-8): the count is stated twice. Minor: Gaussian / Directional / Lens thumbnails are near-identical smears. |
| product-grade | N (partial) | spacing and finish residue only; chip band, list and button read near product level. |

## 2. Round 3 defects

| defect | status |
|---|---|
| "+7 hidden tags" alone on a line | FIXED (shares line with Distort) |
| "+1 hidden tags" plural | FIXED |
| hidden-tag no affordance | FIXED (chevron) |
| Clear not button-like | FIXED (outlined) |
| duplicated instruction | FIXED (rule once below panel; top text only points to it) |
| empty plate before footer | FIXED (list fills to footer) |
| Blur/Glow/Bloom identical thumbnails | MOSTLY FIXED (Bokeh, Tilt, Glow, Bloom distinct; 3 blurs alike) |
| tick vs content edge | not a defect (documented) |
| Category wrap | NEW/remaining: Distort wraps early |
| footer repeats count | NEW |

## 3. Verdict

FIX-THEN-RELEASE (two small fixes): (1) let Category fit Distort on line 1 (wrap only when width is out); (2) drop the footer count or the "8 of 72" (keep one); optional: distinct Gaussian/Lens/Directional thumbnails.
OPEN (owner): accent hue, chip/tab/thumbnail hue.
Stop rule: not met (N on spacing, finish).
