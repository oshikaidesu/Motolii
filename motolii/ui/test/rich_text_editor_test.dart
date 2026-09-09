import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/rich_text_editor.dart';
import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/session/editor_session.dart';

void main() {
  Future<EditorSession> mount(
    WidgetTester tester,
    List<Map<String, dynamic>> sent,
  ) async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.arguments is Map && call.arguments['command'] is String)
            sent.add(jsonDecode(call.arguments['command']));
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'fontFamilies': ['Arial', 'Georgia'],
      'capabilities': [
        'styleText',
        'previewText',
        'setText',
        'commitPreview',
        'cancelPreview',
      ],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 300,
              child: RichTextEditor(
                controller: c,
                layer: {'id': 1},
                text: {
                  'content': 'あカ漢か\u3099😀ab',
                  'styles': [
                    {
                      'id': 0,
                      'size': 40.0,
                      'font': {'family': 'Arial'},
                    },
                  ],
                  'runs': [],
                  'classes': [
                    'hiragana',
                    'katakana',
                    'han',
                    'hiragana',
                    'other',
                    'latin',
                    'latin',
                  ],
                },
              ),
            ),
          ),
        ),
      ),
    );
    return c;
  }

  testWidgets(
    'selected characters and script groups send explicit formatting scope',
    (tester) async {
      final sent = <Map<String, dynamic>>[];
      final c = await mount(tester, sent);
      final field = find.byKey(const ValueKey('rich-text-content'));
      final text = tester.widget<TextField>(field).controller!;
      text.selection = const TextSelection(baseOffset: 0, extentOffset: 1);
      await tester.pump();
      expect(c.textStyleTarget.value?['scope'], 'selection');
      Future<void> size(String value) async {
        final control = find.byKey(const ValueKey('rich-text-size'));
        await tester.tap(control);
        await tester.pump(const Duration(milliseconds: 50));
        await tester.tap(control);
        await tester.pumpAndSettle();
        await tester.enterText(
          find.descendant(of: control, matching: find.byType(TextField)),
          value,
        );
        await tester.testTextInput.receiveAction(TextInputAction.done);
        await tester.pumpAndSettle();
      }

      await size('80');
      final edit = sent.firstWhere((m) => m['op'] == 'styleText');
      expect(edit['start'], 0);
      expect(edit['end'], 1);
      expect(edit['scope'], 'selection');
      expect(edit['size'], 80);
      await tester.tap(find.byType(EditorChoice<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Katakana').last);
      await tester.pumpAndSettle();
      await size('60');
      expect(
        sent.lastWhere((m) => m['op'] == 'styleText')['scope'],
        'katakana',
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets('IME draft waits for composition and Escape restores the text', (
    tester,
  ) async {
    final sent = <Map<String, dynamic>>[];
    final c = await mount(tester, sent);
    await tester.tap(find.byKey(const ValueKey('rich-text-content')));
    await tester.pump();
    tester.testTextInput.updateEditingValue(
      const TextEditingValue(
        text: 'が',
        selection: TextSelection.collapsed(offset: 1),
        composing: TextRange(start: 0, end: 1),
      ),
    );
    await tester.pump();
    expect(sent.where((m) => m['op'] == 'previewText'), isEmpty);
    tester.testTextInput.updateEditingValue(
      const TextEditingValue(
        text: 'が',
        selection: TextSelection.collapsed(offset: 1),
      ),
    );
    await tester.pumpAndSettle();
    expect(sent.where((m) => m['op'] == 'previewText').last['content'], 'が');
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(sent.last['op'], 'cancelPreview');
    expect(
      tester
          .widget<TextField>(find.byKey(const ValueKey('rich-text-content')))
          .controller!
          .text,
      'あカ漢か\u3099😀ab',
    );
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
  testWidgets('saving flushes focused text once and unregisters the editor', (
    tester,
  ) async {
    final sent = <Map<String, dynamic>>[];
    final c = await mount(tester, sent);
    await tester.tap(find.byKey(const ValueKey('rich-text-content')));
    await tester.enterText(
      find.byKey(const ValueKey('rich-text-content')),
      'New text',
    );
    await tester.pumpAndSettle();
    await c.flushEditors();
    await tester.pumpAndSettle();
    expect(sent.where((m) => m['op'] == 'commitPreview').length, 1);
    expect(
      sent.where((m) => m['op'] == 'previewText').last['content'],
      'New text',
    );
    await tester.pumpWidget(const SizedBox());
    expect(c.pendingEditors, isEmpty);
    c.dispose();
  });
}
