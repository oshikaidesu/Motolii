// Easing one interval names its key: the curve lands there and the selection is left as it was (the real app).
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/timeline/timeline.dart';
import 'package:motolii_ui/main.dart' as app;
import 'package:motolii_ui/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('an eased interval is its key, not the selection', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    // a layer with a keyed property: its first key
    late int layer;
    late String property;
    late int frame;
    outer:
    for (final l in c.layers) {
      for (final p in EditorSession.maps(l['properties'])) {
        final keys = EditorSession.maps(p['keys']);
        if (keys.length >= 2) {
          (layer, property, frame) = (l['id'] as int, '${p['id']}', (keys.first['frame'] as num).round());
          break outer;
        }
      }
    }
    await c.command('select', {'ids': [], 'keys': []});
    await frames(t);
    await c.command('ease', {'kind': 'EasyEase', 'keys': [{'layer': layer, 'property': property, 'frame': frame}]});
    await frames(t);
    expect(c.error.value, isNull);
    final keys = EditorSession.maps(EditorSession.maps(c.layers.firstWhere((l) => l['id'] == layer)['properties']).firstWhere((p) => '${p['id']}' == property)['keys']);
    expect(EditorSession.map(keys.first['interp'])['kind'], isNot('Linear'), reason: 'the interval took the curve');
    expect(c.selectedIds, isEmpty, reason: 'the selection is left as it was');
  });
}
