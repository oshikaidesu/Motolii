// The desk on a real document (test/door.dart): it follows the selection, a tool opened by hand stays, Ease writes the
// document's way of a stretch, Depth the camera's lens.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart' show WsHeader;

import 'door.dart';

/// The window on a new document with a cube whose x has two keys, at 0 and 1 s.
Future<(Ws, int)> _keyed(WidgetTester t, {Size size = const Size(1600, 960), bool desk = true}) async {
  final ws = await openWorkspace(t, size: size), m = ws.inside!;
  final cube = m.add('solid', 'cube');
  expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
  ws.reread();
  if (desk) ws.front('Desk');   // (a pane among others in its tabs: brought to the front, as its tab is pressed)
  await t.pumpAndSettle();
  return (ws, cube);
}

Finder get _toolsButton => find.descendant(of: find.byType(WsHeader), matching: find.byType(DeskIconButton));

void main() {
  testWidgets('the desk is Tools whatever is picked: Ease and Camera are panes of their own (c1d8253)', (t) async {
    final (ws, cube) = await _keyed(t);
    expect(ws.desk, 'Tools');
    expect(find.text('Follows selection'), findsOneWidget);
    for (final tool in deskTools) {
      expect(find.text(tool.name), findsWidgets);
    }
    ws.pickKey((layer: '$cube', frame: 0));
    await t.pumpAndSettle();
    expect(ws.desk, 'Tools', reason: 'keys picked: the Ease pane shows them, the desk stays');
    final cam = ws.inside!.add('camera', 'camera');
    ws
      ..reread()
      ..select('$cam');
    await t.pumpAndSettle();
    expect(ws.desk, 'Tools', reason: 'a camera picked: the Camera pane shows it, the desk stays');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a tile opens its tool by hand; Pinned returns to following; the header button opens Tools', (t) async {
    final (ws, cube) = await _keyed(t);
    await t.tap(find.text('Blend').last);
    await t.pumpAndSettle();
    expect([ws.deskManual, ws.desk], ['Blend', 'Blend']);
    await t.tap(find.text('Pinned · Follow'));
    await t.pumpAndSettle();
    expect([ws.deskManual, ws.desk], [null, 'Tools']);
    ws.pickKey((layer: '$cube', frame: 0));
    await t.pumpAndSettle();
    expect(ws.desk, 'Tools', reason: 'followed: Tools (keys are the Ease pane\'s)');
    await t.tap(_toolsButton);
    await t.pumpAndSettle();
    expect(ws.deskManual, 'Tools');
    await t.tap(find.text('Align').last);   // (the desk's tools are Blend and Align: Depth is the Camera pane's since c1d8253)
    await t.pumpAndSettle();
    expect(ws.desk, 'Align');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  test('Ease: every preset said to the document reads back as itself, wherever it sits in the row', () {
    for (final p in easePresets) {
      expect(heard(said(p)).at(.3), closeTo(p.at(.3), 1e-9), reason: p.name);
    }
  });

  testWidgets('Ease: a preset is the document\'s way of the stretch, Reverse turns it, and it is read back from the document', (t) async {
    final (ws, cube) = await _keyed(t, desk: false);
    final m = ws.inside!;
    ws
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');   // (a pane of its own since c1d8253)
    await t.pumpAndSettle();
    for (final (tile, said) in const [('Back', 'Bezier 0.34 1.56'), ('Out', 'Bezier 0 0 0.58 1')]) {
      await t.tap(find.descendant(of: find.byType(DeskEase), matching: find.text(tile)).first);
      await settled(t, ws);
      expect(m.wayOf(cube, 'x', 0), startsWith(said), reason: tile);
    }
    await t.tap(find.descendant(of: find.byType(DeskEase), matching: find.byType(DeskIconButton)).last); // (Reverse)
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Bezier 0.42 0 1 1'), reason: 'Out turned is In');
    // (no key picked: the stretch the playhead is in, of the picked thing, as Alight Motion opens its ease: golf_noise.md ⑫)
    ws
      ..clearKeys()
      ..select('$cube')
      ..frame = 30;   // (0.5 s, between its keys at 0 and 1 s)
    await t.pumpAndSettle();
    expect(find.textContaining('No stretch'), findsNothing, reason: 'the stretch from 0 to 60 is shown');
    await t.tap(find.descendant(of: find.byType(DeskEase), matching: find.text('Out')).first);
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Bezier 0 0 0.58 1'), reason: 'put on the stretch the playhead is in');
    ws.frame = 90;   // (after the last key: no stretch)
    await t.pumpAndSettle();
    expect(find.textContaining('No stretch'), findsOneWidget);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t); // (what the desk shows is read back from the document, not kept by the window)
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Depth: scrubbing Focal changes the camera\'s zoom in the document', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final cam = m.add('camera', 'camera');
    ws
      ..reread()
      ..select('$cam');
    await t.pumpAndSettle();
    final was = m.value(cam, 'zoom', 0) ?? 1;
    await t.timedDrag(find.text('Focal'), const Offset(40, 0), const Duration(milliseconds: 300));
    await settled(t, ws);
    expect(m.value(cam, 'zoom', 0), isNot(was), reason: 'the focal length is its zoom');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the desk lays out at the smallest window without overflow', (t) async {
    for (final setup in <void Function(Ws, int)>[(_, _) {}, (ws, c) => ws.pickKey((layer: '$c', frame: 0)), (ws, _) => ws.openDesk('Notes')]) {
      final (ws, cube) = await _keyed(t, size: const Size(1100, 700));
      setup(ws, cube);
      await t.pumpAndSettle();
      expect(t.takeException(), isNull);
      await closeWorkspace(t);
    }
  }, skip: !insideBuilt);

  testWidgets('Blend: a layer\'s way of lying on what is under, picked from its pictures in After Effects\' groups (Alight Motion\'s blend picker)', (t) async {
    final (ws, _) = await _keyed(t);
    final m = ws.inside!, rect = m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    await t.tap(find.text('Blend').last);
    await t.pumpAndSettle();
    await t.tap(find.byKey(ValueKey('blend.$rect')));
    await t.pumpAndSettle();
    final ways = find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('blend.$rect.'));
    expect(ways.evaluate().length, m.words(rect, 'blend').length, reason: 'every way the inside names, each its picture');
    await t.tap(find.byKey(ValueKey('blend.$rect.multiply')));
    await settled(t, ws);
    expect(m.word(rect, 'blend'), 'multiply', reason: m.trouble);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
