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
    // (the reviewer's: a circle turned 45 degrees touches the edge by its round side, not its box's corner; a cube
    //  turned towards the eye by what of it is seen)
    m
      ..set(b, 'rotate', 45)
      ..set(c, 'rotate_y', 35)
      ..set(c, 'rotate_x', 20);
    ws
      ..reread()
      ..select('$b');
    await open('Align');
    await t.tap(find.byKey(const ValueKey('align.0')));
    await t.pump();
    await wait(150);
    final round = m.seen(b)!.left;
    ws.select('$c');
    await open('Align');
    await t.tap(find.byKey(const ValueKey('align.2')));
    await t.pump();
    await wait(150);
    final cube = m.seen(c)!.right;
    report.add('Align turned: circle left ${round.toStringAsFixed(1)} (want 0); tilted cube right ${cube.toStringAsFixed(1)} (want 1920)');
    expect(round, closeTo(0, 1.5));
    expect(cube, closeTo(1920, 1.5));

    // Sequence: from the front, overlapping by none: each begins where the one before ends, the first where it is, keys
    // going with their layer; one that runs for good (a ground) stays.
    final ground = m.add('shape', 'rect');
    m
      ..set(a, 'duration', 1)
      ..set(b, 'duration', 2)
      ..set(c, 'duration', 1.5)
      ..set(c, 'start', 0.5)
      ..key(a, 'opacity', 0, 0)
      ..key(a, 'opacity', 0.5, 1);
    ws.reread();
    final groundWas = m.value(ground, 'start', 0) ?? 0;
    await open('Sequence');
    await t.tap(find.byKey(const ValueKey('sequence.apply')));
    await t.pump();
    await wait(150);
    await shoot('Sequence');
    final starts = [for (final id in [c, b, a, ground]) m.value(id, 'start', 0) ?? 0], keysOfA = m.keyMoments(a, 'opacity');
    report.add('Sequence: starts cube, circle, rect, ground $starts (want 0.5, 2.0, 4.0, the ground as it was: ${groundWas.toStringAsFixed(3)}); rect keys $keysOfA (want 4.0, 4.5)');
    expect(starts[0], closeTo(0.5, 1e-6));
    expect(starts[1], closeTo(2.0, 1e-6));
    expect(starts[2], closeTo(4.0, 1e-6));
    expect(starts[3], groundWas);
    expect(keysOfA[0], closeTo(4.0, 1e-6));
    expect(keysOfA[1], closeTo(4.5, 1e-6));
    m.undo();
    ws.reread();
    report.add('Sequence undone: rect start ${m.value(a, 'start', 0)}, keys ${m.keyMoments(a, 'opacity')} (want 0, 0.0 0.5)');
    expect(m.keyMoments(a, 'opacity')[0], closeTo(0, 1e-6));

    // Ease, a kind with no seat of its own (the stretch shaken: Random): the rect's opacity stretch (its key at 0 to
    // the one at 0.5) given it from its tile, then its own numbers scrubbed (how far it strays, its seed); still at both
    // keys, other than straight between.
    ws.pickKey((layer: '$a', frame: 0));
    await open('Ease');
    await t.tap(find.byKey(const ValueKey('ease.Random')));
    await t.pump();
    await wait(100);
    await t.pump();
    final given = m.wayOf(a, 'opacity', 0);
    Future<void> scrub(String name, double by) async {
      final r = t.getRect(find.byKey(ValueKey('ease.number.$name')));
      final g = await t.startGesture(r.center, kind: PointerDeviceKind.mouse);
      for (var i = 0; i < 10; i++) {
        await g.moveBy(Offset(by / 10, 0));
        await t.pump(const Duration(milliseconds: 16));
      }
      await g.up();
      await t.pump();
      await wait(100);
      await t.pump();
    }
    await scrub('center_v', 40);
    final shaken = m.wayOf(a, 'opacity', 0);
    await scrub('seed', 30);
    final thrown = m.wayOf(a, 'opacity', 0);
    await shoot('Ease_random');
    final along = [for (var i = 0; i <= 20; i++) m.value(a, 'opacity', 0.5 * i / 20) ?? 0];
    final strays = [for (final (i, v) in along.indexed) (v - i / 20).abs()].reduce((x, y) => x > y ? x : y);
    report.add('Ease Random: $given, then center_v scrubbed $shaken, then seed $thrown (want each different); at the keys ${along.first.toStringAsFixed(3)} ${along.last.toStringAsFixed(3)} (want 0 1); strays up to ${strays.toStringAsFixed(3)} (want > 0.02)');
    expect(given.startsWith('Random '), true);
    expect(shaken != given, true);
    expect(thrown != shaken, true);
    expect(along.first, closeTo(0, 1e-6));
    expect(along.last, closeTo(1, 1e-6));
    expect(strays, greaterThan(.02));
    // (the reviewer's: from a kind with no seat back to Linear; its tile tapped again keeps its numbers; undone, back)
    await t.tap(find.byKey(const ValueKey('ease.Random')));
    await t.pump();
    await wait(100);
    final kept = m.wayOf(a, 'opacity', 0);
    await t.tap(find.byKey(const ValueKey('ease.Linear')));
    await t.pump();
    await wait(100);
    final straight = m.wayOf(a, 'opacity', 0);
    m.undo();
    ws.reread();
    await t.pump();
    final back = m.wayOf(a, 'opacity', 0);
    report.add('Ease back: Random tapped again $kept (want $thrown); Linear tapped $straight (want Linear); undone $back (want $thrown)');
    expect(kept, thrown);
    expect(straight, 'Linear');
    expect(back, thrown);

    // Path: the circle carried by two keys from the left to the right and up; turned the way it goes, its frame turns
    // as its way does (up and to the right: about 45 degrees, counted clockwise as seen: -45).
    m
      ..set(b, 'rotate', 0)
      ..key(b, 'x', 0, -3)
      ..key(b, 'y', 0, -1)
      ..key(b, 'x', 1, 1)
      ..key(b, 'y', 1, 3);
    ws
      ..reread()
      ..select('$b');
    ws.frame = 30;
    await open('Path');
    await t.tap(find.byKey(const ValueKey('path.orient')));
    await t.pump();
    await wait(150);
    await t.pump();
    final turned = m.frame(b)?[4];
    await shoot('Path');
    ws.frame = 110;   // (the reviewer's: after its last key (1 s), it keeps the way it went; still in its time)
    await t.pump();
    await wait(150);
    final still = m.frame(b)?[4];
    ws.frame = 30;
    await t.pump();
    report.add('Path still at 1.8 s: turned ${still?.toStringAsFixed(1)} (want -45)');
    expect(still, closeTo(-45, 3));
    report.add('Path: orient ${m.value(b, 'orient', 0)} (want 1); frame turned ${turned?.toStringAsFixed(1)} (want about -45, up to the right)');
    expect(m.value(b, 'orient', 0), 1);
    expect(turned, closeTo(-45, 3));

    // Links: the circle pointed at the rect as its parent by a tap on the rect's dot (it keeps where it is seen), then
    // let go by a tap on the same dot.
    ws.select('$b');
    await open('Links');
    final seenBefore = m.seen(b);
    await t.tap(find.byKey(ValueKey('links.to.$a')));
    await t.pump();
    await wait(100);
    final parented = m.value(b, 'parent', 0), seenAfter = m.seen(b), keysThen = m.keyMoments(b, 'x').length;
    final ring = m.set(a, 'parent', b.toDouble());   // (the reviewer's: the rect made to go with the circle that goes with it)
    await shoot('Links');
    await t.tap(find.byKey(ValueKey('links.to.$a')));
    await t.pump();
    await wait(100);
    final freed = m.value(b, 'parent', 0);
    report.add('Links: circle parent $parented (want $a), left ${seenBefore?.left.toStringAsFixed(1)} -> ${seenAfter?.left.toStringAsFixed(1)} (want the same); x keys $keysThen then ${m.keyMoments(b, 'x').length} (want 2 2); a ring refused: ${!ring} (${m.trouble}); tapped again $freed (want 0)');
    expect(ring, false);
    expect(keysThen, 2);
    expect(m.keyMoments(b, 'x').length, 2);
    expect(parented, a);
    expect(seenAfter!.left, closeTo(seenBefore!.left, 1));
    expect(freed, 0);

    // Notes: two lines written of the rect, told when the hand leaves; the cube picked from the list does not take it.
    ws.select('$a');
    await open('Notes');
    final note = find.byKey(const ValueKey('notes.field'));
    await t.tap(note);
    await t.pump();
    (t.widget(note) as EditableText).controller.text = 'logo in at 2s\nkeep it red';
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pump();
    await wait(100);
    await t.pump();
    final written = m.word(a, 'note');
    await shoot('Notes');
    await t.tap(find.byKey(ValueKey('notes.$a')));
    await t.pump();
    report.add('Notes: rect note ${written.replaceAll('\n', ' / ')} (want logo in at 2s / keep it red); cube note "${m.word(c, 'note')}" (want none)');
    expect(written, 'logo in at 2s\nkeep it red');
    expect(m.word(c, 'note'), '');

    // Color (the dock's Colors shelf at the desk): the circle given the rect's colour from Used here, then, the
    // stroke card's link aiming, its edge the same.
    ws.select('$b');
    await open('Color');
    Finder swatch(int argb) => find.descendant(of: find.byType(WsDesk), matching: find.byWidgetPredicate((w) => w is Container && w.decoration is BoxDecoration && (w.decoration! as BoxDecoration).color == Color(argb))).first;
    Finder button(String word) => find.descendant(of: find.byType(WsDesk), matching: find.text(word)).last;
    await t.tap(swatch(0xFFFF3366));
    await t.pump();
    await t.tap(button('Fill'));
    await t.pump();
    await wait(100);
    final filled = m.word(b, 'fill');
    ws.aim = 'stroke';
    await t.pump();
    await t.tap(swatch(0xFFFF3366));
    await t.pump();
    await t.tap(button('Set'));
    await t.pump();
    await wait(100);
    ws.aim = null;
    await shoot('Color');
    report.add('Color: circle fill $filled (want ff3366), edge ${m.word(b, 'stroke')} (want ff3366)');
    expect(filled, 'ff3366');
    expect(m.word(b, 'stroke'), 'ff3366');
    // Audio: a film of clicks every half second for 4 s, from the document's start: Marks puts a beat at each; asked
    // again, the beats it put before are not put twice.
    final clicks = m.add('film', '$motoliiDir/check/beats.mp4');
    m.set(clicks, 'start', 0);
    ws
      ..reread()
      ..select('$clicks');
    await open('Audio');
    await wait(1000);   // (its sound read on another isolate)
    await t.pump();
    await t.tap(find.byKey(const ValueKey('audio.marks')));
    await t.pump();
    await wait(100);
    await t.tap(find.byKey(const ValueKey('audio.marks')));
    await t.pump();
    await wait(100);
    await shoot('Audio');
    final beats = [for (final (at, name) in m.marks) if (name == 'beat') at];
    report.add('Audio: beats ${beats.map((b) => b.toStringAsFixed(2)).join(' ')} (want 0.00 0.50 ... 3.50, 8 of them)');
    expect(beats.length, 8);
    for (final (k, b) in beats.indexed) {
      expect(b, closeTo(k * .5, .03));
    }
    File('${out.path}/report.txt').writeAsStringSync(report.join('\n'));
    expect(made, 2);
    expect(named.first, 'Drop');
    expect(ws.marks.where((m) => m.$2 != 'beat').length, 1);
  });
}
