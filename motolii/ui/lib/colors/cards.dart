import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../theme/neutral.dart';
import '../browser/parts.dart' show sans;
import '../theme/metrics.dart' show Dn;

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
    const gap = 4.0;
    final cols = math.max(2, ((width + gap) / 60).floor());
    final cell = (width - gap * (cols - 1)) / cols;
    final ground = N.g10.computeLuminance();
    return Wrap(spacing: gap, runSpacing: gap, children: [
      for (final v in items.take(300))
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
              padding: const EdgeInsets.fromLTRB(6, 0, 4.5, 5),
              alignment: Alignment.bottomLeft,
              decoration: BoxDecoration(color: color, borderRadius: BorderRadius.circular(6), border: Border.all(color: (l - ground).abs() < .06 ? N.g20 : N.glaze9)),
              child: Text('#${(v.$2 & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}', softWrap: false, style: sans(Dn.labelSize, c: l > .45 ? N.g07.withValues(alpha: .8) : N.g100.withValues(alpha: .92), w: FontWeight.w500)),
            );
          }),
        ),
    ]);
  }
}
