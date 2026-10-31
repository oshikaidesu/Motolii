// The Ease desk's ways under the sample's eight (workspace/desk_ease_library.dart): a table picked (All or ★, as the
// font shelf picks one), the tiles scrolling sideways in an order that never moves, the starred a table of their own,
// the last used set apart at the head, both kept in a file.
import 'dart:io';

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart' show easeTitle;
import 'package:lab_book/workspace/desk_ease_library.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/tokens.dart' show Shape;

import 'door.dart';

Finder _in(Finder f) => find.descendant(of: find.byType(DeskEase), matching: f);

void main() {
  testWidgets('the ways a table each, All or ★; a star keeps one in ★ without moving it; the last used come first, set apart', (t) async {
    final kept = File('${Directory.systemTemp.createTempSync('stars').path}/ease_stars.json');
    easeStarsAt(kept);
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('ease.tables')), findsOneWidget, reason: 'the tables');
    final library = find.byKey(const ValueKey('ease.library.All'));
    expect(library, findsOneWidget, reason: 'one row that scrolls sideways');
    expect(t.widget<ListView>(library).scrollDirection, Axis.horizontal);
    Iterable<String> shown() => t.widgetList(find.descendant(of: find.byType(EaseLibrary), matching: find.byWidgetPredicate((w) => w.key is ValueKey<String> && RegExp(r'^ease\.(?!recent|star|library|tables)').hasMatch((w.key as ValueKey<String>).value))))
        .map((w) => (w.key as ValueKey<String>).value.substring(5));
    final firsts = shown().take(2).toList();
    expect(firsts, hasLength(2), reason: 'the ways the sample has no seat for: ${m.wayNames}');
    final way = firsts[1];
    // a star: shown only under the hand, kept in the file, moving nothing
    final star = find.byKey(ValueKey('ease.star.$way'));
    expect(star, findsNothing, reason: 'no star until the hand is over the tile');
    final hand = await t.createGesture(kind: PointerDeviceKind.mouse);
    await hand.addPointer(location: t.getCenter(find.byKey(ValueKey('ease.$way'))));
    await t.pumpAndSettle();
    await t.tap(star);
    await hand.removePointer();
    await t.pumpAndSettle();
    expect(shown().take(2), firsts, reason: 'a star moves nothing: where a tile is stays');
    expect(kept.readAsStringSync(), contains(way), reason: 'kept for the next time');
    await t.tap(_in(find.text('★')));
    await t.pumpAndSettle();
    expect(find.byKey(ValueKey('ease.$way')), findsOneWidget, reason: 'the starred, a table of their own');
    expect(shown(), [way]);
    await t.tap(_in(find.text('All')));
    await t.pumpAndSettle();
    // the tile on the stretch, then at the head, set apart
    await t.tap(find.byKey(ValueKey('ease.$way')));
    await settled(t, ws);
    final put = m.wayOf(cube, 'x', 0);
    expect(put == way || put.startsWith('$way '), isTrue, reason: 'the stretch goes by it (a kind with its numbers, as the library tells its own: desk_ease_library.dart): $put');
    expect((await m.shownAt([(cube, 'x', .5)])).single, closeTo(m.wayAt(put, .5), 1e-3), reason: 'as the way has it half way');
    expect(find.byKey(ValueKey('ease.recent.$way')), findsOneWidget, reason: 'the last used, at the head');
    expect(find.byKey(const ValueKey('ease.recent.end')), findsOneWidget, reason: 'set apart');
    expect(kept.readAsStringSync(), contains('"recent":["$way"]'));
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

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
    expect(find.byKey(const ValueKey('ease-paste-all')), findsNothing, reason: 'nothing copied yet');
    await t.tap(find.byKey(const ValueKey('ease-copy')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ease-paste-all')));
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
    final graph = find.byType(EaseGraph), r = t.getRect(graph), p2 = t.widget<EaseGraph>(graph).ease.p2;
    final g = EdgeInsets.fromLTRB(Shape.s(30), Shape.s(10), Shape.s(14), Shape.s(22)).deflateRect(r);   // (the graph's own box, desk_ease_graph.dart)
    const lo = -.42, hi = 1.62;
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
    await t.tap(_in(find.byKey(const ValueKey('ease.In'))));
    await settled(t, ws);
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

    await carry(_in(find.byKey(const ValueKey('ease.Cyclic'))), t.getCenter(graph), held: () async {
      expect(t.widget<EaseGraph>(graph).ease.name, contains('×'), reason: 'what it would make, seen while held');
      expect(m.wayOf(cube, 'x', 0), alone, reason: 'nothing changed until it is let go');
    });
    final laid = m.wayOf(cube, 'x', 0);
    expect(laid, allOf(contains('×'), contains('Cyclic')), reason: m.trouble);
    expect((await m.shownAt([(cube, 'x', .4)])).single, closeTo(m.wayAt(alone, .4) * m.wayAt('Cyclic', .4), .005), reason: 'the two multiplied (wayAt: read off the picture\'s curve of it, a step at a time)');
    expect(await m.shownAt([(cube, 'x', 0), (cube, 'x', 1)]), [0, 1], reason: 'on its keys at both ends');
    expect(_in(find.byKey(const ValueKey('ease.card.1'))), findsOneWidget, reason: 'each a card');
    expect(easeTitle(unseated(ws).firstWhere((e) => e.name == laid)), 'In × Cyclic', reason: 'and a way of the document, a tile');
    expect(m.undo() && m.wayOf(cube, 'x', 0) == alone, isTrue, reason: 'one step');
    expect(m.redo() && m.wayOf(cube, 'x', 0) == laid, isTrue);
    ws.reread();
    await t.pumpAndSettle();
    final out = t.getCenter(find.byKey(const ValueKey('ease.tables')));
    await carry(_in(find.byKey(const ValueKey('ease.card.1'))), t.getCenter(_in(find.byKey(const ValueKey('ease.card.0')))));
    expect(m.wayOf(cube, 'x', 0), laid, reason: 'let go among them: it stays');
    await carry(_in(find.byKey(const ValueKey('ease.card.1'))), out);
    expect(m.wayOf(cube, 'x', 0), alone, reason: 'carried out: it comes off, the one left alone');
    expect(_in(find.byKey(const ValueKey('ease.card.0'))), findsNothing, reason: 'one way: its numbers again');
    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph));
    expect(m.wayOf(cube, 'x', 0), contains('×'), reason: m.trouble);
    await t.tap(_in(find.byKey(const ValueKey('ease.Linear'))));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), 'Linear', reason: 'a press: in place of them all');
    await carry(_in(find.byKey(const ValueKey('ease.Bounce'))), t.getCenter(graph));
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
