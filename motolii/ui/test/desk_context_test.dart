import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/desk.dart';
import '../lib/panels/registry.dart';
import '../lib/panels/inspector.dart';

void main() {
  testWidgets(
    'Desk reads selection and editing focus without caller-owned tools',
    (tester) async {
      tester.view.physicalSize = const Size(1400, 1000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final calls = <String>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            calls.add(call.method);
            return <String, dynamic>{};
          });
      final c = EditorSession();
      final layers = [
        {
          'id': 1,
          'name': 'Shape',
          'kind': 'Shape',
          'blendMode': 'Multiply',
          'properties': [
            {
              'id': 'position',
              'kind': 'vec2',
              'value': [0.0, 0.0],
              'keys': [
                {
                  'frame': 30,
                  'interp': {'kind': 'Linear'},
                },
                {
                  'frame': 0,
                  'interp': {
                    'kind': 'Bezier',
                    'x1': 0.42,
                    'y1': 0.0,
                    'x2': 1.0,
                    'y2': 1.0,
                  },
                },
              ],
            },
          ],
          'effects': [],
        },
        {
          'id': 2,
          'name': 'Camera',
          'kind': 'Camera',
          'properties': [],
          'effects': [],
        },
        {
          'id': 3,
          'name': 'Text',
          'kind': 'Text',
          'properties': [],
          'effects': [],
        },
      ];
      void select(List<int> ids, [List<Map<String, dynamic>> keys = const []]) {
        c.document.value = {
          'layers': layers,
          'selectedIds': ids,
          'selectedKeys': keys,
          'capabilities': [],
          'easeKinds': [],
        };
      }

      select([1]);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Row(
              children: [
                Expanded(child: InspectorPanel(controller: c)),
                Expanded(child: buildPanel('Desk', c, null)),
              ],
            ),
          ),
        ),
      );
      Finder deskText(String text) => find.descendant(
        of: find.byType(DeskPanel),
        matching: find.text(text),
      );
      expect(deskText('Tools'), findsOneWidget);
      await tester.tap(
        find.descendant(
          of: find.byType(InspectorPanel),
          matching: find.text('Multiply'),
        ),
      );
      await tester.pumpAndSettle();
      expect(deskText('Blend'), findsOneWidget);
      expect(c.deskDrawer.value, isNull);
      c.deskDrawer.value = 'History';
      await tester.pumpAndSettle();
      expect(deskText('History'), findsOneWidget);
      await tester.tap(
        find.descendant(
          of: find.byType(InspectorPanel),
          matching: find.text('Multiply'),
        ),
      );
      await tester.pumpAndSettle();
      expect(deskText('Blend'), findsOneWidget);
      select(
        [1],
        [
          {'layer': 1, 'property': 'position', 'frame': 0},
          {'layer': 1, 'property': 'position', 'frame': 30},
        ],
      );
      await tester.pumpAndSettle();
      expect(deskText('Ease'), findsOneWidget);
      expect(deskText('Apply'), findsOneWidget);
      expect(deskText('x1'), findsOneWidget);
      expect(deskText('0.42'), findsOneWidget);
      c.deskDrawer.value = 'History';
      await tester.pumpAndSettle();
      c.document.value = {...c.state, 'frame': 12};
      await tester.pumpAndSettle();
      expect(deskText('History'), findsOneWidget);
      select([2]);
      await tester.pumpAndSettle();
      expect(deskText('Depth'), findsOneWidget);
      c.focusEditing(2, 'position');
      await tester.pumpAndSettle();
      expect(deskText('Depth'), findsOneWidget);
      select([3]);
      await tester.pumpAndSettle();
      expect(deskText('Tools'), findsOneWidget);
      select([]);
      await tester.pumpAndSettle();
      expect(deskText('Tools'), findsOneWidget);
      c.focusEditing(2, 'position');
      await tester.pumpAndSettle();
      expect(deskText('Tools'), findsOneWidget);
      expect(calls.where((method) => method == 'request'), isEmpty);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
