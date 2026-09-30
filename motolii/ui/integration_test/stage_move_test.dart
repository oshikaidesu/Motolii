// Cmd-click two layers on the Stage and drag one: both move, as the host has them chosen; one release, one undo
// (the real app and its host).
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/timeline/timeline.dart';
import 'package:motolii_ui/main.dart' as app;
import 'package:motolii_ui/stage/session.dart';
import 'package:motolii_ui/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a Cmd-click adds a layer, and a drag carries both', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final s = StageSession.of(c, 'Camera');
    final hands = [for (final l in s.visible) if (StageSession.grabbable(l) && s.corners(l).length >= 3) l];
    expect(hands.length, greaterThanOrEqualTo(2), reason: 'two layers to take on the Stage');
    final (a, b) = (hands[0]['id'] as int, hands[1]['id'] as int);
    List<double> position(int id) => [
          for (final v in EditorSession.maps(c.layers.firstWhere((l) => l['id'] == id)['properties']).firstWhere((p) => p['id'] == 'position')['value'] as List)
            (v as num).toDouble()
        ];
    await c.command('select', {'ids': [a]});
    await frames(t);
    // the Cmd-click, through the session as the skin sends it
    final at = s.corners(hands[1]).reduce((p, q) => p + q) / s.corners(hands[1]).length.toDouble();
    s.press(StLayer(b), at, const StMods(add: true));
    s.release(at, const StMods(add: true), 1);
    await frames(t);
    expect(c.selectedIds.toSet(), {a, b}, reason: 'the host added it');
    final before = [position(a), position(b)];

    s.press(StLayer(b), at, const StMods());
    s.drag(at + const Offset(30, 0), beyondSlop: true, mods: const StMods(), viewScale: 1);
    await frames(t, 10);
    s.release(at + const Offset(30, 0), const StMods(), 1);
    await frames(t);
    expect(c.error.value, isNull);
    expect(position(a), isNot(before[0]), reason: 'the other chosen layer moved too');
    expect(position(b), isNot(before[1]));

    await c.command('undo');
    await frames(t);
    expect([position(a), position(b)], before, reason: 'one release, one undo');
  });
}
