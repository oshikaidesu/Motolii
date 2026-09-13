import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

/// The effects shelf has a reload button and shows why the shelf refused an effect.
void main() {
  testWidgets('reload asks the engine to read vism/ again', (tester) async {
    tester.view.physicalSize = const Size(520, 640);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final calls = <MethodCall>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          calls.add(call);
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['applyEffect', 'reloadEffects'],
      'selectedIds': [1],
      'layers': [
        {'id': 1, 'name': 'Clip', 'kind': 'Media'},
      ],
      'catalog': [
        {'id': 'motolii.blur', 'name': 'Blur', 'stage': 'Pass', 'generation': 3},
      ],
      'catalogErrors': ['glow: expected ; at line 4'],
    };
    c.deskWork.value = {'browserView': 0};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Effects'),
        ),
      ),
    );
    expect(find.text('Effects: glow: expected ; at line 4'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('browser:effects:reload')));
    await tester.pump();
    final request = calls.where((call) => call.method == 'request');
    expect(request, isNotEmpty);
    expect('${request.last.arguments['command']}', contains('"op":"reloadEffects"'));
  });
}
