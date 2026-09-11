import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

/// Applicability changes with selection; the catalog stays visible.
void main() {
  Future<EditorSession> mount(WidgetTester tester, List<int> selected) async {
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
      'selectedIds': selected,
      'layers': [
        {'id': 1, 'name': 'Rectangle', 'kind': 'Shape'},
        {'id': 2, 'name': 'Text', 'kind': 'Text'},
      ],
      'catalog': [
        {'id': 'motolii.blur', 'name': 'Blur', 'stage': 'Pass'},
        {'id': 'motolii.pucker_bloat', 'name': 'Pucker & Bloat', 'stage': 'Path'},
        {'id': 'motolii.turbulent_warp', 'name': 'Turbulent Warp 2D', 'stage': 'Warp'},
        {'id': 'motolii.turbulent_displace', 'name': 'Turbulent Displace 3D', 'stage': 'Field'},
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
    return c;
  }

  Finder tile(String id) => find.byKey(ValueKey('browser:Effects:$id'));

  testWidgets('a path effect stays visible when text is selected', (
    tester,
  ) async {
    await mount(tester, [2]);
    expect(tile('motolii.blur'), findsOneWidget);
    expect(tile('motolii.pucker_bloat'), findsOneWidget);
  });

  testWidgets('the same path family remains visible with a shape selected', (
    tester,
  ) async {
    await mount(tester, [1, 2]);
    expect(tile('motolii.pucker_bloat'), findsOneWidget);
  });
  testWidgets('material warp and spatial field have separate visible families', (tester) async {
    await mount(tester, [2]);
    expect(tile('motolii.turbulent_warp'), findsOneWidget);
    expect(tile('motolii.turbulent_displace'), findsOneWidget);
    await tester.tap(find.text('Distort'));
    await tester.pump();
    expect(tile('motolii.turbulent_warp'), findsOneWidget);
    expect(tile('motolii.turbulent_displace'), findsNothing);
    await tester.tap(find.text('3D'));
    await tester.pump();
    expect(tile('motolii.turbulent_warp'), findsNothing);
    expect(tile('motolii.turbulent_displace'), findsOneWidget);
  });

}
