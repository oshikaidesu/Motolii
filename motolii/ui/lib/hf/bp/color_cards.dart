import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../neutral.dart';
import 'common.dart' show mono;

/// Colours as a pinboard: cards of the colour itself in staggered columns packed tight (4 px), the hex set small at the
/// bottom left in an ink that reads on it; an edge only where a colour would vanish into the ground. Tap uses it, a
/// secondary tap opens its menu.
class SwatchCards extends StatelessWidget {
  const SwatchCards(this.items, this.width, {super.key, this.onTap, this.onMenu});
  final List<(String, int, String)> items; // name, argb, class
  final double width;
  final ValueChanged<(String, int, String)>? onTap;
  final void Function((String, int, String) swatch, Offset at)? onMenu;

  static const _heights = [46.0, 62.0, 52.0, 40.0, 58.0, 44.0];

  @override
  Widget build(BuildContext context) {
    const gap = 4.0;
    final cols = math.max(2, ((width + gap) / 66).floor()); // four across the default seat
    final columns = [for (var i = 0; i < cols; i++) <Widget>[]];
    final heights = List.filled(cols, 0.0);
    final ground = N.g10.computeLuminance();
    for (final (i, v) in items.take(300).indexed) {
      final h = _heights[i % _heights.length];
      var at = 0;
      for (var c = 1; c < cols; c++) {
        if (heights[c] < heights[at]) at = c;
      }
      heights[at] += h + gap;
      final color = Color(v.$2);
      final l = color.computeLuminance();
      columns[at].add(Padding(
        padding: const EdgeInsets.only(bottom: gap),
        child: GestureDetector(
          key: ValueKey('hf-color:${v.$1}'),
          behavior: HitTestBehavior.opaque,
          onTap: onTap == null ? null : () => onTap!(v),
          onSecondaryTapDown: onMenu == null ? null : (e) => onMenu!(v, e.globalPosition),
          child: Container(
            height: h,
            padding: const EdgeInsets.all(5),
            alignment: Alignment.bottomLeft,
            decoration: BoxDecoration(color: color, borderRadius: BorderRadius.circular(4), border: (l - ground).abs() < .08 ? Border.all(color: N.g20) : null),
            child: Text((v.$2 & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase(), softWrap: false, style: mono(8, c: l > .5 ? N.g07.withValues(alpha: .7) : N.g100.withValues(alpha: .85))),
          ),
        ),
      ));
    }
    return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      for (var c = 0; c < cols; c++) ...[
        if (c > 0) const SizedBox(width: gap),
        Expanded(child: Column(children: columns[c])),
      ],
    ]);
  }
}
