import 'dart:convert';

import 'package:flutter/material.dart';
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
    await tester.tap(find.byKey(const ValueKey('browser:collection:2')));
    expect(find.text('Orange'), findsOneWidget);
    await tester.pumpAndSettle();
    expect(tile('Georgia'), findsOneWidget);
    expect(tile('Arial'), findsNothing);
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
}
