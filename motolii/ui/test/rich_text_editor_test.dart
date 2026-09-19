import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/rich_text_editor.dart';
import '../lib/foundation/metrics.dart';
import '../lib/foundation/panel_controls.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';

import '../lib/foundation/leaves.dart';

void main() {
  Future<EditorSession> mount(
    WidgetTester tester,
    List<Map<String, dynamic>> sent, {
    List<Map<String, dynamic>>? styles,
  }) async {
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
        theme: editorTestTheme,
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
                  'styles':
                      styles ??
                      [
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
                    'latin-upper',
                    'latin-lower',
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

  testWidgets('the whole box hands the keys to the text', (tester) async {
    await mount(tester, []);
    final box = find.byType(EditorFieldFrame).first;
    expect(tester.getSize(box).height, EditorMetrics.s96);
    // Below the last line, inside the frame: still the field.
    await tester.tapAt(tester.getBottomLeft(box) - const Offset(-20, 10));
    await tester.pump();
    final field = tester.widget<EditorTextField>(
      find.byKey(const ValueKey('rich-text-content')),
    );
    expect(field.focusNode!.hasFocus, isTrue);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('the box highlights the class the Fonts shelf is dressing', (
    tester,
  ) async {
    final c = await mount(tester, []);
    final field = find.byKey(const ValueKey('rich-text-content'));
    final text = tester.widget<EditorTextField>(field).controller!;
    // A selection in the box is only a caret's business: no span is held.
    text.selection = const TextSelection(baseOffset: 0, extentOffset: 1);
    await tester.pump();
    expect(c.textStyleTarget.value?['scope'], 'all');
    expect(c.textStyleTarget.value?.containsKey('start'), isFalse);
    // The shelf picks the class; the box follows. Case is the other half of
    // a class label: `latin-upper` answers to both `latin` and `upper`.
    c.textStyleTarget.value = {'layer': 1, 'scope': 'upper'};
    await tester.pumpAndSettle();
    expect(
      tester.widget<EditorTextField>(field).controller,
      isA<StyledTextController>().having((t) => t.highlighted, 'highlighted', {
        5,
      }),
    );
    // Typing keeps the shelf's class and carries the draft.
    await tester.enterText(field, 'ABC');
    await tester.pump();
    expect(c.textStyleTarget.value?['scope'], 'upper');
    expect(c.textStyleTarget.value?['text'], 'ABC');
    // No face or size chooser lives in the box any more.
    expect(find.byType(EditorChoice<String>), findsNothing);
    expect(find.byKey(const ValueKey('rich-text-size')), findsNothing);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
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
          .widget<EditorTextField>(
            find.byKey(const ValueKey('rich-text-content')),
          )
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

  testWidgets('the box previews proportion: first style at panel size, runs '
      'at their share, one span per run', (tester) async {
    await mount(
      tester,
      [],
      styles: [
        {
          'id': 0,
          'size': 40.0,
          'font': {'family': 'Arial'},
        },
        {
          'id': 1,
          'size': 20.0,
          'font': {'family': 'Georgia'},
        },
      ],
    );
    final field = find.byKey(const ValueKey('rich-text-content'));
    final box =
        tester.widget<EditorTextField>(field).controller!
            as StyledTextController;
    box.runs = [
      {'len': 5, 'style': 0},
      {'len': 2, 'style': 1},
    ];
    expect(box.previewFamily, 'Arial');
    final span = box.buildTextSpan(
      context: tester.element(field),
      style: const TextStyle(fontSize: EditorMetrics.title),
      withComposing: false,
    );
    expect(span.style?.fontFamily, 'Arial');
    expect(span.style?.fontSize, EditorMetrics.title);
    final runs = span.children!.cast<TextSpan>();
    expect(runs.length, 2, reason: 'one span per run, never per grapheme');
    expect(runs[0].style?.fontSize, EditorMetrics.title);
    expect(runs[1].style?.fontSize, EditorMetrics.title / 2);
    expect(runs[1].style?.fontFamily, 'Georgia');
  });

  testWidgets('a face the machine never listed is not asked for', (
    tester,
  ) async {
    await mount(
      tester,
      [],
      styles: [
        {
          'id': 0,
          'size': 40.0,
          'font': {'family': 'Nowhere Sans'},
        },
      ],
    );
    final box =
        tester
                .widget<EditorTextField>(
                  find.byKey(const ValueKey('rich-text-content')),
                )
                .controller!
            as StyledTextController;
    expect(box.previewFamily, isNull);
  });
}
