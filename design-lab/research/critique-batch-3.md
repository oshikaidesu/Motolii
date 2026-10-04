# Critique batch 3 (independent, FINAL round, live Widgetbook, 2026-10-02)

Method: Parts first sheet at 0.8 scale, first panel use case at scale 1 (1411x840 window). Regression-only (one panel capture): B3 Find, B2 Tag bands, B4, B3 Prefix, B9 Swap, B12 Library, B13 Context, B8 Drop, I2 Number, I2 Grammars, I3 Space, I5 Words, I6 Tone, X Controls, I8 Stack, I11 Clipboard, I9 Lasso, I12 Mixed. Text size / 2-4 px gaps were not zoom-measured except where stated (zoom used for the B3 Tab menu family bar and the B3 Empty query bottom edge). OPEN = accent / tab / thumbnail hue / stage-axis blue (owner-undecided, never N).
Image dir: /Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088/tool-results/ ; files are `mcp-computer-use-blob-<id>.jpg`, only `<id>` listed. P = Parts, U = first panel use case.
Note: Flutter overflow banners ("BOTTOM OVERFLOWED BY ...") are an N on finish (#1).

| component | captures | verdict | remaining N | fix hint |
|---|---|---|---|---|
| B3 Find | P 1790909173432-xarg46; U 1790909180196-l9yj40 | RELEASE | none | - |
| B2 Tag bands | U 1790909185783-ed0y3p | RELEASE | none | - |
| B4 Results band | U 1790909186199-izpn0u | RELEASE | none | - |
| B3 Tab menu | P 1790909191574-iekgsi; U 1790909192009-goiuno; zoom 1790909483262-ikd5zu | RELEASE | none: family bar now has a "›" chevron and the last word fades (FIXED) | - |
| B3 Prefix | U 1790909201678-k7fsj2 | RELEASE | none | - |
| B3 Empty query | P 1790909202176-2armko; U 1790909202586-khb865; zoom 1790909488471-g2m2p7 | RELEASE | none: last row ends in a heavy fade (zoom) (FIXED) | - |
| B2 Chips | P 1790909211625-3eth0j; U 1790909212020-hwur4x | RELEASE | none: band is "8 of 72" only (FIXED) | - |
| B2 Sentence | P 1790909212441-yo0q9r; U 1790909212812-v6bi2f | RELEASE (OPEN) | OPEN: the sentence heading "Soft . Effects . Favorites" restates the tabs/switch above it; this is the variant's thesis (tagged DEPARTS), owner call. Unused hint part gone (FIXED) | - |
| B2 Saved filters | P 1790909225377-9fma0h; U 1790909225793-h0ow4c | RELEASE | none: count appears once beside the place, footer says "67 more hidden" (FIXED) | - |
| B10 Collections | P 1790909226282-cgu8uw; U 1790909226677-gbxgne | RELEASE | none: band "4 of 60" only; row badges are the same grey plate as the slot numbers (FIXED) | - |
| B10 Star and recent | P 1790909240482-h4k3ng; U 1790909241203-scrnno | RELEASE | none: bottom row fades (FIXED) | - |
| B10 Used stack | P 1790909241883-3ltykf; U 1790909242972-5u1hb3 | RELEASE | none: "Effect" column gone, bottom fades (FIXED) | - |
| B12 Similar | P 1790909252749-4xtrf6; U 1790909253169-l57lgs | RELEASE | none: band is "by similarity" only; chip + Source tag name the source once each (borderline, accepted); last tile row fades (FIXED) | - |
| B5 Moving tiles | P 1790909264749-dondn3; U 1790909265128-4sq738 | RELEASE (OPEN) | none; white dot removed (FIXED); picture hue OPEN | - |
| B5 Hover and scrub | P 1790909265542-ewcuhm; U 1790909265903-xwefpo | RELEASE | none: subline is size only when the badge shows kind+length (FIXED) | - |
| B5 Own frame | P 1790909277069-5kvdb2; U 1790909277467-hkt90a | FIX | (a) finish #14: the first list row ("Directional Blur") is cut in half under the Tried/Written strip with no fade, while the bottom fades; (b) #19: key chips "Enter" "Esc" sit inside the panel strip. "Effect" column gone (FIXED) | panel_browser_a.dart (own-frame list): top fade or snap the scroll to a row edge; move Enter/Esc chips outside |
| B5 Preview surface | P 1790909277869-ro5d8s; U 1790909278251-9ivqds | RELEASE | none: "Space" chip gone, "Effect" column gone (FIXED) | - |
| B7 Click | P 1790909292539-j7bmxf; U 1790909292913-nbu3af | RELEASE | none: tile names end in "..." (FIXED); footer legend gone | - |
| B9 Swap | U 1790909293288-3p3t80 | RELEASE | none | - |
| B12 Library | U 1790909300156-kjywzm | RELEASE | none | - |
| B13 Context | U 1790909306988-ydpt5z | RELEASE | none | - |
| B10 Favourites | P 1790909307430-nr2xkv; U 1790909307826-htmi0u | RELEASE | none: Recent strip now fades at the right (FIXED) | - |
| B9 Cards A-B | P 1790909320834-9tyy5t; U 1790909321213-dds3ng | RELEASE | none: the empty ~40 px under the last bar is gone (FIXED) | - |
| B8 Drop | U 1790909321614-c96f0w | RELEASE | none | - |
| B11 Mine | P 1790909333758-sct5do; U 1790909334130-z72sc5 | RELEASE | none: the INSPECTOR stand-in is now a separate "(OUTSIDE)" box beside the panel, not inside it (FIXED) | - |
| B12 Similar Axis | P 1790909334545-yaxoo2; U 1790909334922-xu4b4m | FIX | consistency #9: tile names use a MIDDLE ellipsis ("dusk_rid...01.jpg") while Library and B7 tile names use a tail "..."; within Browser two rules | panel_browser_b.dart: use the Library's tail rule |
| I2 Number | U 1790909350894-thpzl6 | RELEASE | none | - |
| I2 Grammars | U 1790909351287-axekl3 | RELEASE | none | - |
| I3 Space | U 1790909351672-oemunu | RELEASE | none | - |
| I4 Key mark | P 1790909364527-8nn3i9; U 1790909364922-hv8cbb | FIX | legibility: Position X in the pair cell reads "1180." (the .5 is cut by the grip/prefix) at 320 (NOT FIXED) | panel_inspector_a_space.dart / _kit: widen the pair cell value area or drop the decimal rule in pair cells |
| I4 Animate | P 1790909365332-e1uqj9; U 1790909365738-ooffc7 | FIX | same "1180." cut in Position X (NOT FIXED); everything else clean (switch + accent edge + diamond + count) | panel_inspector_a_ease.dart / _kit: same fix |
| I5 Curve | P 1790909377502-vhdys3; U 1790909377869-lq5y9l | RELEASE | none: list ends in a fade; gloss only where it differs (FIXED) | - |
| I5 Words | U 1790909378226-3qowpc | RELEASE | none | - |
| I6 Tone | U 1790909378611-79tr98 | RELEASE | none | - |
| I6 Reset | P 1790909390716-ogdjl0; U 1790909391095-gc9sts | FIX | legibility: "1180." still cut in Position X (NOT FIXED). Heading now wins (full "TRANSFORM トランスフォーム 5 Reset") (FIXED) | panel_inspector_a*.dart: same as I4 |
| I7 Fold | P 1790909391525-7ciscr; U 1790909391895-jblhjs | RELEASE | none: toggle is "[switch] On" like I2 (FIXED) | - |
| I7 Find | P 1790909401702-lbpfte; U 1790909402097-vqv33g | RELEASE | none: "Offset Turbulence X" / "Y" both readable, long label keeps its tail (FIXED); toggle word right (FIXED) | - |
| I7 Pin Recent | P 1790909402508-cwfvy9; U 1790909402899-1vbv6b | RELEASE | none | - |
| X Controls | U 1790909418745-csja9j | RELEASE | none (dim "Undo" still low; unverified) | - |
| X States | P 1790909419195-fw90en; U X5 1790909419543-nm4oa5, X6 1790909419904-ce3cvk, X7 1790909420315-tdt1pp | FIX | finish #1: the State block Parts sheet shows Flutter "BOTTOM OVERFLOWED BY 6 / 21 / 35 PIXELS" banners on the neutral, error and long-JP blocks (clipped text under the stripes). Panels X5/X6/X7 are clean now: X5 one cell grammar + Locked ghost cell (FIXED), X6 title + action words only (FIXED), X7 labels distinguishable (FIXED) | panel_inspector_b_x.dart (state block part): let the block height follow its content (no fixed height) |
| I8 Stack | U 1790909429452-2wsszr | RELEASE | none | - |
| I11 Clipboard | U 1790909437979-1zszk5 | RELEASE | none | - |
| I11 Eyedrop Shelf | P 1790909438931-mass3z; U 1790909439305-k9so2b | FIX | finish/legibility: the "Cancel" button wraps to two lines ("Canc / el") in the active row (new). Grip, chevron, button-shaped Pick, Esc chip gone (FIXED) | panel_inspector_b_i11.dart: widen the cancel cell or use a one-word shorter label |
| I9 Whip | P 1790909448875-tj2h1k; U 1790909449253-11xj7l | RELEASE | none: ON OTHER LAYERS rows are two lines, layer small + property (FIXED) | - |
| I9 Lasso | U 1790909449620-l4ro36 | RELEASE | none | - |
| I9 Macro Inline | P 1790909450030-hhuqfh; U 1790909450496-nr6z2b | RELEASE | none: macro rows are content-height (FIXED) | - |
| I12 Mixed | U 1790909465155-yel6de | RELEASE | none | - |
| I12 Stagger Grab | P 1790909465593-ipvk72; U 1790909465978-1cokx1 | RELEASE (OPEN) | none: one segmented style (Parts = panel), one grip cell grammar (FIXED); the blue bar chart colour is OPEN (stage axis / accent) | - |

## Batch-2 defect status
| batch-2 defect | status |
|---|---|
| B3 Tab menu family bar cut | FIXED (chevron + fade) |
| B3 Empty query half row | FIXED (fade) |
| B2 Chips band restates chips | FIXED |
| B2 Sentence hint part / restate | hint FIXED; restate OPEN (variant thesis) |
| B2 Saved filters "5" x3 | FIXED |
| B10 Collections band/badges | FIXED |
| B10 Star / Used / Own frame / Preview "Effect" per row, cut rows | FIXED (Own frame top row cut NOT FIXED) |
| B12 Similar band triple | FIXED |
| B5 white dot, VID+subline repeat | FIXED |
| B7 Chromatic Aber without ellipsis | FIXED |
| B10 Favourites Recent strip | FIXED |
| B9 Cards rest-card blank | FIXED |
| B11 Mine stand-in | FIXED (moved outside) |
| B12 Axis ellipsis | NOT FIXED in consistency terms (middle vs tail) |
| I4 "1180." cut (Key mark, Animate, I6 Reset) | NOT FIXED |
| I5 Curve half row + repeated glosses | FIXED |
| I6 Reset heading truncation | FIXED |
| I7 toggle word side, identical truncated labels | FIXED |
| X States X5 cell grammar / X6 sentences / X7 labels | FIXED (new: Parts overflow banners) |
| I11 Eyedrop cells / Pick / Esc | FIXED (new: Cancel wraps) |
| I9 Whip truncation, I9 Macro blank | FIXED |
| I12 Stagger styles | FIXED |

## RELEASE-READY (41 of 48)
B3 Find, B2 Tag bands, B4 Results band, B3 Tab menu, B3 Prefix, B3 Empty query, B2 Chips, B2 Sentence (variant OPEN), B2 Saved filters, B10 Collections, B10 Star and recent, B10 Used stack, B12 Similar, B5 Moving tiles, B5 Hover and scrub, B5 Preview surface, B7 Click, B9 Swap, B12 Library, B13 Context, B10 Favourites, B9 Cards A-B, B8 Drop, B11 Mine, I2 Number, I2 Grammars, I3 Space, I5 Curve, I5 Words, I6 Tone, I7 Fold, I7 Find, I7 Pin Recent, X Controls, I8 Stack, I11 Clipboard, I9 Whip, I9 Lasso, I9 Macro Inline, I12 Mixed, I12 Stagger Grab.

## Still FIX (7)
B5 Own frame, B12 Similar Axis, I4 Key mark, I4 Animate, I6 Reset, X States, I11 Eyedrop Shelf.
