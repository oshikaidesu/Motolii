# UI Debugging

Click a thing in the running Motolii window, see where its size comes from, change the value, see the result. Debug builds only: it
uses the Dart VM service that `flutter run` opens, so it is not in a release build and adds no code or dependency to the app.

## Start

```bash
scripts/motolii-ui.sh dev [document.rrd]     # the dev window; hot reload; writes its pid and VM service address under ~/.local/state/motolii-stage5
scripts/motolii-ui.sh inspect                # waits for clicks (watch); Ctrl-C to stop
```

`inspect pick` takes one click and returns; `inspect show` prints what is selected now.

## What you get

Flutter's own Inspector does the picking, the widget hierarchy, the source file and line, and the layout numbers (size,
constraints, padding, text style, colour). It is the service DevTools calls; `inspect` asks it the same questions. DevTools itself
works on the same session (open the link `flutter run` prints; Select Widget mode there selects in the same window, and the
Widget Tree shows the same source locations).

What `inspect` adds is the Motolii part: for the clicked widget and its nearest ancestors that set geometry, which value in the
source is a canonical token (`Surface.chromeRow = 24`, how many uses in how many files), a registered exception
(`Surface.px(8)`), a named non-number (a colour or a style), or a raw number.

```
▶ Container   lib/browser/parts.dart:174:41
      padding        EdgeInsets(8.0, 0.0, 8.0, 0.0)
      render.size    Size(78.5, 23.0)
      ◆ Surface.chromeRow = 24  (canonical token, 13 uses in 10 files, lib/theme/metrics.dart:111)
      ◇ Surface.px(8)  (registered exception: a SCALE-policy custom dimension)
```

A value Flutter computes (flex, intrinsic size, a parent's constraints) shows as the constraints and size it ended with, not as a
reason; follow the ancestors printed below it. Values with no source expression of their own say so.

## Change a value

```bash
scripts/motolii-ui.sh inspect tokens work                         # tokens, value at 100 %, usage
scripts/motolii-ui.sh inspect set Surface.chromeRow 19            # the token's declaration in lib/theme/metrics.dart, then hot reload
scripts/motolii-ui.sh inspect edit lib/browser/parts.dart:174 8=4 # a literal on one line, then hot reload
scripts/motolii-ui.sh inspect changed                             # this session's edits
scripts/motolii-ui.sh inspect reset                               # every edited file back as it was
scripts/motolii-ui.sh inspect scale 78                            # the UI Scale of the running window (not saved)
```

`set` and `edit` write the source, so the change is a normal diff to keep or discard; `reset` restores the files from before the
session's first edit. Values are the 100 % base; the UI Scale multiplies them as usual.

## Not covered

- Flutter's Property Editor edits constructor arguments from the IDE (VS Code, Android Studio); it is not used here. `edit` does
  the same for one number on a line.
- Hit bounds and visual bounds of a control are the Inspector's render size of the widget that takes the pointer (a
  `GestureDetector` / `Listener` in the list) against the widget that paints; Motolii's separate hit seats show as such.
- Computed layout is not re-derived (no layout engine here): what Flutter reports is shown, the rest is not.
