// The window joined to the real inside (filament/out/libmotolii.dylib, built by filament/edit.sh), driven as a hand
// would drive it: what a seat does reaches the document, and one act is one step back. Where the inside is not
// built these are skipped.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show OnOff, PropRow, TextGroup;
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
    await t.enterText(box, 'CATS');
    await t.pump();
    expect(m.word(text, 'text'), '', reason: 'nothing is told while the box is still held');
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pumpAndSettle();
    expect(m.word(text, 'text'), 'CATS');
    expect(ws.layer?.name, 'CATS', reason: 'the timeline row is named by the words');

    expect(m.undo(), isTrue);
    expect(m.word(text, 'text'), '');
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

    expect(m.put(film, 'opacity', .5, .25), isTrue, reason: m.trouble); // (it has keys: a key there)
    expect(m.keyed(film, 'opacity', .5), 2);
    expect(m.put(film, 'x', .5, 1), isTrue, reason: m.trouble); // (it has none: the value that stays)
    expect(m.keyed(film, 'x', .5), 0);

    // (the window judge's fifth finding) a value's first key takes the place of the value that stayed: no key appears at 0,
    // and a second change at another moment is a move between the two
    final cube = m.add('solid', 'cube');
    expect(m.put(cube, 'x', 1, 2) && m.keyHere(cube, 'x', 1) && m.keyHere(cube, 'y', 1), isTrue, reason: m.trouble);
    expect([m.keyMoments(cube, 'x'), m.keyMoments(cube, 'y')], [[1.0], [1.0]], reason: 'one key each, where the hand was');
    expect(m.put(cube, 'y', 2, 3), isTrue, reason: m.trouble);
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
    await t.dragFrom(Offset(tracks.left + tracks.width * .3, row), Offset(perFrame * 30, 0));
    await t.pumpAndSettle();
    final start = m.value(cube, 'start', 0)!;
    expect(start, greaterThan(.5), reason: 'slid to the right: it comes on later');
    expect(m.lasts(cube), lessThan(0), reason: 'still on for good');
    final tail = t.getRect(find.byKey(const ValueKey('ws-tl-tracks'))).right - 1;
    expect(ws.layers.last.outF, Ws.duration);
    expect(m.trim(cube, start, start + 2), isTrue, reason: m.trouble);
    expect(m.lasts(cube), 2);
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
    expect(again.undo(), isTrue, reason: 'and its steps are still there');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a flat shape is a thing; Clip to below shows a thing only inside the one before it; a seat with nothing behind goes back', (t) async {
    final ws = await _pump(t), m = ws.inside!;
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
