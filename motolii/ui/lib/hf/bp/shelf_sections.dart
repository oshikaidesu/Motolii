import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'common.dart';
import '../neutral.dart';

/// The housing Create and Media share (Swiss: one grid, a few sizes, hierarchy from type and space). What sits in a
/// tile is each shelf's own face; only the section rhythm, the column rule and the picked ring live here.

/// The grid's unit: paddings, gaps and heading space are multiples of it.
const kShelfUnit = 4.0;
const kShelfPad = 3 * kShelfUnit, kShelfGap = 2 * kShelfUnit;

/// A section's heading: the class read at a glance (sentence weight, not tiny caps) and how many it holds.
class ShelfHeading extends StatelessWidget {
  const ShelfHeading(this.text, {super.key, this.count});
  final String text;
  final int? count;
  // the same height the tiny caps label took (14 above, 8 below); the weight and the light do the reading
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(0, 10, 0, 4),
        child: Row(crossAxisAlignment: CrossAxisAlignment.baseline, textBaseline: TextBaseline.alphabetic, children: [
          Flexible(child: Text(text, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(11, c: N.g95, w: FontWeight.w600, ls: .2))),
          if (count != null) ...[
            const SizedBox(width: 6),
            Text('$count', style: mono(10, c: kMuted)),
          ],
        ]),
      );
}

/// As many columns as fit at [min] wide (at least one), and the width each then takes.
({int columns, double width}) shelfColumns(double width, double min, {double pad = kShelfPad, double gap = kShelfGap}) {
  final columns = math.max(1, ((width - pad * 2 + gap) / (min + gap)).floor());
  return (columns: columns, width: (width - pad * 2 - gap * (columns - 1)) / columns);
}

/// The picked tile: a light ring inside its corner, thick enough to find in a small screenshot.
class PickedRing extends StatelessWidget {
  const PickedRing({super.key, this.radius = 3});
  final double radius;
  @override
  Widget build(BuildContext context) => IgnorePointer(
        child: DecoratedBox(
          decoration: BoxDecoration(
            border: Border.all(color: N.g95, width: 2),
            borderRadius: BorderRadius.circular(radius),
          ),
        ),
      );
}
