// The door for trying a pane, a desk tool or a preset of your own on a real document, in a test (README.md tells how).
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/workspace/door.dart';
import 'package:lab_book/workspace/timeline.dart' show WsTimeline;

export 'package:lab_book/motolii.dart' show motoliiDir;
export 'package:lab_book/workspace/door.dart';

/// Whether the inside is built (filament/edit.sh): a test that needs it says `skip: !insideBuilt`.
final insideBuilt = File('$motoliiDir/out/libmotolii.dylib').existsSync();

/// The window on a new, empty document (or [document]), with your panes and desk tools; its state.
Future<Ws> openWorkspace(WidgetTester t, {List<WsPane> extra = const [], List<DeskTool> tools = const [], String? document}) async {
  t.view.physicalSize = const Size(1600, 960);
  t.view.devicePixelRatio = 1;
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);   // (no picture of the stage: the seat's texture is the runner's)
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);   // (the system's menu bar is not there in a test)
  final doc = document ?? '${Directory.systemTemp.createTempSync('door').path}/door.motolii';
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc, extra: extra, tools: tools)));
  await t.pumpAndSettle();
  return wsOf(t);
}

/// The window's state, as a pane reads it with WsScope.of.
Ws wsOf(WidgetTester t) => WsScope.read(t.element(find.byType(WsTimeline)));

/// The window taken down, so the inside is closed before the next test opens it.
Future<void> closeWorkspace(WidgetTester t) async {
  await t.pumpWidget(const SizedBox());
  t.view.reset();
}
