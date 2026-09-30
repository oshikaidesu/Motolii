import 'package:flutter/widgets.dart';

import '../../theme/metrics.dart';
import '../../theme/editor_theme.dart';

/// The leaves laid along a line: the rule that marks one, the bar that
/// scrolls one, and the thumb that runs one.



/// A value on a flat 2 px track between [min] and [max]: [EditorTheme.of(context).muted]
/// up to the thumb, [EditorTheme.of(context).line] after, a 5 px ink thumb with a 1 dp
/// shadow (6 while pressed); with [divisions] the value snaps and a label of
/// it stands above the thumb while it is dragged.
class EditorSlider extends StatefulWidget {
  const EditorSlider({
    super.key,
    required this.value,
    required this.onChanged,
    this.onChangeEnd,
    this.min = 0,
    this.max = 1,
    this.divisions,
  });
  final double value, min, max;
  final int? divisions;
  final ValueChanged<double>? onChanged, onChangeEnd;
  @override
  State<EditorSlider> createState() => _EditorSliderState();
}

class _EditorSliderState extends State<EditorSlider> {
  bool _down = false;
  static const _thumb = Step.s5;

  double _fraction(double dx, double width) {
    final span = width - 2 * _thumb;
    return span <= 0 ? 0 : ((dx - _thumb) / span).clamp(0.0, 1.0);
  }

  void _at(double dx, double width) {
    var t = _fraction(dx, width);
    if (widget.divisions != null && widget.divisions! > 0) {
      t = (t * widget.divisions!).round() / widget.divisions!;
    }
    widget.onChanged?.call(widget.min + t * (widget.max - widget.min));
  }

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onChanged != null;
    final span = widget.max - widget.min;
    final t = span <= 0
        ? 0.0
        : ((widget.value - widget.min) / span).clamp(0.0, 1.0);
    return Semantics(
      slider: true,
      enabled: enabled,
      value: widget.value.toStringAsFixed(0),
      child: LayoutBuilder(
        builder: (context, box) => GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: enabled ? (d) => setState(() => _down = true) : null,
          onTapUp: enabled
              ? (d) {
                  _at(d.localPosition.dx, box.maxWidth);
                  setState(() => _down = false);
                  widget.onChangeEnd?.call(widget.value);
                }
              : null,
          onTapCancel: enabled ? () => setState(() => _down = false) : null,
          onHorizontalDragStart: enabled
              ? (d) {
                  setState(() => _down = true);
                  _at(d.localPosition.dx, box.maxWidth);
                }
              : null,
          onHorizontalDragUpdate: enabled
              ? (d) => _at(d.localPosition.dx, box.maxWidth)
              : null,
          onHorizontalDragEnd: enabled
              ? (_) {
                  setState(() => _down = false);
                  widget.onChangeEnd?.call(widget.value);
                }
              : null,
          onHorizontalDragCancel: enabled
              ? () => setState(() => _down = false)
              : null,
          child: CustomPaint(
            size: Size(
              box.maxWidth,
              box.hasBoundedHeight ? box.maxHeight : Surface.workRow,
            ),
            painter: _SliderPainter(
              colors: EditorTheme.of(context),
              t: t,
              enabled: enabled,
              pressed: _down,
              divisions: widget.divisions,
              label: _down && widget.divisions != null
                  ? widget.value.round().toString()
                  : null,
            ),
          ),
        ),
      ),
    );
  }
}

class _SliderPainter extends CustomPainter {
  final EditorTheme colors;

  const _SliderPainter({
    this.colors = EditorTheme.chromatic,
    required this.t,
    required this.enabled,
    required this.pressed,
    required this.divisions,
    required this.label,
  });
  final double t;
  final bool enabled, pressed;
  final int? divisions;
  final String? label;
  static const _thumb = Step.s5, _track = Step.s2;
  @override
  void paint(Canvas canvas, Size size) {
    final left = _thumb, right = size.width - _thumb;
    final cy = size.height / 2;
    final x = left + (right - left) * t;
    final track = Rect.fromLTRB(left, cy - _track / 2, right, cy + _track / 2);
    const r = Radius.circular(_track / 2);
    final active = Paint()..color = enabled ? colors.muted : colors.inkDisabled;
    final inactive = Paint()..color = colors.line;
    canvas.drawRRect(
      RRect.fromRectAndCorners(
        Rect.fromLTRB(track.left, track.top, x, track.bottom),
        topLeft: r,
        bottomLeft: r,
      ),
      active,
    );
    canvas.drawRRect(
      RRect.fromRectAndCorners(
        Rect.fromLTRB(x, track.top, track.right, track.bottom),
        topRight: r,
        bottomRight: r,
      ),
      inactive,
    );
    if (divisions != null && divisions! > 0) {
      final adjusted = track.width - track.height;
      const tick = _track / 2;
      if (adjusted / divisions! >= 3 * tick) {
        for (var i = 0; i <= divisions!; i++) {
          final tx = left + (right - left) * i / divisions!;
          canvas.drawCircle(
            Offset(tx, cy),
            tick / 2,
            Paint()..color = tx <= x ? colors.tickActive : colors.tickInactive,
          );
        }
      }
    }
    final thumb = Path()
      ..addOval(Rect.fromCircle(center: Offset(x, cy), radius: _thumb));
    canvas.drawShadow(thumb, EditorTheme.black, pressed ? 6 : 1, true);
    canvas.drawCircle(
      Offset(x, cy),
      _thumb,
      Paint()..color = enabled ? colors.ink : colors.inkDisabled,
    );
    if (label != null) {
      final text = TextPainter(
        text: TextSpan(
          text: label,
          style: TextStyle(
            fontFamily: EditorTheme.fontFamily,
            fontSize: Dn.labelSize,
            color: colors.tabInk,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      final w = text.width + Step.s16,
          h = text.height + Step.s8;
      final box = Rect.fromCenter(
        center: Offset(x, cy - _thumb - Step.s8 - h / 2),
        width: w,
        height: h,
      );
      canvas.drawRRect(
        RRect.fromRectAndRadius(box, const Radius.circular(Step.s4)),
        Paint()..color = colors.accent,
      );
      text.paint(
        canvas,
        box.topLeft + const Offset(Step.s8, Step.s4),
      );
    }
  }

  @override
  bool shouldRepaint(_SliderPainter old) =>
      colors != old.colors ||
      old.t != t ||
      old.enabled != enabled ||
      old.pressed != pressed ||
      old.divisions != divisions ||
      old.label != label;
}
