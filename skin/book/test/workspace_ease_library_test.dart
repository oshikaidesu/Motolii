// The Ease desk's ways under the sample's eight (workspace/desk_ease_library.dart): a table picked (★, Motolii,
// anime.js), anime.js's one tile a family with the way in or out picked beside them, the tiles scrolling sideways in
// an order that never moves, the starred a table of their own, the last used set apart at the head, both kept in a file.
import 'dart:io';

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_library.dart';

import 'door.dart';

Finder _in(Finder f) => find.descendant(of: find.byType(DeskEase), matching: f);

void main() {
  testWidgets('anime.js\'s ways are a tile a family, its way in or out picked; a star keeps one in ★ without moving it; the last used come first, set apart', (t) async {
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
    expect(_in(find.text('anime.js')), findsOneWidget, reason: 'the tables');
    await t.tap(_in(find.text('anime.js')));
    await t.pumpAndSettle();
    await t.tap(_in(find.text('in')));
    await t.pumpAndSettle();
    Iterable<String> shown() => t.widgetList(_in(find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('ease.anime.')))).map((w) => (w.key as ValueKey<String>).value.substring(5));
    final firsts = shown().toList();
    expect(firsts.take(2), ['anime.inElastic', 'anime.inBounce'], reason: 'way in: only what a bezier cannot draw, in easings.net\'s order');
    expect(find.descendant(of: find.byKey(const ValueKey('ease.anime.inElastic')), matching: find.text('Elastic')), findsOneWidget, reason: 'called by its family: the way is picked beside');
    expect(firsts.where((n) => n.startsWith('anime.out')), isEmpty, reason: 'no way out shown while in is picked');
    expect(find.byKey(const ValueKey('ease.library.anime.js')), findsOneWidget, reason: 'one row that scrolls sideways');
    final list = t.widget<ListView>(find.byKey(const ValueKey('ease.library.anime.js')));
    expect(list.scrollDirection, Axis.horizontal);
    // a star: first in the row, and in the file
    final quint = find.byKey(const ValueKey('ease.star.anime.inBounce'));
    await t.scrollUntilVisible(find.byKey(const ValueKey('ease.anime.inBounce')), 60, scrollable: find.descendant(of: find.byKey(const ValueKey('ease.library.anime.js')), matching: find.byType(Scrollable)));
    await t.pumpAndSettle();
    expect(quint, findsNothing, reason: 'no star until the hand is over the tile');
    final hand = await t.createGesture(kind: PointerDeviceKind.mouse);
    await hand.addPointer(location: t.getCenter(find.byKey(const ValueKey('ease.anime.inBounce'))));
    await t.pumpAndSettle();
    await t.tap(quint);
    await hand.removePointer();
    await t.pumpAndSettle();
    await t.drag(find.byKey(const ValueKey('ease.library.anime.js')), const Offset(3000, 0));   // (back to the row's head)
    await t.pumpAndSettle();
    expect(shown().take(2), ['anime.inElastic', 'anime.inBounce'], reason: 'a star moves nothing: where a tile is stays');
    expect(kept.readAsStringSync(), contains('anime.inBounce'), reason: 'kept for the next time');
    await t.tap(find.text('★'));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('ease.anime.inBounce')), findsOneWidget, reason: 'the starred, a table of their own');
    await t.tap(find.text('anime.js'));
    await t.pumpAndSettle();
    await t.scrollUntilVisible(find.byKey(const ValueKey('ease.anime.inBounce')), 60, scrollable: find.descendant(of: find.byKey(const ValueKey('ease.library.anime.js')), matching: find.byType(Scrollable)));
    await t.pumpAndSettle();
    // the tile on the stretch
    await t.tap(find.byKey(const ValueKey('ease.anime.inBounce')));
    await settled(t, ws);
    expect(m.wayOf(cube, 'x', 0), 'anime.inBounce', reason: 'the stretch goes by it');
    expect((await m.shownAt([(cube, 'x', .5)])).single, closeTo(m.wayAt('anime.inBounce', .5), 1e-3), reason: 'inBounce at half way, as anime.js has it');
    await t.tap(find.text('out'));
    await t.pumpAndSettle();
    await t.drag(find.byKey(const ValueKey('ease.library.anime.js')), const Offset(3000, 0));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('ease.recent.anime.inBounce')), findsOneWidget, reason: 'the last used, at the head while the way out is shown');
    expect(find.byKey(const ValueKey('ease.recent.end')), findsOneWidget, reason: 'set apart');
    expect(kept.readAsStringSync(), contains('"recent":["anime.inBounce"]'));
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
