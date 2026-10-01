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
}
