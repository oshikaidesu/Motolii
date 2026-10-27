// Way E of the Relations tab (lib/sets/relation_tabs/tab_poses.dart) by hand on the real window and a copy of the try
// document: a sweep over HOME makes a move of every thing; a drag in THERE takes the other end away to the right; the
// arrow tapped turns it to going out; the wheel over THERE spreads it; the arrow bent eases it; the strip's left edge
// puts it later. Each read back from the document (moves.dart).
import 'dart:io';

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/relation_tabs/tab_poses.dart';
import 'package:lab_book/workspace/moves.dart';

import 'door.dart';

void main() {
  testWidgets('a move made, pushed, turned, spread, eased and put later in two poses', (t) async {
    final copy = '${Directory.systemTemp.createTempSync('poses').path}/try.motolii';
    File('test/data/relation_try.motolii').copySync(copy);
    final ws = await openWorkspace(t, document: copy, extra: [WsPane('Relations E', null, () => const TabPoses())]);
    addTearDown(() => closeWorkspace(t));
    await t.tap(find.text('Relations E'));
    await settled(t, ws);
    expect(wsMoves(ws), isEmpty);

    Future<void> drag(Offset from, List<Offset> by) async {
      final g = await t.startGesture(from, kind: PointerDeviceKind.mouse);
      for (final d in by) {
        await g.moveBy(d);
        await t.pump();
      }
      await g.up();
      await settled(t, ws);
    }

    Rect part(String name) => t.getRect(find.byKey(ValueKey('poses-$name')));

    // WHO: swept over the whole of HOME
    final home = part('home'), m = ws.inside!;
    for (var i = 0; i < 200 && ws.layers.length < m.things.length; i++) {   // (the things the sweep takes are the window's layers: read once the document's picture has come)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final hit = HitTestResult();   // (what the hand first lands on there: the pane's HOME, or something over it)
    WidgetsBinding.instance.hitTestInView(hit, home.topLeft + const Offset(2, 2), t.view.viewId);
    final under = [for (final e in hit.path.take(8)) e.target.runtimeType.toString()];
    await drag(home.topLeft + const Offset(2, 2), [for (var i = 0; i < 8; i++) (home.size.bottomRight(Offset.zero) - const Offset(4, 4)) / 8]);
    final things = {for (final l in ws.layers) if (int.tryParse(l.id) case final i? when !const {'behaviour', 'camera', 'light'}.contains(m.word(i, 'kind'))) i};
    for (var i = 0; i < 200 && wsMoves(ws).isEmpty; i++) {   // (made by the inside, not waited for by the window)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    expect(wsMoves(ws), hasLength(1), reason: 'a sweep over HOME ${part('home')} makes a move; the things ${things.toList()} at ${[for (final i in things) (m.value(i, 'x', 0), m.value(i, 'y', 0))]}; ${ws.notice}; the inside has ${[for (final i in m.things) m.word(i, 'kind')]}; ${m.trouble}; point ${m.point}, frame ${ws.frame}, as the pane reads them ${[for (final i in things) (wsValue(m, i, 'x', ws.frame / Ws.fps, 0), wsValue(m, i, 'y', ws.frame / Ws.fps, 0))]}; the hand first on $under');
    var mv = wsMoves(ws).single;
    expect(mv.who.toSet(), things);
    expect(ws.selected, '${mv.by}');
    expect([mv.coming, mv.end['x'], mv.end['y']], [true, 0, closeTo(-.15, 1e-9)]);

    // FROM -> TO: THERE dragged to the right, straight across (one step)
    final there = part('there');
    await drag(there.center, [for (var i = 0; i < 6; i++) const Offset(10, 0)]);
    mv = wsMoves(ws).single;
    expect(mv.end['x'], greaterThan(.05));
    expect(mv.end['y'], closeTo(-.15, 1e-9));
    final p = await wsShowsAt(m, mv.who.first, mv.away), seat = (await m.valuesAt([(mv.who.first, 'x', mv.away)])).single!;   // (another moment: asked, BRIDGE.md rule 7)
    expect(p.x, greaterThan(seat));   // (the inside shows it there, to the right of its seat)

    // the arrow tapped: going out
    await t.tapAt(part('arrow').center + const Offset(0, 30), kind: PointerDeviceKind.mouse);
    await settled(t, ws);
    mv = wsMoves(ws).single;
    expect(mv.coming, false);

    // the wheel over THERE: spread out
    final wheel = TestPointer(9, PointerDeviceKind.mouse)..hover(there.center);
    await t.sendEventToBinding(wheel.hover(there.center));
    for (var i = 0; i < 3; i++) {
      await t.sendEventToBinding(wheel.scroll(const Offset(0, -40)));
      await t.pump();
    }
    await t.pump(const Duration(milliseconds: 400));
    await settled(t, ws);
    mv = wsMoves(ws).single;
    expect(mv.end['spread'], closeTo(1.6, 1e-6));

    // the arrow bent all the way up and past: slowing hard, overshooting
    await drag(part('arrow').center, [for (var i = 0; i < 6; i++) const Offset(0, -12)]);
    mv = wsMoves(ws).single;
    expect(mv.out, closeTo(1, 1e-3));
    expect(mv.over, greaterThan(.2));

    // the strip's left edge pulled right: later, as long as it was
    final strip = part('strip'), was = mv.start, long = mv.length;
    await drag(strip.centerLeft + const Offset(3, 0), [for (var i = 0; i < 6; i++) const Offset(8, 0)]);
    mv = wsMoves(ws).single;
    expect(mv.start, greaterThan(was + .2));
    expect(mv.length, closeTo(long, 1e-6));
    expect(mv.coming, false);
  }, skip: !insideBuilt);
}
