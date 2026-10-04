import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/timeline_paint.dart';
import 'package:lab_book/workspace/ws.dart';

const _ref = Size(1226, 300);

Future<Ws> _pump(WidgetTester t, {Size size = _ref}) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  final ws = Ws();
  await t.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(
        ws: ws,
        child: Align(
          alignment: Alignment.topLeft,
          child: SizedBox(width: size.width, height: size.height, child: const WsTimeline()),
        ),
      ),
    ),
  );
  return ws;
}

/// Where a frame of a row lands on screen at Fit zoom.
Offset _at(int row, num frame, {Size size = _ref}) {
  const left = WtM.label + WtM.divider;
  final ppf = (size.width - left - WtM.padL - WtM.padR) / Ws.duration;
  return Offset(left + WtM.padL + frame * ppf, WtM.header + WtM.ruler + row * WtM.row + WtM.row / 2 - .5);
}

void main() {
  testWidgets('lays out at reference and smaller sizes without exceptions', (t) async {
    await _pump(t);
    expect(t.takeException(), isNull);
    await _pump(t, size: const Size(900, 220));
    expect(t.takeException(), isNull);
  });

  testWidgets('label and bar clicks select the layer', (t) async {
    final ws = await _pump(t);
    await t.tap(find.byKey(const ValueKey('ws-tl-label-ring')));
    expect(ws.selected, 'ring');
    await t.tapAt(_at(5, 150)); // chips bar, between keys
    expect(ws.selected, 'chips');
  });

  testWidgets('key clicks pick keys, shift adds, empty track clears', (t) async {
    final ws = await _pump(t);
    await t.tapAt(_at(2, 48));
    expect(ws.keys, {(layer: 'blob', frame: 48)});
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.tapAt(_at(2, 96));
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    expect(ws.keys.length, 2);
    await t.tapAt(_at(3, 10)); // ring row, before its in point
    expect(ws.keys, isEmpty);
  });

  testWidgets('ruler click and drag scrub the frame', (t) async {
    final ws = await _pump(t);
    final y = WtM.header + WtM.ruler - 6;
    await t.tapAt(Offset(_at(0, 150).dx, y));
    expect(ws.frame, 150);
    final g = await t.startGesture(Offset(_at(0, 30).dx, y));
    await g.moveTo(Offset(_at(0, 90).dx, y));
    await g.up();
    expect(ws.frame, 90);
  });

  testWidgets('play button and space toggle playing; the ticker advances and stops', (t) async {
    final ws = await _pump(t);
    final start = ws.frame;
    await t.tap(find.byKey(const ValueKey('ws-tl-play')));
    expect(ws.playing, isTrue);
    await t.pump();
    await t.pump(const Duration(milliseconds: 500));
    expect(ws.frame, greaterThanOrEqualTo(start + 14));
    await t.sendKeyEvent(LogicalKeyboardKey.space);
    expect(ws.playing, isFalse);
    final held = ws.frame;
    await t.pump(const Duration(milliseconds: 500));
    expect(ws.frame, held);
    await t.pump(const Duration(milliseconds: 500));
    expect(t.binding.hasScheduledFrame, isFalse);
  });

  testWidgets('play stops at the end without loop, wraps with loop', (t) async {
    final ws = await _pump(t);
    await t.tap(find.byKey(const ValueKey('ws-tl-loop'))); // loop off
    ws.frame = Ws.duration - 3;
    ws.playing = true;
    await t.pump();
    await t.pump(const Duration(milliseconds: 400));
    expect(ws.playing, isFalse);
    expect(ws.frame, Ws.duration);

    await t.tap(find.byKey(const ValueKey('ws-tl-loop'))); // loop on
    ws.frame = Ws.duration - 3;
    ws.playing = true;
    await t.pump();
    await t.pump(const Duration(milliseconds: 400));
    expect(ws.playing, isTrue);
    expect(ws.frame, lessThan(20));
    ws.playing = false;
    await t.pump();
  });

  testWidgets('transport steps and zoom buttons', (t) async {
    final ws = await _pump(t);
    final f0 = ws.frame;
    await t.tap(find.byKey(const ValueKey('ws-tl-fwd')));
    expect(ws.frame, f0 + 1);
    await t.tap(find.byKey(const ValueKey('ws-tl-back')));
    await t.tap(find.byKey(const ValueKey('ws-tl-back')));
    expect(ws.frame, f0 - 1);
    await t.tap(find.byKey(const ValueKey('ws-tl-start')));
    expect(ws.frame, 0);
    await t.tap(find.byKey(const ValueKey('ws-tl-zoom-in')));
    await t.tap(find.byKey(const ValueKey('ws-tl-zoom-in')));
    await t.pump();
    expect(t.takeException(), isNull);
    await t.tap(find.byKey(const ValueKey('ws-tl-fit')));
    await t.pump();
    expect(t.takeException(), isNull);
  });
}
