// The timeline on a real document (a window's state joined to the inside): its rows are the document's things, a
// label or a bar picks one, a key picks a key, the ruler scrubs, play and the transport step the moment.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/timeline_paint.dart';

import 'door.dart';

const _ref = Size(1226, 300);

/// A window's state on a new document: a rect keyed on x at 0.8 and 1.6 s, a ring on from 1 s.
({Ws ws, int rect, int ring}) _joined() {
  final m = Motolii.open('${Directory.systemTemp.createTempSync('timeline').path}/timeline.motolii')!;
  addTearDown(m.close);
  final ring = m.add('solid', 'ring'), rect = m.add('shape', 'rect');
  expect(m.set(ring, 'start', 1) && m.key(rect, 'x', .8, 0) && m.key(rect, 'x', 1.6, 1), isTrue, reason: m.trouble);
  final ws = Ws()..join(m);
  return (ws: ws, rect: rect, ring: ring);
}

Future<void> _pump(WidgetTester t, Ws ws, {Size size = _ref}) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.binding.setSurfaceSize(size);   // (the live binding lays out at its surface's size: flutter_test_config.dart)
  await t.pumpWidget(
    WidgetsApp(
      color: Color(0xFF000000),
      builder: (_, _) => WsScope(
        ws: ws,
        child: Align(alignment: Alignment.topLeft, child: SizedBox(width: size.width, height: size.height, child: WsTimeline())),
      ),
    ),
  );
  await t.pumpAndSettle();
}

/// Where a frame of a thing's row lands on screen at Fit zoom.
Offset _at(Ws ws, int thing, num frame, {Size size = _ref}) {
  final left = WtM.label + WtM.divider;
  final ppf = (size.width - left - WtM.padL - WtM.padR) / Ws.duration, row = ws.layers.indexWhere((l) => l.id == '$thing');
  return Offset(left + WtM.padL + frame * ppf, WtM.header + WtM.ruler + row * WtM.row + WtM.row / 2 - .5);
}

void main() {
  testWidgets('the timeline lays out at the reference size and smaller', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    expect(t.takeException(), isNull);
    await _pump(t, j.ws, size: const Size(900, 220));
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a label or a bar picks its thing', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    await t.tap(find.byKey(ValueKey('ws-tl-label-${j.ring}')));
    expect(j.ws.selected, '${j.ring}');
    await t.tapAt(_at(j.ws, j.rect, 72)); // (the rect's bar, between its keys)
    expect(j.ws.selected, '${j.rect}');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a key click picks the key, shift adds one, an empty track clears', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    final f = (.8 * Ws.fps).round(), g = (1.6 * Ws.fps).round();
    await t.tapAt(_at(j.ws, j.rect, f));
    expect(j.ws.keys, {(layer: '${j.rect}', frame: f)});
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.tapAt(_at(j.ws, j.rect, g));
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    expect(j.ws.keys.length, 2);
    await t.tapAt(_at(j.ws, j.ring, 10)); // (the ring's row, before it comes on)
    expect(j.ws.keys, isEmpty);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a click on the ruler plays from there and a drag scrubs the moment', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    final y = WtM.header + WtM.ruler - 6;
    await t.tapAt(Offset(_at(j.ws, j.rect, 50).dx, y));
    await t.pump(const Duration(milliseconds: 400));   // (a click is known once no second click came)
    expect(j.ws.playing, isTrue, reason: 'a click plays from there, as Live\'s scrub area (where it plays from is the clock\'s: none moves here)');
    j.ws.playing = false;
    await t.pump(const Duration(seconds: 1));
    final g = await t.startGesture(Offset(_at(j.ws, j.rect, 20).dx, y));
    for (final f in const [30, 55, 80]) {   // (a hand moves in steps: the first past the slop starts the drag, the rest carry it)
      await g.moveTo(Offset(_at(j.ws, j.rect, f).dx, y));
      await t.pump();
    }
    await g.up();
    await t.pump();
    expect(j.ws.frame, 80);
    // (the inside's moment is told with each picture it draws (BRIDGE.md rule 3, the moment's copy): none is drawn here, no live picture)
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the play button starts and stops playing (space is the window\'s key: keys.dart play.toggle)', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    await t.tap(find.byKey(const ValueKey('ws-tl-play')));
    expect(j.ws.playing, isTrue);
    await t.tap(find.byKey(const ValueKey('ws-tl-play')));
    expect(j.ws.playing, isFalse);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the transport steps and the zoom buttons', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    j.ws.frame = 30;
    await t.pump();
    await t.tap(find.byKey(const ValueKey('ws-tl-fwd')));
    expect(j.ws.frame, 31);
    await t.tap(find.byKey(const ValueKey('ws-tl-back')));
    await t.tap(find.byKey(const ValueKey('ws-tl-back')));
    expect(j.ws.frame, 29);
    await t.tap(find.byKey(const ValueKey('ws-tl-start')));
    expect(j.ws.frame, 0);
    await t.tap(find.byKey(const ValueKey('ws-tl-zoom-in')));
    await t.tap(find.byKey(const ValueKey('ws-tl-zoom-in')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('ws-tl-fit')));
    await t.pump();
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
