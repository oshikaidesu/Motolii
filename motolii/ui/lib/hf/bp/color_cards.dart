import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../neutral.dart';
import 'common.dart' show mono;

/// Colours as a spec sheet: small squares of the colour itself in a tight grid (2 px), the hex under each in small
/// mono; an edge only where a colour would vanish into the ground. Tap uses it, a secondary tap opens its menu.
class SwatchCards extends StatelessWidget {
  const SwatchCards(this.items, this.width, {super.key, this.onTap, this.onMenu});
  final List<(String, int, String)> items; // name, argb, class
  final double width;
  final ValueChanged<(String, int, String)>? onTap;
  final void Function((String, int, String) swatch, Offset at)? onMenu;

  @override
  Widget build(BuildContext context) {
    const gap = 2.0, label = 12.0;
    final cols = math.max(3, ((width + gap) / 42).floor()); // six across the default seat
    final cell = (width - gap * (cols - 1)) / cols;
    final ground = N.g10.computeLuminance();
    return Wrap(spacing: gap, runSpacing: 4, children: [
      for (final v in items.take(300))
        GestureDetector(
          key: ValueKey('hf-color:${v.$1}'),
          behavior: HitTestBehavior.opaque,
          onTap: onTap == null ? null : () => onTap!(v),
          onSecondaryTapDown: onMenu == null ? null : (e) => onMenu!(v, e.globalPosition),
          child: SizedBox(
            width: cell,
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Container(
                height: cell,
                decoration: BoxDecoration(color: Color(v.$2), borderRadius: BorderRadius.circular(1), border: (Color(v.$2).computeLuminance() - ground).abs() < .08 ? Border.all(color: N.g20) : null),
              ),
              SizedBox(height: label, child: Align(alignment: Alignment.bottomLeft, child: Text((v.$2 & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase(), softWrap: false, overflow: TextOverflow.clip, style: mono(7, c: N.g56)))),
            ]),
          ),
        ),
    ]);
  }
}
