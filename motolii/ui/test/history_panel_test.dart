import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/history_records.dart';
import '../lib/panels/registry.dart';
import '../lib/session/editor_session.dart';

/// The Desk is narrow; the column has to hold at this width.
const narrow = 240.0;

Map<String, dynamic> ledger(int head) => {
  'capabilities': ['historyGoto', 'undo', 'redo'],
  'history': {
    'head': head,
    'entries': [
      {
        'id': '1-1',
        'parent': null,
        'head': 0,
        'kind': 'open',
        'label': 'sample.rrd',
        'at': 1.0,
      },
      {
        'id': '1-2',
        'parent': '1-1',
        'head': 1,
        'kind': 'edit',
        'label': 'Create',
        'at': 2.0,
      },
      {
        'id': '1-3',
        'parent': '1-2',
        'head': 1,
        'kind': 'save',
        'label': 'Saved',
        'at': 3.0,
      },
      {
        'id': '1-4',
        'parent': '1-3',
        'head': 2,
        'kind': 'edit',
        'label': 'Set property',
        'at': 4.0,
      },
    ],
  },
};

void main() {
  late List<Map<String, dynamic>> commands;
  late EditorSession session;

  /// The session and the futures it chains have to be born inside the test
  /// body, or pumpAndSettle never drains them.
  Future<void> show(
    WidgetTester tester,
    Map<String, dynamic> state, {
    bool fromRegistry = false,
  }) async {
    commands = [];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'request') {
            final arguments = Map<String, dynamic>.from(call.arguments as Map);
            commands.add(
              Map<String, dynamic>.from(
                jsonDecode('${arguments['command']}') as Map,
              ),
            );
          }
          return <String, dynamic>{};
        });
    session = EditorSession();
    session.document.value = state;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SizedBox(
            width: narrow,
            height: narrow,
            child: fromRegistry
                ? buildPanel('History', session, null)
                : HistoryRecords(controller: session),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('Edits and records share one column', (tester) async {
    await show(tester, ledger(2));
    expect(find.text('Create'), findsOneWidget);
    expect(find.text('Set property'), findsOneWidget);
    expect(find.text('Saved'), findsOneWidget, reason: 'a record, same column');
    expect(find.text('sample.rrd'), findsOneWidget);
  });

  testWidgets('Tapping a point walks the document to it', (tester) async {
    await show(tester, ledger(2));
    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();
    expect(commands.single, {'op': 'historyGoto', 'head': 1});
  });

  testWidgets('The point already under the cursor does not move', (
    tester,
  ) async {
    await show(tester, ledger(2));
    await tester.tap(find.text('Set property'));
    await tester.pumpAndSettle();
    expect(commands, isEmpty);
  });

  testWidgets('The current position is marked and the redo tail is not', (
    tester,
  ) async {
    await show(tester, ledger(1));
    TextStyle styleOf(String label) =>
        tester.widget<Text>(find.text(label)).style!;
    expect(styleOf('Saved').color, isNot(styleOf('Set property').color));
    expect(styleOf('Create').color, isNot(styleOf('Set property').color));

    session.document.value = ledger(2);
    await tester.pumpAndSettle();
    expect(styleOf('Saved').color, styleOf('Create').color);
    expect(styleOf('Set property').color, isNot(styleOf('Create').color));
  });

  testWidgets('An empty ledger says so instead of drawing a line', (
    tester,
  ) async {
    await show(tester, {'capabilities': <String>[]});
    expect(find.text('No history'), findsOneWidget);
  });

  testWidgets('The column is the Desk History panel', (tester) async {
    await show(tester, ledger(2), fromRegistry: true);
    expect(find.byType(HistoryRecords), findsOneWidget);
    expect(find.text('Create'), findsOneWidget);
  });
}
