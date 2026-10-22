// Way F of the Relations tab on the real window and a copy of the try document: three things picked, a tile tapped,
// and the document holds a move of those three made of that tile's numbers; another tile, the move made so with its
// ease kept; the stairs pulled sideways, its lag grows; the pick whip then the cube: they come out of the cube.
// (The tiles loop for ever, so the window is never still: pumped by hand here, not settled.)
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/relation_tabs/tab_samples.dart';
import 'package:lab_book/workspace/moves.dart';

import 'door.dart';

void main() {
  testWidgets('a sample tapped makes the move it shows; the stairs pull the lag', (t) async {
    final copy = '${Directory.systemTemp.createTempSync('tab_samples').path}/try.motolii';
    File('test/data/relation_try.motolii').copySync(copy);
    final ws = await openWorkspace(t, document: copy, extra: [WsPane('Relations F', null, () => const TabSamples())]), m = ws.inside!;
    Future<void> rest() async {
      for (var i = 0; i < 80; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
        await t.pump(const Duration(milliseconds: 16));
        if (!ws.waiting && i > 2) break;
      }
    }

    final three = [
      for (final l in ws.layers)
        if (int.tryParse(l.id) case final x? when !const {'behaviour', 'camera', 'light'}.contains(m.word(x, 'kind'))) l.id,
    ].take(3).toList();
    expect(three, hasLength(3));
    await t.tap(find.text('Relations F').first);
    await rest();
    ws.pickMany(three);
    await rest();

    final box = t.getRect(find.byType(TabSamples)), lay = samplesLayout(box.size);
    const pop = 9;   // (size 0, past the seat and back)
    await t.tapAt(box.topLeft + lay.come[pop].center);
    await rest();
    var mv = wsMoves(ws).single;
    expect(mv.who.map((x) => '$x'), unorderedEquals(three));   // (in the inside's order)
    expect(mv.coming, true);
    for (final e in samples[pop].end.entries) {
      expect(mv.end[e.key], closeTo(e.value, 1e-6), reason: e.key);
    }
    expect([mv.out, mv.over], [closeTo(samples[pop].out, 1e-3), closeTo(samples[pop].over, 1e-3)]);
    expect(ws.selected, '${mv.by}', reason: 'the new move picked');

    const left = 1;   // (the picked move told going out to the left)
    await t.tapAt(box.topLeft + lay.go[left].center);
    await rest();
    mv = wsMoves(ws).single;
    expect(mv.coming, false);
    expect([mv.end['x'], mv.end['size'], mv.over], [closeTo(-.35, 1e-6), closeTo(1, 1e-6), closeTo(samples[pop].over, 1e-3)]);   // (its ease kept: the Ease desk's)

    final lag = mv.lag;
    await t.dragFrom(box.topLeft + lay.pulls[0].center, const Offset(80, 0));
    await rest();
    mv = wsMoves(ws).single;
    expect(mv.lag, greaterThan(lag + .1));

    final cube = ws.layers.firstWhere((l) => m.word(int.parse(l.id), 'kind') == 'solid').id;
    await t.tapAt(box.topLeft + lay.come[whip].center);   // (the pick whip, then the cube picked as a hand picks it)
    await rest();
    ws.select(cube);
    await rest();
    await rest();
    mv = wsMoves(ws).single;
    final at = mv.away, c = int.parse(cube);
    double mid(String n) => mv.who.fold(0.0, (a, o) => a + (m.value(o, n, at) ?? 0)) / mv.who.length;
    expect(mv.coming, true);
    expect([mv.end['spread'], mv.end['x']! + mid('x'), mv.end['y']! + mid('y')], [closeTo(0, 1e-6), closeTo(m.value(c, 'x', at)!, 1e-6), closeTo(m.value(c, 'y', at)!, 1e-6)]);
    expect(ws.selected, '${mv.by}', reason: 'the move picked again');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
