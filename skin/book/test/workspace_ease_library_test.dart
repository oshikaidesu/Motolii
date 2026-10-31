// The Ease desk: its menu, its arrows, its handles past the top, and ways laid on one another.
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/services.dart' show SystemChannels;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_state.dart' show easeOvershoot;
import 'package:lab_book/workspace/desk_ease_cards.dart' show EaseCards, easeDefines, easeFromText, easeText;
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_handles.dart';
import 'package:lab_book/workspace/desk_ease_shelf.dart';
import 'package:lab_book/sets/inspector/inspector_rows.dart' show FoldMark;
import 'package:lab_book/workspace/desk_ease_numbers.dart';
import 'package:lab_book/workspace/desk_ease_side.dart';
import 'package:lab_book/workspace/desk_kit.dart' show DeskScrub;
import 'package:lab_book/workspace/keys.dart' show WsKeys;
import 'package:lab_book/workspace/timeline.dart' show WsTimeline;

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
    final out = t.getTopLeft(find.byType(DeskEase)) - const Offset(40, 40);
    await carry(_in(find.byKey(const ValueKey('ease.card.1'))), t.getCenter(graph));
    expect(m.wayOf(cube, 'x', 0), laid, reason: 'let go on the graph: it stays');
    await carry(_in(find.byKey(const ValueKey('ease.card.1'))), t.getCenter(_in(find.byKey(const ValueKey('ease.card.0')))));
    expect(m.wayOf(cube, 'x', 0), startsWith('Bounce'), reason: 'carried up the flow: it takes that place');
    await carry(_in(find.byKey(const ValueKey('ease.card.0'))), out);
    expect(m.wayOf(cube, 'x', 0), alone, reason: 'carried out: it comes off, the one left alone');
    expect(_in(find.byKey(const ValueKey('ease.whole'))), findsNothing, reason: 'one way: its numbers again');
    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph));
    expect(m.wayOf(cube, 'x', 0), contains('×'), reason: m.trouble);
    expect(_in(find.byKey(const ValueKey('ease.whole'))), findsOneWidget, reason: 'what they make, at the foot of the flow');
    expect(t.widget<EaseGraph>(graph).onion?.name, contains('×'), reason: 'and faint behind the card in hand');
    await t.tap(_in(find.byKey(const ValueKey('ease.whole'))));
    await t.pumpAndSettle();
    expect(t.widget<EaseGraph>(graph).ease.name, contains('×'), reason: 'that at the foot pressed: the graph shows the whole');
    expect(t.widget<EaseGraph>(graph).onion, isNull, reason: 'no skin behind it');
    expect(easeHandles(t.widget<EaseGraph>(graph).ease), isEmpty, reason: 'the whole has no handles');
    await t.tap(_in(find.byKey(const ValueKey('ease.card.0'))));
    await t.pumpAndSettle();
    expect(t.widget<EaseGraph>(graph).ease.p1, const Offset(.42, 0), reason: 'a card pressed: the graph shows it, its handles in hand');
    await t.tap(_in(find.byKey(const ValueKey('ease.Elastic'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Elastic'), reason: 'a press: in place of the card pressed, and the other stays');
    expect(m.wayOf(cube, 'x', 0), contains('Bounce'), reason: m.trouble);
    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph));
    await t.tap(_in(find.byKey(const ValueKey('ease.card.0'))));   // (which card is in hand is the panel's, not the document's: the first, as it opens)
    await t.pumpAndSettle();
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
    await t.tap(_in(find.byKey(const ValueKey('ease.card.0'))));   // (which card is in hand is the panel's, not the document's: the first, as it opens)
    await t.pumpAndSettle();
    await expectSameAfterReopen(t);
    await t.pumpAndSettle();
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Alight Motion\'s four beziers; the shelf drawn out and put away; a curve or what the cards make kept on it, a press brings it back whole, its × takes it off', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    final file = File('${Directory.systemTemp.createTempSync('kept').path}/ease_kept.json');
    easeKeptAt(file);
    final graph = _in(find.byType(EaseGraph)), column = t.getCenter(_in(find.byType(EaseShapes)));
    Future<void> carry(Finder from, Offset to) async {
      final hand = await t.startGesture(t.getCenter(from), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(-14, 0));
      await t.pump();
      await hand.moveTo(to);
      await t.pump();
      await hand.up();
      await settled(t, ws);
      await hand.removePointer();
    }

    for (final (shape, word) in [('In', 'Bezier 0.42 0 1 1'), ('Out', 'Bezier 0 0 0.58 1'), ('In-out', 'Bezier 0.42 0 0.58 1')]) {
      await t.tap(_in(find.byKey(ValueKey('ease.$shape'))));
      await settled(t, ws);
      expect(m.wayOf(cube, 'x', 0), word, reason: m.trouble);
    }
    expect(find.byType(EaseShelf), findsNothing, reason: 'put away, it is not there');
    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph));
    final laid = m.wayOf(cube, 'x', 0);
    expect(laid, contains('×'), reason: m.trouble);
    await carry(_in(find.byKey(const ValueKey('ease.whole'))), column);
    expect(find.byType(EaseShelf), findsOneWidget, reason: 'a card carried to the shapes opens it');
    expect(easeKept(), [laid], reason: 'what the cards make, kept as one');
    expect(m.wayOf(cube, 'x', 0), laid, reason: 'and still on the stretch');
    expect(m.way(cube, 'x', 0, 'Bezier 0.2 0 0 1'), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    await t.tap(_in(find.byKey(const ValueKey('ease.shelf.slot'))));
    await t.pumpAndSettle();
    expect(easeKept().last, 'Bezier 0.2 0.0 0.0 1.0', reason: 'the empty frame pressed: what is on the stretch');
    easeKeptAt(file);
    expect(easeKept(), hasLength(2), reason: 'and still there when the window opens again');
    expect(find.byType(EaseCards), findsOneWidget);
    await t.tap(_in(find.byKey(ValueKey('ease.kept.$laid'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), laid, reason: 'a press brings it back, ${m.trouble}');
    expect(find.byKey(const ValueKey('ease.card.1')), findsOneWidget, reason: 'each its own card again');
    await t.tap(_in(find.byKey(const ValueKey('ease.kept.Bezier 0.2 0.0 0.0 1.0'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), 'Bezier 0.2 0 0 1', reason: 'a press on another: in place of all of it, ${m.trouble}');
    await t.tap(_in(find.byKey(ValueKey('ease.kept.$laid'))));
    await settled(t, ws);
    await t.tap(_in(find.byKey(ValueKey('ease.kept.$laid'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), laid, reason: 'pressed again: as it was, no more cards');
    final hand = await t.createGesture(kind: PointerDeviceKind.mouse);
    await hand.addPointer(location: t.getCenter(_in(find.byKey(ValueKey('ease.kept.$laid')))));
    await t.pump();
    await hand.moveBy(const Offset(1, 0));
    await t.pump();
    await hand.moveTo(t.getCenter(find.byKey(ValueKey('ease.forget.$laid'))));
    await t.pump();
    await hand.down(t.getCenter(find.byKey(ValueKey('ease.forget.$laid'))));
    await hand.up();
    await t.pump();
    await hand.removePointer();
    expect(easeKept(), ['Bezier 0.2 0.0 0.0 1.0'], reason: 'its × takes it off');
    expect(m.wayOf(cube, 'x', 0), laid, reason: 'not off the stretch');
    await t.tap(_in(find.byKey(const ValueKey('ease.shelf'))));
    await t.pumpAndSettle();
    expect(find.byType(EaseShelf), findsNothing, reason: 'put away');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the shelf holds fifty: what is kept scrolls, none of it lost', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    for (var i = 0; i < 50; i++) {
      easeKeep('Bounce~${(.2 + i / 100).toStringAsFixed(2)}~0.8×Elastic');
    }
    await t.tap(_in(find.byKey(const ValueKey('ease.shelf'))));
    await t.pumpAndSettle();
    final last = find.byKey(ValueKey('ease.kept.${easeKept().last}')), row = find.descendant(of: find.byType(EaseShelf), matching: find.byType(Scrollable));
    expect(last, findsNothing, reason: 'the fiftieth is further down');
    final wheel = t.state<ScrollableState>(row).position;   // (a hand held on a tile carries it)
    wheel.jumpTo(wheel.maxScrollExtent);
    await t.pump();
    expect(last.hitTestable(), findsOneWidget);
    await t.tap(last);
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), easeKept().last, reason: m.trouble);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the shelf out, in a small window and a tall one, pushes the rest aside: nothing cut, the flow still in reach', (t) async {
    for (final size in [const Size(1100, 700), tallWindow]) {
      final ws = await openWorkspace(t, size: size), m = ws.inside!;
      final cube = m.add('solid', 'cube');
      expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
      ws
        ..reread()
        ..pickKey((layer: '$cube', frame: 0))
        ..front('Ease');
      await t.pumpAndSettle();
      final before = t.getRect(_in(find.byType(EaseGraph)));
      await t.tap(_in(find.byKey(const ValueKey('ease.shelf'))));
      await t.pumpAndSettle();
      expect(t.getRect(_in(find.byType(EaseGraph))).width, lessThan(before.width), reason: 'the graph gives it room');
      expect(_in(find.byKey(const ValueKey('ease.slot'))).hitTestable(), findsOneWidget, reason: 'the flow not under it');
      expect(_in(find.byKey(const ValueKey('ease.shelf.slot'))).hitTestable(), findsOneWidget);
      expect(t.takeException(), isNull);
      await closeWorkspace(t);
    }
  }, skip: !insideBuilt);

  testWidgets('a tile held over the graph is tried on the stage; carried away again nothing was made, not even a step', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.way(cube, 'x', 0, 'Bezier 0.42 0 1 1'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    final alone = m.wayOf(cube, 'x', 0), before = (await m.shownAt([(cube, 'x', .4)])).single!;
    final hand = await t.startGesture(t.getCenter(_in(find.byKey(const ValueKey('ease.Bounce')))), kind: PointerDeviceKind.mouse);
    await hand.moveBy(const Offset(-14, 0));
    await t.pump();
    await hand.moveTo(t.getCenter(_in(find.byType(EaseGraph))));
    await t.pump();
    await settled(t, ws);
    expect((await m.shownAt([(cube, 'x', .4)])).single, closeTo(m.wayExact('Bounce', m.wayExact(alone, .4)), .001), reason: 'the stage goes by what it would make');
    await hand.moveTo(t.getTopLeft(find.byType(DeskEase)) - const Offset(40, 40));
    await t.pump();
    await settled(t, ws);
    expect((await m.shownAt([(cube, 'x', .4)])).single, closeTo(before, .001), reason: 'carried away: as it was');
    await hand.up();
    await settled(t, ws);
    await hand.removePointer();
    expect(m.wayOf(cube, 'x', 0), alone, reason: m.trouble);
    expect((await m.shownAt([(cube, 'x', .4)])).single, closeTo(before, .001));
    expect(m.undo(), isTrue, reason: m.trouble);
    expect(m.wayOf(cube, 'x', 0), isNot(alone), reason: 'one undo goes to what was before it: no step of its own');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the whole in hand: a press on a shape takes its place, not a card\'s', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.way(cube, 'x', 0, 'Bounce'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    final hand = await t.startGesture(t.getCenter(_in(find.byKey(const ValueKey('ease.Elastic')))), kind: PointerDeviceKind.mouse);
    await hand.moveBy(const Offset(-14, 0));
    await t.pump();
    await hand.moveTo(t.getCenter(_in(find.byType(EaseGraph))));
    await t.pump();
    await hand.up();
    await settled(t, ws);
    await hand.removePointer();
    expect(m.wayOf(cube, 'x', 0), contains('×'), reason: m.trouble);
    await t.tap(_in(find.byKey(const ValueKey('ease.whole'))));
    await t.pumpAndSettle();
    await t.tap(_in(find.byKey(const ValueKey('ease.Cyclic'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Cyclic'), reason: m.trouble);
    expect(m.wayOf(cube, 'x', 0), isNot(contains('×')));
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  test('a curve from a page: cubic-bezier() and linear(), wherever they stand; what is no curve is none', () {
    final b = easeFromText('transition: left .3s cubic-bezier(.17, .67, .83, -.2) .1s;', null)!;
    expect((b.p1, b.p2), (const Offset(.17, .67), const Offset(.83, -.2)));
    expect(easeFromText('cubic-bezier(1.2, 0, .5, 1)', null), isNull, reason: 'time goes only forward (CSS Easing 2)');
    expect(easeFromText('cubic-bezier(0, 1)', null), isNull);
    expect(said(easeFromText('animation-timing-function: linear(0, 0.8 25%, 1);', null)!), 'linear(0,0.8~25%,1)');
    expect(easeDefines('linear(0,0.8~25%,1)').single.$2, contains('''anime('linear(0,"0.8 25%",1)', u)'''));
    expect(easeFromText('hello', null), isNull);
    expect(easeFromText(null, null), isNull);
    expect(easeText(easeFromText(easeText(const Ease(EaseKind.bezier, Offset(.2, 0), Offset(0, 1))), null)!), 'cubic-bezier(0.2, 0.0, 0.0, 1.0)');
  });

  testWidgets('Paste Curve takes what a page gave; Copy Curve says it so that a page can take it', (t) async {
    String? board;
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.platform, (c) async {
      if (c.method == 'Clipboard.setData') board = (c.arguments as Map)['text'] as String?;
      return c.method == 'Clipboard.getData' ? {'text': board} : null;
    });
    addTearDown(() => t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.platform, null));
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    board = 'ease: cubic-bezier(.2, 0, 0, 1)';
    await menu(t, 'paste');
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), 'Bezier 0.2 0 0 1', reason: m.trouble);
    board = null;
    await menu(t, 'copy');
    expect(board, 'cubic-bezier(0.2, 0.0, 0.0, 1.0)');
    board = 'linear(0, 0.8 25%, 1)';
    await menu(t, 'paste');
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), 'linear(0,0.8~25%,1)', reason: m.trouble);
    expect((await m.shownAt([(cube, 'x', .25)])).single, closeTo(.8, .001), reason: 'the stage goes by its points');
    await t.pumpAndSettle();
    expect(t.widget<EaseGraph>(_in(find.byType(EaseGraph))).ease.at(.25), closeTo(.8, .02), reason: 'and the graph draws them');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('in a tall pane the cards are laid across, and a card is laid, moved and taken off as in a wide one', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.way(cube, 'x', 0, 'Bounce'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    final tall = OverlayEntry(builder: (_) => const Positioned(left: 0, top: 0, width: 283, height: 560, child: ColoredBox(color: Color(0xFF000000), child: DeskEase())));   // (a pane so shaped, as a dock's splits make it)
    Overlay.of(t.element(find.byType(WsTimeline))).insert(tall);
    addTearDown(tall.remove);
    await t.pumpAndSettle();
    Finder _in(Finder f) => find.descendant(of: find.byWidgetPredicate((w) => w is Positioned && w.width == 283), matching: f);
    expect(t.widget<EaseCards>(_in(find.byType(EaseCards))).across, isTrue);
    Future<void> carry(Finder from, Offset to) async {
      final hand = await t.startGesture(t.getCenter(from), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(0, -24));
      await t.pump();
      await hand.moveTo(to);
      await t.pump();
      await hand.up();
      await settled(t, ws);
      await hand.removePointer();
    }
    await carry(_in(find.byKey(const ValueKey('ease.Elastic'))), t.getCenter(_in(find.byType(EaseGraph))));
    expect(m.wayOf(cube, 'x', 0), allOf(startsWith('Bounce'), contains('Elastic')), reason: m.trouble);
    await carry(_in(find.byKey(const ValueKey('ease.card.1'))), t.getCenter(_in(find.byKey(const ValueKey('ease.card.0')))));
    expect(m.wayOf(cube, 'x', 0), startsWith('Elastic'), reason: 'carried along the flow: it takes that place');
    await carry(_in(find.byKey(const ValueKey('ease.card.0'))), const Offset(700, 300));
    expect(m.wayOf(cube, 'x', 0), startsWith('Bounce'), reason: m.trouble);
    expect(m.wayOf(cube, 'x', 0), isNot(contains('×')), reason: 'carried out: it comes off');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a way\'s numbers: its name pressed opens them, a number written goes on the stretch, waves said by how many, a card of a stack and a turned one too', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    Future<void> write(String number, String to) async {
      final field = _in(find.byKey(ValueKey('ease-number-$number')));
      await t.tap(field);
      await t.pump();
      await t.enterText(find.descendant(of: field, matching: find.byType(EditableText)), to);
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump();
      await settled(t, ws);
      await t.pumpAndSettle();
    }

    addTearDown(() => easeNumbersOpen = false);
    expect(_in(find.byKey(const ValueKey('ease-number-X1'))), findsNothing, reason: 'nothing shown unasked');
    expect(find.descendant(of: _in(find.byKey(const ValueKey('ease-numbers'))), matching: find.byType(FoldMark)), findsOneWidget, reason: 'and the sign that there is more, the Inspector\'s');
    await t.tap(_in(find.byKey(const ValueKey('ease-numbers'))));
    await t.pumpAndSettle();
    await write('X1', '0.3');
    await write('Y2', '1.5');
    expect(m.wayOf(cube, 'x', 0), 'Bezier 0.3 0 1 1.5', reason: m.trouble);
    await t.tap(_in(find.byKey(const ValueKey('ease.Cyclic'))));
    await settled(t, ws);
    await t.pumpAndSettle();
    await write('Waves', '8');
    expect(m.wayOf(cube, 'x', 0), startsWith('Cyclic 0.125 '), reason: 'how many: ${m.trouble}');
    await write('Waves', '900');
    expect(m.wayOf(cube, 'x', 0), startsWith('Cyclic 0.02 '), reason: 'no further than a hand can take it');
    await t.tap(_in(find.byKey(const ValueKey('ease-reverse'))));
    await settled(t, ws);
    await t.pumpAndSettle();
    await write('Waves', '4');
    expect(m.wayOf(cube, 'x', 0), startsWith('Reverse:Cyclic~0.25~'), reason: 'turned, it stays so: ${m.trouble}');
    final hand = await t.startGesture(t.getCenter(_in(find.byKey(const ValueKey('ease.Bounce')))), kind: PointerDeviceKind.mouse);
    await hand.moveBy(const Offset(-14, 0));
    await t.pump();
    await hand.moveTo(t.getCenter(_in(find.byType(EaseGraph))));
    await t.pump();
    await hand.up();
    await settled(t, ws);
    await hand.removePointer();
    await t.pumpAndSettle();
    await t.tap(_in(find.byKey(const ValueKey('ease.whole'))));
    await t.pumpAndSettle();
    expect(find.text('RESULT'), findsOneWidget);
    expect(_in(find.byType(DeskScrub)), findsNothing, reason: 'the whole has no numbers');
    await t.tap(_in(find.byKey(const ValueKey('ease.card.0'))));
    await t.pumpAndSettle();
    expect(_in(find.byType(DeskScrub)), findsWidgets, reason: 'a card again: as they were left, not lost');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('between two cards the flow says how the lower takes the upper: an arrow, a ×, a +', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.way(cube, 'x', 0, 'Bounce'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    Future<void> lay(String shape) async {
      await seeShape(t, shape);
      final hand = await t.startGesture(t.getCenter(_in(find.byKey(ValueKey('ease.$shape')))), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(-14, 0));
      await t.pump();
      await hand.moveTo(t.getCenter(_in(find.byType(EaseGraph))));
      await t.pump();
      await hand.up();
      await settled(t, ws);
      await hand.removePointer();
      await t.pumpAndSettle();
    }

    String? mark(int i) => find.byKey(ValueKey('ease.by.$i')).evaluate().isEmpty ? null : t.widget<Text>(find.descendant(of: find.byKey(ValueKey('ease.by.$i')), matching: find.byType(Text))).data;
    await lay('Elastic');
    expect(mark(1), isNull, reason: 'it goes through: the arrow');
    await lay('Cyclic');
    expect(mark(2), '×', reason: 'a wave is as large as what is above it');
    await lay('Random');
    expect(mark(3), '+', reason: 'a shake is added');
    expect(m.wayOf(cube, 'x', 0).split('×'), hasLength(4), reason: m.trouble);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('what a reader found: on Linear a shape goes in as pressed; the empty frame takes what is picked next; beats fall where the film has them; paste to all needs no key picked', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 2, 1) && m.key(cube, 'x', 4, 0), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    final graph = _in(find.byType(EaseGraph));
    Future<void> lay(String shape) async {
      final hand = await t.startGesture(t.getCenter(_in(find.byKey(ValueKey('ease.$shape')))), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(-14, 0));
      await t.pump();
      await hand.moveTo(t.getCenter(graph));
      await t.pump();
      await hand.up();
      await settled(t, ws);
      await hand.removePointer();
      await t.pumpAndSettle();
    }

    await lay('Cyclic');
    expect(m.wayOf(cube, 'x', 0), startsWith('Cyclic'), reason: 'on a straight line alone: as a press, ${m.trouble}');
    expect(m.wayOf(cube, 'x', 0), isNot(contains('×')));
    await t.tap(_in(find.byKey(const ValueKey('ease.In-out'))));
    await settled(t, ws);
    await t.tap(_in(find.byKey(const ValueKey('ease.slot'))));
    await t.pumpAndSettle();
    await t.tap(_in(find.byKey(const ValueKey('ease.Bounce'))));
    await settled(t, ws);
    await t.pumpAndSettle();
    expect(m.wayOf(cube, 'x', 0), allOf(contains('×'), contains('Bounce')), reason: 'the frame pressed: the next goes in it, ${m.trouble}');
    expect(_in(find.byKey(const ValueKey('ease.card.1'))), findsOneWidget);
    await t.tap(_in(find.byKey(const ValueKey('ease.Elastic'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), allOf(contains('×'), contains('Elastic'), isNot(contains('Bounce'))), reason: 'and only the next: then a press takes the card\'s place');
    expect(m.set(0, 'bpm', 120), isTrue, reason: m.trouble);   // (two beats a second: 2, 3... at frames 30, 60, 90 of 120)
    ws.reread();
    await t.pumpAndSettle();
    final inOut = easePreset('In-out');
    expect(t.widget<EaseGraph>(graph).beats.first, closeTo(inOut.at(.25), 1e-3), reason: 'the card under In-out is walked by it: the first beat is where In-out has come to');
    await t.tap(_in(find.byKey(const ValueKey('ease.card.0'))));
    await t.pumpAndSettle();
    expect(t.widget<EaseGraph>(graph).beats.first, closeTo(.25, 1e-6), reason: 'the top one by the clock');
    ws.clearKeys();
    ws.frame = 30;
    await t.pumpAndSettle();
    await menu(t, 'copy');
    await menu(t, 'all');
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 2), m.wayOf(cube, 'x', 0), reason: 'the other stretch too, no key picked');
    expect(easeIsUsual('Cyclic 0.285714286 0 0.5 0 0'), isTrue);
    expect(easeIsUsual('Cyclic 0.125 0 0.5 0 0'), isFalse, reason: 'its tile draws its own curve');
    expect(easeIsUsual('Bounce'), isTrue);
    expect(easeIsUsual('Bezier 0.42 0.0 1.0 1.0'), isTrue, reason: 'it draws its numbers as it is');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a stretch picked by its line: Ease is on that value\'s alone, the other value keyed there stays; Delete puts it straight, the thing stays', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1) && m.key(cube, 'y', 0, 0) && m.key(cube, 'y', .5, 1), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    ws.pickStretch('$cube', 'x', 0);
    await t.pumpAndSettle();
    expect(find.byType(DeskEase), findsOneWidget, reason: 'Ease to the front');
    expect(easeSpan(ws)?.to, 60, reason: 'to that value\'s next key, not y\'s at 0.5 s');
    await t.tap(_in(find.byKey(const ValueKey('ease.Bounce'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), startsWith('Bounce'), reason: m.trouble);
    expect(m.wayOf(cube, 'y', 0), 'Linear', reason: 'y keyed at the same frame stays as it was');
    t.widget<WsKeys>(find.byType(WsKeys)).commands['thing.remove']!();
    await settled(t, ws);
    await t.pumpAndSettle();
    expect(m.wayOf(cube, 'x', 0), 'Linear', reason: 'Delete: straight again, ${m.trouble}');
    expect(m.things, contains(cube), reason: 'and the thing still there');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a bounce that will not fit goes in the room left, lower, and lands on the key; one died away stays still; the usual numbers as Alight Motion\'s', (t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    expect(m.wayExact('Bounce 0.3 0.8', .98), 1, reason: 'the usual: under 0.05 of room, flat as Alight Motion\'s');
    expect(m.wayExact('Bounce 0.2 0.5', .6), 1, reason: 'died away at 0.3: still');
    expect(m.wayExact('Bounce 0.346 0.906', .885), lessThan(.5), reason: 'the third bounce, where it lay flat');
    expect(m.wayExact('Bounce 0.346 0.906', .995), greaterThan(.9), reason: 'and on the key at the end');
    double? was;
    for (var l = 300; l <= 400; l++) {
      final now = m.wayExact('Bounce ${l / 1000} 0.906', .9);
      if (was case final w?) expect((now - w).abs(), lessThan(.1), reason: 'no leap as Length goes: at ${l / 1000}');
      was = now;
    }
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
      await seeShape(t, on);
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
    await laid('Bezier 0.34 1.56 0.64 1', 'Bounce');
    expect((await m.shownAt([(cube, 'x', .6)])).single, greaterThan(1), reason: 'an overshoot stays one, through what comes after');
    expect(t.widget<EaseCards>(_in(find.byType(EaseCards))).whole!.drawn!.reduce(math.max), greaterThan(1), reason: 'and the whole shows it');
    expect(await m.shownAt([(cube, 'x', 0), (cube, 'x', 1)]), [0, 1], reason: 'on its keys at both ends, each');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
