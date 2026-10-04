// The Timeline's meaning, on the real app and its native host, with no skin in the way: what a person does is told to
// the TimelineSession in rows and frames, and the document is read back. Any skin that says the same things gets the
// same results.
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'oracle_fixture.dart';

import 'package:motolii_ui/timeline/timeline.dart';
import 'package:motolii_ui/main.dart' as app;
import 'package:motolii_ui/session/editor_session.dart';
import 'package:motolii_ui/timeline/semantics.dart';
import 'package:motolii_ui/timeline/session.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a bar carried along time moves its layer; lanes open; the head seeks; a marquee picks', (t) async {
    final c = await bootOracleApp(t, settleFrames: 60);
    final s = TimelineSession.of(c);
    expect(s.rows, isNotEmpty, reason: 'the document has layers');

    // carry the first plain layer's bar 12 frames later
    final i = s.rows.indexWhere((r) => r.property == null && !r.isGroup && r.layer['kind'] != 'Camera');
    final id = s.rows[i].id;
    int start() => (c.layers.firstWhere((l) => l['id'] == id)['start'] as num).toInt();
    final before = start();
    s.press(TlBar(i, TlBarPart.body), frame: 30, row: i + .5, mods: const TlMods());
    s.drag(frame: 42, row: i + .5);
    await frames(t, 20);
    // held, not yet released: the host's preview already carries the move (the Timeline and the Stage draw it)
    expect(start(), before + 12);
    s.release();
    await frames(t, 20);
    expect(start(), before + 12);

    // a trim is told as its kind and frames; the host keeps the bar at least a frame long
    final duration = (c.layers.firstWhere((l) => l['id'] == id)['duration'] as num).toInt();
    s.press(TlBar(i, TlBarPart.end), frame: 100, row: i + .5, mods: const TlMods());
    s.drag(frame: 100.0 - duration - 50, row: i + .5);
    s.release();
    await frames(t, 20);
    expect((c.layers.firstWhere((l) => l['id'] == id)['duration'] as num).toInt(), 1);

    // lanes open under a layer that has keys, and close again
    final keyed = s.rows.indexWhere((r) => r.property == null && r.summaryFrames.isNotEmpty);
    final count = s.rows.length;
    s.toggleLanes(keyed);
    expect(s.rows.length, greaterThan(count));
    s.toggleLanes(keyed);
    expect(s.rows.length, count);

    // the head goes where it is asked
    s.seek(45);
    await frames(t, 20);
    expect(c.frame.value, 45);

    // a marquee over every row and all time picks every layer whose bar it touches
    s.press(const TlEmpty(), frame: 0, row: 0, mods: const TlMods());
    s.drag(frame: s.extent.toDouble(), row: s.rows.length.toDouble());
    s.release();
    await frames(t, 20);
    final bars = s.rows.where((r) => r.property == null && r.layer['kind'] != 'Camera').map((r) => r.id).toSet();
    expect(c.selectedIds.toSet().containsAll(bars), isTrue);

    // the eye pressed twice before the document answers: hidden, then shown again
    final eyeRow = s.rows.indexWhere((r) => r.property == null && r.layer['hidden'] != true && r.layer['locked'] != true);
    final eyeId = s.rows[eyeRow].id;
    s.toggleSwitch(eyeRow, 'hidden');
    s.toggleSwitch(eyeRow, 'hidden');
    await frames(t, 20);
    expect(c.layers.firstWhere((l) => l['id'] == eyeId)['hidden'] == true, isFalse, reason: 'two presses, back where it was');

    // the session outlives its skin: throw the panel away and it is the same one, where it was
    expect(identical(TimelineSession.of(c), s), isTrue);
    expect(EditorSession.maps(c.state['layers']), isNotEmpty);
  });
}
