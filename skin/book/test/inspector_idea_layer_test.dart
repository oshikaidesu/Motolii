import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

Finder _painted(String name) => find.byWidgetPredicate((w) => w is CustomPaint && w.painter.runtimeType.toString() == name);

Future<void> _drag(WidgetTester tester, Offset from, Offset to) async {
  final g = await tester.startGesture(from);
  for (var i = 1; i <= 6; i++) {
    await g.moveTo(Offset.lerp(from, to, i / 6)!);
    await tester.pump();
  }
  await g.up();
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('Layer ideas build and each stage writes to Doc', (tester) async {
    tester.view.physicalSize = const Size(600, 3200);
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
                    child: SizedBox(width: 372, child: LayerIdeas()),
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
    expect(find.text('Mask stage'), findsWidgets);

    // A: a mask point drags on the stage
    final stage = _painted('_LiMaskPaint'), so = tester.getTopLeft(stage), sw = tester.getSize(stage).width;
    final lr = Rect.fromLTRB(12, 8, sw - 12, 168 - 20), x0 = doc.get('li_m0.p0.x')!;
    final p0 = so + lr.topLeft + Offset(x0 * lr.width, doc.get('li_m0.p0.y')! * lr.height);
    await _drag(tester, p0, p0 + const Offset(16, 0));
    expect(doc.get('li_m0.p0.x'), greaterThan(x0));
    // ...and its mode chip writes the mode
    await tester.tap(find.text('INTER').first);
    await tester.pumpAndSettle();
    expect(doc.s2['li_m0.mode'], 'Intersect');

    // B: dragging the bar body delays the layer
    final strip = _painted('_LiTimePaint'), to = tester.getTopLeft(strip), tw = tester.getSize(strip).width;
    final bar = to + Offset(8 + (tw - 16) / 6, 41);
    await _drag(tester, bar, bar + const Offset(30, 0));
    expect(doc.get('li_t.delay'), greaterThan(.3));

    // C: a wire from Null 1 · Position X to Position Y links them
    await _drag(tester, tester.getCenter(find.byKey(const ValueKey('li_src2'))), tester.getCenter(find.byKey(const ValueKey('li_tgt2'))));
    expect(doc.s2['li_l.2.src'], '2');
    expect(doc.s2['li_l.sel'], '2');

    // D: clicking the picture steps the matte mode; clicking Card makes it the matte
    await tester.tap(find.byKey(const ValueKey('li_matte_preview')));
    await tester.pumpAndSettle();
    expect(doc.s2['li_d.mode'], 'Inverted Luma');
    await tester.tap(find.text('Card'));
    await tester.pumpAndSettle();
    expect(doc.s2['li_d.src'], 'Card');
    expect(tester.takeException(), isNull);
  });
}
