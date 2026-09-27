import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../bp/common.dart' as tab;

/// Dock chrome for the hf client. `docking` owns only layout, split, resize and drop: its own tab strip is off,
/// because every seat draws the Browser's seat strip (`Leaf`) itself — see `DockWorkspace._seat`.
TabbedViewThemeData hfDockTabs() {
  final theme = TabbedViewThemeData.dark();
  theme.tabsArea.visible = false;
  theme.contentArea
    ..decoration = const BoxDecoration(color: tab.kGround)
    ..decorationNoTabsArea = const BoxDecoration(color: tab.kGround)
    ..padding = EdgeInsets.zero;
  return theme;
}

MultiSplitViewThemeData hfDockSplit() => MultiSplitViewThemeData(
  dividerThickness: 3,
  dividerPainter: DividerPainters.background(color: tab.kRule, highlightedColor: tab.kSel),
);
