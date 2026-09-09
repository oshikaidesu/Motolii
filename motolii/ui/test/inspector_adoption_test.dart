import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/inspector.dart';
import '../lib/session/editor_session.dart';
import '../lib/workspace/layout.dart';

void main() {
  test('saved Test tabs become one Inspector', () {
    final dock = DockNode.read({
      'id': 'right',
      'tabs': ['Inspector', 'Test'],
      'active': 'Test',
    });
    expect(dock.tabs, ['Inspector']);
    expect(dock.active, 'Inspector');
  });

  testWidgets(
    'linked scale preserves ratio, unlink isolates axes, focus reveals',
    (tester) async {
      final commands = <Map<String, dynamic>>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            final args = call.arguments;
            if (args is Map && args['command'] is String)
              commands.add(jsonDecode(args['command'] as String));
            return <String, dynamic>{};
          });
      final c = EditorSession();
      addTearDown(c.dispose);
      c.document.value = {
        'layers': [
          {
            'id': 1,
            'name': 'shape',
            'kind': 'Shape',
            'properties': [
              {
                'id': 'scale',
                'label': 'Scale',
                'kind': 'vec2',
                'value': [2.0, 3.0],
                'keys': [],
              },
              {'id': 'opacity', 'label': 'Opacity', 'value': 1.0, 'keys': []},
            ],
            'effects': [],
          },
        ],
        'selectedIds': [1],
        'selectedKeys': [],
        'capabilities': ['previewProperties', 'commitPreview', 'cancelPreview'],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(body: InspectorPanel(controller: c)),
        ),
      );
      Future<void> typeScale(String value) async {
        final field = find.byKey(const ValueKey('inspector:1:scale:0'));
        await tester.tap(field);
        await tester.pump(const Duration(milliseconds: 50));
        await tester.tap(field);
        await tester.pumpAndSettle();
        await tester.enterText(find.byType(TextField), value);
        await tester.testTextInput.receiveAction(TextInputAction.done);
        await tester.pumpAndSettle();
      }

      await typeScale('400');
      expect(
        FocusManager.instance.primaryFocus?.debugLabel,
        'Inspector scale 0',
      );
      expect(
        commands
            .where((m) => m['op'] == 'previewProperties')
            .last['edits'][0]['value'],
        [4.0, 6.0],
      );
      expect(commands.where((m) => m['op'] == 'commitPreview').length, 1);
      final link = find.byTooltip(
        'Keep the shape: one number scales both axes',
      );
      await tester.tap(link);
      await tester.pump();
      await typeScale('400');
      expect(
        commands
            .where((m) => m['op'] == 'previewProperties')
            .last['edits'][0]['value'],
        [4.0, 3.0],
      );
      c.focusProperty.value = 'scale';
      await tester.pump();
      await tester.pump();
      expect(
        FocusManager.instance.primaryFocus?.debugLabel,
        'Inspector scale 0',
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('declared text and RGBA are editable, including alpha', (
    tester,
  ) async {
    final commands = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String)
            commands.add(jsonDecode(args['command'] as String));
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'layers': [
        {
          'id': 1,
          'name': 'shape',
          'kind': 'Shape',
          'properties': [
            {
              'id': 'caption',
              'label': 'Caption',
              'kind': 'text',
              'value': 'Hello',
            },
            {
              'id': 'tint',
              'label': 'Tint',
              'kind': 'color',
              'value': [1.0, 0.0, 0.0, 0.5],
            },
          ],
          'effects': [],
        },
      ],
      'selectedIds': [1],
      'selectedKeys': [],
      'capabilities': ['previewProperties', 'commitPreview', 'cancelPreview'],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(body: InspectorPanel(controller: c)),
      ),
    );
    for (final entry in {'caption': 'Changed', 'tint': '#00ff0080'}.entries) {
      final field = find.descendant(
        of: find.byKey(ValueKey('1:${entry.key}')),
        matching: find.byType(TextField),
      );
      await tester.ensureVisible(field);
      await tester.enterText(field, entry.value);
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
    }
    final edits = commands
        .where((m) => m['op'] == 'previewProperties')
        .map((m) => m['edits'][0])
        .toList();
    expect(edits[0]['value'], 'Changed');
    expect(edits[1]['value'], [0.0, 1.0, 0.0, 128 / 255]);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('pad keeps only latest pending point and cancels on unmount', (
    tester,
  ) async {
    final gate = Completer<void>();
    final points = <Offset>[];
    var cancelled = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: EditorPad(
              x: 0,
              y: 0,
              size: 100,
              onBegin: () {},
              onPreview: (x, y) async {
                points.add(Offset(x, y));
                await gate.future;
              },
              onFinish: () async {},
              onCancel: () async {
                cancelled++;
              },
            ),
          ),
        ),
      ),
    );
    final pointer = await tester.startGesture(
      tester.getCenter(find.byType(EditorPad)),
    );
    await pointer.moveBy(const Offset(25, 0));
    await tester.pump();
    await pointer.moveBy(const Offset(10, 5));
    await tester.pump();
    await pointer.moveBy(const Offset(10, 5));
    await tester.pump();
    expect(points.length, 1);
    await tester.pumpWidget(const SizedBox());
    gate.complete();
    await tester.pumpAndSettle();
    expect(points.length, 1);
    expect(cancelled, 1);
    await pointer.up();
  });

  for (final cancel in [false, true]) {
    testWidgets(
      'dial coalesces delayed previews and ${cancel ? 'cancels' : 'commits'} after drain',
      (tester) async {
        final gate = Completer<void>();
        final values = <double>[];
        var finished = 0, cancelled = 0;
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: Center(
                child: EditorDial(
                  degrees: 0,
                  size: 100,
                  onBegin: () {},
                  onPreview: (v) async {
                    values.add(v);
                    if (values.length == 1) await gate.future;
                  },
                  onFinish: () async {
                    finished++;
                  },
                  onCancel: () async {
                    cancelled++;
                  },
                ),
              ),
            ),
          ),
        );
        final center = tester.getCenter(find.byType(EditorDial));
        final pointer = await tester.startGesture(center + const Offset(45, 0));
        await pointer.moveTo(center + const Offset(0, 45));
        await tester.pump();
        await pointer.moveTo(center + const Offset(-45, 0));
        await tester.pump();
        await pointer.moveTo(center + const Offset(0, -45));
        await tester.pump();
        await pointer.moveTo(center + const Offset(45, 0));
        await tester.pump();
        expect(
          values.length,
          1,
          reason: 'no FIFO backlog while native is busy',
        );
        if (cancel) {
          await pointer.cancel();
        } else {
          await pointer.up();
        }
        await tester.pump();
        expect(finished + cancelled, 0);
        gate.complete();
        await tester.pumpAndSettle();
        expect(values.length, cancel ? 1 : 2);
        expect(finished, cancel ? 0 : 1);
        expect(cancelled, cancel ? 1 : 0);
      },
    );
  }
}
