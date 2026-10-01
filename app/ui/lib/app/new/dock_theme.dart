import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../../foundation/shell_tokens.dart';

/// The New shell's chrome over the dock's tab strips: flat, mono, one rule.
TabbedViewThemeData shellTabs() {
  final theme = TabbedViewThemeData.dark();
  theme.tabsArea
    ..color = ShellTokens.ground
    ..border = const Border(bottom: BorderSide(color: ShellTokens.rule))
    ..initialGap = 0
    ..middleGap = 0
    ..gapBottomBorder = BorderSide.none;
  theme.tab
    ..padding = const EdgeInsets.symmetric(horizontal: 12, vertical: 6)
    ..textStyle = ShellTokens.kickerStyle(ShellTokens.inkMuted)
    ..decoration = const BoxDecoration(color: ShellTokens.ground)
    ..selectedStatus = (TabStatusThemeData()
      ..decoration = const BoxDecoration(
        color: ShellTokens.surface,
        border: Border(bottom: BorderSide(color: ShellTokens.ink, width: ShellTokens.activeRule)),
      )
      ..fontColor = ShellTokens.ink);
  theme.contentArea
    ..decoration = const BoxDecoration(color: ShellTokens.surface)
    ..padding = EdgeInsets.zero;
  return theme;
}

MultiSplitViewThemeData shellSplit() => MultiSplitViewThemeData(
  dividerThickness: ShellTokens.ruleWidth * 3,
  dividerPainter: DividerPainters.background(
    color: ShellTokens.rule,
    highlightedColor: ShellTokens.ruleStrong,
  ),
);
