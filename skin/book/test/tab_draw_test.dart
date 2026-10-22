// Way C of the Relations tab, by a hand's strokes on the real window (a copy of the try document): a loop round the
// three words makes a move of them; a stroke out from one of them, slowing at its end, shifts their other end that
// way and says how long and how it eases; a small ring round their middle gathers them in; a zigzag scatters them, a
// flick up clears them (or back); a stroke along the strip puts them in time. Read back through moves.dart.
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/relation_tabs/tab_draw.dart';
import 'package:lab_book/workspace/moves.dart';

import 'door.dart';

void main() {
  testWidgets('a move drawn: looped, shifted, gathered, scattered, cleared, timed', (t) async {
    final copy = '${Directory.systemTemp.createTempSync('tab_draw').path}/try.motolii';
    File('test/data/relation_try.motolii').copySync(copy);
    final ws = await openWorkspace(t, document: copy, extra: [WsPane('Relations C', null, () => const TabDraw())]);
    await t.tap(find.text('Relations C').first);
    await t.pumpAndSettle();
    final tab = t.state<TabDrawState>(find.byType(TabDraw)), o = t.getTopLeft(find.byType(TabDraw));
    Offset at(int thing) => o + tab.where(thing);
    final k = (at(4).dx - at(8).dx) / 1.7;   // (screen points a unit: photos 4 and 8 lie at x .85 and -.85)

    var ms = 0;
    Future<void> draw(List<Offset> pts, {List<int>? gaps}) async {
      final g = await t.createGesture(kind: PointerDeviceKind.mouse);
      await g.down(pts.first, timeStamp: Duration(milliseconds: ms));
      for (final (i, p) in pts.indexed.skip(1)) {
        ms += gaps?[i - 1] ?? 16;
        await g.moveTo(p, timeStamp: Duration(milliseconds: ms));
      }
      await g.up(timeStamp: Duration(milliseconds: ms));
      await settled(t, ws);
    }

    List<Offset> ring(Offset c, double rx, double ry) => [for (var i = 0; i <= 40; i++) c + Offset(math.cos(i / 40 * 2 * math.pi) * rx, math.sin(i / 40 * 2 * math.pi) * ry)];

    // a loop round the three words (and nothing else: the photos above are further than its height)
    final seat = {for (final w in [10, 11, 12]) w: at(w)};   // (where they settle: the move starts now, so from now on they show at its other end)
    final words = (at(10) + at(12)) / 2, ry = (at(6).dy - at(11).dy).abs() * .45;
    await draw(ring(words, (at(12).dx - at(10).dx) / 2 + 30, ry));
    var mv = wsMoves(ws).single;
    expect(mv.who, [10, 11, 12]);
    expect(ws.selected, '${mv.by}', reason: 'the move made is picked');

    // from word 10 out, left and a little down, 100 points over: fast at first, slow at its end
    final a = at(10), z = a + const Offset(-80, 60);
    await draw([for (var i = 0; i <= 10; i++) Offset.lerp(a, z, i / 10)!], gaps: [8, 8, 8, 8, 8, 8, 8, 40, 60, 90]);
    mv = wsMoves(ws).single;
    expect(mv.end['x'], closeTo(-80 / k, .01));
    expect(mv.end['y'], closeTo(-60 / k, .01));
    expect(mv.end['spread'], 1);
    expect(mv.length, closeTo(240 / (100 / .246), .02), reason: 'drawn at about 400 points a second: about .6 s');
    expect(mv.out, greaterThan(.5), reason: 'slowing at its end: eases out');

    // a small ring round their middle: gathered in, as much as it is small
    final wide = seat[12]!.dx - seat[10]!.dx;
    await draw(ring(seat[11]!, 12, 12));
    mv = wsMoves(ws).single;
    expect(mv.end['spread'], closeTo(24 / wide, .05));
    expect(mv.end['x'], closeTo(-80 / k, .01), reason: 'the shift kept');

    // a zigzag out from one of them: scattered, as much as it is big
    final b = at(12);
    await draw([for (var i = 0; i <= 6; i++) b + Offset(i * 10.0, i.isEven ? 0 : -16)]);
    mv = wsMoves(ws).single;
    expect(mv.end['scatter'], closeTo(30 / k, .02));

    // up from one of them with a flick at its end: clear (it came clear: back to whole)
    final c = at(10);
    await draw([for (var i = 0; i <= 10; i++) c + Offset(0, -5.0 * i), c + const Offset(6, -54), c + const Offset(14, -56)]);
    mv = wsMoves(ws).single;
    expect(mv.end['opacity'], 1);
    expect(mv.end['scatter'], closeTo(30 / k, .02), reason: 'the scatter kept');

    // along the strip, from 1 s to 1.4 s: starts at 1 s, the three .2 s apart
    final box = t.getRect(find.byType(TabDraw));
    Offset lane(double s) => Offset(box.left + 12 + s / (Ws.duration / Ws.fps) * (box.width - 24), box.bottom - 20);
    await draw([for (var i = 0; i <= 8; i++) Offset.lerp(lane(1), lane(1.4), i / 8)!]);
    mv = wsMoves(ws).single;
    expect([mv.start, mv.lag], [closeTo(1, .01), closeTo(.2, .01)]);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
