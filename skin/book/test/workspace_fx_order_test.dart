// A thing's effects are one stack in any order, ThorVG's own and the files' alike (After Effects' Effect Controls):
// their cards stand in the order they are laid, and a card's arrows move its effect up or down, one step.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show ValuesCard;

import 'door.dart';

List<String> _cards(WidgetTester t) => [for (final e in find.byType(ValuesCard).evaluate()) (e.widget as ValuesCard).card.name];

void main() {
  testWidgets('a blur and a halftone: cards in the order laid; the blur moved down lays after the halftone', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final halftone = '$motoliiDir/effects/halftone.wgsl';
    final rect = m.add('shape', 'rect');
    expect(m.effect(rect, 'Gaussian Blur', 0), isTrue, reason: m.trouble);
    ws..reread()..select('$rect');
    expect(ws.screenToggle(halftone), isTrue);
    await t.pumpAndSettle();
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
    await t.pumpAndSettle();
    expect(m.word(rect, 'fx'), 'blur $halftone', reason: 'the file put at the end of its effects');
    final down = find.byKey(const ValueKey('card-move:Gaussian Blur:1'));
    await seen(t, down);   // (the effects' cards are last: scrolled to)
    final cards = _cards(t);
    expect(cards.indexOf('Gaussian Blur'), allOf(greaterThan(-1), lessThan(cards.indexOf('Halftone'))), reason: 'cards in the order laid: $cards');
    await t.tap(down);
    await t.pumpAndSettle();
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
    await t.pumpAndSettle();
    expect(m.word(rect, 'fx'), '$halftone blur', reason: 'moved after the halftone');
    await seen(t, find.byKey(const ValueKey('card-move:Gaussian Blur:-1')));
    final after = _cards(t);
    expect(after.indexOf('Halftone'), lessThan(after.indexOf('Gaussian Blur')), reason: 'and its card with it: $after');
    expect(m.undo(), isTrue);
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100))); // (a read the busy hand refuses shows the copy read before)
    await t.pumpAndSettle();
    expect(m.word(rect, 'fx'), 'blur $halftone', reason: 'one step back');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Remove on an effect\'s card takes that effect off, a file\'s or ThorVG\'s own, one step (Alight Motion\'s Delete on an effect)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final halftone = '$motoliiDir/effects/halftone.wgsl';
    final rect = m.add('shape', 'rect');
    expect(m.effect(rect, 'Gaussian Blur', 0), isTrue, reason: m.trouble);
    ws..reread()..select('$rect');
    expect(ws.screenToggle(halftone), isTrue);
    await t.pumpAndSettle();
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('card-remove:Transform')), findsNothing, reason: 'the thing\'s own cards are not taken off');
    for (final (card, left) in [('Halftone', 'blur'), ('Gaussian Blur', '')]) {
      final remove = find.byKey(ValueKey('card-remove:$card'));
      await seen(t, remove);
      await t.tap(remove);
      for (var i = 0; i < 100 && m.word(rect, 'fx') != left; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
        await t.pump();
      }
      expect(m.word(rect, 'fx'), left, reason: '$card taken off: ${m.trouble}');
      await t.pumpAndSettle();
      expect(_cards(t), isNot(contains(card)), reason: 'and its card gone');
    }
    expect(m.value(rect, 'blur', 0) ?? 0, 0, reason: 'the blur unsaid, as before it was put on');
    expect(m.undo(), isTrue);
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
    expect(m.value(rect, 'blur', 0), 12, reason: 'one step back: the blur on again');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Hide on an effect\'s card turns it off, its card and values kept, and Show turns it on again (Alight Motion\'s eye on an effect)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final halftone = '$motoliiDir/effects/halftone.wgsl';
    final rect = m.add('shape', 'rect');
    ws..reread()..select('$rect');
    expect(ws.screenToggle(halftone), isTrue);
    await t.pumpAndSettle();
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
    await t.pumpAndSettle();
    for (final (word, off) in [('hide', 'Halftone'), ('show', '')]) {
      final eye = find.byKey(ValueKey('card-$word:Halftone'));
      await seen(t, eye);
      await t.tap(eye);
      for (var i = 0; i < 100 && m.word(rect, 'fx_off') != off; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
        await t.pump();
      }
      expect(m.word(rect, 'fx_off'), off, reason: '$word: ${m.trouble}');
      expect(m.word(rect, 'fx'), halftone, reason: 'the effect itself kept on the thing');
      await t.pumpAndSettle();
      expect(_cards(t), contains('Halftone'), reason: 'and its card, its values there to be changed');
    }
    await t.tap(find.byKey(const ValueKey('card-more:Halftone')));   // (its ⋮: Effect Info, in its maker's words, as AbsentMotion's)
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('card-info:Halftone')));
    await t.pumpAndSettle();
    expect(ws.notice, startsWith('Halftone: '), reason: 'what it does said');
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
