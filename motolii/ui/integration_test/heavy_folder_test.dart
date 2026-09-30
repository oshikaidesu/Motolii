// A person drops a big, untidy folder (a Downloads folder: thousands of files, duplicates, every kind) onto the Browser.
// The window must stay alive: no frame of the real app may take more than a moment, in any view, while it is indexed and drawn.
// HEAVY_DIR names the folder (the test is skipped without it).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/browser/media/catalog_session.dart';
import 'package:motolii_ui/timeline/timeline.dart';
import 'package:motolii_ui/main.dart' as app;

const heavy = String.fromEnvironment('HEAVY_DIR');

/// Real seconds of frames; the longest single frame and how many were slow.
Future<({int worst, int slow, int frames})> watch(WidgetTester t, int seconds) async {
  final clock = Stopwatch()..start();
  var worst = 0, slow = 0, frames = 0;
  while (clock.elapsedMilliseconds < seconds * 1000) {
    final one = Stopwatch()..start();
    await t.pump(const Duration(milliseconds: 16));
    final ms = one.elapsedMilliseconds;
    frames++;
    if (ms > worst) worst = ms;
    if (ms > 120) slow++;
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 4)));
  }
  return (worst: worst, slow: slow, frames: frames);
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a big folder dropped on the Browser leaves the window responsive in every view', (t) async {
    app.main();
    await watch(t, 3);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    c.placePanel('Media', 'show');
    await watch(t, 1);
    final s = CatalogSession(c);
    final added = Stopwatch()..start();
    await t.runAsync(() async {
      await s.addSource(heavy, name: 'heavy');
    });
    // ignore: avoid_print
    print('HEAVY addSource returned after ${added.elapsedMilliseconds} ms, ${s.entries.length} entries');
    await t.tap(find.text('All').first);
    final report = <String, ({int worst, int slow, int frames})>{};
    report['thumbnail, while indexing'] = await watch(t, 10);
    await t.tap(find.byKey(const ValueKey('view-list')));
    report['list'] = await watch(t, 6);
    await t.tap(find.byKey(const ValueKey('view-explore')));
    report['explore'] = await watch(t, 10);
    await t.tap(find.byKey(const ValueKey('view-thumbnail')));
    report['thumbnail, settled'] = await watch(t, 6);
    for (final e in report.entries) {
      // ignore: avoid_print
      print('HEAVY ${e.key}: worst frame ${e.value.worst} ms, ${e.value.slow} of ${e.value.frames} frames over 120 ms');
    }
    for (final e in report.entries) {
      expect(e.value.worst, lessThan(1000), reason: '${e.key}: the window froze for ${e.value.worst} ms');
    }
    await t.runAsync(() async {
      await s.removeSource(s.sources.firstWhere((x) => x.name == 'heavy').id);
    });
    s.dispose();
  }, skip: heavy.isEmpty);
}
