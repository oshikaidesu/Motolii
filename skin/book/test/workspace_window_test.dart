import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart';
import 'package:lab_book/workspace/ws.dart';

Future<Ws> _pump(WidgetTester t, Size size) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => const Workspace()));
  await t.pumpAndSettle();
  expect(t.takeException(), isNull);
  return WsScope.read(t.element(find.byType(WsTimeline)));
}

void main() {
  testWidgets('the window lays out at the reference size, the minimum, and scaled below it', (t) async {
    addTearDown(t.view.reset);
    for (final s in const [Size(1600, 960), Size(1280, 800), Size(1100, 700)]) {
      await _pump(t, s);
    }
    for (final pane in const ['Stage', 'Inspector', 'Timeline', 'Desk']) {
      expect(find.text(pane), findsOneWidget, reason: 'a named tab per pane');
    }
    for (final shelf in const ['Media', 'Effects', 'Fonts', 'Colors', 'Create']) {
      expect(find.bySemanticsLabel(shelf), findsOneWidget, reason: 'an icon tab per shelf');
    }
  });

  testWidgets('selection reaches the Inspector; its colour link opens the dock shelf', (t) async {
    addTearDown(t.view.reset);
    final ws = await _pump(t, const Size(1600, 960));
    expect(find.text('Blob'), findsWidgets);

    ws.select('ring');
    await t.pumpAndSettle();
    expect(find.text('Ring'), findsWidgets);
    expect(t.takeException(), isNull);

    final link = find.byKey(const ValueKey('link:colors:'));
    await t.ensureVisible(link);
    await t.pumpAndSettle();
    await t.tap(link);
    await t.pumpAndSettle();
    expect(ws.shelf, 'Colors');
    expect(find.text('Search palettes'), findsOneWidget, reason: 'the route brings the Colors tab forward');

    ws.select('cam');
    await t.pumpAndSettle();
    expect(t.takeException(), isNull);
  });

  testWidgets('the shade switch turns the window light and back, keeping the edit', (t) async {
    addTearDown(t.view.reset);
    addTearDown(() => Grey.shade.value = Shade.dark);
    final ws = await _pump(t, const Size(1600, 960));
    ws.select('ring');
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
    expect(ws.layer?.id, 'ring');
    expect(t.takeException(), isNull);

    await flip();
    expect(Grey.light, isFalse);
    expect(WsT.card, N.g13);
  });

  testWidgets('each shelf is its own tab: clicking its icon brings it forward and sets Ws.shelf', (t) async {
    addTearDown(t.view.reset);
    final ws = await _pump(t, const Size(1600, 960));
    expect(find.text('Search files and media'), findsOneWidget);
    for (final (shelf, hint) in const [('Effects', 'Search effects and features'), ('Fonts', 'Search 15 families'), ('Media', 'Search files and media')]) {
      await t.tap(find.bySemanticsLabel(shelf));
      await t.pumpAndSettle();
      expect(ws.shelf, shelf);
      expect(find.text(hint), findsOneWidget);
    }
    expect(t.takeException(), isNull);
  });
}
