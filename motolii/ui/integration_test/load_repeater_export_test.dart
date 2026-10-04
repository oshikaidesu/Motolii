// Production path: seed work → save → reopen → Repeater from the Effects shelf → export MP4.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'oracle_fixture.dart';

import 'package:motolii_ui/browser/session.dart' show BrowserSession;
import 'package:motolii_ui/effects/shelf.dart';
import 'package:motolii_ui/session/editor_session.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('save, reopen, apply Repeater from Effects, export', (t) async {
    final semantics = t.ensureSemantics();
    final ffmpeg = await Process.run('sh', ['-c', 'command -v ffmpeg']);
    expect(ffmpeg.exitCode, 0, reason: 'ffmpeg on PATH for export');

    final c = await bootOracleApp(t);
    final dir = Directory.systemTemp.createTempSync('motolii-load-export.');
    final rrd = '${dir.path}/work.rrd';
    final mp4 = '${dir.path}/out.mp4';

    int shapeLayerId() =>
        c.layers.firstWhere((l) => '${l['name']}'.contains('Shape'))['id'] as int;

    await c.command('select', {'ids': [shapeLayerId()]});
    await oracleFrames(t, 15);

    await c.command('save', {'path': rrd});
    await oracleFrames(t, 10);
    expect(File(rrd).existsSync(), isTrue);

    await c.open(rrd);
    await oracleFrames(t, 30);
    expect(c.error.value, isNull);
    expect('${c.state['path']}', rrd);

    final layerId = shapeLayerId();
    await c.command('select', {'ids': [layerId]});
    await c.placePanel('Effects', 'show');
    await oracleFrames(t, 40);

    final browser = BrowserSession.of(c);
    expect(
      browser.bindings['motolii.repeat'],
      isNotNull,
      reason: 'host Repeater wired (hostCapabilities=${EditorSession.maps(c.state['hostCapabilities']).length})',
    );
    expect(find.byType(EffectsPanel), findsOneWidget, reason: 'Effects seat is open');

    final repeater = find.descendant(
      of: find.byType(EffectsPanel),
      matching: find.bySemanticsLabel('Repeater'),
    );
    if (repeater.evaluate().isNotEmpty) {
      await t.tap(repeater.first);
    } else {
      await browser.use('Effects', 'motolii.repeat');
    }
    await oracleFrames(t, 25);
    expect(c.error.value, isNull);
    expect(layerHasRepeater(c, layerId), isTrue, reason: 'Repeater applied from the Effects shelf');

    await runExport(c, t, mp4, start: 0, end: 6);
    expect(File(mp4).existsSync(), isTrue);
    expect(File(mp4).lengthSync(), greaterThan(500), reason: 'a non-empty MP4');
    semantics.dispose();
  });
}
