import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../shell/place.dart';

/// Dock chrome for the hf client. The docking package owns only layout/dragging;
/// all visible tokens come from the hf reference grammar.
TabbedViewThemeData hfDockTabs() {
  final theme = TabbedViewThemeData.dark();
  theme.tabsArea
    ..color = H.window
    ..border = const Border(bottom: BorderSide(color: H.rule))
    ..initialGap = 0
    ..middleGap = 0
    ..gapBottomBorder = BorderSide.none;
  theme.tab
    ..padding = const EdgeInsets.symmetric(horizontal: 12, vertical: 7)
    ..textStyle = H.s(11, color: H.text3, w: FontWeight.w600)
    ..decoration = const BoxDecoration(color: H.window)
    ..selectedStatus = (TabStatusThemeData()
      ..decoration = const BoxDecoration(
        color: H.sel,
        border: Border(bottom: BorderSide(color: H.text, width: 2)),
      )
      ..fontColor = H.text);
  theme.contentArea
    ..decoration = const BoxDecoration(color: H.window)
    ..padding = EdgeInsets.zero;
  return theme;
}

MultiSplitViewThemeData hfDockSplit() => MultiSplitViewThemeData(
  dividerThickness: 3,
  dividerPainter: DividerPainters.background(
    color: H.rule,
    highlightedColor: H.selHi,
  ),
);
