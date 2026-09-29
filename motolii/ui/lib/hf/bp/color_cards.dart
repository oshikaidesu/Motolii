import 'package:flutter/widgets.dart';

import '../neutral.dart';
import 'common.dart' show mono;

/// Colours as cards (a pinboard): rounded blocks of the colour itself in staggered columns, the hex written on the
/// colour in an ink that reads on it. Tap uses it, a secondary tap opens its menu, as the chips did.
class SwatchCards extends StatelessWidget {
  const SwatchCards(this.items, this.width, {this.onTap, this.onMenu});
  final List<(String, int, String)> items; // name, argb, class
  final double width;
  final ValueChanged<(String, int, String)>? onTap;
  final void Function((String, int, String) swatch, Offset at)? onMenu;

  static const _heights = [78.0, 104.0, 88.0, 66.0, 96.0, 72.0];

  @override
  Widget build(BuildContext context) {
    const gap = 6.0;
    final cols = width >= 250 ? 3 : 2;
    final columns = [for (var i = 0; i < cols; i++) <Widget>[]];
    final heights = List.filled(cols, 0.0);
    for (final (i, v) in items.take(300).indexed) {
      final h = _heights[i % _heights.length];
      var at = 0;
      for (var c = 1; c < cols; c++) {
        if (heights[c] < heights[at]) at = c;
      }
      heights[at] += h + gap;
      final color = Color(v.$2);
      final ink = color.computeLuminance() > .45 ? N.g10.withValues(alpha: .72) : N.g100.withValues(alpha: .86);
      columns[at].add(Padding(
        padding: const EdgeInsets.only(bottom: gap),
        child: GestureDetector(
          key: ValueKey('hf-color:${v.$1}'),
          behavior: HitTestBehavior.opaque,
          onTap: onTap == null ? null : () => onTap!(v),
          onSecondaryTapDown: onMenu == null ? null : (e) => onMenu!(v, e.globalPosition),
          child: Container(
            height: h,
            padding: const EdgeInsets.fromLTRB(8, 0, 8, 7),
            alignment: Alignment.bottomLeft,
            decoration: BoxDecoration(color: color, borderRadius: BorderRadius.circular(10), border: Border.all(color: N.glaze9)),
            child: Text((v.$2 & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase(), softWrap: false, style: mono(9.5, c: ink)),
          ),
        ),
      ));
    }
    return Padding(
      padding: const EdgeInsets.only(top: 2),
      child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        for (var c = 0; c < cols; c++) ...[
          if (c > 0) const SizedBox(width: gap),
          Expanded(child: Column(children: columns[c])),
        ],
      ]),
    );
  }
}
