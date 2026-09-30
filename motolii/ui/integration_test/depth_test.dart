// A layer carried on the Depth plan: the host turns the plan's movement into its position, one release is one undo
// (the real app and its host).
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/live_hf/adapters/desk_session.dart';
import 'package:motolii_stage5/timeline/face.dart';
import 'package:motolii_stage5/app/main.dart' as app;
import 'package:motolii_stage5/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('dragging a layer on the Depth plan moves it, and one undo takes it back', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final depth = DeskSession.of(c).depth;
    final plan = EditorSession.maps(EditorSession.map(c.state['depthLayout'])['items']);
    final hit = plan.indexWhere((i) => i['locked'] != true);
    expect(hit, isNonNegative, reason: 'a layer to carry is on the plan');
    final id = plan[hit]['id'];
    List<double> position() => [
          for (final v in EditorSession.maps(c.layers.firstWhere((l) => l['id'] == id)['properties']).firstWhere((p) => p['id'] == 'position')['value'] as List)
            (v as num).toDouble()
        ];
    final before = position();

    depth.press(hit);
    depth.drag(hit, 80, 0);
    await frames(t, 10);
    depth.release();
    await frames(t);
    expect(c.error.value, isNull);
    expect(c.selectedIds, [id], reason: 'the press chose it');
    expect(position(), isNot(before), reason: 'the layer moved on the plan');

    await c.command('undo');
    await frames(t);
    expect(position(), before, reason: 'one release, one undo');
  });
}
