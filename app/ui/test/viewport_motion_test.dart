import 'package:flutter_test/flutter_test.dart';

import '../lib/input/viewport_motion.dart';

void main() {
  for (final mode in ViewportGestureMode.values) {
    testWidgets(
      '$mode preserves its mapping through release and interruption',
      (tester) async {
        var scale = 1.0, x = 0.0, y = 0.0;
        final motion = ViewportMotion((s, h, v) {
          scale = s;
          x = h;
          y = v;
        });
        motion.begin(mode: mode, scale: 1, frame: 300, anchor: 200, y: 100);
        for (var n = 1; n <= 5; n++)
          motion.update(
            Offset(-n * 10.0, -n * 10.0),
            1,
            Duration(milliseconds: n * 16),
          );
        final releaseScale = scale, releaseX = x, releaseY = y;
        motion.end();
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 100));
        expect(x, greaterThan(releaseX));
        switch (mode) {
          case ViewportGestureMode.pan:
            expect(y, greaterThan(releaseY));
            expect(scale, releaseScale);
          case ViewportGestureMode.scrubZoom:
            expect(scale, greaterThan(releaseScale));
            expect(y, 100);
        }
        motion.stop();
        final stopped = (scale, x, y);
        await tester.pump(const Duration(milliseconds: 200));
        expect((scale, x, y), stopped);
        motion.dispose();
      },
    );
  }
  testWidgets('pinch release retains zoom anchor', (tester) async {
    var scale = 1.0, x = 0.0;
    final motion = ViewportMotion((s, h, v) {
      scale = s;
      x = h;
    });
    motion.begin(
      mode: ViewportGestureMode.pan,
      scale: 1,
      frame: 300,
      anchor: 200,
      y: 0,
    );
    for (var n = 1; n <= 5; n++)
      motion.update(Offset.zero, 1 + n * .1, Duration(milliseconds: n * 16));
    final release = scale;
    motion.end();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(scale, greaterThan(release));
    expect((x + 200) / scale, closeTo(300, .0001));
    motion.dispose();
  });
}
