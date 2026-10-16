// The desk on a real document (test/door.dart): it follows the selection, a tool opened by hand stays, Ease writes the
// document's way of a stretch, Depth the camera's lens.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart' show WsHeader;

import 'door.dart';

/// The window on a new document with a cube whose x has two keys, at 0 and 1 s.
Future<(Ws, int)> _keyed(WidgetTester t, {Size size = const Size(1600, 960)}) async {
  final ws = await openWorkspace(t, size: size), m = ws.inside!;
  final cube = m.add('solid', 'cube');
  expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
  ws.reread();
  await t.pumpAndSettle();
  return (ws, cube);
}

Finder get _toolsButton => find.descendant(of: find.byType(WsHeader), matching: find.byType(DeskIconButton));

void main() {
  testWidgets('the desk follows the selection: Tools, then Ease for keys, then Depth for the camera', (t) async {
    final (ws, cube) = await _keyed(t);
    expect(ws.desk, 'Tools');
    expect(find.text('Follows selection'), findsOneWidget);
    for (final tool in deskTools) {
      expect(find.text(tool.name), findsWidgets);
    }
    ws.pickKey((layer: '$cube', frame: 0));
    await t.pumpAndSettle();
    expect(ws.desk, 'Ease');
    final cam = ws.inside!.add('camera', 'camera');
    ws
      ..reread()
      ..select('$cam');
    await t.pumpAndSettle();
    expect(ws.desk, 'Depth');
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
    expect(ws.desk, 'Ease');
    await t.tap(_toolsButton);
    await t.pumpAndSettle();
    expect(ws.deskManual, 'Tools');
    await t.tap(find.text('Depth').last);
    await t.pumpAndSettle();
    expect(ws.desk, 'Depth');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  test('Ease: every preset said to the document reads back as itself, wherever it sits in the row', () {
    for (final p in easePresets) {
      expect(heard(said(p)).at(.3), closeTo(p.at(.3), 1e-9), reason: p.name);
    }
  });

  testWidgets('Ease: a preset is the document\'s way of the stretch, Reverse turns it, and it is read back from the document', (t) async {
    final (ws, cube) = await _keyed(t);
    final m = ws.inside!;
    ws.pickKey((layer: '$cube', frame: 0));
    await t.pumpAndSettle();
    for (final (tile, said) in const [('Back', 'Bezier 0.34 1.56'), ('Out', 'Bezier 0 0 0.58 1')]) {
      await t.tap(find.descendant(of: find.byType(WsDesk), matching: find.text(tile)).first);
      await settled(t, ws);
      expect(m.wayOf(cube, 'x', 0), startsWith(said), reason: tile);
    }
    await t.tap(find.descendant(of: find.byType(WsDesk), matching: find.byType(DeskIconButton)).last); // (Reverse)
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Bezier 0.42 0 1 1'), reason: 'Out turned is In');
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
    await t.drag(find.text('Focal'), const Offset(40, 0));
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
}
