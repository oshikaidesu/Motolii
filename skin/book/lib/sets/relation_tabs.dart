// The Relations tab, made many ways, to pick one: each way a pane beside the inspector on the real window, joined to a
// fresh copy of the try document (a cube, eight photos round it, three words, no relation yet), all telling the
// document through moves.dart (who, when, from where to where, how). Widgetbook: Relation › Relations tabs.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../motolii.dart';
import '../workspace/dock_glyphs.dart';
import '../workspace/panes.dart';
import '../workspace/workspace.dart';
import 'relation_tabs/tab_curve.dart';
import 'relation_tabs/tab_draw.dart';
import 'relation_tabs/tab_hands.dart';
import 'relation_tabs/tab_poses.dart';
import 'relation_tabs/tab_samples.dart';

/// Every way of the tab, by name (one file each in relation_tabs/).
final relationTabs = <(String, Widget Function())>[
  ('F one tap', () => const TabSamples()),
  ('A by hand', () => const TabHands()),
  ('B stagger curve', () => const TabCurve()),
  ('C draw it', () => const TabDraw()),
  ('E two poses', () => const TabPoses()),
];

final _copies = <String, String>{};

/// A copy of the try document for one way, made once a run (kept across a hot reload, so what was done stays).
String _fresh(String way) => _copies.putIfAbsent(way, () {
  final path = '${Directory.systemTemp.createTempSync('relation_tab').path}/try.motolii';
  File('$motoliiDir/../skin/book/test/data/relation_try.motolii').copySync(path);
  return path;
});

WidgetbookComponent relationTabsSet() => WidgetbookComponent(
  name: 'Relations tabs',
  useCases: [
    for (final (name, build) in relationTabs) WidgetbookUseCase(name: name, builder: (c) => Workspace(document: _fresh(name), extra: [WsPane('Relations $name', DockG.desk, build)])),
  ],
);
