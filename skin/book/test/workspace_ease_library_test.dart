// The Ease desk: its menu, its arrows, its handles past the top, and ways laid on one another.
import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_state.dart' show easeOvershoot;
import 'package:lab_book/workspace/desk_ease_cards.dart' show EaseCards;
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_handles.dart';

import 'door.dart';

Finder _in(Finder f) => find.descendant(of: find.byType(DeskEase), matching: f);

void main() {
  Future<void> menu(WidgetTester t, String word) async {
    await t.tap(find.byKey(const ValueKey('card-more:Ease')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(ValueKey('card-$word:Ease')));
    await t.pumpAndSettle();
  }

  testWidgets('a curve copied in the Ease desk, then pasted to every key of the layer (Alight Motion: Copy, Paste to All Keyframes)', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.key(cube, 'x', 2, 0) && m.key(cube, 'y', 1, 0) && m.key(cube, 'y', 2, 1), isTrue, reason: m.trouble);
    expect(m.way(cube, 'x', 0, 'Bounce'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('card-more:Ease')));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('card-all:Ease')), findsNothing, reason: 'nothing copied yet');
    await t.tap(find.byKey(const ValueKey('card-copy:Ease')));
    await t.pumpAndSettle();
    await menu(t, 'all');
    await settled(t, ws);
    for (final (n, at) in [('x', 1.0), ('y', 1.0)]) {
      expect(m.wayOf(cube, n, at).split(' ').first, 'Bounce', reason: 'the stretch of $n after $at s goes by it too');
    }
    expect(m.wayOf(cube, 'x', 0).split(' ').first, 'Bounce');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  });

  testWidgets('the Ease desk\'s arrows go to the curve before and after (Alight Motion\'s previous and next curve)', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.key(cube, 'x', 2, 0), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('ease-previous')), findsNothing, reason: 'none before the first');
    await t.tap(find.byKey(const ValueKey('ease-next')));
    await t.pumpAndSettle();
    expect(ws.keys, {(layer: '$cube', frame: Ws.fps)}, reason: 'the next curve: from its key at 1 s');
    expect(find.byKey(const ValueKey('ease-next')), findsNothing, reason: 'none after the last');
    await t.tap(find.byKey(const ValueKey('ease-previous')));
    await t.pumpAndSettle();
    expect(ws.keys, {(layer: '$cube', frame: 0)}, reason: 'back to the first');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  });

  testWidgets('a handle of the Ease graph is pulled past the top: the curve goes beyond the next key\'s value and comes back (Alight Motion\'s overshoot)', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.way(cube, 'x', 0, 'Bezier 0.42 0 0.58 1'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    await menu(t, 'overshoot');
    addTearDown(() => easeOvershoot = false);
    final graph = find.byType(EaseGraph), r = t.getRect(graph), p2 = t.widget<EaseGraph>(graph).ease.p2;
    final g = easeBox(r.size).shift(r.topLeft);   // (AbsentMotion's geoOf)
    const lo = -2.0, hi = 3.0;
    Offset px(Offset v) => Offset(g.left + v.dx * g.width, g.bottom - (v.dy - lo) / (hi - lo) * g.height);
    await t.timedDragFrom(px(p2), px(Offset(p2.dx, 1.4)) - px(p2), const Duration(milliseconds: 300));
    await settled(t, ws);
    final way = m.wayOf(cube, 'x', 0).split(' ');
    expect(double.parse(way[4]), greaterThan(1.2), reason: 'its second handle above the top: $way');
    expect((await m.shownAt([(cube, 'x', .8)])).single, greaterThan(1), reason: 'past where it goes, before it comes back');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  });

  testWidgets('a press puts a way in place, a tile laid on the graph multiplies, a card carried out comes off: each one step, seen while held, kept with the document', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    expect(m.way(cube, 'x', 0, 'Bezier 0.42 0 1 1'), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final alone = m.wayOf(cube, 'x', 0);
    expect(alone, startsWith('Bezier 0.42'), reason: m.trouble);
    final graph = _in(find.byType(EaseGraph));
    Future<void> carry(Finder from, Offset to, {Future<void> Function()? held}) async {
      final hand = await t.startGesture(t.getCenter(from), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(0, -24));
      await t.pump();
      await hand.moveTo(to);
      await t.pump();
      await held?.call();
      await hand.up();
      await settled(t, ws);
    }

    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph), held: () async {
      expect(t.widget<EaseGraph>(graph).onion?.name, contains('×'), reason: 'what it would make, seen while held');
      expect(m.wayOf(cube, 'x', 0), alone, reason: 'nothing changed until it is let go');
    });
    final laid = m.wayOf(cube, 'x', 0);
    expect(laid, allOf(contains('×'), contains('Bounce')), reason: m.trouble);
    expect((await m.shownAt([(cube, 'x', .4)])).single, closeTo(m.wayExact('Bounce', m.wayExact(alone, .4)), .001), reason: 'the line bends the time of what is laid on it');
    expect(await m.shownAt([(cube, 'x', 0), (cube, 'x', 1)]), [0, 1], reason: 'on its keys at both ends');
    expect(_in(find.byKey(const ValueKey('ease.card.1'))), findsOneWidget, reason: 'each a card');
    expect(m.wayNames, contains(laid), reason: 'a way of the document');
    expect(_in(find.byKey(const ValueKey('ease.slot'))), findsOneWidget, reason: 'and room for the next');
    expect(m.undo() && m.wayOf(cube, 'x', 0) == alone, isTrue, reason: 'one step');
    expect(m.redo() && m.wayOf(cube, 'x', 0) == laid, isTrue);
    ws.reread();
    await t.pumpAndSettle();
    final out = t.getCenter(find.byKey(const ValueKey('ease.Linear')));
    await carry(_in(find.byKey(const ValueKey('ease.card.1'))), t.getCenter(graph));
    expect(m.wayOf(cube, 'x', 0), laid, reason: 'let go on the graph: it stays');
    await carry(_in(find.byKey(const ValueKey('ease.card.1'))), t.getCenter(_in(find.byKey(const ValueKey('ease.card.0')))));
    expect(m.wayOf(cube, 'x', 0), startsWith('Bounce'), reason: 'carried up the flow: it takes that place');
    await carry(_in(find.byKey(const ValueKey('ease.card.0'))), out);
    expect(m.wayOf(cube, 'x', 0), alone, reason: 'carried out: it comes off, the one left alone');
    expect(_in(find.byKey(const ValueKey('ease.card.0'))), findsNothing, reason: 'one way: its numbers again');
    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph));
    expect(m.wayOf(cube, 'x', 0), contains('×'), reason: m.trouble);
    expect(_in(find.byKey(const ValueKey('ease.whole'))), findsOneWidget, reason: 'what they make, at the foot of the flow');
    expect(t.widget<EaseGraph>(graph).onion?.name, contains('×'), reason: 'and faint behind the card in hand');
    await t.tap(_in(find.byKey(const ValueKey('ease.card.0'))));
    await t.pumpAndSettle();
    expect(t.widget<EaseGraph>(graph).ease.p1, const Offset(.42, 0), reason: 'a card pressed: the graph shows it, its handles in hand');
    await t.tap(_in(find.byKey(const ValueKey('ease.Elastic'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Elastic'), reason: 'a press: in place of the card pressed, and the other stays');
    expect(m.wayOf(cube, 'x', 0), contains('Bounce'), reason: m.trouble);
    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph));
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a card of a stack is pulled by its handles, the document keeps one way for it; a shape goes in between; any way turns end for end', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    final graph = _in(find.byType(EaseGraph));
    Future<void> carry(Finder from, Offset to) async {
      final hand = await t.startGesture(t.getCenter(from), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(0, -24));
      await t.pump();
      await hand.moveTo(to);
      await t.pump();
      await hand.up();
      await settled(t, ws);
    }

    await t.tap(_in(find.byKey(const ValueKey('ease.Cyclic'))));
    await settled(t, ws);
    await carry(_in(find.byKey(const ValueKey('ease.Linear'))), t.getCenter(graph));
    expect(m.wayOf(cube, 'x', 0), allOf(startsWith('Cyclic'), endsWith('Linear')), reason: m.trouble);
    expect(t.widget<EaseGraph>(graph).ease.name, 'Linear', reason: 'the one just laid in hand');
    final r = t.getRect(graph), g = easeBox(r.size).shift(r.topLeft);
    Offset px(double x, double y) => Offset(g.left + x * g.width, g.bottom - (y + .25) / 1.5 * g.height);
    List<double> whole() => t.widget<EaseCards>(_in(find.byType(EaseCards))).whole!.drawn!;
    final was = whole();
    final hand = await t.startGesture(px(.25, .25), kind: PointerDeviceKind.mouse);
    for (var i = 1; i <= 12; i++) {
      await hand.moveBy((px(.25, .9) - px(.25, .25)) / 12);
      await t.pump();
    }
    expect(whole(), isNot(was), reason: 'what they make, at the flow\'s foot, while the hand is on it');
    expect(t.widget<EaseGraph>(graph).onion?.drawn, whole(), reason: 'and the skin behind the graph');
    await hand.up();
    await settled(t, ws);
    final pulled = m.wayOf(cube, 'x', 0);
    expect(pulled, allOf(startsWith('Cyclic'), contains('×Bezier')), reason: 'its handle, on its stretch: ${m.trouble}');
    expect((await m.shownAt([(cube, 'x', .5)])).single, greaterThan(.3), reason: 'and the stage goes by it');
    expect([for (final n in m.wayNames) if (n.contains('×')) n], [pulled], reason: 'and the document keeps only the way it goes by');
    expect(m.undo() && m.wayOf(cube, 'x', 0).endsWith('Linear'), isTrue, reason: 'all one step: ${m.wayOf(cube, 'x', 0)}');
    expect(m.redo(), isTrue);
    ws.reread();
    await t.pumpAndSettle();
    await carry(_in(find.byKey(const ValueKey('ease.Elastic'))), t.getCenter(_in(find.byKey(const ValueKey('ease.card.1')))));
    expect(m.wayOf(cube, 'x', 0).split('×').map((w) => w.split('~').first), ['Cyclic', 'Elastic', 'Bezier'], reason: 'let go on a card it goes in before it');
    await hand.moveTo(t.getCenter(_in(find.byKey(const ValueKey('ease.card.1')))));   // (the hand over it)
    await t.pumpAndSettle();
    await t.tap(_in(find.byKey(const ValueKey('ease.off.1'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0).split('×').map((w) => w.split('~').first), ['Cyclic', 'Bezier'], reason: 'its ×: that card off, the others stay');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await t.pumpAndSettle();
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a wave turned end for end, and back (AbsentMotion\'s reverse)', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    await t.tap(_in(find.byKey(const ValueKey('ease.Bounce'))));
    await settled(t, ws);
    await t.tap(_in(find.byKey(const ValueKey('ease-reverse'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Reverse:Bounce'), reason: m.trouble);
    expect((await m.shownAt([(cube, 'x', .3)])).single, closeTo(1 - m.wayAt('Bounce', .7), .02), reason: 'its curve turned');
    expect(easeHandles(t.widget<EaseGraph>(_in(find.byType(EaseGraph))).ease), isNotEmpty, reason: 'its handles with it');
    await t.tap(_in(find.byKey(const ValueKey('ease-reverse'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Bounce'), reason: 'turned back');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('each card acts as its intent asks: Linear changes nothing, a line times what goes through it, steps cut it, a wave is as large as it, a shake is added', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    final graph = _in(find.byType(EaseGraph));
    Future<double> laid(String first, String on) async {
      expect(m.way(cube, 'x', 0, first), isTrue, reason: m.trouble);
      ws.reread();
      await t.pumpAndSettle();
      final hand = await t.startGesture(t.getCenter(_in(find.byKey(ValueKey('ease.$on')))), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(0, -24));
      await t.pump();
      await hand.moveTo(t.getCenter(graph));
      await t.pump();
      await hand.up();
      await settled(t, ws);
      expect(m.wayOf(cube, 'x', 0), contains('×'), reason: m.trouble);
      return (await m.shownAt([(cube, 'x', .4)])).single!;
    }

    const out = 'Bezier 0 0 0.58 1';
    final line = m.wayExact(out, .4);
    expect(await laid('Cyclic', 'Linear'), closeTo(m.wayExact('Cyclic', .4), .001), reason: 'Linear: nothing');
    expect(await laid(out, 'Steps'), closeTo(m.wayExact('Steps', line), .001), reason: 'steps cut what goes through them');
    expect(await laid(out, 'Elastic'), closeTo(m.wayExact('Elastic', line), .001), reason: 'and a bounce is timed by the line');
    expect(await laid(out, 'Cyclic'), closeTo(line * m.wayExact('Cyclic', .4), .001), reason: 'a wave as large as the line');
    expect(await laid(out, 'Random'), closeTo(line + m.wayExact('Random', .4) - .4, .001), reason: 'a shake added to it');
    expect(await m.shownAt([(cube, 'x', 0), (cube, 'x', 1)]), [0, 1], reason: 'on its keys at both ends, each');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
