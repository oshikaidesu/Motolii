import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

Future<Doc> mount(WidgetTester tester) async {
  tester.view.physicalSize = const Size(800, 4000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final doc = Doc(Kind.shape);
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
                  child: SizedBox(width: 372, child: PlacementIdeas()),
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  return doc;
}

Future<void> drag(WidgetTester tester, Offset from, Offset by) async {
  final g = await tester.createGesture(kind: PointerDeviceKind.mouse);
  await g.addPointer(location: from);
  await g.down(from);
  await tester.pump();
  for (var i = 1; i <= 5; i++) {
    await g.moveTo(from + by * (i / 5));
    await tester.pump();
  }
  await g.up();
  await g.removePointer();
  await tester.pump(const Duration(milliseconds: 400));
  expect(tester.takeException(), isNull);
}

Rect box(WidgetTester tester, String key) => tester.getRect(find.byKey(ValueKey(key)));

void main() {
  testWidgets('placement ideas lay out and each concept edits the Doc by hand', (tester) async {
    final doc = await mount(tester);
    expect(tester.takeException(), isNull);
    expect(find.text('Grab the array'), findsOneWidget);

    // A: copy 2 sits 60 px left of the stage centre at the default fit; dragging it sets Position Each
    expect(doc.get('pli_a_pos_each.x'), 100);
    final a = box(tester, 'pli_a_stage');
    await drag(tester, a.center + const Offset(-60, 0), const Offset(20, 10));
    expect(doc.get('pli_a_pos_each.x'), 150);
    expect(doc.get('pli_a_pos_each.y'), 25);
    expect(doc.undoSteps, 1);

    // B: the scatter track writes Position Random on both axes; the shape switch keeps the stage alive
    final b = box(tester, 'pli_b_scatter');
    await drag(tester, Offset(b.left + 58 + (b.width - 64 - 58) / 2, b.center.dy), Offset.zero);
    expect(doc.get('pli_b_pos_rand.x'), 100);
    expect(doc.get('pli_b_pos_rand.y'), 100);
    await tester.tap(find.text('GRID'));
    await tester.pump(const Duration(milliseconds: 400));
    expect(doc.get('pli_b_mode'), 2);
    expect(tester.takeException(), isNull);

    // C: dragging the Rotation graph's end up raises Rotation Each
    expect(doc.get('pli_c_rot_each'), 15);
    final c = box(tester, 'pli_c_rot_each');
    await drag(tester, c.centerRight - const Offset(30, 0), const Offset(0, -8));
    expect(doc.get('pli_c_rot_each'), greaterThan(15));

    // D: a press anywhere places the real anchor, snapped to the layer's top-left corner when near; the rail sets depth
    final d = box(tester, 'pli_d_stage');
    final left = (d.width - 34) / 2 - 66, topY = 78 - 42.0;
    await drag(tester, d.topLeft + Offset(left + 3, topY - 2), const Offset(1, 1));
    expect(doc.get('anchor.x'), 0);
    expect(doc.get('anchor.y'), 0);
    await drag(tester, d.topLeft + Offset(left - 40, topY + 20), Offset.zero);
    expect(doc.get('anchor.x'), lessThan(0));
    await drag(tester, Offset(d.right - 17, d.top + 26), Offset.zero);
    expect(doc.get('anchor.z'), 50);
  });
}
