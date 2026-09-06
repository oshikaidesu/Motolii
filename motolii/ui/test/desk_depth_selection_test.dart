import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/registry.dart';
import '../lib/panels/depth_desk.dart';

void main() {
  testWidgets('Depth owns internal selection until an outside editing action', (
    tester,
  ) async {
    final c = EditorSession();
    var selected = 2;
    final calls = <String>[];
    Map<String, dynamic> state() => {
      'selectedIds': [selected],
      'selectedKeys': [],
      'layers': [
        {'id': 1, 'kind': 'Text', 'name': 'Text'},
        {'id': 2, 'kind': 'Camera', 'name': 'Camera'},
      ],
      'depthLayout': {
        'items': [
          {
            'id': 1,
            'name': 'Text',
            'point': [0.0, 0.0],
            'local': [0.0, 0.0, 0.0],
            'inverseX': [1.0, 0.0, 0.0],
            'inverseZ': [0.0, 0.0, 1.0],
          },
        ],
      },
    };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'request') {
            final cmd = jsonDecode((call.arguments as Map)['command']);
            calls.add(cmd['op']);
            if (cmd['op'] == 'select') selected = (cmd['ids'] as List).first;
          }
          return state();
        });
    c.document.value = state();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Row(
            children: [
              SizedBox(width: 320, child: buildPanel('Desk', c, null)),
              TextButton(
                onPressed: () => c.command('select', {
                  'ids': [1],
                }),
                child: const Text('Outside selection'),
              ),
            ],
          ),
        ),
      ),
    );
    expect(find.byType(DepthDesk), findsOneWidget);
    final marker = find.byTooltip('Text');
    final drag = await tester.startGesture(tester.getCenter(marker));
    await tester.pumpAndSettle();
    expect(c.selectedIds, [1]);
    expect(find.byType(DepthDesk), findsOneWidget);
    await drag.moveBy(const Offset(10, 0));
    await tester.pumpAndSettle();
    expect(find.byType(DepthDesk), findsOneWidget);
    await drag.up();
    await tester.pumpAndSettle();
    expect(calls, ['select', 'previewProperties', 'commitPreview']);
    expect(find.byType(DepthDesk), findsOneWidget);
    await tester.tap(find.text('Outside selection'));
    await tester.pumpAndSettle();
    expect(find.byType(DepthDesk), findsNothing);
    expect(find.text('Tools'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
