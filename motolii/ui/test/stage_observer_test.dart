import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/stage.dart';
import '../lib/foundation/theme.dart';

/// rerun 3D view の取説: object をダブルクリックで Focus、背景をダブルクリックで Reset view。
class ObserverSession extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  @override
  Future<void> refreshPreview() async {}
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    commands.add((op, args));
  }
}

void main() {
  testWidgets(
    'double-click focuses the object under the pointer, background resets',
    (tester) async {
      final c = ObserverSession();
      c.document.value = {
        'width': 400,
        'height': 400,
        'frame': 0,
        'documentRevision': 'r1',
        'stageView': 'User',
        'observer': {
          'front': false,
          'orbit': [10, 20],
        },
        'layers': [
          {
            'id': 7,
            'bounds': {
              'corners': [
                [0, 0],
                [200, 0],
                [200, 200],
                [0, 200],
              ],
            },
          },
        ],
        'selectedIds': [],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(
            body: SizedBox(
              width: 464,
              height: 480,
              child: StagePanel(controller: c),
            ),
          ),
        ),
      );
      await tester.pump();
      final stage = tester.getRect(find.byType(StagePanel));
      // The 400×400 composition fits the 464×(480−toolbar) viewport with a 16px margin.
      final inside = stage.topLeft + const Offset(60, 80);
      final outside = stage.topLeft + const Offset(440, 440);
      await tester.tapAt(inside);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(inside);
      await tester.pump();
      expect(c.commands.last, ('stageView', {'focus': 7}));
      await tester.pump(const Duration(seconds: 1));
      await tester.tapAt(outside);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(outside);
      await tester.pump();
      expect(c.commands.last, ('stageView', {'reset': true}));
      expect(find.text('Front'), findsOneWidget);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
