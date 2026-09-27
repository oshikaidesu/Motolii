import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../bp/common.dart' as tab;

/// Dock chrome for the hf client. The `docking` package owns only layout/dragging; every visible token is the
/// Browser family's own tab grammar (`hf/bp/common.dart`'s `Leaf`/`_Tab`), promoted here so every dock seat —
/// Browser, Stage, Inspector, Timeline, the desks — reads as one family. Nothing here invents a new tab design;
/// see `DockWorkspace.item()`/`_relabel` for the fold ("identity travels, fold don't miniaturize") this theme's
/// sizes and colours are tuned to match.
TabbedViewThemeData hfDockTabs() {
  final theme = TabbedViewThemeData.dark();
  theme.tabsArea
    ..color = tab.kGround
    ..border = const Border(bottom: BorderSide(color: tab.kRule))
    ..initialGap = 0
    ..middleGap = 0
    ..gapBottomBorder = BorderSide.none;
  theme.tab
    ..padding = const EdgeInsets.symmetric(horizontal: 10, vertical: 7)
    ..textStyle = tab.sans(12, c: tab.kMuted, w: FontWeight.w500)
    ..decoration = const BoxDecoration(color: tab.kGround, border: Border(right: BorderSide(color: tab.kRule2)))
    ..selectedStatus = (TabStatusThemeData()
      ..decoration = const BoxDecoration(color: tab.kSel, border: Border(right: BorderSide(color: tab.kRule2)))
      ..fontColor = const Color(0xFFF2F2F2));
  theme.contentArea
    ..decoration = const BoxDecoration(color: tab.kGround)
    ..padding = EdgeInsets.zero;
  // Narrower than the package default (200) so a tab's own menu (its "…" kebab, or the
  // hidden-tabs list when a strip is too narrow to show every tab) never fills a whole
  // panel: the panel keeps a visible, clickable strip of its glass backdrop to cancel
  // the menu without picking an item. Our narrowest panels are ~180px wide.
  theme.menu
    ..maxWidth = 140
    ..color = tab.kGround
    ..border = Border.all(color: tab.kRule)
    ..textStyle = tab.sans(11, c: const Color(0xFFDADBDC))
    ..hoverColor = tab.kSel;
  return theme;
}

MultiSplitViewThemeData hfDockSplit() => MultiSplitViewThemeData(
  dividerThickness: 3,
  dividerPainter: DividerPainters.background(
    color: tab.kRule,
    highlightedColor: tab.kSel,
  ),
);
