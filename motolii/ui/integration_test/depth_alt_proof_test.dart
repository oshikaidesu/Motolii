// TEMPORARY — the desks' Skin Swap Proof, driven on the real app: the table skin moves a layer and the camera through
// the same DepthHost. Deleted with depth_alt.dart.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/live_hf/adapters/depth_alt.dart';
import 'package:motolii_stage5/live_hf/adapters/timeline.dart';
import 'package:motolii_stage5/live_hf/main.dart' as app;
import 'package:motolii_stage5/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('the table skin carries a layer and the camera', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    await c.placePanel('Depth', 'show');
    altDepthSkin.value = true;
    await frames(t, 20);
    final plan = EditorSession.maps(EditorSession.map(c.state['depthLayout'])['items']);
    final hit = plan.indexWhere((i) => i['locked'] != true);
    final id = plan[hit]['id'];
    Object? prop(Object? layer, String p) => EditorSession.maps(c.layers.firstWhere((l) => l['id'] == layer)['properties']).firstWhere((r) => r['id'] == p)['value'];
    final before = [prop(id, 'position'), prop(id, 'position.z')];
    await t.drag(find.byKey(ValueKey('alt-depth-$hit')), const Offset(30, 20));
    await frames(t);
    expect(c.error.value, isNull);
    expect([prop(id, 'position'), prop(id, 'position.z')], isNot(before), reason: 'the layer moved on the plan');

    final camera = EditorSession.map(EditorSession.map(c.state['depthLayout'])['camera'])['layer'];
    if (camera != null) {
      final orbit = prop(camera, 'camera.orbit');
      await t.drag(find.byKey(const ValueKey('alt-depth--1')), const Offset(60, 0));
      await frames(t);
      expect(c.error.value, isNull);
      expect(prop(camera, 'camera.orbit'), isNot(orbit), reason: 'the eye went round the target');
    }
    altDepthSkin.value = false;
  });
}
