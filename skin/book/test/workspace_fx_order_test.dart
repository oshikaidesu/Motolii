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
}
