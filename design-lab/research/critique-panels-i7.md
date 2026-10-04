# Critique: Inspector I7-a "Advanced fold" panel

Judge: independent critic, no code touched. Source: one real-window still (mcp-computer-use-blob-1790903538292-n6zdb3.jpg). The file as read is 847x504, i.e. downscaled from the 1411x840 capture, and no zoom tool was used. Therefore text size, contrast, 1px lines and 2-4px gaps are `unverified` throughout. Author file: book/lib/sets/panel_inspector_b_i7.dart (story at line 238, `_ParamPanel`, fold row at ~185-204).

## 9 items (panel: grain_overlay.mov / Fractal Noise, 7 main / 21 more, fold closed)

| # | item | result | observation |
|---|---|---|---|
| 1 | hierarchy | Y | Layer name (bold) > effect row "Fractal Noise / 7 main - 21 more" > parameter rows > "Advanced 21" fold row; the fold sits last and is visibly quieter. |
| 2 | colour discipline | Y | Panel is grey only; the single hue is the blue-violet toggle (accent, hue OPEN). No relation colours used. |
| 3 | depth | Y | Panel is a slightly lighter surface than the ground, value wells darker; no shadows seen. Step sizes unverified. |
| 4 | consistency | N | Numeric wells have mixed content alignment: "Contrast 100" and "Scale 100 %" right-aligned with a unit, "Complexity 6.0", "Evolution 0.0 deg" and "Opacity 100 %" similar, but Contrast has no unit while neighbours do; the two dropdowns (Fractal Type, Noise Type) are a different well width/shape than the numeric wells. |
| 5 | spacing | unverified | Row rhythm looks even (about 15 px at this scale) but 2-4px gaps cannot be measured. Left edge of labels vs the effect title: roughly aligned. |
| 6 | legibility | unverified | Labels and the fold count "21" look small and dim at this scale; the unit glyphs (%, deg) and the header subtitle "Footage layer - Fractal Noise - 28 properties" look under or near 10px. Not measurable. |
| 7 | states | N | Only the closed fold is shown; the fold's hover and open states, the count "21" meaning (hidden rows) and the toggle's off state are not visible in this still. The only evidence of the fold affordance is a small chevron and the word "Advanced". |
| 8 | finish | Y | No clipping or ellipsis in the panel; all labels and values complete. |
| 9 | product-grade | N | The panel body occupies a narrow column (about 165 px) in a very large empty canvas, the "HABIT" tag and contract text above are harness text; as a component the panel reads as a plain form, but the fold row is easy to miss. Judged on the component only. |

OPEN (not judged): accent hue of the toggle; any axis colours (none shown).

## Fine-detail defects (observable facts)

1. Unit inconsistency: Contrast shows a bare "100" while Scale and Opacity show "100 %"; Evolution shows "0.0 deg" with the degree mark. Mixed unit presence in like cells.
2. The two dropdown wells (Basic, Soft Linear) have a chevron and a different fill/shape than numeric wells; width matches but internal text alignment differs (left vs right).
3. The fold row "Advanced 21" is the least prominent text in the panel yet carries 21 hidden properties; the count is small and dim (size/contrast unverified).
4. The effect header's subtitle "7 main - 21 more" repeats the count that the fold row also shows ("21"): the same fact twice, in two places.
5. Header subtitle line "Footage layer - Fractal Noise - 28 properties" is the smallest and dimmest text; likely below the 10px / g56 floor (unverified).
6. Toggle (top right of the effect row) is the only coloured control; its on/off meaning is unlabelled.
7. Hover vs selected cannot be told apart: no row shows hover in the still (not exercised), so DESIGN rule "rows: g20 fill + 2px g95 tick" is neither confirmed nor refuted.
8. Weight difference between the layer name (heavier) and the effect name is fine; but 1px hairlines between rows are not visible at this scale (unverified).

## Verdict

Inspector I7-a Advanced fold: FIX-THEN-RELEASE.
Fixes for the author (panel_inspector_b_i7.dart / panel_inspector_b_parts.dart):
- Show the unit consistently, or none, across numeric wells (Contrast bare vs Scale/Opacity with "%").
- Drop one of the two "21" counts (header "N more" or the fold row), or make them read as one fact.
- Capture and verify at full size: header subtitle and fold count size >= 10px and >= g56; hairlines.
- Capture the fold hover, open and focus states and the toggle's off state; verify hover is distinguishable from selected.
