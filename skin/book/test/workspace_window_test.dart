// The window as a whole, on a real document (test/door.dart): its panes, a pane and a desk tool of your own, the
// selection reaching the inspector, the shade, the shelves.
import 'package:flutter/services.dart' show LogicalKeyboardKey;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/keys.dart' show WsKeys;
import 'package:lab_book/workspace/ws.dart' show WsT;

import 'door.dart';

void main() {
  testWidgets('the window lays out at the reference size, the minimum, and scaled below it', (t) async {
    for (final s in const [Size(1600, 960), Size(1280, 800), Size(1100, 700)]) {
      await openWorkspace(t, size: s);
      expect(t.takeException(), isNull, reason: 'at $s');
    }
    for (final pane in const ['Stage', 'Inspector', 'Timeline', 'Desk']) {
      expect(find.text(pane), findsWidgets, reason: 'a named tab per pane');
    }
    for (final shelf in const ['Media', 'Effects', 'Fonts', 'Colors', 'Create']) {
      expect(find.bySemanticsLabel(shelf), findsOneWidget, reason: 'an icon tab per shelf');
    }
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a pane of your own joins the window through Workspace(extra:), no file of the window changed', (t) async {
    final notes = WsPane('Notes', null, () => Builder(builder: (c) => Text('picked: ${WsScope.of(c).selected}')));
    final ws = await openWorkspace(t, extra: [notes]), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(find.text('Notes'), findsWidgets, reason: 'its tab, beside the Inspector');
    await t.tap(find.text('Notes').first);
    await t.pumpAndSettle();
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(find.text('picked: $cube'), findsOneWidget, reason: 'it reads the window as the panes of the window do');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a desk tool of your own joins the desk through Workspace(tools:), no file of the window changed', (t) async {
    const mine = DeskTool('Snap', Color(0xFF55AA88), 'Picked thing', build: _Snap.new);
    final ws = await openWorkspace(t, tools: [mine]);
    expect(ws.tools.map((x) => x.name), contains('Snap'), reason: 'after the window\'s own');
    ws.openDesk('Snap');
    await t.pumpAndSettle();
    expect(find.text('my snap tool'), findsOneWidget, reason: 'the desk holds what the tool builds');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('selection reaches the Inspector', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), ring = m.add('solid', 'ring');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    expect(find.text('Rect'), findsWidgets);
    ws.select('$ring');
    await t.pumpAndSettle();
    expect(find.text('Ring'), findsWidgets);
    ws.select(null);
    await t.pumpAndSettle();
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the shade switch turns the window light and back, keeping the edit', (t) async {
    addTearDown(() => Grey.shade.value = Shade.dark);
    final ws = await openWorkspace(t), m = ws.inside!;
    final ring = m.add('solid', 'ring');
    ws
      ..reread()
      ..select('$ring');
    await t.pumpAndSettle();
    final shade = find.byKey(const ValueKey('ws-shade'));
    Future<void> flip() async {
      await t.tap(shade);
      // The flip reassembles the app, which holds pointer events until its frame lands.
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pumpAndSettle();
    }

    await flip();
    expect(Grey.light, isTrue);
    expect(WsT.card, const Color(0xFFFFFFFF));
    expect(ws.layer?.id, '$ring');
    expect(t.takeException(), isNull);
    await flip();
    expect(Grey.light, isFalse);
    expect(WsT.card, N.g13);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('each shelf is its own tab: clicking its icon brings it forward and sets Ws.shelf', (t) async {
    final ws = await openWorkspace(t);
    for (final (shelf, hint) in const [('Effects', 'Search effects'), ('Fonts', 'Search '), ('Media', 'Search files')]) {
      await t.tap(find.bySemanticsLabel(shelf));
      await t.pumpAndSettle();
      expect(ws.shelf, shelf);
      expect(find.textContaining(hint), findsWidgets);
    }
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Edit > Deselect All, Cmd+Shift+A (After Effects\', Premiere Pro\'s): nothing picked, with a thing over the whole stage too', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(ws.selected, '$cube');
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.keyA);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await t.pumpAndSettle();
    expect(ws.selected, isNull, reason: 'golf_noise.md (63): no way back to nothing picked but a click below the rows');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Edit > Extract Work Area takes the work area out of every thing and closes the gap, one step (After Effects\'; golf (72), P18)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'rect');
    expect(m.set(a, 'duration', 10) && m.set(b, 'start', 6) && m.set(b, 'duration', 6), isTrue, reason: m.trouble);
    Future<void> settle() async {
      for (var i = 0; i < 200 && ws.waiting; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
        await t.pump();
      }
      ws.reread();
      await t.pumpAndSettle();
    }
    ws.after(m.later('work', number: 4, word: '6'));
    await settle();
    t.widget<WsKeys>(find.byType(WsKeys)).commands['edit.extractWork']!();   // (the menu's: Edit > Extract Work Area)
    await settle();
    final c = m.things.firstWhere((x) => x != a && x != b);
    expect([for (final x in [a, b, c]) (m.value(x, 'start', 0), m.value(x, 'duration', 0))], [(0, 4), (4, 6), (4, 4)], reason: 'cut at 4, the part after brought back to 4; the other from 6 to 4');
    expect(m.seconds, closeTo(10, 1e-6), reason: '12 seconds made 10');
    expect(m.undo(), isTrue);
    expect(m.things.length, 2, reason: 'one step');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Cmd+Opt+B sets the work area to the picked things, or to the whole with nothing picked (After Effects\'; golf (82))', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'rect');
    expect(m.set(a, 'start', 2) && m.set(a, 'duration', 3) && m.set(b, 'start', 6) && m.set(b, 'duration', 6), isTrue, reason: m.trouble);
    Future<void> settle() async {
      for (var i = 0; i < 200 && ws.waiting; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
        await t.pump();
      }
      ws.reread();
      await t.pumpAndSettle();
    }
    Future<void> cmdOptB() async {
      await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await t.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.keyB);
      await t.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
      await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await settle();
    }
    ws.after(m.later('work', number: 4, word: '6'));
    await settle();
    ws.select('$a');
    await t.pumpAndSettle();
    await cmdOptB();
    expect(m.work, (2.0, 5.0), reason: 'the picked thing\'s own time');
    ws.pickMany(['$a', '$b']);
    await t.pumpAndSettle();
    await cmdOptB();
    expect(m.work, (2.0, 12.0), reason: 'from the first start to the last end');
    t.widget<WsKeys>(find.byType(WsKeys)).commands['edit.deselectAll']!();
    await t.pumpAndSettle();
    await cmdOptB();
    expect(m.work, (0.0, m.seconds), reason: 'nothing picked: the whole');
    expect(m.undo() && m.work == (2.0, 12.0), isTrue, reason: 'one step each');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}

class _Snap extends StatelessWidget {
  const _Snap();
  @override
  Widget build(BuildContext context) => const Text('my snap tool');
}
