# Critique batch 2 (independent, live Widgetbook, 2026-10-02)

Method: full-scale screenshots (1411x840 window; Parts at 0.8 scale, panels at 1.0) of the first Parts sheet and the first panel use case of every released component, except the approved B3 Find / B2 Tag bands / B4 Results band and the older Workflow / inspector / inspector-gui (not opened). REWORK items (I4 Animate, X States) had every panel use case captured. Text size and 2-4 px gaps were NOT zoom-measured: 'unverified' where stated. OPEN = accent / tab / thumbnail hue (owner-undecided; never an N). Note: this app only takes clicks at the pointer position (scroll dy 0 first, then click).

Image dir: /Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088/tool-results/ ; every file is named `mcp-computer-use-blob-<id>.jpg`, only `<id>` is listed below. P = Parts sheet, U = first panel use case.

Verdict key: RELEASE = no N on hierarchy / colour / consistency / spacing / legibility / states / finish. FIX = one or a few small N. Batch-1 defects marked FIXED / NOT FIXED / NEW.

## Browser A / Browser B (panel_browser_a.dart / panel_browser_b.dart)

| component | captures | verdict | N items (observation) | fixes for the author |
|---|---|---|---|---|
| B3 Tab menu | P 1790908117744-zit6gx; U 1790908125200-cesdji, 1790908067121-mnj19m | FIX | finish: family bar still cut mid-word at the right edge ("Tin", cut at x~732) with no fade/chevron (NOT FIXED). Footer prose "Tab opens this menu..." gone (FIXED), "+49 more" only (FIXED), family column only in All (FIXED) | panel_browser_b.dart: fade or scroll chevron on the family bar |
| B3 Prefix | P 1790908133418-p51tep; U 1790908133854-54wefd | RELEASE | none. "+15 more" only, no "type more letters" text (FIXED); kind not repeated beside subline (FIXED). Accent OPEN | - |
| B3 Empty query | P 1790908176555-w9ofl0; U 1790908177070-4l387s | FIX | finish: list ends in a half-cut row ("Tint") under ALL OTHERS 47 with no "+N more" or fade | panel_browser_b.dart: fit height or fade/"+N more" |
| B2 Chips | P 1790908185359-7rfz60; U 1790908185801-xsvgzx | FIX | #10: band "2 filters" restates the two chips shown above it; chips + "8 of 72" already say it | panel_browser_a.dart: band "8 of 72" only |
| B2 Sentence | P 1790908194375-597l5o; U 1790908194809-e52u8a | FIX | #10: the sentence heading "Soft . Effects . Favorites" restates the selected tabs and switch just above it (this is the variant's thesis: owner call); Parts keeps an unused hint part "Click a word in the heading to drop that filter." (#19) | panel_browser_a.dart: drop the unused hint part; owner to decide the variant |
| B2 Saved filters | P 1790908207540-li4y8q; U 1790908207992-arujeq | FIX | #10: "5" appears three times (Soft blurs 5, count beside Save as place, 5 rows) | panel_browser_a.dart: show the count once |
| B10 Collections | P 1790908217632-z7bw5u; U 1790908218048-7jxo4z | FIX | #10: "1 filter . 4 of 60 . in 2 Glows" repeats the highlighted slot; row badges "2 5" are unlabelled digits (learnable only from the slot buttons) [21] | panel_browser_a.dart: band "4 of 60"; badge = slot-style chip with the same grey plate as the slot number |
| B10 Star and recent | P 1790908225628-twnlob; U 1790908226061-490kmi | FIX | finish: list cut mid-row at the bottom (ALL 60, half row) with no cue; key chips "F / Enter" outside the panel is ok | panel_browser_a.dart: fit / fade |
| B10 Used stack | P 1790908233919-ikjmyp; U 1790908234359-w4xyaw | FIX | #10: "Effect" in the right column of every row of an Effects list (14 identical words); list cut mid-row at the bottom | panel_browser_a.dart: drop the kind column when the tab fixes the kind |
| B12 Similar | P 1790908240862-te8p6f; U 1790908241323-d7t0v6 | FIX | #10: chip "similar: dusk_ridge.jpg" + "Source" tag + band "1 filter . 36 of 36 . by similarity" say the same thing; last tile row cut with no cue | panel_browser_a.dart: band "by similarity" only; fade |
| B5 Moving tiles | P 1790908269261-gzazuu; U 1790908269718-yowky5 | FIX | #11/#21: a new unlabelled white dot on every tile and "dot 10" in the header (a new symbol with no word); last tile row cut. Picture hue OPEN | panel_browser_a.dart: drop the dot or say it by position/word |
| B5 Hover and scrub | P 1790908277661-jk5f1y; U 1790908278082-udlwk3 | FIX | #10: "VID" badge + "12.0s" on the picture AND "Video . 12.0s" subline; last row cut | panel_browser_a.dart: subline without kind/length when the badge shows it |
| B5 Own frame | P 1790908283971-z2n5vv; U 1790908284465-785v0i | FIX | #10: "Effect" on every row; top row clipped under the status strip | panel_browser_a.dart: as Used stack |
| B5 Preview surface | P 1790908290972-6bncn7; U 1790908291431-f5xd8k | FIX | #10: "Effect" on every row; "Space" key chip beside "15 of 60" is usage text by key (#19); list cut at the bottom | panel_browser_a.dart: as above; chip to story |
| B7 Click | P 1790908304969-4vy8m6; U 1790908305397-faanl3 | FIX | #9: tile name "Chromatic Aber" is clipped with NO ellipsis (JP line has "..."); footer legend gone (FIXED, 2-line wrap gone); bottom tile row cut. Accent underline OPEN | panel_browser_b.dart: one ellipsis rule on tile names |
| B9 Swap | P 1790908313201-jhk21u; U 1790908313632-a1a541 | RELEASE | none. Footer "next / prev", "Link Q" and "Undo x0" are gone (FIXED); duplicate ON LAYER vs card removed (FIXED) | - |
| B12 Library | P 1790908320668-yn0bkc; U 1790908321099-4tu59w | RELEASE | none. Bottom row now fades (cue, FIXED); kind not said twice (tag VID/3D/AUD + dims/time) (FIXED); names end in "..." (FIXED). Default Items=5,000 still gives the tall list but with a fade | - |
| B13 Context | P 1790908332903-r1yh73; U 1790908333376-rbvxgh | RELEASE | none. Stand-in chip row moved out (FIXED), footer gone (FIXED), "54 shown . 9 hidden" now "9 hidden + Show all" (FIXED), last row fades (FIXED) | - |
| B10 Favourites | P 1790908340326-zc3sum; U 1790908340803-ci4y3l | FIX | finish: Recent strip is cut at the right ("Lc") with no cue (NOT FIXED). Footer legend gone (FIXED); badges now match the collection buttons above (FIXED) | panel_browser_b.dart: fade on the Recent strip |
| B9 Cards A-B | P 1790908348307-aowhc4; U 1790908348754-v52qgw | FIX | spacing: the rest card in Parts keeps ~40 px empty under the last parameter bar | panel_browser_b.dart: card height by content |
| B8 Drop | P 1790908360157-doaf1v; U 1790908360604-vr320r | RELEASE | none | - |
| B11 Mine | P 1790908378877-yj6kaw; U 1790908367207-cmsq5k, 1790908367650-ww3qpk | FIX | hierarchy: an INSPECTOR stand-in block (4 rows) takes ~40% of the panel (test rig, as B13 in batch 1); kind chip "Effects" repeats the active tab word | panel_browser_b.dart: move the stand-in to a knob |
| B12 Similar Axis | P 1790908379300-mcguo6; U 1790908379716-4kf2qj | FIX | #9: tile names ("dusk_ridge_00", "wind_pad_001....") are cut with no/uneven ellipsis; last row fades (ok) | panel_browser_b.dart: one ellipsis rule |

## Inspector A / Inspector B (panel_inspector_a*.dart / panel_inspector_b*.dart)

| component | captures | verdict | N items (observation) | fixes for the author |
|---|---|---|---|---|
| I2 Number | P 1790908391212-8iyuzv; U 1790908391665-95df5x | RELEASE | none. Changed = corner mark + brighter label + dotted underline (shape, FIXED); grip "||" on every field; pair cells carry X/Y (FIXED); footer gone (FIXED). Reject red / accent OPEN | - |
| I2 Grammars | P 1790908401524-cue04f; U 1790908401963-yi34do | RELEASE | none. One pair grammar (X/Y) now matches I2-b/I3/I4 (FIXED); decimals in dock are dropped (1181 for 1180.5) but this is a stated rule | - |
| I3 Space | P 1790908409511-wi1hac; U 1790908409936-xottxv | RELEASE | none. Handle now hollow ring (rest) vs filled dot (changed): shape, not brightness alone (FIXED) | - |
| I4 Key mark | P 1790908417587-w5ju7s; U 1790908418011-5jmw8h | FIX | legibility: Position X in the pair cell reads "1180." (the .5 is cut by the key-mark column) in the 320 panel; legend row removed (FIXED) | panel_inspector_a_space.dart / _kit: narrower X prefix or drop the decimal rule symmetrically |
| I4 Animate (REWORK) | P 1790908432125-e1ouln; U Off 1790908432546-dkgl94, On 1790908432968-mcxsse, dock 1790908439521-jncxp9, time lane 1790908439959-fb1kv4 (withdrawn variant) | FIX (was REWORK) | Prose strip is gone, replaced by switch + accent bar edge + hollow diamonds + "22" count (FIXED); off/on differ by switch fill, edge, diamond colour (not brightness alone). N: "1180." cut in the pair cell (320); dock 220 shows 1181 / 540 for the same values (precision drop) | panel_inspector_a_ease/_kit: same fix as Key mark |
| I5 Curve | P 1790908449217-j8btvq; U 1790908449636-zelffw | FIX | finish: preset list ends in a half-cut row ("Cubic In") with no scroll bar or fade; JP glosses repeat ("なめらか" x3, "ふつう" x3) = no information (#10) | panel_inspector_a_ease.dart: fade or +N; gloss only when it differs |
| I5 Words | P 1790908457134-v7nguw; U 1790908457546-e3w68b | RELEASE | none (scroll bar visible; Keys list is a scrolling area) | - |
| I6 Tone | P 1790908469703-swwtm8; U 1790908470139-ld2g69 | RELEASE | none (same grammar as I2) | - |
| I6 Reset | P 1790908475819-9aqw27; U 1790908476267-fk8o7p | FIX | legibility: group heading "TRANSFORM トランスフ..." is truncated to make room for "5 changed + Reset" while other panels show the full name; "1180." cut as in Key mark | panel_inspector_a*.dart: heading wins, count shortens |
| I7 Fold | P 1790908482853-qxipmu; U 1790908483294-mp5k2z | FIX | consistency: header toggle puts the word LEFT of the switch ("On [switch]") while I2 puts it RIGHT ("[switch] On") (NOT FIXED); duplicate "28 properties" vs "Advanced 21 more" is gone, one chevron (FIXED) | panel_inspector_b_i7.dart: one toggle word side everywhere |
| I7 Find | P 1790908492492-9pe8ag; U 1790908492931-k1536e | FIX | consistency: "Off [switch]" word left (NOT FIXED); legibility: two rows both read "Offset Turbulenc..." (and 960 / 540) so they cannot be told apart; labels now ellipsis instead of 3 lines (FIXED) | panel_inspector_b_i7.dart: tail-keeping ellipsis or a 2-line cell for long labels |
| I7 Pin Recent | P 1790908505422-s5x9wb; U 1790908505870-x47yi6 | FIX | same toggle word side and the same pair of identical truncated labels; label left edge is now one x (FIXED); star = shape only (good) | panel_inspector_b_i7.dart |
| X Controls | P 1790908514575-eqfv2i; U 1790908515011-yhxe02 | RELEASE | none. "Try, then keep" and the Kept band are gone, hex shows only on the selected swatch (FIXED). The dim "Undo" at the bottom is below g63 by eye (unverified) | panel_inspector_b_x.dart (optional: lift Undo) |
| X States (REWORK) | P 1790908522421-bilhpz; U X5 1790908522836-nhnz37, X6 1790908531790-qeefln, X7 1790908532226-2e8q73, X8 1790908532647-e3ntup | FIX (was REWORK) | X5: the prose band is replaced by THIS LAYER / SELECTION / ALL KEYS tabs + tick rows + Locked ghost cell (FIXED); N: target cells have no grip or unit while the Opacity cell has both (grammar). X6: each state still has 1-2 sentences ("Pick a layer on the Stage or in the Timeline.") = instruction text inside the panel [#19]. X7: three labels read "カラー化の..." (identical). X8 clean | panel_inspector_b_x.dart: X5 one cell grammar; X6 title + action words only; X7 label truncation |
| I8 Stack | P 1790908544137-nkxx36; U 1790908544554-fhc8lp | RELEASE | none | - |
| I11 Clipboard | P 1790908557680-058doa; U 1790908558086-3l8xb0 | RELEASE | none | - |
| I11 Eyedrop Shelf | P 1790908565338-ooyssm; U 1790908565753-f23eov | FIX | consistency/#21: number cells have no grip and colour cell has no chevron (other Inspector panels have grips); "Pick" is bare text with no button shape; "Esc" chip inside the panel | panel_inspector_b_i11.dart: shared cell widget |
| I9 Whip | P 1790908572970-33ck5r; U 1790908573391-gktxzq | FIX | legibility: ON OTHER LAYERS rows cut the layer + property to "title_card Positio...", "bg_gradient Hue ...", "kick_loop.wav Le..." | panel_inspector_b_i9.dart: two lines or tail-keeping ellipsis |
| I9 Lasso | P 1790908579927-dsvpgn; U 1790908580348-ca21ki | RELEASE | none (locked = hollow ring, followed = filled orange: shape) | - |
| I9 Macro Inline | P 1790908586763-5w1170; U 1790908587180-liv1qq | FIX | spacing: macro row in Parts keeps ~40 px of empty space under the controls | panel_inspector_b_i9.dart: row height by content |
| I12 Mixed | P 1790908597194-etk998; U 1790908597617-859661 | RELEASE | none (Mixed = dash + word + range line + left tick) | - |
| I12 Stagger Grab | P 1790908604391-jd6cmi; U 1790908604834-741ole | FIX | consistency: Parts show the order as plain words (underline + fill), the panel uses an upper-case segmented bar; third number grammar (blue bar with label inside) next to the grip cells | panel_inspector_b_i12.dart: one segmented style, one bar-cell style |

## Batch-1 defects: status

| batch-1 defect | status |
|---|---|
| Instruction/footer text in panels (B3, B7, B9, B10, B13, I2, I4, X Controls) | FIXED (all footers, legends, "Try" bands gone); still present: I4 withdrawn variant none, X6 sentences, B5 Preview "Space" chip, Eyedrop "Esc" chip, B2 Sentence unused hint part |
| State by brightness alone (I2 changed label, I3 handle) | FIXED (corner mark + dotted underline; ring vs dot) |
| One Inspector grammar (X/Y prefixes, decimals, chevron) | PARTLY: X/Y and chevron FIXED; toggle word side (I2 right vs I7/X7 left), number cell (I11 Eyedrop, X5 target rows), value bars (I12) NOT FIXED; dock decimal drop deliberate |
| Same info twice (counts, family/kind) | PARTLY: B3, B12 Library, B13, I7 Fold FIXED; NOT FIXED in B2 Chips/Saved filters, B10 Collections/Used/Own frame/Preview ("Effect" per row), B12 Similar, B5 Hover |
| Clipped last row / non-Fit / tab overflow | PARTLY: Library, B13, Similar Axis fade (FIXED); NOT FIXED: B3 Tab menu family bar, B10 Favourites Recent strip, B5 tiles, B10 Star/Used, B12 Similar, B3 Empty query, I5 Curve |
| Stage stand-in rig inside panel (B13) | FIXED in B13; NEW in B11 Mine |
| Labels wrapped to 3 lines (I7 Find, Pin) | FIXED to ellipsis; NEW: two truncated labels identical |
| I4 prose strip (REWORK) / X States prose (REWORK) | FIXED to form; X6 sentences remain |

## Summary
RELEASE-READY (15): B3 Prefix, B9 Swap, B12 Library, B13 Context, B8 Drop, I2 Number, I2 Grammars, I3 Space, I5 Words, I6 Tone, X Controls, I8 Stack, I11 Clipboard, I9 Lasso, I12 Mixed.
FIX (small): B3 Tab menu, B3 Empty query, B2 Chips, B2 Sentence, B2 Saved filters, B10 Collections, B10 Star, B10 Used stack, B12 Similar, B5 Moving tiles / Hover / Own frame / Preview, B7 Click, B10 Favourites, B9 Cards A-B, B11 Mine, B12 Similar Axis, I4 Key mark, I4 Animate, I5 Curve, I6 Reset, I7 Fold / Find / Pin Recent, X States, I11 Eyedrop, I9 Whip, I9 Macro, I12 Stagger.
REWORK: none.

Most common remaining defects:
1. Same info twice: a kind/family word on every row ("Effect" x14), band counts that restate chips or tabs (B2, B10, B12, B5).
2. Last row cut mid-height with no fade/"+N" cue (B3 Empty query, B10 Star/Used, B5 tiles, B12 Similar, I5 Curve, B10 Recent strip, B3 family bar).
3. Truncation that loses the distinguishing part: identical "Offset Turbulenc...", "カラー化の...", "1180." in pair cells, "Chromatic Aber" without ellipsis (B7, B12 Axis, I7, X7, I9 Whip, I4, I6).
4. Inspector grammar leftovers: toggle word side, number cell without grip (I11 Eyedrop, X5), third value-bar style (I12), Parts vs panel mismatch.
5. Test-rig / empty-reserve space: INSPECTOR stand-in in B11, empty bands inside rest cards in B9 Cards and I9 Macro Parts.
