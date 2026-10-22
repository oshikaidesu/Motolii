// A group's move told and read back (moves.dart): made of three things it comes in at 1 s, each later, from below
// and clear; at the other end they show there (asked of the inside); put later, turned round, eased: read back so.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/key_drag.dart';
import 'package:lab_book/workspace/moves.dart';

import 'door.dart';

void main() {
  testWidgets('a move made, read, moved, turned, eased', (t) async {
    final m = Motolii.open('${Directory.systemTemp.createTempSync('moves').path}/moves.motolii')!;
    addTearDown(m.close);
    final ws = Ws()..join(m);
    final made = [for (var i = 0; i < 3; i++) m.add('shape', 'circle')];
    for (final (i, t) in made.indexed) {
      m
        ..set(t, 'x', -0.4 + i * 0.4)
        ..set(t, 'y', 0.2);
    }
    ws.reread();
    await t.pumpWidget(const SizedBox());
    Future<void> rest() async {
      for (var i = 0; i < 80 && ws.waiting; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
        await t.pump();
      }
    }

    wsMakeMove(ws, [for (final t in made) '$t'], 1);
    await rest();
    var mv = wsMoves(ws).single;
    expect(mv.who, made);
    expect([mv.start, mv.length, mv.lag], [1.0, closeTo(.6, 1e-9), .08]);
    expect(mv.coming, true);
    expect(mv.end['opacity'], 0);
    final away = wsShows(m, made[1], mv.away), home = wsShows(m, made[1], mv.last + .1);
    expect([away.y, away.opacity], [closeTo(.05, 1e-6), closeTo(0, 1e-6)]);   // (from below, clear)
    expect([home.y, home.opacity], [closeTo(.2, 1e-6), closeTo(1, 1e-6)]);   // (at its seat)

    wsMoveTime(ws, mv, 2, 1);
    await rest();
    mv = wsMoves(ws).single;
    expect([mv.start, mv.length, mv.out], [2.0, closeTo(1, 1e-9), closeTo(.8, 1e-3)]);

    wsMoveTurn(ws, mv);
    await rest();
    mv = wsMoves(ws).single;
    expect(mv.coming, false);
    expect(wsShows(m, made[0], 1.5).y, closeTo(.2, 1e-6));   // (going out: at its seat before)

    wsMoveEase(ws, mv, 0, .3);
    await rest();
    mv = wsMoves(ws).single;
    expect([mv.out, mv.over], [closeTo(0, 1e-3), closeTo(.3, 1e-3)]);

    wsSlideKeys(ws, [(layer: '${mv.by}', frame: 120)], 30);   // (the first key half a second later on the timeline: its way goes with it)
    await rest();
    mv = wsMoves(ws).single;
    expect([mv.start, mv.length, mv.over], [closeTo(2.5, 1e-9), closeTo(.5, 1e-9), closeTo(.3, 1e-3)]);
  }, skip: !insideBuilt);
}
