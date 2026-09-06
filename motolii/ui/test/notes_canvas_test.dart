import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/notes_desk.dart';

void main() {
  testWidgets(
    'freeform text, move, page switching and references use document commands',
    (tester) async {
      final c = EditorSession();
      final pages = <Map<String, dynamic>>[];
      final calls = <Map<String, dynamic>>[];
      Map<String, dynamic> snapshot() => {
        'notebook': {'pages': jsonDecode(jsonEncode(pages))},
        'selectedIds': [1],
        'selectedKeys': [
          {'layer': 1, 'property': 'scale', 'frame': 12},
          {'layer': 1, 'property': 'scale', 'frame': 30},
        ],
        'layers': [
          {'id': 1, 'name': 'Cube', 'kind': 'Mesh'},
        ],
        'capabilities': ['notes'],
      };
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            if (call.method == 'request') {
              final command = Map<String, dynamic>.from(
                jsonDecode((call.arguments as Map)['command']),
              );
              calls.add(command);
              if (command['op'] == 'notes') {
                if (command['action'] == 'addPage') {
                  pages.add({
                    'id': command['page'],
                    'title': command['title'],
                    'blocks': <Map<String, dynamic>>[],
                  });
                } else {
                  final page = pages.firstWhere(
                    (p) => p['id'] == command['page'],
                  );
                  final blocks = page['blocks'] as List;
                  switch (command['action']) {
                    case 'putBlock':
                      blocks.add(Map<String, dynamic>.from(command['block']));
                    case 'patchBlock':
                      final block = blocks.firstWhere(
                        (b) => b['id'] == command['id'],
                      );
                      block.addAll(command['patch']);
                    case 'renamePage':
                      page['title'] = command['title'];
                    case 'deleteBlock':
                      blocks.removeWhere((b) => b['id'] == command['id']);
                  }
                }
              }
            }
            return snapshot();
          });
      c.document.value = snapshot();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(width: 640, child: NotesPanel(controller: c)),
          ),
        ),
      );
      final canvas = find.byType(InteractiveViewer);
      await tester.tapAt(tester.getTopLeft(canvas) + const Offset(40, 40));
      await tester.pumpAndSettle();
      expect(pages.length, 1);
      expect((pages.first['blocks'] as List).length, 1);
      final input = find.byWidgetPredicate(
        (w) => w is TextField && w.decoration?.hintText == 'Write a note',
      );
      await tester.enterText(input, 'Lighting and texture references');
      await tester.tap(find.byTooltip('New page'));
      await tester.pumpAndSettle();
      expect(pages.length, 2);
      expect(
        pages.first['blocks'][0]['text'],
        'Lighting and texture references',
      );
      await tester.tap(find.widgetWithText(TextButton, 'Untitled page').first);
      await tester.pumpAndSettle();
      expect(find.text('Lighting and texture references'), findsOneWidget);
      await tester.drag(find.byIcon(Icons.drag_handle), const Offset(70, 90));
      await tester.pumpAndSettle();
      expect(pages.first['blocks'][0]['x'], closeTo(110, 1));
      expect(pages.first['blocks'][0]['y'], closeTo(130, 1));
      await tester.drag(find.byIcon(Icons.south_east), const Offset(40, 30));
      await tester.pumpAndSettle();
      expect(pages.first['blocks'][0]['width'], closeTo(260, 1));
      await tester.tap(find.byTooltip('Link selection'));
      await tester.pumpAndSettle();
      expect(pages.first['blocks'][1]['kind'], 'reference');
      expect(pages.first['blocks'][1]['start'], 12);
      expect(pages.first['blocks'][1]['end'], 30);
      expect(calls.every((call) => call['op'] == 'notes'), isTrue);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
