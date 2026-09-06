import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/timeline.dart';

void main() {
  testWidgets('trackpad pan and anchored pinch only change the viewport', (
    tester,
  ) async {
    final calls = <String>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          calls.add(call.method);
          return <String, dynamic>{};
        });
    final controller = EditorSession();
    controller.document.value = {
      'fps': 30,
      'durationFrames': 3000,
      'selectedIds': [1],
      'selectedKeys': [],
      'layers': [
        for (var id = 1; id <= 80; id++)
          {
            'id': id,
            'name': 'Layer $id',
            'kind': 'Shape',
            'start': 0,
            'duration': 3000,
            'parent': null,
            'properties': [],
          },
      ],
    };
    controller.frame.value = 123;
    final original = controller.document.value;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: TimelinePanel(controller: controller)),
      ),
    );
    dynamic painter() => tester
        .widgetList<CustomPaint>(find.byType(CustomPaint))
        .map((w) => w.painter)
        .firstWhere((p) => p.runtimeType.toString() == '_TimelinePainter');
    final initialScale = painter().scale as double;
    await tester.sendEventToBinding(
      const PointerScrollEvent(
        position: Offset(500, 250),
        scrollDelta: Offset(120, 100),
        kind: PointerDeviceKind.mouse,
      ),
    );
    await tester.pump();
    expect(painter().offset, closeTo(120, 0.01));
    final vertical = tester
        .stateList<ScrollableState>(find.byType(Scrollable))
        .firstWhere((s) => s.axisDirection == AxisDirection.down)
        .position;
    expect(vertical.pixels, closeTo(100, 0.01));
    final anchor =
        (painter().offset + 500 - painter().labelWidth) / initialScale;
    final gesture = await tester.createGesture(
      kind: PointerDeviceKind.trackpad,
    );
    await gesture.panZoomStart(const Offset(500, 250));
    await gesture.panZoomUpdate(const Offset(500, 250), scale: 1.1);
    await tester.pump();
    await gesture.panZoomUpdate(const Offset(500, 250), scale: 2);
    await tester.pump();
    await tester.pump();
    expect(painter().scale, closeTo(initialScale * 2, 0.01));
    expect(
      (painter().offset + 500 - painter().labelWidth) / painter().scale,
      closeTo(anchor, 0.01),
    );
    final before = painter().offset as double;
    final beforeY = vertical.pixels;
    await gesture.panZoomUpdate(
      const Offset(500, 250),
      scale: 2,
      pan: const Offset(-80, -40),
    );
    await tester.pump();
    await tester.pump();
    expect(painter().offset, closeTo(before + 80, 0.01));
    expect(vertical.pixels, closeTo(beforeY + 40, 0.01));
    await gesture.panZoomEnd();
    await tester.pump();
    final rulerScrollScale = painter().scale as double;
    final rulerScrollAnchor =
        (painter().offset + 500 - painter().labelWidth) / painter().scale;
    await tester.sendEventToBinding(
      const PointerScrollEvent(
        position: Offset(500, 35),
        scrollDelta: Offset(0, -100),
        kind: PointerDeviceKind.mouse,
      ),
    );
    await tester.pump();
    await tester.pump();
    expect(painter().scale, greaterThan(rulerScrollScale));
    expect(
      (painter().offset + 500 - painter().labelWidth) / painter().scale,
      closeTo(rulerScrollAnchor, .01),
    );
    final rulerScale = painter().scale as double;
    final ruler = await tester.createGesture(kind: PointerDeviceKind.trackpad);
    await ruler.panZoomStart(const Offset(500, 35));
    await ruler.panZoomUpdate(const Offset(500, 35), scale: 1.1);
    await tester.pump();
    await ruler.panZoomUpdate(const Offset(500, 35), scale: 1.5);
    await tester.pump();
    await tester.pump();
    expect(painter().scale, closeTo(rulerScale * 1.5, .01));
    await ruler.panZoomEnd();
    await tester.pump();
    final scrub = await tester.createGesture(kind: PointerDeviceKind.trackpad);
    await scrub.panZoomStart(
      const Offset(500, 35),
      timeStamp: const Duration(milliseconds: 20),
    );
    for (var n = 1; n <= 5; n++) {
      await scrub.panZoomUpdate(
        const Offset(500, 35),
        pan: Offset(0, -n * 3.0),
        timeStamp: Duration(milliseconds: 20 + n * 16),
      );
      await tester.pump(const Duration(milliseconds: 16));
    }
    final scrubRelease = painter().scale as double;
    final scrubY = vertical.pixels;
    await scrub.panZoomEnd(timeStamp: const Duration(milliseconds: 101));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(painter().scale, greaterThan(scrubRelease));
    expect(vertical.pixels, scrubY);
    final fling = await tester.createGesture(kind: PointerDeviceKind.trackpad);
    await fling.panZoomStart(
      const Offset(500, 250),
      timeStamp: const Duration(milliseconds: 100),
    );
    for (var i = 1; i <= 5; i++) {
      await fling.panZoomUpdate(
        const Offset(500, 250),
        pan: Offset(-i * 20.0, -i * 10.0),
        timeStamp: Duration(milliseconds: 100 + i * 16),
      );
      await tester.pump(const Duration(milliseconds: 16));
    }
    final releaseX = painter().offset as double;
    final releaseY = vertical.pixels;
    await fling.panZoomEnd(timeStamp: const Duration(milliseconds: 181));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(painter().offset, greaterThan(releaseX));
    expect(vertical.pixels, greaterThan(releaseY));
    final stop = await tester.createGesture(kind: PointerDeviceKind.trackpad);
    await stop.panZoomStart(const Offset(500, 250));
    await stop.panZoomUpdate(const Offset(500, 250), scale: 1.01);
    await tester.pump();
    await stop.panZoomEnd();
    await tester.pumpAndSettle();
    expect(controller.frame.value, 123);
    expect(identical(controller.document.value, original), isTrue);
    expect(calls, isEmpty);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    controller.dispose();
    await tester.pump();
  });
}
