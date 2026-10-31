// The desk on a real document (test/door.dart): it follows the selection, a tool opened by hand stays, Ease writes the
// document's way of a stretch, Depth the camera's lens.
import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_state.dart' show easeJump;
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
    await t.tap(find.byKey(const ValueKey('ease.Bounce')));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Bounce'), reason: m.trouble);
    expect(m.way(cube, 'x', 0, 'Bezier 0 0 0.58 1'), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ease-reverse')));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Bezier 0.42 0 1 1'), reason: 'Out turned is In');
    // (no key picked: the stretch the playhead is in, of the picked thing, as Alight Motion opens its ease: golf_noise.md ⑫)
    ws
      ..clearKeys()
      ..select('$cube')
      ..frame = 30;   // (0.5 s, between its keys at 0 and 1 s)
    await t.pumpAndSettle();
    expect(find.textContaining('between two keys'), findsNothing, reason: 'the stretch from 0 to 60 is shown');
    await t.tap(find.byKey(const ValueKey('ease.Linear')));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), 'Linear', reason: 'put on the stretch the playhead is in, a curve back straight');
    ws.frame = 90;   // (after the last key: no stretch)
    await t.pumpAndSettle();
    expect(find.textContaining('between two keys'), findsOneWidget, reason: 'no stretch: it says how to have one (golf (66))');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t); // (what the desk shows is read back from the document, not kept by the window)
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Ease: the keys at the two ends are pulled up and down; with Jump on a key a frame outside keeps the value as it was (three points), off only the key moves', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', .5, 0) && m.key(cube, 'x', 1, 0), isTrue, reason: m.trouble);   // (a flat stretch: the hit is made from it)
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 30))
      ..front('Ease');
    await t.pumpAndSettle();
    final r = t.getRect(find.byType(EaseGraph)), g = r.deflate(r.width * 42 / 575);   // (AbsentMotion's geoOf)
    double y(double v) => g.bottom - (v + .25) / 1.5 * g.height;   // (the graph's measure: desk_ease_handles.dart easeReach)
    Future<void> pull(Offset from, double to, {String? says}) async {
      final hand = await t.startGesture(from, kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(0, -6));   // (up and down first: the key, not the handle on it)
      await hand.moveTo(Offset(from.dx, y(to)));
      await t.pump();
      if (says != null) {   // (while it is pulled, what it makes the value, in the inspector's units: golf_noise.md (70))
        expect((t.widget(find.byKey(const ValueKey('ease-end-says'))) as Text).data, startsWith(says));
      }
      await hand.up();
      await settled(t, ws);
    }
    List<double> keys() => [for (final s in m.keyMoments(cube, 'x')) (s * 60).roundToDouble()];
    final bar = r.width * 42 / 575 * .55;
    await pull(Offset(g.left - bar, y(0)), .25, says: 'Position X');   // (the start up a quarter: a quarter of x's knob span, 8, as the stretch is flat)
    expect(keys(), [29, 30, 60], reason: 'a key a frame before keeps the value as it was');
    expect(m.keyValues(cube, 'x')[0], closeTo(0, 1e-9));
    expect(m.keyValues(cube, 'x')[1], closeTo(2, .1), reason: 'the start key moved up: it jumps there');
    expect(m.keyValues(cube, 'x')[2], closeTo(0, 1e-9), reason: 'and the stretch goes down to the end key\'s value, as it was');
    await t.tap(find.byKey(const ValueKey('card-more:Ease')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('card-jump:Ease')));
    await t.pumpAndSettle();
    addTearDown(() => easeJump = true);
    expect(easeJump, isFalse);
    expect(t.widget<EaseGraph>(find.byType(EaseGraph)).ends.$1, closeTo(.25, .02), reason: 'let go, it stands where it was left');
    await pull(Offset(g.right + bar, y(1)), .5);   // (the end down half the bar: half of x's knob span, 8)
    expect(keys(), [29, 30, 60], reason: 'Jump off: no key after it');
    expect(m.keyValues(cube, 'x')[2], closeTo(-4, .1));
    expect(t.widget<EaseGraph>(find.byType(EaseGraph)).ends.$2, closeTo(.5, .02));
    expect(m.undo() && t.widget<EaseGraph>(find.byType(EaseGraph)).ends.$2 > 0, isTrue);
    ws.reread();
    await t.pumpAndSettle();
    expect(t.widget<EaseGraph>(find.byType(EaseGraph)).ends.$2, closeTo(1, .02), reason: 'undone, it goes back up its bar');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Ease: the shapes in one row beside the graph, a press each, each shape\'s own handles on its curve', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    for (final name in ['Linear', 'Bounce', 'Elastic', 'Cyclic', 'Random', 'Steps', 'ElasticSteps']) {
      expect(find.byKey(ValueKey('ease.$name')).hitTestable(), findsOneWidget, reason: 'in sight, nothing to open');
    }
    await t.tap(find.byKey(const ValueKey('ease.Cyclic')));
    await settled(t, ws);
    final said = m.wayOf(cube, 'x', 0).split(' ');
    expect(said.first, 'Cyclic', reason: m.trouble);
    final graph = find.byType(EaseGraph), r = t.getRect(graph), g = r.deflate(r.width * 42 / 575);
    Offset px(double x, double y) => Offset(g.left + x * g.width, g.bottom - (y + .5) / 2 * g.height);   // (Cyclic's reach: easeReach)
    final period = double.parse(said[1]);
    await t.timedDragFrom(px(period, 0), px(period + .2, 0) - px(period, 0), const Duration(milliseconds: 300));
    await settled(t, ws);
    expect(double.parse(m.wayOf(cube, 'x', 0).split(' ')[1]), closeTo(period + .2, .03), reason: 'its handle carries its period along');
    expect(t.takeException(), isNull);
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
