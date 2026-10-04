import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

Future<Doc> mount(WidgetTester tester) async {
  tester.view.physicalSize = const Size(800, 2400);
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
              child: SingleChildScrollView(
                child: SizedBox(width: 372, child: Column(children: const [PathOpsStack(), Text('below')])),
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

List<String> order(Doc d) => d.s2['pop_order']!.split(',');

void main() {
  testWidgets('default stack: three cards that scrub, choose and fold', (tester) async {
    final doc = await mount(tester);
    expect(tester.takeException(), isNull);
    expect(order(doc), ['trim1', 'round2', 'wiggle3']);
    for (final t in ['Trim Paths', 'Rounded Corners', 'Wiggle Paths']) {
      expect(find.text(t), findsOneWidget);
    }

    final radius = find.byWidgetPredicate((w) => w is NumField && w.id == 'pop_round2_radius');
    final g = await tester.startGesture(tester.getCenter(radius));
    await g.moveBy(const Offset(30, 0));
    await g.up();
    await tester.pump(const Duration(milliseconds: 400));
    expect(doc.get('pop_round2_radius'), greaterThan(10));

    await tester.tap(find.text('INDIVIDUALLY'));
    await tester.pump();
    expect(doc.get('pop_trim1_multiple'), 1);

    await tester.tap(find.text('Trim Paths'));
    await tester.pump();
    expect(doc.b['pop_trim1.open'], false);
    expect(find.text('0–100%'), findsOneWidget);
    expect(find.text('Individually'), findsOneWidget, reason: 'folded brief says the mode');

    await tester.tap(find.text('Advanced'));
    await tester.pump();
    expect(find.text('Points'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('add floats a list, a mode hides its field, More removes', (tester) async {
    final doc = await mount(tester);
    final below = find.text('below'), y = tester.getTopLeft(below).dy;
    await tester.tap(find.text('Add path operation'));
    await tester.pump();
    expect(find.text('Oscillator'), findsOneWidget);
    expect(tester.getTopLeft(below).dy, y, reason: 'the list floats');
    await tester.tap(find.text('Offset Paths'));
    await tester.pump();
    expect(order(doc), ['trim1', 'round2', 'wiggle3', 'offset4']);
    expect(find.text('Miter Limit'), findsOneWidget);
    await tester.tap(find.text('ROUND'));
    await tester.pump();
    expect(doc.get('pop_offset4_join'), 1);
    expect(find.text('Miter Limit'), findsNothing);

    await tester.tap(find.text('Add path operation'));
    await tester.pump();
    await tester.tap(find.text('Reverse Path'));
    await tester.pump();
    expect(order(doc).last, 'reverse5');
    expect(find.text('Flips direction'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('pop_reverse5.more')));
    await tester.pump();
    await tester.tap(find.text('Remove'));
    await tester.pump();
    expect(order(doc), ['trim1', 'round2', 'wiggle3', 'offset4']);
    expect(tester.takeException(), isNull);
  });

  testWidgets('every kind opens and folds without overflow', (tester) async {
    final doc = await mount(tester);
    doc.str('pop_order', 'trim1,round2,pucker3,zigzag4,offset5,twist6,wiggle7,smooth8,subdivide9,reverse10,extend11,chop12,resample13,bend14,osc15');
    doc.s2['pop_next'] = '16';
    await tester.pump();
    expect(tester.takeException(), isNull);
    for (final t in ['Bend', 'Oscillator', 'Chop Path', 'Twist', 'Subdivide']) {
      expect(find.text(t), findsOneWidget);
    }
    for (final k in order(doc)) {
      doc.b['pop_$k.open'] = false;
    }
    doc.poke();
    await tester.pump();
    expect(find.byType(ParamGrid), findsNothing);
    expect(tester.takeException(), isNull);
  });

  testWidgets('holding a card head and dragging it reorders the stack', (tester) async {
    final doc = await mount(tester);
    for (final k in ['trim1', 'round2', 'wiggle3']) {
      doc.flag('pop_$k.open', false);
    }
    await tester.pump();
    final from = tester.getCenter(find.text('Wiggle Paths')), to = tester.getTopLeft(find.text('Trim Paths')) + const Offset(4, 2);
    final g = await tester.startGesture(from);
    await tester.pump(const Duration(milliseconds: 300));
    await g.moveTo(Offset(from.dx, (from.dy + to.dy) / 2));
    await tester.pump();
    await g.moveTo(Offset(from.dx, to.dy));
    await tester.pump();
    await g.up();
    await tester.pump();
    expect(order(doc), ['wiggle3', 'trim1', 'round2']);
    expect(tester.takeException(), isNull);
  });
}
