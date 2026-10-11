import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

Future<Doc> mount(WidgetTester tester) async {
  tester.view.physicalSize = const Size(800, 3200);
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
              child: SingleChildScrollView(
                child: Align(
                  alignment: Alignment.topLeft,
                  child: SizedBox(width: 372, child: PathIdeas()),
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

Future<void> drag(WidgetTester tester, String handle, Offset by) async {
  final g = await tester.startGesture(tester.getCenter(find.byKey(ValueKey('h:$handle'))));
  for (var i = 1; i <= 4; i++) {
    await g.moveBy(by / 4);
    await tester.pump();
  }
  await g.up();
  await tester.pump();
}

List<String> order(Doc doc, String lane) => doc.s2['pi_${lane}_order']!.split(',');

Future<void> tapKey(WidgetTester tester, String key) async {
  await tester.tap(find.byKey(ValueKey(key)));
  await tester.pump();
}

/// Opens an instance's More and picks [item].
Future<void> more(WidgetTester tester, String lane, String inst, String item) async {
  await tapKey(tester, 'pi_${lane}_$inst.more');
  await tester.tap(find.text(item));
  await tester.pump();
}

void main() {
  testWidgets('A: the seeded stack repeats a kind; trim ends, corner radius and shape are edited on the stage', (tester) async {
    final doc = await mount(tester);
    expect(tester.takeException(), isNull);
    expect(order(doc, 'a').where((s) => s.startsWith('round')).length, 2, reason: 'repetition is the normal case');
    expect(find.text('Rounded Corners 2'), findsOneWidget);

    expect(doc.get('pi_a_trim5_end'), 72);
    final undo = doc.undoSteps;
    await drag(tester, 'pi_a_trim5_end', const Offset(-24, -30));
    expect(doc.get('pi_a_trim5_end'), isNot(72));
    expect(doc.undoSteps, undo + 1, reason: 'one drag, one undo');

    await tester.tap(find.text('Rounded Corners').first);
    await tester.pump();
    expect(doc.s2['pi_a_sel'], 'round1');
    final r = doc.get('pi_a_round1_radius')!;
    await drag(tester, 'pi_a_round1_radius', const Offset(4, 10));
    expect(doc.get('pi_a_round1_radius'), greaterThan(r));
    expect(doc.get('pi_a_round3_radius'), 5, reason: 'the other Rounded Corners keeps its own radius');

    await tester.tap(find.text('RECT'));
    await tester.pump();
    expect(doc.s2['pi_a_shape'], 'Rect');
    expect(tester.takeException(), isNull);
  });

  testWidgets('A: the same kind added twice gives two independent instances that both shape the path', (tester) async {
    final doc = await mount(tester);
    final before = order(doc, 'a');
    for (var i = 0; i < 2; i++) {
      await tapKey(tester, 'pi_a_add');
      await tapKey(tester, 'pi_a_add_pucker');
    }
    final now = order(doc, 'a'), added = now.skip(before.length).toList();
    expect(now.take(before.length), before);
    expect(added.length, 2);
    expect(added.every((s) => s.startsWith('pucker')), isTrue);
    expect(added.toSet().length, 2, reason: 'each add is a new instance');
    expect(doc.s2['pi_a_sel'], added.last);
    expect(find.text('Pucker & Bloat 3'), findsOneWidget);

    final (p1, p2) = (added[0], added[1]);
    doc.setMany({'pi_a_${p1}_amount': 0, 'pi_a_${p2}_amount': 0});
    final o0 = pathIdeaOutline(doc, 'a');
    doc.set('pi_a_${p1}_amount', 40);
    final o1 = pathIdeaOutline(doc, 'a');
    expect(o1, isNot(equals(o0)));
    doc.set('pi_a_${p2}_amount', -30);
    final o2 = pathIdeaOutline(doc, 'a');
    expect(o2, isNot(equals(o1)));
    expect(doc.get('pi_a_${p1}_amount'), 40, reason: 'editing one instance leaves the other alone');
    doc.flag('pi_a_$p1.on', false);
    expect(pathIdeaOutline(doc, 'a'), isNot(equals(o2)));
    await tester.pump();
    expect(tester.takeException(), isNull);
  });

  testWidgets('A: duplicate copies values into a new instance; move and remove reorder the stack', (tester) async {
    final doc = await mount(tester);
    await more(tester, 'a', 'round3', 'Duplicate');
    var o = order(doc, 'a');
    final copy = o[o.indexOf('round3') + 1];
    expect(copy, isNot('round3'));
    expect(copy, startsWith('round'));
    expect(doc.get('pi_a_${copy}_radius'), doc.get('pi_a_round3_radius'));
    expect(doc.s2['pi_a_sel'], copy);
    doc.set('pi_a_${copy}_radius', 12);
    expect(doc.get('pi_a_round3_radius'), 5);

    await more(tester, 'a', copy, 'Move earlier');
    o = order(doc, 'a');
    expect(o.indexOf(copy), o.indexOf('round3') - 1);
    await more(tester, 'a', 'round1', 'Move later');
    expect(order(doc, 'a').take(2), ['pucker2', 'round1']);

    await more(tester, 'a', copy, 'Remove');
    expect(order(doc, 'a'), isNot(contains(copy)));
    expect(find.byKey(ValueKey('pi_a_row_$copy')), findsNothing);
    expect(tester.takeException(), isNull);
  });

  testWidgets('B: one frame per instance; the open card edits zig zag by its peak; duplicates read as "Twist 2"', (tester) async {
    final doc = await mount(tester);
    final amp = doc.get('pi_b_zigzag1_amp')!;
    await drag(tester, 'pi_b_zigzag1_peak', const Offset(0, -12));
    expect(doc.get('pi_b_zigzag1_amp'), greaterThan(amp));

    await tester.tap(find.text('Twist').first);
    await tester.pump();
    expect(doc.b['pi_b_twist4.open'], true);
    final a = doc.get('pi_b_twist4_angle')!;
    await drag(tester, 'pi_b_twist4_angle', const Offset(30, 20));
    expect(doc.get('pi_b_twist4_angle'), greaterThan(a));

    doc.flag('pi_b_offset3.open', true);
    await tester.pump();
    final off = doc.get('pi_b_offset3_amount')!;
    await drag(tester, 'pi_b_offset3_amount', const Offset(16, 0));
    expect(doc.get('pi_b_offset3_amount'), isNot(off));

    await more(tester, 'b', 'twist4', 'Duplicate');
    expect(order(doc, 'b'), ['zigzag1', 'round2', 'offset3', 'twist4', 'twist5']);
    expect(doc.get('pi_b_twist5_angle'), doc.get('pi_b_twist4_angle'));
    expect(doc.b['pi_b_twist5.open'], true);
    expect(find.text('Twist 2'), findsNWidgets(2), reason: 'its frame and its card');

    await tapKey(tester, 'pi_b_add');
    await tapKey(tester, 'pi_b_add_round');
    expect(order(doc, 'b').last, startsWith('round'));
    expect(find.text('Round 2'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('C: keys drag on the timeline, presets rewrite them, the Ease link routes, hovering never edits', (tester) async {
    final doc = await mount(tester);
    final t = doc.get('pi_c_e1t')!;
    await drag(tester, 'pi_c_e1', const Offset(40, 0));
    expect(doc.get('pi_c_e1t'), greaterThan(t));
    expect(doc.s2['pi_c_sel'], 'e1');

    await tester.tap(find.text('Draw off'));
    await tester.pump();
    expect(doc.get('pi_c_s1v'), 100);
    expect(doc.get('pi_c_e0v'), 100);

    final ease = doc.get('pi_c_ease');
    await tapKey(tester, 'link:ease:Ease');
    expect(doc.s2['route'], 'Ease');
    expect(doc.get('pi_c_ease'), ease, reason: 'the Inspector shows the ease; the Ease panel picks it');

    final before = doc.undoSteps, at = tester.getCenter(find.byKey(const ValueKey('h:pi_c_s0'))) + const Offset(60, 0);
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: at);
    await mouse.moveTo(at + const Offset(20, 0));
    await tester.pump();
    expect(find.textContaining('(hover)'), findsOneWidget);
    expect(doc.undoSteps, before);
    await mouse.removePointer();
    expect(tester.takeException(), isNull);
  });

  testWidgets('every card folds, every instance switches off, long and empty stacks fit without overflow', (tester) async {
    final doc = await mount(tester);
    for (final s in order(doc, 'a')) {
      doc.b['pi_a_$s.on'] = !(doc.b['pi_a_$s.on'] ?? true);
    }
    for (final s in order(doc, 'b')) {
      doc.b['pi_b_$s.on'] = false;
      doc.b['pi_b_$s.open'] = true;
    }
    for (final k in ['trim', 'trim', 'wiggle', 'zigzag']) {
      await tapKey(tester, 'pi_b_add');
      await tapKey(tester, 'pi_b_add_$k');
    }
    expect(order(doc, 'b').length, 8);
    for (final s in ['Line', 'Rect']) {
      doc.str('pi_a_shape', s);
      await tester.pump();
      expect(tester.takeException(), isNull);
    }
    doc.str('pi_a_order', '');
    doc.str('pi_b_order', '');
    await tester.pump();
    expect(find.textContaining('No path operations'), findsWidgets);
    expect(tester.takeException(), isNull);
    doc.flag('pi_a.open', false);
    doc.flag('pi_c.open', false);
    await tester.pump();
    expect(tester.takeException(), isNull);
  });
}
