// The window's keys as Ableton Live 12 has them (its manual, chapter 42; workspace/ableton.dart): Space plays from where
// it began and stops back there, Shift+Space goes on; the arrows nudge what is picked, or the playhead; up and down go
// between rows; Cmd+R renames in the row; Cmd+L loops; + and - zoom, Z zooms to what is picked and X goes back.
import 'dart:io';

import 'package:flutter/gestures.dart' show kSecondaryButton, PointerDeviceKind;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/ableton.dart';
import 'package:lab_book/workspace/keys.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/timeline_paint.dart';

import 'door.dart';

const _size = Size(1226, 300);

/// A new document: a rect keyed on x at 0.8 and 1.6 s, a ring on from 1 s.
({Ws ws, int rect, int ring}) _joined() {
  final m = Motolii.open('${Directory.systemTemp.createTempSync('ableton').path}/ableton.motolii')!;
  addTearDown(m.close);
  final ring = m.add('solid', 'ring'), rect = m.add('shape', 'rect');
  expect(m.set(ring, 'start', 1) && m.key(rect, 'x', .8, 0) && m.key(rect, 'x', 1.6, 1), isTrue, reason: m.trouble);
  return (ws: Ws()..join(m), rect: rect, ring: ring);
}

Future<void> _pump(WidgetTester t, Ws ws) async {
  t.view.physicalSize = _size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.binding.setSurfaceSize(_size);
  await t.pumpWidget(WidgetsApp(
    color: const Color(0xFF000000),
    builder: (_, _) => WsScope(ws: ws, child: Align(alignment: Alignment.topLeft, child: SizedBox(width: _size.width, height: _size.height, child: Overlay(initialEntries: [OverlayEntry(builder: (_) => WsTimeline())])))),   // (the window brings its own overlay: workspace.dart)
  ));
  await t.pumpAndSettle();
}

/// The frame the ruler is at, at x: a hand comes down there and drags a little away and back (a drag scrubs).
Future<int> _scrubAt(WidgetTester t, Ws ws, double x) async {
  final y = WtM.header + WtM.ruler - 6;
  final g = await t.startGesture(Offset(x, y));
  await g.moveBy(const Offset(30, 0));
  await t.pump();
  await g.moveBy(const Offset(-30, 0));
  await t.pump();
  await g.up();
  await t.pump(const Duration(milliseconds: 400));
  return ws.frame;
}

/// How many frames lie across the ruler between x = 400 and x = 1000.
Future<int> _across(WidgetTester t, Ws ws) async => await _scrubAt(t, ws, 1000) - await _scrubAt(t, ws, 400);

void main() {
  test('the keys are Live\'s: Cmd+R renames, Cmd+Shift+R exports, Space plays from the start, the arrows nudge', () {
    final k = Keymap.load('/nonexistent');
    String? of(String key) => k.pairs.where((p) => p.$1 == activatorOf(key)).firstOrNull?.$2;
    bool same(SingleActivator a, SingleActivator b) => a.trigger == b.trigger && a.meta == b.meta && a.shift == b.shift;
    String? said(String key) => k.pairs.where((p) => same(p.$1, activatorOf(key)!)).firstOrNull?.$2;
    expect(of('cmd+r') ?? said('cmd+r'), 'thing.rename');
    expect(said('cmd+shift+r'), 'file.render');
    expect(said('space'), 'play.fromStart');
    expect(said('shift+space'), 'play.continue');
    expect(said('left'), 'nudge.left');
    expect(said('up'), 'layer.previous');
    expect(said('cmd+l'), 'loop.toggle');
    expect(said('z'), 'zoom.selection');
    expect(said(','), 'frame.previous');
  });

  testWidgets('Space plays from where it began and stops back there; Shift+Space goes on from where it stopped', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    ws.frame = 30;
    does['play.fromStart']!();
    expect(ws.playing, isTrue);
    ws.frame = 50;   // (as playing carries it)
    does['play.fromStart']!();
    expect((ws.playing, ws.frame), (false, 30), reason: 'stopped back at the start');
    ws.frame = 40;
    does['play.continue']!();
    ws.frame = 60;
    does['play.continue']!();
    expect((ws.playing, ws.frame), (false, 60), reason: 'stays where it stopped');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the arrows nudge the picked keys, else the picked bar, else the playhead; up and down go between rows', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    Future<void> answered() async {   // (asked without waiting (later): done in the inside's turn, later under a loaded machine, as the locators' test waits;
      //  it read the key before the inside moved it, three times at a whole run's heavy start: golf_noise.md, the test wobbles)
      for (var i = 0; i < 200 && ws.waiting; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
        await t.pump();
      }
      await t.pumpAndSettle();
    }
    final f = (.8 * Ws.fps).round();
    ws.pickKey((layer: '${j.rect}', frame: f));
    does['nudge.right']!();
    await answered();
    ws.reread();
    expect(ws.layers.firstWhere((l) => l.id == '${j.rect}').keys, contains(f + 1), reason: 'the key went a frame later');
    ws.select('${j.ring}');
    final inF = ws.layer!.inF;
    does['nudge.left10']!();
    await answered();
    ws.reread();
    expect(ws.layers.firstWhere((l) => l.id == '${j.ring}').inF, inF - 10, reason: 'the bar went 10 frames earlier');
    ws.select(null);
    ws.frame = 20;
    does['nudge.right']!();
    expect(ws.frame, 21, reason: 'nothing picked: the playhead');
    ws.select(ws.layers.first.id);
    does['layer.next']!();
    expect(ws.selected, ws.layers[1].id);
    does['layer.previous']!();
    expect(ws.selected, ws.layers.first.id);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('Cmd+R writes the picked thing\'s name in its row; Enter keeps it, Esc leaves it as it was', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    ws.select('${j.rect}');
    does['thing.rename']!();
    await t.pumpAndSettle();
    final field = find.byKey(ValueKey('ws-tl-rename-${j.rect}'));
    expect(field, findsOneWidget);
    await t.enterText(field, 'Hero');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    ws.reread();
    await t.pumpAndSettle();
    expect(ws.layers.firstWhere((l) => l.id == '${j.rect}').name, 'Hero');
    does['thing.rename']!();
    await t.pumpAndSettle();
    await t.enterText(field, 'Nope');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.layers.firstWhere((l) => l.id == '${j.rect}').name, 'Hero', reason: 'Esc keeps the name');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('+ and - zoom, W fits, Z zooms to the picked keys and X goes back, Cmd+L loops', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    final fit = await _across(t, ws);
    does['zoom.in']!();
    await t.pumpAndSettle();
    final zoomed = await _across(t, ws);
    expect(zoomed, lessThan(fit), reason: '+ zooms in');
    does['zoom.width']!();
    await t.pumpAndSettle();
    expect(await _across(t, ws), fit, reason: 'W fits the width');
    ws.pickKeys([(layer: '${j.rect}', frame: (.8 * Ws.fps).round()), (layer: '${j.rect}', frame: (1.6 * Ws.fps).round())]);   // (the rect's bar is the whole document: its keys are less)
    does['zoom.selection']!();
    await t.pumpAndSettle();
    expect(await _across(t, ws), lessThan(fit), reason: 'Z zooms to the picked keys');
    does['zoom.back']!();
    await t.pumpAndSettle();
    expect(await _across(t, ws), fit, reason: 'X goes back');
    final looping = ws.inside!.looping;
    does['loop.toggle']!();
    await t.pumpAndSettle();
    expect(ws.inside!.looping, !looping);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('locators as Live\'s, by the mouse alone: the ruler\'s right-click menu adds one, a click goes to it and picks it, its own menu names it, carried it keeps where it was taken, the Marker button sets and takes away', (t) async {
    final j = _joined(), ws = j.ws, does = wsAbletonCommands(ws);
    await _pump(t, ws);
    Future<void> menu(Offset at, String item) async {
      await t.tapAt(at, buttons: kSecondaryButton);
      await t.pumpAndSettle();
      await t.tap(find.byKey(ValueKey('ws-menu-$item')));
      await t.pumpAndSettle();
      for (var i = 0; i < 200 && ws.waiting; i++) {   // (asked without waiting (later): done in the inside's turn, later under a loaded machine)
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
        await t.pump();
      }
      ws.reread();
    }
    await menu(Offset(600, WtM.header + WtM.ruler - 6), 'Add Marker');
    expect(ws.marks.length, 1, reason: 'the ruler\'s menu adds a locator (6.4: Add Locator in the scrub area\'s context menu)');
    final (at, name) = ws.marks.single;
    expect(name, 'M1');
    expect(wsMarkPicked.value, ws.marks.single, reason: 'picked once added');
    ws.frame = 0;
    final flag = Offset(600.0 + 3, WtM.header + WtM.band + 6);   // (on its flag, just right of where it stands)
    await t.pump(const Duration(seconds: 1));
    await t.tapAt(flag);
    await t.pump(const Duration(milliseconds: 400));
    expect(ws.frame, (at * Ws.fps).round(), reason: 'a click on the flag goes there');
    expect(wsMarkPicked.value, isNotNull, reason: 'and picks it');
    await menu(flag, 'Rename…');
    final words = t.widget<EditableText>(find.descendant(of: find.byKey(const ValueKey('ws-tl-mark-name')), matching: find.byType(EditableText))).controller;
    expect(words.selection, const TextSelection(baseOffset: 0, extentOffset: 2), reason: 'its words picked: what is typed takes their place (Final Cut Pro\'s marker)');
    await t.enterText(find.byKey(const ValueKey('ws-tl-mark-name')), 'Drop');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    for (var i = 0; i < 200 && ws.waiting; i++) {   // (as menu above: settled is not done, under a loaded machine)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
      await t.pump();
    }
    ws.reread();
    expect(ws.marks.single.$2, 'Drop', reason: 'the flag\'s menu names it');
    await t.tapAt(flag);
    await t.pump(const Duration(milliseconds: 50));
    await t.tapAt(flag);
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('ws-tl-mark-name')), findsOneWidget, reason: 'a double click on a marker writes its name, as the best-selling editors (Premiere Pro, Resolve, Final Cut Pro)');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.marks.single.$2, 'Drop', reason: 'left as it was');
    Future<double> carry(Offset from) async {   // (20 points to the right, then put back)
      final g = await t.startGesture(from, kind: PointerDeviceKind.mouse);
      await t.pump();
      for (var i = 1; i <= 4; i++) {
        await g.moveBy(const Offset(5, 0));
        await t.pump();
      }
      await g.up();
      await settled(t, ws);   // (the carry answered by the inside before it is read: under the whole set's load a read came first, 0 carried)
      ws.reread();
      final moved = (ws.marks.single.$1 - at) * Ws.fps;
      ws.inside!.undo();
      await t.pumpAndSettle();
      ws.reread();
      return moved;
    }
    final nearLine = await carry(flag), farRight = await carry(flag + const Offset(14, 0));
    expect(nearLine, greaterThan(3), reason: 'carried');
    expect(farRight, closeTo(nearLine, 1), reason: 'carried as far as the hand went wherever on its flag it was taken, not to where the hand is');
    ws.frame = 0;
    await t.pumpAndSettle();
    final shaken = await t.startGesture(flag, kind: PointerDeviceKind.mouse);   // (a click whose hand shook: taken for a drag, let go where it was)
    await t.pump();
    await shaken.moveBy(const Offset(3, 0));
    await t.pump();
    await shaken.moveBy(const Offset(-3, 0));
    await t.pump();
    await shaken.up();
    await t.pumpAndSettle();
    ws.reread();
    expect([ws.frame, ws.marks.single.$1], [(at * Ws.fps).round(), at], reason: 'a click all the same: it goes there, the marker stays (golf_noise.md (88))');
    ws.frame = (ws.marks.single.$1 * Ws.fps).round();
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ws-tl-mark')));
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.marks, isEmpty, reason: 'on a marker, the Marker button takes it away (Live\'s Set becomes Delete Locator)');
    await t.tap(find.byKey(const ValueKey('ws-tl-mark')));
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.marks.single.$1 * Ws.fps, closeTo(ws.frame, .5), reason: 'elsewhere it sets one where the playhead is (Set Locator)');
    await menu(Offset(300, WtM.header + WtM.ruler - 6), 'Add Marker and Name…');
    expect(find.byKey(const ValueKey('ws-tl-mark-name')), findsOneWidget, reason: 'added with its name opened (Final Cut Pro\'s Add Marker and Modify)');
    await t.enterText(find.byKey(const ValueKey('ws-tl-mark-name')), 'Build');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    ws.reread();
    expect(ws.marks.map((k) => k.$2), contains('Build'));
    await t.pumpWidget(const SizedBox());
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('a mark takes a colour from its flag\'s menu, as Premiere Pro\'s, Resolve\'s and CapCut\'s markers; carried it keeps it, taken away it goes with it (golf ㉕)', (t) async {
    final ws = _joined().ws, m = ws.inside!;
    await _pump(t, ws);
    Future<void> settle() async {
      for (var i = 0; i < 200 && ws.waiting; i++) {   // (asked without waiting (later))
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
        await t.pump();
      }
      ws.reread();
      await t.pumpAndSettle();
    }
    Future<void> menu(Offset at) async {
      await t.tapAt(at, buttons: kSecondaryButton);
      await t.pumpAndSettle();
    }
    await menu(Offset(600, WtM.header + WtM.ruler - 6));
    await t.tap(find.byKey(const ValueKey('ws-menu-Add Marker')));
    await settle();
    final at = ws.marks.single.$1, flag = Offset(600.0 + 3, WtM.header + WtM.band + 6);
    await menu(flag);
    expect(find.byKey(const ValueKey('ws-menu-colour-15')), findsOneWidget, reason: 'its menu ends in the sixteen label colours');
    await t.tap(find.byKey(const ValueKey('ws-menu-colour-2')));
    await settle();
    expect(ws.markColours, {at: 3}, reason: 'the third, kept in the document');
    final g = await t.startGesture(flag, kind: PointerDeviceKind.mouse);   // (carried 20 points to the right)
    await t.pump();
    for (var i = 1; i <= 4; i++) {
      await g.moveBy(const Offset(5, 0));
      await t.pump();
    }
    await g.up();
    await settle();
    final to = ws.marks.single.$1;
    expect([to, ws.markColours], [greaterThan(at), {to: 3}], reason: 'carried: its colour with it, one step');
    await menu(flag + const Offset(20, 0));
    await t.tap(find.byKey(const ValueKey('ws-menu-Delete')));
    await settle();
    expect([ws.marks, ws.markColours], [isEmpty, isEmpty], reason: 'taken away: its colour too');
    m.undo();
    await settle();
    expect(ws.markColours, {to: 3}, reason: 'undone: both back');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);
}
