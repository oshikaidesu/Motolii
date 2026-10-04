import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/main.dart' as app;
import 'package:motolii_ui/session/editor_session.dart';
import 'package:motolii_ui/timeline/timeline.dart';
/// Frames without pumpAndSettle — the Stage clock keeps running.
Future<void> oracleFrames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

String get _seedScript {
  final fromEnv = Platform.environment['MOTOLII_ORACLE_SEED'];
  if (fromEnv != null && fromEnv.isNotEmpty) return fromEnv;
  return '${Directory.current.path}/integration_test/fixtures/oracle_seed.js';
}

/// Opens the real app and seeds [oracle_seed.js] when the work has fewer than two editable layers.
Future<EditorSession> bootOracleApp(WidgetTester t, {int settleFrames = 60}) async {
  app.main();
  await oracleFrames(t, settleFrames);
  final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
  final editable = c.layers.where((l) => l['kind'] != 'Camera' && l['kind'] != 'Group').length;
  if (editable < 2) {
    final script = _seedScript;
    expect(File(script).existsSync(), isTrue, reason: 'oracle seed at $script');
    await c.command('runScript', {'path': script});
    await oracleFrames(t, 40);
    expect(c.error.value, isNull, reason: 'runScript oracle_seed');
  }
  return c;
}

bool layerHasRepeater(EditorSession c, int layerId) {
  final layer = c.layers.cast<Map<String, dynamic>?>().firstWhere(
        (l) => l!['id'] == layerId,
        orElse: () => null,
      );
  if (layer == null) return false;
  return EditorSession.maps(layer['effects']).any((e) => '${e['pluginId']}' == 'motolii.repeat');
}

/// Same `export` / `exportStatus` path the Export sheet uses, without a save panel.
Future<void> runExport(EditorSession c, WidgetTester t, String path, {int start = 0, int end = 6}) async {
  await c.command('export', {'path': path, 'start': start, 'end': end});
  for (var i = 0; i < 400; i++) {
    await c.command('exportStatus');
    final phase = '${EditorSession.map(c.state['export'])['phase'] ?? ''}';
    if (phase == 'done') return;
    if (phase == 'failed') {
      fail('export failed: ${c.state['export']}');
    }
    if (!['running', 'cancelling', 'starting', ''].contains(phase)) return;
    await oracleFrames(t, 10);
  }
  fail('export timed out: ${c.state['export']}');
}
