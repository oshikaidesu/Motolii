import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

void main() {
  Future<Doc> pump(WidgetTester tester) async {
    tester.view.physicalSize = const Size(600, 2400);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final doc = Doc(Kind.shape);
    await tester.pumpWidget(
      WidgetsApp(
        color: const Color(0xFF000000),
        builder: (c, _) => ListenableBuilder(
          listenable: doc,
          builder: (c, _) => Ctx(
            cfg: const Cfg(),
            doc: doc,
            child: Pop(
              child: Overlay.wrap(
                child: const SingleChildScrollView(
                  child: SizedBox(width: 372, child: Column(children: [TextAnimatorCard(), TextMorphCard(), ParagraphCard()])),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    return doc;
  }

  testWidgets('text cards build, open their folds, and every interaction writes Doc', (tester) async {
    final doc = await pump(tester);
    expect(tester.takeException(), isNull);
    expect(doc.get('ta_end'), 40);
    expect(doc.s2['ta_shape'], 'Ramp up');

    // scrub Start sideways
    final start = find.byWidgetPredicate((w) => w is NumField && w.id == 'ta_start');
    await tester.drag(start, const Offset(80, 0));
    await tester.pumpAndSettle();
    expect(doc.get('ta_start'), isNot(0));

    // drag between the brackets of the preview strip moves Offset
    final strip = find.byKey(const ValueKey('ta_strip'));
    final box = tester.getRect(strip);
    doc.set('ta_start', 0);
    await tester.pump();
    final g = await tester.startGesture(Offset(box.left + 10 + (box.width - 20) * .2, box.center.dy));
    await g.moveBy(const Offset(30, 0));
    await g.moveBy(const Offset(30, 0));
    await g.up();
    await tester.pumpAndSettle();
    expect(doc.get('ta_offset'), greaterThan(0));

    // Based on: Segmented
    await tester.tap(find.text('WORDS'));
    await tester.pumpAndSettle();
    expect(doc.get('ta_based'), 2);

    // Advanced fold reveals Units; Index swaps the range blocks
    await tester.tap(find.text('Advanced'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('INDEX'));
    await tester.pumpAndSettle();
    expect(doc.get('ta_units'), 1);
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'ta_starti'), findsOneWidget);
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'ta_start'), findsNothing);

    // the seed is always reachable in Advanced, Randomize or not
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'ta_seed'), findsOneWidget);
    await tester.tap(find.text('Randomize'));
    await tester.pumpAndSettle();
    expect(doc.get('ta_rand'), 1);
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'ta_seed'), findsOneWidget);

    // paragraph: justify glyphs, More options, fold
    await tester.tap(find.byKey(const ValueKey('para_justify_2')));
    await tester.pumpAndSettle();
    expect(doc.get('para_justify'), 2);
    await tester.tap(find.text('More options'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('TRIM START'));
    await tester.pumpAndSettle();
    expect(doc.get('para_trim'), 3);
    await tester.tap(find.text('Paragraph'));
    await tester.pumpAndSettle();
    expect(doc.b['fold.Paragraph'], isTrue);
    expect(find.text('Leading auto'), findsOneWidget);

    // morph: Amount is off until a target exists
    expect(find.textContaining('Pick a text layer'), findsOneWidget);
    doc.str('tm_target', 'Caption');
    await tester.pumpAndSettle();
    expect(find.textContaining('Pick a text layer'), findsNothing);
    await tester.drag(find.byWidgetPredicate((w) => w is NumField && w.id == 'tm_amount'), const Offset(80, 0));
    await tester.pumpAndSettle();
    expect(doc.get('tm_amount'), isNot(0));

    // folding the animator shows its brief
    await tester.tap(find.text('Text Animator'));
    await tester.pumpAndSettle();
    expect(find.text('Words'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
