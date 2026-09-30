// Two New Page presses before the notebook answers make two pages (the host names them) (the real app and its host).
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/desks/hosts.dart';
import 'package:motolii_ui/timeline/timeline_view.dart';
import 'package:motolii_ui/main.dart' as app;
import 'package:motolii_ui/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('two quick New Page presses make two pages', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    int pages() => EditorSession.maps(EditorSession.map(c.state['notebook'])['pages']).length;
    final before = pages();
    final notes = DeskSession.of(c).notes;
    await Future.wait([notes.addPage(), notes.addPage()]);
    await frames(t);
    expect(c.error.value, isNull);
    expect(pages(), before + 2);
  });
}
