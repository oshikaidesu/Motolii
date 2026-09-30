import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../theme/surface.dart' show Surface;

/// Dock chrome for the hf client. `docking` owns only layout, split, resize and drop: its own tab strip is off,
/// because every seat draws the Browser's seat strip (`Leaf`) itself — see `DockWorkspace._seat`.
TabbedViewThemeData hfDockTabs() {
  final theme = TabbedViewThemeData.dark();
  theme.tabsArea.visible = false;
  theme.contentArea
    ..decoration = const BoxDecoration(color: Surface.base)
    ..decorationNoTabsArea = const BoxDecoration(color: Surface.base)
    ..padding = EdgeInsets.zero;
  return theme;
}

MultiSplitViewThemeData hfDockSplit() => MultiSplitViewThemeData(
  dividerThickness: 3,
  dividerPainter: DividerPainters.background(color: Surface.divider, highlightedColor: Surface.selected),
);
