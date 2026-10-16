// The shelves on a real document (the inside joined, test/door.dart): each lays out at the dock's sizes; Media is one
// browser, its tree beside the contents when wide and above them when narrow, its search over where it is.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/dock.dart';

import 'door.dart';

const _shelves = ['Media', 'Effects', 'Fonts', 'Colors', 'Create'];

/// A window's state joined to a new document in a folder of its own, with a picture beside it.
Ws _joined() {
  final dir = Directory.systemTemp.createTempSync('dock');
  File('$motoliiDir/out/audit-cubes.png').copySync('${dir.path}/still.png');
  final m = Motolii.open('${dir.path}/dock.motolii')!;
  addTearDown(m.close);
  return Ws()
    ..join(m)
    ..documentFolder = dir.path;
}

Future<void> _shelf(WidgetTester t, Ws ws, Size size, String shelf) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(
        ws: ws,
        child: Align(alignment: Alignment.topLeft, child: SizedBox(width: size.width, height: size.height, child: WsShelf(shelf, key: ValueKey(shelf)))),
      ),
    ),
  );
  await t.pumpAndSettle();
}

void main() {
  testWidgets('every shelf lays out at the reference size and when smaller', (t) async {
    final ws = _joined();
    for (final size in const [Size(240, 918), Size(240, 600)]) {
      for (final s in _shelves) {
        await _shelf(t, ws, size, s);
        expect(t.takeException(), isNull, reason: '$s at $size');
        expect(find.text(s), findsWidgets);
      }
    }
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('Media is one browser: tree beside the contents when wide, above them when narrow; search covers the location', (t) async {
    final ws = _joined();
    await _shelf(t, ws, const Size(520, 900), 'Media');
    expect(find.byKey(const ValueKey('browser:tree-grip')), findsNothing, reason: 'wide: the tree stands beside the contents');
    await _shelf(t, ws, const Size(300, 900), 'Media');
    expect(find.byKey(const ValueKey('browser:tree-grip')), findsOneWidget, reason: 'narrow: the tree above, with a grip to size it');
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}')); // (the document's own folder, a tile of Project: opened)
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    await t.pumpAndSettle();
    await t.enterText(find.byType(EditableText), 'still');
    for (var i = 0; i < 10 && find.text('still.png').evaluate().isEmpty; i++) { // (a folder of the disk is read without waiting: its files come a moment later)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    expect(find.text('still.png'), findsWidgets, reason: 'the picture beside the document, found by its name');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
