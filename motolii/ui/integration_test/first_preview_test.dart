// A number grabbed in the Inspector and moved once: the very first non-zero move already previews on the Stage. Traced
// through pointer down -> begin gesture -> first previewProperties -> native preview -> first render (the real app and its host).
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/timeline/face.dart';
import 'package:motolii_ui/app/main.dart' as app;
import 'package:motolii_ui/session/editor_session.dart';
import 'package:motolii_ui/session/latency_probe.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('the first move of an Inspector number previews on the Stage', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final plain = [for (final l in c.layers) if (l['kind'] != 'Camera' && l['kind'] != 'Group' && l['locked'] != true) l['id'] as int];
    await c.command('select', {'ids': [plain.first]});
    await frames(t, 12);
    LatencyProbe.enable();
    final failures = <String>[];
    Future<void> sweep(String label) async {
    final keys = <String>[
      for (final w in t.widgetList(find.byWidgetPredicate((w) => w.key is ValueKey && '${(w.key as ValueKey).value}'.startsWith('toy-'))))
        '${(w.key as ValueKey).value}'
    ];
    keys.sort((a, b) => (b.contains('effect') ? 1 : 0) - (a.contains('effect') ? 1 : 0)); // effect rows first: a gesture's undo must not reach the effect itself
    // ignore: avoid_print
    print('FIRST [$label] fields on screen: $keys');
    for (final key in keys) {
      for (final dx in [24.0, 6.0]) {
        final field = find.byKey(ValueKey(key));
        if (field.evaluate().isEmpty) continue;
        // ignore: avoid_print
        print('FIRST [$label] trying $key at y=${t.getCenter(field.first).dy}');
        if (t.getCenter(field.first).dy > 560) await t.ensureVisible(field.first); // scrolled into view like a hand would
        await frames(t, 3);
        final centre = t.getCenter(field.first);
        if (centre.dy > 800) continue;
        await frames(t, 4);
        LatencyProbe.events.clear();
        final stageBefore = c.stagePublishedFrames;
        final boundsBefore = '${c.state['selectedBounds']}';
        final g = await t.startGesture(centre, kind: PointerDeviceKind.mouse);
        await t.pump(const Duration(milliseconds: 30));
        await g.moveBy(Offset(dx, 0)); // ONE move
        await frames(t, 4);
        final sent = LatencyProbe.events.where((e) => e.name == 'cmd:previewProperties').length;
        final open = c.state['preview'] == true;
        final stageFrames = c.stagePublishedFrames - stageBefore;
        final drawn = '${c.state['selectedBounds']}' != boundsBefore;
        // ignore: avoid_print
        print('FIRST [$label] $key move=$dx -> previewProperties sent=$sent, host preview open=$open, Stage texture frames published=$stageFrames, first render drew it=$drawn, error=${c.error.value}');
        // the camera's centre and target rows are locked while the camera aims at a layer (nothing moves, by design)
        if ((sent < 1 || !open || stageFrames < 1) && !(label == 'camera' && (key.contains('center') || key.contains('target')))) failures.add('[$label] $key@$dx sent=$sent open=$open');
        await g.up();
        await frames(t, 4);
        await c.command('undo');
        await frames(t, 10);
      }
    }
    }

    await sweep('transform');
    // an effect on the layer: its own parameter rows
    await c.command('applyEffect', {'pluginIds': ['motolii.tint']});
    await frames(t, 12);
    await sweep('with effect');
    // the camera's rows
    final camera = c.layers.where((l) => l['kind'] == 'Camera').toList();
    if (camera.isNotEmpty) {
      await c.command('select', {'ids': [camera.first['id']]});
      await frames(t, 12);
      await sweep('camera');
    }
    // ignore: avoid_print
    print('FIRST failures: $failures');
    expect(failures, isEmpty, reason: 'every row previews on its first non-zero move');
  });
}
