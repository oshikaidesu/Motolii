import 'package:flutter/gestures.dart';
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

  testWidgets('A long name slides under the pointer instead of shrinking', (
    tester,
  ) async {
    final c = await mount(tester);
    c.document.value = {
      ...c.document.value,
      'assets': [
        {
          'id': 'long',
          'name': 'a very long name that will not fit in one tile.mp4',
          'mime': 'video/mp4',
        },
      ],
    };
    c.deskWork.value = {'browserView': 0};
    await tester.pumpAndSettle();
    final name = find.text('a very long name that will not fit in one tile');
    expect(name, findsOneWidget);
    final style = tester.widget<Text>(name).style!;
    expect(style.fontSize, 11);
    final before = tester.getTopLeft(name).dx;
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    addTearDown(mouse.removePointer);
    // The pointer rests on the caption line, not on the part that hangs out.
    final line = find.byKey(const ValueKey('browser:name:long'));
    await mouse.moveTo(tester.getCenter(line));
    // The first frame starts the clock; the second is two seconds in.
    await tester.pump();
    await tester.pump(const Duration(seconds: 2));
    expect(tester.getTopLeft(name).dx, lessThan(before));
    // Leaving snaps back at once.
    await mouse.moveTo(Offset.zero);
    await tester.pump();
    expect(tester.getTopLeft(name).dx, closeTo(before, .5));
  });

  testWidgets('Pictures alone: the card under the pointer says its name', (
    tester,
  ) async {
    final c = await mount(tester);
    c.deskWork.value = {};
    await tester.pump();
    expect(find.text('clip1'), findsNothing);
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    addTearDown(mouse.removePointer);
    await mouse.moveTo(
      tester.getCenter(find.byKey(const ValueKey('browser:Media:a1'))),
    );
    await tester.pump();
    expect(find.byKey(const ValueKey('browser:band:a1')), findsOneWidget);
    await mouse.moveTo(Offset.zero);
    await tester.pump();
    expect(find.byKey(const ValueKey('browser:band:a1')), findsNothing);
  });

  testWidgets('Carried files get a drop hint; an import shows its wheel', (
    tester,
  ) async {
    final c = await mount(tester);
    final hint = find.byKey(const ValueKey('browser:drop-hint'));
    expect(
      tester.widget<AnimatedOpacity>(
        find.ancestor(of: hint, matching: find.byType(AnimatedOpacity)),
      ).opacity,
      0,
    );
    c.dragging.value = true;
    await tester.pumpAndSettle();
    expect(
      tester.widget<AnimatedOpacity>(
        find.ancestor(of: hint, matching: find.byType(AnimatedOpacity)),
      ).opacity,
      1,
    );
    expect(find.byKey(const ValueKey('browser:importing')), findsNothing);
    c.importing.value = 3;
    await tester.pump();
    expect(find.byKey(const ValueKey('browser:importing')), findsOneWidget);
    expect(find.text('3'), findsOneWidget);
    c.importing.value = 0;
    await tester.pump();
    expect(find.byKey(const ValueKey('browser:importing')), findsNothing);
  });
}
