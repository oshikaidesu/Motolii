// The door for trying a pane, a desk tool or a preset of your own on a real document, in a test (README.md tells how).
import 'dart:io';

import 'package:flutter/gestures.dart' show kDoubleTapMinTime;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/workspace/door.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Chooser, Ctx, InspectorPanel;
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart' show PropRow;
import 'package:lab_book/theme.dart' show ColorTheme;
import 'package:lab_book/workspace/zoom.dart' show WsZoom;
import 'package:lab_book/workspace/dock_parts.dart' show dockStarsAt;
import 'package:lab_book/workspace/timeline.dart' show WsTimeline;

export 'package:lab_book/motolii.dart' show motoliiDir;
export 'package:lab_book/workspace/door.dart';

/// Whether the inside is built (filament/edit.sh): a test that needs it says `skip: !insideBuilt`.
final insideBuilt = File('$motoliiDir/out/libmotolii.dylib').existsSync();

/// The window on a new, empty document (or [document]), with your panes and desk tools; its state.
Future<Ws> openWorkspace(WidgetTester t, {List<WsPane> extra = const [], List<DeskTool> tools = const [], String? document, Map<String, Object?> view = const {}, Size size = const Size(1600, 960)}) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  await t.binding.setSurfaceSize(size);   // (the live binding lays out at its surface's size, not the view's: flutter_test_config.dart)
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);   // (no picture of the stage: the seat's texture is the runner's)
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);   // (the system's menu bar is not there in a test)
  final doc = document ?? '${Directory.systemTemp.createTempSync('door').path}/door.motolii';
  openedDocument = doc;
  ColorTheme.at = '${Directory.systemTemp.path}/motolii_test_no_theme.xml'; // (the window's own skin and 100%, whatever the home folder keeps)
  dockStarsAt(File('${Directory.systemTemp.createTempSync('stars').path}/effect_stars.json'));   // (none starred, and the maker's own kept as they are)
  WsZoom.set(1, keep: false);
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc, extra: extra, tools: tools, view: view)));
  for (var i = 0; i < 1500 && find.byType(WsTimeline).evaluate().isEmpty; i++) {   // (the inside answers in real time, not the test's: Flutter's runAsync, as settled does; up to 30 s, a slow machine's opening)
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
    await t.pump();
  }
  await t.pumpAndSettle();
  return wsOf(t);
}

/// The document the window was last opened on.
String? openedDocument;

/// Every word the window shows now (each text and each field), sorted: the window as a hand sees it; not the words of
/// a moment (when it was saved, a change refused just now, where the mouse is).
List<String> windowWords(WidgetTester t) {
  // (nor where the mouse is: the stage's readout of it, under the hand of the moment)
  // (nor how many times the window waited for the inside: a count of the session, not of the document)
  final hud = find.descendant(of: find.byWidgetPredicate((w) => const {'_Hud', '_Waited'}.contains(w.runtimeType.toString())), matching: find.byType(Text)).evaluate().toSet();
  return [
    for (final e in find.byType(Text).evaluate())
      if (!hud.contains(e)) '${_rowOf(e)}${(e.widget as Text).data ?? (e.widget as Text).textSpan?.toPlainText() ?? ''}',
    for (final e in find.byType(EditableText).evaluate()) (e.widget as EditableText).controller.text,
  ].where((w) => !w.startsWith('Saved ') && !w.startsWith('Not taken')).toList()..sort();
}

/// The row a word lies in, if any ('opacity: '): a word told with where it is.
String _rowOf(Element e) {
  var row = '';
  e.visitAncestorElements((a) {
    if (a.widget is PropRow) {
      row = '${(a.widget as PropRow).id}: ';
      return false;
    }
    return true;
  });
  return row;
}

/// The inspector's list, for scrolling it (it builds only the cards in sight).
Finder get inspectorList => find.descendant(of: find.byType(InspectorPanel), matching: find.byType(Scrollable)).first;

/// The cards the inspector lists, by name, in sight or not; and its effects' cards, by name.
List<String> inspectorCards(WidgetTester t) => [for (final c in Ctx.read(t.element(find.byType(InspectorPanel))).doc.cards) c.name];
List<String> inspectorEffects(WidgetTester t) {
  final doc = Ctx.read(t.element(find.byType(InspectorPanel))).doc;
  return [for (final id in doc.fxOrder) doc.fxDefs.firstWhere((f) => f.id == id).name];
}

/// What [f] finds, brought into sight as a hand scrolls to it: the inspector builds only the cards in sight, so one
/// further down is not there until the list is scrolled to it.
Future<void> seen(WidgetTester t, Finder f) async {
  if (f.evaluate().isEmpty && inspectorList.evaluate().isNotEmpty) {
    final at = t.state<ScrollableState>(inspectorList).position; // (moved as a wheel moves it: a drag would land on a knob and scrub it)
    for (var y = 0.0; f.evaluate().isEmpty && y <= at.maxScrollExtent + 120; y += 120) {
      at.jumpTo(y.clamp(0, at.maxScrollExtent));
      await t.pump();
    }
  }
  await t.ensureVisible(f.first);
  await t.pumpAndSettle();
}

/// The window is the document's alone: closed and opened again on the same document, at the same thing, moment, keys
/// and desk tool, it shows the same words. What differs was held by the window, not the document.
Future<void> expectSameAfterReopen(WidgetTester t, {List<WsPane> extra = const [], List<DeskTool> tools = const []}) async {
  final ws = wsOf(t);
  for (var i = 0; i < 600 && ws.waiting; i++) {   // (the changes a hand told, done: as a hand sees the window once it rests; 30 s at most, as settled)
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
    await t.pump();
  }
  ws.reread();   // (what a test changed in the inside itself, past the window, read as the window reads a hand's)
  await t.pumpAndSettle();
  for (final e in find.byType(Scrollable).evaluate()) {   // (every list seen from its top, as a window opened again shows it: where a list is scrolled to is the hand's, not the document's)
    if ((e as StatefulElement).state case final ScrollableState l when l.position.hasPixels) l.position.jumpTo(l.position.minScrollExtent);
  }
  await t.pumpAndSettle();
  final selected = ws.selected, frame = ws.frame, keys = ws.keys.toList(), desk = ws.deskManual, shelf = ws.shelf, ink = ws.ink, edge = ws.edge, aim = ws.aim, animating = ws.animating, view = Map.of(ws.view), fronted = ws.fronted?.$1, doc = openedDocument!;
  final before = windowWords(t);
  await closeWorkspace(t);
  final again = await openWorkspace(t, extra: extra, tools: tools, document: doc, view: view);
  again.select(selected);
  for (final k in keys) {
    again.pickKey(k, add: true);
  }
  again.frame = frame;
  if (desk != null) again.openDesk(desk);
  if (fronted != null) again.front(fronted);   // (the pane a hand brought to the front: the window's, as the shelf is)
  again
    ..shelf = shelf
    ..ink = ink
    ..edge = edge
    ..aim = aim
    ..animating = animating   // (Animate all: how the hand works now, not the document's)
    ..reread();   // (the window told to look again: a view put back that equals nothing changed tells nobody)
  await t.pumpAndSettle();
  final after = windowWords(t);
  // (each word as many times as it shows: a None more, a name fewer)
  final lost = [...before], gained = <String>[];
  for (final w in after) {
    if (!lost.remove(w)) gained.add(w);
  }
  for (final w in [for (final w in lost) '- $w', for (final w in gained) '+ $w']) {
    debugPrint('REOPEN\t${t.testDescription.split(':').first}\t$w');
  }
  expect([for (final w in lost) '- $w', for (final w in gained) '+ $w'], isEmpty, reason: 'what the window showed (-) and shows (+) when opened again on the same document');
}

/// A word picked on a card's choice of that id, as a hand picks it: among its buttons (four or fewer) or from its menu.
Future<void> pick(WidgetTester t, String id, String word) async {
  final menu = find.byWidgetPredicate((w) => w is Chooser && w.id == id), buttons = find.byKey(ValueKey('choice:$id'));
  await seen(t, menu.evaluate().isNotEmpty ? menu : buttons);
  final on = menu.evaluate().isNotEmpty ? menu.first : buttons.first;
  if (menu.evaluate().isNotEmpty) {
    await t.tap(on);
    await t.pumpAndSettle();
    final words = find.text(word); // (the word in the list that floats over the rest: a thing's name is also a row of the timeline beneath it)
    await t.tap([for (var i = words.evaluate().length - 1; i >= 0; i--) words.at(i)].firstWhere((w) => t.hitTestOnBinding(t.getCenter(w)).path.any((e) => e.target == t.renderObject(w)), orElse: () => words.last));
  } else {
    await t.tap(find.descendant(of: on, matching: find.byWidgetPredicate((w) => w is Text && w.data?.toLowerCase() == word.toLowerCase()))); // (buttons say it in capitals)
  }
  await settled(t, wsOf(t));   // (the choice told to the inside, done in real time)
}

/// The changes a hand told the inside, all done (as the window shows them once it rests).
Future<void> settled(WidgetTester t, Ws ws) async {
  await t.pumpAndSettle();
  for (var i = 0; i < 600 && ws.waiting; i++) {   // (30 s at most, as opening is given: a clean CI Mac answers a step of many changes in more than 3)
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
    await t.pump();
  }
  await t.pumpAndSettle();
}

/// Two presses, as Flutter's own tests make a double tap (tap, kDoubleTapMinTime, tap): a dock tile is put on so (f052e7e).
Future<void> doubleTap(WidgetTester t, Finder f) async {
  await t.tap(f);
  await t.pump(kDoubleTapMinTime);
  await t.tap(f);
}

/// The window's state, as a pane reads it with WsScope.of.
Ws wsOf(WidgetTester t) => WsScope.read(t.element(find.byType(WsTimeline)));

/// The window taken down, so the inside is closed before the next test opens it.
Future<void> closeWorkspace(WidgetTester t) async {
  await t.pumpWidget(const SizedBox());
  t.view.reset();
}
