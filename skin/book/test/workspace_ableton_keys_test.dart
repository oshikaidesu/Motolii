// The window's keys as Ableton Live 12 has them (its manual, chapter 42; workspace/ableton.dart): Space plays from where
// it began and stops back there, Shift+Space goes on; the arrows nudge what is picked, or the playhead; up and down go
// between rows; Cmd+R renames in the row; Cmd+L loops; + and - zoom, Z zooms to what is picked and X goes back.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/ableton.dart';
import 'package:lab_book/workspace/keys.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/timeline_paint.dart';

import 'door.dart';

const _size = Size(1226, 300);

/// A new document: a rect keyed on x at 0.8 and 1.6 s, a ring on from 1 s.
({Ws ws, int rect, int ring}) _joined() {
  final m = Motolii.open('${Directory.systemTemp.createTempSync('ableton').path}/ableton.motolii')!;
  addTearDown(m.close);
  final ring = m.add('solid', 'ring'), rect = m.add('shape', 'rect');
  expect(m.set(ring, 'start', 1) && m.key(rect, 'x', .8, 0) && m.key(rect, 'x', 1.6, 1), isTrue, reason: m.trouble);
  return (ws: Ws()..join(m), rect: rect, ring: ring);
}

Future<void> _pump(WidgetTester t, Ws ws) async {
  t.view.physicalSize = _size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.binding.setSurfaceSize(_size);
  await t.pumpWidget(WidgetsApp(
    color: const Color(0xFF000000),
    builder: (_, _) => WsScope(ws: ws, child: Align(alignment: Alignment.topLeft, child: SizedBox(width: _size.width, height: _size.height, child: WsTimeline()))),
  ));
  await t.pumpAndSettle();
}

/// How many frames lie across the ruler between x = 400 and x = 1000.
Future<int> _across(WidgetTester t, Ws ws) async {
  final y = WtM.header + WtM.ruler - 6;
  await t.pump(const Duration(seconds: 1));   // (taps a second apart: never a double-click, which marks)
  await t.tapAt(Offset(400, y));
  final a = ws.frame;
  await t.pump(const Duration(seconds: 1));
  await t.tapAt(Offset(1000, y));
  return ws.frame - a;
}

void main() {
  test('the keys are Live\'s: Cmd+R renames, Cmd+Shift+R exports, Space plays from the start, the arrows nudge', () {
    final k = Keymap.load('/nonexistent');
    String? of(String key) => k.pairs.where((p) => p.$1 == activatorOf(key)).firstOrNull?.$2;
    bool same(SingleActivator a, SingleActivator b) => a.trigger == b.trigger && a.meta == b.meta && a.shift == b.shift;
    String? said(String key) => k.pairs.where((p) => same(p.$1, activatorOf(key)!)).firstOrNull?.$2;
    expect(of('cmd+r') ?? said('cmd+r'), 'thing.rename');
    expect(said('cmd+shift+r'), 'file.render');
    expect(said('space'), 'play.fromStart');
    expect(said('shift+space'), 'play.continue');
    expect(said('left'), 'nudge.left');
    expect(said('up'), 'layer.previous');
    expect(said('cmd+l'), 'loop.toggle');
    expect(said('z'), 'zoom.selection');
    expect(said(','), 'frame.previous');
  });

  testWidgets('Space plays from where it began and stops back there; Shift+Space goes on from where it stopped', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    ws.frame = 30;
    does['play.fromStart']!();
    expect(ws.playing, isTrue);
    ws.frame = 50;   // (as playing carries it)
    does['play.fromStart']!();
    expect((ws.playing, ws.frame), (false, 30), reason: 'stopped back at the start');
    ws.frame = 40;
    does['play.continue']!();
    ws.frame = 60;
    does['play.continue']!();
    expect((ws.playing, ws.frame), (false, 60), reason: 'stays where it stopped');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the arrows nudge the picked keys, else the picked bar, else the playhead; up and down go between rows', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    final f = (.8 * Ws.fps).round();
    ws.pickKey((layer: '${j.rect}', frame: f));
    does['nudge.right']!();
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.layers.firstWhere((l) => l.id == '${j.rect}').keys, contains(f + 1), reason: 'the key went a frame later');
    ws.select('${j.ring}');
    final inF = ws.layer!.inF;
    does['nudge.left10']!();
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.layers.firstWhere((l) => l.id == '${j.ring}').inF, inF - 10, reason: 'the bar went 10 frames earlier');
    ws.select(null);
    ws.frame = 20;
    does['nudge.right']!();
    expect(ws.frame, 21, reason: 'nothing picked: the playhead');
    ws.select(ws.layers.first.id);
    does['layer.next']!();
    expect(ws.selected, ws.layers[1].id);
    does['layer.previous']!();
    expect(ws.selected, ws.layers.first.id);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('Cmd+R writes the picked thing\'s name in its row; Enter keeps it, Esc leaves it as it was', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    ws.select('${j.rect}');
    does['thing.rename']!();
    await t.pumpAndSettle();
    final field = find.byKey(ValueKey('ws-tl-rename-${j.rect}'));
    expect(field, findsOneWidget);
    await t.enterText(field, 'Hero');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    ws.reread();
    await t.pumpAndSettle();
    expect(ws.layers.firstWhere((l) => l.id == '${j.rect}').name, 'Hero');
    does['thing.rename']!();
    await t.pumpAndSettle();
    await t.enterText(field, 'Nope');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.layers.firstWhere((l) => l.id == '${j.rect}').name, 'Hero', reason: 'Esc keeps the name');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('+ and - zoom, W fits, Z zooms to the picked keys and X goes back, Cmd+L loops', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    final fit = await _across(t, ws);
    does['zoom.in']!();
    await t.pumpAndSettle();
    final zoomed = await _across(t, ws);
    expect(zoomed, lessThan(fit), reason: '+ zooms in');
    does['zoom.width']!();
    await t.pumpAndSettle();
    expect(await _across(t, ws), fit, reason: 'W fits the width');
    ws.pickKeys([(layer: '${j.rect}', frame: (.8 * Ws.fps).round()), (layer: '${j.rect}', frame: (1.6 * Ws.fps).round())]);   // (the rect's bar is the whole document: its keys are less)
    does['zoom.selection']!();
    await t.pumpAndSettle();
    expect(await _across(t, ws), lessThan(fit), reason: 'Z zooms to the picked keys');
    does['zoom.back']!();
    await t.pumpAndSettle();
    expect(await _across(t, ws), fit, reason: 'X goes back');
    final looping = ws.inside!.looping;
    does['loop.toggle']!();
    await t.pumpAndSettle();
    expect(ws.inside!.looping, !looping);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
