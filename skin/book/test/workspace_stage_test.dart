// The stage on a real document (a window's state joined to the inside): it selects what is clicked where the inside
// says it lies on the picture, repaints at any frame and view, its tools switch the options strip, it fits small seats.
import 'dart:io';

import 'package:flutter/services.dart' show MethodChannel;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/parts/glyphs.dart';
import 'package:lab_book/workspace/stage.dart';
import 'package:lab_book/workspace/stage_art.dart' show WsArt;

import 'door.dart';

Ws _joined() {
  final m = Motolii.open('${Directory.systemTemp.createTempSync('stage').path}/stage.motolii')!;
  addTearDown(m.close);
  return Ws()..join(m);
}

Future<void> _stage(WidgetTester t, Ws ws, Size size) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);
  await t.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(
        ws: ws,
        child: Align(alignment: Alignment.topLeft, child: SizedBox(width: size.width, height: size.height, child: const WsStage())),
      ),
    ),
  );
  await t.pumpAndSettle();
}

/// Where a thing lies on the screen: the inside's place for it on the picture, through the composition as laid out.
Offset _on(WidgetTester t, Motolii m, int thing) {
  final comp = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), f = m.frame(thing)!;
  return comp.topLeft + Offset(f[0], f[1]) * (comp.width / WsArt.comp.width);
}

void main() {
  testWidgets('the stage selects what is clicked, where the inside says it lies', (t) async {
    final ws = _joined(), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws.reread();
    await _stage(t, ws, const Size(924, 616));
    ws.select(null);
    await t.pump();
    await t.tapAt(_on(t, m, rect));
    await t.pump();
    expect(ws.selected, '$rect');
    final comp = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.tapAt(comp.topLeft + const Offset(6, 6));
    await t.pump();
    expect(ws.selected, isNull, reason: 'nothing there');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('frame changes, view controls and the camera switch repaint without errors', (t) async {
    final ws = _joined(), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.set(rect, 'start', 2), isTrue, reason: m.trouble);
    ws.reread();
    await _stage(t, ws, const Size(924, 616));
    for (final f in [0, 12, 30, 60, 100, 150]) {
      ws.frame = f;
      await t.pump();
    }
    ws
      ..frame = 0
      ..select('$rect');
    await t.pump();
    expect(find.text('Not on this frame'), findsOneWidget, reason: 'it comes on at 2 s');
    await t.tap(find.text('100%'));
    await t.pump();
    await t.tap(find.text('Grid'));
    await t.tap(find.text('Safe'));
    await t.tap(find.text('Camera'));
    await t.pump();
    expect(find.text('Free'), findsOneWidget);
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the tool column switches the options strip', (t) async {
    final ws = _joined();
    await _stage(t, ws, const Size(924, 616));
    expect(find.text('SELECT'), findsOneWidget);
    for (final (g, name) in const [(G.rect, 'RECTANGLE'), (G.pen, 'PEN'), (G.text, 'TEXT')]) {
      await t.tap(find.byWidgetPredicate((w) => w is Glyph && w.kind == g));
      await t.pump();
      expect(find.text(name), findsOneWidget);
    }
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the stage fits a narrower, shorter seat', (t) async {
    final ws = _joined();
    for (final s in const [Size(640, 400), Size(520, 300)]) {
      await _stage(t, ws, s);
      expect(t.takeException(), isNull, reason: 'at $s');
    }
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
