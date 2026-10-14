# Dense panels: what Ableton and others publish (research, 2026-10-14 night)

The question: how do dense pro tools make a parameter panel readable at a glance, so the controls speak for themselves (the user: 「パッと見で密度よね、漢は語らない」)?

Words searched: Max for Live Production Guidelines, Short Name / Long Name, Unit Style, Info View (インフォビュー / Info-Ansicht), parameter banks, Best of Bank, Push parameter tabs, Bitwig Remote Controls, NKS parameter mapping, TouchDesigner parameter page, Cavalry attribute editor.

## Rules found
1. **Short names, about 8 characters or fewer.** live.dial: a short name is "generally in a range of 5 to 7 characters" (docs.cycling74.com/reference/live.dial). The Max for Live Production Guidelines give each parameter a Short Name and a Long Name (github.com/Ableton/maxdevtools, m4l-production-guidelines). Push shows names such as "LP Freq" and "Osc 1 Wave": a section prefix, then a short word.
2. **Grey out what does not apply; don't explain it.** TouchDesigner turns a parameter's enable flag off. Push greys out a muted device.
3. **The glyph follows the data type.** NKS parameter types: continuous one-sided (an arc from the left), continuous two-sided (an arc from the centre), discrete (a segmented bar), menu (the word, shown large), switch (a power icon). Cavalry shows a slider only when the attribute has a hard minimum and maximum.
4. **Large value, small grey unit, fixed decimals per unit.** Push displays "20.0 kHz" and "-6.0 dB". An automated value gets a dot.
5. **First bank of at most 8; further banks named by function.** Push and live.banks: banks of 8, the first called "Best of" or Main. Bitwig: 8 per page. NKS: a Performance view and an Edit view.

Not adopted: hover explanations (Ableton's Info View, TouchDesigner's Parameter Help). The user asked for no explaining text.

## What this means for Motolii's cards (cards_dense.jpg)
- Shorten the tile names:
  - "Gap X" / "Gap Y"
  - "Turn"
  - "Turn ea" / "Grow ea"
  - "Wider"

  Keep the plain long name for the timeline and for search.
- Remove "Off while Stroke is None". Grey out Stroke width instead.
- Draw a glyph for each kind of value:
  - Signed values: the centre glyph.
  - Counts: the stepper or a segmented bar.
  - Words: the word shown large instead of an uppercase strip.
  - Unbounded values: no bar.
- Show the unit small and grey, with fixed decimals: s and × to 1 decimal; px, ° and counts to none.
- Rename "Advanced" by what it holds, for example "Each copy" and "Timing".

Not reached: NI's NKS developer guidelines (partner only), an official Push character limit, Bitwig pictures.
