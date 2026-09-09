import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

/// The Browser tile: a picture, one line of name under it, and the format and
/// the status marks riding on the picture so nothing takes the name's width.
void main() {
  Future<EditorSession> mount(WidgetTester tester) async {
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
      'capabilities': ['placeAsset'],
      'assets': [
        {'id': 'a0', 'name': 'clip0.mp4', 'mime': 'video/mp4'},
        {'id': 'a1', 'name': 'torus.obj', 'path': '/m/torus.obj'},
        {'id': 'a2', 'name': 'sphere.obj', 'path': '/m/sphere.obj'},
        {
          'id': 'a3',
          'name': 'sky.png',
          'path': '/m/sky.png',
          'mime': 'image/png',
          'used': true,
        },
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

  Finder tile(String id) => find.byKey(ValueKey('browser:Media:$id'));
  BoxDecoration frame(WidgetTester tester, String id) =>
      tester
              .widgetList<Container>(
                find.descendant(of: tile(id), matching: find.byType(Container)),
              )
              .first
              .decoration
          as BoxDecoration;

  testWidgets('The extension becomes a quiet badge, the name keeps the width', (
    tester,
  ) async {
    await mount(tester);
    expect(find.text('clip0'), findsOneWidget);
    expect(find.text('clip0.mp4'), findsNothing);
    expect(find.text('MP4'), findsOneWidget);
    // The caption line is as wide as the tile; the badge sits inside it at
    // the right edge, the way AEViewer labels a card.
    final line = find.byKey(const ValueKey('browser:name:a0'));
    // The tile's frame is the only thing between them: one hairline a side.
    expect(tester.getSize(line).width, tester.getSize(tile('a0')).width - 2);
    final badge = tester.getRect(find.byKey(const ValueKey('browser:format:a0')));
    final caption = tester.getRect(line);
    expect(badge.top, greaterThanOrEqualTo(caption.top));
    expect(badge.bottom, lessThanOrEqualTo(caption.bottom));
    expect(badge.right, closeTo(caption.right - 4, .5));
    expect(
      tester.getRect(find.text('clip0')).right,
      lessThanOrEqualTo(badge.left),
    );
  });

  testWidgets('Meshes of different bodies get different marks', (tester) async {
    await mount(tester);
    expect(find.byKey(const ValueKey('browser:shape:torus')), findsOneWidget);
    expect(find.byKey(const ValueKey('browser:shape:sphere')), findsOneWidget);
    expect(find.byIcon(Icons.view_in_ar), findsNothing);
  });

  testWidgets('Being in use and being selected are two different marks', (
    tester,
  ) async {
    await mount(tester);
    expect(find.byKey(const ValueKey('browser:used')), findsOneWidget);
    expect(frame(tester, 'a3').border!.top.color, Colors.transparent);

    await tester.tap(tile('a0'));
    // The tile also listens for a double click, so the single one lands late.
    await tester.pump(const Duration(seconds: 1));
    // The frame marks the selection; the dot stays where it was, on the
    // picture of the item that a layer uses.
    expect(frame(tester, 'a0').border!.top.color, EditorTheme.spatial);
    expect(frame(tester, 'a3').border!.top.color, Colors.transparent);
    expect(find.byKey(const ValueKey('browser:used')), findsOneWidget);
  });

  testWidgets(
    'The count says whether it counts the shelf, the search or the selection',
    (tester) async {
      await mount(tester);
      Text count() =>
          tester.widget<Text>(find.byKey(const ValueKey('browser:count')));
      expect(count().data, '4 items');

      await tester.enterText(find.byType(TextField), 'torus');
      await tester.pump();
      expect(count().data, '1 of 4 shown');

      await tester.enterText(find.byType(TextField), '');
      await tester.pump();
      await tester.tap(tile('a0'));
      await tester.pump(const Duration(seconds: 1));
      expect(count().data, '1 of 4 selected');
    },
  );

  testWidgets('Media opens on pictures alone; the chosen one says its name', (
    tester,
  ) async {
    final c = await mount(tester);
    c.deskWork.value = {};
    await tester.pump();
    expect(find.text('clip0'), findsNothing);
    expect(find.byKey(const ValueKey('browser:format:a0')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('browser:Media:a0')));
    // A single tap lands once the double-tap window has passed.
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.byKey(const ValueKey('browser:band:a0')), findsOneWidget);
    expect(find.text('clip0'), findsOneWidget);
    expect(find.byKey(const ValueKey('browser:band:a1')), findsNothing);
  });
}
