import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/panels/adjust_panels.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets('records show details and checkpoints flush, save and restore', (
    tester,
  ) async {
    final c = EditorSession();
    final calls = <String>[];
    var entries = <Map<String, dynamic>>[];
    Map<String, dynamic> history() => {
      'records': [
        {
          'title': 'Document saved',
          'detail': '/work/project.rrd',
          'kind': 'save',
          'time': '2026-09-09T01:02:03Z',
        },
      ],
      'checkpoints': entries,
      'reports': [],
    };
    final state = {
      'undo': 0,
      'redo': 0,
      'layers': [],
      'capabilities': ['restoreCheckpoint'],
    };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          calls.add(call.method);
          if (call.method == 'checkpoint') {
            final args = call.arguments as Map;
            entries = [
              {
                'id': 'one',
                'name': args['name'],
                'source': '/work/project.rrd',
                'time': '2026-09-09T01:02:03Z',
              },
            ];
            return history();
          }
          if (call.method == 'restoreCheckpoint')
            return {'status': state, 'history': history()};
          if (call.method == 'render') return state;
          if (call.method == 'history') return history();
          return {};
        });
    c.document.value = state;
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: SizedBox(
            width: 240,
            height: 320,
            child: HistoryPanel(controller: c),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Records'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Document saved'));
    await tester.pumpAndSettle();
    expect(find.textContaining('/work/project.rrd'), findsOneWidget);
    expect(find.text('Copy'), findsOneWidget);
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Checkpoints'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('New checkpoint'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'First idea');
    await tester.pump();
    calls.clear();
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(calls, containsAllInOrder(['flushEditors', 'checkpoint']));
    expect(find.text('First idea'), findsOneWidget);
    await tester.tap(find.byTooltip('Restore checkpoint'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(calls, isNot(contains('restoreCheckpoint')));
    await tester.tap(find.byTooltip('Restore checkpoint'));
    await tester.pumpAndSettle();
    calls.clear();
    await tester.tap(find.text('Restore'));
    await tester.pumpAndSettle();
    expect(
      calls,
      containsAllInOrder([
        'flushEditors',
        'restoreCheckpoint',
        'render',
        'history',
      ]),
    );
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
