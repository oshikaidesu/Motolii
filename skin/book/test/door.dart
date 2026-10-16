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
  openedDocument = doc;
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc, extra: extra, tools: tools)));
  await t.pumpAndSettle();
  return wsOf(t);
}

/// The document the window was last opened on.
String? openedDocument;

/// Every word the window shows now (each text and each field), sorted: the window as a hand sees it; not the words of
/// a moment (when it was saved, a change refused just now).
List<String> windowWords(WidgetTester t) => [
  for (final e in find.byType(Text).evaluate()) (e.widget as Text).data ?? (e.widget as Text).textSpan?.toPlainText() ?? '',
  for (final e in find.byType(EditableText).evaluate()) (e.widget as EditableText).controller.text,
].where((w) => !w.startsWith('Saved ') && !w.startsWith('Not taken')).toList()..sort();

/// The window is the document's alone: closed and opened again on the same document, at the same thing, moment, keys
/// and desk tool, it shows the same words. What differs was held by the window, not the document.
Future<void> expectSameAfterReopen(WidgetTester t, {List<WsPane> extra = const [], List<DeskTool> tools = const []}) async {
  final ws = wsOf(t)..reread();   // (what a test changed in the inside itself, past the window, read as the window reads a hand's)
  await t.pumpAndSettle();
  final selected = ws.selected, frame = ws.frame, keys = ws.keys.toList(), desk = ws.deskManual, shelf = ws.shelf, ink = ws.ink, edge = ws.edge, aim = ws.aim, view = Map.of(ws.view), doc = openedDocument!;
  final before = windowWords(t);
  await closeWorkspace(t);
  final again = await openWorkspace(t, extra: extra, tools: tools, document: doc);
  again.view.addAll(view);   // (first: what is done next tells the window to look again)
  again.select(selected);
  for (final k in keys) {
    again.pickKey(k, add: true);
  }
  again.frame = frame;
  if (desk != null) again.openDesk(desk);
  again
    ..shelf = shelf
    ..ink = ink
    ..edge = edge
    ..aim = aim
    ..reread();   // (the window told to look again: a view put back that equals nothing changed tells nobody)
  await t.pumpAndSettle();
  final after = windowWords(t);
  final lost = [...before]..removeWhere(after.contains), gained = [...after]..removeWhere(before.contains);
  for (final w in [for (final w in lost) '- $w', for (final w in gained) '+ $w']) {
    debugPrint('REOPEN\t${t.testDescription.split(':').first}\t$w');
  }
  expect([for (final w in lost) '- $w', for (final w in gained) '+ $w'], isEmpty, reason: 'what the window showed (-) and shows (+) when opened again on the same document');
}

/// The window's state, as a pane reads it with WsScope.of.
Ws wsOf(WidgetTester t) => WsScope.read(t.element(find.byType(WsTimeline)));

/// The window taken down, so the inside is closed before the next test opens it.
Future<void> closeWorkspace(WidgetTester t) async {
  await t.pumpWidget(const SizedBox());
  t.view.reset();
}
