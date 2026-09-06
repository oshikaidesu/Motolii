import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/editor_window.dart';
import '../lib/session/editor_session.dart';
import '../lib/workspace/layout.dart';
import '../lib/foundation/panel_catalog.dart';
import '../lib/foundation/theme.dart';

void main() {
  testWidgets('Settings shares placement while only auxiliary tools use Desk', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    var windowId = 0;
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      switch (call.method) {
        case 'windowInfo':
          return {'id': 'main', 'main': true};
        case 'readSettings':
          return {
            'dock': initialDock().json(),
            'panelPlacements': <String, String>{},
          };
        case 'openPanelWindow':
          return {
            'id': 'window-${windowId++}',
            'panels': (call.arguments as Map)['panels'],
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
      MaterialApp(theme: EditorTheme.data, home: const EditorWindow()),
    );
    await tester.pumpAndSettle();
    final dynamic host = tester.state(find.byType(EditorWindow));
    final c = host.c as EditorSession;
    for (final spec in panelCatalog) {
      for (final destination in [
        if (spec.drawer) 'drawer',
        'tab',
        'window',
        'hidden',
        if (spec.drawer) 'drawer',
      ]) {
        final drawerBefore = c.deskDrawer.value;
        final idleBefore = c.deskDefault.value;
        await c.panelPlacementRequested!(spec.name, destination);
        await tester.pumpAndSettle();
        final layout = host.workspace as WorkspaceLayout;
        final tabs = layout.root.leaves
            .expand((n) => n.tabs)
            .where((name) => name == spec.name)
            .length;
        final windows = (host.detached as Map<String, List<String>>).values
            .expand((v) => v)
            .where((name) => name == spec.name)
            .length;
        expect(c.panePlaces.value[spec.name], destination, reason: spec.name);
        expect(tabs, destination == 'tab' ? 1 : 0, reason: spec.name);
        expect(windows, destination == 'window' ? 1 : 0, reason: spec.name);
        expect(c.deskDrawer.value, drawerBefore);
        expect(c.deskDefault.value, idleBefore);
      }
    }
    await tester.pumpWidget(const SizedBox());
  });
}
