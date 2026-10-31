// The handles broken on purpose (the maker, 10/31: "ハンドルを壊しのテスト"), not by chosen cases but as QuickCheck asks a
// program (Claessen & Hughes, 2000): any move at all, from a seed kept so that a failure comes again, and one thing that must hold of
// each. Here: what the hand last saw is what stays, in the panel and in the document, and nothing else is seen on the way.
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_handles.dart';
import 'package:lab_book/workspace/desk_ease_side.dart' show easeShapes;
import 'package:lab_book/workspace/desk_ease_state.dart';
import 'door.dart';

// (the same way: the inside says its numbers shorter than the window does)
bool same(String a, String b) {
  final x = a.split(RegExp('[ ~×]')), y = b.split(RegExp('[ ~×]'));
  if (x.length != y.length) return false;
  for (var i = 0; i < x.length; i++) {
    final p = double.tryParse(x[i]), q = double.tryParse(y[i]);
    if (p == null || q == null ? x[i] != y[i] : (p - q).abs() > 1e-4) return false;
  }
  return true;
}

void main() {
  late WidgetTester t;
  late Ws ws;
  late int cube;
  late Finder graph;
  EaseGraph g() => t.widget<EaseGraph>(graph);
  Offset place(EaseHandle h) {
    final r = t.getRect(graph), box = easeBox(r.size).shift(r.topLeft), e = g().ends;
    final (lo, hi) = easeFit(g().ease, g().overshoot, e, g().onion);
    return Offset(box.left + h.x * box.width, box.bottom - (e.$1 + (e.$2 - e.$1) * h.y - lo) / (hi - lo) * box.height);
  }

  Future<void> open(WidgetTester tester) async {
    t = tester;
    ws = await openWorkspace(t, size: const Size(1600, 960));
    final m = ws.inside!;
    cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue);
    ws..reread()..pickKey((layer: '$cube', frame: 0))..front('Ease');
    await t.pumpAndSettle();
    graph = find.descendant(of: find.byType(DeskEase), matching: find.byType(EaseGraph));
  }

  Future<void> press(String key) async {
    if (easeShapes.any((s) => key == 'ease.$s')) await seeShape(t, key.substring(5));
    await t.tap(find.byKey(ValueKey(key)));
    await settled(t, ws);
  }

  Future<void> layOn(String shape) async {
    await seeShape(t, shape);
    final hand = await t.startGesture(t.getCenter(find.byKey(ValueKey('ease.$shape'))), kind: PointerDeviceKind.mouse);
    await hand.moveBy(const Offset(-14, 0));
    await t.pump();
    await hand.moveTo(t.getCenter(graph));
    await t.pump();
    await hand.up();
    await settled(t, ws);
  }

  // One handle carried to `to` a little at a time and let go; what it makes, as it was let go.
  Future<String> pull(EaseHandle h, Offset to, int steps) async {
    final from = place(h), hand = await t.startGesture(from, kind: PointerDeviceKind.mouse);
    for (var i = 1; i <= steps; i++) {
      await hand.moveTo(Offset.lerp(from, to, i / steps)!);
      await t.pump();
    }
    final shown = said(g().ease);
    await hand.up();
    for (var f = 0; f < 8; f++) {
      await t.pump();
      expect(same(said(g().ease), shown), isTrue, reason: 'seen after letting go: ${said(g().ease)}, let go at: $shown');
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 6)));
    }
    await settled(t, ws);
    await t.pump();
    await hand.removePointer();
    return shown;
  }

  Future<void> wrench(math.Random dice, String what, int times) async {
    for (var round = 0; round < times; round++) {
      final hs = easeHandles(g().ease);
      if (hs.isEmpty) return;
      final r = t.getRect(graph), box = easeBox(r.size).shift(r.topLeft);
      final to = [box.topLeft, box.topRight, box.bottomLeft, box.bottomRight, box.center, box.topLeft - const Offset(300, 300), box.bottomRight + const Offset(300, 300), Offset(dice.nextDouble() * 1600, dice.nextDouble() * 960)][dice.nextInt(8)];
      final shown = await pull(hs[dice.nextInt(hs.length)], to, 2 + dice.nextInt(7));
      final now = said(g().ease), kept = easeCards(ws.inside!.wayOf(cube, 'x', 0));
      expect(same(now, shown), isTrue, reason: '$what: held $shown, left $now');
      expect(kept.any((c) => same(said(heard(c)), shown) || same(easeIsTurned(c) ? c.replaceAll('~', ' ') : c, shown)), isTrue, reason: '$what: held $shown, the document says $kept');
    }
    expect(ws.inside!.trouble, isEmpty, reason: what);
  }

  testWidgets('any shape, any handle, anywhere: it stays where it is let go', (tester) async {
    await open(tester);
    final dice = math.Random(7);
    for (final s in easeShapes) {
      await press('ease.$s');
      await wrench(dice, s, 12);
    }
    await closeWorkspace(t);
  }, timeout: const Timeout(Duration(minutes: 3)));

  testWidgets('and so on a shape turned, on each card laid, past 0 and 1', (tester) async {
    await open(tester);
    final dice = math.Random(11);
    for (final s in ['Bounce', 'Cyclic', 'Random']) {
      await press('ease.$s');
      await press('ease-reverse');
      await wrench(dice, '$s turned', 8);
    }
    await press('ease.Bounce');
    await layOn('Elastic');
    await layOn('Random');
    for (final card in [2, 0, 1]) {
      await press('ease.card.$card');
      await wrench(dice, 'card $card', 8);
    }
    await press('ease.whole');
    await press('ease.card.0');
    await press('ease.Linear');
    easeOvershoot = true;
    addTearDown(() => easeOvershoot = false);
    await settled(t, ws);
    await wrench(dice, 'past 0 and 1', 12);
    await closeWorkspace(t);
  }, timeout: const Timeout(Duration(minutes: 3)));

  testWidgets('a curve pulled straight is still a curve: its handles stay in the corners', (tester) async {
    await open(tester);
    await press('ease.Linear');
    final r = t.getRect(graph), box = easeBox(r.size).shift(r.topLeft);
    await pull(easeHandles(g().ease)[0], box.bottomLeft + const Offset(-200, 200), 4);
    await pull(easeHandles(g().ease)[1], box.topRight + const Offset(200, -200), 4);
    final hs = easeHandles(g().ease);
    expect([hs[0].x, hs[0].y, hs[1].x, hs[1].y], [0, 0, 1, 1]);
    await press('ease.Linear');
    expect(easeHandles(g().ease)[0].x, .25);
    await closeWorkspace(t);
  });
}
