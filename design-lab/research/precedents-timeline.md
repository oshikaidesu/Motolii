# W4 Timeline precedents: Ableton Live / Blender / After Effects

Method: 6 pages opened. Ableton arrangement manual [seen]. Blender dope sheet intro/editing and AE shortcut/keyframe help pages
were NOT usable (Blender pages returned nav-only; Adobe helpx 403 x2). AE facts come from a search snippet [snippet].
No pixel numbers could be read from any official doc. Do NOT treat unmarked cells as measured. DaVinci/Premiere not consulted.
Tags: [seen]=read on page, [snippet]=search excerpt, [inferred]=from memory/general knowledge, unverified.

| Q | Ableton Live | Blender (dope sheet/NLA) | After Effects |
|---|---|---|---|
| Q1 edge grab zone px + hover | px not in manual. Edge drag changes length [seen]; edge cursor change [inferred] | px not found. Strip/key handles; cursor change [inferred] | px not found. Layer bar in/out trim; cursor change to trim arrows [inferred] |
| Q2 key hit vs drawn size | n/a for clips (automation breakpoints: not looked up) | not found; hit area larger than drawn diamond [inferred] | not found; click on diamond [inferred] |
| Q3 snap targets, default | grid, other clip edges, locators, time-signature changes [seen]; default ON (grid) [inferred] | frame-snapped (keys on whole frames), snap-to-frame/second/marker options [inferred]; not read | markers, comp start/end, "significant points" [snippet: Shift-drag layer bar]; default: snap only while Shift held [snippet] |
| Q4 temp toggle, indicator | Cmd (Mac)/Alt (Win) held bypasses grid [seen]; Cmd/Ctrl+4 toggles on/off [seen]. Indicator not stated | Ctrl held inverts snap during G-move [inferred]; indicator not read | Shift held ENABLES snap [snippet]; snap shown as bar jumping (no line known) [inferred] |
| Q5 drag threshold | not documented | Preferences has separate drag threshold for Mouse / Tablet / Touch in px [seen: page confirms they exist, values not shown; ~3px mouse recalled, [inferred]] | not documented [inferred: few px] |
| Q6 zoom/pan | Cmd/Ctrl+wheel or trackpad = zoom; +/- keys; Z zoom to selection, X revert; double-click ruler = zoom to all/selection; Cmd+Opt(Mac)/Ctrl+Alt(Win) drag = pan [seen]. Anchor not stated; Follow mode scrolls with playhead [seen] | Ctrl+wheel = horizontal scroll, wheel = zoom, MMB drag = pan, Home = view all [inferred] | Opt/Alt+wheel zooms at pointer over ruler/time navigator [snippet]; Shift+wheel scrolls horizontally [snippet]; anchor = pointer [snippet]; `;` zoom shortcut [inferred] |
| Q7 marquee vs scrub | Drag on empty area = marquee/timespan selection across tracks; Shift extends [seen]. Scrub = click ruler [inferred] | Drag on empty = box select (select-tweak); ruler/frame-strip drag scrubs [inferred] | Drag on empty timeline area = marquee for keyframes, Shift-click add, Shift-marquee deselect [snippet]; ruler drag scrubs [inferred] |
| Q8 relative timing, frames vs free | Selection moves together, relative offsets kept [inferred]; clips free but grid-snapped by default [seen]; Alt/Cmd off = free (sub-grid) [seen] | Selected keys move together; whole-frame by default, subframe only with snap off [inferred] | Selected keys move together keeping offsets; frame-based [inferred]; Alt+arrow nudge 1 frame [inferred] |

## Consensus / split
| Topic | Consensus | Split |
|---|---|---|
| Drag edge = resize, drag body = move | all three [seen/inferred] | edge grab width unknown for all |
| Empty-area drag = marquee, ruler = scrub | all three (Ableton [seen], AE [snippet]) | none seen |
| Modifier+wheel zoom, other modifier scroll | Ableton/AE [seen/snippet] | which modifier: Cmd/Ctrl (Ableton) vs Alt (AE) vs plain wheel (Blender) |
| Snap default | | Ableton ON, bypass by modifier [seen]; AE OFF, enable by Shift [snippet]; Blender frame-only [inferred] |
| Snap modifier polarity | | hold to disable (Ableton) vs hold to enable (AE) |
| Pan | | Ableton modifier+drag; Blender MMB; AE hand tool/Space [inferred] |
| Zoom anchor | AE = pointer [snippet] | Ableton unstated; Follow mode ties to playhead |
| Move granularity | multi-select keeps offsets [inferred] | grid (Ableton) vs frames (Blender/AE) |

## Candidate Widgetbook options (knobs, options only)
1. Snap: default ON, modifier held = bypass (Ableton model). Alternative: default OFF, modifier = enable (AE model). Make polarity a knob.
2. Snap targets set: {playhead, markers, bar edges, keys, frame grid}; knob per target. Ableton lists edges+locators+grid; AE lists markers+comp ends.
3. Zoom anchor: pointer (AE) vs playhead (Follow-style). Wheel: plain=scroll+modifier=zoom vs plain=zoom. Edge grab zone and drag threshold: sweep widths (e.g. a few px steps) in Widgetbook rather than copy a number; no precedent number was verifiable.

## Not opened / unresolved
Blender dope sheet navigating/editing pages (nav-only), NLA, Adobe AE keyframe + shortcut pages (403), Resolve, Premiere.
Q1, Q2, Q5 numbers remain unknown; need source-code or hands-on measurement.
