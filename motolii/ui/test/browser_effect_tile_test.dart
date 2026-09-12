import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/native_visual_sample.dart';

/// An effect's tile asks for the effect's snapshot picture and nothing else.
void main() {
  testWidgets('every tile asks for its effect snapshot', (tester) async {
    tester.view.physicalSize = const Size(520, 640);
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
      'capabilities': ['applyEffect'],
      'selectedIds': [1],
      'layers': [
        {'id': 1, 'name': 'Clip', 'kind': 'Media'},
      ],
      'catalog': [
        {'id': 'motolii.blur', 'name': 'Blur', 'stage': 'Pass', 'generation': 3},
      ],
    };
    c.deskWork.value = {'browserView': 0};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Effects'),
        ),
      ),
    );
    final tile = find.byKey(const ValueKey('browser:effect:motolii.blur'));
    expect(tile, findsOneWidget);
    // The glyph stays under the picture, so an effect without one keeps it.
    expect(find.text('ƒ'), findsOneWidget);
    expect(tester.widget<NativeVisualSample>(tile).request, {
      'kind': 'effect',
      'id': 'motolii.blur',
      'generation': 3,
    });
  });
}
