// The History desk's Undo is ⌘Z: one edit back (the real app and its host).
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'oracle_fixture.dart';

import 'package:motolii_ui/timeline/timeline.dart';
import 'package:motolii_ui/main.dart' as app;

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('Undo on the History desk undoes one edit', (t) async {
    final c = await bootOracleApp(t, settleFrames: 60);
    final layer = c.layers.firstWhere((l) => l['kind'] != 'Camera' && l['kind'] != 'Group' && l['locked'] != true);
    final id = layer['id'] as int;
    bool hidden() => c.layers.firstWhere((l) => l['id'] == id)['hidden'] == true;
    final was = hidden();
    await c.command('toggle', {'layer': id, 'flag': 'hidden'});
    await frames(t);
    expect(hidden(), !was);

    c.placePanel('History', 'show');
    await frames(t, 20);
    // the seat may show the desk as its strip; the button is the one the full desk shows
    t.widget<GestureDetector>(find.byKey(const ValueKey('history-undo'))).onTap!();
    await frames(t);
    expect(c.error.value, isNull);
    expect(hidden(), was, reason: 'one edit back');
  });
}
