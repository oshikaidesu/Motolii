# Precedents for a dark, dense pro tool vs the Motolii Timeline draft

Read-only research, 2026-10-02. Every number below was seen in a fetched page or file; anything else is tagged `[inferred]`. "Draft" = `/Users/member_ottoto/rust_ae/design-sense-lab/DESIGN.md` + `book/lib/tokens.dart`.

Two cautions before reading the numbers:

1. **The VoltAgent DESIGN.md files describe marketing websites, not in-app chrome.** Heights like 36-44 px, 96 px section padding, hero type of 64-110 px are web-page values. PostHog's file lists "In-product app chrome specs" as an absence. Only 4 of the 11 are dark (Linear, Raycast, Framer, Warp); Figma, Cursor, Vercel, Notion, Miro, PostHog are light; Supabase is light with a `#1c1c1c` night canvas.
2. **The real in-product evidence is Adobe Spectrum token data (git) and Blender's default theme source (git).** These are the strongest sources here for a dense tool. Ableton and Figma UI3 gave no usable numbers (see section 1).

---

## 1. Sources actually read

Read in full (fetched, numbers extracted):

| Source | URL | What it is |
|---|---|---|
| VoltAgent Linear DESIGN.md | https://raw.githubusercontent.com/VoltAgent/awesome-design-md/main/design-md/linear.app/DESIGN.md | linear.app marketing site, dark only |
| VoltAgent Figma | .../design-md/figma/DESIGN.md | figma.com marketing, light |
| VoltAgent Raycast | .../design-md/raycast/DESIGN.md | raycast.com marketing, dark |
| VoltAgent Framer | .../design-md/framer/DESIGN.md | framer.com marketing, dark |
| VoltAgent Cursor | .../design-md/cursor/DESIGN.md | cursor.com marketing, light |
| VoltAgent Warp | .../design-md/warp/DESIGN.md | warp.dev marketing, warm dark |
| VoltAgent Vercel | .../design-md/vercel/DESIGN.md | vercel.com marketing, light |
| VoltAgent Notion | .../design-md/notion/DESIGN.md | notion.com marketing, light |
| VoltAgent Miro | .../design-md/miro/DESIGN.md | miro.com marketing, light |
| VoltAgent Supabase | .../design-md/supabase/DESIGN.md | supabase.com marketing, light + night canvas |
| VoltAgent PostHog | .../design-md/posthog/DESIGN.md | posthog.com marketing, light |
| Adobe Spectrum token data (real, per-theme) | https://raw.githubusercontent.com/adobe/spectrum-design-data/main/packages/tokens/src/{color-palette,color-aliases,layout,layout-component,typography}.json | The machine-readable source behind spectrum.adobe.com. I parsed the `dark` set. |
| Blender default theme | https://raw.githubusercontent.com/blender/blender/main/release/datafiles/userdef/userdef_default_theme.c | Shipped default dark theme (Action/dope sheet, NLA, regions, widgets) |
| Blender UI constants | https://raw.githubusercontent.com/blender/blender/main/source/blender/editors/include/UI_interface_c.hh and .../interface/interface_style.cc and .../windowmanager/intern/wm_window.cc | font points; `widget_unit = round(18*scale) + 2*pixelsize` |
| Radix Colors dark scale | https://raw.githubusercontent.com/radix-ui/colors/main/src/dark.ts and https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale | 12-step grey scale with per-step roles |
| Linear blog | https://linear.app/now/how-we-redesigned-the-linear-ui | qualitative; few numbers |
| Figma blogs | https://www.figma.com/blog/our-approach-to-designing-ui3/ , https://www.figma.com/blog/illuminating-dark-mode/ | qualitative; token-schema counts |
| Adobe Design story | https://adobe.design/stories/design-for-scale/reinventing-adobe-spectrum-s-colors | contrast ratios of 3 text steps; "14 tints per colour" |
| Dark-mode articles | https://uxmagic.ai/blog/dark-mode-ui-design-guide , https://colorarchive.org/guides/dark-mode-palette-guide/ | numbers cited by the articles (secondary, not measured from products) |

Could not open / no usable numbers:

- `spectrum.adobe.com/page/{color-system,corner-radius,spacing,typography,density,motion}/` all returned 404 via fetch (SPA). Used the token JSON instead. `spectrum.adobe.com/page/design-tokens` opened but has only methodology. **Spectrum motion durations: not found** (no duration/ease key exists in the token files I parsed).
- **Ableton Live**: no public design-system numbers. Searches only found "theme files are `.ask` XML inside the app bundle" and that Live 12 reworked radius/padding/spacing (marketing/CDM prose). I did not open an `.ask` file. **No Ableton numbers in this document.**
- **Figma UI3**: blogs give structure (docked resizable panels, ~350 semantic tokens, menus/toolbars stay dark in both themes) but **no hex, height or radius values**.
- **Blender Human Interface Guidelines** (developer.blender.org): "no specific numeric values" (page says it is unfinished).
- Material 3 dark-theme page returned an empty shell. Numbers like "#121212" appear only through the articles above.
- The DESIGN.md files document **no hover/motion** for any of the 11 (most say so explicitly). The draft's "120 ms ease-out" cannot be compared to any DESIGN.md.

---

## 2. Extracted numbers per source

### 2.1 Adobe Spectrum (token data, `dark` set) - the closest pro-tool evidence

Grey scale, 13 steps (`gray-25 ... gray-1000`), dark set:

| step | dark | step | dark |
|---|---|---|---|
| 25 | `#111111` | 500 | `#6D6D6D` |
| 50 | `#1B1B1B` | 600 | `#8A8A8A` |
| 75 | `#222222` | 700 | `#AFAFAF` |
| 100 | `#2C2C2C` | 800 | `#DBDBDB` |
| 200 | `#323232` | 900 | `#F2F2F2` |
| 300 | `#393939` | 1000 | `#FFFFFF` |
| 400 | `#444444` | | |

- Roles (color-aliases): `background-base` = gray-25; `background-layer-1` = gray-50; `background-layer-2` and `background-elevated` (dark) = gray-75; `background-pasteboard` (dark) = gray-25 (the canvas behind artboards is the darkest).
- Adjacent step sizes in 0-255 levels: 10, 7, 10, 6, 7, 11, 41 (to 500).
- Contrast (Adobe Design story): gray-700 8.77:1, gray-800 14.96:1, gray-900 19.78:1 (these are quoted for the system; I did not recompute per theme). [inferred] on the dark gray-100 `#2C2C2C`, gray-600 `#8A8A8A` computes to 4.05:1, gray-700 6.37:1.
- Hover/pressed/key-focus = content colour at **0.1 opacity** (`background-opacity-hover/down/key-focus = 0.1`); `opacity-disabled = 0.3`; opacity scale 0.02, 0.03, 0.04, 0.06, 0.08, 0.12, 0.16 ...
- Focus indicator: **2 px thick, 2 px gap**, colour blue-800. Border widths 1 / 2 / 4 px.
- Corner radius scale: 0, 3, 4, 5, 6, 7, 8, 9, 10, 16, full. "small" family default 4; small sizes S/M/L/XL = 3/4/5/6; "medium" family default 8, sizes XS..XL = 6/7/8/9/10; large default 10; extra-large 16.
- Component heights (desktop / mobile): 20/26, 24/30, **32/40**, 40/50, 48/60, 56/70, 64/80 (tokens `component-height-50 ... 500`).
- Spacing: 1, 2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96.
- Type: font "Adobe Clean Spectrum VF"; UI sizes desktop **10, 11, 12, 14, 16, 18, 20** (mobile 12, 13, 15, 17...). Line-heights 12/14/16/18/20 px for 10/11/12/14/16 (ratio 1.2-1.3). Letter-spacing `0em`. Weights regular / medium / bold. Line-height ratios 1.3 and 1.5.
- Shadows (dark values are ~3x the light alpha): emphasized = 3 layers, y 2/1/0, blur 8/4/1; elevated = y 4/2/0, blur 12/6/2; dragged = y 12/6/0, blur 16/8/6. Dark alphas ambient 0.24, transition 0.12, key 0.24-0.48; light alphas 0.08/0.04/0.08-0.16. Overlay black at 0.6 (dark) vs 0.4 (light).
- Padding examples (desktop): text-to-edge 8 / 9 / 12 / 15 / 18 px for sizes 50/75/100/200/300; top-to-text 3/4/6/9/12.
- Motion: **not in the tokens**. Density names "compact / regular / spacious" exist as a component prop (search result, not seen in the token files).

### 2.2 Blender default theme (shipped source)

- Widgets (`wcol_regular`, `wcol_tool`): outline `#3D3D3D`, inner `#545454`, selected inner `#4772B3` (blue), text `#E6E6E6`, selected text `#FFFFFF`; text-field inner `#1D1D1D`; toolbar item inner `#282828`. `roundness = 0.2` (fraction of widget height; [inferred] about 4 px at the 20 px unit).
- Panels: `panel_back` `#3D3D3D`, `panel_sub_back` `#0000001F`, `panel_outline` `#FFFFFF11` (about 7% white), `panel_roundness = 0.4`, `panel_active` `#4772B3`.
- Regions: channel list back `#1D1D1D`, text `#B8B8B8`, selected text `#FFAF23` (orange); scrubbing strip back `#1D1D1D`; headers `#303030B3` (70% alpha); editor `back` `#303030` (Action, Graph, NLA) with alpha 00 (region draws its own).
- Dope sheet (`space_action`): **grid `#161616`**, text `#A6A6A6`, `keyborder = #000000` (black edge on keys), key colour `#BFBFBF`, selected key `#FFBE33` (orange); typed keys: extreme `#E8B3CC` (pink), breakdown `#B3DBE8` (cyan), jitter `#94E575` (green), moving hold `#808080`, generated `#585858`; interpolation lines linear `#94E575CC` / constant `#E59C7BCC` / other `#5DBABEB3`; `outline_width = 1`.
- `row_alternate` (zebra) = `#FFFFFF04` and `#FFFFFF05` (about 1.6-2% white) in File browser and Sequencer.
- NLA: strip `#0D0D0D80`, grid `#2A2A2A`, selected strip `#FF8C00`. Time marker `#FFFFFF80`, selected `#FFFFFF`.
- UI unit: `widget_unit = round(18 * scale) + 2 * pixelsize` = **20 px at scale 1** [inferred from formula]; widget / title / tooltip font all **11 pt**.
- Hue use: selection/active = one blue (`#4772B3`); keyframe selection + selected channel text = orange; categorical colours only for keyframe types and interpolation.

### 2.3 Linear (DESIGN.md = marketing site; blog = app)

- Greys: canvas `#010102`; surface-1 `#0F1011`, -2 `#141516`, -3 `#18191A`, -4 `#191A1B` (4 surface steps); hairline `#23252A`, strong `#34343A`, tertiary `#3E3E44` (3 borders); text `#F7F8F8`, `#D0D6E0`, `#8A8F98`, `#62666D` (4 text steps). One accent `#5E6AD2` (hover `#828FFF`); one semantic green `#27A644`.
- Borders: 1 px solid hex, never alpha. Input focus = 2 px `#5E69D1` at 50% opacity.
- Radii: 4 (chips/badges), 6, 8 (buttons, inputs), 12 (cards), 16, 24, pill. Spacing base 4: 4/8/12/16/24/32/48/96.
- Type: Linear Display/Text/Mono; body 14 / caption 12 / button 14 weight 500; eyebrow 13/500/+0.4 px; display tracking -3.0 px at 80 px scaling to 0 at 14-16 px. Mono 13.
- Depth: levels 0-3 are surface+hairline steps, level 4 is a focus outline. "No ... drop shadows".
- Accent rule: lavender only on brand mark, primary CTA, focus rings, link emphasis. "No second chromatic colour."
- Blog: theme generated in **LCH from 3 variables (base, accent, contrast)**, down from 98; Inter Display for headings, Inter for the rest; text and neutral icons made darker in light / lighter in dark. No px values.

### 2.4 Raycast (marketing, dark)

- Canvas `#07080A`; surface `#0D0D0D`, elevated `#101111`, card `#121212`. Border hairline `#242728` (solid), and `rgba(255,255,255,0.08)` soft / `0.16` strong (both forms in one system). Text `#F4F4F6`, `#CDCDCD`, `#9C9C9D`, `#6A6B6C`, `#434345`.
- Accents: blue `#57C1FF`, red `#FF6161`, green `#59D499`, yellow `#FFC533`, each with a 15% soft tint; "saturated accents reserved for extension illustrations, never on chrome."
- Inter with `ss03`; body 14, caption 12/+0.4 px, button 14/500/+0.2 px. Radii 4/6/8/10/16. Heights: button 36, input 36, **keycap 20 px** (padding 1 px 6 px, radius 4).
- Depth: "No drop shadows"; ladder is surface colour + 1 px borders. Focus = stronger hairline, no coloured ring.

### 2.5 Framer (marketing, dark)

- Canvas `#090909`, surface `#141414`, `#1C1C1C`; hairline `#262626`, soft `#1A1A1A`; text `#FFFFFF` / `#999999`. One UI accent `#0099FF` (links, focus, selection). Gradients are decorative spotlight cards.
- Radii 4/6/10/15/20/30, pill 100. Spacing 1/4/8/12/15/20/30/40 (non-4 grid). Body 15/14/13 with negative tracking (-0.15 / -0.14 / -0.13 px); button 14/500.
- Depth: floating card = 0.5 px white 10% top edge + `0 10px 30px rgba(0,0,0,0.25)`; focus = `0 0 0 1px rgba(0,153,255,0.15)`; pressed = scale, not fill change.

### 2.6 Warp (marketing, warm dark)

- Canvas `#2B2622` (warm), soft `#383330`, hairline `#3F3A36`; text `#F7F5F0`, `#DAD2C1`, `#C9C0AD`, `#AEA69C`. **No accent colour** ("the off-white IS the brand").
- Radii 1/2/**3 (button)**/4/6. Inter 14/12 body, DM Mono 13. Tracking 0 below 24 px. No shadows, no gradients; depth = soft fill + 1 px hairline.

### 2.7 Light-theme DESIGN.md files (for radius/spacing/type/accent only)

| System | Radii | Spacing base | Control heights | Accent rule | Shadow |
|---|---|---|---|---|---|
| Figma | 2/6/8/24/32/pill | 8 (4..96) | icon button 40, nav 56 | pastel blocks + magenta promo; no hover/focus/motion given | L2 `0 4px 16px rgba(0,0,0,.06)` |
| Cursor | 4/6/8/12/16/pill | 4 | button 40, 44 | one orange `#F54E00`; 5 pastel "timeline" colours only inside product mockups (`#DFA88F #9FC9A2 #9FBBE0 #C0A8DD #C08532`), caption caps 11/600/+0.88 px | none |
| Vercel | 4/6/8/12/16 | 4 | input 32/40/48 | 3 gradients, link blue | stacked 2-3 layers, alpha 0.03-0.15 |
| Notion | 4/6/8/12/16 | 4 | input 44 | purple CTA + link blue + 5 brand hues | alpha 0.08-0.20 |
| Miro | 4/8/12/16/28/32/pill | 4 | input 44, search 40 | blue/yellow/coral/teal; micro uppercase 11/600/+0.5 px | alpha 0.06-0.12 |
| Supabase | 4/6/8/12/16 | 8 | button 8+8 padding | green `#3ECF8E` + 8 further hues | 0.06-0.12 |
| PostHog | 2/4/6/8 | 4 | button 40, input 36 | amber CTA + 4 semantic hues | none, "1 px solid throughout" |

### 2.8 Radix dark grey scale (UI step roles)

`#111111 #191919 #222222 #2A2A2A #313131 #3A3A3A #484848 #606060 #6E6E6E #7B7B7B #B4B4B4 #EEEEEE` (12 steps). Roles: 1-2 app/subtle background, 3 component bg, 4 hovered component, 5 pressed/selected, 6 subtle borders, 7 borders + focus rings, 8 stronger borders, 9-10 solid, 11 low-contrast text, 12 high-contrast text. (Radix source: https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale)

### 2.9 Dark-mode articles (secondary; their claims, not measurements)

- uxmagic: base `#121212`, card `#1E1E1E`, modal `#2A2A2A`; each tier 5-8% brighter; border `rgba(255,255,255,0.08)`; text emphasis 87% / 60% / 38%; reduce accent saturation 20-30% and raise luminance; "shadow can't render darker than an already near-black surface." https://uxmagic.ai/blog/dark-mode-ui-design-guide
- colorarchive: each layer +4-8% lightness; scale 8% base / 12% cards / 16% hover / 20% active; base 8-14% lightness, avoid `#000`; reduce saturation 15-25%; body text 92-95% lightness not pure white. https://colorarchive.org/guides/dark-mode-palette-guide/
- Figma dark-mode post: ~350 semantic tokens, 5 dimensions (type / element / role / prominence / interaction), CI linter forbids non-semantic colours. https://www.figma.com/blog/illuminating-dark-mode/

---

## 3. Draft vs precedents

Computed on the draft (WCAG contrast, 0-255 levels): bandA `#1E1E1E` vs ground `#191919` = +5 levels, 1.05:1; bandB `#2A2A2A` vs bandA = **+12 levels, 1.16:1**; `N.g20` border on ground 1.41:1; rowLine `#161616` on bandA 1.09:1, on bandB 1.26:1; g56 text `#8E8E8E` on bandA 5.09:1, on bandB 4.38:1; g95 on bandB 12.82:1. Draft key/bar sizes: bar 0.87*23 = 20.01 px, radius 2.07 px, key 7.59 px.

| # | Draft rule | Verdict | What the precedents do |
|---|---|---|---|
| 1 | Ground `#191919`, well `#131313` | AGREES with 5 (Spectrum `#111`/`#1B1B1B`, Radix `#111`/`#191919`, Supabase night `#1C1C1C`, both articles `#121212`, Framer `#090909` close) / DIFFERS from 3 (Linear `#010102`, Raycast `#07080A` far darker; Warp `#2B2622` warm and lighter) | Spectrum puts the **darkest** step on the *pasteboard* behind the work, and layers lighter on top. Draft's "well" (darker than ground) matches that idea. |
| 2 | 15 greys (g00..g100) + 3 timeline-only + 2 glazes = 20 neutral values | DIFFERS from 4 of 4 with a counted scale | Spectrum 13, Radix 12, Linear 4 surfaces + 3 borders + 4 text = 11, Raycast 4 surfaces + 3 borders + 5 text = 12, Framer 2 surfaces + 2 hairlines + 2 text. Draft has more grey values than any precedent and uses two (`bandA/bandB`) that are not on its own ladder. |
| 3 | Row zebra `#1E1E1E`/`#2A2A2A` (+12 levels) | DIFFERS from 1 of 1 that has a zebra; no other precedent documents one | Blender zebra is `#FFFFFF04..05` (~1.6-2% white, about +4 levels). Largest *adjacent grey step* in any ladder read: Spectrum 11 (gray-300 to gray-400 is 11; most are 6-10), Radix 8, Linear 5, Raycast 3. Draft's band step (12) exceeds all of them. |
| 4 | Row separator `#161616` (darker than both bands) | AGREES with 1 (Blender dope-sheet grid is `#161616`, identical value) / DIFFERS from the light-line camp | Blender draws dark grid in the dope sheet; draft's own *time grid* uses light lines (white 16%/9%) and says "no dark lines". Inside one draft, row line is dark and column grid is light; Blender uses dark for both `[inferred from grid key only]`. |
| 5 | 1 px border `N.g20` `#343434` (solid) | AGREES with 6 on "1 px solid" (Linear, Raycast, Framer, Warp, PostHog, Blender widget outline) / split on solid vs alpha (see section 4) | Spectrum border width 1/2/4 px. Linear border `#23252A` / `#34343A` / `#3E3E44`; draft's `#343434` on `#191919` is 1.41:1, Linear's strong `#34343A` on `#0F1011` is similar `[inferred, not computed]`. |
| 6 | Depth by grey level, shadow only on floating | AGREES with 6 (Linear, Raycast, Warp, Cursor, PostHog: no shadows; Framer & Spectrum: shadow only for floating/raised) | Spectrum *does* use both: grey layers (111 > 1B > 22) **and** 3-layer shadows whose alpha is ~3x higher in dark (0.36/0.48/0.6 vs 0.12/0.16/0.2). Draft elevation steps +7/+6/+14 levels: Spectrum 10/7/10, Radix 8/8, Linear 5/4/1, Raycast 3/2. |
| 7 | Control radius 4-8, bar radius ~2 px | AGREES with 9 on 4-8 (Spectrum 4 default small / 8 medium, Linear 8, Raycast 8, Cursor 8, Notion 8, Supabase 6, PostHog 6, Blender 0.2 of height, Warp 3) / bar 2 px: AGREES with Figma xs 2, Warp xs 2, PostHog 2, Spectrum 3 | Cursor's in-product timeline pills are **full pills** (marketing mockup). Draft has no pill and no radius scale > 8. |
| 8 | Row height 23 | AGREES with the pro tools (Blender unit 20, Spectrum 20/24 desktop sizes, Raycast keycap 20) / DIFFERS from marketing controls 32-44 | Spectrum heights are **integer steps** 20/24/32/40; Blender rounds with `roundf`. Draft derives 20.01 / 2.07 / 7.59 px from fractions of 23 (and says hairlines snap), so most sizes are fractional before snapping. |
| 9 | Spacing: no scale; 3 px between rows, `Surface.px(n)` escape | DIFFERS from 8 of 8 that list a scale | Base 4: Linear, Cursor, Vercel, Notion, Miro, PostHog, Warp; base 8: Figma, Supabase, Raycast; Spectrum 1/2/4/6/8/12/16... (has 2, 4, 6 but 3 appears only inside `component-top-to-text-50`). Draft's own 3 px is on no scale. |
| 10 | Inter 11 / 10 / 9.5 (+ Menlo 11) | 11 AGREES with Spectrum size-50 and Blender (11 pt for widget/title/tooltip) / 10 AGREES with Spectrum size-25 / 9.5 is below every size read (smallest: Spectrum 10; marketing minimum 11-12) | Spectrum desktop UI scale 10/11/12/14/16; Linear/Raycast/Warp captions 12. |
| 11 | Letter-spacing +0.05 (names), +0.1 (labels), +0.15 (micro) on mixed-case | DIFFERS from Spectrum (`0em`), Linear body (0), Warp (0 below 24 px); AGREES in direction with Raycast (+0.2 on 14 px buttons, +0.4 on 12 px caption), Linear eyebrow (+0.4 at 13 px) | Positive tracking in precedents is mostly on **small uppercase/eyebrow** text and is bigger: Cursor +0.88 px at 11 px (8%), Miro +0.5 px at 11 px (4.5%), Raycast +0.4 at 12 (3.3%). Draft's is 0.5-1.6% on mixed-case. |
| 12 | Line box 1.0 x size | DIFFERS from Spectrum (12/14/16/18 px for 10/11/12/14 = 1.2-1.3) and all marketing captions (1.33-1.5); AGREES only with buttons in Cursor, Supabase, Framer (line-height 1.0) | Fixed-height single-line row makes 1.0 workable `[inferred]`; no precedent uses 1.0 for body/labels. |
| 13 | Weights 400 / 500 (name) / 600 (title) | AGREES with 8 (400 body, 500 button/heading, 600 for display or small caps: Linear, Cursor, Miro; Miro "no 700") | Spectrum: regular / medium / bold. |
| 14 | Mono (Menlo 11) for values and time | UNCLEAR: 0 precedents read use mono for numeric *values*; mono is for code (Linear Mono 13, DM Mono 13, Geist Mono, JetBrains Mono 13) | Raycast keycap and Figma eyebrow use mono only for labels/captions. I did not inspect Figma/Linear apps themselves, so tabular-sans vs mono for values is untested here. |
| 15 | Hues: 6 family hues for bars (relations only) | AGREES with 2 (Cursor: 5 pastel timeline stages; Blender: 4 key-type tints + 3 interpolation tints) / DIFFERS from 3 (Linear, Framer, Warp: one accent or none) | Raycast: 4 accents, kept off chrome. Spectrum: many hues x 14 tints, for data/status. Draft `tokens.dart` also defines **4 more colours not mentioned in DESIGN.md** (`C.playhead #6EA6DB`, `C.play #7BCC9E`, `C.record #F03C8A`, `C.mode #7A87E3`), so the real count is 10 hues. |
| 16 | Bar colour: sat x0.85, light x0.93 at rest | AGREES with 3 (uxmagic -20-30% sat; colorarchive -15-25%; Blender key tints are pastels `#E8B3CC #B3DBE8 #94E575`) | Draft's x0.85 = -15% saturation, at the mild end of the articles' range. |
| 17 | Selection / highlight colour | DIFFERS from Blender (selected key is **orange** `#FFBE33`, widgets blue `#4772B3`: two selection hues); Spectrum/Linear/Framer use one blue/lavender for focus/selection | Draft: "panel's accent" and playhead blue `#6EA6DB` (near the Stagger family blue). |
| 18 | Key: near-white diamond with 0.75 px white-60% edge | DIFFERS from Blender (key `#BFBFBF` with **black** `keyborder #000000`; selected `#FFBE33`) | Draft deliberately forbids dark edges. No other precedent documents a key marker. |
| 19 | Hover = g15, selected = g20 (grey), focus ring not specified | DIFFERS: precedents define focus explicitly | Spectrum focus 2 px thick + 2 px gap; Linear 2 px at 50% alpha; Framer 1 px 15%; PostHog 2 px; Raycast = stronger hairline (no ring). Hover as **overlay at fixed opacity**: Spectrum 0.1, Radix step 4 (`#2A2A2A`). Draft DESIGN.md/tokens.dart contain no focus rule (grep of the two files). |
| 20 | Motion 120 ms ease-out (per brief; not found in DESIGN.md or tokens.dart) | NO EVIDENCE either way | No DESIGN.md documents motion; Spectrum tokens contain none. Notion/Miro files carry a "recommend 150-200 ms ease" note that is the extractor's suggestion, not a measured value (treat as unreliable). |
| 21 | Desktop only, UI scale 50-200% | AGREES with Spectrum (desktop and mobile scales both exist; mobile ~1.25x: 14 -> 17 px, 32 -> 40 px) and Blender (`widget_unit` scales with `scale_factor`) | |
| 22 | Text greys g95 `#F2F2F2` primary, g56 `#8E8E8E` muted | AGREES with Spectrum dark gray-900 `#F2F2F2` (exact) and gray-600 `#8A8A8A` (near) | Draft's muted text on bandB is 4.38:1, just under 4.5; Spectrum's equivalent on its `#2C2C2C` is 4.05:1. Raycast mute 7.1:1 on its surface, Linear subtle 5.9:1 on surface-1 (both computed by me). |

---

## 4. Where precedents split

**A. Border: solid hex vs white alpha**
- Solid hex: Linear (`#23252A`, `#34343A`, `#3E3E44`), Framer (`#262626`), Warp (`#3F3A36`), PostHog, Blender widget outline `#3D3D3D`.
- Alpha: uxmagic `rgba(255,255,255,0.08)`, Blender `panel_outline #FFFFFF11`, Raycast soft 0.08 / strong 0.16 (Raycast ships *both*), Spectrum content-colour overlays at 0.1.
- Draft: solid hex (`N.g20`) for borders, alpha ("glaze9/15", white 16%/9%) for the time grid and bar edges, i.e. already in both camps.

**B. How many grey steps / surface ladder fineness**
- Fine, few levels: Linear surfaces +5/+4/+1 over a near-black canvas; Raycast +3/+2.
- Coarse, many levels: Spectrum (13 greys, 6-11 apart), Radix (12, 8 apart).
- Draft sits with the coarse camp for its ladder (7/6/14) but adds a separate zebra pair (12 apart).

**C. Depth: grey only vs grey + shadow**
- Grey/hairline only: Linear, Raycast, Warp, Cursor, PostHog.
- Grey + shadow for floating: Framer (0.25 alpha, 30 px blur), Spectrum (three-layer, dark alpha 0.12-0.6), Figma, Vercel, Notion (light-theme shadows are 0.06-0.2).
- Draft is in the second camp but only for menus/tooltips.

**D. Accent count**
- One accent: Linear (+1 semantic green), Framer, Cursor (+ 5 product-only tints), Warp (0).
- A few semantic hues kept off chrome: Raycast (4), PostHog (4), Notion.
- Many hues with tints: Spectrum (14 tints each), Supabase (8), Blender (blue + orange + type tints).
- Draft: 6 family + 4 utility hues; "bars carry colour, the rest is grey" matches Cursor's "pastels only in product UI" and Raycast's "never on chrome".

**E. Positive letter-spacing on small text**
- None (0): Spectrum, Linear body/caption, Warp, Notion, Supabase.
- Small positive on caps/captions: Linear eyebrow +0.4, Raycast +0.2/+0.4, Cursor +0.88, Miro +0.5, Figma mono +0.54.
- Negative on body: Framer (-0.13..-0.15), Vercel body-sm (-0.28).
- Draft: +0.05..+0.15 on mixed-case.

**F. Control height**
- Marketing 36-48 px (Raycast 36, PostHog 36/40, Linear/Notion/Miro 40-44).
- Pro tools 20-24 px (Blender 20, Spectrum 20/24 compact sizes, Raycast keycap 20).
- Draft 23 is in the pro camp.

**G. Selection colour**
- One hue for focus and selection: Spectrum (blue), Linear (lavender), Framer (blue).
- Different hue for selected *data* items: Blender (blue for widgets, orange for keys/channels/strips).
- Draft: accent for selected key + playhead blue.

**H. Zebra stripes**
- Only Blender documents one, at ~2% white. Spectrum and every DESIGN.md I read give none (no row/stripe token seen). Evidence is thin, so "precedents split" does not apply; there is one example.

---

## 5. The 10 differences most worth the owner's attention

1. **Zebra step is large.** Draft `#1E1E1E`/`#2A2A2A` = +12 levels (1.16:1). Blender's shipped zebra is `#FFFFFF04..05` (~+4 levels); the biggest adjacent step in Spectrum is 11, Radix 8, Linear 5, Raycast 3. Also bandB `#2A2A2A` equals Radix's "hovered component" step, so hover and bandB are the same grey. Evidence: Blender theme file lines `.row_alternate`; Radix dark.ts.
2. **More greys than any scale (20 values vs 11-13)**, two of which (bandA/bandB) are off-ladder. Spectrum 13: https://github.com/adobe/spectrum-design-data/blob/main/packages/tokens/src/color-palette.json ; Radix 12.
3. **Row-separator is dark while the column grid is light.** Blender dope sheet uses dark grid `#161616` (same value as draft's rowLine), draft's time grid is white 16%/9% and states "no dark lines". Two line colours in one panel; the draft rule "a line style is one style" is violated by its own choice `[inferred]`.
4. **Key edge: draft forbids dark edges; Blender's dope-sheet keys have a black border** (`keyborder #000000`) and the only other timeline precedent (Cursor) is a light-theme mockup. No precedent supports a white-edge diamond with a 0.75 px stroke. Evidence: userdef_default_theme.c `.space_action`.
5. **Tracking: draft uses +0.05..0.15 on mixed-case 9.5-11 px; Spectrum is `0em`, Linear/Warp body is 0.** Positive tracking in precedents is 3-8% and only on small caps/captions (Cursor +0.88 px @ 11, Miro +0.5 @ 11, Raycast +0.4 @ 12). Evidence: Spectrum typography.json, Cursor/Miro/Raycast DESIGN.md.
6. **9.5 px text is below every size read** (Spectrum smallest 10 px / 12 px line box; marketing minimum 11-12). Line-height 1.0 versus Spectrum 1.2-1.3. Evidence: Spectrum `font-size-25 = 10px`, `line-height-font-size-25 = 12px`.
7. **No focus indicator and no pressed state in the draft**, while Spectrum (2 px + 2 px gap), Linear (2 px @ 50%), Framer (1 px @ 15%), PostHog (2 px) and Raycast (stronger hairline) all specify one. Hover as a fixed 0.1 overlay (Spectrum) vs draft's absolute greys g15/g20.
8. **Sizes are fractions of the row (20.01, 2.07, 7.59 px)** while Spectrum and Blender keep integer sizes (Blender rounds; Spectrum heights 20/24/32/40). Draft states hairlines snap to device pixels but not the size tokens. Evidence: Spectrum layout.json heights; Blender `wm_window.cc` `roundf`.
9. **Spacing has no scale** (3 px between rows, `Surface.px(n)` escape) while 8 of 8 precedents with a scale use 4 or 8 base; the 3 px is on none of them (Spectrum has 2/4/6).
10. **Hue count in code (10) exceeds the DESIGN.md (6).** `C.play`, `C.record`, `C.mode`, `C.playhead` are extra. Precedents with one accent (Linear, Framer, Warp) and Raycast ("never on chrome") would call the extras chrome colour. Related: Blender uses a second selection hue (orange) for keys while the draft uses the playhead blue near the Stagger family blue (`#6EA6DB` vs `#4781E5`).

Not on the list because there is no evidence from precedents either way: motion duration (120 ms, no precedent measured), mono for values, zebra on property rows (only one example, Blender).

---

Key sources (links used above): https://github.com/adobe/spectrum-design-data/tree/main/packages/tokens/src , https://github.com/blender/blender/blob/main/release/datafiles/userdef/userdef_default_theme.c , https://github.com/VoltAgent/awesome-design-md , https://github.com/radix-ui/colors/blob/main/src/dark.ts , https://linear.app/now/how-we-redesigned-the-linear-ui , https://www.figma.com/blog/illuminating-dark-mode/ , https://adobe.design/stories/design-for-scale/reinventing-adobe-spectrum-s-colors .
