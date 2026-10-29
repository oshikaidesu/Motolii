// The window joined to the real inside (filament/out/libmotolii.dylib, built by filament/edit.sh), driven as a hand
// would drive it: what a seat does reaches the document, and one act is one step back. Where the inside is not
// built these are skipped.
import 'dart:ffi';
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' as ui show ImageByteFormat;

import 'package:ffi/ffi.dart';

import 'package:flutter/gestures.dart' show PointerDeviceKind, kDoubleTapTimeout;
import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';

import 'door.dart';
import 'package:lab_book/parts/controls.dart' show Segmented;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Ctx, Chooser, GlyphPlate, AnchorCube, AnchorGeometry, LinkMark, ValuesCard, FillStroke, InspHeader, KeyS, NumField, OnOff, PropRow, Space, TextGroup;
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart' show Sect;
import 'package:lab_book/sets/inspector/inspector_direct_transform_canvas.dart' show TransformCanvas;
import 'package:lab_book/sets/inspector/inspector_direct_transform_layout.dart' show TransformLayout;
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/workspace/dock_fonts.dart';
import 'package:lab_book/workspace/dock_make.dart' show CreateShelf;
import 'package:lab_book/workspace/desk_kit.dart' show DeskScrub;
import 'package:lab_book/workspace/dock_parts.dart' show DockFooter, DockHover, DockSearch;
import 'package:lab_book/workspace/workspace.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/stage_view.dart' show StageView, StageHandles;
import 'package:lab_book/workspace/timeline_paint.dart' show WtTracksPainter;
part 'workspace_joined_test_2.dart';
part 'workspace_joined_test_3.dart';
part 'workspace_joined_test_4.dart';

final _built = insideBuilt;
final _font = '$motoliiDir/out/arial_black.ttf';

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
    expect(m.kindOf('$motoliiDir/out/vid/check_film.mp4'), 'film');
    final film = m.add('', '$motoliiDir/out/vid/check_film.mp4'); // (the inside makes of a file what it is)
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
    expect((await m.shownAt([(cube, 'y', 1.5)])).single, 1.5, reason: 'it travels from where it was to where it was put');   // (another moment: asked, as BRIDGE.md rule 7 says)
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
    await doubleTap(t, find.byKey(const ValueKey('ws-top-size')));   // (two presses open its numbers: f052e7e)
    await t.pumpAndSettle();
    await t.enterText(find.byWidgetPredicate((w) => w is EditableText && w.focusNode.hasFocus), '1080x1920');
    await t.testTextInput.receiveAction(TextInputAction.done);
    for (var i = 0; i < 100 && identical(ws.inside, m); i++) {   // (opened again in real time, once the inside has the size: BRIDGE.md rule 6)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await settled(t, ws);
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
    await settled(t, ws);
    expect(t.widget<OnOff>(clip).on, isTrue, reason: 'the switch is on (however the skin draws it)');
    ws.reread();
    await t.pumpAndSettle();
    expect([for (final e in find.byWidgetPredicate((w) => w is CustomPaint && w.painter is WtTracksPainter).evaluate()) for (final r in ((e.widget as CustomPaint).painter as WtTracksPainter).rows) if (r.clipped) r.name], ['Cube'], reason: 'its row marked ↘, as Alight Motion marks a clipped layer');
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
    addTearDown(() {   // (and as it was after: the tests after this one are not in its folder)
      final unsetenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>), int Function(Pointer<Utf8>)>('unsetenv');
      final n = 'MOTOLII_OWN'.toNativeUtf8();
      unsetenv(n);
      malloc.free(n);
    });
    final ws = await _pump(t), m = ws.inside!;
    for (var i = 0; i < 60 && !m.wayNames.contains('Brisk'); i++) {   // (the folders are looked at again twice a second: waited for till it is seen, in real time)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
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

  testWidgets('two whole-screen effects with a knob of one name are two values: Posterize\'s gamma and Gamma Correction\'s', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    String file(String name) => m.effects.firstWhere((e) => e.$2.endsWith('/$name.fs')).$2;   // (Vidvox's own ISF files, by their own names; Boxinator was taken out, d825350, CC BY-NC-SA in it)
    expect(m.say(0, 'script', '${file('Posterize')} ${file('Gamma Correction')}'), isTrue, reason: m.trouble);
    final names = m.values(0).where((n) => n.endsWith('_gamma')).toList()..sort();
    expect(names, ['js_Gamma_Correction_gamma', 'js_Posterize_gamma']);
    final other = m.value(0, names[0], 0);
    expect(m.value(0, names[1], 0), 1.25, reason: 'Posterize\'s as its own head says');
    expect(m.set(0, 'js_Posterize_gamma', 1.5), isTrue, reason: m.trouble);
    expect([m.value(0, names[0], 0), m.value(0, names[1], 0)], [other, 1.5], reason: 'the other stays');
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
    final linear = (await m.shownAt([(box, 'x', .5)])).single!;   // (another moment: asked, BRIDGE.md rule 7)
    expect(m.way(box, 'x', 0, 'Snappy'), isTrue, reason: m.trouble);
    expect((await m.shownAt([(box, 'x', .5)])).single, closeTo(2 * bezierY(const Offset(.2, 0), const Offset(0, 1), .5), 1e-3), reason: 'the stretch goes by it');
    expect(m.ways, contains('defined here: Snappy'), reason: 'written into the document, which goes so wherever it is opened');
    expect(m.undo() && (await m.shownAt([(box, 'x', .5)])).single == linear && !m.ways.contains('defined here: Snappy'), isTrue, reason: 'one step: the way and its writing');
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
    expect((await m.shownAt([(text, 'x', 2)])).single, 1);   // (another moment: asked, BRIDGE.md rule 7)
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

  _tests2();
  _tests3();
  _tests4();
}
