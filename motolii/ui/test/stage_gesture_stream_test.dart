// A Stage drag streams to native as: begin, then only the newest positions, then commit last (never overtaken).
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/stage/session.dart';
import 'package:motolii_ui/session/editor_session.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('begin, latest updates, commit last', (tester) async {
    final sent = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      if (call.method == 'request') {
        final command = jsonDecode(call.arguments['command']) as Map<String, dynamic>;
        if (command['op'] == 'stageGesture') sent.add(command);
        await Future<void>.delayed(const Duration(milliseconds: 5));
      }
      return <String, dynamic>{'needsRender': false};
    });
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['stageGesture', 'select'],
      'selectedIds': [1],
      'selectedId': 1,
      'layers': [{'id': 1}],
    };
    final stage = StageSession.of(c, 'User');
    const mods = StMods();
    // a hover is still in flight when the press comes: begin must not fall behind the updates that follow it
    stage.hover(const Offset(9, 9), 1);
    stage.press(const StHandle('se'), const Offset(10, 10), mods);
    for (var i = 1; i <= 40; i++) {
      stage.drag(Offset(10.0 + i, 10), beyondSlop: true, mods: mods, viewScale: 1);
    }
    stage.release(const Offset(50, 10), mods, 1);
    await tester.pump(const Duration(milliseconds: 500));
    final phases = [for (final m in sent) m['phase']];
    expect(phases.indexOf('begin'), lessThan(phases.indexOf('update')), reason: '$phases');
    expect(phases.first, 'hover');
    expect(phases.last, 'commit');
    final updates = sent.where((m) => m['phase'] == 'update').toList();
    expect(updates.length, lessThan(10), reason: 'not one native request per pointer event');
    expect(updates.last['point'], [50.0, 10.0]);
    c.dispose();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, null);
  });
}
