// Relations v0 in the New shell: from a value's menu to the Relations panel, a lasso for the members, a destination and
// its range, one `relate`; then the relation read back from the status, a second mapping, and the Inspector's badges.
import 'dart:ui' show PointerDeviceKind;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new_shell.dart';
import '../lib/app/new/relations/relations_panel.dart';
import 'support/editor_test_theme.dart';
import 'support/dock_test_utils.dart';
import 'support/native_channel.dart';
import 'support/new_inspector_host.dart' show loadFonts;
import 'support/window_fixture.dart';

Map<String, dynamic> fixture() {
  Map<String, dynamic> thing(int id, String kind, String name, double x, double y) {
    final l = windowLayer(id, kind, x);
    return {...l, 'name': name, 'x': x, 'y': y, 'start': 0, 'duration': 300, 'properties': [
      for (final p in l['properties'] as List) if ((p as Map)['id'] == 'position') <String, dynamic>{...p, 'value': [x, y]} else p,
    ]};
  }
  return {
    ...windowStatus(1, 0),
    'layers': [
      thing(1, 'Null', 'Null 1', 960.0, 540.0),
      thing(2, 'Shape', 'Circle A', 400.0, 300.0),
      thing(3, 'Shape', 'Circle B', 600.0, 300.0),
      thing(4, 'Shape', 'Circle C', 400.0, 700.0),
      thing(5, 'Shape', 'Circle D', 600.0, 700.0),
    ],
    'selectedId': 1,
    'selectedIds': [1],
    'capabilities': ['previewProperties', 'commitPreview', 'cancelPreview', 'toggleKey', 'relate', 'unrelate', 'select'],
    'animate': false,
  };
}

void main() {
  setUpAll(loadFonts);

  testWidgets('Null.X → four circles: Scale, then Rotation, from the Inspector through the Relations panel', (tester) async {
    tester.view.physicalSize = const Size(1600, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final native = Native()..document = fixture();
    native.install();
    ignoreSqueezedTabChips();
    await tester.pumpWidget(MaterialApp(theme: editorTestTheme, home: const NewShell()));
    await tester.pumpAndSettle();

    // Position X of the selected Null, right-clicked: Relation…
    final toy = find.byKey(const ValueKey('toy-position-0'));
    await tester.ensureVisible(toy);
    await tester.tapAt(tester.getCenter(toy), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Relation…'));
    await tester.pumpAndSettle();
    expect(find.text('CHOOSE THE THINGS IT DRIVES'), findsOneWidget, reason: 'the Relations panel opened in member-selection mode');
    expect(find.text('Null 1 · Position X'), findsWidgets);

    // Lasso the four circles: their dots are where they are on the Stage, scaled into the graph.
    final graph = find.byKey(const ValueKey('relations-graph'));
    final box = tester.getRect(graph);
    final w = (native.document!['width'] as num).toDouble(), h = (native.document!['height'] as num).toDouble();
    Offset dot(double x, double y) => Offset(box.left + 28 + x / w * (box.width - 56), box.top + 28 + y / h * (box.height - 56));
    final a = dot(400, 300), d = dot(600, 700);
    final drag = await tester.startGesture(Offset(a.dx - 30, a.dy - 30), kind: PointerDeviceKind.mouse);
    for (final p in [Offset(d.dx + 30, a.dy - 30), Offset(d.dx + 30, d.dy + 30), Offset(a.dx - 30, d.dy + 30), Offset(a.dx - 30, a.dy - 30)]) {
      await drag.moveTo(p);
      await tester.pump();
    }
    await drag.up();
    await tester.pumpAndSettle();
    expect(find.text('4 things selected'), findsOneWidget);

    await tester.tap(find.byKey(const ValueKey('dest-scale')));
    await tester.pumpAndSettle();
    expect(find.text('50%'), findsOneWidget);
    expect(find.text('150%'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('relations-create')));
    await tester.pumpAndSettle();
    final relate = native.operations.where((o) => o['op'] == 'relate').single;
    expect(relate['members'], [2, 3, 4, 5]);
    expect(relate['property'], 'scale');
    expect(relate['source'], {'layer': 1, 'property': 'position', 'component': 0});
    expect((relate['outMin'], relate['outMax']), (0.5, 1.5), reason: 'in the document\'s units');
    expect(relate['inMin'], lessThan(relate['inMax']));

    // The relation is read back from the links the status carries; a second mapping shares its source.
    expect(find.byKey(const ValueKey('mapping-scale')), findsOneWidget);
    expect(find.text('4 things'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('add-mapping-rotation')));
    await tester.pumpAndSettle();
    final second = native.operations.where((o) => o['op'] == 'relate').last;
    expect(second['property'], 'rotation');
    expect((second['outMin'], second['outMax']), (-30.0, 30.0));
    expect(second['members'], [2, 3, 4, 5]);
    expect(find.byKey(const ValueKey('mapping-rotation')), findsOneWidget);

    // The source's row says what it drives; a member's row says what drives it.
    expect(find.byKey(const ValueKey('relation-position'), skipOffstage: false), findsOneWidget);
    expect(find.text('◉ 8', skipOffstage: false), findsOneWidget, reason: 'four things times two mappings');
    // A click on a dot in the graph selects that thing (the host's select), and the Inspector shows its rows.
    await tester.tapAt(dot(400, 300));
    await tester.pumpAndSettle();
    expect(native.operations.where((o) => o['op'] == 'select').last['ids'], [2]);
    expect(find.text('◉ Null 1', skipOffstage: false), findsWidgets);

    // Removing a mapping is one unrelate over the members; the relation keeps the other.
    await tester.tap(find.byKey(const ValueKey('mapping-remove-rotation')));
    await tester.pumpAndSettle();
    final un = native.operations.where((o) => o['op'] == 'unrelate').single;
    expect(un['property'], 'rotation');
    expect(un['layers'], [2, 3, 4, 5]);
    expect(find.byKey(const ValueKey('mapping-rotation')), findsNothing);
    expect(find.byKey(const ValueKey('mapping-scale')), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
