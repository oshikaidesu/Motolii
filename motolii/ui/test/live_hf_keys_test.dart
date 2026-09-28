import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/keys.dart';
import '../lib/session/editor_session.dart';

void main() {
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      if (call.method == 'placePanel') sent.add({'op': 'placePanel', ...args as Map});
      return <String, dynamic>{};
    });
  });

  testWidgets('keys send the host operations', (tester) async {
    final c = EditorSession()..document.value = {'layers': [{'id': 4}, {'id': 7}], 'selectedIds': [4], 'durationFrames': 90};
    final node = FocusNode();
    late BuildContext ctx;
    final keys = LiveKeys(c, () => ctx);
    await tester.pumpWidget(Builder(builder: (context) {
      ctx = context;
      return Focus(focusNode: node, onKeyEvent: keys.handle, child: const SizedBox());
    }));
    node.requestFocus();
    await tester.pump();

    Future<String?> press(LogicalKeyboardKey k, {bool meta = false, bool shift = false}) async {
      if (meta) await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      if (shift) await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(k);
      if (shift) await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      if (meta) await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await tester.pump();
      return sent.isEmpty ? null : sent.last['op'] as String?;
    }

    expect(await press(LogicalKeyboardKey.keyZ, meta: true), 'undo');
    expect(await press(LogicalKeyboardKey.keyZ, meta: true, shift: true), 'redo');
    expect(await press(LogicalKeyboardKey.delete), 'delete');
    expect(await press(LogicalKeyboardKey.keyD, meta: true), 'duplicate');
    expect(await press(LogicalKeyboardKey.keyM), 'addMarker');
    expect(await press(LogicalKeyboardKey.arrowDown), 'select');
    expect(sent.last['ids'], [7]);
  });

  testWidgets('keys recovered from Classic: Stage view, Alt+arrow nudge, P shows the Inspector', (tester) async {
    final c = EditorSession()
      ..document.value = {
        'layers': [
          {'id': 4, 'corners': [[10, 20], [30, 20], [30, 40], [10, 40]]},
        ],
        'selectedIds': [4],
        'durationFrames': 90,
      };
    final node = FocusNode();
    late BuildContext ctx;
    final keys = LiveKeys(c, () => ctx);
    await tester.pumpWidget(Builder(builder: (context) {
      ctx = context;
      return Focus(focusNode: node, onKeyEvent: keys.handle, child: const SizedBox());
    }));
    node.requestFocus();
    await tester.pump();

    final views = <String?>[];
    c.viewCommand.addListener(() => views.add(c.viewCommand.value));
    for (final k in [LogicalKeyboardKey.digit0, LogicalKeyboardKey.digit1, LogicalKeyboardKey.equal, LogicalKeyboardKey.minus]) {
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(k);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    }
    expect(views.whereType<String>(), ['Fit', 'Actual', 'In', 'Out']);

    sent.clear();
    await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowUp);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
    await tester.pump();
    final gestures = [for (final m in sent) if (m['op'] == 'stageGesture') m];
    expect(gestures.map((m) => m['phase']), ['begin', 'update', 'commit', 'begin', 'update', 'commit']);
    expect(gestures[1]['point'], [11.0, 20.0]);
    expect(gestures[4]['point'], [10.0, 19.0]);

    await tester.sendKeyEvent(LogicalKeyboardKey.keyP);
    await tester.pump();
    // The host answers `placePanel` by calling back `placePanel` on the shell (session_native).
    expect(sent.where((m) => m['op'] == 'placePanel').map((m) => '${m['name']}:${m['placement']}'), contains('Inspector:show'));
    expect(c.focusProperty.value, 'position');
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.pump();
    await tester.pump();
    expect(c.focusProperty.value, 'anchor', reason: 'Shift+A reveals the Anchor');
  });

}
