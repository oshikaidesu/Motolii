// The desk's tools in a real window, each opened over a small document and shot (reports/desk/<tool>.png), and each
// used the way a hand would: what it says it does is checked in the document, not on the screen alone.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/desk_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/desk.dart' show WsDesk;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('the desk\'s tools, each used', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    t.view.physicalSize = Size(t.view.physicalSize.width, t.view.physicalSize.width * 1.2);
    addTearDown(t.view.resetPhysicalSize);
    final doc = '${Directory.systemTemp.createTempSync('desk').path}/desk.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle'), c = m.add('solid', 'cube');
    m
      ..say(a, 'fill', 'ff3366')
      ..set(a, 'x', -2)
      ..say(b, 'fill', '33aaff')
      ..set(b, 'x', 2)
      ..set(c, 'y', 1);
    ws
      ..reread()
      ..select('$a');
    final out = Directory('$motoliiDir/../reports/desk')..createSync(recursive: true);
    final report = <String>[];
    Future<void> wait(int ms) => t.runAsync(() => Future<void>.delayed(Duration(milliseconds: ms)));
    Future<void> shoot(String name) async {
      await wait(250);
      await t.pump();
      // ignore: invalid_use_of_protected_member
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
      final r = t.getRect(find.byType(WsDesk)), dpr = t.view.devicePixelRatio;
      File('${out.path}/desk_rect.txt').writeAsStringSync('${r.left * dpr} ${r.top * dpr} ${r.right * dpr} ${r.bottom * dpr}');
    }

    Future<void> open(String tool) async {
      ws.openDesk(tool);
      for (var i = 0; i < 6; i++) {
        await t.pump(const Duration(milliseconds: 30));
      }
    }

    final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);

    // Markers: + at two moments, a name written over, one taken away.
    await open('Markers');
    final plus = find.byKey(const ValueKey('marks.add'));
    ws.frame = 30;
    await t.pump();
    await t.tap(plus);
    await t.pump();
    ws.frame = 90;
    await t.pump();
    await t.tap(plus);
    await t.pump();
    final made = ws.marks.length;
    final field = find.descendant(of: find.byType(WsDesk), matching: find.byType(EditableText)).first;
    await t.tap(field);
    await t.pump();
    (t.widget(field) as EditableText).controller.text = 'Drop'; // (as typed: the window's own text input does not reach a test's)
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pump();
    FocusManager.instance.primaryFocus?.unfocus(); // (the field left: its name is said)
    await t.pump();
    await wait(100);
    await t.pump();
    final named = ws.marks.map((m) => m.$2).toList();
    await shoot('Markers');
    final cross = find.byKey(const ValueKey('mark.remove')).last;
    await t.tap(cross);
    await t.pump();
    report.add('Markers: + twice made $made (want 2); names after writing over the first: $named (want Drop first); after x: ${ws.marks.length} (want 1)');
    // (the reviewer's: a mark at a frame that is no round number of seconds is named and taken away all the same)
    ws.frame = 7;
    await t.pump();
    await t.tap(plus);
    await t.pump();
    final odd = ws.marks.firstWhere((m) => (m.$1 * Ws.fps).round() == 7);
    final oddField = find.descendant(of: find.byKey(ValueKey(odd.$1)), matching: find.byType(EditableText));
    await t.tap(oddField);
    await t.pump();
    (t.widget(oddField) as EditableText).controller.text = 'Odd';
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pump();
    await wait(100);
    await t.pump();
    final oddNamed = ws.marks.where((m) => (m.$1 * Ws.fps).round() == 7).map((m) => m.$2).toList();
    await t.tap(find.descendant(of: find.byKey(ValueKey(ws.marks.firstWhere((m) => (m.$1 * Ws.fps).round() == 7).$1)), matching: find.byKey(const ValueKey('mark.remove'))));
    await t.pump();
    final oddLeft = ws.marks.where((m) => (m.$1 * Ws.fps).round() == 7).length;
    report.add('Markers at frame 7: named $oddNamed (want [Odd]); after x: $oddLeft left (want 0)');
    expect(oddNamed, ['Odd']);
    expect(oddLeft, 0);
    // (a tick carried along the strip: the mark moved, its name kept)
    final strip = t.getRect(find.descendant(of: find.byType(WsDesk), matching: find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Strip')));
    final keep = ws.marks.first, fromX = strip.left + 8 + (strip.width - 16) * (keep.$1 / (Ws.duration / Ws.fps));
    final drag = await t.startGesture(Offset(fromX, strip.center.dy), kind: PointerDeviceKind.mouse);
    for (var i = 0; i < 10; i++) {
      await drag.moveBy(const Offset(6, 0));
      await t.pump(const Duration(milliseconds: 16));
    }
    await drag.up();
    await t.pump();
    await wait(100);
    report.add('Markers moved: $keep -> ${ws.marks} (want the same name, later)');
    expect(ws.marks.length, 1);
    expect(ws.marks.first.$2, keep.$2);
    expect(ws.marks.first.$1, greaterThan(keep.$1));

    // Blend: the rect made to add, the circle cleared half, the cube (no blend) only named.
    await open('Blend');
    await t.tap(find.byKey(ValueKey('blend.$a')));   // (its way's glyph: every way opens under the row)
    await t.pump();
    await t.tap(find.byKey(ValueKey('blend.$a.overlay')));
    await t.pump();
    final bar = find.byKey(ValueKey('opacity.$b'));
    final r = t.getRect(bar);
    final hand = await t.startGesture(r.centerRight - const Offset(2, 0), kind: PointerDeviceKind.mouse);
    for (var i = 0; i < 8; i++) {
      await hand.moveBy(Offset(-r.width / 16, 0));
      await t.pump(const Duration(milliseconds: 16));
    }
    await hand.up();
    await t.pump();
    await wait(150);
    await t.tap(find.byKey(ValueKey('clip.$b')));
    await t.pump();
    await shoot('Blend');
    final opacity = m.value(b, 'opacity', 0) ?? 1;
    report.add('Blend: rect blend ${m.word(a, 'blend')} (want overlay); circle opacity ${opacity.toStringAsFixed(2)} (want about 0.5); circle clip ${m.value(b, 'clip', 0)} (want 1); cube has a blend row: ${find.byKey(ValueKey('blend.$c')).evaluate().isNotEmpty} (want false)');
    expect(m.word(a, 'blend'), 'overlay');
    expect(opacity, inInclusiveRange(0.35, 0.65));
    expect(m.value(b, 'clip', 0), 1);

    // Align: the rect against the picture's left edge, then across its middle, then its bottom.
    ws.select('$a');
    await open('Align');
    Rect seen() {
      final f = m.frame(a)!;
      return Rect.fromCenter(center: Offset(f[0], f[1]), width: f[2], height: f[3]);
    }
    await t.tap(find.byKey(const ValueKey('align.0')));
    await t.pump();
    await wait(150);
    final left = seen().left;
    await t.tap(find.byKey(const ValueKey('align.1')));
    await t.pump();
    await wait(150);
    final middle = seen().center.dx;
    await t.tap(find.byKey(const ValueKey('align.5')));
    await t.pump();
    await wait(150);
    final bottom = seen().bottom;
    await shoot('Align');
    report.add('Align: left edge at ${left.toStringAsFixed(1)} (want 0); middle at ${middle.toStringAsFixed(1)} (want 960); bottom at ${bottom.toStringAsFixed(1)} (want 1080)');
    expect(left, closeTo(0, 1));
    expect(middle, closeTo(960, 1));
    expect(bottom, closeTo(1080, 1));

    // Sequence: from the back, overlapping by none: each begins where the one before ends.
    await open('Sequence');
    await t.tap(find.byKey(const ValueKey('sequence.apply')));
    await t.pump();
    await wait(150);
    await shoot('Sequence');
    final order = [for (final l in ws.layers.reversed) int.parse(l.id)];
    final starts = [for (final id in order) m.value(id, 'start', 0) ?? 0], lasts = [for (final id in order) m.lasts(id)];
    report.add('Sequence: starts $starts, lasts $lasts (want each start where the one before ends)');
    for (var k = 1; k < order.length; k++) {
      expect(starts[k], closeTo(starts[k - 1] + lasts[k - 1], 1e-6));
    }

    // Color: the circle given the rect's colour, then its edge the same.
    ws.select('$b');
    await open('Color');
    await t.tap(find.byKey(const ValueKey('swatch.ff3366')));
    await t.pump();
    await wait(100);
    final filled = m.word(b, 'fill');
    await t.tap(find.byKey(const ValueKey('color.edge')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('swatch.33aaff')));
    await t.pump();
    await wait(100);
    await shoot('Color');
    report.add('Color: circle fill $filled (want ff3366), edge ${m.word(b, 'stroke')} (want 33aaff)');
    expect(filled, 'ff3366');
    expect(m.word(b, 'stroke'), '33aaff');
    File('${out.path}/report.txt').writeAsStringSync(report.join('\n'));
    expect(made, 2);
    expect(named.first, 'Drop');
    expect(ws.marks.length, 1);
  });
}
