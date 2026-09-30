// Browse an external Source, place a file from it, and it is the work's own asset: one step in, one undo out (the real app).
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/live_hf/adapters/catalog_session.dart';
import 'package:motolii_stage5/live_hf/adapters/project_source.dart';
import 'package:motolii_stage5/timeline/face.dart';
import 'package:motolii_stage5/app/main.dart' as app;
import 'package:motolii_stage5/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a file from a Source becomes the work\'s asset when placed, and one undo takes both back', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final dir = Directory.systemTemp.createTempSync('motolii-place.');
    final root = dir.resolveSymbolicLinksSync();
    // a real (tiny) picture: 4x4 PNG
    File('$root/cloud.png').writeAsBytesSync(const [
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0, 0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]);
    final s = CatalogSession(c);
    await t.runAsync(() async {
      await s.addSource(root, name: 'place-test');
      await s.load();
    });
    final entry = s.entries.firstWhere((e) => e.json['sourceName'] == 'place-test');
    int assets() => EditorSession.maps(c.state['assets']).length;
    int layers() => EditorSession.maps(c.state['layers']).length;
    final (a0, l0) = (assets(), layers());
    // browsing changed nothing in the work
    expect(ProjectSource(c, kinds: () => {}, text: () => '').items.where((i) => i.name.startsWith('cloud')), isEmpty);
    await c.command('placeCatalogAsset', {'id': entry.id});
    await frames(t);
    expect(c.error.value, isNull);
    expect((assets(), layers()), (a0 + 1, l0 + 1), reason: 'the first use admits the asset and places a layer');
    expect(ProjectSource(c, kinds: () => {}, text: () => '').items.any((i) => i.name.startsWith('cloud')), isTrue, reason: 'and it now shows under This project');
    await c.command('undo');
    await frames(t);
    expect((assets(), layers()), (a0, l0), reason: 'one undo takes the layer and the admission back together');
    await t.runAsync(() async {
      await s.removeSource(s.sources.firstWhere((x) => x.name == 'place-test').id);
    });
    s.dispose();
    dir.deleteSync(recursive: true);
  });
}
