// The Media Browser as a person uses it (the real app): open Explore, pick an asset on the map, keep it, place it from the
// map itself, and find it again under Favorites and Recent. No coordinates: faces are found by the asset they show.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/live_hf/adapters/catalog_session.dart';
import 'package:motolii_stage5/timeline/face.dart';
import 'package:motolii_stage5/app/main.dart' as app;
import 'package:motolii_stage5/session/editor_session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

const _png = [
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0, 0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('Explore -> pick -> keep -> Place from the map, then Favorites and Recent bring it back', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final dir = Directory.systemTemp.createTempSync('motolii-explore.');
    final root = dir.resolveSymbolicLinksSync();
    for (final name in ['cloud_a.png', 'cloud_b.png', 'cloud_c.png', 'tower.png']) {
      File('$root/$name').writeAsBytesSync(_png);
    }
    final s = CatalogSession(c);
    await t.runAsync(() async {
      await s.addSource(root, name: 'explore-test');
      await s.load();
    });
    final ids = {for (final e in s.entries.where((e) => e.json['sourceName'] == 'explore-test')) e.name: e.id};
    expect(ids.length, 4);

    // the Media seat, in Explore
    c.placePanel('Media', 'show');
    await frames(t, 30);
    await t.tap(find.text('All').first);
    await frames(t, 20);
    await t.tap(find.byKey(const ValueKey('view-explore')));
    await frames(t, 40);
    Finder face(String name) => find.byKey(ValueKey('face-${ids[name]}'));
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 800)));
    await frames(t, 20);
    expect(face('cloud_a.png'), findsOneWidget, reason: 'the asset is on the map');
    expect(face('tower.png'), findsOneWidget);

    // pick on the map; the map itself does not move
    final before = t.getTopLeft(face('cloud_b.png'));
    await t.tap(face('cloud_b.png'));
    await frames(t, 10);
    expect(t.getTopLeft(face('cloud_b.png')), before, reason: 'choosing does not move the asset');
    expect(t.getTopLeft(face('tower.png')), isNotNull);

    // F keeps it (one key), and it is saved in the user library by the asset's id
    await t.sendKeyEvent(LogicalKeyboardKey.keyF);
    await frames(t, 10);
    expect(EditorSession.map(c.deskWork.value['collections'])['catalog/${ids['cloud_b.png']}'], 1);

    // and it is on disk with the settings, so it is there when the app is opened again
    await t.runAsync(() async {
      await Future<void>.delayed(const Duration(milliseconds: 400));
      final saved = EditorSession.map(EditorSession.map(await c.native('readSettings'))['deskWork']);
      expect(EditorSession.map(saved['collections'])['catalog/${ids['cloud_b.png']}'], 1);
    });

    // Place from the map: Enter on the chosen asset puts it on the Stage's timeline
    int layers() => EditorSession.maps(c.state['layers']).length;
    final l0 = layers();
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await frames(t, 40);
    expect(c.error.value, isNull);
    expect(layers(), l0 + 1, reason: 'the asset chosen on the map was placed without leaving Explore');
    expect(find.byKey(const ValueKey('view-explore')), findsOneWidget);

    // the way back: Favorites shows what was kept, Recent what was used
    await t.tap(find.text('★ Favorites'));
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 600)));
    await frames(t, 30);
    expect(face('cloud_b.png'), findsOneWidget, reason: 'the kept asset is under Favorites');
    expect(face('tower.png'), findsNothing, reason: 'and only it');
    await t.tap(find.text('Recent'));
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 600)));
    await frames(t, 30);
    expect(face('cloud_b.png'), findsOneWidget, reason: 'the placed asset is under Recent');
    expect(face('cloud_a.png'), findsNothing);
    expect(EditorSession.map(c.deskWork.value['recent'])['catalog'], contains(ids['cloud_b.png']));

    await t.runAsync(() async {
      await s.removeSource(s.sources.firstWhere((x) => x.name == 'explore-test').id);
    });
    s.dispose();
    dir.deleteSync(recursive: true);
  });
}
