// Relations tab, way B (tab_curve.dart), by hand on the real window and a copy of the try document: tap the graph to
// scrub, sweep down over the waiting chips (a move at the playhead), pull its last step (lag), grab a bar (start), pull
// a bar's end (length), pull the curve up (overshoot), tap the arrow (going out), tap a tile, drag across one; each
// read back from the document. (Where things sit is the tab's own layout: tiles 40 high, ruler 14, chips' column 26,
// graph to 8 from the right, chips 16 apart, the picked move's rows 14.)
import 'dart:io';

import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/relation_tabs/tab_curve.dart';
import 'package:lab_book/workspace/moves.dart';

import 'door.dart';

void main() {
  testWidgets('a move made, spread, moved, stretched, eased, turned and its other end picked, by hand', (t) async {
    final copy = '${Directory.systemTemp.createTempSync('tab_curve').path}/try.motolii';
    File('test/data/relation_try.motolii').copySync(copy);
    final ws = await openWorkspace(t, document: copy, extra: [WsPane('Relations B', null, () => const TabCurve())]);
    await t.tap(find.text('Relations B').first);
    await settled(t, ws);

    final r = t.getRect(find.byType(TabCurve)), secs = Ws.duration / Ws.fps, gl = r.left + 26, gw = r.width - 34, top = r.top + 40 + 14 + 4, pps = gw / secs;
    double x(double s) => gl + s / secs * gw;
    double row(int k) => top + k * 14 + 7;
    Offset tile(int i) {
      final w = (r.width - 4) / 7;
      return Offset(r.left + 4 + i * w + (w - 4) / 2, r.top + 20);
    }

    Future<void> drag(Offset from, Offset by) async {
      await t.dragFrom(from, by, kind: PointerDeviceKind.mouse);
      await settled(t, ws);
    }

    Future<void> tap(Offset at) async {
      await t.tapAt(at, kind: PointerDeviceKind.mouse);
      await settled(t, ws);
    }

    expect(wsMoves(ws), isEmpty);
    await tap(Offset(x(1), r.bottom - 20));
    expect(ws.frame, Ws.fps, reason: 'a tap on the empty graph scrubs');

    await drag(Offset(r.left + 13, row(0)), const Offset(0, 32));   // (chips 0, 1, 2)
    var mv = wsMoves(ws).single;
    expect(mv.who.length, 3);
    expect(mv.start, closeTo(1, 1e-6), reason: 'made at the playhead');
    expect(ws.selected, '${mv.by}');

    await drag(Offset(x(mv.start + 2 * mv.lag) + 8, row(2)), Offset(.2 * 2 * pps, 0));
    mv = wsMoves(ws).single;
    expect(mv.lag, closeTo(.28, .01), reason: 'the last step pulled: the steps spread');
    expect(mv.start, closeTo(1, 1e-6));

    await drag(Offset(x(mv.start) + 8, row(0)), Offset(.5 * pps, 0));
    mv = wsMoves(ws).single;
    expect(mv.start, closeTo(1.5, 1 / 30), reason: 'a bar grabbed: the whole staircase in time');
    expect([mv.length, mv.lag], [closeTo(.6, 1e-6), closeTo(.28, .01)]);

    await drag(Offset(x(mv.start + mv.length), row(0)), Offset(.4 * pps, 0));
    mv = wsMoves(ws).single;
    expect(mv.length, closeTo(1, 1 / 30), reason: 'a bar\'s end pulled: longer');
    expect(mv.start, closeTo(1.5, 1 / 30));

    final last = x(mv.start + 2 * mv.lag + mv.length), curve = Offset(last.clamp(0, r.right - 30) + 6 + 11, top + 2 * 14 + 1 - 4 + 11);
    await drag(curve, const Offset(0, -20));   // (from .8 up by .4: past the seat)
    mv = wsMoves(ws).single;
    expect([mv.out, mv.over], [closeTo(1, 1e-3), closeTo(.2, .01)], reason: 'the curve pulled over its top: overshoot');

    expect(mv.coming, true);
    await tap(Offset(x(mv.start) - 10, row(0) - .5));
    mv = wsMoves(ws).single;
    expect(mv.coming, false, reason: 'the arrow tapped: going out');

    expect(mv.end['spread'], 1);
    await tap(tile(2));
    mv = wsMoves(ws).single;
    expect(mv.end['spread'], closeTo(2, 1e-6), reason: 'the burst tile on');

    await drag(tile(3), const Offset(60, 0));
    mv = wsMoves(ws).single;
    expect(mv.end['scatter'], closeTo(1, .02), reason: 'the scatter tile dragged across');
    expect(mv.who.length, 3);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
