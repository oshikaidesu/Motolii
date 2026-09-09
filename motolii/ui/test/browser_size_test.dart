import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

void main() {
  Future<EditorSession> mount(WidgetTester tester) async {
    tester.view.physicalSize = const Size(400, 600);
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
      'capabilities': ['placeAsset'],
      'assets': [
        for (var i = 0; i < 4; i++)
          {'id': 'a$i', 'name': 'clip$i.mp4', 'mime': 'video/mp4'},
      ],
    };
    // These read the grid; Media itself opens on pictures alone.
    c.deskWork.value = {'browserView': 0};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Media'),
        ),
      ),
    );
    return c;
  }

  double cardWidth(WidgetTester tester) => tester
      .getSize(
        find
            .ancestor(of: find.text('clip0'), matching: find.byType(Tooltip))
            .first,
      )
      .width;

  testWidgets('Tile size from settings changes how many fit a row', (
    tester,
  ) async {
    final c = await mount(tester);
    final wide = cardWidth(tester);
    c.deskWork.value = {'browserView': 0, 'browserTile': BrowserSize.min};
    await tester.pump();
    expect(cardWidth(tester), lessThan(wide / 1.5));
    await tester.tap(find.byKey(const ValueKey('browser:tile-larger')));
    await tester.pump(const Duration(seconds: 1));
    // One step is a tenth of the default: 60% + 10% of 120.
    expect(c.deskWork.value['browserTile'], closeTo(84.0, .0001));
  });

  testWidgets('Category rail drags narrower and folds away', (tester) async {
    final c = await mount(tester);
    expect(find.text('MEDIA'), findsOneWidget);
    final grip = find.byKey(const ValueKey('browser:rail-grip'));
    await tester.drag(grip, const Offset(-200, 0));
    await tester.pump();
    expect(c.deskWork.value['browserRail'], 0.0);
    expect(find.text('MEDIA'), findsNothing);
    await tester.tap(find.byKey(const ValueKey('browser:rail-tab')));
    await tester.pump(const Duration(seconds: 1));
    expect(find.text('MEDIA'), findsOneWidget);
  });
}
