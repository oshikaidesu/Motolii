import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/inspector.dart';

void main() {
  testWidgets('Dragging an effect head reorders the pipeline as moveEffect', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(900, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final commands = <String>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            commands.add(args['command'] as String);
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    Map<String, dynamic> effect(int id, String name) => {
      'id': id,
      'pluginId': 'motolii.$name',
      'name': name,
      'placement': false,
      'params': [
        {'id': 'effect.$id.param.amount', 'label': 'Amount', 'value': 1.0},
      ],
    };
    c.document.value = {
      'layers': [
        {
          'id': 1,
          'name': 'paper',
          'kind': 'Shape',
          'properties': [],
          'effects': [effect(0, 'blur'), effect(1, 'glow'), effect(2, 'tint')],
        },
      ],
      'selectedIds': [1],
      'selectedKeys': [],
      'capabilities': ['moveEffect', 'removeEffect', 'setProperty'],
      'easeKinds': [],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(body: InspectorPanel(controller: c)),
      ),
    );
    final handles = find.byTooltip('Drag to reorder');
    expect(handles, findsNWidgets(3));
    // The last effect's head, carried up past the first two.
    final from = tester.getCenter(handles.at(2));
    final to = tester.getCenter(handles.at(0)) - const Offset(0, 12);
    final gesture = await tester.startGesture(from);
    await tester.pump();
    for (var i = 1; i <= 10; i++) {
      await gesture.moveTo(Offset.lerp(from, to, i / 10)!);
      await tester.pump(const Duration(milliseconds: 16));
    }
    await gesture.up();
    await tester.pumpAndSettle();
    final sent = commands.map(jsonDecode).whereType<Map>().toList();
    expect(
      sent.any(
        (m) =>
            m['op'] == 'moveEffect' &&
            m['layer'] == 1 &&
            m['id'] == 2 &&
            m['to'] == 0,
      ),
      isTrue,
      reason: 'the drop asks native for the final index: $commands',
    );
  });
}
