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

    m.undo();   // (one step: all of it)
    ws.reread();
    expect(m.things.length, before);
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
