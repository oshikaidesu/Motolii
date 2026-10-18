// The cards-by-hand table (cards_table.dart), one use case for each thing the hand does; every use case is the whole table.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'cards_table.dart';

class _Keep extends StatefulWidget {
  const _Keep(this.make, this.tag);
  final CardTable Function() make;
  final String tag;
  @override
  State<_Keep> createState() => _KeepState();
}

class _KeepState extends State<_Keep> {
  late final t = widget.make();
  @override
  void dispose() {
    t.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => CardsTable(t);
}

/// The ring with its wave up (40 px), so what the wave card does shows.
CardTable _waving() => starRing()..valOr('waveH')!.v = 40;

/// The same, the wave card laid before the copies.
CardTable _waveFirst() {
  final t = _waving(), w = t.cards.firstWhere((c) => c.name == 'Wave');
  return t..cards.remove(w)..cards.insert(1, w);
}

/// The use cases, in the order they were made; each is the whole table, only its start differs.
final cardUses = <(String, CardTable Function(), String)>[
  ('Lay', starRing, 'a hand card laid on a value: shown while it hovers, the same again wider, another swaps it, its mark carried off'),
  ('Flip', () => starRing()..cards[0].flipped = true..cards[1].flipped = true, 'a card turned over: every number'),
  ('Order', _waveFirst, 'the order of the cards is what is done first (wave first: the whole ring; copies first: each copy)'),
  ('Face down', () => _waving()..cards[2].down = true, 'a card face down does nothing'),
  ('Throw', () => starRing(wave: false), 'a shelf card thrown on the stage is worn; its head carried to the shelf takes it off'),
  ('Stage ring', starRing, 'a hand card resting on the stage opens a ring of values; the way the hand goes picks one'),
  ('Group', starRing, 'a tile held by its name and dropped on another ties them; the group scrubs them all'),
  ('Link', manyThings, "one thing's value moves many others': kept in the hand, laid on one thing's value, then tapped on the next things; or carried onto a thing on the stage"),
];

WidgetbookComponent cardsSet() => WidgetbookComponent(
  name: 'Cards',
  useCases: [for (final (name, make, tag) in cardUses) WidgetbookUseCase(name: name, builder: (c) => _Keep(make, tag))],
);
