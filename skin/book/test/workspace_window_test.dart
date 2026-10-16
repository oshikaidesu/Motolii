// The window as a whole, on a real document (test/door.dart): its panes, a pane and a desk tool of your own, the
// selection reaching the inspector, the shade, the shelves.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/tokens.dart';
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
}

class _Snap extends StatelessWidget {
  const _Snap();
  @override
  Widget build(BuildContext context) => const Text('my snap tool');
}
