// A trackpad's two fingers and pinch, as Flutter hands them over (pan/zoom events, not a wheel's scroll:
// docs.flutter.dev/release/breaking-changes/trackpad-gestures), mean what Apple says: two fingers move what is shown, a
// pinch zooms it. On the timeline and the stage they do, and they never take hold of a thing as a mouse's drag would.
import 'dart:io';

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart' show MethodChannel;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/stage.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/timeline_paint.dart';

import 'door.dart';

Ws _joined(String name) {
  final m = Motolii.open('${Directory.systemTemp.createTempSync(name).path}/$name.motolii')!;
  addTearDown(m.close);
  return Ws()..join(m);
}

Future<void> _pump(WidgetTester t, Ws ws, Size size, Widget seat) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.binding.setSurfaceSize(size);
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);
  await t.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(ws: ws, child: Align(alignment: Alignment.topLeft, child: SizedBox(width: size.width, height: size.height, child: seat))),
    ),
  );
  await t.pumpAndSettle();
}

/// Two fingers on the trackpad at [at]: moved by [by] in ten steps, or spread to [scale] times.
Future<void> _fingers(WidgetTester t, Offset at, {Offset by = Offset.zero, double scale = 1}) async {
  final p = TestPointer(9, PointerDeviceKind.trackpad);
  await t.sendEventToBinding(p.panZoomStart(at));
  for (var i = 1; i <= 10; i++) {
    await t.sendEventToBinding(p.panZoomUpdate(at, pan: by * (i / 10), scale: 1 + (scale - 1) * i / 10));
  }
  await t.sendEventToBinding(p.panZoomEnd());
  await t.pump();
}

const _tl = Size(1226, 300);

/// The frame the ruler is at, at x: a hand comes down there and drags a little away and back (a drag scrubs; a click
/// would play, as Live's scrub area does).
Future<int> _frameAt(WidgetTester t, Ws ws, double x) async {
  final g = await t.startGesture(Offset(x, WtM.header + WtM.ruler - 6));
  await g.moveBy(const Offset(30, 0));
  await t.pump();
  await g.moveBy(const Offset(-30, 0));
  await t.pump();
  await g.up();
  await t.pump(const Duration(milliseconds: 400));
  return ws.frame;
}

void main() {
  testWidgets('on the timeline, two fingers carry the tracks and a pinch zooms; neither takes hold of a bar', (t) async {
    final ws = _joined('tp_timeline'), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws.reread();
    await _pump(t, ws, _tl, WsTimeline());
    for (var i = 0; i < 3; i++) {
      await t.tap(find.byKey(const ValueKey('ws-tl-zoom-in')));
    }
    await t.pump();
    final over = Offset(700, WtM.header + WtM.ruler + WtM.row / 2);   // (on the rect's bar)
    final before = await _frameAt(t, ws, 700), start = ws.layers.firstWhere((l) => l.id == '$rect').inF;
    await _fingers(t, over, by: const Offset(-300, 0));   // (the fingers go left: later frames come into view)
    final carried = await _frameAt(t, ws, 700);
    expect(carried, greaterThan(before), reason: 'two fingers carry the tracks');
    ws.reread();
    expect(ws.layers.firstWhere((l) => l.id == '$rect').inF, start, reason: 'the bar under the fingers is not slid');
    final a = await _frameAt(t, ws, 400), b = await _frameAt(t, ws, 1000);
    await _fingers(t, over, scale: 2);
    final a2 = await _frameAt(t, ws, 400), b2 = await _frameAt(t, ws, 1000);
    expect(b2 - a2, lessThan(b - a), reason: 'a pinch apart zooms in: fewer frames across the same width');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('on the stage, a pinch steps the zoom and two fingers then move the picture; a thing is not carried', (t) async {
    final ws = _joined('tp_stage'), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws.reread();
    await _pump(t, ws, const Size(924, 616), const WsStage());
    final comp = find.byKey(const ValueKey('ws-stage-comp'));
    expect(ws.viewOf('stage.zoom', 0), 0);
    final at = t.getRect(comp).center;
    await _fingers(t, at, scale: 1.4);
    await t.pumpAndSettle();
    expect(ws.viewOf('stage.zoom', 0), 1, reason: 'a pinch apart steps Fit to 50%');
    final where = t.getRect(comp).topLeft, frame = m.frame(rect);
    await _fingers(t, t.getRect(comp).center, by: const Offset(40, 30));
    await t.pumpAndSettle();
    expect(t.getRect(comp).topLeft - where, const Offset(40, 30), reason: 'the picture goes with the fingers');
    expect(m.frame(rect), frame, reason: 'the thing under the fingers stays where it was');
    await _fingers(t, t.getRect(comp).center, scale: .7);
    await t.pumpAndSettle();
    expect(ws.viewOf('stage.zoom', 0), 0, reason: 'a pinch together steps back to Fit');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
