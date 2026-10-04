# Critique: Inspector I2-b "Face grab" (panel_inspector_a.dart, use case 'I2-b Value face, three jobs')

Judge: independent critic, no code touched. Source: 1 real-window still. NOTE: the file actually delivered is 847x504, not 1411x840 as briefed, and I could not zoom beyond that. So text size, contrast and 2-4 px gaps are `unverified` throughout; only structure, relative brightness and presence/absence are judged. Left Widgetbook tree and right knob panel (Effect groups slider) are harness, not judged.

## 9 items (panel at 282 px, 7 groups)

| # | item | result | observation |
|---|---|---|---|
| 1 | hierarchy | Y | header (layer name bold, "Shape group" above it dim) > section captions (small caps dim) > rows; values are brightest, in boxed cells right-aligned |
| 2 | colour discipline | Y | all grey except the Repeat Edge Pixels switch (blue-violet, On) as the single accent. Accent hue itself: OPEN |
| 3 | depth | Y | levels of grey only (panel < header < value cells), no shadow seen |
| 4 | consistency | N | label brightness and language are mixed: "位置 Position", "回転", "不透明度", "波の幅" and several English labels (Anchor Point, Scale, Blurriness, Glow Radius/Intensity, Wave Height) read brighter than their neighbours (Glow Threshold, Wave Type, Phase read dimmer); Anchor Point/Scale/Rotation/Opacity mix Japanese-only, bilingual and English-only labels. Whether bright = "edited" is not stated anywhere on screen |
| 5 | spacing | unverified | row pitch looks even and section gaps larger than row gaps; 2-4 px gaps not measurable at this scale |
| 6 | legibility | unverified | section captions (TRANSFORM トランスフォーム etc.), the story caption above the panel and the footer "undo 0 - nothing yet" look small and dim; the 10 px / g56 floor cannot be confirmed. Value text is clear |
| 7 | states | N | only the normal state is on screen; the knob-less default shows no hover, scrub, edit or locked state (those are other use cases). Within the frame, no cue says which rows differ from default (see 4) |
| 8 | finish | N | the panel is cut at the bottom of the window mid-group (Wave Warp ends at "Phase 0.0" with the footer pinned below it, no scroll cue visible); the Rotation and Direction cells carry a small caret (15.0 v, 90.0 v) that Position/Scale/Blur cells do not, while Blur Dimensions and Wave Type use a right chevron: three different end-of-cell marks |
| 9 | product-grade | N | would pass except for items 4, 7, 8; reads as a credible dense Inspector otherwise |

## Fine-detail defects (observable facts)

1. Label ink: two grey levels among like labels (bright: Anchor Point, Scale, Blurriness, Glow Radius, Glow Intensity, Wave Height, Direction, plus the Japanese ones; dim: Glow Threshold, Wave Type, Wave Speed, Phase, Blur Dimensions). Meaning not shown on screen. Exact values `unverified`.
2. Mixed label languages in one group (Transform): "Anchor Point", "Scale" English-only, "位置 Position" bilingual, "回転" and "不透明度" Japanese-only. Section captions are bilingual, rows are not.
3. End-of-cell marks differ: caret on Rotation and Direction, chevron on dropdowns (Horizontal, Sine, Original Colors), none on plain numbers. Two marks for "this cell has more" (caret vs chevron).
4. Unit suffixes: "%", "px" are small and right of the digits in some cells (Opacity 80 %, Threshold 60 %, Blurriness 24.0 px, Wave Height 34.0 px) but absent on Glow Radius 38.0 and Wave Speed 1.00; units are not uniform across like quantities. Whether the unit is dim enough to be under g56 is `unverified`.
5. Paired cells (Anchor Point, Position, Scale) are two equal boxes; single-value cells span the full width of both, so right edges align. Good; no defect.
6. Header: the layer icon and "Shape group / Jewel Field" sit left with a large empty right side; no visible state mark there. Empty space only, not a defect by itself.
7. Hint line ("Try: drag, click, ...") is a full-width strip under the header at low contrast; 10 px floor `unverified`.
8. Panel bottom: Wave Warp group visibly truncated by the frame; the last rows (Phase) touch the footer with no visible padding. Scroll affordance not visible.
9. Story caption above the panel (I2-b Face grab, HABIT, one-sentence rationale) is tiny and dim, two lines wrapped; size `unverified`.
10. No clipped or ellipsized text in the panel itself; every value and label complete.
11. Hover vs selected: not exercised in this still. `unverified`.

## Fixes for the author (naming only; file book/lib/sets/panel_inspector_a.dart, kit in panel_inspector_a_kit.dart / _parts.dart)

- Make label ink one level, or state what the brighter level means (edited/non-default) in the hint; pick one label language per group.
- Use one end-of-cell mark (caret or chevron) for "opens a choice".
- Show unit suffixes consistently (all or none for like quantities).
- Give the panel a visible scroll cue or bottom padding so the last group is not cut against the footer.
- Verify at 1411 px zoom: text >= 10 px, >= g56 for captions, hint, footer, story caption.

## Verdict

Inspector I2-b Face grab (this still): FIX-THEN-RELEASE (items 4, 8; 5, 6, 7 `unverified` until a full-resolution capture and the hover/scrub/locked stills are checked).
