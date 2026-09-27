import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new_shell.dart';
import '../lib/app/new/browser/shelf_panel.dart';
import '../lib/app/new/desk/new_desk_host.dart';
import '../lib/app/new/inspector/new_inspector_panel.dart';
import '../lib/panels/stage.dart';
import '../lib/panels/timeline.dart';
import '../lib/session/editor_session.dart';
import 'support/dock_test_utils.dart';
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
    ignoreSqueezedTabChips();
    await tester.pumpWidget(
      MaterialApp(theme: editorTestTheme, home: const NewShell()),
    );
    await tester.pumpAndSettle();

    expect(find.byType(ShelfPanel), findsOneWidget);
    expect(find.byType(StagePanel), findsOneWidget);
    expect(find.byType(NewInspectorPanel), findsOneWidget, reason: 'the New shell draws its own Inspector host now, not the Classic one');
    expect(find.byType(NewDeskHost), findsOneWidget, reason: 'the New shell draws its own Desk host now, not the Classic one');
    expect(find.byType(TimelinePanel), findsOneWidget);
    // Each panel is a dock tab named after it; a strip too narrow for all its
    // tabs keeps the rest off the face, so only the ones in view are asserted.
    for (final tab in ['OBJECTS', 'STAGE', 'CAMERA', 'INSPECTOR', 'TIMELINE', 'DESK'])
      expect(find.text(tab), findsWidgets, reason: tab);

    final dynamic shell = tester.state(find.byType(NewShell));
    final c = shell.c as EditorSession;
    // Settings are read, never written: Classic's layout file stays Classic's.
    expect(c.animateFrom, isFalse);
    expect(calls, isNot(contains('writeSettings')));

    // A panel asking to be shown picks its tab or its drawer.
    await c.panelPlacementRequested!('Fonts', 'show');
    await c.panelPlacementRequested!('Ease', 'show');
    await tester.pumpAndSettle();
    expect(shell.dock.isShown('Fonts'), isTrue);
    expect(c.deskDrawer.value, 'Ease');

    await tester.tap(find.text('CAMERA'));
    await tester.pumpAndSettle();
    expect(shell.dock.isShown('Camera'), isTrue);
    expect(find.byType(StagePanel), findsOneWidget);

    // Closing a panel and asking for it by name brings it back.
    shell.dock.close('Notes');
    await tester.pumpAndSettle();
    expect(shell.dock.isOpen('Notes'), isFalse);
    await c.panelPlacementRequested!('Notes', 'show');
    await tester.pumpAndSettle();
    expect(shell.dock.isShown('Notes'), isTrue);

    // Closing a clean document does not ask.
    expect(await c.confirmClose!(), isTrue);
    await tester.pumpWidget(const SizedBox());
  });
}
