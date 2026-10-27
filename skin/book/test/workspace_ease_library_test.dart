// The Ease desk's ways under the sample's eight (workspace/desk_ease_library.dart): a table picked (All or ★, as the
// font shelf picks one), the tiles scrolling sideways in an order that never moves, the starred a table of their own,
// the last used set apart at the head, both kept in a file.
import 'dart:io';

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_library.dart';

import 'door.dart';

Finder _in(Finder f) => find.descendant(of: find.byType(DeskEase), matching: f);

void main() {
  testWidgets('the ways a table each, All or ★; a star keeps one in ★ without moving it; the last used come first, set apart', (t) async {
    final kept = File('${Directory.systemTemp.createTempSync('stars').path}/ease_stars.json');
    easeStarsAt(kept);
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('ease.tables')), findsOneWidget, reason: 'the tables');
    final library = find.byKey(const ValueKey('ease.library.All'));
    expect(library, findsOneWidget, reason: 'one row that scrolls sideways');
    expect(t.widget<ListView>(library).scrollDirection, Axis.horizontal);
    Iterable<String> shown() => t.widgetList(find.descendant(of: find.byType(EaseLibrary), matching: find.byWidgetPredicate((w) => w.key is ValueKey<String> && RegExp(r'^ease\.(?!recent|star|library|tables)').hasMatch((w.key as ValueKey<String>).value))))
        .map((w) => (w.key as ValueKey<String>).value.substring(5));
    final firsts = shown().take(2).toList();
    expect(firsts, hasLength(2), reason: 'the ways the sample has no seat for: ${m.wayNames}');
    final way = firsts[1];
    // a star: shown only under the hand, kept in the file, moving nothing
    final star = find.byKey(ValueKey('ease.star.$way'));
    expect(star, findsNothing, reason: 'no star until the hand is over the tile');
    final hand = await t.createGesture(kind: PointerDeviceKind.mouse);
    await hand.addPointer(location: t.getCenter(find.byKey(ValueKey('ease.$way'))));
    await t.pumpAndSettle();
    await t.tap(star);
    await hand.removePointer();
    await t.pumpAndSettle();
    expect(shown().take(2), firsts, reason: 'a star moves nothing: where a tile is stays');
    expect(kept.readAsStringSync(), contains(way), reason: 'kept for the next time');
    await t.tap(_in(find.text('★')));
    await t.pumpAndSettle();
    expect(find.byKey(ValueKey('ease.$way')), findsOneWidget, reason: 'the starred, a table of their own');
    expect(shown(), [way]);
    await t.tap(_in(find.text('All')));
    await t.pumpAndSettle();
    // the tile on the stretch, then at the head, set apart
    await t.tap(find.byKey(ValueKey('ease.$way')));
    await settled(t, ws);
    final put = m.wayOf(cube, 'x', 0);
    expect(put == way || put.startsWith('$way '), isTrue, reason: 'the stretch goes by it (a kind with its numbers, as the library tells its own: desk_ease_library.dart): $put');
    expect((await m.shownAt([(cube, 'x', .5)])).single, closeTo(m.wayAt(put, .5), 1e-3), reason: 'as the way has it half way');
    expect(find.byKey(ValueKey('ease.recent.$way')), findsOneWidget, reason: 'the last used, at the head');
    expect(find.byKey(const ValueKey('ease.recent.end')), findsOneWidget, reason: 'set apart');
    expect(kept.readAsStringSync(), contains('"recent":["$way"]'));
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
