// Relations made on the stage (relate.dart): drawn round where no thing is, the things inside picked together, the
// chips beside them; Around makes one behaviour with each of them connected, as one step, picked; undone, all gone.
import 'dart:io';

import 'package:flutter/services.dart' show MethodChannel;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/relate.dart';
import 'package:lab_book/workspace/stage.dart';

import 'door.dart';

void main() {
  testWidgets('drawn round, Around: one behaviour, every thing inside connected, one step', (t) async {
    final m = Motolii.open('${Directory.systemTemp.createTempSync('relate').path}/relate.motolii')!;
    addTearDown(m.close);
    final ws = Ws()..join(m);
    final made = [for (var i = 0; i < 5; i++) m.add('shape', 'circle')];
    for (final (i, (x, y)) in const [(-0.5, 0.3), (0.0, 0.4), (0.5, 0.2), (-0.3, -0.3), (0.4, -0.35)].indexed) {
      m
        ..set(made[i], 'size', .15)
        ..set(made[i], 'x', x)
        ..set(made[i], 'y', y);
    }
    ws.reread();
    t.view.physicalSize = const Size(924, 616);
    t.view.devicePixelRatio = 1;
    addTearDown(t.view.reset);
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);
    await t.pumpWidget(WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(ws: ws, child: const Align(alignment: Alignment.topLeft, child: SizedBox(width: 924, height: 616, child: WsStage()))),
    ));
    await t.pumpAndSettle();

    // (round the whole picture, from a corner where nothing is)
    final comp = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    final g = await t.startGesture(comp.topLeft + const Offset(4, 4));
    for (final p in [comp.topRight + const Offset(-4, 4), comp.bottomRight - const Offset(4, 4), comp.bottomLeft + const Offset(4, -4), comp.topLeft + const Offset(4, 8)]) {
      await g.moveTo(p);
      await t.pump();
    }
    await g.up();
    await t.pumpAndSettle();
    expect(ws.many, {for (final i in made) '$i'}, reason: 'every thing inside the line picked together');
    expect(find.byKey(const ValueKey('ws-relate-around')), findsOneWidget);

    final before = m.things.length;
    await t.tap(find.byKey(const ValueKey('ws-relate-around')));
    await settled(t, ws);
    final b = m.things.firstWhere((i) => !made.contains(i));
    expect(m.things.length, before + 1);
    expect(m.word(b, 'kind'), 'behaviour');
    expect(m.word(b, 'file'), 'around');
    for (final i in made) {
      expect(m.value(i, 'x_by', 0), b.toDouble(), reason: 'thing $i connected');
    }
    expect(ws.selected, '$b', reason: 'the relation picked: its card, its row');
    expect(wsMembersOf(ws), {for (final i in made) '$i'}, reason: 'what it moves, lit on the stage and the timeline');
    expect(ws.many, isEmpty);

    // (one taken out with its ×: it stays where it shows, and so does every other)
    List<double> where(int i) => m.frame(i)!.sublist(0, 2);
    Map<int, List<double>> all() => {for (final i in made) i: where(i)};
    void same(Map<int, List<double>> a, Map<int, List<double>> b, String why) {
      for (final i in made) {
        expect((a[i]![0] - b[i]![0]).abs() + (a[i]![1] - b[i]![1]).abs(), lessThan(0.5), reason: '$why: thing $i moved');
      }
    }

    // (its size carried by its handle on the stage: the ring grows)
    final r0 = m.value(b, 'radius', 0)!;
    final hand = await t.startGesture(t.getCenter(find.byKey(const ValueKey('ws-relation-size'))));   // (as a hand does: down, a moment, then away)
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
    for (var k = 1; k <= 16; k++) {   // (a finger in a test starts a drag only past 36 points)
      await hand.moveBy(const Offset(5, 0));
      await t.pump(const Duration(milliseconds: 16));
    }
    await hand.up();
    await settled(t, ws);
    expect(m.value(b, 'radius', 0)!, greaterThan(r0), reason: 'the ring grows as its handle goes out');
    m.undo();
    ws.reread();
    await t.pumpAndSettle();
    // (one in it carried by a hand: it moves on the picture as far as the hand went, the others stay)
    final held = made[2], was = where(held), others = {for (final i in made) if (i != held) i: where(i)};
    expect(m.hold(held), isTrue, reason: m.trouble);
    m.carry('place', 30, 0);
    m.letGo(true);
    ws.reread();
    expect(where(held)[0] - was[0], closeTo(30, 1.5), reason: 'a hand moves a thing in a relation');
    for (final e in others.entries) {
      expect((where(e.key)[0] - e.value[0]).abs(), lessThan(0.5), reason: 'thing ${e.key} moved with it');
    }
    final ring = all();
    await t.tap(find.byKey(ValueKey('ws-free-${made.first}')));
    await settled(t, ws);
    expect(m.value(made.first, 'x_by', 0), 0.0, reason: 'out of it');
    same(ring, all(), 'one taken out');
    expect(m.word(b, 'members').split(' '), contains('_'), reason: 'its place kept empty');
    // (released: every one stays where it shows, the relation gone)
    ws.select('$b');
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ws-relate-release')));
    await settled(t, ws);
    expect(m.things.contains(b), isFalse);
    same(ring, all(), 'released');

    for (var k = 0; k < 4; k++) {   // (each a step: made, carried, one out, released)
      m.undo();
    }
    ws.reread();
    expect(m.things.length, before);
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  test('a relation on a relation: Around over Push starts from what Push gives; out of Around, back in Push; nothing jumps', () {
    final m = Motolii.open('${Directory.systemTemp.createTempSync('stack').path}/stack.motolii')!;
    addTearDown(m.close);
    final made = [for (var i = 0; i < 5; i++) m.add('shape', 'circle')];
    for (final (i, x) in const [-0.6, -0.2, 0.1, 0.4, 0.8].indexed) {
      m
        ..set(made[i], 'size', i == 0 ? .4 : .12)
        ..set(made[i], 'x', x)
        ..set(made[i], 'y', i.isEven ? .1 : -.1);
    }
    List<double> where(int i) => m.frame(i)!.sublist(0, 2);
    void stays(Map<int, List<double>> was, String why) {
      for (final e in was.entries) {
        expect((where(e.key)[0] - e.value[0]).abs() + (where(e.key)[1] - e.value[1]).abs(), lessThan(0.5), reason: '$why: thing ${e.key} moved');
      }
    }

    final push = m.add('behaviour', 'push');
    expect(m.later('relate', thing: push, word: made.join(' ')), greaterThan(0));
    while (m.answer(m.later('together')) < 0) {}
    expect(m.value(made[1], 'x_by', 0), push.toDouble(), reason: 'in Push');
    final before = {for (final i in made.skip(1)) i: where(i)};
    final around = m.add('behaviour', 'around');
    m.set(around, 'strength', 0);   // (none of it yet: what shows is what Push gives, if Around starts from it)
    m.later('relate', thing: around, word: made.skip(1).take(3).join(' '));
    while (m.answer(m.later('together')) < 0) {}
    expect(m.value(made[1], 'x_by', 0), around.toDouble(), reason: 'in Around now');
    expect(m.word(around, 'under').split(' '), everyElement('$push'), reason: 'Around starts from Push for each');
    stays(before, 'Around made at strength 0');
    final out = {for (final i in made.skip(1)) i: where(i)};
    while (m.answer(m.later('free', thing: made[1], number: around.toDouble())) < 0) {}
    expect(m.value(made[1], 'x_by', 0), push.toDouble(), reason: 'back in Push');
    stays(out, 'one out of Around');
  }, skip: !insideBuilt);

  test('the pusher taken away (a row over the push): every thing stays where it shows', () {
    final m = Motolii.open('${Directory.systemTemp.createTempSync('lead').path}/lead.motolii')!;
    addTearDown(m.close);
    final things = [for (var i = 0; i < 6; i++) m.add('shape', 'circle')];
    for (final (i, x) in const [-0.6, -0.3, 0.0, 0.3, 0.6, 0.05].indexed) {
      m
        ..set(things[i], 'size', i == 5 ? .3 : .12)
        ..set(things[i], 'x', x)
        ..set(things[i], 'y', i == 5 ? 0.0 : 0.1);
    }
    void wait(int ticket) {
      while (m.answer(ticket) < 0) {}
    }

    final push = m.add('behaviour', 'push'), row = m.add('behaviour', 'row');
    wait(m.later('relate', thing: push, word: things.join(' ')));
    expect(m.value(push, 'leader', 0), things[5].toDouble(), reason: 'the largest pushes');
    wait(m.later('relate', thing: row, word: things.take(3).join(' ')));
    final before = {for (final t in things.take(5)) t: m.frame(t)!.sublist(0, 2)};
    wait(m.later('remove', thing: things[5]));
    expect(m.things.contains(push), isFalse, reason: 'what it led let go');
    for (final t in things.take(5)) {
      final now = m.frame(t)!;
      expect((now[0] - before[t]![0]).abs() + (now[1] - before[t]![1]).abs(), lessThan(0.5), reason: 'thing $t jumped');
    }
  }, skip: !insideBuilt);

  test('a follow led by another: nothing moves, the old leader follows, the new one leads', () {
    final m = Motolii.open('${Directory.systemTemp.createTempSync('lead2').path}/lead2.motolii')!;
    addTearDown(m.close);
    final things = [for (var i = 0; i < 4; i++) m.add('shape', 'circle')];
    for (final (i, x) in const [-0.6, -0.2, 0.2, 0.6].indexed) {
      m
        ..set(things[i], 'size', .12)
        ..set(things[i], 'x', x)
        ..set(things[i], 'y', i * .1);
    }
    void wait(int ticket) {
      while (m.answer(ticket) < 0) {}
    }

    List<double> where(int t) => m.frame(t)!.sublist(0, 2);
    final follow = m.add('behaviour', 'follow');
    wait(m.later('relate', thing: follow, word: things.join(' ')));
    expect(m.value(follow, 'leader', 0), things[0].toDouble(), reason: 'the leftmost leads');
    expect(m.value(follow, 'lag', 0), closeTo(.1, 1e-9), reason: 'its lag said');
    final before = {for (final t in things) t: where(t)};
    wait(m.later('lead', thing: follow, number: things[2].toDouble()));
    for (final t in things) {
      expect((where(t)[0] - before[t]![0]).abs() + (where(t)[1] - before[t]![1]).abs(), lessThan(0.5), reason: 'thing $t jumped');
    }
    final members = m.word(follow, 'members').split(' ');
    expect(members, contains('${things[0]}'), reason: 'the old leader follows');
    expect(members, isNot(contains('${things[2]}')), reason: 'the new leader is not one it moves');
    m.set(follow, 'lag', 0);
    final was = {for (final t in things) t: where(t)};
    m.set(things[2], 'x', 0.5);   // (the new leader moved 0.3 to the right: all go with it)
    for (final t in things) {
      expect(where(t)[0] - was[t]![0], greaterThan(10), reason: 'thing $t did not follow');
    }
  }, skip: !insideBuilt);
}
