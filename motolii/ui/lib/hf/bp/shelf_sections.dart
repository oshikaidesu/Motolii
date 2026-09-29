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
  const ShelfHeading(this.text, {super.key, this.count, this.first = false});
  final String text;
  final int? count;

  /// The first section under the strip: the strip's own rule is enough.
  final bool first;
  @override
  Widget build(BuildContext context) => SwissHeading(text, count: count, rule: !first);
}

/// The Browser's one heading voice, set like a grid poster: a hairline across the column, the section's name large
/// and tight under it, its count in small mono in the second ink at the far edge.
class SwissHeading extends StatelessWidget {
  const SwissHeading(this.text, {super.key, this.count, this.rule = true});
  final String text;
  final int? count;
  final bool rule;
  @override
  Widget build(BuildContext context) => Container(
        margin: EdgeInsets.only(top: rule ? 16 : 8, bottom: 8),
        padding: const EdgeInsets.only(top: 5),
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: N.g38, width: .5))),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Expanded(child: Text(text, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(17, c: N.g95, w: FontWeight.w700, ls: -.5))),
          if (count != null) Padding(padding: const EdgeInsets.only(top: 1), child: Text(count!.toString().padLeft(2, '0'), style: mono(9, c: N.signal))),
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
