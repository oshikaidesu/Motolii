// The desk's tools in a real window, each opened over a small document and shot (reports/desk/<tool>.png), and each
// used the way a hand would: what it says it does is checked in the document, not on the screen alone.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/desk_test.dart
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/gestures.dart' show PointerDeviceKind, kSecondaryButton;
import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/desk.dart' show WsDesk;
import 'package:lab_book/workspace/desk_ease_side.dart' show EaseShapes;
import 'package:lab_book/workspace/dock.dart' show WsShelf;
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
      if (find.byType(WsDesk).evaluate().isNotEmpty) {
        final r = t.getRect(find.byType(WsDesk)), dpr = t.view.devicePixelRatio;
        File('${out.path}/desk_rect.txt').writeAsStringSync('${r.left * dpr} ${r.top * dpr} ${r.right * dpr} ${r.bottom * dpr}');
      }
    }

    Future<void> open(String tool) async {
      ws.openDesk(tool);
      for (var i = 0; i < 6; i++) {
        await t.pump(const Duration(milliseconds: 30));
      }
    }

    final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);

    // Markers, on the timeline's ruler (After Effects' composition markers): a double click puts one, twice; a double
    // click on a flag names it; a flag carried later keeps its name; one carried off the ruler is gone.
    final ruler = t.getRect(find.byKey(const ValueKey('ws-tl-ruler')));
    Future<void> twice(Offset p) async {
      await t.tapAt(p);
      await wait(70);   // (a real window's clock: a second press sooner than 40 ms is no double one, Flutter's kDoubleTapMinTime)
      await t.tapAt(p);
      await wait(350);
      await t.pump();
    }
    final a1 = Offset(ruler.left + ruler.width * .3, ruler.top + 16), a2 = Offset(ruler.left + ruler.width * .6, ruler.top + 16);
    Future<void> addMark(Offset p) async {   // (the ruler's right-click menu, as Live's Add Locator)
      await t.tapAt(p, buttons: kSecondaryButton);
      await t.pump(const Duration(milliseconds: 100));
      await t.tap(find.byKey(const ValueKey('ws-menu-Add Marker')));
      await t.pump(const Duration(milliseconds: 100));
      await wait(100);
      ws.reread();
      await t.pump();
    }
    await addMark(a1);
    await addMark(a2);
    final made = ws.marks.length;
    await twice(a1 + const Offset(4, 0));   // (on the first one's flag: its name written over)
    t.widget<EditableText>(find.descendant(of: find.byKey(const ValueKey('ws-tl-mark-name')), matching: find.byType(EditableText))).controller.text = 'Drop';   // (a real window has no test's keyboard)
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pump();   // (the field left at the next frame, and only then the name told: timeline.dart _named)
    for (var i = 0; i < 100 && (ws.waiting || ws.marks.first.$2 != 'Drop'); i++) {
      await wait(20);
      ws.reread();
      await t.pump();
    }
    final named = ws.marks.map((m) => m.$2).toList();
    await shoot('Markers');
    Future<void> carry(Offset from, Offset by) async {
      final g = await t.startGesture(from, kind: PointerDeviceKind.mouse);
      for (var i = 0; i < 10; i++) {
        await g.moveBy(by / 10);
        await t.pump(const Duration(milliseconds: 16));
      }
      await g.up();
      await t.pump();
      await wait(100);
      await t.pump();
    }
    final keep = ws.marks.first;
    await carry(a1 + const Offset(4, 0), const Offset(40, 0));
    final moved = ws.marks.first;
    await carry(a2 + const Offset(4, 0), const Offset(0, 60));   // (the second, carried down off the ruler)
    report.add('Markers: the menu made $made (want 2); named $named (want Drop first); carried ${keep.$1.toStringAsFixed(2)} -> ${moved.$1.toStringAsFixed(2)} ${moved.$2} (want later, Drop); one carried off: ${ws.marks.length} left (want 1)');
    expect(made, 2);
    expect(named.first, 'Drop');
    expect(moved.$2, 'Drop');
    expect(moved.$1, greaterThan(keep.$1));
    expect(ws.marks.length, 1);

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
    Rect seen() => m.seen(a)!;   // (asked of the inside itself, not the peek: the moment just changed)
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
    m.sequence(0);   // (the Animation menu's Sequence Layers)
    ws.reread();
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
    // the one at 0.5) given it from its tile; still at both keys, other than straight between.
    ws.pickKey((layer: '$a', frame: 0));
    await open('Ease');
    await t.ensureVisible(find.byKey(const ValueKey('ease.Random')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('ease.Random')));
    await t.pump();
    await wait(100);
    await t.pump();
    final thrown = m.wayOf(a, 'opacity', 0);
    await shoot('Ease_random');
    final along = [for (var i = 0; i <= 20; i++) m.value(a, 'opacity', 0.5 * i / 20) ?? 0];
    final strays = [for (final (i, v) in along.indexed) (v - i / 20).abs()].reduce((x, y) => x > y ? x : y);
    report.add('Ease Random: $thrown; at the keys ${along.first.toStringAsFixed(3)} ${along.last.toStringAsFixed(3)}, strays ${strays.toStringAsFixed(3)}');
    expect(thrown.startsWith('Random '), true);
    expect(along.first, closeTo(0, 1e-6));
    expect(along.last, closeTo(1, 1e-6));
    expect(strays, greaterThan(.02));
    // (the reviewer's: from a kind with no seat back to Linear; its tile tapped again keeps its numbers; undone, back)
    await t.ensureVisible(find.byKey(const ValueKey('ease.Random')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('ease.Random')));
    await t.pump();
    await wait(100);
    final kept = m.wayOf(a, 'opacity', 0);
    t.state<ScrollableState>(find.descendant(of: find.byType(EaseShapes), matching: find.byType(Scrollable))).position.jumpTo(0);   // (its row back to its head)
    await t.pump();
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
    await t.pump();
    m.set(b, 'orient', 1);   // (Auto-orient, a switch on its Transform card)
    ws.reread();
    await t.pump();
    await wait(150);
    await t.pump();
    final turned = m.frame(b)?[4];
    await shoot('Orient');
    expect(m.cards(b).any((c) => c.$1 == 'Motion' && c.$2.contains('orient')), true);   // (the card the window draws from the inside's word)
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

    // Parent (the Layer card's): the circle given the rect as its parent keeps where it is seen and its keys; a ring is
    // refused; let go again.
    ws.select('$b');
    await t.pump();
    final seenBefore = m.seen(b);
    m.set(b, 'parent', a.toDouble());
    ws.reread();
    await t.pump();
    await wait(100);
    final parented = m.value(b, 'parent', 0), seenAfter = m.seen(b), keysThen = m.keyMoments(b, 'x').length;
    final ring = m.set(a, 'parent', b.toDouble());   // (the reviewer's: the rect made to go with the circle that goes with it)
    m.set(b, 'parent', 0);
    ws.reread();
    await t.pump();
    final freed = m.value(b, 'parent', 0);
    report.add('Parent: circle parent $parented (want $a), left ${seenBefore?.left.toStringAsFixed(1)} -> ${seenAfter?.left.toStringAsFixed(1)} (want the same); x keys $keysThen then ${m.keyMoments(b, 'x').length} (want 2 2); a ring refused: ${!ring} (${m.trouble}); tapped again $freed (want 0)');
    expect(ring, false);
    expect(keysThen, 2);
    expect(m.keyMoments(b, 'x').length, 2);
    expect(parented, a);
    expect(seenAfter!.left, closeTo(seenBefore!.left, 1));
    expect(freed, 0);

    // Note (its own card): two lines of the rect's shown there; the cube has none.
    ws.select('$a');
    m.say(a, 'note', 'logo in at 2s\nkeep it red');
    ws.reread();
    await t.pump();
    await wait(100);
    await t.pump();
    final written = m.word(a, 'note'), shown = m.cards(a).any((c) => c.$1 == 'Note' && c.$2.contains('note'));
    await shoot('Note');
    report.add('Note: rect note ${written.replaceAll('\n', ' / ')} shown on its card: $shown; cube note "${m.word(c, 'note')}" (want none)');
    expect(shown, true);
    expect(m.word(c, 'note'), '');

    // Colors (the dock's shelf): the circle given the rect's colour from Used here, then, the stroke card's link aiming,
    // its edge the same.
    ws
      ..select('$b')
      ..shelf = 'Colors';
    await t.pump();
    await t.pump();
    Finder swatch(int argb) => find.descendant(of: find.byType(WsShelf), matching: find.byWidgetPredicate((w) => w is Container && w.decoration is BoxDecoration && (w.decoration! as BoxDecoration).color == Color(argb))).first;
    Finder button(String word) => find.descendant(of: find.byType(WsShelf), matching: find.text(word)).last;
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
    // Sound: a film of clicks every half second for 4 s, from the document's start: its bar shows its loudness (the clicks
    // loud, between them silent); its Beats (the inspector's Sound card) put where it strikes on the ruler.
    final clicks = m.add('film', '$motoliiDir/check/beats.mp4');
    m
      ..set(clicks, 'start', 0)
      ..set(clicks, 'beats', 9);
    ws
      ..reread()
      ..select('$clicks');
    await t.pump();
    await wait(1200);   // (its sound read on another isolate)
    await t.pump();
    await wait(100);
    await t.pump();
    await shoot('Sound');
    final wave = await t.runAsync(() => m.loudnessOver(clicks, 0, 1, 60)) ?? const <double>[];
    final beats = await t.runAsync(() => m.strikes(clicks, m.value(clicks, 'beats', 0) ?? 0)) ?? const <double>[];
    final card = find.text('Beats').evaluate().isNotEmpty;
    report.add('Sound: the inspector shows Beats: $card; bar loudest ${wave.reduce(math.max).toStringAsFixed(1)} dB, quietest ${wave.reduce(math.min).toStringAsFixed(1)} dB in the first second; beats ${beats.map((b) => b.toStringAsFixed(2)).join(' ')} (want 0.00 0.50 ... 3.50)');
    expect(card, true);
    expect(wave.reduce(math.max), greaterThan(-20));
    expect(wave.reduce(math.min), lessThan(-60));
    expect(beats.length, 8);
    for (final (k, b) in beats.indexed) {
      expect(b, closeTo(k * .5, .03));
    }
    // Keys (keys.dart, the window's own table): Space plays and stops, the arrows step a frame, with Shift ten; while
    // words are typed, Space is the words'.
    Future<void> press(LogicalKeyboardKey l, PhysicalKeyboardKey p) async {
      await t.sendKeyDownEvent(l, platform: 'macos', physicalKey: p);
      await t.sendKeyUpEvent(l, platform: 'macos', physicalKey: p);
    }
    FocusManager.instance.primaryFocus?.unfocus();
    ws
      ..playing = false
      ..frame = 30;
    await t.pump();
    await press(LogicalKeyboardKey.space, PhysicalKeyboardKey.space);
    await t.pump();
    final played = ws.playing;
    await press(LogicalKeyboardKey.space, PhysicalKeyboardKey.space);
    await t.pump();
    final stopped = !ws.playing;
    ws.frame = 30;
    await t.pump();
    await press(LogicalKeyboardKey.arrowRight, PhysicalKeyboardKey.arrowRight);
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft, platform: 'macos', physicalKey: PhysicalKeyboardKey.shiftLeft);
    await press(LogicalKeyboardKey.arrowRight, PhysicalKeyboardKey.arrowRight);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft, platform: 'macos', physicalKey: PhysicalKeyboardKey.shiftLeft);
    await t.pump();
    final stepped = ws.frame;
    final search = find.descendant(of: find.byType(WsShelf), matching: find.byType(EditableText)).first;
    await t.tap(search);
    await t.pump();
    await press(LogicalKeyboardKey.space, PhysicalKeyboardKey.space);
    await t.pump();
    final typedPlays = ws.playing;
    FocusManager.instance.primaryFocus?.unfocus();
    report.add('Keys: Space played $played and stopped $stopped; right, shift+right: frame $stepped (want 41); Space while typing played $typedPlays (want false)');
    expect(played, true);
    expect(stopped, true);
    expect(stepped, 41);
    expect(typedPlays, false);
    File('${out.path}/report.txt').writeAsStringSync(report.join('\n'));
    expect(made, 2);
    expect(named.first, 'Drop');
    expect(ws.marks.length, 1);
  });
}
