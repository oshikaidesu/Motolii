// The window joined to the real inside (filament/out/libmotolii.dylib, built by filament/edit.sh), driven as a hand
// would drive it: what a seat does reaches the document, and one act is one step back. Where the inside is not
// built these are skipped.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/parts/controls.dart' show Segmented;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show AnchorCube, AnchorGeometry, EffectCard, ValuesCard, FillStroke, InspHeader, KeyS, NumField, OnOff, PropRow, Space, SpacePlate, TextGroup;
import 'package:lab_book/workspace/dock_fonts.dart';
import 'package:lab_book/workspace/dock_make.dart' show CreateShelf;
import 'package:lab_book/workspace/desk_kit.dart' show DeskScrub;
import 'package:lab_book/workspace/dock_parts.dart' show DockFooter, DockHover, DockSearch;
import 'package:lab_book/workspace/fx_catalog.dart' show fxCatalog;
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
    expect(m.say(rect, 'fill', 'ff3366'), isTrue);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final seat = find.byWidgetPredicate((w) => w is PropRow && w.id == 'stroke');
    await t.ensureVisible(seat);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: seat, matching: find.text('None')));
    await t.pumpAndSettle();
    expect(find.text('#F2F2F2'), findsNothing, reason: 'offered: the colours the document has, not the sample\'s greys');
    await t.tap(find.text('#FF3366').last);
    await t.pumpAndSettle();
    expect(m.word(rect, 'stroke'), 'ff3366', reason: m.trouble);
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
    final row = t.getRect(find.descendant(of: seat, matching: find.byType(Segmented))); // (three marks in a row: Left, Centre, Right)
    await t.tapAt(Offset(row.left + row.width * 1.5 / 3, row.center.dy));
    await t.pumpAndSettle();
    expect(m.value(text, 'align', 0), .5, reason: m.trouble);
    expect(find.descendant(of: seat, matching: find.text('Justify')), findsNothing, reason: 'not offered: ThorVG has no justified lines');
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
    expect(wsTransform(ws, '$rect')['blur_amount'], 12, reason: 'the Blur card says the document\'s blur');
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

    await pick('Trim Paths');
    await t.tap(find.text('Add'));
    await t.pumpAndSettle();
    expect(m.keyMoments(rect, 'trim'), [0, 1], reason: 'drawn on in a second, from the playhead');
    expect([m.value(rect, 'trim', 0), m.value(rect, 'trim', 1)], [1, 0]);
    await t.tap(find.text('Remove'));
    await t.pumpAndSettle();
    expect([m.keyMoments(rect, 'trim'), m.value(rect, 'trim', 0)], [isEmpty, 0], reason: 'taken off with its keys');

    await pick('Twirl');
    expect(find.descendant(of: find.byType(DockFooter), matching: find.text('No seat yet')), findsNWidgets(2), reason: 'what it is to the document, and its button');
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
    expect(m.set(camera, 'aperture', 1.4) && m.set(camera, 'focus', 6) && m.set(camera, 'blur_level', 3), isTrue, reason: m.trouble);
    ws.reread();
    expect(ws.layers.first.kind, WsKind.camera);
    final seen = wsTransform(ws, '$camera');
    expect([seen['cam.yaw'], seen['cam.zoom'], seen['cam.dist']], [30, 200, closeTo(1.5 * 6 * m.point, .1)], reason: 'the Camera card says the document');
    expect(m.set(m.things[1], 'points', 8) && m.set(m.things[1], 'inner', .7), isTrue, reason: m.trouble);
    expect(m.set(m.things[0], 'points', 2), isFalse, reason: 'a polygon has three points at least');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Create: Repeater and Radial lay the picked thing out in copies, as one step', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final star = m.add('shape', 'star');
    ws
      ..reread()
      ..select('$star')
      ..shelf = 'Create';
    await t.pumpAndSettle();
    Future<void> search(String what) async {
      await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), what);
      await t.pumpAndSettle();
    }

    await search('Radial');
    final tile = find.descendant(of: find.byType(CreateShelf), matching: find.text('Radial')).first;
    await t.ensureVisible(tile);
    await t.pumpAndSettle();
    await t.tap(tile);
    await t.pumpAndSettle();
    await t.tap(find.text('Lay out'));
    await t.pumpAndSettle();
    expect([m.value(star, 'copies', 0), m.value(star, 'copy_rotate', 0)], [8, 45], reason: m.trouble);
    expect(m.undo() && m.value(star, 'copies', 0) == 1, isTrue, reason: 'one step');
    expect(m.set(star, 'copies', 1001), isFalse, reason: 'a thousand at most');
    await search('Grid');
    final grid = find.descendant(of: find.byType(CreateShelf), matching: find.text('Grid')).first;
    await t.ensureVisible(grid);
    await t.pumpAndSettle();
    await t.tap(grid);
    await t.pumpAndSettle();
    await t.tap(find.text('Lay out'));
    await t.pumpAndSettle();
    expect([m.value(star, 'copies', 0), m.value(star, 'copy_columns', 0)], [9, 3], reason: m.trouble);
    expect(t.takeException(), isNull, reason: 'an anchor outside the box is named Custom');
    await _close(t);
  }, skip: !_built);

  testWidgets('the Depth desk is the camera\'s lens: its focal length is the zoom, a scrub of the aperture is one step', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    final camera = m.add('camera', 'camera');
    ws
      ..reread()
      ..select('$camera')
      ..openDesk('Depth');
    await t.pumpAndSettle();
    expect(find.text('45 mm'), findsWidgets, reason: 'zoom 1 on Filament\'s 24 mm film');
    expect(find.text('On ${m.word(cube, 'name')}'), findsOneWidget, reason: 'the focus is on what lies where it looks: the document says so, not the window');
    final stop = find.widgetWithText(DeskScrub, 'Aperture');
    await t.ensureVisible(stop);
    await t.drag(stop, const Offset(-80, 0));
    await t.pumpAndSettle();
    final aperture = m.value(camera, 'aperture', 0)!;
    expect(aperture, inExclusiveRange(1.4, 22), reason: 'the f-number the hand left');
    expect(m.undo() && m.value(camera, 'aperture', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Colors: a saved gradient fills a flat thing as a ramp of all its colours; Used here is the document\'s fills', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.say(rect, 'fill', '2a4bd8'), isTrue);
    ws
      ..reread()
      ..select('$rect')
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'gradient');
    await t.pumpAndSettle();
    final tile = find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Grad').at(1); // (the second saved gradient: Hot pink)
    await t.ensureVisible(tile);
    await t.pumpAndSettle();
    await t.tap(tile);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Fill')));
    await t.pumpAndSettle();
    expect(m.word(rect, 'fill'), 'e974ab f08a5d efcb4e', reason: m.trouble);
    expect(m.say(m.add('solid', 'cube'), 'fill', 'e974ab f08a5d'), isFalse, reason: 'a solid is filled with one colour');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('no decoration: the document\'s name on the stage and the timeline, its own marks and work area on the ruler', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    expect(find.text('Hit'), findsNothing, reason: 'the sample\'s marks are not the document\'s');
    final name = ws.document!.replaceFirst('.motolii', '');
    expect(find.text(name), findsWidgets, reason: 'the timeline says the document');
    expect(m.work, (0.0, m.seconds), reason: 'not said: all of it');
    expect(m.setWork(1, 3) && m.work == (1.0, 3.0), isTrue, reason: m.trouble);
    expect(m.undo() && m.work == (0.0, m.seconds), isTrue, reason: 'one step');
    expect(m.setWork(3, 1), isFalse);
    expect(m.mark(2, 'Drop') && m.mark(.5, 'Hit'), isTrue, reason: m.trouble);
    expect(m.marks, [(.5, 'Hit'), (2.0, 'Drop')]);
    ws.reread();
    await t.pumpAndSettle();
    expect(m.mark(2, '') && m.marks.length == 1, isTrue, reason: 'a mark said with nothing is taken away');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Edit and Animate are one truth with the inspector\'s Animate all (the inside\'s keying); a new kind picked keeps it', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    await t.tap(find.text('Animate'));
    await t.pumpAndSettle();
    expect(m.keyingNow, isTrue, reason: 'Animate keys every change');
    ws.select('$cube');
    await t.pumpAndSettle();
    expect(m.keyingNow, isTrue, reason: 'another kind picked does not turn it off');
    await t.tap(find.text('Edit'));
    await t.pumpAndSettle();
    expect(m.keyingNow, isFalse);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('guides are the document\'s: pulled out of the top ruler, a level one where it is let go; back over the ruler, gone', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.guides, isEmpty, reason: 'the sample\'s guides are not the document\'s');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    final view = t.getRect(find.ancestor(of: find.byKey(const ValueKey('ws-stage-comp')), matching: find.byType(LayoutBuilder)).first);
    await t.dragFrom(Offset(stage.center.dx, view.top + 5), Offset(0, stage.center.dy - view.top - 5));
    await t.pumpAndSettle();
    expect(m.guides.length, 1, reason: m.trouble);
    expect(m.guides.single.$1, 'y');
    expect(m.guides.single.$2, closeTo(540, 30), reason: 'about the middle of the picture');
    expect(m.undo() && m.guides.isEmpty, isTrue, reason: 'one step');
    expect(m.guide('x', 150, true) && m.guide('x', 150, false) && m.guides.isEmpty, isTrue);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the tools draw things of the document: a rectangle as wide and high as the box drawn, a line from point to point', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    ws.reread();
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.tap(find.byKey(const ValueKey('ws-tool-2'))); // (the Rectangle tool)
    await t.pumpAndSettle();
    await t.dragFrom(stage.center - Offset(stage.width * .2, stage.height * .1), Offset(stage.width * .4, stage.height * .2));
    await t.pumpAndSettle();
    final rect = int.parse(ws.selected!);
    expect([m.word(rect, 'kind'), m.word(rect, 'shape')], ['shape', 'rect'], reason: m.trouble);
    final box = m.frame(rect)!, k = 1920 / stage.width;
    expect(box[2], closeTo(stage.width * .4 * k, 12), reason: 'as wide as drawn');
    expect(box[3], closeTo(stage.height * .2 * k, 12), reason: 'and as high');
    expect(m.undo() && m.things.isEmpty, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inks and the strip mean the document: a colour filled with nothing picked is what is drawn next; the Move tool along X keeps y', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    ws
      ..reread()
      ..select(null)
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'gradient');
    await t.pumpAndSettle();
    final tile = find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Grad').first;
    await t.ensureVisible(tile);
    await t.tap(tile);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Fill')));
    await t.pumpAndSettle();
    expect(ws.ink, 'b3c66b 4781e5', reason: 'nothing picked: the ink');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.tap(find.byKey(const ValueKey('ws-tool-2')));
    await t.pumpAndSettle();
    await t.dragFrom(stage.center, Offset(stage.width * .1, stage.height * .1));
    await t.pumpAndSettle();
    final rect = int.parse(ws.selected!);
    expect(m.word(rect, 'fill'), 'b3c66b 4781e5', reason: 'drawn in the ink');
    await t.tap(find.byKey(const ValueKey('ws-tool-1')));
    await t.pumpAndSettle();
    await t.tap(find.text('X').first); // (the strip's, before the inspector's)
    await t.pumpAndSettle();
    final y = m.value(rect, 'y', 0);
    await t.dragFrom(stage.center + Offset(stage.width * .05, stage.height * .05), Offset(stage.width * .1, stage.height * .1));
    await t.pumpAndSettle();
    expect(m.value(rect, 'y', 0), y, reason: 'carried along X only');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('solo, lock and loop are the document\'s and the inside\'s, not the window\'s', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.set(a, 'solo', 1), isTrue, reason: m.trouble);
    expect([m.frame(a), m.frame(b)], [isNotNull, isNull], reason: 'while one is solo only it is seen');
    expect(m.set(a, 'solo', 0) && m.frame(b) != null, isTrue);
    expect(m.set(b, 'lock', 1), isTrue);
    expect(m.set(b, 'x', 1), isFalse);
    expect(m.trouble, 'it is locked');
    expect(m.remove(b), isFalse, reason: 'nor taken away');
    expect(m.set(b, 'lock', 0) && m.set(b, 'x', 1), isTrue, reason: 'unlocked, it is changed again');
    expect(m.looping, isTrue, reason: 'play goes round, as a document is opened');
    m.looping = false;
    expect(m.looping, isFalse);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the strip means the document: Snap sticks to the picture\'s middle, Group picks the parent, a Mask rectangle cuts the picked thing; the pivot is the anchor', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'circle'), b = m.add('shape', 'rect');
    expect(m.set(a, 'x', -1.5) && m.set(b, 'x', 1.5) && m.set(b, 'anchor_x', 1), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final fb = m.frame(b)!;
    expect(fb[5], closeTo(fb[0] + fb[2] / 2, 1), reason: 'the point it turns about is its anchor: its right edge');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), k = 1920 / stage.width;
    Offset at(int thing) => stage.topLeft + Offset(m.frame(thing)![0], m.frame(thing)![1]) / k;
    await t.tap(find.byKey(const ValueKey('ws-tool-1')));
    await t.pumpAndSettle();
    await t.dragFrom(at(a), stage.center + const Offset(3, -2) - at(a));
    await t.pumpAndSettle();
    expect(m.frame(a)![0], closeTo(960, .5), reason: 'Snap: its middle on the picture\'s, though let go three points off');
    expect(m.frame(a)![1], closeTo(540, .5));
    expect(m.set(a, 'parent', b.toDouble()), isTrue, reason: m.trouble);
    ws.reread();
    await t.tap(find.byKey(const ValueKey('ws-tool-0')));
    await t.pumpAndSettle();
    await t.tap(find.text('Group'));
    await t.pumpAndSettle();
    await t.tapAt(at(a));
    await t.pumpAndSettle();
    expect(ws.selected, '$b', reason: 'Group: what goes with another is picked as that one');
    ws.select('$a');
    await t.tap(find.byKey(const ValueKey('ws-tool-2')));
    await t.pumpAndSettle();
    await t.tap(find.text('Mask'));
    await t.pumpAndSettle();
    await t.dragFrom(at(a) - Offset(stage.width * .03, stage.height * .03), Offset(stage.width * .06, stage.height * .06));
    await t.pumpAndSettle();
    final cut = int.parse(ws.selected!), all = m.things;
    expect(all.indexOf(cut), all.indexOf(a) - 1, reason: 'the mask lies just behind the thing it cuts: ${m.trouble}');
    expect([m.value(a, 'clip', 0), m.value(cut, 'opacity', 0)], [1, 0], reason: 'it is clipped by it, which is not seen itself');
    expect(m.undo() && m.things.length == 2 && m.value(a, 'clip', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the shelves\' headers count what the inside has: effects it does, colours the document uses, tiles that make', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    expect(m.say(a, 'fill', 'ff3366'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    expect(find.text('1 used here'), findsOneWidget);
    ws.shelf = 'Effects';
    await t.pumpAndSettle();
    final fx = fxCatalog.where((f) => fxValues.containsKey(f.name)).length;
    expect(fx, greaterThan(3));
    expect(find.text('$fx of ${fxCatalog.length} work'), findsOneWidget);
    ws.shelf = 'Create';
    await t.pumpAndSettle();
    expect(find.text('28 of 40 work'), findsOneWidget);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Depth says how deep a thing is (a flat one is Flat, a solid its depth); a seat the document has no value for says so', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final row = find.byWidgetPredicate((w) => w is NumField && w.id == 'depth');
    String shown() => [for (final e in find.descendant(of: row, matching: find.byType(RichText)).evaluate()) (e.widget as RichText).text.toPlainText()].last;
    expect(shown(), 'Flat');
    await t.ensureVisible(row);
    await t.drag(row, const Offset(60, 0));
    await t.pumpAndSettle();
    expect(ws.notice, startsWith('Not taken: the document has no depth'));
    expect(shown(), 'Flat', reason: 'the number goes back to the document\'s');
    ws.select('$cube');
    await t.pumpAndSettle();
    expect(shown(), startsWith('${m.frame(cube)![7].round()}'), reason: 'a solid: how deep it is');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a camera picked shows its own card (where it looks, how far, its zoom), not a thing\'s scale, opacity or depth', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cam = m.add('camera', 'camera');
    ws
      ..reread()
      ..select('$cam');
    await t.pumpAndSettle();
    final ids = {for (final e in find.byType(NumField).evaluate()) (e.widget as NumField).id};
    expect(ids, containsAll(['cam.t.x', 'cam.dist', 'cam.zoom']));
    expect(ids.intersection({'scale.x', 'op', 'depth'}), isEmpty, reason: 'a camera has none of these');
    expect([for (final e in find.byType(ValuesCard).evaluate()) (e.widget as ValuesCard).card.name], ['Projection'], reason: 'its lens: orthographic, shifted');
    expect(m.say(cam, 'projection', 'orthographic') && m.key(cam, 'shift_y', 0, 0.2), isTrue, reason: m.trouble);
    final dist = find.byWidgetPredicate((w) => w is NumField && w.id == 'cam.dist');
    await t.ensureVisible(dist);
    await t.drag(dist, const Offset(60, 0));
    await t.pumpAndSettle();
    expect(m.value(cam, 'distance', 0), isNot(anyOf(isNull, 1.0)), reason: 'the distance is the document\'s: ${ws.notice}');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector\'s own: a card folds and stays folded; Ghost the document has not says so; the text box is the picked text\'s', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    const font = '/System/Library/Fonts/Supplemental/Arial Unicode.ttf';
    final a = m.add('text', font), b = m.add('text', font);
    expect(m.say(a, 'text', 'first') && m.say(b, 'text', 'second'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final box = find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText));
    expect((t.widget(box) as EditableText).controller.text, 'first');
    ws.select('$b');
    await t.pumpAndSettle();
    expect((t.widget(box) as EditableText).controller.text, 'second', reason: 'another text picked: its own words');
    await t.tap(box);
    await t.pumpAndSettle();
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pumpAndSettle();
    expect([m.word(a, 'text'), m.word(b, 'text')], ['first', 'second'], reason: 'in and out of the box writes nothing stale');
    expect(find.text('Justify'), findsNothing, reason: 'ThorVG has not');
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'pos.x'), findsOneWidget);
    await t.tap(find.text('Transform').first);
    await t.pumpAndSettle();
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'pos.x'), findsNothing, reason: 'folded, and it stays so');
    final ghost = find.descendant(of: find.byWidgetPredicate((w) => w is PropRow && w.id == 'ghost'), matching: find.byType(OnOff));
    await t.ensureVisible(ghost);
    await t.tap(ghost);
    await t.pumpAndSettle();
    expect((t.widget(ghost) as OnOff).on, isFalse, reason: 'it goes back');
    expect(ws.notice, startsWith('Not taken: the document has no ghost'));
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector\'s effects are the document\'s on the thing: a card each, its fields the numbers the inside has for it', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    expect(ws.fxToggle('Drop Shadow') && ws.fxToggle('Glow'), isTrue, reason: m.trouble);
    await t.pumpAndSettle();
    final cards = [for (final e in find.byType(EffectCard).evaluate()) (e.widget as EffectCard).fx.name];
    expect(cards, ['Drop Shadow', 'Glow']);
    final ids = {for (final e in find.byType(NumField).evaluate()) (e.widget as NumField).id};
    expect(ids, containsAll(['shadow_distance', 'shadow_angle', 'shadow_blur', 'shadow_opacity', 'glow_blur', 'glow_opacity']));
    final far = find.byWidgetPredicate((w) => w is NumField && w.id == 'shadow_distance'), was = m.value(a, 'shadow_distance', 0);
    await t.ensureVisible(far);
    await t.drag(far, const Offset(40, 0));
    await t.pumpAndSettle();
    expect(m.value(a, 'shadow_distance', 0), isNot(was), reason: 'the distance is the document\'s: ${ws.notice}');
    expect(m.undo(), isTrue);
    expect(m.value(a, 'shadow_distance', 0), was, reason: 'one step');
    ws.reread();
    await t.pumpAndSettle();
    final more = find.descendant(of: find.byType(EffectCard).first, matching: find.text('More'));
    await t.ensureVisible(more);
    await t.tap(more);
    await t.pumpAndSettle();
    await t.tap(find.text('Move later'));
    await t.pumpAndSettle();
    expect(ws.notice, startsWith('Not taken: the effects have no order'));
    if (find.text('Remove').evaluate().isEmpty) await t.tap(more); // (the menu closes once used)
    await t.pumpAndSettle();
    await t.tap(find.text('Remove'));
    await t.pumpAndSettle();
    expect([ws.fxOf('$a'), m.word(a, 'shadow')], [['Glow'], ''], reason: 'taken off the document');
    expect([for (final e in find.byType(EffectCard).evaluate()) (e.widget as EffectCard).fx.name], ['Glow']);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Saved says when the document\'s file was last written: a change is written as it is made', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    final at = File('${ws.documentFolder}/${ws.document}').lastModifiedSync();
    expect(DateTime.now().difference(at).inSeconds, lessThan(5), reason: 'the change is in the file already');
    expect(find.text('Saved ${at.hour.toString().padLeft(2, '0')}:${at.minute.toString().padLeft(2, '0')}'), findsOneWidget);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector says what a thing is in the document\'s word, and shows only the cards of values it has', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final film = m.add('', '$motoliiDir/out/vid/happy.mp4'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$film');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(InspHeader), matching: find.text('Film')), findsOneWidget);
    expect(find.byType(FillStroke), findsNothing, reason: 'a film has no fill: ${m.values(film)}');
    ws.select('$cube');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(InspHeader), matching: find.text('Solid')), findsOneWidget);
    expect(find.byType(FillStroke), findsOneWidget, reason: 'a solid\'s fill is a colour or glass');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inside\'s own cards: a text\'s Type (ThorVG\'s spacing and slant), a shape\'s Line (its ends, corners, dashes), each from the document', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font), rect = m.add('shape', 'rect');
    expect(m.say(text, 'text', 'MOTOLII') && m.say(rect, 'stroke', 'ff3366') && m.set(rect, 'stroke_width', 12), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$text');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Type')), findsOneWidget);
    expect(m.cards(text).map((c) => c.$1).contains('Gradient'), isFalse, reason: 'one colour: no ramp to turn');
    expect(m.say(text, 'fill', 'ff3366 1b2fd8') && m.cards(text).map((c) => c.$1).contains('Gradient'), isTrue, reason: 'two: its ramp');
    expect(m.say(text, 'fill', 'ffffff'), isTrue);
    final wide = m.frame(text)![2], letters = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_letter_spacing');
    expect(letters, findsOneWidget);
    await t.ensureVisible(letters);
    await t.drag(letters, const Offset(80, 0));
    await t.pumpAndSettle();
    expect(m.value(text, 'letter_spacing', 0), greaterThan(1), reason: 'the letters further apart: ${ws.notice}');
    expect(m.frame(text)![2], greaterThan(wide), reason: 'and the words wider on the stage');
    ws.select('$rect');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Line')), findsOneWidget);
    final round = find.descendant(of: find.byType(ValuesCard), matching: find.text('ROUND')).first; // (the ends: butt, round, square, as the sample's segments write them)
    await t.ensureVisible(round);
    await t.tap(round);
    await t.pumpAndSettle();
    expect(m.word(rect, 'cap'), 'round', reason: m.trouble);
    expect(m.undo() && m.word(rect, 'cap').isEmpty, isTrue, reason: 'one step');
    final cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Surface')), findsOneWidget, reason: 'Filament\'s own surface of a solid');
    expect(m.say(cube, 'fill', 'glass'), isTrue, reason: m.trouble);
    expect(m.cards(cube).map((c) => c.$1), ['Glass'], reason: 'filled with glass: the glass\'s own card, not the surface\'s');
    expect(m.cards(rect).map((c) => c.$1).contains('Matte'), isFalse, reason: 'not clipped: no matte to say how');
    expect(m.set(rect, 'clip', 1) && m.cards(rect).map((c) => c.$1).contains('Matte'), isTrue);
    expect(m.set(cube, 'ior', 2.2) && m.value(cube, 'ior', 0) == 2.2, isTrue, reason: m.trouble);
    expect(m.say(cube, 'fill', 'ff3366') && m.cards(cube).map((c) => c.$1).join() == 'Surface', isTrue);
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_metallic'), findsOneWidget);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('nothing picked, the inspector is the whole picture\'s: Filament\'s grade, glow and room, the stage\'s own values', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('solid', 'cube');
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    final titles = [for (final e in find.byType(ValuesCard).evaluate()) (e.widget as ValuesCard).card.name];
    expect(titles, ['Grade', 'Lens', 'Room']);
    final bloom = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_bloom');
    await t.ensureVisible(bloom);
    await t.drag(bloom, const Offset(60, 0));
    await t.pumpAndSettle();
    expect(m.value(0, 'bloom', 0), greaterThan(0), reason: 'the stage glows: ${ws.notice}');
    expect(m.undo() && m.value(0, 'bloom', 0) == 0, isTrue, reason: 'one step');
    // (a colour on a card: its link aims the Colors shelf at it; a swatch picked there is set to it)
    await t.tap(find.byKey(const ValueKey('link:colors:v_background')));
    await t.pumpAndSettle();
    expect(ws.aim, 'background');
    await t.tap(find.descendant(of: find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Strip'), matching: find.byType(DockHover)).first);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Set')));
    await t.pumpAndSettle();
    expect(m.word(0, 'background'), hasLength(6), reason: 'the stage\'s colour behind everything: ${m.trouble}');
    expect(m.say(0, 'tone_mapper', 'agx') && m.word(0, 'tone_mapper') == 'agx', isTrue, reason: m.trouble);
    expect(m.say(0, 'tone_mapper', 'sepia'), isFalse, reason: 'one of Filament\'s own');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a light is a thing of the document, Filament\'s own: made from the Create shelf, its Light card, its colour its fill', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('solid', 'cube');
    ws
      ..reread()
      ..shelf = 'Create';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Spot');
    await t.pumpAndSettle();
    final tile = find.descendant(of: find.byType(CreateShelf), matching: find.text('Spot')).first;
    await t.ensureVisible(tile);
    await t.tap(tile);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Create')));
    await t.pumpAndSettle();
    final spot = int.parse(ws.selected!);
    expect([m.word(spot, 'kind'), m.word(spot, 'file')], ['light', 'spot'], reason: m.trouble);
    expect(find.descendant(of: find.byType(InspHeader), matching: find.text('Light')), findsOneWidget);
    expect([for (final e in find.byType(ValuesCard).evaluate()) (e.widget as ValuesCard).card.name], ['Light']);
    expect(m.set(spot, 'shadows', 1) && m.say(spot, 'fill', 'ffb36b') && m.set(spot, 'cone', 20), isTrue, reason: m.trouble);
    expect(m.set(spot, 'shadows', 2), isFalse, reason: 'shadows is 0 or 1');
    expect(m.key(spot, 'intensity', 0, 0) && m.key(spot, 'intensity', 1, 3), isTrue, reason: 'its strength moves: ${m.trouble}');
    expect(m.add('light', 'lamp'), 0, reason: 'a light is a sun, a point or a spot');
    // (Animate on: a card's number dragged is a key at the playhead, as every change then is; the field shows it)
    expect(m.unkey(spot, 'intensity', 0) && m.unkey(spot, 'intensity', 1), isTrue, reason: m.trouble);
    ws
      ..animating = true
      ..reread();
    await t.pumpAndSettle();
    final strong = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_intensity');
    await t.ensureVisible(strong);
    await t.drag(strong, const Offset(40, 0));
    await t.pumpAndSettle();
    expect(m.keyMoments(spot, 'intensity'), [0], reason: 'a key where the playhead is: ${ws.notice}');
    expect(wsKeyMarks(ws, '$spot')['v_intensity'], KeyS.at);
    ws.animating = false;
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('an After Effects file plays a named stretch of its own (ThorVG\'s Lottie markers), chosen on its Play card', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final pic = m.add('picture', '$motoliiDir/third_party/thorvg_example/res/lottie/swinging.json');
    expect(pic, isNot(0), reason: m.trouble);
    ws
      ..reread()
      ..select('$pic');
    await t.pumpAndSettle();
    expect(m.words(pic, 'marker'), ['all', 'Start', 'End'], reason: 'its file\'s own names');
    expect([for (final e in find.byType(ValuesCard).evaluate()) (e.widget as ValuesCard).card.name], contains('Play'));
    expect(m.say(pic, 'marker', 'Start') && m.word(pic, 'marker') == 'Start', isTrue, reason: m.trouble);
    // (a 3D file plays one of its own movements, at its speed)
    final fox = m.add('solid', '$motoliiDir/third_party/filament_src/third_party/models/Fox/Fox.glb');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.words(fox, 'animation'), ['Survey', 'Walk', 'Run'], reason: 'its file\'s own movements');
    expect(m.say(fox, 'animation', 'Run') && m.set(fox, 'speed', 2), isTrue, reason: m.trouble);
    expect(m.cards(fox).firstWhere((c) => c.$1 == 'Play').$2, ['animation', 'speed']);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a recipe is a picture in one step (as Cavalry builds it from a shape, a duplicator and a behaviour): the Create shelf makes it, its knobs on its cards', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    expect(m.recipes.map((r) => r.$1), containsAll(['Wave lines', 'Iris', 'Dot wave', 'Tunnel', 'Burst']));
    ws
      ..reread()
      ..shelf = 'Create';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Iris');
    await t.pumpAndSettle();
    final tile = find.descendant(of: find.byType(CreateShelf), matching: find.text('Iris')).first;
    await t.ensureVisible(tile);
    await t.tap(tile);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Create')));
    await t.pumpAndSettle();
    final iris = int.parse(ws.selected!);
    expect([m.value(iris, 'copies', 0), m.word(iris, 'copy_fill')], [48, 'ff3366 ffd166 1b2fd8 ff3366'], reason: m.trouble);
    expect([for (final e in find.byType(ValuesCard).evaluate()) (e.widget as ValuesCard).card.name], containsAll(['Copies', 'Wave']));
    expect(m.undo() && m.things.isEmpty, isTrue, reason: 'one step');
    expect(m.make('Nope'), 0);
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
