import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new_shell.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/desk.dart';
import '../lib/panels/inspector.dart';
import '../lib/panels/stage.dart';
import '../lib/panels/timeline.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';

void main() {
  testWidgets('New shell opens the session with every region on one face', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1600, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final calls = <String>[];
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      calls.add(call.method);
      switch (call.method) {
        case 'windowInfo':
          return {'id': 'main', 'main': true};
        case 'readSettings':
          return {
            'deskWork': {'animateFrom': false},
          };
        case 'attach':
        case 'render':
          return {
            'layers': [],
            'selectedIds': [],
            'selectedKeys': [],
            'capabilities': [],
            'width': 1920,
            'height': 1080,
            'durationFrames': 300,
            'fps': 30,
          };
        default:
          return <String, dynamic>{};
      }
    });
    await tester.pumpWidget(
      MaterialApp(theme: editorTestTheme, home: const NewShell()),
    );
    await tester.pumpAndSettle();

    expect(find.byType(BrowserPanel), findsOneWidget);
    expect(find.byType(StagePanel), findsOneWidget);
    expect(find.byType(InspectorPanel), findsOneWidget);
    expect(find.byType(DeskPanel), findsOneWidget);
    expect(find.byType(TimelinePanel), findsOneWidget);
    for (final tab in ['OBJECTS', 'EFFECTS', 'STAGE', 'CAMERA', 'TIMELINE'])
      expect(find.text(tab), findsOneWidget, reason: tab);

    final dynamic shell = tester.state(find.byType(NewShell));
    final c = shell.c as EditorSession;
    // Settings are read, never written: Classic's layout file stays Classic's.
    expect(c.animateFrom, isFalse);
    expect(calls, isNot(contains('writeSettings')));

    // A panel asking to be shown picks its tab or its drawer.
    await c.panelPlacementRequested!('Fonts', 'show');
    await c.panelPlacementRequested!('Ease', 'show');
    await tester.pumpAndSettle();
    expect(shell.browserTab, 'Fonts');
    expect(c.deskDrawer.value, 'Ease');

    await tester.tap(find.text('CAMERA'));
    await tester.pumpAndSettle();
    expect(shell.centerTab, 'Camera');
    expect(find.byType(StagePanel), findsOneWidget);

    // Closing a clean document does not ask.
    expect(await c.confirmClose!(), isTrue);
    await tester.pumpWidget(const SizedBox());
  });
}
