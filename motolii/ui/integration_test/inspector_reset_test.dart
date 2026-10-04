// The Inspector's ↺ on Position with two layers chosen: both go back to their default, in one step; a group's
// diamond keys its rows in one step (the real app and its host).
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'oracle_fixture.dart';

import 'package:motolii_ui/inspector/session.dart';
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

  testWidgets('reset and a group key are one step each over the selection', (t) async {
    final c = await bootOracleApp(t, settleFrames: 60);
    final plain = [for (final l in c.layers) if (l['kind'] != 'Camera' && l['kind'] != 'Group' && l['locked'] != true) l['id'] as int];
    final (a, b) = (plain[0], plain[1]);
    await c.command('setProperty', {'layer': a, 'property': 'position', 'value': [120.0, 40.0]});
    await c.command('setProperty', {'layer': b, 'property': 'position', 'value': [0.0, 90.0]});
    await c.command('select', {'ids': [a, b]});
    await frames(t);
    List<double> position(int id) => [
          for (final v in EditorSession.maps(c.layers.firstWhere((l) => l['id'] == id)['properties']).firstWhere((p) => p['id'] == 'position')['value'] as List)
            (v as num).toDouble()
        ];
    final store = InspectorSession.of(c).transform!;

    store.resetMany(['position']);
    await frames(t);
    expect(c.error.value, isNull);
    expect([position(a), position(b)], [[0.0, 0.0], [0.0, 0.0]], reason: 'each chosen layer to its own default, both axes');
    await c.command('undo');
    await frames(t);
    expect(position(b), [0.0, 90.0], reason: 'one reset, one undo');

    int keys(String p) => EditorSession.maps(EditorSession.maps(c.layers.firstWhere((l) => l['id'] == store.activeId)['properties']).firstWhere((r) => r['id'] == p)['keys']).length;
    final before = [keys('position'), keys('scale')];
    store.toggleKeys(['position', 'scale']);
    await frames(t);
    expect([keys('position'), keys('scale')], isNot(before), reason: 'the rows toggled their key here');
    await c.command('undo');
    await frames(t);
    expect([keys('position'), keys('scale')], before, reason: 'one diamond, one undo');
  });
}
