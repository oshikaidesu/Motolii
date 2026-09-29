import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../neutral.dart';
import 'common.dart' show mono;

/// Colours as blocks: the colour itself in large squares, three across the default seat, 2 px apart, its hex set
/// inside at the bottom left in an ink that reads on it (a swatch card, as current palettes are shown); an edge only
/// where a colour would vanish into the ground. Tap uses it, a secondary tap opens its menu.
class SwatchCards extends StatelessWidget {
  const SwatchCards(this.items, this.width, {super.key, this.onTap, this.onMenu});
  final List<(String, int, String)> items; // name, argb, class
  final double width;
  final ValueChanged<(String, int, String)>? onTap;
  final void Function((String, int, String) swatch, Offset at)? onMenu;

  @override
  Widget build(BuildContext context) {
    const gap = 2.0;
    final cols = math.max(2, ((width + gap) / 80).floor());
    final cell = (width - gap * (cols - 1)) / cols;
    final ground = N.g10.computeLuminance();
    return Wrap(spacing: gap, runSpacing: gap, children: [
      for (final (i, v) in items.take(300).indexed)
        GestureDetector(
          key: ValueKey('hf-color:${v.$1}'),
          behavior: HitTestBehavior.opaque,
          onTap: onTap == null ? null : () => onTap!(v),
          onSecondaryTapDown: onMenu == null ? null : (e) => onMenu!(v, e.globalPosition),
          child: Builder(builder: (context) {
            final color = Color(v.$2);
            final l = color.computeLuminance();
            return Container(
              width: cell,
              height: cell * .78,
              padding: const EdgeInsets.fromLTRB(6, 5, 6, 6),
              decoration: BoxDecoration(color: color, border: (l - ground).abs() < .06 ? Border.all(color: N.g26, width: .5) : null),
              // a specimen sheet: its number at the top, its value at the foot, both in small mono in an ink that reads on it
              child: Builder(builder: (_) {
                final ink = l > .45 ? N.g07.withValues(alpha: .78) : N.g100.withValues(alpha: .9);
                return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Text((i + 1).toString().padLeft(2, '0'), style: mono(8, c: ink)),
                  const Spacer(),
                  Text('#${(v.$2 & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}', softWrap: false, style: mono(9, c: ink)),
                ]);
              }),
            );
          }),
        ),
    ]);
  }
}
