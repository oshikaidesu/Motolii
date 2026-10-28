// The Render dialog closes as a Mac dialog's Cancel does: Esc or ⌘. (Apple's), and leaves no film asked for.
import 'dart:io';

import 'package:flutter/services.dart' show LogicalKeyboardKey;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/top_bar.dart';

import 'door.dart';

Future<Ws> _pump(WidgetTester t) async {
  final m = Motolii.open('${Directory.systemTemp.createTempSync('render').path}/render.motolii')!;
  addTearDown(m.close);
  final ws = Ws()..join(m);
  const size = Size(1400, 700);   // (room for the dialog over the window)
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.binding.setSurfaceSize(size);
  await t.pumpWidget(WidgetsApp(
    color: const Color(0xFF000000),
    builder: (_, _) => WsScope(ws: ws, child: Overlay(initialEntries: [OverlayEntry(builder: (_) => const Align(alignment: Alignment.topLeft, child: SizedBox(width: 1400, child: WsTopBar())))])),   // (the dialog is an overlay: the window has one)
  ));
  await t.pumpAndSettle();
  return ws;
}

void main() {
  for (final (name, key, meta) in const [('Esc', LogicalKeyboardKey.escape, false), ('⌘.', LogicalKeyboardKey.period, true)]) {
    testWidgets('$name closes the Render dialog without rendering', (t) async {
      final ws = await _pump(t);
      wsAskRender.value++;
      await t.pumpAndSettle();
      expect(find.text('Format'), findsOneWidget, reason: 'the dialog is open');
      if (meta) await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await t.sendKeyEvent(key);
      if (meta) await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await t.pumpAndSettle();
      expect(find.text('Format'), findsNothing, reason: 'closed');
      expect(ws.renderTo, isNull, reason: 'no film asked for');
      await t.pumpWidget(const SizedBox());
    }, skip: !insideBuilt);
  }

  testWidgets('the Format chips set the name\'s ending, and Render asks for that kind of file', (t) async {
    final ws = await _pump(t);
    wsAskRender.value++;
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('render-GIF')));
    await t.pumpAndSettle();
    expect(find.textContaining('.gif'), findsWidgets, reason: 'the name now ends .gif');
    expect(find.textContaining('15 frames a second'), findsOneWidget);
    await t.tap(find.byKey(const ValueKey('render-HEVC')));
    await t.pumpAndSettle();
    var asked = '';
    ws.onRender = () => asked = ws.renderTo ?? '';
    await t.tap(find.text('Render').last);
    await t.pumpAndSettle();
    expect(asked, endsWith('.mov'), reason: 'HEVC: written as a .mov');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('Resolution, Rate and Quality are the document\'s, said as they are pressed (Alight Motion\'s export rows)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    wsAskRender.value++;
    await t.pumpAndSettle();
    for (final (key, name, v) in const [('render-render_height-720', 'render_height', 720), ('render-render_rate-30', 'render_rate', 30), ('render-render_mbps-8', 'render_mbps', 8)]) {
      await t.tap(find.byKey(ValueKey(key)));
      for (var i = 0; i < 100 && m.value(0, name, 0) != v; i++) {   // (told the inside without waiting: read once it has it)
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
        await t.pump();
      }
      expect(m.value(0, name, 0), v, reason: '$name: ${m.trouble}');
    }
    await t.pumpAndSettle();
    expect(t.takeException(), isNull, reason: 'the dialog lays out with its rows');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
