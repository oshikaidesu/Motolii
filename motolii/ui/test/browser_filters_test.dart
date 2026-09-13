import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/panels/browser.dart';
import '../lib/session/editor_session.dart';

/// Live 12's browser, on the Fonts shelf: filter groups the item's own facts
/// fill, the user's tags and collections kept with the desk, and a filter
/// saved as a label.
void main() {
  Future<(EditorSession, Map<String, dynamic>)> mount(
    WidgetTester tester,
  ) async {
    tester.view.physicalSize = const Size(900, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final settings = <String, dynamic>{};
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'readSettings') return settings;
          if (call.method == 'writeSettings') {
            settings.addAll(Map<String, dynamic>.from(call.arguments as Map));
            return {};
          }
          if (call.arguments is Map && call.arguments['command'] is String) {
            final command = jsonDecode(call.arguments['command']);
            if (command['op'] == 'fontFacts')
              return {
                'facts': [
                  {
                    'family': 'Arial',
                    'styles': 1,
                    'weights': 1,
                    'monospaced': true,
                    'axes': [],
                    'scripts': ['Latin'],
                    'color': false,
                  },
                  {
                    'family': 'Georgia',
                    'styles': 4,
                    'weights': 2,
                    'monospaced': false,
                    'axes': ['wght'],
                    'scripts': ['Latin', 'Cyrillic'],
                    'color': false,
                  },
                ],
              };
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'layers': const [],
      'selectedIds': const <int>[],
      'fontFamilies': ['Arial', 'Georgia'],
      'capabilities': ['setFont', 'create'],
    };
    c.deskWork.value = {'browserRail': 120.0};
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Fonts'),
        ),
      ),
    );
    await tester.pumpAndSettle();
    return (c, settings);
  }

  Finder tile(String family) =>
      find.byKey(ValueKey('browser:Fonts:font:$family'));
  Finder tag(String group, String tag) =>
      find.byKey(ValueKey('browser:filter:$group:$tag'));

  testWidgets('filter groups narrow with AND across and OR within', (
    tester,
  ) async {
    final (c, _) = await mount(tester);
    expect(find.byKey(const ValueKey('browser:filters')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('browser:filters-toggle')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('browser:filters')), findsOneWidget);
    // The item's own facts fill the groups, with what each tag keeps.
    expect(tag('Script', 'Cyrillic'), findsOneWidget);
    await tester.tap(tag('Script', 'Cyrillic'));
    await tester.pumpAndSettle();
    expect(tile('Georgia'), findsOneWidget);
    expect(tile('Arial'), findsNothing);
    // Another group is AND: a mono face among Cyrillic ones is none.
    await tester.tap(tag('Kind', 'Mono'));
    await tester.pumpAndSettle();
    expect(tile('Georgia'), findsNothing);
    expect(find.text('No matches'), findsOneWidget);
    expect(find.text('0 · 2 filters'), findsOneWidget);
    // Clear puts everything back.
    await tester.tap(find.text('Clear'));
    await tester.pumpAndSettle();
    expect(tile('Georgia'), findsOneWidget);
    expect(tile('Arial'), findsOneWidget);
    // A group folds to its name; its chosen tags stay in force and are named.
    await tester.tap(tag('Script', 'Cyrillic'));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('browser:filter-group:Script')));
    await tester.pumpAndSettle();
    expect(tag('Script', 'Cyrillic'), findsNothing);
    expect(find.text('Cyrillic'), findsOneWidget);
    expect(tile('Arial'), findsNothing);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('a row dragged onto a collection joins it', (tester) async {
    final (c, settings) = await mount(tester);
    final from = tester.getCenter(tile('Arial'));
    final to = tester.getCenter(
      find.byKey(const ValueKey('browser:collection:3')),
    );
    final gesture = await tester.startGesture(from);
    await tester.pump(const Duration(milliseconds: 100));
    await gesture.moveTo(from + const Offset(0, 30));
    await tester.pump(const Duration(milliseconds: 100));
    await gesture.moveTo(to);
    await tester.pump(const Duration(milliseconds: 100));
    await gesture.up();
    await tester.pumpAndSettle();
    expect(
      Map<String, dynamic>.from(settings['deskWork'] as Map)['collections'],
      {'Fonts/font:Arial': 3},
    );
    // The row wears the collection's colour as a dot.
    expect(
      find.byKey(const ValueKey('browser:collected:font:Arial')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('browser:collected:font:Georgia')),
      findsNothing,
    );
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('a digit files picked rows in a collection the rail filters by', (
    tester,
  ) async {
    final (c, settings) = await mount(tester);
    await tester.tap(tile('Georgia'));
    await tester.pumpAndSettle();
    await tester.sendKeyEvent(LogicalKeyboardKey.digit2);
    await tester.pumpAndSettle();
    final desk = Map<String, dynamic>.from(settings['deskWork'] as Map);
    expect(desk['collections'], {'Fonts/font:Georgia': 2});
    // A collection row also takes a double-click (rename), so a click lands
    // after the double-tap deadline.
    await tester.tap(find.byKey(const ValueKey('browser:collection:2')));
    expect(find.text('Orange'), findsOneWidget);
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pumpAndSettle();
    expect(tile('Georgia'), findsOneWidget);
    expect(tile('Arial'), findsNothing);
    // The same row again lets the collection go.
    await tester.tap(find.byKey(const ValueKey('browser:collection:2')));
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pumpAndSettle();
    expect(tile('Arial'), findsOneWidget);
    // The context menu offers the collections too, and a way out.
    await tester.tap(tile('Arial'), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Purple').last);
    await tester.pumpAndSettle();
    expect(
      Map<String, dynamic>.from(settings['deskWork'] as Map)['collections'],
      {'Fonts/font:Georgia': 2, 'Fonts/font:Arial': 6},
    );
    await tester.tap(tile('Arial'), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Remove from collection'));
    await tester.pumpAndSettle();
    expect(
      Map<String, dynamic>.from(settings['deskWork'] as Map)['collections'],
      {'Fonts/font:Georgia': 2},
    );
    // A double-click renames the collection; the name is kept with the desk.
    await tester.tap(find.byKey(const ValueKey('browser:collection:2')));
    await tester.pump(const Duration(milliseconds: 60));
    await tester.tap(find.byKey(const ValueKey('browser:collection:2')));
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('browser:collection:rename')),
      findsOneWidget,
      reason: 'dialog',
    );
    await tester.enterText(
      find.byKey(const ValueKey('browser:collection:rename')),
      'Headlines',
    );
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('browser:collection:rename')),
      findsNothing,
      reason: 'dialog closed',
    );
    expect(
      Map<String, dynamic>.from(settings['deskWork'] as Map)['collectionNames'],
      {'2': 'Headlines'},
    );
    expect(find.text('Headlines'), findsOneWidget);
    await tester.tap(tile('Georgia'));
    await tester.pumpAndSettle();
    // 0 takes it back out.
    await tester.sendKeyEvent(LogicalKeyboardKey.digit0);
    await tester.pumpAndSettle();
    expect(
      Map<String, dynamic>.from(settings['deskWork'] as Map)['collections'],
      isEmpty,
    );
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets(
    'quick tags add a user tag that becomes a filter, and a label keeps a filter',
    (tester) async {
      final (c, settings) = await mount(tester);
      await tester.tap(tile('Arial'));
      await tester.pumpAndSettle();
      // The picked row's own facts are quiet chips; Add… takes a tag of ours.
      expect(find.byKey(const ValueKey('browser:quicktags')), findsOneWidget);
      await tester.enterText(
        find.byKey(const ValueKey('browser:quicktags:add')),
        'serif',
      );
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
      expect(Map<String, dynamic>.from(settings['deskWork'] as Map)['tags'], {
        'Fonts/font:Arial': ['serif'],
      });
      expect(
        find.byKey(const ValueKey('browser:quicktag:serif')),
        findsOneWidget,
      );
      // Our tag is one more group in the Filter View.
      await tester.tap(find.byKey(const ValueKey('browser:filters-toggle')));
      await tester.pumpAndSettle();
      await tester.tap(tag('Tags', 'serif'));
      await tester.pumpAndSettle();
      expect(tile('Arial'), findsOneWidget);
      expect(tile('Georgia'), findsNothing);
      // Saved as a label, cleared, then brought back from the rail.
      await tester.tap(find.byKey(const ValueKey('browser:label:add')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Clear'));
      await tester.pumpAndSettle();
      expect(tile('Georgia'), findsOneWidget);
      await tester.tap(find.widgetWithText(InkWell, 'serif').last);
      await tester.pumpAndSettle();
      expect(tile('Georgia'), findsNothing);
      // The × on the chip takes our tag off again; the default facts stay.
      await tester.tap(find.text('Clear'));
      await tester.pumpAndSettle();
      await tester.tap(
        find.descendant(
          of: find.byKey(const ValueKey('browser:quicktag:serif')),
          matching: find.byIcon(Icons.close),
        ),
      );
      await tester.pumpAndSettle();
      expect(
        Map<String, dynamic>.from(settings['deskWork'] as Map)['tags'],
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  Future<EditorSession> mountShelf(
    WidgetTester tester,
    String tab,
    Map<String, dynamic> state, {
    Map<String, dynamic>? settings,
  }) async {
    tester.view.physicalSize = const Size(900, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final kept = settings ?? <String, dynamic>{};
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'readSettings') return kept;
          if (call.method == 'writeSettings') {
            kept.addAll(Map<String, dynamic>.from(call.arguments as Map));
            return {};
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = state;
    c.deskWork.value = {'browserRail': 120.0, 'browserView': 0};
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: tab),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('browser:filters-toggle')));
    await tester.pumpAndSettle();
    return c;
  }

  testWidgets(
    'Media: the values files carry become tags, and duration is cut by the user',
    (tester) async {
      final settings = <String, dynamic>{};
      final c = await mountShelf(tester, 'Media', {
        'capabilities': ['placeAsset'],
        'assets': [
          {
            'id': 'v1',
            'name': 'clip.mp4',
            'mime': 'video/mp4',
            'path': '/m/clip.mp4',
            'facts': {
              'width': 1920,
              'height': 1080,
              'fps': 29.97,
              'seconds': 12.0,
            },
          },
          {
            'id': 'v2',
            'name': 'long.mov',
            'mime': 'video/quicktime',
            'path': '/m/long.mov',
            'facts': {
              'width': 3840,
              'height': 2160,
              'fps': 24,
              'seconds': 95.0,
            },
          },
          {
            'id': 'i1',
            'name': 'sky.png',
            'mime': 'image/png',
            'path': '/m/sky.png',
            'facts': {'width': 1920, 'height': 1080},
          },
        ],
      }, settings: settings);
      Finder item(String id) => find.byKey(ValueKey('browser:Media:$id'));
      // Resolution and frame rate are the values present, nothing invented.
      expect(tag('Resolution', '1920×1080'), findsOneWidget);
      expect(tag('Resolution', '3840×2160'), findsOneWidget);
      expect(tag('Frame rate', '29.97'), findsOneWidget);
      expect(tag('Frame rate', '24'), findsOneWidget);
      await tester.tap(tag('Resolution', '1920×1080'));
      await tester.pumpAndSettle();
      expect(item('v1'), findsOneWidget);
      expect(item('i1'), findsOneWidget);
      expect(item('v2'), findsNothing);
      await tester.tap(find.text('Clear'));
      await tester.pumpAndSettle();
      // Duration: the seeds, then a range of the user's own; a still has none.
      await tester.tap(tag('Duration', '30-'));
      await tester.pumpAndSettle();
      expect(item('v2'), findsOneWidget);
      expect(item('v1'), findsNothing);
      expect(item('i1'), findsNothing);
      await tester.tap(find.text('Clear'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('browser:range:Duration')));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('browser:range:min')),
        '10',
      );
      await tester.enterText(
        find.byKey(const ValueKey('browser:range:max')),
        '60',
      );
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
      expect(tag('Duration', '10-60'), findsOneWidget);
      expect(Map<String, dynamic>.from(settings['deskWork'] as Map)['ranges'], {
        'Media/Duration': ['-5', '5-30', '30-', '10-60'],
      });
      await tester.tap(tag('Duration', '10-60'));
      await tester.pumpAndSettle();
      expect(item('v1'), findsOneWidget);
      expect(item('v2'), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  testWidgets('Effects: seat, time and inputs come from the declaration', (
    tester,
  ) async {
    final c = await mountShelf(tester, 'Effects', {
      'capabilities': ['applyEffect'],
      'catalog': [
        {
          'id': 'motolii.blur',
          'name': 'Blur',
          'stage': 'Pass',
          'usesClock': false,
          'persistent': false,
          'readsBackdrop': false,
          'layerInputs': 0,
          'paramCount': 1,
        },
        {
          'id': 'motolii.hold',
          'name': 'Hold',
          'stage': 'Pass',
          'usesClock': true,
          'persistent': true,
          'readsBackdrop': false,
          'layerInputs': 0,
          'paramCount': 2,
        },
        {
          'id': 'import.plasma',
          'name': 'Plasma',
          'stage': 'Pass',
          'usesClock': true,
          'persistent': false,
          'readsBackdrop': true,
          'layerInputs': 1,
          'paramCount': 4,
        },
      ],
    });
    Finder item(String id) => find.byKey(ValueKey('browser:Effects:$id'));
    await tester.tap(tag('Time', 'Feedback'));
    await tester.pumpAndSettle();
    expect(item('motolii.hold'), findsOneWidget);
    expect(item('motolii.blur'), findsNothing);
    await tester.tap(find.text('Clear'));
    await tester.pumpAndSettle();
    await tester.tap(tag('Origin', 'Imported'));
    await tester.pumpAndSettle();
    expect(item('import.plasma'), findsOneWidget);
    expect(item('motolii.hold'), findsNothing);
    expect(tag('Parameters', '4'), findsOneWidget);
    await tester.tap(tag('Inputs', 'Reads below'));
    await tester.pumpAndSettle();
    expect(item('import.plasma'), findsOneWidget);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets(
    'Colors: solid or gradient, and the blend and stops of a gradient',
    (tester) async {
      final c = await mountShelf(
        tester,
        'Colors',
        {
          'capabilities': ['applyPalette', 'setGradient'],
          'palette': [
            {
              'rgba': [1.0, 0.0, 0.0, 1.0],
              'hex': 'ff0000',
              'used': true,
            },
          ],
        },
        settings: {
          'deskWork': {
            'swatches': [
              {
                'stops': [
                  [1.0, 0.0, 0.0, 1.0],
                  [0.0, 0.0, 1.0, 1.0],
                  [0.0, 1.0, 0.0, 1.0],
                ],
                'blend': 'oklch_short',
              },
            ],
          },
        },
      );
      c.deskWork.value = {
        ...c.deskWork.value,
        'swatches': [
          {
            'stops': [
              [1.0, 0.0, 0.0, 1.0],
              [0.0, 0.0, 1.0, 1.0],
              [0.0, 1.0, 0.0, 1.0],
            ],
            'blend': 'oklch_short',
          },
        ],
      };
      await tester.pumpAndSettle();
      expect(tag('Stops', '3'), findsOneWidget);
      expect(tag('Blend', 'Oklch short'), findsOneWidget);
      await tester.tap(tag('Kind', 'Gradient'));
      await tester.pumpAndSettle();
      expect(
        find.byKey(const ValueKey('browser:Colors:saved:0')),
        findsOneWidget,
      );
      expect(find.byKey(const ValueKey('browser:Colors:ff0000')), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
