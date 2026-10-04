import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

void main() {
  testWidgets('Layout ideas solve, paint and write to Doc', (tester) async {
    tester.view.physicalSize = const Size(600, 3200);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final doc = Doc(Kind.group);
    await tester.pumpWidget(
      WidgetsApp(
        color: const Color(0xFF000000),
        builder: (c, _) => ListenableBuilder(
          listenable: doc,
          builder: (_, _) => Ctx(
            cfg: const Cfg(),
            doc: doc,
            child: Pop(
              child: Overlay.wrap(
                child: const SingleChildScrollView(
                  child: Align(
                    alignment: Alignment.topLeft,
                    child: SizedBox(width: 372, child: LayoutIdeas()),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(find.text('Box model'), findsOneWidget);
    expect(find.text('Grid tracks'), findsOneWidget);
    expect(find.text('Flex playground'), findsOneWidget);

    // A: dragging the left padding band widens Padding X; the Grid segment re-solves as a grid
    final a = tester.getTopLeft(find.byKey(const ValueKey('ly_a_stage')));
    await tester.dragFrom(a + const Offset(5, 63), const Offset(20, 0));
    await tester.pumpAndSettle();
    expect(doc.get('ly_a_px'), greaterThan(20));
    await tester.tap(find.text('GRID'));
    await tester.pumpAndSettle();
    expect(doc.s2['ly_a_mode'], 'Grid');
    expect(find.text('Cols'), findsWidgets);

    // B: a click on a column header cycles fr -> px; dragging Logo one cell right moves its column start
    final b = tester.getTopLeft(find.byKey(const ValueKey('ly_b_stage')));
    expect(doc.s2['ly_b_c0.k'], 'fr');
    await tester.tapAt(b + const Offset(47, 9));
    await tester.pumpAndSettle();
    expect(doc.s2['ly_b_c0.k'], 'px');
    expect(doc.s2['ly_b_track'], 'c0');
    final b2 = tester.getTopLeft(find.byKey(const ValueKey('ly_b_stage')));
    final logo = doc.get('ly_b_k0.c');
    await tester.dragFrom(b2 + const Offset(70, 66), const Offset(90, 0));
    await tester.pumpAndSettle();
    expect(doc.get('ly_b_k0.c'), greaterThan(logo!));

    // C: rotate flips the flow; the width handle narrows the box; a child chip steps its sizing
    await tester.tap(find.byKey(const ValueKey('ly_c_rotate')));
    await tester.pumpAndSettle();
    expect(doc.s2['ly_c_dir'], 'Column');
    final c = tester.getTopLeft(find.byKey(const ValueKey('ly_c_stage')));
    await tester.dragFrom(c + Offset(440 * 356 / 520, 60), const Offset(-100, 0));
    await tester.pumpAndSettle();
    expect(doc.get('ly_c_w'), lessThan(440));
    await tester.tap(find.text('Logo').last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Logo').last);
    await tester.pumpAndSettle();
    expect(doc.s2['ly_c_k0.sz'], 'Fill');

    // every card folds to its brief
    for (var i = 0; i < 3; i++) {
      await tester.tap(find.text('Layout').at(i));
      await tester.pumpAndSettle();
    }
    expect(doc.b['ly_a.open'], isFalse);
    expect(doc.b['ly_c.open'], isFalse);
    expect(tester.takeException(), isNull);
  });
}
