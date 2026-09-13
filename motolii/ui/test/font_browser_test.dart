import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/font_browser.dart';
import '../lib/panels/inspector.dart';
import '../lib/foundation/theme.dart';
import '../lib/foundation/color_field.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets(
    'font shelf searches renderer families and edits the selected text layer',
    (tester) async {
      final sent = <Map<String, dynamic>>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            if (call.arguments is Map && call.arguments['command'] is String)
              sent.add(jsonDecode(call.arguments['command']));
            return <String, dynamic>{};
          });
      final c = EditorSession();
      c.document.value = {
        'layers': [
          {
            'id': 7,
            'kind': 'Text',
            'name': 'Title',
            'text': {'content': 'Hello', 'fontFamily': 'Arial'},
          },
        ],
        'selectedIds': [7],
        'fontFamilies': ['Arial', 'Georgia'],
        'capabilities': ['setFont'],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(body: FontBrowser(controller: c)),
        ),
      );
      await tester.enterText(find.byType(TextField), 'geo');
      await tester.pump();
      expect(find.byKey(const ValueKey('font:Arial')), findsNothing);
      await tester.tap(find.byKey(const ValueKey('font:Georgia')));
      await tester.pumpAndSettle();
      expect(sent.single, {'op': 'setFont', 'layer': 7, 'family': 'Georgia'});
      c.document.value = {...c.state, 'selectedIds': <int>[]};
      await tester.pumpAndSettle();
      sent.clear();
      await tester.tap(find.byKey(const ValueKey('font:Georgia')));
      await tester.pumpAndSettle();
      expect(sent, isEmpty);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  testWidgets(
    'A colour row is a value; its swatch sends the focus to the wheel',
    (tester) async {
      final sent = <Map<String, dynamic>>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            if (call.arguments is Map && call.arguments['command'] is String)
              sent.add(jsonDecode(call.arguments['command']));
            return <String, dynamic>{};
          });
      final c = EditorSession();
      final slot = {
        'ShapeFill': {
          'layer': 7,
          'path': [0],
        },
      };
      c.document.value = {
        'layers': [
          {
            'id': 7,
            'name': 'box',
            'kind': 'Shape',
            'properties': [
              {
                'id': 'shape.fill_color',
                'label': 'Fill',
                'kind': 'color',
                'alpha': false,
                'slot': slot,
                'value': [1.0, 0.0, 0.0, 1.0],
                'keys': const [],
              },
            ],
            'effects': [],
          },
        ],
        'selectedIds': [7],
        'selectedKeys': [],
        'capabilities': [
          'previewProperties',
          'commitPreview',
          'cancelPreview',
          'focusColor',
        ],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(body: InspectorPanel(controller: c)),
        ),
      );
      // The row is the value: a swatch and the hex, no wheel of its own.
      final field = tester.widget<EditorColorField>(
        find.byType(EditorColorField),
      );
      expect(field.allowAlpha, isFalse);
      expect(find.text('#ff0000'), findsOneWidget);
      expect(
        find.descendant(
          of: find.byType(EditorColorField),
          matching: find.byType(Slider),
        ),
        findsNothing,
      );

      await tester.tap(find.byTooltip('Pick Fill in the Browser'));
      await tester.pumpAndSettle();
      expect(sent.last['op'], 'focusColor');
      expect(sent.last['layer'], 7);
      expect(sent.last['property'], 'shape.fill_color');
      expect(c.browserTab.value, 'Colors');

      // Typing a hex writes the property like any other value.
      final hex = find.descendant(
        of: find.byType(EditorColorField),
        matching: find.byType(TextField),
      );
      await tester.enterText(hex, '00ff00');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
      final edit = sent.lastWhere((m) => m['op'] == 'previewProperties');
      expect(edit['edits'][0]['property'], 'shape.fill_color');
      expect(edit['edits'][0]['value'], [0.0, 1.0, 0.0, 1.0]);
      expect(sent.last['op'], 'commitPreview');
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
