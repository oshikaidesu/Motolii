// The window joined to the real inside (filament/out/libmotolii.dylib, built by filament/edit.sh), driven as a hand
// would drive it: what a seat does reaches the document, and one act is one step back. Where the inside is not
// built these are skipped.
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';

import 'door.dart';
import 'package:lab_book/parts/controls.dart' show Segmented;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Ctx, Chooser, GlyphPlate, AnchorCube, AnchorGeometry, LinkMark, ValuesCard, FillStroke, InspHeader, KeyS, NumField, OnOff, PropRow, Space, TextGroup;
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/workspace/dock_fonts.dart';
import 'package:lab_book/workspace/dock_make.dart' show CreateShelf;
import 'package:lab_book/workspace/desk_kit.dart' show DeskScrub;
import 'package:lab_book/workspace/dock_parts.dart' show DockFooter, DockHover, DockSearch;
import 'package:lab_book/workspace/workspace.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/ws.dart';

final _built = insideBuilt;
const _font = '$motoliiDir/out/arial_black.ttf';

Future<Ws> _pump(WidgetTester t) async {
  final ws = await openWorkspace(t);
  expect(ws.inside, isNotNull, reason: 'the inside opened');
  return ws;
}

Future<void> _close(WidgetTester t) => closeWorkspace(t);

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
    await seen(t, box);
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
    await expectSameAfterReopen(t);
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
    expect(m.put(film, 'opacity', 0, .5), isTrue, reason: m.trouble); // (not being keyed, a value with keys still gets a key there, as After Effects keys it)
    expect([m.value(film, 'opacity', 0), m.keyed(film, 'opacity', 0)], [.5, 2]);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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

    final clip = find.descendant(of: find.byWidgetPredicate((w) => w is PropRow && w.id == 'v_clip'), matching: find.byType(OnOff)); // (the Blending card's switch)
    await seen(t, clip);
    await t.tap(clip);
    await t.pumpAndSettle();
    expect(m.value(cube, 'clip', 0), 1, reason: 'the seat is the document\'s clip');
    for (var i = 0; i < 20 && ws.waiting; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(find.descendant(of: clip, matching: find.text('On')), findsOneWidget);
    expect(find.byWidgetPredicate((w) => w is PropRow && w.id == 'ghost'), findsNothing, reason: 'the document has no ghost: no switch that moves nothing');
    expect(m.undo() && m.value(cube, 'clip', 0) == 0, isTrue, reason: 'one step back: the clip');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('with Animate on, a knob of a thing with keys is keyed where the hand let go, with what the hand made: the window marks no key of its own', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final box = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$box');
    ws.fxToggle('Trim Paths'); // (trim keyed: the thing has keys)
    ws.frame = 15;
    ws.animating = true;
    await t.pumpAndSettle();
    final knob = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_trim_offset');
    await seen(t, knob);
    await t.dragFrom(t.getCenter(knob.first), const Offset(-60, 0));
    await t.pumpAndSettle();
    for (var i = 0; i < 40 && m.keyMoments(box, 'trim_offset').isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(m.keyMoments(box, 'trim_offset'), [.25], reason: 'one key, where the hand let go');
    expect(m.value(box, 'trim_offset', .25), closeTo(-.6, 1e-9), reason: 'holding what the hand made, not what was before');
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a preset is in the maker\'s own folder, not the program\'s: one put there is a tile at once', (t) async {
    final own = Directory.systemTemp.createTempSync('own');
    Directory('${own.path}/ways').createSync();
    File('${own.path}/ways/brisk.way').writeAsStringSync('// Brisk: quick and done\na + (b - a) * bezier(u, .5, 0, .2, 1)\n');
    final setenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('setenv');
    final name = 'MOTOLII_OWN'.toNativeUtf8(), path = own.path.toNativeUtf8();
    setenv(name, path, 1); // (the maker's folder here: a test does not write into the real one)
    malloc
      ..free(name)
      ..free(path);
    final ws = await _pump(t), m = ws.inside!;
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 600))); // (the folders are looked at again twice a second)
    expect(m.wayNames, containsAll(['Brisk', 'Snappy']), reason: 'the maker\'s own and the one the program comes with');
    expect(unseated(ws).map((e) => e.name), contains('Brisk'));
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('an effect\'s knob reads as the inside says (Trim in hundredths) and a keyed one is keyed where the hand let go', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final box = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$box');
    ws.fxToggle('Trim Paths'); // (keys: 1 at 0, 0 at 1 s)
    ws.frame = 90;
    await t.pumpAndSettle();
    final knob = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_trim');
    await seen(t, knob);
    await t.dragFrom(t.getCenter(knob.first), const Offset(60, 0));
    await t.pumpAndSettle();
    for (var i = 0; i < 20 && ws.waiting; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(m.keyMoments(box, 'trim'), [0, 1, 1.5], reason: 'a key where the hand let go, the others as they were');
    expect(m.value(box, 'trim', 1.5), greaterThan(0), reason: 'what the hand made, in the document\'s own units (0 to 1)');
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('two whole-screen effects with a knob of one name are two values: Posterize\'s gamma and Boxinator\'s', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    String file(String name) => m.effects.firstWhere((e) => e.$2.contains(name)).$2;
    expect(m.say(0, 'script', '${file('posterize')} ${file('boxinator')}'), isTrue, reason: m.trouble);
    final names = m.values(0).where((n) => n.endsWith('_gamma')).toList()..sort();
    expect(names, ['js_isf_boxinator_gamma', 'js_isf_posterize_gamma']);
    expect([m.value(0, names[0], 0), m.value(0, names[1], 0)], [-.3, 1.25], reason: 'each as its own head says');
    expect(m.set(0, 'js_isf_posterize_gamma', 1.5), isTrue, reason: m.trouble);
    expect([m.value(0, names[0], 0), m.value(0, names[1], 0)], [-.3, 1.5], reason: 'the other stays');
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a preset of your own is the maker\'s, a file in ways/: a tile of the Ease desk in every document, written into the document when put on a key', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    expect(m.wayNames, contains('Snappy'), reason: 'filament/ways/snappy.way, the maker\'s own');
    expect(unseated(ws).map((e) => e.name), contains('Snappy'), reason: 'a tile, nothing of the window changed');
    for (final u in [.1, .3, .5, .8]) {
      expect(m.wayAt('Snappy', u), closeTo(bezierY(const Offset(.2, 0), const Offset(0, 1), u), 1e-3), reason: 'at $u');
    }
    final box = m.add('solid', 'cube');
    expect(m.key(box, 'x', 0, 0) && m.key(box, 'x', 1, 2), isTrue, reason: m.trouble);
    final linear = m.value(box, 'x', .5)!;
    expect(m.way(box, 'x', 0, 'Snappy'), isTrue, reason: m.trouble);
    expect(m.value(box, 'x', .5), closeTo(2 * bezierY(const Offset(.2, 0), const Offset(0, 1), .5), 1e-3), reason: 'the stretch goes by it');
    expect(m.ways, contains('defined here: Snappy'), reason: 'written into the document, which goes so wherever it is opened');
    expect(m.undo() && m.value(box, 'x', .5) == linear && !m.ways.contains('defined here: Snappy'), isTrue, reason: 'one step: the way and its writing');
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('round 3 of the judges: a thing put aside is not held; Animate all travels from where it was; a value with keys changed not being keyed gets a key there', (t) async {
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

    // (not being keyed: a value with keys gets a key where it is changed, the others as they were (After Effects))
    expect(m.at(1), isTrue);
    final was = [m.value(cube, 'x', 0), m.value(cube, 'x', 2)];
    expect(m.put(cube, 'x', 1, 2.5), isTrue, reason: m.trouble);
    expect([m.value(cube, 'x', 0), m.value(cube, 'x', 1), m.value(cube, 'x', 2)], [was[0], 2.5, was[1]]);
    expect(m.keyed(cube, 'x', 1), 2, reason: 'a key was made there');

    // (the frame of a thing turned about Y is the box of what it covers: a cube seen edge-on is wider than face-on)
    final faceOn = m.frame(cube)!;
    expect(m.set(cube, 'rotate_y', 45), isTrue, reason: m.trouble);
    final edgeOn = m.frame(cube)!;
    expect(edgeOn[2], greaterThan(faceOn[2] * 1.2));
    expect(edgeOn[4], 0);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    final link = find.byKey(const ValueKey('link:colors:v_stroke'));
    await seen(t, link);
    await t.tap(link);
    await t.pumpAndSettle();
    expect(ws.aim, 'stroke', reason: 'the Colors shelf is aimed at its edge, as at a fill');
    expect(m.say(rect, 'stroke', 'ff3366'), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(FillStroke), matching: find.text('#FF3366')), findsWidgets, reason: 'the edge shown as the document says it');
    expect(m.word(rect, 'stroke'), 'ff3366', reason: m.trouble);
    expect(m.set(rect, 'stroke_width', 6), isTrue, reason: m.trouble);
    expect(wsTransform(ws, '$rect')['v_stroke_width'], 6, reason: 'the width is shown as the document says it');
    expect(m.say(rect, 'stroke', ''), isTrue, reason: 'None: no edge');
    expect(m.value(cube, 'stroke_width', 0), isNull, reason: 'a solid thing has no edge to say');
    // (a value the thing has not, changed in the inspector: the top bar says why it was not taken)
    ws.select('$cube');
    await t.pumpAndSettle();
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'op'), findsNothing, reason: 'a solid has no opacity: no knob for it');
    expect(m.set(rect, 'opacity', .5), isTrue);
    expect(m.say(rect, 'stroke', 'grey'), isFalse);
    expect(m.trouble, contains('six digits'));
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Anchor points into the thing: it turns and grows about that point, and it is where it stands', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final before = m.frame(rect)!;
    expect([before[0], before[1]], [closeTo(960, 1), closeTo(540, 1)]);
    await seen(t, find.byType(AnchorCube)); // (the Anchor card's box: 27 points, one click each)
    final box = t.getRect(find.byType(AnchorCube));
    await t.tapAt(box.topLeft + AnchorGeometry(box.size).at(0, 0, 0)); // (its top left corner)
    await settled(t, ws);
    expect([m.value(rect, 'anchor_x', 0), m.value(rect, 'anchor_y', 0), m.value(rect, 'anchor_z', 0)], [-1, 1, 0], reason: 'its top left corner: ${m.trouble}');
    final after = m.frame(rect)!;
    expect(after[0], greaterThan(before[0] + 10), reason: 'its top left corner is where it stands now: it lies right of there');
    expect(after[1], greaterThan(before[1] + 10), reason: 'and below');
    expect(m.undo() && m.value(rect, 'anchor_x', 0) == 0 && m.value(rect, 'anchor_y', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    await pick(t, 'v_parent', 'Rect'); // (the inside's Transform card: Parent, another thing)
    expect(m.value(cube, 'parent', 0), rect, reason: m.trouble);
    final given = m.frame(cube)!;
    expect([given[0], given[1]], [closeTo(before[0], 1), closeTo(before[1], 1)], reason: 'it did not move when it was given a parent');
    expect(m.set(rect, 'x', 0), isTrue);
    expect(m.frame(cube)![0], greaterThan(given[0] + 50), reason: 'it goes with its parent');
    expect(m.set(rect, 'parent', cube.toDouble()), isFalse, reason: 'no ring of parents');
    expect(m.set(cube, 'parent', 0), isTrue);
    expect(m.frame(cube)![0], closeTo(m.frame(cube)![0], 1));
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a text of two lines, its size and its alignment are the inside\'s Text card', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font);
    ws
      ..reread()
      ..select('$text');
    await t.pumpAndSettle();
    final box = find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText));
    await seen(t, box);
    await t.tap(box);
    await t.enterText(box, 'MOTOLII\nmakes moves');
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pumpAndSettle();
    expect(m.word(text, 'text'), 'MOTOLII\nmakes moves', reason: m.trouble);
    expect(ws.layer?.name, 'MOTOLII makes moves', reason: 'listed on one line');
    expect(wsTransform(ws, '$text')['v_size'], closeTo(.5 * m.point, .1), reason: 'its size in points of the picture');
    final centre = find.descendant(of: find.byWidgetPredicate((w) => w is PropRow && w.id == 'v_align'), matching: find.byWidgetPredicate((w) => w is Text && w.data?.toLowerCase() == 'centre')); // (buttons say it in capitals)
    await seen(t, centre);
    await t.tap(centre);
    await settled(t, ws);
    expect(m.value(text, 'align', 0), .5, reason: 'Centre is ThorVG\'s half way: ${m.trouble}');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Effects shelf: Gaussian Blur and Drop Shadow are ThorVG\'s own, put on and taken off; the rest are not on the shelf', (t) async {
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
      await seen(t, row);
      await t.tap(row);
      await t.pumpAndSettle();
    }

    await pick('Gaussian Blur');
    await t.tap(find.text('Add'));
    await t.pumpAndSettle();
    expect(m.value(rect, 'blur', 0), 12, reason: m.trouble);
    expect(ws.fxOf('$rect'), ['Gaussian Blur']);
    expect(wsTransform(ws, '$rect')['v_blur'], 12, reason: 'the Blur card says the document\'s blur');
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

    const none = 'Extrude'; // (one After Effects has and the inside does not)
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), none);
    await t.pumpAndSettle();
    expect(find.text(none), findsOneWidget, reason: 'only the search box\'s own words: $none, which the inside does not do, is not on the shelf');
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), '');
    await t.pumpAndSettle();
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Space: a flat thing is 2D (by the order) or 2.5D (in the world, by depth); a solid thing is 3D', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final ring = m.add('solid', 'ring'), rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final plate = find.byWidgetPredicate((w) => w is GlyphPlate && w.label == '2.5D'); // (the Transform card's three plates)
    await seen(t, plate);
    await t.tap(plate);
    await settled(t, ws);
    expect(m.word(rect, 'space'), '2.5d', reason: m.trouble);
    expect(m.undo() && m.word(rect, 'space').isEmpty, isTrue, reason: 'one step');
    ws.select('$ring');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('choice:v_space')), findsNothing, reason: 'a solid thing is 3D, and only that');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    expect([seen['v_rotate_y'], seen['v_zoom'], seen['v_distance']], [30, 200, closeTo(1.5 * 6 * m.point, .1)], reason: 'the Camera card says the document');
    expect(m.set(m.things[1], 'points', 8) && m.set(m.things[1], 'inner', .7), isTrue, reason: m.trouble);
    expect(m.set(m.things[0], 'points', 2), isFalse, reason: 'a polygon has three points at least');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    await seen(t, tile);
    await t.tap(tile);
    await t.pump(const Duration(milliseconds: 400)); // (two presses would make it: one is known when the second does not come)
    await t.pumpAndSettle();
    await t.tap(find.text('Lay out'));
    await t.pumpAndSettle();
    expect([m.value(star, 'copies', 0), m.value(star, 'copy_rotate', 0)], [8, 45], reason: m.trouble);
    expect(m.undo() && m.value(star, 'copies', 0) == 1, isTrue, reason: 'one step');
    expect(m.set(star, 'copies', 1001), isFalse, reason: 'a thousand at most');
    await search('Grid');
    final grid = find.descendant(of: find.byType(CreateShelf), matching: find.text('Grid')).first;
    await seen(t, grid);
    await t.tap(grid);
    await t.pump(const Duration(milliseconds: 400)); // (two presses would make it: one is known when the second does not come)
    await t.pumpAndSettle();
    await t.tap(find.text('Lay out'));
    await t.pumpAndSettle();
    expect([m.value(star, 'copies', 0), m.value(star, 'copy_columns', 0)], [9, 3], reason: m.trouble);
    expect(t.takeException(), isNull, reason: 'an anchor outside the box is named Custom');
    await expectSameAfterReopen(t);
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
    await seen(t, stop);
    await t.drag(stop, const Offset(-80, 0));
    await t.pumpAndSettle();
    final aperture = m.value(camera, 'aperture', 0)!;
    expect(aperture, inExclusiveRange(1.4, 22), reason: 'the f-number the hand left');
    expect(m.undo() && m.value(camera, 'aperture', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    await seen(t, tile);
    await t.tap(tile);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Fill')));
    await t.pumpAndSettle();
    expect(m.word(rect, 'fill'), 'e974ab f08a5d efcb4e', reason: m.trouble);
    expect(m.say(m.add('solid', 'cube'), 'fill', 'e974ab f08a5d'), isFalse, reason: 'a solid is filled with one colour');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await seen(t, tile);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    final fx = m.thingEffects.length + m.effects.length;
    expect(fx, greaterThan(3));
    expect(find.text('$fx ready'), findsOneWidget);
    ws.shelf = 'Create';
    await t.pumpAndSettle();
    final r = ws.inside!.recipes.length; // (a recipe is a file: the shelf grows by what is in recipes/)
    expect(find.text('${18 + r} ready'), findsOneWidget);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('scale is three numbers of the document, each its own; a flat thing has no scale z (its thickness to come), a solid has', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    Finder knob(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);
    expect(knob('v_scale_z'), findsNothing, reason: 'a flat thing: no depth to scale yet');
    expect(knob('depth'), findsNothing, reason: 'no Depth knob: a thing\'s thickness is its scale z');
    ws.select('$cube');
    await t.pumpAndSettle();
    await seen(t, knob('v_scale_z')); // (the inside's Transform card: each axis its own number)
    await t.pumpAndSettle();
    await t.drag(knob('v_scale_z'), const Offset(60, 0));
    await t.pumpAndSettle();
    expect([m.value(cube, 'scale_x', 0), m.value(cube, 'scale_y', 0)], [m.value(cube, 'scale_z', 0), m.value(cube, 'scale_z', 0)], reason: 'linked, as After Effects\' scale is at first: all three together');
    expect(m.value(cube, 'scale_z', 0), isNot(1), reason: m.trouble);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    expect(ids, containsAll(['v_x', 'v_distance', 'v_zoom']), reason: 'the inside\'s Camera card, as After Effects\' camera');
    expect(ids.intersection({'v_scale_x', 'v_opacity', 'depth'}), isEmpty, reason: 'a camera has none of these');
    expect(inspectorCards(t), ['Camera', 'Projection'], reason: 'where it is and looks, then its lens: orthographic, shifted');
    expect(m.say(cam, 'projection', 'orthographic') && m.key(cam, 'shift_y', 0, 0.2), isTrue, reason: m.trouble);
    final dist = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_distance');
    await seen(t, dist);
    await t.drag(dist, const Offset(60, 0));
    await t.pumpAndSettle();
    expect(m.value(cam, 'distance', 0), isNot(anyOf(isNull, 1.0)), reason: 'the distance is the document\'s: ${ws.notice}');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector\'s own: a card folds and stays folded; Ghost the document has not is not shown; the text box is the picked text\'s', (t) async {
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
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x'), findsOneWidget); // (the inside's Transform card)
    await t.tap(find.text('Transform').first);
    await t.pumpAndSettle();
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x'), findsNothing, reason: 'folded, and it stays so');
    expect(find.byWidgetPredicate((w) => w is PropRow && w.id == 'ghost'), findsNothing, reason: 'Ghost the document has not is not shown');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector\'s effects are the document\'s on the thing: a card each, its fields the numbers the inside has for it', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    ws
      ..fxToggle('Drop Shadow')
      ..fxToggle('Glow');   // (sent without waiting: put on as the window looks again)
    await t.pumpAndSettle();
    expect(inspectorCards(t), containsAll(['Drop Shadow', 'Glow']), reason: 'an effect on it is one of its cards');
    final far = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_shadow_distance'), was = m.value(a, 'shadow_distance', 0);
    await seen(t, far);
    final ids = {for (final e in find.byType(NumField).evaluate()) (e.widget as NumField).id};
    expect(ids, containsAll(['v_shadow_distance', 'v_shadow_angle', 'v_shadow_blur', 'v_shadow_opacity']));
    await t.drag(far, const Offset(40, 0));
    await t.pumpAndSettle();
    expect(m.value(a, 'shadow_distance', 0), isNot(was), reason: 'the distance is the document\'s: ${ws.notice}');
    expect(m.undo(), isTrue);
    expect(m.value(a, 'shadow_distance', 0), was, reason: 'one step');
    ws.fxToggle('Drop Shadow'); // (taken off as the shelf's Remove takes it)
    await t.pumpAndSettle();
    expect([ws.fxOf('$a'), m.word(a, 'shadow')], [['Glow'], ''], reason: 'taken off the document');
    expect(inspectorCards(t).contains('Drop Shadow'), isFalse);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
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
    await seen(t, find.descendant(of: find.byType(ValuesCard), matching: find.text('Type')));
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Type')), findsOneWidget);
    expect(m.cards(text).map((c) => c.$1).contains('Gradient'), isFalse, reason: 'one colour: no ramp to turn');
    expect(m.say(text, 'fill', 'ff3366 1b2fd8') && m.cards(text).map((c) => c.$1).contains('Gradient'), isTrue, reason: 'two: its ramp');
    expect(m.say(text, 'fill', 'ffffff'), isTrue);
    final wide = m.frame(text)![2], letters = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_letter_spacing');
    expect(letters, findsOneWidget);
    await seen(t, letters);
    await t.drag(letters, const Offset(80, 0));
    await t.pumpAndSettle();
    expect(m.value(text, 'letter_spacing', 0), greaterThan(1), reason: 'the letters further apart: ${ws.notice}');
    expect(m.frame(text)![2], greaterThan(wide), reason: 'and the words wider on the stage');
    ws.select('$rect');
    await t.pumpAndSettle();
    await seen(t, find.descendant(of: find.byType(ValuesCard), matching: find.text('Line')));
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Line')), findsOneWidget);
    final round = find.descendant(of: find.byType(ValuesCard), matching: find.text('ROUND')).first; // (the ends: butt, round, square, as the sample's segments write them)
    await seen(t, round);
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
    expect(m.cards(cube).map((c) => c.$1), ['Fill', 'Transform', 'Blending', 'Glass', 'Lit', 'Physics'], reason: 'filled with glass: the glass\'s own card, not the surface\'s');
    expect(m.cards(rect).map((c) => c.$1).contains('Matte'), isFalse, reason: 'not clipped: no matte to say how');
    expect(m.set(rect, 'clip', 1) && m.cards(rect).map((c) => c.$1).contains('Matte'), isTrue);
    expect(m.set(cube, 'ior', 2.2) && m.value(cube, 'ior', 0) == 2.2, isTrue, reason: m.trouble);
    expect(m.say(cube, 'fill', 'ff3366') && m.cards(cube).map((c) => c.$1).join(' ') == 'Fill Transform Blending Surface Lit Physics', isTrue);
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_metallic'), findsOneWidget);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('nothing picked, the inspector is the whole picture\'s: Filament\'s grade, glow and room, the stage\'s own values', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('solid', 'cube');
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    final titles = inspectorCards(t);
    expect(titles, ['Grade', 'Lens', 'Room', 'Physics']);
    final bloom = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_bloom');
    await seen(t, bloom);
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
    await expectSameAfterReopen(t);
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
    await seen(t, tile);
    await t.tap(tile);
    await t.pump(const Duration(milliseconds: 400)); // (two presses would make it: one is known when the second does not come)
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Create')));
    await t.pumpAndSettle();
    final spot = int.parse(ws.selected!);
    expect([m.word(spot, 'kind'), m.word(spot, 'file')], ['light', 'spot'], reason: m.trouble);
    expect(find.descendant(of: find.byType(InspHeader), matching: find.text('Light')), findsOneWidget);
    expect(inspectorCards(t), ['Fill', 'Transform', 'Light'], reason: 'its colour, where it is and how it is turned, as After Effects\' lights have, then its own');
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
    await seen(t, strong);
    await t.drag(strong, const Offset(40, 0));
    await t.pumpAndSettle();
    expect(m.keyMoments(spot, 'intensity'), [0], reason: 'a key where the playhead is: ${ws.notice}');
    expect(wsKeyMarks(ws, '$spot')['v_intensity'], KeyS.at);
    ws.animating = false;
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    expect(inspectorCards(t), contains('Play'));
    expect(m.say(pic, 'marker', 'Start') && m.word(pic, 'marker') == 'Start', isTrue, reason: m.trouble);
    // (a 3D file plays one of its own movements, at its speed)
    final fox = m.add('solid', '$motoliiDir/third_party/filament_src/third_party/models/Fox/Fox.glb');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.words(fox, 'animation'), ['Survey', 'Walk', 'Run'], reason: 'its file\'s own movements');
    expect(m.say(fox, 'animation', 'Run') && m.set(fox, 'speed', 2), isTrue, reason: m.trouble);
    expect(m.cards(fox).firstWhere((c) => c.$1 == 'Play').$2, ['animation', 'speed']);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
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
    await seen(t, tile);
    await t.tap(tile);
    await t.pump(const Duration(milliseconds: 400)); // (a tile that makes on two presses knows one only when the second is not coming)
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Create')));
    await t.pumpAndSettle();
    final iris = int.parse(ws.selected!);
    expect([m.value(iris, 'copies', 0), m.word(iris, 'copy_fill')], [120, 'ff3366 ffd166 1b2fd8 22c55e ff3366'], reason: m.trouble);
    expect(inspectorCards(t), containsAll(['Copies', 'Wave']));
    expect(m.undo() && m.things.isEmpty, isTrue, reason: 'one step');
    // (two presses on the tile make it at once, without the foot's button)
    await t.tap(tile);
    await t.pump(const Duration(milliseconds: 60));
    await t.tap(tile);
    await t.pumpAndSettle();
    expect(m.things.length, 1, reason: 'made by two presses: ${m.trouble}');
    expect(m.word(m.things.single, 'name'), 'Iris');
    expect(m.make('Nope'), 0);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a recipe is a file: one put in recipes/ is on the shelf at once, its knobs on a Script card, a drag on one is the document\'s', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final file = File('$motoliiDir/recipes/zz_window_test.js');
    addTearDown(() => file.existsSync() ? file.deleteSync() : null);
    file.writeAsStringSync('// Window test: dots in a ring\n// make shape circle\n// set copies 24\n// --track@reach:Reach,0,3,1,0.01\n\n'
        'function each(i, n, t, k) { const a = 2 * Math.PI * i / n; return { x: k.reach * Math.cos(a), y: k.reach * Math.sin(a) }; }\n');
    expect(m.recipes.map((r) => r.$1), contains('Window test'), reason: 'on the shelf without a build');
    final dots = m.make('Window test');
    expect(dots, isNot(0), reason: m.trouble);
    ws
      ..reread()
      ..select('$dots');
    await t.pumpAndSettle();
    expect(inspectorCards(t), contains('Script'));
    final knob = find.byWidgetPredicate((w) => w is NumField && w.id.contains('js_reach'));
    await seen(t, knob);
    expect(knob, findsOneWidget, reason: 'ids: ${[for (final e in find.byType(NumField).evaluate()) (e.widget as NumField).id]}');
    await seen(t, knob);
    await t.drag(knob, const Offset(60, 0));
    await t.pumpAndSettle();
    expect(m.value(dots, 'js_reach', 0), isNot(1.0), reason: 'the knob is the document\'s: ${ws.notice}');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a whole-screen effect is a file: the Effects shelf lists effects/, a press puts it over the stage, its knobs on the stage\'s card, a press takes it off', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final kaleido = m.effects.firstWhere((e) => e.$2.endsWith('/kaleido.wgsl'));
    ws
      ..reread()
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    final row = find.byKey(ValueKey('ws-screen-${kaleido.$1}'));
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), kaleido.$1); // (found as a hand finds one among forty: by its name)
    await t.pumpAndSettle();
    await t.tap(row);
    await t.pumpAndSettle();
    expect(m.word(0, 'script'), kaleido.$2, reason: m.trouble);
    expect(find.descendant(of: row, matching: find.text('ON 1')), findsOneWidget);
    final own = 'js_${kaleido.$2.split('/').last.replaceAll('.wgsl', '')}_'; // (a whole-screen effect's knobs: named by the effect too)
    expect(m.cards(0).firstWhere((c) => c.$1 == 'Script').$2, containsAll(['${own}sides', '${own}angle']));
    expect(m.set(0, '${own}sides', 3), isTrue, reason: m.trouble);
    await t.tap(row);
    await t.pumpAndSettle();
    expect(ws.screenFx, isEmpty, reason: 'taken off');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('with a thing picked, an effect of the list goes on it (as the camera sees it, its fx), not on the whole stage; a press takes it off', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    final half = m.effects.firstWhere((e) => e.$2.endsWith('/halftone.wgsl'));
    ws
      ..reread()
      ..select('$cube')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    final row = find.byKey(ValueKey('ws-screen-${half.$1}'));
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), half.$1);
    await t.pumpAndSettle();
    await t.tap(row);
    await t.pumpAndSettle();
    expect(m.word(cube, 'fx'), half.$2, reason: m.trouble);
    expect(m.word(0, 'script'), anyOf('', 'none'), reason: 'the whole stage is left as it was');
    expect(find.descendant(of: row, matching: find.text('ON 1')), findsOneWidget);
    await t.tap(row);
    await t.pumpAndSettle();
    expect(ws.screenFx, isEmpty, reason: 'taken off');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a motion is a file: the Effects shelf lists animators/ for the picked thing, a press gives it (a text then moves letter by letter), its knobs on its card, a press takes it off', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', '/System/Library/Fonts/Supplemental/Arial Black.ttf');
    expect(m.say(text, 'text', 'MOTOLII'), isTrue, reason: m.trouble);
    final spring = m.animators.firstWhere((e) => e.$2.endsWith('/spring_in.js'));
    expect(m.animators.length, 13, reason: 'Cavalry\'s thirteen');
    ws
      ..reread()
      ..select('$text')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    final row = find.byKey(ValueKey('ws-motion-${spring.$1}'));
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), spring.$1);
    await t.pumpAndSettle();
    await t.tap(row);
    await t.pumpAndSettle();
    expect(m.word(text, 'script'), spring.$2, reason: m.trouble);
    expect(find.descendant(of: row, matching: find.text('ON')), findsOneWidget);
    expect(m.cards(text).firstWhere((c) => c.$1 == 'Script').$2, containsAll(['js_length', 'js_delay', 'js_far']));
    await t.tap(row);
    await t.pumpAndSettle();
    expect(ws.motion, isEmpty, reason: 'taken off');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a knob of the stage\'s own cards in its units (Bloom in hundredths): a second drag right goes on from where the first left it, up', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('solid', 'cube');
    ws..reread()..select(null);
    await t.pumpAndSettle();
    final bloom = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_bloom');
    await seen(t, bloom);
    await t.pump();
    for (final by in const [Offset(90, 0), Offset(45, 40)]) {
      final g = await t.startGesture(t.getCenter(bloom));
      for (var i = 1; i <= 10; ++i) { await g.moveBy(by / 10); await t.pump(const Duration(milliseconds: 16)); }
      await g.up();
      await t.pumpAndSettle();
    }
    expect(m.value(0, 'bloom', 0), greaterThan(0.45), reason: 'the second drag began from 45, not from the document\'s 0.45 read as hundredths');
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a knob dragged up or down as well as across scrubs its number and does not scroll the inspector under the hand', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', '/System/Library/Fonts/Supplemental/Arial Black.ttf');
    ws..reread()..select('$text');
    await t.pumpAndSettle();
    final bloom = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_letter_spacing');
    await seen(t, bloom);
    final list = find.ancestor(of: bloom, matching: find.byType(Scrollable)).first;
    final scrolled = t.state<ScrollableState>(list).position.pixels, was = m.value(text, 'letter_spacing', 0) ?? 1;
    expect(t.state<ScrollableState>(list).position.maxScrollExtent, greaterThan(0), reason: 'the inspector is longer than its pane');
    final g = await t.startGesture(t.getCenter(bloom));
    for (var i = 1; i <= 10; ++i) {
      await g.moveBy(const Offset(9, -8));
      await t.pump(const Duration(milliseconds: 16));
    }
    await g.up();
    await t.pumpAndSettle();
    expect(t.state<ScrollableState>(list).position.pixels, scrolled, reason: 'the inspector stays where it was');
    expect(m.value(text, 'letter_spacing', 0), greaterThan(was), reason: 'the number went up');
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('with Animate on, a flat thing\'s scale dragged goes into the document from the first drag, keyed (no key asked of a value it has not, scale_z, to take the step back)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final c = m.add('shape', 'rect');
    ws..reread()..select('$c');
    ws.animating = true;
    await t.pumpAndSettle();
    final k = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_scale_x'); // (the inside's Transform card)
    await seen(t, k);
    var was = m.value(c, 'scale_x', ws.frame / Ws.fps)!;
    for (final (by, more) in const [(Offset(40, 0), true), (Offset(-80, 0), false)]) {
      final g = await t.startGesture(t.getCenter(k));
      for (var i = 1; i <= 3; ++i) {
        await g.moveBy(by / 3);
        await t.pump(const Duration(milliseconds: 16));
      }
      await g.up();
      await t.pumpAndSettle();
      for (var i = 0; i < 20 && ws.waiting; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
        await t.pump();
      }
      final now = m.value(c, 'scale_x', ws.frame / Ws.fps)!;
      expect(more ? now > was : now < was, isTrue, reason: 'dragged by $by: $was to $now, from the first drag');
      expect(m.keyMoments(c, 'scale_x'), isNotEmpty, reason: 'Animate keys it');
      was = now;
    }
    ws.animating = false;
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the stage\'s Follow names a thing by its name (a choice, None first), not by a number dragged', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    final cube = m.add('solid', 'cube');
    expect(m.say(0, 'script', m.effects.firstWhere((e) => e.$2.endsWith('/kaleido.wgsl')).$2), isTrue, reason: m.trouble); // (the stage's Script card: shown with an effect on it)
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_follow'), findsNothing, reason: 'not a knob of numbers');
    final follow = find.byWidgetPredicate((w) => w is Segmented && w.items.contains('Cube')); // (a few things: a segmented choice, None first, by their names)
    await seen(t, follow);
    await t.tap(find.descendant(of: follow, matching: find.textContaining(RegExp('cube', caseSensitive: false))));
    await t.pumpAndSettle();
    expect(m.value(0, 'follow', 0), cube, reason: m.trouble);
    await expectSameAfterReopen(t);
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
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);
}
