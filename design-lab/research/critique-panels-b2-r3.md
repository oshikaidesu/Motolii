# Critique round 3: Browser B2-a Tag bands, Ableton type (320)

Judge: independent critic, no code touched. Capture 1411x840 full-resolution. Method: PIL grey peaks (JPEG noise about 5 levels; thin glyphs under-read). Font px vs the 10 px floor is `unverified` (no ruler in frame). g56 reads as peak about 142.

Measured: plate 25; selected row 52; rest chip 43; group labels 149-152; "+N hidden tags" 185-186; counts 184; second line 155; "2 filters" 242; Clear 186; footer 154; empty-state hint 139; tick 243 at x 436-437 (content starts x 443).

## 1. Nine items

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | field > chips > "2 filters" (242) > rows; unchanged. |
| colour discipline | OPEN | grey everywhere, only the underline under on chips is lavender (accent hue owner-undecided). |
| depth | Y | plate 25 < chip 43 < selected row 52 by grey; no shadow. |
| consistency | N | selected tick still sits at x 436-437 while field/chips/labels/thumbnails start at x 443-444: 6-7 px offset, left edge still two lines. |
| spacing | N | Category still wraps "+7 hidden tags" onto its own line (y 232); a 26 px band holds one grey caption. |
| legibility | N | empty-state hint "Drag an effect onto a layer..." peaks 139, below the g56 floor (about 142); footer 154 and labels 149-152 pass narrowly. Font px vs 10 floor unverified. |
| states | N | "+N hidden tags" (186) and Clear (186) are plain text, no hover/pressed look and no sign "+N" expands; no chip hover/focus in this frame. |
| finish | N | (1) about 135 px of empty plate between Ripple (y 616) and footer; the empty hint floats inside it; (2) instruction printed twice (under the title and under the panel); (3) "+1 hidden tags" plural with 1 and does not say the tags are empty. |
| product-grade | N (partial) | consistency, spacing, states, finish defects above; chip band itself close to product level. |

## 2. Previous (round 2) defects

| defect | status |
|---|---|
| tick flush vs content inset (9 px) | NOT FIXED (now 6-7 px by this measure, still two edges) |
| "+7 hidden tags" alone on a line | NOT FIXED |
| "+1 hidden tags" plural / wording | NOT FIXED |
| "+N hidden tags" no affordance | NOT FIXED (brighter, 160 -> 186, but still text) |
| group label vs hidden-note grey nearly equal | FIXED (150 vs 186 now distinct) |
| identical Blur / Glow thumbnails | NOT FIXED (five Blur tiles same smear, Glow = Bloom) |
| Clear / footer different greys, Clear not button-like | NOT FIXED (186 vs 154, no hover shown) |
| duplicated instruction line | NOT FIXED |
| empty plate before footer | NOT FIXED |
| on-chip count dimmer than name | noted, acceptable by hierarchy |

## 3. Fine-detail defects (facts)

1. Tick x 436-437 vs content x 443-444.
2. "+7 hidden tags" alone on y 232 line.
3. "+1 hidden tags": plural with 1; wording does not say zero-count.
4. "+N hidden tags" and Clear: same plain text as labels, no control look.
5. Empty-state hint peak 139 (below g56 about 142), icon and text small.
6. Footer 154 vs Clear 186 vs status 242: three greys in one band.
7. Five identical Blur thumbnails; Glow and Bloom identical.
8. Instruction duplicated above and below the panel.
9. About 135 px empty plate below the list.
10. Chip fill measured 43 vs DESIGN raised g20 (about 32) : OPEN owner.

## 4. Verdict

FIX-THEN-RELEASE. Fixes: align tick/row fill to the 9-10 px content edge; keep "+N hidden tags" on the chip line (or drop Category wrap) and fix plural/wording; give it and Clear a hover/underline control look; lift empty-state hint to g56 or above; distinct effect thumbnails or none; delete one copy of the click rule.
OPEN (owner): accent hue, chip selected fill.
Stop rule: not met (N on consistency, spacing, legibility, states, finish).
