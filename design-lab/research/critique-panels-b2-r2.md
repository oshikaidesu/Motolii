# Critique round 2: Browser B2-a Tag bands, Ableton type (320; zero-count chips collapsed into "+N hidden tags")

Judge: independent critic, no code touched. Capture 1411x840 real window. Method: PIL grey peaks (0-255, JPEG noise about 5 levels; thin glyphs under-read) and a 4x zoom crop of the chip band. Panel is rendered at about 0.97 nominal; font size vs the 10 px floor is `unverified` (no ruler in frame).

Measured: plate 25; selected row 52; rest chip fill about 40 (text peak 232); on chip text 238; group labels peak 146-152; "+N hidden tags" peak 160; counts 161; second line 155; status "2 filters" 242; "Clear" 196; footer "8 shown / 64 more..." 156; selected tick peak 243 at x 434-437.

## 1. Nine items

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | field > chips > status "2 filters" (242) > rows; on chips (Effect, Soft) brightest in the band. |
| colour discipline | OPEN | band, rows, thumbnails grey; only the 1 px underline under on chips is lavender (accent hue owner-undecided). |
| depth | Y | plate 25 < rest chip about 40 < on chip about 65 < selected row 52 by fill; no shadow. |
| consistency | N | the selected-row tick sits flush at the panel edge (x 434-437, 0-3 px inset) while field, chips, labels and thumbnails start at x 443-444 (9-10 px); the left edge is still two lines. |
| spacing | N | the Category group wraps "+7 hidden tags" onto a line of its own (y 232) with nothing else on it, so a whole 26 px band is spent on one grey caption; Kind and Source fit the note on the chip line. Rows otherwise even (pitch 33). |
| legibility | Y / unverified | label, hidden note, counts and second line all peak 146-161, above the g56 peak of about 142 that earlier rounds read; the previous 126-136 zero chips are gone. Font px vs 10 floor unverified. |
| states | N | on (fill + 1-2 px lavender underline, now visible in zoom), rest, selected row shown. "+N hidden tags" is plain text with no hover/pressed look and no sign whether it expands; Clear (196) likewise not shown as actionable. Hover/pressed/focus of chips are not in this frame (they are in the Parts sheet). |
| finish | N | (1) about 135 px of empty plate between the last row (Ripple, y 616) and the footer (y 767); (2) the same instruction is printed twice, above the panel and below it ("Click = only that tag. Cmd/Shift-click = add..."); (3) "+1 hidden tags" is ungrammatical for 1 and says "hidden" not what the tags are (zero-count). |
| product-grade | N (partial) | see defects 1-4 below; the chip band itself is close to product level. |

## 2. Previous (critique-panels-b2) defects

| # | defect | status |
|---|---|---|
| B1 | per-row kind "Effect" and category repeating the active filter | FIXED for the right-hand kind (gone). The second line "· Blur / Glow / Distort" remains but Category is not an active filter here (no Category chip on), so it carries information: acceptable. |
| B2 | 8 of 24 disabled chips at same weight as live chips | FIXED: zero-count chips collapsed to one grey "+N hidden tags" per group. |
| B3 | left-edge rhythm 4 / 9 / 10 | NOT FIXED (tick 0-3 / thumbnail 9 / chips 9). |
| B4 | faint 1 px mode line under on chips | FIXED / improved: lavender line clearly visible under Effect and Soft in the zoom (about 2 px, peak distinct from the chip). |
| B5 | dead area under the list | NOT FIXED (about 135 px now, partly the harness size; the footer now sits at the plate bottom). |
| B6 | identical Blur/Glow thumbnails | NOT FIXED: five Blur tiles are the same smear, Glow and Bloom the same dot. |
| B7 | Clear not signalled as clickable | NOT FIXED (196 grey text, no hover shown). |
| B8 | "2 filters" says how many not which | FINE (chips above say which). |
| legibility | zero chips and group labels at 126-136 | FIXED: now 146-161; group label (about 150) vs hidden note (160) are still within 10 levels (nearly the same grey, differentiated only by position). |
| cue "n more" | | FIXED: footer "8 shown / 64 more hidden by the filters". |

## 3. Fine-detail defects (observable facts)

1. Tick flush at x 434 vs content inset at x 443: 9 px difference.
2. "+7 hidden tags" alone on its own line under Category; wasted row.
3. "+1 hidden tags" plural with 1; wording does not say they are empty/zero-count.
4. "+N hidden tags" has no affordance (same grey as a label, no underline or hover); unknown if it expands.
5. Group label grey (about 150) and hidden-note grey (160) nearly equal.
6. Five identical Blur thumbnails; two identical Glow/Bloom.
7. Footer text (156) and "Clear" (196) at different greys for two secondary items in the same band; Clear not button-like.
8. Instruction line duplicated above and below the panel (harness text, minor).
9. About 135 px empty plate before the footer.
10. On-chip count "8" (about 161) dimmer than the name (238) on the brightest object: fine by hierarchy, noted.

## 4. Verdict

- Tag-band panel (B2-a, 320): FIX-THEN-RELEASE. Fixes: align the selected tick inset with the 9-10 px content edge (or inset the row fill like the content); put "+N hidden tags" on the chip line whenever it fits, or drop the Category wrap; make it read as a control (hover colour/underline) or as a count and fix the plural; give the effect thumbnails distinct glyphs or drop them for effects; show Clear hover.
- OPEN (owner): accent hue of the on-chip underline; chip selected fill g20 vs measured about g26.
- Stop rule: not met (N on consistency, spacing, states, finish).
