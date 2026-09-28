import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'common.dart';

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
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(0, 5 * kShelfUnit, 0, 2 * kShelfUnit),
        child: Row(crossAxisAlignment: CrossAxisAlignment.baseline, textBaseline: TextBaseline.alphabetic, children: [
          Flexible(child: Text(text, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(14, c: const Color(0xFFF2F2F4), w: FontWeight.w600))),
          if (count != null) ...[
            const SizedBox(width: 2 * kShelfUnit),
            Text('$count', style: mono(11, c: kMuted)),
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
            border: Border.all(color: const Color(0xFFF2F2F4), width: 2),
            borderRadius: BorderRadius.circular(radius),
          ),
        ),
      );
}
