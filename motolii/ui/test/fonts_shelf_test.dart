import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/browser.dart';
import '../lib/session/editor_session.dart';

/// The Fonts shelf: the machine's families as rows, dressed on the selected
/// text layer's characters; which characters, how big and how aligned are
/// chosen at the top of the shelf, not in the Inspector.
void main() {
  Future<(EditorSession, List<Map<String, dynamic>>)> mount(
    WidgetTester tester,
  ) async {
    tester.view.physicalSize = const Size(520, 640);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
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
          'text': {
            'content': 'Hello',
            'fontFamily': 'Arial',
            'styles': [
              {
                'id': 0,
                'size': 40.0,
                'font': {'family': 'Arial'},
              },
            ],
          },
          'properties': [
            {'id': 'text_justify', 'label': 'Alignment', 'value': 0},
          ],
        },
      ],
      'selectedIds': [7],
      'fontFamilies': ['Arial', 'Georgia'],
      'capabilities': ['setFont', 'styleText', 'setProperty'],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Fonts'),
        ),
      ),
    );
    await tester.pump();
    return (c, sent);
  }

  Finder tile(String family) =>
      find.byKey(ValueKey('browser:Fonts:font:$family'));

  testWidgets('a family row dresses the selected text layer', (tester) async {
    final (c, sent) = await mount(tester);
    await tester.enterText(find.byType(TextField).first, 'geo');
    await tester.pump();
    expect(tile('Arial'), findsNothing);
    await tester.tap(tile('Georgia'));
    await tester.pumpAndSettle();
    expect(sent.single, {
      'op': 'setFont',
      'layer': 7,
      'scope': 'all',
      'family': 'Georgia',
    });
    // Nothing selected: the rows stay, the click does nothing.
    c.document.value = {...c.state, 'selectedIds': <int>[]};
    await tester.pumpAndSettle();
    sent.clear();
    await tester.tap(tile('Georgia'));
    await tester.pumpAndSettle();
    expect(sent, isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('the class being dressed and the alignment are chosen here', (
    tester,
  ) async {
    final (c, sent) = await mount(tester);
    await tester.tap(find.byKey(const ValueKey('fonts:scope')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Hiragana').last);
    await tester.pumpAndSettle();
    expect(c.textStyleTarget.value?['scope'], 'hiragana');
    await tester.tap(tile('Georgia'));
    await tester.pumpAndSettle();
    expect(sent.last['op'], 'setFont');
    expect(sent.last['scope'], 'hiragana');
    await tester.tap(find.byKey(const ValueKey('fonts:justify:2')));
    await tester.pumpAndSettle();
    expect(sent.last, {
      'op': 'setProperty',
      'layer': 7,
      'property': 'text_justify',
      'value': 2,
    });
    expect(find.byType(EditorChoice<String>), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
