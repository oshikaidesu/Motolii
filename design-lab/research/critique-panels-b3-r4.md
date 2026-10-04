# Critique round 4: Browser B3-a Find unified query (320, query "bl")

Judge: independent critic, no code touched. One 1411x840 real-window capture. Method: pixel peaks (grey 0-255, JPEG noise about 5) plus a 4x enlargement of the top of the panel. Cap height of the second line about 7 px (enlarged); 10 px floor is `unverified` (borderline).

Measured peaks: name 254, tabs 160, count line 150, second line 152, kind 159-160, footer 149, clear mark 170; panel 25, field 19, selected row 52; tick 240.

| item | Y/N | observation |
|---|---|---|
| hierarchy | Y | tabs (rail) > field > count > rows (bold name, bold match, dim second line, kind at right) > footer. |
| colour discipline | OPEN | thumbnail tiles are grey (VHS tile brightest, peak 212); no accent hue shown in this frame. Accent hue and tab hue are owner-undecided. |
| depth | Y | rail darker than panel, field 19, panel 25, selected row 52; no shadow. |
| consistency | Y | left edges unified: rail ~443, field 443, count text 443, thumbnails 444, footer text 443. Only the selected tick sits flush at x 434 (by the rows rule, not an N). |
| spacing | Y | row pitch 33 equal; name and second line share x=484; kind right edge 734 equal to field right edge. |
| legibility | Y / unverified | all secondary text 149-160 (>= g56 peak 142). Size vs 10 px floor: cap height 7 px, borderline, `unverified`. |
| states | unverified | selected row shown. Tab on / hover, clear-mark hover, no-result are not in this frame (no tab is on, so no tab state can show). Not an N. |
| finish | Y | footer "10 shown / nothing more (all places searched)" gives the end cue. Blank band between last row (y ~548) and footer (y ~753) is a fixed-height harness panel; minor. |
| product-grade | Y | no N above. |

## Previous defects (r3 B)
1. Duplicate kind "Project" + "Video": FIXED (lens_bloom.mp4 line 2 is "1920x1080 . 4.2 MB"; kind "Project" only at right).
2. Active tab invisible: NOT FIXED / unverified. All four tabs identical (160) but no tab is on here; the rail now frames them. Check in the tab-on use case.
3. Thumbnails: FIXED for Blur (equal tiles); lens_bloom tile still dark (mean 71) vs neighbours; minor.
4. No cue for unshown results: FIXED (footer; here "nothing more", the "more exist" variant is not in frame, unverified).
5. Dead area below list: NOT FIXED but reduced to a footer-bounded blank; minor.
6. Tick/thumb/inset alignment: FIXED for rail/field/thumb/count (all ~443-444); tick flush at 434 remains.
7. Media hue: OPEN.
8. Secondary text below g56 (r3 new 1): FIXED (149-160 now).
Count line wording "no tab on: all places": unchanged, FIXED earlier.

## Fine-detail defects (facts)
1. "Tilt Shift" has no highlighted letters; it matches only through "Blur" in line 2, which is never bolded in any row, so the reason for this match is not shown.
2. Five Blur rows and two Bloom/Glow rows use identical thumbnail tiles; rows differ by name only.
3. "Directional Blur (Motion Trail, 8-sampl..." is cut mid-word by ellipsis (intended).
4. Tick (x 434-436) touches the panel edge while all other content is inset about 10 px.
5. Selected-row fill spans the full panel width while rail and field are inset (by design).

## Verdict
- Unified query panel (B3-a): RELEASE-READY. No N on hierarchy, colour discipline, consistency, spacing, legibility, states, finish. Carry-over checks (not Ns): tab-on state and clear-mark hover in their own use cases; 10 px floor by real-size zoom; the "N more" footer variant.
- OPEN (owner): accent hue, tab hue, media thumbnail colour.
