// Two fingers on a trackpad over the Stage carry the picture and a pinch zooms it (the real app).

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/timeline/face.dart';
import 'package:motolii_ui/app/main.dart' as app;
import 'package:motolii_ui/stage/panel.dart' show StagePanel;
import 'package:motolii_ui/stage/session.dart' show StageSession;

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('trackpad pan and pinch move and zoom the Stage', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final session = StageSession.of(c, 'User');
    final stage = find.byType(StagePanel).first;
    final at = t.getCenter(stage);
    final size = t.getSize(stage);
    final scale0 = session.scaleIn(size);
    final origin0 = session.originIn(size);

    const pointer = 7;
    await t.sendEventToBinding(PointerPanZoomStartEvent(device: pointer, pointer: pointer, position: at));
    await t.sendEventToBinding(PointerPanZoomUpdateEvent(device: pointer, pointer: pointer, position: at, pan: const Offset(40, 0), panDelta: const Offset(40, 0), scale: 1));
    await t.pump(const Duration(milliseconds: 50));
    expect(session.originIn(size).dx, greaterThan(origin0.dx + 30), reason: 'two fingers carried the picture');
    expect(session.scaleIn(size), scale0, reason: 'a pan alone does not zoom');

    await t.sendEventToBinding(PointerPanZoomUpdateEvent(device: pointer, pointer: pointer, position: at, pan: const Offset(40, 0), panDelta: Offset.zero, scale: 1.5));
    await t.pump(const Duration(milliseconds: 50));
    expect(session.scaleIn(size), greaterThan(scale0 * 1.3), reason: 'a pinch zoomed it');
    await t.sendEventToBinding(PointerPanZoomEndEvent(device: pointer, pointer: pointer, position: at));
    await frames(t, 4);
    expect(c.error.value, isNull);
  });
}
