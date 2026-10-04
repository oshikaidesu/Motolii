import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

Future<void> _pump(WidgetTester tester, Ws ws, {Size size = const Size(372, 300)}) async {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1;
  await tester.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (c, _) => WsScope(
        ws: ws,
        child: SizedBox(width: size.width, height: size.height, child: const WsDesk()),
      ),
    ),
  );
  await tester.pumpAndSettle();
  expect(tester.takeException(), isNull);
}

Finder get _toolsButton => find.descendant(of: find.byType(WsHeader), matching: find.byType(DeskIconButton));

void main() {
  testWidgets('Desk follows the selection: Tools, then Ease for keys, then Depth for the camera', (tester) async {
    addTearDown(tester.view.reset);
    final ws = Ws();
    await _pump(tester, ws);
    expect(ws.desk, 'Tools');
    expect(find.text('Follows selection'), findsOneWidget);
    for (final t in deskTools) {
      expect(find.text(t.name), findsWidgets);
    }

    ws.pickKey((layer: 'blob', frame: 48));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(find.text('Ease'), findsWidgets);
    expect(find.text('Blob'), findsOneWidget);
    expect(find.text('96'), findsWidgets);

    ws.select('cam');
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(ws.desk, 'Depth');
    expect(find.text('On Blob'), findsOneWidget);
  });

  testWidgets('A tile opens its tool by hand; Pinned returns to following; the header button opens Tools', (tester) async {
    addTearDown(tester.view.reset);
    final ws = Ws();
    await _pump(tester, ws);

    await tester.tap(find.text('Blend'));
    await tester.pumpAndSettle();
    expect(ws.deskManual, 'Blend');
    expect(ws.desk, 'Blend');
    expect(find.text('Not designed in the lab yet.'), findsOneWidget);

    await tester.tap(find.text('Pinned · Follow'));
    await tester.pumpAndSettle();
    expect(ws.deskManual, isNull);
    expect(ws.desk, 'Tools');

    ws.pickKey((layer: 'title', frame: 36));
    await tester.pumpAndSettle();
    expect(ws.desk, 'Ease');
    await tester.tap(_toolsButton);
    await tester.pumpAndSettle();
    expect(ws.deskManual, 'Tools');
    expect(find.text('AUTO'), findsOneWidget);

    await tester.tap(find.text('Depth'));
    await tester.pumpAndSettle();
    expect(ws.desk, 'Depth');
    expect(tester.takeException(), isNull);
  });

  testWidgets('Ease: presets, scrubbed numbers and Reverse change the curve', (tester) async {
    addTearDown(tester.view.reset);
    final ws = Ws()..pickKey((layer: 'blob', frame: 48));
    await _pump(tester, ws);
    expect(find.text('0.25'), findsNWidgets(2));

    // Drag the out handle (0.25, 0.10) on the graph: the numbers follow.
    final graph = tester.getRect(find.byWidgetPredicate((w) => w is CustomPaint && '${w.painter.runtimeType}' == '_GraphPainter'));
    final box = const EdgeInsets.fromLTRB(30, 10, 14, 22).deflateRect(graph);
    final handle = Offset(box.left + .25 * box.width, box.bottom - (.1 + .42) / 2.04 * box.height);
    await tester.dragFrom(handle, const Offset(30, -30));
    await tester.pumpAndSettle();
    expect(find.text('0.25'), findsOneWidget);
    expect(find.text('0.10'), findsNothing);

    await tester.tap(find.text('Back'));
    await tester.pumpAndSettle();
    expect(find.text('1.56'), findsOneWidget);
    expect(find.text('34'), findsOneWidget);

    await tester.drag(find.text('x1'), const Offset(20, 0));
    await tester.pumpAndSettle();
    expect(find.text('0.34'), findsNothing);

    await tester.tap(find.text('Linear'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Out'));
    await tester.pumpAndSettle();
    expect(find.text('0.58'), findsOneWidget);
    final reverse = find.descendant(of: find.byType(WsDesk), matching: find.byType(DeskIconButton)).last;
    await tester.tap(reverse);
    await tester.pumpAndSettle();
    expect(find.text('0.42'), findsOneWidget);

    // The shaped ease is kept for the span when the desk leaves and comes back.
    ws.select('cam');
    await tester.pumpAndSettle();
    ws.pickKey((layer: 'blob', frame: 48));
    await tester.pumpAndSettle();
    expect(find.text('0.42'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Depth: scrubbing focal and focus updates the lens; a free focus drops the target', (tester) async {
    addTearDown(tester.view.reset);
    final ws = Ws()..select('cam');
    await _pump(tester, ws);
    expect(find.text('50 mm'), findsWidgets);

    await tester.drag(find.text('Focal'), const Offset(40, 0));
    await tester.pumpAndSettle();
    expect(find.text('50 mm'), findsNothing);

    await tester.drag(find.text('Dist'), const Offset(30, 0));
    await tester.pumpAndSettle();
    expect(find.text('Focus free'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Desk lays out narrower and shorter without overflow', (tester) async {
    addTearDown(tester.view.reset);
    for (final setup in <void Function(Ws)>[(_) {}, (ws) => ws.pickKey((layer: 'blob', frame: 48)), (ws) => ws.select('cam'), (ws) => ws.openDesk('Notes')]) {
      final ws = Ws();
      setup(ws);
      await _pump(tester, ws, size: const Size(300, 230));
    }
  });
}
