import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

void main() {
  testWidgets('Stops stack under the wheel and save as a gradient swatch', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(320, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['applyPalette', 'setColor', 'setGradient'],
      'selectedIds': [1],
      'layers': [
        {'id': 1, 'kind': 'Solid'},
      ],
      'colorTarget': {
        'rgba': [1, 0, 0, 1],
        'layer': 1,
        'slot': 0,
      },
      'palette': [
        {
          'hex': '#00ff00',
          'rgba': [0, 1, 0, 1],
        },
      ],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Colors'),
        ),
      ),
    );
    // Cards carry no label any more.
    expect(find.text('#00ff00'), findsNothing);

    final add = find.byKey(const ValueKey('browser:stop-add'));
    await tester.tap(add);
    await tester.pump();
    c.document.value = {
      ...c.document.value,
      'colorTarget': {
        'rgba': [0, 0, 1, 1],
        'layer': 1,
        'slot': 0,
      },
    };
    await tester.pump();
    await tester.tap(add);
    await tester.pump();
    expect(find.byKey(const ValueKey('browser:stop:1')), findsOneWidget);

    await tester.tap(find.byKey(const ValueKey('browser:stop-save')));
    await tester.pump(const Duration(seconds: 1));
    final saved = c.deskWork.value['swatches'] as List;
    expect(saved.length, 1);
    expect((saved.single['stops'] as List).length, 2);
    expect(find.byKey(const ValueKey('browser:stop:0')), findsNothing);

    // 色の札は 1 回押すと当たる。保存した gradient なら stops も戻る。
    final card = find.byKey(const ValueKey('browser:Colors:saved:0'));
    expect(card, findsOneWidget);
    await tester.tap(card);
    await tester.pump(const Duration(seconds: 1));
    expect(find.byKey(const ValueKey('browser:stop:1')), findsOneWidget);
  });
}
