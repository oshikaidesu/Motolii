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
  Widget build(BuildContext context) => CardsTable(t, tag: widget.tag);
}

/// The ring with its wave up (40 px), so what the wave card does shows.
CardTable _waving() => starRing()..val('waveH').v = 40;

/// The same, the wave card laid before the copies.
CardTable _waveFirst() {
  final t = _waving(), w = t.cards.firstWhere((c) => c.name == '波');
  return t..cards.remove(w)..cards.insert(1, w);
}

/// The use cases, in the order the night made them.
final cardUses = <(String, CardTable Function(), String)>[
  ('重ねる', starRing, '手札のカードを値に重ねる（乗せただけで先に効く・同じカードでもっと強く・違うカードで入れ替え・引きはがして外す）'),
  ('裏返す', () => starRing()..cards[1].flipped = true, '裏返すと全部の数値'),
  ('順番', _waveFirst, 'カードの順番が効く順（波が先＝輪ごと、複製が先＝1つずつ）'),
  ('伏せる', () => _waving()..cards[2].down = true, '伏せたカードは効かない'),
  ('投げる', () => starRing(wave: false), '棚のカードを舞台に投げると付く（乗せただけで先に見える）、カードの角を掴んで棚に戻すと外れる'),
  ('束ねる', starRing, 'タイルを長押しで持ち上げて別のタイルに落とすと束になる、束をこすると全部動く（ダブルタップでほどく）'),
];

WidgetbookComponent cardsSet() => WidgetbookComponent(
  name: 'カード',
  useCases: [for (final (name, make, tag) in cardUses) WidgetbookUseCase(name: name, builder: (c) => _Keep(make, tag))],
);
