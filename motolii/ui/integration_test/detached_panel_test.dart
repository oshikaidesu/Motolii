// A panel detached into its own window: the main session keeps editing the same work.
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'oracle_fixture.dart';

import 'package:motolii_ui/timeline/timeline.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('detaching Timeline leaves the session able to seek the work', (t) async {
    final c = await bootOracleApp(t);
    expect(find.byType(LiveTimeline), findsOneWidget);
    expect(c.detachPanelRequested, isNotNull);
    await c.detachPanelRequested!('Timeline');
    await oracleFrames(t, 40);
    expect(find.byType(LiveTimeline), findsNothing, reason: 'Timeline moved off the main window');
    await c.command('seek', {'frame': 72});
    await oracleFrames(t, 15);
    expect(c.error.value, isNull);
    expect(c.frame.value, 72, reason: 'the same document answers on the main session');
  });
}
