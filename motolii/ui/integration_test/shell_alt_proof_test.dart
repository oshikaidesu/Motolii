// TEMPORARY — the shell's Skin Swap Proof on the real app: in a tabbed one-pane container, the Create shelf places a
// layer and the Timeline shows it, through the same panels and sessions. Deleted with shell_alt.dart.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/live_hf/adapters/timeline.dart';
import 'package:motolii_stage5/live_hf/main.dart' as app;
import 'package:motolii_stage5/live_hf/shell_alt.dart';
import 'package:motolii_stage5/timeline_core/session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('panels work in a different container', (t) async {
    final semantics = t.ensureSemantics();
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final count = c.layers.length;
    final session = TimelineSession.of(c);
    altShellSkin.value = true;
    await frames(t, 10);
    expect(find.byType(LiveTimeline), findsOneWidget, reason: 'the Timeline tab is shown first');
    expect(identical(TimelineSession.of(c), session), isTrue, reason: 'the session outlived the dock');

    await t.tap(find.byKey(const ValueKey('alt-tab-Create')));
    await frames(t, 20);
    await t.tap(find.bySemanticsLabel('Rectangle').first);
    await frames(t, 30);
    expect(c.layers.length, count + 1, reason: 'the Create shelf placed a layer from the other container');

    await t.tap(find.byKey(const ValueKey('alt-tab-Timeline')));
    await frames(t, 20);
    session.seek(12);
    await frames(t, 20);
    expect(c.frame.value, 12, reason: 'the Timeline works in the tab');
    expect(session.rows.where((r) => r.property == null).length, count + 1);

    altShellSkin.value = false;
    await frames(t, 10);
    expect(find.byType(LiveTimeline), findsOneWidget, reason: 'the dock comes back');
    semantics.dispose();
  });
}
