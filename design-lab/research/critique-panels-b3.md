# Critique: Browser B3 Find (parts sheet + unified query panel)

Judge: independent critic, no code touched. Rules: PROCESS.md step 5, DESIGN.md (calm greys, 1px hairlines, text >=10px and >=g56 on dark, colour only for relation families + one accent, few symbols, selected-state rules).
Sources: (A) 1411x840 capture "Parts @200% Find chrome" (search field states). (B) capture "B3-a Find unified query", 320 wide, query "bl"; this image reached me at 847x504 (downscaled), so every size/contrast/gap in B is `unverified`. In A the sheet is shown at 200% (the title says @200%), so rendered sizes are 2x the real ones; real px = rendered/2 and cannot be checked against the 10px floor except by that division. Author file for naming: book/lib/sets/panel_browser_a.dart (_Search l.549, _SearchPart l.3582, _pFindChrome l.3607, B3-a l.801).

## A. Parts @200%: field states

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | Section label "SEARCH FIELD", then a g56 caption per state, then the field on its own plate; the input text (bl, long text) is clearly the brightest thing. |
| colour discipline | Y | Only greys; focus is shown by a lighter (g63-ish) edge, no hue. The only colour is the sidebar's purple selection, which is Widgetbook chrome, not the part. |
| depth | Y | Field sits one grey step above its plate, 1px edge, no shadow. |
| consistency | N | "narrow dock 232" plate fits its field tight (about 16 rendered px of plate each side) while the 300 plates have about 16 px left but a wide right margin to the plate edge (plate 613 wide vs field 580): plate widths differ between rows (613 vs 443) and the "/" key mark is a filled chip in the rest state but absent in focused state, by design ("no / key"); the field height itself looks equal in all (about 54 rendered). The N is the plate/field relationship, not the field. |
| spacing | unverified | Field-to-plate padding looks about 16 rendered (8 real) all round; cannot measure 2-4px gaps from this capture's scale. |
| legibility | Y | Hint text "Search effects, presets, media" is g56-like grey on near-black; at 200% it reads well. Real size = rendered/2 (about 11px), consistent with the 11px name style. Captions (e.g. "rest, empty: hint + / key") are about 8-9 rendered px high at 200% sheet, which would be about 4-5 real px of x-height; they are sheet annotations, not product text, so not judged. |
| states | Y | rest empty, focused empty, typed resting (clear mark), typed focused, long text clipped, narrow dock, are all shown. Hover over the field and the clear mark, and a disabled state, are not shown (not N, missing evidence). |
| finish | N | "long text clips at the edge": the text "Directional Blur motion trail 8-sample GPU soft" stops at about 820 and the clear mark follows with a visible gap, but it is a hard clip with no fade or ellipsis; the neighbouring hint uses ellipsis (maxLines 1, overflow ellipsis, l.577) while typed text does not, so two clip behaviours exist. |
| product-grade | Y with N above | Field reads as a finished input. |

Fine-detail defects (A)
1. Typed text vs hint use different clip treatments (typed: hard cut at the edge, no ellipsis; hint: ellipsis in code). Observable in the "long text" row.
2. Focused edge is a clear step brighter than rest edge, and also thicker looking (about 2 rendered = 1 real px at 200%): OK, but typed-resting vs typed-focused differ only by that edge; the difference is small. Whether it passes at 100% is `unverified`.
3. "/" key mark: lighter chip (g20-ish) with a "/" glyph; at the narrow dock it is slightly smaller-looking than in the 300 field (chip 36 vs 36 rendered, same) : same size, fine. The "/" is a symbol that duplicates the text "Esc clears  /  focuses" help line in B (see B3).
4. Clear mark "x" is a g56-like x, thin; at rest it reads dimmer than the typed text but passes the g56 floor by eye; contrast `unverified`.
5. Sidebar entry text "Parts @200% Find chro..." is ellipsized (Widgetbook chrome, not the product).
6. The 4th plate (typed, focused) and the 3rd (typed, resting) have the same plate height as the empty plates: consistent.

## B. B3-a unified query panel (320 wide, query "bl")

All measurements `unverified` (image downscaled to 847 px; the panel shows at about 185 px wide in the capture).

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | Tabs (Effects, Presets, Project, Mine), field, count line, rows with name bold-matched, second line dim, kind on the right. Query match is shown in bold ("Gaussian **Bl**ur"). |
| colour discipline | N | Thumbnails carry colour (a pink/magenta tile for "VHS Wobble", greenish tile for lens_bloom.mp4): colour not tied to a relation family or accent. They are media previews, so likely legitimate content; flagged as N only against the strict rule, owner to confirm. |
| depth | Y | Panel body one step above the ground; the selected first row has a lighter fill, no shadow. |
| consistency | N | Row 1 "Gaussian Blur" is selected and has a left tick plus fill; the row text is two lines everywhere, but "Directional Blur (Motion Trail, 8-samp..." is ellipsized while all others fit: expected, but the right-hand kind labels (Effects, Mine, Project, Video?) are not on the same x for the "Mine" row (x about 433 vs 432) : negligible. Real N: the right label for the media row says "Project" while its second line says "Video"; the place word appears twice in one row. |
| spacing | unverified | Row rhythm looks even (about 20 px pitch in the capture, 34 real expected from the code comment); 2-4 px gaps not measurable. |
| legibility | N (unverified) | Second lines (Japanese reading + category, e.g. "ガウスぼかし · Blur") and the count line "10 of 72 · all places" / "Esc clears · / focuses" look very small and dim in this capture. Size is `unverified`; the code comment says 10px g56, which is at the floor; the image itself gives no basis to confirm. Needs a 100% real-window zoom. |
| states | N | Only the selected row and rest rows are shown. Hover row, focused ring on the field (the field has a whitish edge, so focus is shown), and no-result are in other use cases (B3-a no match). Hover vs selected cannot be told apart because hover is not in this frame: `unverified`. |
| finish | Y | No clipping of Inter text except the intended ellipsis; thumbnails aligned left; kind column right-aligned. |
| product-grade | N (partial) | Panel itself is close; the HABIT 手癖 tag and the long help sentence above the panel are harness text outside the panel frame, not judged. The panel's own bottom text "Type to filter. Up/Down moves, Enter uses the selected one." sits outside the panel at the bottom and is very dim: `unverified`. |

Fine-detail defects (B), all as observed, size not measured
1. Duplicate place info in a row: right label "Project" and second line "1920x1080 · 4.2 MB · Video" (and "Effects" label with "· Blur" category): the kind is stated at right and again in some second lines ("Video", "Mine"/"Glow" in the sub). Mild duplication.
2. Header line "10 of 72 · all places" and right "Esc clears · / focuses": two key hints in a status line while the field already carries a "x" clear mark; "Esc clears" and the "x" repeat one action (duplicate symbol/word).
3. Four tabs (Effects, Presets, Project, Mine) and a right-hand per-row kind label repeat the same classification. The count label says "all places" while the tabs also imply a place: unclear which tab is active. No tab appears highlighted in this capture: either "all" has no tab, or the active state is invisible. Hidden state, `unverified` at this scale.
4. The selected row shows a left tick and a lighter fill (consistent with the rows rule), plus a thumbnail; fine.
5. Row 1's thumbnail is blank/greyish; others show tiny images: fine.
6. Query "bl" is rendered in the field in normal weight, the match in rows is bold only: consistent.
7. No scrollbar or "more" cue is visible for 72 results, only 10 shown: hidden state (count says 10 of 72, so the cap is stated; no way to see the rest is shown).

## Verdicts

- Search field (A): FIX-THEN-RELEASE. Fixes: (1) make typed-text overflow use the same clip treatment as the hint (ellipsis or a fade), in `_Search` (typed text path near l.577); (2) show hover for the field and the clear mark in `_pFindChrome`, or state it is not designed; (3) the plate widths in `_PC` rows to be one width per row set (300/212 plates currently differ in margins).
- Unified query panel (B, B3-a): FIX-THEN-RELEASE. Fixes: (1) mark the active tab visibly, and say which tab "all places" belongs to; (2) drop one of the duplicated kind statements per row (right label vs second line); (3) one clear-action mark only (x or "Esc clears" in the status line, not both); (4) re-capture at full resolution to verify 10px text and g56 contrast of second lines, count line and hint line, which stay `unverified` here; (5) confirm with the owner that media thumbnail colour is acceptable (colour only for relation families + one accent).
- Colour of the selected-state accent, tab hue: OPEN (owner-undecided).
