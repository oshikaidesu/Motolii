// The number field's picture, kept out of the field's own file: that file is pulled in by the inspector, and the inspector
// is pulled in by the forms, so the field cannot import the forms back. [bindNumber] is set when the forms load.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../tokens.dart';

class NumberSpec {
  const NumberSpec({
    required this.value,
    required this.width,
    required this.height,
    required this.fillColor,
    required this.borderColor,
    required this.rangeColor,
    this.editor,
    this.lead,
    this.glyph,
    this.caption,
    this.fill,
    this.zeroAt = 0,
    this.bipolar = false,
    this.block = false,
    this.pop = false,
    this.editing = false,
    this.glyphTall = false,
    this.on = true,
    this.hot = false,
    this.live = false,
    this.axis,
    this.keyDot,
    this.keyEmpty = false,
    this.pad = 8,
    this.glyphBox = 22,
  });

  /// The number as text, when it is not being typed.
  final Widget value;

  /// The text being typed. The field owns it; a form only places it.
  final Widget? editor;
  final Widget? lead, glyph;
  final String? caption;
  final double width, height, pad, glyphBox, zeroAt;

  /// 0..1 along a bounded field. Null: the field has no range, so no mark travels.
  final double? fill;
  final bool bipolar, block, pop, editing, glyphTall, on, hot, live;
  final Color fillColor, borderColor, rangeColor;
  final Color? axis, keyDot;

  /// The key mark drawn hollow: the value takes keys and has none here (Alight Motion's diamond beside a value, empty).
  final bool keyEmpty;
}

/// A value's key mark: a diamond, as Alight Motion's beside each value and the timeline's keys (filled: a key here;
/// faint: keys elsewhere; [empty]: none, yet it takes them). Pressed, the field keys the value there (num_field_state).
Widget keyDiamond(Color colour, {bool empty = false}) => Transform.rotate(
  angle: math.pi / 4,
  child: Container(width: Shape.z(5), height: Shape.z(5), decoration: BoxDecoration(color: empty ? null : colour, border: empty ? Border.all(color: colour) : null)),
);

typedef NumberPaint = Widget Function(BuildContext context, NumberSpec spec);

NumberPaint _paint = usualNumber;

/// A form's [Forms.number], installed as the forms library loads.
void bindNumber(NumberPaint paint) => _paint = paint;

Widget paintNumber(BuildContext context, NumberSpec spec) => _paint(context, spec);

/// The field as the window has drawn it: a filled well, a range along the bottom, the axis as a stripe.
Widget usualNumber(BuildContext context, NumberSpec n) {
  final shown = n.editing && n.editor != null ? n.editor! : n.value;
  return Container(
    height: n.height,
    decoration: BoxDecoration(color: n.fillColor, borderRadius: BorderRadius.circular(n.pop ? 6 : 4), border: Border.all(color: n.borderColor)),
    child: ClipRRect(
      borderRadius: BorderRadius.circular(n.pop ? 5 : 3),
      child: Stack(
        children: [
          if (n.fill != null && n.bipolar)
            Positioned(
              left: (n.width - 2) * (n.fill! < n.zeroAt ? n.fill! : n.zeroAt),
              bottom: 0,
              height: Shape.z(2),
              width: math.max(1, (n.width - 2) * (n.fill! - n.zeroAt).abs()),
              child: ColoredBox(color: n.rangeColor),
            )
          else if (n.fill != null)
            Positioned(left: 0, bottom: 0, height: Shape.z(2), width: (n.width - 2) * n.fill!, child: ColoredBox(color: n.rangeColor)),
          if (n.keyDot != null)
            Positioned(
              top: n.block ? null : 3,
              bottom: n.block ? 7 : null,
              right: n.block ? n.pad : 3,
              child: keyDiamond(n.keyDot!, empty: n.keyEmpty),
            ),
          if (n.axis != null && !n.pop) Positioned(left: 0, top: 5, bottom: 5, width: Shape.z(2), child: ColoredBox(color: n.axis!)),
          if (n.glyph != null)
            Positioned(
              right: n.glyphTall ? 4 : n.pad - 3,
              top: 4,
              width: n.glyphBox,
              height: n.glyphTall ? null : n.glyphBox,
              bottom: n.glyphTall ? 4 : null,
              child: Center(child: n.glyph),
            ),
          if (n.block)
            Positioned.fill(
              child: Padding(
                padding: EdgeInsets.fromLTRB(n.pad, Shape.s(4), n.glyph != null ? n.glyphBox + n.pad : n.pad, Shape.s(5)),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    if (n.caption != null) FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerLeft, child: Text(n.caption!, maxLines: 1, style: T.micro(n.on ? Grey.g63 : Grey.g44).copyWith(height: 1.1))),
                    SizedBox(height: Shape.z(18), child: Align(alignment: Alignment.centerLeft, child: shown)),
                  ],
                ),
              ),
            )
          else
            Positioned.fill(
              child: Padding(
                padding: EdgeInsets.only(left: n.axis != null && !n.pop ? Shape.s(8) : Shape.s(7), right: Shape.s(6)),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    if (n.lead != null) Flexible(child: n.lead!) else const SizedBox.shrink(),
                    if (!n.editing) Expanded(flex: 4, child: Align(alignment: n.pop ? Alignment.center : Alignment.centerRight, child: n.value)) else const Spacer(),
                    if (n.editing && n.editor != null) SizedBox(width: (n.width - 36).clamp(28, 72), child: n.editor),
                  ],
                ),
              ),
            ),
        ],
      ),
    ),
  );
}
