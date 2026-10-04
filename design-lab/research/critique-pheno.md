# Silhouette critique: Phenomenon A-E (30 miniatures)

Judge: independent, running Widgetbook (lab_book), Palette B, Words Full unless noted, 2026-10-02. No code touched.
Method: full-resolution screenshot per use case (default state; shows the large 320x200 and the 282 px Inspector row), then one app_drag on the miniature for all 30 (second screenshot at scale 0.6). Words Hidden tested on 12 (A1 A2 A5 A6 B1 B2 B3 B5 B6 C3 D1 D4).
Screenshots: copies in `/Users/member_ottoto/rust_ae/design-sense-lab/research/shots/pheno-<name>.jpg` (default-state full res). Originals/drag shots: `/Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088/tool-results/mcp-computer-use-blob-<ts>-<id>.jpg`, ids are in the table (default / drag).

Columns: Sil = silhouette phrase; Mini = miniature of the phenomenon; N4 = number is quiet 4th element; Tone = calm grey, 1px, fine, dense; Play = playful by detail/small life; WH = purpose clear with Words Hidden (- = not tested); Row = readable and unclipped at 282 px.

| # | Name | Silhouette (one phrase) | Mini | N4 | Tone | Play | WH | Row | Defects | Verdict | shot ids (default / drag) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A1 | Scatter | loose flock of dots around a hollow ring, tiny die in corner | Y | Y | Y | Y | Y | Y (dots merge into a blob) | modest drag took count 36 -> 1 (amount/spread conflated, far too sensitive); die icon ~14px hit zone | FIX (drag sensitivity, row density) | sv6b1p / qxp7pf |
| A2 | Stagger | hand of cards fanned from one hinge | Y | Y | Y | Y | Y | Y | readout "4.0f 0.50 x7" has no unit cue for 0.50; fine otherwise | KEEP | u3a6ph / awk6tw |
| A3 | Repeat | diagonal file of shrinking tiles, ghost slot at the end | Y | Y | Y | Y | - | Y | tiles' white corner square slightly loud at large size | KEEP | qdpvtg / rwbme8 |
| A4 | Grid | tiled frame of empty rounded cells | Y | Y | Y | N | - | Y | static, nothing alive; reads as a layout wireframe/table; drag gap->0 makes cells flash white (loud) | FIX (give the cells small life, e.g. breathing/first-cell mark) else DROP | gzsscq / ln87wp |
| A5 | Falloff | dot field with a ring and a sloped line with two dots underneath | Y | Y | Y | Y | Y (partly: line reads as a slider-ish curve) | N | in the row the region ring clips at 294px; four shape icons ~10px, hit zones unclear; sloped line + 2 handles looks like a slider | FIX (clipping, icon size, curve line) | ytvqhn / 1db0tk |
| A6 | Gravity | three spiral arms of dust round a violet speck | Y | Y | Y | Y | Y | Y | none significant; direction drag works | KEEP | 76blw1 / ey3dq9 |
| B1 | Glow star | eight-pointed star in a soft halo | Y | Y | Y | Y | Y | Y | caption says four-spike, picture has eight; drag shows only radius change on the label | KEEP | grj94o / 5p292j |
| B2 | Refraction ray | slanted glass slab with a kinked beam | Y | Y | Y | Y | Y | Y | source handle can be dragged out of the frame; which gesture changes n is unclear (n 1.45 did not change on source drag) | KEEP (clarify n gesture) | jm36ub / zc2slw |
| B3 | Shadow and sun | small block on dotted floor, shadow, sun on a tether | Y | Y | Y | Y | Y | Y | sun and tether tiny in row; yellow sun is the only warm accent (acceptable) | KEEP (a best toy) | ep9vpp / gb35jn |
| B4 | Depth of field | diagonal file of photo cards, one bracketed, near ones smeared | Y | Y | Y | Y | - | Y | cards ~25px at row size, brackets thin | KEEP | myfgtc / ah7jn9 |
| B5 | Blur bokeh | lone hexagon with a bright dot | Y | Y | Y | N (idle) / Y (after drag) | N | Y | idle state is a bare hexagon (reads as a badge); life only appears on drag (stretches into a streak) | FIX (idle needs small life) | knh0de / iue9bp |
| B6 | Mask feather | pale pebble blob on dotted ground | Y | Y | N | N | N | Y | brightest filled shape in the set: loud against calm grey, flat gradient fill; blob unreadable without words; rim-rub only discovered by drag | FIX (tone down fill; show rim softness at rest) | d6inzh / bxebqe |
| C1 | Roughness | shaded bead with specks, light fan and highlight | Y | Y | Y | Y | - | Y | drag effect subtle (0.25 -> 0.31); gradient sphere heavier than 1px style | KEEP | 8kog6z / ufkihz |
| C2 | Noise | wavy ridge line over faint contours, gravel below | Y | Y | Y | Y | - | Y (ridge touches the right edge) | silhouette risks reading as a line chart; ridge hits row edge | KEEP (watch edge) | cf7i3r / ar65vk |
| C3 | Warp | rubber grid with a bulge | Y | Y | Y | Y | Y | Y | with Words Hidden the caption line above ("Warp | strength + radius | silhouette...") is still visible: Words addon leak in set C | KEEP (fix leak) | 145dts / orwxgj |
| C4 | Extrude | hex block rising out of its footprint | Y | Y | Y | Y | - | Y | none; drag up grows 60 -> 300px nicely | KEEP | 6rjxxs / e8js5u |
| C5 | Stroke | one thick wavy brush line | Y | Y | N | N | - | Y | plain thick near-white line is the loudest, least detailed thing; drag collapsed 12px -> 1px (too sensitive); dash/cap not visible at rest | FIX (tapered/dashed detail, quieter) | 3aowzm / y4kebg |
| C6 | Opacity | small cat behind a misted pane with fingerprint wipe | Y | Y | Y | Y | - | Y | none; drag wipes the mist | KEEP (a best toy) | w0g7vz / rmy56d |
| D1 | Easing | cubic-bezier curve with two tangent handles beside a hop arc | Y | Y | Y | Y (ball on arc) | Y | Y | silhouette is the standard pen-tool bezier editor (precedent, not slider); four unlabeled numbers | KEEP | fzz53w / ejhidf |
| D2 | Spring | coiled spring and ball in a ticked tube, ring-down trace beside it | Y | Y | Y | Y | - | Y | tube with ticks on both sides reads as a vertical gauge/slider until the coil is noticed | KEEP (remove tube ticks) | 753uxu / x2qgso |
| D3 | Wiggle | jittery thread pinned at both ends, pink bead, grey ghost threads | Y | Y | Y | Y | - | Y | reads as an oscilloscope line; fine | KEEP | 2gq9md / ext9xo |
| D4 | Loop | ring with a dot and a diamond seam | Y | Y | Y | N | N | Y | ring + dot = rotary dial silhouette (fails the test); rails/sleepers invisible at rest | FIX (read as a track, not a dial) | fovm27 / a9n8lg |
| D5 | Time remap | long rounded belt with cleats, ball above | Y | Y | Y | Y | - | Y | long thin ticked bar = ruler/slider silhouette; rollers/ball life weak at rest | FIX (not a bar) | 9wa580 / dmpdb8 |
| D6 | Camera shake | viewfinder corner brackets over a mountain line with trail | Y | Y | Y | Y | - | Y | sparse; trail faint at rest | KEEP | o50ogr / nyyxte |
| E1 | Macro | small round face with blush, a column of eight numbers beside it | Y | N (8 readouts) | Y | Y | - | N (right column clipped: "chrm 0.0px", "satu 100%") | number list competes with the face; clipped in row; face squints and glows on drag (good) | FIX (collapse the numbers, fix clip) | 6aomgy / zirgjf |
| E2 | Variations | big face beside a 4x2 shelf of tiny faces, linked from/to rings | Y | Y | Y | Y | - | Y | shelf may read as a bank of 8 round buttons; tiny faces ~25px hit zones | KEEP (check hit size) | yyf3yr / (no drag shot) |
| E3 | React-to | two orbs joined by a sagging wire | Y | Y (3 labels, a bit wordy) | Y | Y | - | Y | wire drag turns it S-shaped, label curve +180 exceeds the +40 default scale without a clear max | KEEP | neco56 / 0cpm67 |
| E4 | Echo | dot on a figure-eight path trailing fading beads | Y | Y | Y | Y | - | Y | none; moving comet is the liveliest | KEEP (a best toy) | eg64qr / 0l2tk0 |
| E5 | Spin | striped top standing on a grainy ellipse floor | Y | Y | Y | Y | - | Y | none; drag on the top changes speed (240 -> 315) | KEEP (a best toy) | e87pet / a3j8yd |
| E6 | Scale | speckled lump of dough with corner ticks | Y | Y | Y | Y | - | Y | none; x 128 -> 181 on drag | KEEP | rq6pv5 / y8ntvo |

## Totals
- Silhouette pass (not slider/knob/dial/XY/bar): 27 of 30. Fail or at risk: D4 Loop (dial), D5 Time remap (bar), D2 Spring (tube reads as gauge, saved by the coil).
- KEEP 19, FIX 11, DROP 0.
- Words Hidden: clear for A1 A2 A5 A6 B1 B2 B3 C3 D1; not clear for B5, B6, D4.

## Top defects
1. Dial / bar silhouettes: D4 Loop (ring), D5 Time remap (belt), D2 Spring tube ticks.
2. Idle state too plain or loud: B5 bare hexagon, A4 Grid, C5 thick white stroke, B6 bright pebble.
3. Drag too sensitive / ranges: A1 count 36 -> 1, C5 12px -> 1px, E3 curve beyond scale.
4. Row view: E1 numbers clipped; A5 region ring clipped, 10px icons.
5. Words addon leak: set C shows its caption line when Words is Hidden.
6. Unclear gesture: B2 where to change n.
