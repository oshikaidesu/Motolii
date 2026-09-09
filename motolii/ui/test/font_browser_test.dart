import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/font_browser.dart';
import '../lib/panels/inspector.dart';
import '../lib/panels/browser.dart';
import '../lib/foundation/theme.dart';
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
    'Inspector color reveals the existing Colors panel for its exact slot',
    (tester) async {
      final sent = <Map<String, dynamic>>[];
      final placements = <dynamic>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            if (call.method == 'placePanel') placements.add(call.arguments);
            if (call.arguments is Map && call.arguments['command'] is String)
              sent.add(jsonDecode(call.arguments['command']));
            return <String, dynamic>{};
          });
      final c = EditorSession();
      final slot = {
        'TextFill': {'layer': 7, 'style': 0},
      };
      c.document.value = {
        'layers': [],
        'assets': [],
        'palette': [],
        'capabilities': ['focusColor', 'setColor'],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 300,
                child: EditorColorRow(
                  controller: c,
                  layer: {'id': 7},
                  color: {
                    'label': 'Fill',
                    'rgba': [1.0, 0.0, 0.0, 1.0],
                    'slot': slot,
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byTooltip('Choose Fill'));
      await tester.pumpAndSettle();
      expect(sent.single, {'op': 'focusColor', 'layer': 7, 'slot': slot});
      expect(c.browserTab.value, 'Colors');
      expect(placements.single, {'name': 'Colors', 'placement': 'show'});
      expect(find.byType(BrowserPanel), findsNothing);
      expect(find.byType(MenuAnchor), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
