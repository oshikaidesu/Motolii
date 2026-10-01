# UI Debugging

Click a thing in the running Motolii window, see where its size comes from, change the number, see the result, undo, keep or discard.
Debug builds only: it uses the Dart VM service that `flutter run` opens and Flutter's own widget Inspector; a release build has none of
it (`kDebugMode` in `lib/main.dart`).

## Design Mode (in the window)

```bash
scripts/motolii-ui.sh design [document.rrd]     # the dev window with Design Mode on; hot reload; Ctrl-C in this terminal ends it
```

Click what looks too big. Flutter's Select Widget overlay marks it, and a small HUD lists what decides its size, for the clicked widget and
the nearest ancestors that set geometry:

| row | meaning |
|---|---|
| `TOKEN 13×` | a canonical `Surface` / `Dn` token (13 uses): changing it changes every place that uses it |
| `EXCEPTION` | `Surface.px(7)`: a registered custom dimension, local to that line |
| `RAW` | a number written in the source |
| `DERIVED` | what Flutter computed (size, constraints, `flex 3 of 3 : 2` in a Row): not editable, change what causes it |
| `NAMED` / `CONST` | a colour or style; a `static const` (needs a hot restart, so read-only here) |

Every property is edited by its type, not only numbers: numbers (width, height, padding, margin, gap, radius, border width,
opacity, fontSize, lineHeight, letterSpacing, flex) by wheel / `↑` `↓` / typing; bools by toggle; enums (Alignment, TextAlign,
FontWeight, Axis, Main/CrossAxisAlignment, MainAxisSize, FlexFit, TextOverflow, TextBaseline, …) by `↑` `↓` or `Enter` for the
list; colours by the canonical token list or a hex literal; `EdgeInsets` and `BorderRadius` per side. A property the source does not
set yet is listed too; editing it inserts the named argument (or a `.copyWith(...)` on a style). The `TOKEN` / `HERE` chip says
whether an edit changes the shared declaration or only this call (the chip, or `T`); it never converts one into
the other silently. A local variable, parameter default or named constant is followed to its one declaration; when there is none
(a computed or passed-in value) the row is `READ-ONLY` and says why.

The active value is marked `▸`, an edited one `●`. The orange box in the window is the selected widget's bounds; for a `padding` row
the padding area is tinted orange and the content inside it outlined blue (a Container's padding is found a few render objects down). The
HUD list scrolls with the wheel: up to twelve ancestors that set geometry are listed, nearest first.

For a width, height or padding row the selected box carries a **grip** (an orange bar on the edge that value moves; a padding's grip sits
on the inner edge of the padding area). Pull it and the number follows in half units, written and hot-reloaded as it goes; the whole
pull is one undo.

| key | |
|---|---|
| click / `Tab`, `Shift+Tab` | choose the value (the HUD rows can also be dragged sideways) |
| mouse wheel, `↑` `↓` | ±1 (`Shift` 0.1, `Alt` 5) |
| `A` | BEFORE (the session's start) ↔ CURRENT |
| `Z` or `Cmd+Z`, `Shift+Z` | undo, redo |
| `R` | this value back as it was at the session's start |
| `Enter` | type a value / open the choices of an enum or colour (`Enter` applies, `Esc` cancels); the `TYPE` chip always types, a colour as hex |
| `Esc` | cancel what was just done to this value |
| `C` | the session's changes (`24 → 19`, per line) |
| `[` `]` | UI Scale −1 / +1 (`Shift` 10; not saved) |
| `F2` or `Cmd+Option+D` | Design Mode off (and on again); with changes it asks `KEEP` (Enter) or `DISCARD` (Esc, or the chips) first; `F2` again goes back to work |

The HUD's buttons (`A/B`, `UNDO`, `REDO`, `RESET`, `CHANGES`, `DONE`) do the same with the mouse. While Design Mode is on, a click
selects instead of acting and the keys above are the HUD's; off, Motolii is itself again.

A change is written into the source (a token's declaration in `lib/theme/metrics.dart`, or the number in the widget's call) and the
existing hot reload (`SIGUSR1` to the pid `dev` keeps in `~/.local/state/motolii-stage5/`) puts it on screen. KEEP leaves the diff in
the working tree for you to read and commit; DISCARD writes the session's start back. A `static const` colour or number is compiled into its users, so changing one needs a hot restart: the HUD writes the session
(history, BEFORE / CURRENT) to the state directory, sends `SIGUSR2`, and the restarted app brings Design Mode back with the session in it.
If a reload does not land (a compile error
somewhere), the last change is put back, the HUD says so and Design Mode goes on.

## Command line

The same ground without the window, for scripts and for a DevTools session (`scripts/motolii-ui.sh inspect`; `pick` waits for one
click, `watch` for each, `show` prints what is selected, `tokens [text]`, `set Surface.chromeRow 19`, `edit lib/x.dart:174 8=4`,
`changed`, `reset`, `scale 78`, `census [Widget.arg]`: how many visual/layout arguments in `lib/` are editable, read-only or unsupported). Flutter DevTools works on the same session (the link `flutter run` prints; Select Widget mode there
selects in the same window).

## Coverage

`inspect census` counts the visual / layout arguments written in `lib/` (behaviour, data and accessibility arguments are listed apart) and
says which Design Mode can edit, which it shows read-only with a reason, and which it does not know. At the last count: 85.1 % of all
arguments and 87.9 % of the visual / layout ones are editable (7.0 % read-only, 5.1 % unknown, mostly object-valued arguments such as
`Positioned.rect` or a gradient built by a function); `VISUAL/LAYOUT EDITABLE COVERAGE` is the second line it prints. What stays read-only is
computed or passed in (`SizedBox.height: size`, `Positioned.top: yOf(i)`): the HUD names the parameter or expression to change instead.

## Keys that automation cannot send

Esc (the exit question's DISCARD), Shift+Z, Enter and F2 are checked by `test/design_mode_keys_test.dart` through the real key path; the
manual check is: change a number, press F2, press Esc, and the source is as it was.

## Not covered

- Flutter's Property Editor edits constructor arguments from the IDE (VS Code, Android Studio); it is not used here: the HUD does the
  same for the numbers in Motolii's own widgets.
- Source rewrite is by expression span, never a regex over a file; anything it cannot place uniquely is read-only.
- The grip covers width, height and padding / margin sides. A gap between widgets, a `flex` and the choices are changed by wheel, keys and
  the lists; the hit bounds of a widget are not drawn.
- No production widget sets `textBaseline` yet; it is edited like any other choice (tested on source text, not seen in the window).
- Computed layout is not re-derived: what Flutter reports is shown, the rest is not.
