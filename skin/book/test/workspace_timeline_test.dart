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
        child: Align(alignment: Alignment.topLeft, child: SizedBox(width: size.width, height: size.height, child: Overlay(initialEntries: [OverlayEntry(builder: (_) => WsTimeline())]))),   // (the window brings its own overlay: workspace.dart)
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
  test('an effect\'s value takes keys as any value does: every change keyed, a key there (its first leaving what it was where the keying began); with keys, a key where it is changed, the others kept (After Effects\' rule, doc_put)', () {
    final dir = Directory.systemTemp.createTempSync('fxkeys').path, m = Motolii.open('$dir/k.motolii')!;
    addTearDown(m.close);
    final t = m.add('shape', 'rect'), knob = 'js_am_rays_reach';
    bool at(double s) => m.at(s) && m.still('$dir/at.png');   // (the moment the hand lets go at: the stage's, drawn there; a window draws each moment)
    expect(m.say(t, 'fx', '$motoliiDir/effects/am_rays.wgsl') && at(0), isTrue, reason: m.trouble);
    m.keying(true);
    expect(at(.5) && m.hold(t) && m.carry(knob, .3) && m.letGo(true), isTrue, reason: m.trouble);
    expect(m.keyMoments(t, knob), [0, .5], reason: 'keyed: what it was at 0, 0.3 at 0.5 (golf_noise.md 44: none, before)');
    m.keying(false);
    expect(at(1) && m.hold(t) && m.carry(knob, .6) && m.letGo(true), isTrue, reason: m.trouble);
    expect([m.keyMoments(t, knob), m.value(t, knob, .5)], [[0, .5, 1], .3], reason: 'with keys: one more where it was changed, the one at 0.5 kept (before, all of them were taken away)');
  }, skip: !insideBuilt);

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

  testWidgets('animating, each keyed value has its own lane, as Ableton\'s automation lanes: a point clicked is taken away, the line clicked puts one', (t) async {
    final j = _joined(), m = j.ws.inside!;
    await _pump(t, j.ws);
    final lane = find.byKey(ValueKey('ws-tl-lane-${j.rect}-x'));
    expect(lane, findsNothing, reason: 'not animating: the keys are marks in the bar alone');
    j.ws.animating = true;
    await t.pumpAndSettle();
    expect(find.descendant(of: lane, matching: find.text(j.ws.inside!.shown('x')?.label ?? 'x')), findsOneWidget, reason: 'named as the inspector names it');
    expect(lane, findsOneWidget, reason: 'animating: its keyed value in a lane of its own (Live 12 manual 25.5)');
    expect(find.byKey(ValueKey('ws-tl-lane-${j.rect}-y')), findsNothing, reason: 'a value without keys has none');
    final y = t.getCenter(lane).dy;
    // (the line between two keys goes as their way does, not straight: Ableton draws a curved segment curved, golf_noise.md ⑫)
    expect(m.way(j.rect, 'x', .8, 'Bezier 0 0 0.58 1'), isTrue, reason: m.trouble);
    j.ws.reread();
    await t.pumpAndSettle();
    final tracks = find.byWidgetPredicate((w) => w is CustomPaint && w.painter is WtTracksPainter);
    final row = (t.widget<CustomPaint>(tracks).painter! as WtTracksPainter).rows.firstWhere((r) => r.lane != null && r.name == 'x');
    expect(row.laneWays!.first[4], greaterThan(.3), reason: 'Out rises quickly: a quarter along it is well past a quarter (${row.laneWays!.first})');
    Future<void> until(bool Function() done) async {
      for (var i = 0; i < 100 && !done(); i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
        await t.pump();
      }
    }

    await t.tapAt(Offset(_at(j.ws, j.rect, 48).dx, y));   // (its key at 0.8 s)
    await until(() => m.keyMoments(j.rect, 'x').length == 1);
    expect(m.keyMoments(j.rect, 'x'), [closeTo(1.6, 1e-6)], reason: 'a breakpoint clicked: taken away (25.5.2)');
    await t.tapAt(Offset(_at(j.ws, j.rect, 150).dx, y));   // (on the line: level after its one key)
    await until(() => m.keyMoments(j.rect, 'x').length == 2);
    expect(m.keyMoments(j.rect, 'x'), [closeTo(1.6, 1e-6), closeTo(2.5, 1e-6)], reason: 'the line clicked: a breakpoint put there (25.5.2)');
    j.ws.animating = false;
    await t.pumpAndSettle();
    expect(lane, findsNothing);
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a timeline put away stops hearing the window: played after it is made again, nothing is thrown', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    await t.pumpWidget(const SizedBox());   // (put away: its ticker disposed)
    await _pump(t, j.ws);
    j.ws.playing = true;
    await t.pump();
    j.ws.playing = false;
    await t.pump();
    expect(t.takeException(), isNull, reason: 'the one put away no longer starts its disposed ticker (Ticker.start\'s _startTime == null)');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a click on the ruler puts the playhead there and stops play, and a drag scrubs the moment', (t) async {
    final j = _joined();
    await _pump(t, j.ws);
    final y = WtM.header + WtM.ruler - 6;
    await t.tapAt(Offset(_at(j.ws, j.rect, 50).dx, y));
    await t.pump(const Duration(milliseconds: 400));   // (a click is known once no second click came)
    expect([j.ws.frame, j.ws.playing], [50, false], reason: 'the playhead goes there and stands (Audacity\'s ruler as it comes)');
    j.ws.playing = true;
    await t.pump();
    await t.tapAt(Offset(_at(j.ws, j.rect, 30).dx, y));
    await t.pump(const Duration(milliseconds: 400));
    expect([j.ws.frame, j.ws.playing], [30, false], reason: 'playing, a click puts it there and stops play, as any touch of Alight Motion\'s timeline does (AbsentMotion\'s Timeline.kt)');
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

  testWidgets('the time is typed as Premiere Pro\'s: digits from the right are frames, seconds, minutes; + and - go so far on or back', (t) async {
    expect([wtTimeOf('852', 0, 60), wtTimeOf('8:52', 0, 60), wtTimeOf('1:00:00', 0, 60), wtTimeOf('+30', 502, 60), wtTimeOf('-2', 502, 60), wtTimeOf('go', 0, 60)], [532, 532, 3600, 532, 500, null]);
    final j = _joined();
    await _pump(t, j.ws);
    j.ws.frame = 10;
    await t.pump();
    final words = find.descendant(of: find.byKey(const ValueKey('ws-tl-time-words')), matching: find.byType(EditableText));
    await t.tap(find.byKey(const ValueKey('ws-tl-time')));
    await t.pump();
    await t.enterText(words, '+5');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pump();
    expect(j.ws.frame, 15, reason: '+5 is 5 frames on');
    await t.tap(find.byKey(const ValueKey('ws-tl-time')));
    await t.pump();
    await t.enterText(words, '0');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pump();
    expect([j.ws.frame, find.byKey(const ValueKey('ws-tl-time')).evaluate().length], [15, 1], reason: 'Esc leaves the playhead where it was, and the time is shown again');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
