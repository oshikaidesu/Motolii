// Renders the New shell with the Relations fixture and writes PNGs (a capture, not an assertion).
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new_shell.dart';
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
  return {...windowStatus(1, 0), 'layers': [thing(1, 'Null', 'Null 1', 800.0, 500.0), thing(2, 'Shape', 'Circle A', 400.0, 300.0), thing(3, 'Shape', 'Circle B', 600.0, 300.0), thing(4, 'Shape', 'Circle C', 400.0, 700.0), thing(5, 'Shape', 'Circle D', 600.0, 700.0)],
    'selectedId': 1, 'selectedIds': [1], 'capabilities': ['previewProperties', 'commitPreview', 'cancelPreview', 'toggleKey', 'relate', 'unrelate', 'select'], 'animate': false};
}

Future<void> shot(WidgetTester tester, GlobalKey key, String name) async {
  await tester.runAsync(() async {
    final b = key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
    final img = await b.toImage(pixelRatio: 1);
    final bytes = await img.toByteData(format: ui.ImageByteFormat.png);
    File('/private/tmp/claude-501/shots/$name.png').writeAsBytesSync(bytes!.buffer.asUint8List());
  });
}

void main() {
  setUpAll(loadFonts);
  testWidgets('capture', (tester) async {
    tester.view.physicalSize = const Size(1600, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final native = Native()..document = fixture();
    native.install();
    ignoreSqueezedTabChips();
    final key = GlobalKey();
    await tester.pumpWidget(MaterialApp(theme: editorTestTheme, home: RepaintBoundary(key: key, child: const NewShell())));
    await tester.pumpAndSettle();
    final toy = find.byKey(const ValueKey('toy-position-0'));
    await tester.ensureVisible(toy);
    await tester.tapAt(tester.getCenter(toy), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    await shot(tester, key, '1-menu');
    await tester.tap(find.text('Relation…'));
    await tester.pumpAndSettle();
    final box = tester.getRect(find.byKey(const ValueKey('relations-graph')));
    final w = (native.document!['width'] as num).toDouble(), h = (native.document!['height'] as num).toDouble();
    Offset dot(double x, double y) => Offset(box.left + 28 + x / w * (box.width - 56), box.top + 28 + y / h * (box.height - 56));
    final a = dot(400, 300), d = dot(600, 700);
    final drag = await tester.startGesture(Offset(a.dx - 30, a.dy - 30), kind: PointerDeviceKind.mouse);
    for (final p in [Offset(d.dx + 30, a.dy - 30), Offset(d.dx + 30, d.dy + 30), Offset(a.dx - 30, d.dy + 30)]) { await drag.moveTo(p); await tester.pump(); }
    await shot(tester, key, '2-lasso');
    await drag.moveTo(Offset(a.dx - 30, a.dy - 30));
    await drag.up();
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('dest-scale')));
    await tester.pumpAndSettle();
    await shot(tester, key, '3-draft');
    await tester.tap(find.byKey(const ValueKey('relations-create')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('add-mapping-rotation')));
    await tester.pumpAndSettle();
    await shot(tester, key, '4-relation');
    await tester.tapAt(dot(400, 300));
    await tester.pumpAndSettle();
    await shot(tester, key, '5-member');
  });
}
