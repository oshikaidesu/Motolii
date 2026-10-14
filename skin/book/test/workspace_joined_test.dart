// The window joined to the real inside (filament/out/libmotolii.dylib, built by filament/edit.sh), driven as a hand
// would drive it: what a seat does reaches the document, and one act is one step back. Where the inside is not
// built these are skipped.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/parts/controls.dart' show Segmented;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show AnchorCube, AnchorGeometry, NumField, OnOff, PropRow, Space, SpacePlate, TextGroup;
import 'package:lab_book/workspace/dock_fonts.dart';
import 'package:lab_book/workspace/dock_parts.dart' show DockSearch;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart';
import 'package:lab_book/workspace/ws.dart';

final _built = File('$motoliiDir/out/libmotolii.dylib').existsSync();
const _font = '$motoliiDir/out/arial_black.ttf';

/// The window on a new, empty document (no picture of the stage is asked for: the seat's texture is the runner's).
Future<Ws> _pump(WidgetTester t) async {
  t.view.physicalSize = const Size(1600, 960);
  t.view.devicePixelRatio = 1;
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null); // (the system's menu bar is not there in a test)
  final doc = '${Directory.systemTemp.createTempSync('joined').path}/joined.motolii';
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
  await t.pumpAndSettle();
  final ws = WsScope.read(t.element(find.byType(WsTimeline)));
  expect(ws.inside, isNotNull, reason: 'the inside opened');
  return ws;
}

/// The window taken down, so the inside is closed before the next test opens it.
Future<void> _close(WidgetTester t) async {
  await t.pumpWidget(const SizedBox());
  t.view.reset();
}

void main() {
  testWidgets('a text typed in the inspector is the document\'s text when the box is left; undo takes it back', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font);
    expect(text, isNot(0), reason: m.trouble);
    ws
      ..reread()
      ..select('$text');
    await t.pumpAndSettle();

    final box = find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText));
    expect(box, findsOneWidget);
    await t.ensureVisible(box);
    await t.pumpAndSettle();
    await t.tap(box);
    await t.pump();
    // (Edit > Undo while a text box has the keys is the box's own undo, not the document's)
    expect(m.set(text, 'x', 1), isTrue, reason: m.trouble);
    final edit = t.widget<PlatformMenuBar>(find.byType(PlatformMenuBar)).menus.whereType<PlatformMenu>().firstWhere((e) => e.label == 'Edit');
    PlatformMenuItem item(String label) => edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == label);
    item('Undo').onSelected!();
    await t.pump();
    expect(m.value(text, 'x', 0), 1, reason: 'the document did not go back');
    expect(['Cut', 'Copy', 'Paste', 'Select All'].every((l) => item(l).onSelected != null), isTrue);
    await t.enterText(box, 'CATS');
    await t.pump();
    expect(m.word(text, 'text'), 'Text', reason: 'nothing is told while the box is still held');
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pumpAndSettle();
    expect(m.word(text, 'text'), 'CATS');
    expect(ws.layer?.name, 'CATS', reason: 'the timeline row is named by the words');

    expect(m.undo(), isTrue);
    expect(m.word(text, 'text'), 'Text');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('what the judges broke: keys of every value that moves are on the bar; a way reads back in one form; one act is one step', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    expect(m.kindOf('$motoliiDir/out/vid/happy.mp4'), 'film');
    final film = m.add('', '$motoliiDir/out/vid/happy.mp4'); // (the inside makes of a file what it is)
    expect(film, isNot(0), reason: m.trouble);
    expect(m.word(film, 'kind'), 'film');
    expect(m.moving(film), containsAll(['x', 'y', 'z', 'rotate', 'size', 'opacity']));

    expect(m.key(film, 'opacity', 0, 0) && m.key(film, 'opacity', 1, 1), isTrue, reason: m.trouble);
    ws.reread();
    expect(ws.layers.single.keys, [0, Ws.fps], reason: 'opacity keys are keys on the bar');
    expect(m.keyed(film, 'opacity', 1), 2);
    expect(m.keyed(film, 'opacity', .5), 1);
    expect(m.keyed(film, 'x', 1), 0);

    expect(m.way(film, 'opacity', 0, 'bezier 0.2 0.1 0.3 1'), isTrue, reason: m.trouble);
    expect(m.wayOf(film, 'opacity', 0), 'Bezier 0.2 0.1 0.3 1');
    expect(m.way(film, 'opacity', 0, 'Bounce 0.3 0.5'), isTrue, reason: m.trouble);
    expect(m.wayOf(film, 'opacity', 0), 'Bounce 0.3 0.5');

    m.keying(true);
    expect(m.put(film, 'opacity', .5, .25), isTrue, reason: m.trouble); // (it has keys and changes are being keyed: a key there)
    expect(m.keyed(film, 'opacity', .5), 2);
    m.keying(false);
    expect(m.put(film, 'opacity', 0, .5), isFalse, reason: 'not being keyed the whole movement is carried: its last key would be 1.5');
    expect(m.value(film, 'opacity', 0), 0);
    expect(m.put(film, 'x', .5, 1), isTrue, reason: m.trouble); // (it has none: the value that stays)
    expect(m.keyed(film, 'x', .5), 0);

    // (the window judge's fifth finding) a value's first key takes the place of the value that stayed: no key appears at 0,
    // and a second change at another moment is a move between the two
    final cube = m.add('solid', 'cube');
    expect(m.put(cube, 'x', 1, 2) && m.keyHere(cube, 'x', 1) && m.keyHere(cube, 'y', 1), isTrue, reason: m.trouble);
    expect([m.keyMoments(cube, 'x'), m.keyMoments(cube, 'y')], [[1.0], [1.0]], reason: 'one key each, where the hand was');
    m.keying(true);
    expect(m.put(cube, 'y', 2, 3), isTrue, reason: m.trouble);
    m.keying(false);
    expect(m.keyMoments(cube, 'y'), [1.0, 2.0]);
    expect(m.value(cube, 'y', 1.5), 1.5, reason: 'it travels from where it was to where it was put');
    expect(m.unkey(cube, 'y', 1) && m.unkey(cube, 'y', 2), isTrue, reason: m.trouble);
    expect([m.keyed(cube, 'y', 2), m.value(cube, 'y', 0)], [0, 3], reason: 'its last key gone, it stays what that key held');
    expect(m.set(cube, 'rotate_x', 30) && m.set(cube, 'rotate_y', 40), isTrue, reason: m.trouble);

    m.together(() {
      m.set(film, 'x', 2);
      m.set(film, 'y', 3);
    });
    expect(m.undo(), isTrue);
    expect([m.value(film, 'x', 0), m.value(film, 'y', 0)], [1, 0], reason: 'both changes of the one act went back');
    expect(m.redo(), isTrue);
    expect([m.value(film, 'x', 0), m.value(film, 'y', 0)], [2, 3]);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a thing held on the stage and carried is put where it was let go, x and y as one step; a new thing lies in front', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.dragFrom(stage.center, Offset(stage.width * .2, -stage.height * .1));
    await t.pumpAndSettle();
    expect(ws.selected, '$cube', reason: 'held, it is picked');
    final x = m.value(cube, 'x', 0)!, y = m.value(cube, 'y', 0)!;
    expect(x, greaterThan(.3), reason: 'carried to the right');
    expect(y, greaterThan(.1), reason: 'and up');
    expect(m.undo(), isTrue);
    expect([m.value(cube, 'x', 0), m.value(cube, 'y', 0)], [0, 0], reason: 'the one act goes back whole');
    expect(m.undo(), isTrue, reason: 'the step before it is the cube\'s own making');
    expect(m.things, isEmpty);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a bar slid on the timeline is when the thing comes on; cut at its tail it is on that long; each is one step', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.set(m.add('solid', 'ring'), 'duration', 6), isTrue, reason: m.trouble); // (so the document is 6 seconds long)
    ws.reread();
    await t.pumpAndSettle();
    final tracks = t.getRect(find.byKey(const ValueKey('ws-tl-tracks')));
    final row = tracks.top + 36, perFrame = (tracks.width - 60) / Ws.duration; // (about: the room at the two ends is the timeline's own)
    await t.dragFrom(Offset(tracks.left + tracks.width * .3, row), Offset(perFrame * Ws.fps, 0)); // (a second's worth)
    await t.pumpAndSettle();
    final start = m.value(cube, 'start', 0)!;
    expect(start, greaterThan(.5), reason: 'slid to the right: it comes on later');
    expect(m.lasts(cube), lessThan(0), reason: 'still on for good');
    final tail = t.getRect(find.byKey(const ValueKey('ws-tl-tracks'))).right - 1;
    expect(ws.layers.last.outF, Ws.duration);
    // (its tail carried four seconds earlier than the document's end, which is at 6: it is on for start..2)
    expect(m.hold(cube) && m.carry('tail', 2 - (6 - start)) && m.letGo(true), isTrue, reason: m.trouble);
    expect(m.lasts(cube), closeTo(2, 1e-9));
    expect(m.undo() && m.lasts(cube) < 0 && m.value(cube, 'start', 0) == start, isTrue, reason: 'the cut goes back as one step');
    expect(m.undo() && m.value(cube, 'start', 0) == 0, isTrue, reason: 'and the slide as one');
    expect(tail, greaterThan(tracks.left));
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a document says its own size: the size chip turns the stage tall, and it is tall when opened again', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    expect(m.size, (1920, 1080));
    final cube = m.add('solid', 'cube');
    expect(cube, isNot(0));
    await t.tap(find.byKey(const ValueKey('ws-top-size')));
    await t.pumpAndSettle();
    final again = ws.inside!;
    expect(identical(again, m), isFalse, reason: 'closed and opened again');
    expect(again.size, (1080, 1920));
    expect(find.text('1080 × 1920'), findsOneWidget);
    expect(again.things, [cube], reason: 'the same document');
    // (the size is a step: undo makes the stage wide again, and it is the size change that went back, not the cube)
    expect(again.at(1), isTrue);
    expect(again.undo(), isTrue);
    ws.reread();
    await t.pumpAndSettle();
    final wide = ws.inside!;
    expect(wide.size, (1920, 1080), reason: 'undo took the size back');
    expect(wide.things, [cube]);
    expect(wide.now, 1, reason: 'opened again at the moment it was at');
    expect(wide.redo(), isTrue);
    ws.reread();
    await t.pumpAndSettle();
    expect(ws.inside!.size, (1080, 1920), reason: 'and redo makes it tall again');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a flat shape is a thing; Clip to below shows a thing only inside the one before it; a seat with nothing behind goes back', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    // (the Create shelf's Solid: a flat colour as large as the stage, by the inside's own doing; it is called Solid)
    final solid = m.add('shape', 'solid');
    expect(solid, isNot(0), reason: m.trouble);
    final all = m.frame(solid)!;
    expect([all[0], all[1], all[2], all[3]], [closeTo(960, 1), closeTo(540, 1), closeTo(1920, 2), closeTo(1080, 2)]);
    expect(m.word(solid, 'name'), 'Solid');
    expect(m.undo() && m.things.isEmpty, isTrue, reason: 'made in one step');
    final rect = m.add('', 'rect');
    expect([m.word(rect, 'kind'), m.word(rect, 'shape')], ['shape', 'rect'], reason: m.trouble);
    expect(m.say(rect, 'fill', '1b2fd8') && m.set(rect, 'wide', 2), isTrue, reason: m.trouble);
    final cube = m.add('solid', 'cube');
    expect(m.set(cube, 'clip', 2), isFalse, reason: 'clip is 0 or 1');
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(ws.layers.map((l) => l.name), ['Cube', 'Rect'], reason: 'the top row is the nearest thing');
    // (a row carried down one row: the thing is one place further back)
    await t.drag(find.byKey(ValueKey('ws-tl-label-$cube')), const Offset(0, 26));
    await t.pumpAndSettle();
    expect(m.things, [cube, rect]);
    expect(ws.layers.map((l) => l.name), ['Rect', 'Cube']);
    expect(m.undo(), isTrue);
    ws.reread();
    await t.pumpAndSettle();
    expect(m.things, [rect, cube], reason: 'one step back: the order');

    Finder seat(String label) => find.descendant(of: find.ancestor(of: find.text(label), matching: find.byType(PropRow)), matching: find.byType(OnOff));
    await t.ensureVisible(seat('Clip to below'));
    await t.pumpAndSettle();
    await t.tap(seat('Clip to below'));
    await t.pumpAndSettle();
    expect(m.value(cube, 'clip', 0), 1, reason: 'the seat is the document\'s clip');
    expect(t.widget<OnOff>(seat('Clip to below')).on, isTrue);
    await t.tap(seat('Ghost'));
    await t.pumpAndSettle();
    expect(t.widget<OnOff>(seat('Ghost')).on, isFalse, reason: 'nothing is behind it: it goes back');
    expect(m.undo() && m.value(cube, 'clip', 0) == 0, isTrue, reason: 'one step back: the clip');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('round 2 of the judges: a thing put aside is in the document; every way has a seat; a new text has a word; acts within acts', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('', _font);
    expect([m.word(text, 'kind'), m.word(text, 'text'), m.value(text, 'size', 0)], ['text', 'Text', .5], reason: 'a font placed is a text with a word, at a size that fits');
    expect(m.set(text, 'visible', 0), isTrue, reason: m.trouble);
    expect(m.value(text, 'visible', 0), 0);
    expect(m.undo() && m.value(text, 'visible', 0) == 1, isTrue);

    expect(m.wayNames, containsAll(['Hold', 'Linear', 'Bezier', 'Bounce', 'Elastic', 'Cyclic', 'Random', 'Steps', 'ElasticSteps']));
    expect([m.wayAt('Linear', .25), m.wayAt('Hold', .5), m.wayAt('Elastic', 0), m.wayAt('Elastic', 1)], [.25, 0, 0, 1]);
    expect(m.key(text, 'x', 0, 0) && m.key(text, 'x', 1, 1) && m.way(text, 'x', 0, 'Elastic'), isTrue, reason: m.trouble);
    expect(m.wayOf(text, 'x', 0), startsWith('Elastic '));

    // (an act within an act is one step; a refused change unmakes the whole act)
    final before = m.value(text, 'y', 0);
    final kept = m.together(() {
      m.set(text, 'y', 1);
      return m.together(() => m.set(text, 'opacity', .5)) && m.set(text, 'rotate', 10);
    });
    expect(kept, isTrue);
    expect(m.undo(), isTrue);
    expect([m.value(text, 'y', 0), m.value(text, 'opacity', 0), m.value(text, 'rotate', 0)], [before, 1, 0], reason: 'the three changes were one step');
    m.together(() {
      m.set(text, 'y', 2);
      m.set(text, 'opacity', 7); // (refused: from 0 to 1)
    });
    expect(m.value(text, 'y', 0), before, reason: 'the act was not made at all');

    // (Animate all: whatever a hand changes or carries gets a key there, whether or not it had keys)
    final ring = m.add('solid', 'ring');
    m.keying(true);
    expect(m.hold(ring) && m.carry('place', 120, 0) && m.letGo(true), isTrue, reason: m.trouble);
    expect([m.keyed(ring, 'x', 0), m.keyed(ring, 'y', 0)], [2, 2], reason: 'carried with Animate all on: keys where the hand let go');
    m.keying(false);
    expect(m.undo() && m.keyed(ring, 'x', 0) == 0, isTrue);

    // (a thing held and let go without keeping: nothing happened; kept: one step, whatever was carried)
    expect(m.hold(text) && m.carry('place', 100, 0) && m.letGo(false), isTrue, reason: m.trouble);
    expect(m.value(text, 'x', 2), 1);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a thing that lies off the picture can be held and carried back; Render writes the document as a film beside it', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.set(cube, 'x', 3.3) && m.set(cube, 'duration', 1), isTrue, reason: m.trouble); // (off to the right of a 1920 x 1080 picture; a film of one second)
    ws.reread();
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    final frame = m.frame(cube)!;
    expect(frame[0] - frame[2] / 2, greaterThan(1890), reason: 'its frame is all but off the picture');
    final held = Offset(stage.right + 10, stage.top + frame[1] * (stage.width / 1920)); // (a point of its frame that is off the picture)
    await t.dragFrom(held, Offset(-stage.width * .5, 0));
    await t.pumpAndSettle();
    expect(m.value(cube, 'x', 0), lessThan(2), reason: 'carried back on to the picture');

    final mp4 = File('${Directory.systemTemp.path}/joined_render_${DateTime.now().microsecondsSinceEpoch}.mp4');
    // (while it is written the inside still answers, so the window does not stand still; nothing is held or changed)
    var slowest = 0, refused = 0;
    final done = await t.runAsync(() async {
      var over = false;
      final written = m.write(mp4.path)..whenComplete(() => over = true);
      final clock = Stopwatch()..start();
      while (!over && clock.elapsedMilliseconds < 90000) {
        await Future<void>.delayed(const Duration(milliseconds: 40));
        if (over) break;
        final from = clock.elapsedMilliseconds;
        final things = m.things;
        if (m.hold(cube)) {
          m.letGo(false);
        } else if (m.trouble.contains('being written')) {
          refused++;
        }
        if (things.length != 1) return false;
        if (clock.elapsedMilliseconds - from > slowest) slowest = clock.elapsedMilliseconds - from;
      }
      return written;
    });
    expect(done, isTrue, reason: m.trouble);
    expect(refused, greaterThan(3), reason: 'asked while it was being written, and told so');
    expect(slowest, lessThan(1000), reason: 'an answer came between two pictures');
    debugPrint('while a film was written: $refused askings answered, the slowest in $slowest ms');
    expect(mp4.lengthSync(), greaterThan(1000));
    mp4.deleteSync();
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('round 3 of the judges: a thing put aside is not held; Animate all travels from where it was; a movement not being keyed is carried whole', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.set(cube, 'visible', 0), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    expect(m.frame(cube), isNull, reason: 'what is not seen has no frame');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.dragFrom(stage.center, Offset(stage.width * .2, 0));
    await t.pumpAndSettle();
    expect(ws.selected, isNull, reason: 'the hand went through it');
    expect(m.value(cube, 'x', 0), 0);
    expect(m.set(cube, 'visible', 1), isTrue);

    // (Animate all turned on at 0, the thing carried at 2: it is where it was at 0 and travels to where it was let go)
    expect(m.at(0), isTrue);
    m.keying(true);
    expect(m.at(2), isTrue);
    expect(m.hold(cube) && m.carry('x', 3, 0) && m.letGo(true), isTrue, reason: m.trouble);
    expect([m.value(cube, 'x', 0), m.value(cube, 'x', 1), m.value(cube, 'x', 2)], [0, 1.5, 3], reason: 'from where it was to where it was let go');
    expect(m.undo(), isTrue, reason: 'both keys are one step');
    expect(m.keyed(cube, 'x', 2), 0);
    expect(m.redo(), isTrue);
    m.keying(false);

    // (not being keyed: the whole movement is carried, every key by as much)
    expect(m.at(1), isTrue);
    expect(m.put(cube, 'x', 1, 2.5), isTrue, reason: m.trouble);
    expect([m.value(cube, 'x', 0), m.value(cube, 'x', 1), m.value(cube, 'x', 2)], [1, 2.5, 4]);
    expect(m.keyed(cube, 'x', 1), 1, reason: 'no key was made');

    // (the frame of a thing turned about Y is the box of what it covers: a cube seen edge-on is wider than face-on)
    final faceOn = m.frame(cube)!;
    expect(m.set(cube, 'rotate_y', 45), isTrue, reason: m.trouble);
    final edgeOn = m.frame(cube)!;
    expect(edgeOn[2], greaterThan(faceOn[2] * 1.2));
    expect(edgeOn[4], 0);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Fonts shelf offers the machine\'s font files: the picked text is set in the one applied, as one step', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font);
    await t.runAsync(() => fontsRead(ws)); // (the shelf reads the folders without waiting; here the reading is done first)
    ws
      ..reread()
      ..select('$text')
      ..shelf = 'Fonts';
    await t.pumpAndSettle();
    final georgia = find.text('Andale Mono');
    await t.scrollUntilVisible(georgia, 300, scrollable: find.descendant(of: find.byType(FontsShelf), matching: find.byType(Scrollable)).first, maxScrolls: 200);
    expect(georgia, findsWidgets, reason: 'a font file of the system is on the shelf');
    await t.pumpAndSettle();
    await t.tap(georgia.first);
    await t.pumpAndSettle();
    await t.tap(find.text('Apply'));
    await t.pumpAndSettle();
    expect(m.word(text, 'file'), endsWith('/Andale Mono.ttf'), reason: m.trouble);
    expect(find.text('In use'), findsOneWidget);
    expect(m.undo(), isTrue);
    expect(m.word(text, 'file'), _font);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Stroke in the inspector is the edge of a flat shape or a text: a colour, None, and a width', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final seat = find.byWidgetPredicate((w) => w is PropRow && w.id == 'stroke');
    await t.ensureVisible(seat);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: seat, matching: find.text('None')));
    await t.pumpAndSettle();
    await t.tap(find.text('#F2F2F2').last);
    await t.pumpAndSettle();
    expect(m.word(rect, 'stroke'), 'f2f2f2', reason: m.trouble);
    expect(m.set(rect, 'stroke_width', 6), isTrue, reason: m.trouble);
    expect(wsTransform(ws, '$rect')['stroke.w'], 6, reason: 'the width is shown as the document says it');
    expect(m.say(rect, 'stroke', ''), isTrue, reason: 'None: no edge');
    expect(m.value(cube, 'stroke_width', 0), isNull, reason: 'a solid thing has no edge to say');
    // (a value the thing has not, changed in the inspector: the top bar says why it was not taken)
    ws.select('$cube');
    await t.pumpAndSettle();
    final op = find.byWidgetPredicate((w) => w is PropRow && w.id == 'op');
    if (op.evaluate().isNotEmpty) {
      await t.ensureVisible(op.first);
      await t.pumpAndSettle();
    }
    final field = find.byWidgetPredicate((w) => w is NumField && w.id == 'op').first;
    await t.ensureVisible(field);
    await t.pumpAndSettle();
    await t.drag(field, const Offset(-60, 0));
    await t.pumpAndSettle();
    expect(ws.notice, 'Not taken: a solid has no opacity');
    expect(m.set(rect, 'opacity', .5), isTrue);
    expect(m.say(rect, 'stroke', 'grey'), isFalse);
    expect(m.trouble, contains('six digits'));
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Anchor box points into the thing: it turns and grows about that point, and it is where it stands', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final before = m.frame(rect)!;
    expect([before[0], before[1]], [closeTo(960, 1), closeTo(540, 1)]);
    await t.ensureVisible(find.byType(AnchorCube));
    await t.pumpAndSettle();
    final box = t.getRect(find.byType(AnchorCube));
    await t.tapAt(box.topLeft + AnchorGeometry(box.size).at(0, 0, 0)); // (its top left corner)
    await t.pumpAndSettle();
    expect([m.value(rect, 'anchor_x', 0), m.value(rect, 'anchor_y', 0), m.value(rect, 'anchor_z', 0)], [-1, 1, 0], reason: m.trouble);
    final after = m.frame(rect)!;
    expect(after[0], greaterThan(before[0] + 10), reason: 'its top left corner is where it stands now: it lies right of there');
    expect(after[1], greaterThan(before[1] + 10), reason: 'and below');
    expect(m.undo() && m.value(rect, 'anchor_x', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Parent: a thing goes with another without moving when it is given; moves with it after; no ring of parents', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    expect(m.set(rect, 'x', -1) && m.set(rect, 'size', 2) && m.set(rect, 'rotate', 30) && m.set(cube, 'x', 1) && m.set(cube, 'y', .5), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    final before = m.frame(cube)!;
    final seat = find.byWidgetPredicate((w) => w is PropRow && w.id == 'parent');
    final chooser = seat.evaluate().isEmpty ? find.text('None').first : find.descendant(of: seat, matching: find.text('None'));
    await t.ensureVisible(chooser);
    await t.pumpAndSettle();
    await t.tap(chooser);
    await t.pumpAndSettle();
    await t.tap(find.text('Rect').last);
    await t.pumpAndSettle();
    expect(m.value(cube, 'parent', 0), rect, reason: m.trouble);
    final given = m.frame(cube)!;
    expect([given[0], given[1]], [closeTo(before[0], 1), closeTo(before[1], 1)], reason: 'it did not move when it was given a parent');
    expect(m.set(rect, 'x', 0), isTrue);
    expect(m.frame(cube)![0], greaterThan(given[0] + 50), reason: 'it goes with its parent');
    expect(m.set(rect, 'parent', cube.toDouble()), isFalse, reason: 'no ring of parents');
    expect(m.set(cube, 'parent', 0), isTrue);
    expect(m.frame(cube)![0], closeTo(m.frame(cube)![0], 1));
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a text of two lines, and Alignment: Left, Centre, Right are ThorVG\'s own; Justify it has not', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font);
    ws
      ..reread()
      ..select('$text');
    await t.pumpAndSettle();
    final box = find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText));
    await t.ensureVisible(box);
    await t.pumpAndSettle();
    await t.tap(box);
    await t.enterText(box, 'MOTOLII\nmakes moves');
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pumpAndSettle();
    expect(m.word(text, 'text'), 'MOTOLII\nmakes moves', reason: m.trouble);
    expect(ws.layer?.name, 'MOTOLII makes moves', reason: 'listed on one line');
    expect(wsTransform(ws, '$text')['text.size'], closeTo(.5 * m.point, .1), reason: 'its size in points of the picture');
    final seat = find.byWidgetPredicate((w) => w is PropRow && w.id == 'align');
    await t.ensureVisible(seat);
    await t.pumpAndSettle();
    final row = t.getRect(find.descendant(of: seat, matching: find.byType(Segmented))); // (four marks in a row: Left, Centre, Right, Justify)
    await t.tapAt(Offset(row.left + row.width * 1.5 / 4, row.center.dy));
    await t.pumpAndSettle();
    expect(m.value(text, 'align', 0), .5, reason: m.trouble);
    await t.tapAt(Offset(row.left + row.width * 3.5 / 4, row.center.dy));
    await t.pumpAndSettle();
    expect(m.value(text, 'align', 0), .5, reason: 'not taken: ThorVG has no justified lines');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Effects shelf: Gaussian Blur and Drop Shadow are ThorVG\'s own, put on and taken off; the rest have no seat', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    Future<void> pick(String name) async {
      await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), name); // (the shelf's own search: the row is in sight)
      await t.pumpAndSettle();
      final row = find.text(name).last; // (the first is the search box's own words)
      await t.ensureVisible(row);
      await t.pumpAndSettle();
      await t.tap(row);
      await t.pumpAndSettle();
    }

    await pick('Gaussian Blur');
    await t.tap(find.text('Add'));
    await t.pumpAndSettle();
    expect(m.value(rect, 'blur', 0), 12, reason: m.trouble);
    expect(ws.fxOf('$rect'), ['Gaussian Blur']);
    expect(wsTransform(ws, '$rect')['blur_radius'], 12, reason: 'the Blur card says the document\'s blur');
    await t.tap(find.text('Remove'));
    await t.pumpAndSettle();
    expect(m.value(rect, 'blur', 0), 0);

    await pick('Drop Shadow');
    await t.tap(find.text('Add'));
    await t.pumpAndSettle();
    expect(m.word(rect, 'shadow'), '000000', reason: m.trouble);
    expect(m.undo() && m.word(rect, 'shadow').isEmpty, isTrue, reason: 'one step');

    await pick('Duotone');
    await t.tap(find.text('Add'));
    await t.pumpAndSettle();
    expect(m.value(rect, 'tint', 0), 1, reason: m.trouble);
    await pick('Gradient Map');
    await t.tap(find.text('Add'));
    await t.pumpAndSettle();
    expect(m.value(rect, 'tritone', 0), 1, reason: m.trouble);
    expect(ws.fxOf('$rect'), ['Duotone', 'Gradient Map']);

    await pick('Glow');
    await t.tap(find.text('Add'));
    await t.pumpAndSettle();
    expect(m.word(rect, 'glow'), 'ffd166', reason: m.trouble);

    await pick('Twirl');
    expect(find.text('No seat yet'), findsOneWidget);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Space: a flat thing is 2D (by the order) or 2.5D (in the world, by depth); a solid thing is 3D', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final ring = m.add('solid', 'ring'), rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final plate = find.byWidgetPredicate((w) => w is SpacePlate && w.space == Space.d25);
    await t.ensureVisible(plate);
    await t.pumpAndSettle();
    await t.tap(plate);
    await t.pumpAndSettle();
    expect(m.word(rect, 'space'), '2.5d', reason: m.trouble);
    expect(m.undo() && m.word(rect, 'space').isEmpty, isTrue, reason: 'one step');
    ws.select('$ring');
    await t.pumpAndSettle();
    final on = t.widget<SpacePlate>(find.byWidgetPredicate((w) => w is SpacePlate && w.space == Space.d3));
    expect(on.on, isTrue, reason: 'a solid thing is 3D');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Create: Polygon, Star, Line and Null are ThorVG shapes of the document; a Null is a thing to go with', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    for (final (shape, name) in const [('polygon', 'Polygon'), ('star', 'Star'), ('line', 'Line'), ('null', 'Null')]) {
      final made = m.add('shape', shape);
      expect(made, isNot(0), reason: m.trouble);
      expect(m.word(made, 'name'), name);
      expect(m.frame(made), isNotNull, reason: 'it can be held, even if nothing of it is seen');
    }
    final camera = m.add('camera', 'camera');
    expect(camera, isNot(0), reason: m.trouble);
    expect([m.word(camera, 'name'), m.frame(camera)], ['Camera', null], reason: 'a camera is not seen: nothing to hold on the stage');
    expect(m.set(camera, 'rotate_y', 30) && m.set(camera, 'zoom', 2) && m.set(camera, 'distance', 1.5), isTrue, reason: m.trouble);
    expect(m.set(camera, 'size', 2), isFalse, reason: 'a camera has no size');
    ws.reread();
    expect(ws.layers.first.kind, WsKind.camera);
    expect(m.set(m.things[1], 'points', 8) && m.set(m.things[1], 'inner', .7), isTrue, reason: m.trouble);
    expect(m.set(m.things[0], 'points', 2), isFalse, reason: 'a polygon has three points at least');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Backspace on the timeline takes the picked thing out; undo brings it back', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final ring = m.add('solid', 'ring'), cube = m.add('solid', 'cube');
    expect(m.say(cube, 'fill', 'glass'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(ws.layers.map((l) => l.name), ['Cube', 'Ring'], reason: 'the top row is the nearest thing');

    await t.tap(find.byKey(ValueKey('ws-tl-label-$cube')));
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.backspace);
    await t.pumpAndSettle();
    expect(m.things, [ring]);
    expect(ws.layers.map((l) => l.id), ['$ring']);

    expect(m.undo(), isTrue);
    ws.reread();
    expect(m.things, [ring, cube]);
    expect(m.word(cube, 'fill'), 'glass', reason: 'it comes back as it was: glass is its fill');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);
}
