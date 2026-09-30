// A camera box carried on the Stage: the host turns the pointer into the camera's values, one release is one undo
// (the real app and its host).
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/timeline/face.dart';
import 'package:motolii_ui/app/main.dart' as app;
import 'package:motolii_ui/stage/session.dart';
import 'package:motolii_ui/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('carrying a camera box moves its centre, and one undo takes it back', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final had = {for (final l in c.layers) l['id']};
    await c.command('create', {'kind': 'camera'});
    await frames(t);
    expect(c.error.value, isNull);
    final id = c.layers.map((l) => l['id']).firstWhere((i) => !had.contains(i)) as int;
    final s = StageSession.of(c, 'User');
    final camera = s.cameras.firstWhere((g) => g['id'] == id);
    final box = s.cameraBox(camera);
    expect(box, hasLength(4), reason: 'the new camera box is on the Stage');
    final middle = box.reduce((a, b) => a + b) / 4;
    List<double> centre() => [for (final v in EditorSession.maps(c.layers.firstWhere((l) => l['id'] == id)['properties']).firstWhere((p) => p['id'] == 'camera.center')['value'] as List) (v as num).toDouble()];
    final before = centre();

    const none = StMods();
    s.press(StCameraHandle(camera, 'center'), middle, none);
    s.drag(middle + const Offset(40, 0), beyondSlop: true, mods: none, viewScale: 1);
    await frames(t, 10);
    s.release(middle + const Offset(40, 0), none, 1);
    await frames(t);
    expect(c.error.value, isNull);
    final after = centre();
    expect(after[0], greaterThan(before[0] + 1), reason: 'the camera moved with the box');
    expect(after[1], closeTo(before[1], 1e-6));

    await c.command('undo');
    await frames(t);
    expect(centre(), before, reason: 'one release, one undo');
  });
}
