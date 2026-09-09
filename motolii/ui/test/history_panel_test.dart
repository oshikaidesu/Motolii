import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/panels/adjust_panels.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets('history jumps both ways and stops when undo is rejected', (
    tester,
  ) async {
    final c = EditorSession();
    var head = 3;
    var reject = false;
    final commands = <String>[];
    Map<String, dynamic> snapshot() => {
      'undo': head,
      'redo': 3 - head,
      'capabilities': ['undo', 'redo'],
      'layers': [],
    };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'request') {
            final args = Map<String, dynamic>.from(call.arguments as Map);
            final command = jsonDecode(args['command']) as Map;
            final op = command['op'] as String;
            commands.add(op);
            if (!reject) head += op == 'undo' ? -1 : 1;
          }
          return snapshot();
        });
    c.document.value = snapshot();
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: SizedBox(
            width: 240,
            height: 240,
            child: HistoryPanel(controller: c),
          ),
        ),
      ),
    );
    await tester.tap(find.text('Edit 1'));
    await tester.pumpAndSettle();
    expect(commands, ['undo', 'undo']);
    expect(head, 1);
    expect(find.text('1 / 3'), findsOneWidget);
    await tester.tap(find.text('Edit 3'));
    await tester.pumpAndSettle();
    expect(head, 3);
    expect(commands, ['undo', 'undo', 'redo', 'redo']);
    reject = true;
    await tester.tap(find.text('History start'));
    await tester.pumpAndSettle();
    expect(commands.length, 5);
    expect(head, 3);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
