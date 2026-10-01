> **SUPERSEDED (2026-10-02):** this research assumed a calm Swiss/Tufte-style editorial reading of 'like a magazine'. The owner meant an entertainment magazine cover (e.g. CoroCoro Comic): loud, colourful, playful, dense, with strength from fine detail. See research/pop-magazine.md. Numbers here (hairline weights, type ratios) may still be useful for the 'fine detail' part.

# Editorial / "magazine" precedents for a dark, dense tool

Date 2026-10-02. Scope: collect checkable rules so the owner can choose. No recommendation is made.
Rule of this file: a number appears only if it was in a page I fetched, or in a WebSearch result text (marked `[search]`, weaker: I did not open the page). Anything else is `[inferred]`.
Units: Flutter logical px. `em` = fraction of the font size. Flutter `letterSpacing` is in px, so +0.15 at 10 px = 0.015 em.

---

## 1. Sources

### Opened (page body fetched)
| source | what I got |
|---|---|
| Butterick, Practical Typography: line-spacing, letterspacing, point-size, bold-or-italic, all-caps | numbers below |
| Wikipedia: Data-ink ratio (redirect page) | chartjunk definition, categories; no formula |
| Wikipedia: Sparkline, Small multiple, Leading, Swiss Style, Müller-Brockmann | definitions / quotes, few numbers |
| modularscale.com | named ratios |
| Few/Tufte Design Library (garygisclair.github.io) | token counts, gridline colour, hue limit |
| superdesign.dev dark-mode token sheet | dark surface, text, border values (a third-party sheet, not a shipped product) |
| Material 2 dark theme page | #121212, 87/60/38 %, 4.5:1 |
| WWDC20 "details of UI typography" (transcript) | optical size, tracking idea, leading +-2 pt |
| blakecrosley.com SF Pro article (third party) | Dynamic Type sizes, 20 pt optical break |
| Inter page (rsms.me) | optical sizes, tnum exist |
| Linear redesign blog | Inter Display for headings, LCH, 3 variables, limited chrome colour |
| Figma UI3 blog | what they changed (borders/backgrounds added), 200 icons |
| Things 3.18 blog | 14 text sizes on Mac, everything scales together |
| iA duospace article, iA "web is typography" | no numbers (only reasoning) |
| Ableton redesign case study (third-party concept, not Ableton) | density slider idea; no numbers |

### Not openable or no usable content
- Apple HIG Typography (both URLs): page body empty to the fetcher. Apple numbers are `[search]` snippets only (macOS Body 13/16, Caption 1 10/13; SF tracking +12 @10pt, +6 @11pt, 0 @12pt, -6 @13pt, from a search summary, not a page I saw).
- Adobe Spectrum typography (404), Material 3 type scale (empty), Bloomberg UX story (403), designsystems.com (404), Wikipedia Data-ink_ratio-as-formula (no formula in page), Umich Tufte PDF (binary, unreadable), Apple-Fonts-Documentation (no tracking table), Bergoom/Bloomberg Prop (no metrics).
- No first-hand page for: Bloomberg Terminal, Ableton Live's own UI, Teenage Engineering, Arc/Are.na, Bear, Logic/Final Cut, Pentagram, Rams. **Evidence for "shipped editorial-feeling dense UIs with measurable numbers" is therefore weak.** The owner's best next evidence is measuring screenshots (see section 6), not more reading.
- Tufte's own books were not read; only secondary summaries.

---

## 2. Numbers per source

**Butterick (fetched)**
- Line spacing: "120-145% of the point size" for text (12 pt -> 14.4-17.4). Word "Single" ~117 %.
- Caps: "5-12% extra space with caps, but not with lowercase" = 0.05-0.12 em. Lowercase smaller than 9 pt may get letterspacing to keep letters distinct.
- Print body 10-12 pt; business cards "often only 6-8 points"; web body 15-25 px. "As you reduce point size, also reduce line spacing and line length."
- Emphasis: bold OR italic, "as little as possible"; semibold is "easier to read" than bold with less contrast; small caps named as another option; sans italic is weak.
- The all-caps page itself gives no number, only "always add letterspacing to caps".
- Wikipedia "Leading" summary line says Butterick 20-45 %: that is leading (extra), same range as 120-145 % of size.

**Print rules (WebSearch results `[search]`)**
- Hairline = under 0.25 pt; most common rule is the half-point; safe floor 0.5 pt positive, 0.75-1 pt for a line reversed out of a dark field (spread eats the gap). This last one is directly relevant to light-on-dark.
- 0.25 pt ~ 0.0035 in.

**Modular scale (fetched)**: minor second 1.067, major second 1.125, minor third 1.2, major third 1.25, perfect fourth 1.333, perfect fifth 1.5, golden 1.618, octave 2.

**Tufte (secondary)**
- Data-ink ratio: ink for non-redundant data / total ink; 1.0 is ideal `[search]`. Five rules: show the data, maximise data-ink ratio, erase non-data-ink, erase redundant data-ink, revise.
- Data density = entries in data matrix / area of graphic; "more data per square inch" is better `[search]`.
- Chartjunk categories: unnecessary gridlines, text and fonts, ornamented axes and frames, 3-D, noisy backgrounds.
- Sparkline: "data-intense, design-simple, word-sized"; about the same height as the surrounding text.
- Small multiples: same scales, comparison enforced by the design.
- Few/Tufte library: gridline #d8d4ce (muted), at most 4 hues per chart, 4-5 series before small multiples, direct end-labels replace legends, right-justify numbers, 16 colour / 10 type / 9 spacing tokens (values not given).

**Apple**
- WWDC20 (fetched): SF Text under 20 pt, SF Display from 20 pt; variable font blends 17-28 pt. "Increase letter spacing (tracking) as text gets smaller." Thin parts get sturdier at small sizes. Leading tight/loose = -/+2 pt.
- Crosley (fetched): Dynamic Type default sizes 34/28/22/20/17/16/15/13/12/11 pt (caption2 11).
- Search: tracking +12/1000 em at 10 pt, +6 at 11 pt, 0 at 12 pt, -6 at 13 pt `[search]`. In px for 10/11 pt that is 0.12 and 0.07, i.e. 1.2 % and 0.6 % of em.

**Dark UI**
- superdesign sheet (fetched): surfaces #121212 / #1e1e1e / #2a2a2a / #333333 ("steps roughly 4-6 % in lightness"); text #e6e6e6 (15.0:1), #a3a3a3 (7.4:1), disabled #6b6b6b (3.5:1); border rgba(255,255,255,0.12); "Elevation comes from the lighter surface, not the shadow".
- Material 2 (fetched): base #121212; text opacity 87 / 60 / 38 %; 15.8:1 target; 4.5:1 minimum; desaturated colours.
- Search snippets `[search]`: rgba(255,255,255,.08) hairlines and .06 sheen in one system; ".12" in another; "0.5px for retina table borders, 1px for article rules and list views"; "a half-pixel line at 8-10% alpha all but disappears on 1x screens".

**Shipped UI (fetched, qualitative only)**
- Linear: Inter Display for headings, Inter Regular for body; LCH; 3 theme variables replaced 98; chrome colour (blue) deliberately limited; several surface elevation levels; increased hierarchy and density of navigation.
- Figma UI3: after years of "sharp edges, abstract icons, subtle affordances" they added backgrounds on inputs and borders around dropdowns because it became "harder to parse". Counter-evidence to rules-only minimalism.
- Things: 14 Mac text sizes; icons are vector and scale with text; layout scales as one.
- Figma forum `[search]`: 9 px normal-weight subdued text is hard to read; 10-11 px all caps suggested.

---

## 3. How emphasis is made without weight (ranked by strength of evidence)

1. **Size contrast on a ratio, few steps.** Modular ratios 1.125-1.333 are the usable range for a dense UI [inferred]; Apple's own list at small sizes is 10/11/12/13 (1.08-1.18 steps) and changes the cut (Text vs Display) instead of the weight at 20 pt (WWDC20, Crosley). Evidence: strong, but "magazine" scale ratios were not fetched.
2. **Colour value / opacity steps, not hue.** Material 87/60/38 %; superdesign #e6e6e6 / #a3a3a3 / #6b6b6b; Linear limits chrome hue. Three text values cover primary, secondary, disabled. Evidence: strong.
3. **Caps + tracking.** Butterick 5-12 % on caps, none on lowercase; WWDC: smaller text gets more tracking. Caps are the one way to get a label to read as a label without bold. Evidence: strong for the number, medium for use in a UI.
4. **Surface step instead of a line or shadow.** 4-6 % lightness per tier (superdesign, Material). Evidence: medium (sheet and spec, no product measured).
5. **Position / alignment / flush-left grid.** Swiss Style: flush-left, asymmetric, modular grid, whitespace (Wikipedia; Müller-Brockmann). No numbers; the grid unit is the designer's. Evidence: strong as principle, empty as numbers.
6. **Rules instead of boxes, thin and muted.** Print: half-point most common; Tufte: gridlines muted (#d8d4ce); chartjunk list names frames and gridlines. Evidence: strong in print and Tufte; Figma UI3 is a documented counter-case (boxes were added back for parsing).
7. **Direct labelling and word-sized graphics.** Tufte: end-labels replace legends, sparklines same height as text. Evidence: strong for charts, [inferred] for chrome (a value readout beside its mark).
8. **Typographic details: tabular numerals, optical size, small-size tracking, semibold over bold.** Inter has tnum and a text/display split; Butterick says semibold has less contrast than bold. Evidence: strong that they exist, no measure of how much "presence" they add.
9. **Whitespace/leading.** Butterick 120-145 % for paragraphs. Evidence: strong for text, not for a 23 px row [inferred].

Not found: any fetched page that explains "presence from fine detail" as a numeric method. That synthesis is inference from 1-9.

---

## 4. Draft vs precedents

Draft = DESIGN.md + book/lib/tokens.dart. Hex of g20 on g10: (0x34-0x19)/(255-0x19) = 11.7 % white-equivalent [inferred calc].

| draft rule | verdict | evidence |
|---|---|---|
| Inter 11 name / 10 label / 10 micro, Menlo 11 | AGREES | Apple caption 10-11 pt, Dynamic Type 11/12/13 (Crosley); Figma forum: 9 px weak, 10-11 caps fine. Butterick: print cards 6-8 pt is the floor. Chrome text at 10-11 px is normal for dense tools. |
| Only 3-4 sizes (13 title, 11, 10, 10) | AGREES, but narrow | Ratio 13/11 = 1.18, 11/10 = 1.1: below minor third. Few/Tufte keeps 10 type tokens. Editorial scales jump 1.2-1.5 [inferred]; the draft has almost no size contrast, so presence has to come from elsewhere. |
| Tracking +0.05 / +0.1 px on lowercase (name, label) | AGREES | SF at 10-11 pt: 1.2 % and 0.6 % of em [search]; draft 0.1/10 = 1.0 %, 0.05/11 = 0.5 %. |
| micro +0.15 px "small caps / marks" | DIFFERS | If these are caps, Butterick 5-12 % = 0.5-1.2 px at 10 px. Draft is 1.5 % = 3-8x lower. If they are lowercase, it agrees. Check whether micro is ever set in caps. |
| Line height 1.0 | DIFFERS (paragraphs) / NEUTRAL (single line in a fixed row) | Butterick 120-145 %; Apple macOS 13/16 = 1.23, 10/13 = 1.3 [search]. Single-line cells have no leading to speak of; true for 23 px rows [inferred]. title 1.1 is the only multi-line-safe one. |
| Weights 500 names, 600 title, 400 labels | PARTLY DIFFERS | The brief says emphasis not by weight. Butterick: semibold = lower contrast than bold, so 500/600 is the mild end; but it is still weight doing the job. |
| 1 px solid g20 borders | PARTLY AGREES | Value ~ rgba(255,255,255,0.12) (superdesign 0.12; other systems 0.06-0.08 [search]). Weight differs: print uses half-point, retina table borders 0.5 px [search]; 1 logical px on 2x is 2 device px. Print reversed-out lines want 0.75-1 pt (print search): so 1 px is not "too thick" for light-on-dark. |
| Depth by grey level, no shadow except floating | AGREES | superdesign: elevation from lighter surface; Material overlays. Draft bands #1E1E1E / #2A2A2A equal superdesign tiers 1 and 2 exactly; ground #191919 is slightly lighter than #121212. |
| Bands alternate #1E / #2A with #16 row line | DIFFERS from "rules not boxes" | #2A vs #1E is a ~6 % step, the top of the 4-6 % tier range, and bands plus a line is two separators. Tufte (erase redundant ink) and Swiss (rule or space) would use one. |
| Row 23 px, 4 px rhythm, label col 180 | UNVERIFIED | No fetched source gives a row height. 23/11 = 2.1 line box (derived). 100/23 = 4.3 rows per 100 px [inferred]. Apple body 16 line + 4 px = 20-24 row is plausible, not evidenced. |
| Colour only on relation marks, greys elsewhere | AGREES | Linear limits chrome colour; Tufte <=4 hues per chart (Few/Tufte); Material desaturated colours. Six families exceeds 4 hues if they show on one screen at once. |
| Text floor 10 px and g56 (#8E8E8E) | AGREES | Computed contrast 5.4:1 on #191919 [inferred calc]; >4.5:1 (Material), above superdesign "secondary" role, below its 7.4:1; Material medium emphasis 60 % white on #121212 ~ similar. |
| Ruler labels Menlo 9.5 px | DIFFERS | Below the 10 px floor the draft itself sets (and the Figma-forum caution on 9 px). Probably an old line. |
| Selected = fill + 2 px tick; tabs = pill + 1 px underline | NEUTRAL | No precedent fetched. A tick/underline is a rule-style emphasis (technique 6), consistent with editorial. |
| UI scale 50-200 %, hairlines snap to device px | AGREES | Things scales all sizes together (14 steps); half-pixel at 1x vanishes `[search]`, so snapping is right. |

---

## 5. Three distinct options for "magazine-like" (not ranked, no recommendation)

All three keep: no shadows on chrome, no weight above 500, hue only on relation marks. Values are starting points to be tuned in the real window.

### Option A. "Ruled column" (Swiss rules, caps labels)
- Separation: rules, no boxes, no bands. Ground flat; one tier (+4 %) only for the Stage well. Rule = 1 device px, white 10 % (range 8-12), 0.75 device-px equivalent not available on 1x.
- Type: 3 sizes, ratio 1.2: 10 / 12 / 14.4 (round 10 / 12 / 14). Values Menlo 11 tabular. Lowercase 12 regular, tracking +0.005 em; section label 10 px CAPS +0.10 em (Butterick 5-12 %), colour 60 % white.
- Emphasis: caps + tracking for labels, size for titles, 87 / 60 / 38 % text steps. No weight above 400 except title 500.
- Grid: 4 px, label column and value column fixed; flush-left, values right-aligned (Few/Tufte).
- Density target: row 22-24 px, 4.3 rows / 100 px, panel gutter 12-16 px.
- From: Butterick caps; Swiss flush-left grid; print half-point rule; Tufte erase redundant ink; Material opacity steps.
- Risk: Figma's lesson (rules alone harder to parse in input-heavy panels).

### Option B. "Quiet tone" (Tufte + dark elevation)
- Separation: grey step only. Tiers #131313 / #191919 / #202020 (steps ~3-4 %); no borders except a 1 device-px white 6 % keyline on inputs (Figma added borders for parse). Band stripes removed or reduced to 2-3 % step.
- Type: 4 sizes, ratio 1.125: 10 / 11 / 12.5 / 14. Names 11 regular, text steps #E7E7E7 / #A1A1A1 / #717171. Tracking 0 to +0.01 em, no caps except 2-3 fixed labels.
- Emphasis: colour value and surface step only; direct labelling (value inside or at the end of its mark); sparkline-sized graphics at text height.
- Density target: higher than A: row 20-22 px, 4.5-5 rows / 100 px; data per area is the metric (Tufte).
- From: superdesign/Material elevation; Tufte data-ink, direct labels, sparklines; Linear limited chrome.
- Risk: presence depends entirely on the grey ladder; zero hairlines may read as "flat".

### Option C. "Scale contrast" (editorial hierarchy)
- Separation: one hairline under group headings only (1 device px, white 14 %), whitespace elsewhere (gaps 8 / 16 px). No box, no band.
- Type: 4 sizes, ratio 1.333: 9.5-10 (not below 10) / 13 / 17 / 22 for hero numerals only (a big time or value readout, Inter Display-like light at 300-400). Body 11-12. Caps labels 10 px +0.12 em; numerals tabular, lowercase +0.01 em.
- Emphasis: size contrast on a few key readouts (current time, selected value), caps labels, position; everything else small and grey (60 %).
- Density target: row 22 px, but 2-3 big readouts per panel anchor the eye; calm comes from the contrast, not the spacing.
- From: Butterick scale and 5-12 % caps; Apple Text/Display split and 17-28 pt blend; Linear Inter Display for headings; Swiss hierarchy.
- Risk: large numerals eat area; the whole density target is traded for presence.

(Option values are [inferred] assemblies from the cited numbers, not copies of any shipped product.)

---

## 6. What to build first (to compare A/B/C in the real window)

A "presence study" screen in the real Flutter window, not screenshots: 6-8 representative widgets rendered with one theme object and live knobs:
1. Inspector property row (label / value / unit, with slider track).
2. Section heading + group of 5 rows (the separation test: rule vs band vs grey step).
3. Timeline 4 rows (layer label, bar, key) with ruler.
4. Tab / segmented control with selected state.
5. List row in Browser, selected + hover.
6. Numeric readout (time / value) at scale ladder.
7. Menu / tooltip (only floating thing with a shadow).
8. A text input with and without keyline.
Knobs: separator style (rule / band / grey step / none), hairline alpha 6-14 % and 1 device px vs 0.5 px, type scale ratio 1.125 / 1.2 / 1.333, caps tracking 0-0.12 em, weight cap 400 / 500, row height 20-24, density gap 4 / 8 / 12.
Also: log the current screen's measured values next to each option (rows per 100 px, chars per line, element gap) so density is a number. Owner picks by eye, then the winner becomes tokens; one hairline and one tracking constant for the whole tool.
