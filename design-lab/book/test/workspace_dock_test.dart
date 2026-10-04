import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/dock.dart';
import 'package:lab_book/workspace/ws.dart';

const _shelves = ['Media', 'Effects', 'Fonts', 'Colors', 'Create'];

Future<Ws> _pump(WidgetTester tester, Size size, [String shelf = 'Media']) async {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final ws = Ws();
  await tester.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(
        ws: ws,
        child: Align(
          alignment: Alignment.topLeft,
          child: SizedBox(
            width: size.width,
            height: size.height,
            child: WsShelf(shelf, key: ValueKey(shelf)),
          ),
        ),
      ),
    ),
  );
  return ws;
}

void main() {
  testWidgets('every shelf lays out at the reference size and when smaller', (tester) async {
    for (final size in const [Size(240, 918), Size(240, 600)]) {
      for (final s in _shelves) {
        await _pump(tester, size, s);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: '$s at $size');
        expect(find.text(s), findsWidgets);
      }
    }
  });

  testWidgets('picking items, chips, search and the footer action', (tester) async {
    final ws = await _pump(tester, const Size(300, 918));

    await tester.tap(find.text('city_pass.mp4'));
    await tester.pumpAndSettle();
    expect(find.textContaining('0:47 · 3840×2160'), findsOneWidget);

    await tester.tap(find.text('Place'));
    await tester.pumpAndSettle();
    expect(find.text('Placed'), findsOneWidget);

    await tester.tap(find.byKey(const ValueKey('browser:folder:Stills')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('background.png'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Select'));
    await tester.pumpAndSettle();
    expect(ws.selected, 'bg');

    await tester.tap(find.byKey(const ValueKey('browser:folder:HDR')));
    await tester.pumpAndSettle();
    expect(find.text('coda_intro.mov'), findsNothing);
    expect(find.text('studio_soft.exr'), findsOneWidget);

    await _pump(tester, const Size(300, 918), 'Effects');
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText), 'trim');
    await tester.pumpAndSettle();
    expect(find.text('Trim Paths'), findsOneWidget);
    expect(find.text('Bloom'), findsNothing);
    await tester.tap(find.text('Trim Paths'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Add'));
    await tester.pumpAndSettle();
    expect(find.text('ON'), findsOneWidget);

    await _pump(tester, const Size(300, 918));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('browser:crumb:Project')));
    await tester.pumpAndSettle();
    expect(find.text('coda_v12.mp4'), findsNothing);
    await tester.tap(find.byKey(const ValueKey('browser:file:Renders')));
    await tester.pumpAndSettle();
    expect(find.text('coda_v12.mp4'), findsOneWidget, reason: 'a folder tile opens the folder');

    expect(tester.takeException(), isNull);
  });

  testWidgets('Media is one browser: tree beside the contents when wide, above them when narrow; search covers the location', (tester) async {
    await _pump(tester, const Size(520, 900));
    expect(find.byKey(const ValueKey('browser:tree-grip')), findsNothing);
    final tree = tester.getTopLeft(find.byKey(const ValueKey('browser:folder:Footage')));
    final file = tester.getTopLeft(find.byKey(const ValueKey('browser:file:city_pass')));
    expect(file.dx, greaterThan(tree.dx + 100), reason: 'wide: contents stand right of the tree');

    await _pump(tester, const Size(300, 900));
    expect(find.byKey(const ValueKey('browser:tree-grip')), findsOneWidget);
    expect(
      tester.getTopLeft(find.byKey(const ValueKey('browser:file:city_pass'))).dy,
      greaterThan(tester.getTopLeft(find.byKey(const ValueKey('browser:folder:Footage'))).dy),
    );

    await tester.enterText(find.byType(EditableText), '.wav');
    await tester.pumpAndSettle();
    expect(find.text('Search in Project'), findsOneWidget);
    expect(find.text('Beat.wav'), findsWidgets);
    expect(find.text('city_pass.mp4'), findsNothing);
    expect(tester.takeException(), isNull);
  });
}
