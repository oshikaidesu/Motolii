// The finished Browser bodies over the production shelves: what each shelf lists, what a tile does, and the user's own
// views kept in the desk settings both Browsers share.
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new/browser/shelf_panel.dart';
import '../lib/hf/bp/faces.dart';
import '../lib/hf/bp/shell.dart' show HeaderKey;
import '../lib/panels/browser/color_picker.dart';
import 'support/dock_test_utils.dart';
import 'support/editor_test_theme.dart';
import 'support/native_channel.dart';
import 'support/new_inspector_host.dart' show Recording;

Map<String, dynamic> textLayer(String family) => {
      'id': 1,
      'name': 'Text 1',
      'kind': 'Text',
      'locked': false,
      'properties': const [],
      'text': {'content': 'Hello', 'fontFamily': family},
      'fill': {'slot': 'fill', 'kind': 'solid'},
    };

Future<(Recording, Native)> mount(WidgetTester tester, String shelf, Map<String, dynamic> doc) async {
  tester.view.physicalSize = const Size(700, 900);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final native = Native()..install();
  ignoreSqueezedTabChips();
  final c = Recording();
  c.document.value = {
    'layers': const [],
    'selectedIds': const <int>[],
    'capabilities': ['create', 'setFont', 'applyPalette', 'setColor', 'placeAsset', 'import'],
    ...doc,
  };
  await tester.pumpWidget(MaterialApp(
    theme: editorTestTheme,
    home: Scaffold(body: SizedBox(width: 420, height: 800, child: ShelfPanel(controller: c, name: shelf))),
  ));
  await tester.pumpAndSettle();
  return (c, native);
}

/// A Create key by its name: the board prints no caption; the name is the key's semantics label.
Finder mark(String name) => find.byWidgetPredicate((w) => w is Semantics && w.properties.label == name);

void main() {
  testWidgets('Fonts: one row per installed family, search narrows, a click picks, Enter sets the family on the text', (tester) async {
    final (c, _) = await mount(tester, 'Fonts', {
      'fontFamilies': ['Arial', 'Georgia', 'Menlo'],
      'layers': [textLayer('Arial')],
      'selectedIds': [1],
      'selectedId': 1,
    });
    expect(find.byType(ThingFace), findsNWidgets(3));
    await tester.tap(find.byType(ThingFace).at(1));
    await tester.pumpAndSettle();
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pumpAndSettle();
    final set = c.commands.where((e) => e.$1 == 'setFont').last.$2;
    expect(set['family'], 'Georgia');
    expect(set['layer'], 1);

    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyF);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).first, 'men');
    await tester.pumpAndSettle();
    expect(find.byType(ThingFace), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('Colors: the wheel is the editor above the swatches, and a swatch applies to the selection on a click', (tester) async {
    final (c, _) = await mount(tester, 'Colors', {
      'palette': [
        {'id': 'red', 'name': 'Red', 'hex': '#FF0000', 'rgba': [1.0, 0.0, 0.0, 1.0], 'used': true},
        {'id': 'sky', 'name': 'Sky', 'hex': '#3388FF', 'rgba': [0.2, 0.5, 1.0, 1.0]},
      ],
      'layers': [textLayer('Arial')],
      'selectedIds': [1],
      'selectedId': 1,
    });
    expect(find.byType(ColorPicker), findsOneWidget, reason: 'the shelf editor sits in the editor slot');
    expect(find.byType(ThingFace), findsWidgets);
    await tester.tap(find.byType(ThingFace).first);
    await tester.pumpAndSettle();
    expect(c.commands.map((e) => e.$1), contains('applyPalette'));
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('Media: the document assets are listed, a double click places one', (tester) async {
    final (c, _) = await mount(tester, 'Media', {
      'assets': [
        {'id': 'a1', 'name': 'clip1.mp4', 'mime': 'video/mp4'},
        {'id': 'a2', 'name': 'clip2.mp4', 'mime': 'video/mp4'},
      ],
    });
    expect(find.text('clip1.mp4'), findsOneWidget);
    expect(find.text('clip2.mp4'), findsOneWidget);
    final tile = find.byType(ThingFace).last;
    await tester.tap(tile);
    await tester.pump(const Duration(milliseconds: 40));
    await tester.tap(tile);
    await tester.pumpAndSettle();
    final placed = c.commands.where((e) => e.$1 == 'placeAsset').last.$2;
    expect(placed['id'], 'a2');
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('Files: the walk to a place is a menu beside search, the listing is the shelf\'s own', (tester) async {
    final (_, _) = await mount(tester, 'Files', {'importExtensions': ['png']});
    expect(find.text('Places'), findsOneWidget);
    expect(find.text('Import'), findsNothing, reason: 'Files imports by applying a row, not with a tool');
    await tester.tap(find.text('Places'));
    await tester.pumpAndSettle();
    for (final place in ['Home', 'Desktop', 'Downloads', 'Pictures', 'Movies', 'Music']) {
      expect(find.text(place), findsWidgets, reason: place);
    }
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('a search is kept from the header menu, listed as a view, and dropped again', (tester) async {
    final (c, _) = await mount(tester, 'Fonts', {'fontFamilies': ['Arial', 'Georgia', 'Menlo']});
    await tester.tap(find.byType(HeaderKey).first);
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).first, 'geo');
    await tester.pumpAndSettle();
    await tester.tap(find.byType(HeaderKey).last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Save search "geo"'));
    await tester.pumpAndSettle();
    expect(((c.deskWork.value['searches'] as Map)['Fonts'] as Map)['geo'], 'geo');
    await tester.enterText(find.byType(EditableText).first, '');
    await tester.pumpAndSettle();
    await tester.tap(find.text('geo'));
    await tester.pumpAndSettle();
    expect(find.byType(ThingFace), findsOneWidget);
    await tester.tap(find.byType(HeaderKey).last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Remove saved "geo"'));
    await tester.pumpAndSettle();
    expect((c.deskWork.value['searches'] as Map)['Fonts'], isEmpty);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('user views: a digit files the pick in a collection the desk keeps, Favorites lists it, apply is Recent, saved searches persist', (tester) async {
    final (c, native) = await mount(tester, 'Fonts', {
      'fontFamilies': ['Arial', 'Georgia', 'Menlo'],
      'layers': [textLayer('Arial')],
      'selectedIds': [1],
      'selectedId': 1,
    });
    await tester.tap(find.byType(ThingFace).at(1));
    await tester.pumpAndSettle();
    await tester.sendKeyEvent(LogicalKeyboardKey.digit1);
    await tester.pumpAndSettle();
    final filed = (c.deskWork.value['collections'] as Map);
    expect(filed.length, 1);
    expect(filed.values.single, 1, reason: 'the same row Classic writes');
    expect(filed.keys.single, startsWith('Fonts/'));
    expect((native.settings['deskWork'] as Map)['collections'], isNotNull, reason: 'through the settings transport');

    await tester.tap(find.text('Favorites'));
    await tester.pumpAndSettle();
    expect(find.byType(ThingFace), findsOneWidget);

    await tester.tap(find.text('All').first);
    await tester.pumpAndSettle();
    await tester.tap(find.byType(ThingFace).at(2));
    await tester.pumpAndSettle();
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pumpAndSettle();
    expect(((c.deskWork.value['recent'] as Map)['Fonts'] as List).first, 'font:Menlo');
    await tester.tap(find.text('Recent'));
    await tester.pumpAndSettle();
    expect(find.byType(ThingFace), findsNWidgets(2), reason: 'a click on a bare tile applies, so the first pick is Recent too');
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('the tile size set in Settings (desk key browserTile) resizes the tiles', (tester) async {
    final (c, _) = await mount(tester, 'Media', {
      'assets': [for (var i = 0; i < 4; i++) {'id': 'a$i', 'name': 'clip$i.mp4', 'mime': 'video/mp4'}],
    });
    final before = tester.getSize(find.byType(ThingFace).first).width;
    await c.storeDesk('browserTile', 200.0);
    await tester.pumpAndSettle();
    final after = tester.getSize(find.byType(ThingFace).first).width;
    expect(after, greaterThan(before));
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('user tags: Tag… from the item menu is kept in the desk tags, found by tag:, and removed again', (tester) async {
    final (c, _) = await mount(tester, 'Fonts', {'fontFamilies': ['Arial', 'Georgia', 'Menlo']});
    await tester.tapAt(tester.getCenter(find.byType(ThingFace).at(1)), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Tag…'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byKey(const ValueKey('browser:prompt')).last, 'Serif Pick');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    final tags = c.deskWork.value['tags'] as Map;
    expect(tags.values.single, ['Serif Pick']);
    expect(tags.keys.single, startsWith('Fonts/'));

    await tester.tap(find.byType(HeaderKey).first);
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).first, 'tag:serif-pick');
    await tester.pumpAndSettle();
    expect(find.byType(ThingFace), findsOneWidget, reason: 'the structured search finds what the user tagged');

    await tester.enterText(find.byType(EditableText).first, '');
    await tester.pumpAndSettle();
    await tester.tapAt(tester.getCenter(find.byType(ThingFace).at(1)), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Remove tag "Serif Pick"'));
    await tester.pumpAndSettle();
    expect(c.deskWork.value['tags'], isEmpty);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('range cuts: a duration range from the header menu is a word of the search, and the user can cut their own', (tester) async {
    final (c, _) = await mount(tester, 'Media', {
      'assets': [
        {'id': 'a', 'name': 'short.mp4', 'mime': 'video/mp4', 'seconds': 3},
        {'id': 'b', 'name': 'middle.mp4', 'mime': 'video/mp4', 'seconds': 10},
        {'id': 'c', 'name': 'long.mp4', 'mime': 'video/mp4', 'seconds': 60},
      ],
    });
    expect(find.byType(ThingFace), findsNWidgets(3));
    await tester.tap(find.byType(HeaderKey).last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Duration -5 s +'));
    await tester.pumpAndSettle();
    expect(find.byType(ThingFace), findsOneWidget);
    expect(find.text('short.mp4'), findsOneWidget);
    await tester.tap(find.byType(HeaderKey).last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Add duration range…'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byKey(const ValueKey('browser:prompt')).last, '8-12');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect((c.deskWork.value['ranges'] as Map)['Media/Duration'], contains('8-12'));
    await tester.enterText(find.byType(EditableText).first, 'duration:8-12');
    await tester.pumpAndSettle();
    expect(find.text('middle.mp4'), findsOneWidget);
    expect(find.byType(ThingFace), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('Repeater is a Create capability: Create lists it under Copies and applies it as before; Effects no longer lists it', (tester) async {
    final repeater = {'id': 'motolii.repeat', 'name': 'Repeater', 'stage': 'Placement', 'owner': 'host'};
    final blur = {'id': 'motolii.blur', 'name': 'Blur', 'stage': 'Pass', 'owner': 'effect'};
    final doc = {
      'layers': [textLayer('Arial')],
      'selectedIds': [1],
      'selectedId': 1,
      'catalog': [repeater, blur],
      'createKinds': [
        {'id': 'rectangle', 'name': 'Rectangle', 'detail': 'Adds a shape layer', 'rail': 'Shapes'},
      ],
      'hostCapabilities': [
        {'id': 'motolii.repeat', 'name': 'Repeater', 'detail': 'Copies of the layer', 'rail': 'Copies', 'apply': 'applyEffect'},
      ],
      'capabilities': ['create', 'applyEffect'],
    };
    final (c, _) = await mount(tester, 'Create', doc);
    expect(mark('Repeater'), findsOneWidget);
    expect(find.text('Copies'), findsWidgets, reason: 'the class column files it under Copies');
    final tile = mark('Repeater');
    await tester.tap(tile);
    await tester.pump(const Duration(milliseconds: 40));
    await tester.tap(tile);
    await tester.pumpAndSettle();
    expect(c.commands.where((e) => e.$1 == 'applyEffect').single.$2, {'pluginIds': ['motolii.repeat']}, reason: 'the same operation and id as from Effects');
    expect(c.ops, isNot(contains('create')), reason: 'a capability is added to a layer, it makes no layer');
    await tester.pumpWidget(const SizedBox());

    await mount(tester, 'Effects', doc);
    expect(find.text('Blur'), findsWidgets);
    expect(find.text('Repeater'), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });
}
