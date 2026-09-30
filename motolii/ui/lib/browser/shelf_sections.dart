import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'parts.dart';
import '../theme/neutral.dart';
import '../theme/surface.dart' show Dn;

/// The housing Create and Media share (Swiss: one grid, a few sizes, hierarchy from type and space). What sits in a
/// tile is each shelf's own face; only the section rhythm, the column rule and the picked ring live here.

/// The grid's unit: paddings, gaps and heading space are multiples of it.
const kShelfUnit = 4.0;
const kShelfPad = 3 * kShelfUnit, kShelfGap = 2 * kShelfUnit;

/// A section's heading: the class read at a glance (sentence weight, not tiny caps) and how many it holds.
class ShelfHeading extends StatelessWidget {
  const ShelfHeading(this.text, {super.key, this.count, this.first = false});
  final String text;
  final int? count;

  /// The first section under the strip: the strip's own rule is enough.
  final bool first;
  @override
  Widget build(BuildContext context) => SwissHeading(text, count: count, rule: !first);
}

/// The Browser's one heading voice: the section's name in plain semibold, its count in a small quiet pill.
class SwissHeading extends StatelessWidget {
  const SwissHeading(this.text, {super.key, this.count, this.rule = true});
  final String text;
  final int? count;
  final bool rule; // kept for callers; a heading no longer draws a rule
  @override
  Widget build(BuildContext context) => Padding(
        padding: EdgeInsets.only(top: rule ? 14 : 10, bottom: 6),
        child: Row(children: [
          Flexible(child: Text(text, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: N.g91, w: FontWeight.w600))),
          if (count != null)
            Container(
              margin: const EdgeInsets.only(left: 4.5),
              padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
              decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(3)),
              child: Text('$count', style: sans(Dn.microSize, c: N.g63, w: FontWeight.w500)),
            ),
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
            border: Border.all(color: N.g95, width: 1.5),
            borderRadius: BorderRadius.circular(radius),
          ),
        ),
      );
}
