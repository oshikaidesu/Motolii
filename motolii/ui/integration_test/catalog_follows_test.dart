// A file added, renamed and removed under a registered folder shows in the Browser's result with nothing asked of it: the
// host's watcher refreshes the index and tells the window, which asks the catalog again (the real app and its host).
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/live_hf/adapters/catalog_session.dart';
import 'package:motolii_stage5/timeline/face.dart';
import 'package:motolii_stage5/app/main.dart' as app;

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('the Browser follows the folder without being asked', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final dir = Directory.systemTemp.createTempSync('motolii-follow.');
    final root = dir.resolveSymbolicLinksSync();
    File('$root/one.png').writeAsBytesSync(List.filled(64, 1));
    File('$root/two.png').writeAsBytesSync(List.filled(64, 2));
    final s = CatalogSession(c);
    await t.runAsync(() async {
      await s.addSource(root, name: 'follow-test');
      await s.load();
    });
    Set<String> names() => {for (final e in s.entries) if (e.json['sourceName'] == 'follow-test') e.name};
    expect(names(), {'one.png', 'two.png'});
    final oneId = s.entries.firstWhere((e) => e.name == 'one.png').id;
    final before = c.catalogTick.value;

    // what Finder would do: nothing goes through the app
    File('$root/three.mov').writeAsBytesSync(List.filled(64, 3));
    File('$root/one.png').renameSync('$root/renamed.png');
    File('$root/two.png').deleteSync();
    for (var i = 0; i < 200 && !(names().contains('three.mov') && names().contains('renamed.png') && !names().contains('two.png')); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
      await t.pump();
    }
    expect(names(), {'renamed.png', 'three.mov'}, reason: 'add, rename and delete all reached the result, unasked');
    expect(c.catalogTick.value, greaterThan(before), reason: 'the host told the window');
    expect(s.entries.firstWhere((e) => e.name == 'renamed.png').id, oneId, reason: 'a rename is the same asset');

    await t.runAsync(() async {
      final mine = s.sources.firstWhere((x) => x.name == 'follow-test');
      await s.removeSource(mine.id);
    });
    s.dispose();
    dir.deleteSync(recursive: true);
  });
}
