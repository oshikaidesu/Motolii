// The stage on a real document (a window's state joined to the inside): it selects what is clicked where the inside
// says it lies on the picture, repaints at any frame and view, its tools switch the options strip, it fits small seats.
import 'dart:io';

import 'package:flutter/gestures.dart' show PointerDeviceKind;
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

  testWidgets('an effect put on shows its card under the thing\'s own, before the cards of what it may do besides, as Premiere Pro lays the effects under Motion and Opacity (golf (58))', (t) async {
    final ws = _joined(), m = ws.inside!;
    final text = m.add('text', ws.bodyFace);
    ws
      ..reread()
      ..select('$text');
    for (var i = 0; i < 200 && ws.waiting; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
    }
    final file = '$motoliiDir/effects/Glitch Shifter.fs';
    expect(ws.screenToggle(file), isTrue);
    for (var i = 0; i < 200 && (ws.waiting || !m.fxTitles(text).contains('Glitch Shifter')); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
      ws.reread();
    }
    final titles = [for (final (title, _) in m.cards(text)) title];
    expect(titles.indexOf('Glitch Shifter'), allOf(greaterThan(titles.indexOf('Transform')), lessThan(titles.indexOf('Copies'))), reason: '$titles');
  }, skip: !insideBuilt);

  testWidgets('a camera takes a motion from the Effects shelf, as AbsentMotion\'s camera is moved by its own; an effect it has no picture for is not taken, and the top bar says why (golf ㊴)', (t) async {
    final ws = _joined(), m = ws.inside!;
    final cam = m.add('camera', 'camera');
    ws
      ..reread()
      ..select('$cam');
    Future<void> settle() async {
      for (var i = 0; i < 200 && ws.waiting; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
      }
    }
    expect(ws.screenToggle('$motoliiDir/effects/am_autoshake.am'), isTrue);
    await settle();
    expect(m.word(cam, 'fx'), endsWith('am_autoshake.am'), reason: 'Auto-Shake on the camera');
    ws.screenToggle('$motoliiDir/effects/Shake.fs');
    await settle();
    expect(m.word(cam, 'fx'), endsWith('am_autoshake.am'), reason: 'a picture\'s effect not taken');
    expect(ws.notice, 'Not taken: a camera takes motions only (it has no picture of its own)');
  }, skip: !insideBuilt);

  testWidgets('a camera picked is on the frame while its bar is: it has no picture of its own to be seen (golf (76))', (t) async {
    final ws = _joined(), m = ws.inside!;
    m.add('shape', 'rect');
    final cam = m.add('camera', 'camera');
    ws.reread();
    await _stage(t, ws, const Size(924, 616));
    ws
      ..frame = 0
      ..select('$cam');
    await t.pump();
    expect(find.text('Not on this frame'), findsNothing, reason: 'the playhead is in its bar');
    expect(find.text('S'), findsNothing, reason: 'a camera has no size: its name alone');
    expect(m.set(cam, 'start', 2), isTrue, reason: m.trouble);
    ws.reread();
    await t.pump();
    expect(find.text('Not on this frame'), findsOneWidget, reason: 'it comes on at 2 s');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('Snapshot keeps the picture as it is; Show lays it over the picture only while held (After Effects\' Take and Show Snapshot; P19)', (t) async {
    final ws = _joined(), m = ws.inside!;
    m.add('shape', 'rect');
    ws.reread();
    await _stage(t, ws, const Size(924, 616));
    expect(find.byKey(const ValueKey('ws-snapshot-show')), findsNothing, reason: 'nothing taken yet');
    await t.tap(find.byKey(const ValueKey('ws-snapshot-take')));
    await t.pump();
    expect(ws.notice, isNot(startsWith('No snapshot')));
    final hand = await t.startGesture(t.getCenter(find.byKey(const ValueKey('ws-snapshot-show'))));
    await t.pump();
    final shot = find.byKey(const ValueKey('ws-stage-snapshot'));
    expect([shot.evaluate().length, t.getRect(shot) == t.getRect(find.byKey(const ValueKey('ws-stage-comp')))], [1, true], reason: 'held: the still, where the picture lies');
    expect(File(((t.widget(shot) as Image).image as FileImage).file.path).existsSync(), isTrue);
    await hand.up();
    await t.pump();
    expect(shot, findsNothing, reason: 'let go: the picture as it is now');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the Camera tool (After Effects\' Unified Camera tool, C): a drag turns the camera round where it looks, the wheel brings it nearer; each one step', (t) async {
    final ws = _joined(), m = ws.inside!;
    final cam = m.add('camera', 'camera');
    m.add('shape', 'rect');
    ws
      ..reread()
      ..select(null);
    await _stage(t, ws, const Size(924, 616));
    await t.tap(find.byKey(const ValueKey('ws-tool-7')));
    await t.pump();
    expect(find.text('CAMERA'), findsOneWidget, reason: 'its options strip');
    final comp = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    final yaw = m.value(cam, 'rotate_y', 0) ?? 0, far = m.value(cam, 'distance', 0) ?? 1;
    await t.dragFrom(comp.center, const Offset(80, 0));
    await t.pump();
    for (var i = 0; i < 100 && ws.waiting; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
    }
    expect((m.value(cam, 'rotate_y', 0) ?? 0) - yaw, closeTo(-20, 3), reason: '80 px sideways: about 20 degrees round');
    final mouse = TestPointer(1, PointerDeviceKind.mouse);
    await t.sendEventToBinding(mouse.hover(comp.center));
    await t.sendEventToBinding(mouse.scroll(const Offset(0, -200)));
    await t.pump(const Duration(milliseconds: 400));   // (the wheel at rest: let go)
    for (var i = 0; i < 100 && ws.waiting; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
    }
    expect(m.value(cam, 'distance', 0)!, lessThan(far), reason: 'the wheel turned up: nearer');
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
