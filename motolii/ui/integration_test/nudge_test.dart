// Alt+Shift+Right on the chosen layer moves it on the output in one step; Down chooses the next layer (the real app).
import 'package:flutter/services.dart';
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

  testWidgets('Alt+Shift+Right nudges the chosen layer, and one undo takes it back', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final layer = c.layers.firstWhere((l) => l['kind'] != 'Camera' && l['kind'] != 'Group' && l['locked'] != true);
    final id = layer['id'] as int;
    await c.command('select', {'ids': [id]});
    await frames(t);
    List<double> position() => [
          for (final v in EditorSession.maps(c.layers.firstWhere((l) => l['id'] == id)['properties']).firstWhere((p) => p['id'] == 'position')['value'] as List)
            (v as num).toDouble()
        ];
    final before = position();

    await t.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
    await frames(t);
    expect(c.error.value, isNull);
    final after = position();
    expect(after, isNot(before), reason: 'the layer moved');

    await c.command('undo');
    await frames(t);
    expect(position(), before, reason: 'one key, one undo');

    // Down chooses the next layer in the listed order
    final order = [for (final l in c.layers) l['id'] as int];
    await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await frames(t);
    expect(c.selectedIds, [order[(order.indexOf(id) + 1).clamp(0, order.length - 1)]]);
  });
}
