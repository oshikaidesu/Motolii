// The window joined to the real inside (filament/out/libmotolii.dylib, built by filament/edit.sh), driven as a hand
// would drive it: what a seat does reaches the document, and one act is one step back. Where the inside is not
// built these are skipped.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show TextGroup;
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

  testWidgets('Backspace on the timeline takes the picked thing out; undo brings it back', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final ring = m.add('solid', 'ring'), cube = m.add('solid', 'cube');
    expect(m.say(cube, 'fill', 'glass'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(ws.layers.map((l) => l.name), ['Ring', 'Cube']);

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
