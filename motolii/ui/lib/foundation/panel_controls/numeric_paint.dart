part of 'numeric.dart';

/// What the well draws beside its digits: the ladder the scrub is on, and
/// the track that tells a bounded amount.

/// The ladder while a scrub is on it: the four rungs, the one the pointer
/// is on in ink, the rest faint (Lumit's value field shows the same four).
class _RungPill extends StatelessWidget {
  const _RungPill({required this.rung});
  final double rung;
  static const _rungs = [10.0, 1.0, .1, .01];
  static String _name(double r) => r == 10
      ? '×10'
      : r == 1
      ? '×1'
      : r == .1
      ? '×0.1'
      : '×0.01';
  @override
  Widget build(BuildContext context) => Container(
    constraints: const BoxConstraints(minHeight: EditorMetrics.control),
    padding: const EdgeInsets.symmetric(
      horizontal: EditorMetrics.s8,
      vertical: EditorMetrics.s4,
    ),
    decoration: BoxDecoration(
      color: EditorTheme.of(context).tooltip,
      borderRadius: BorderRadius.all(Radius.circular(EditorMetrics.s4)),
    ),
    child: Text.rich(
      TextSpan(
        children: [
          for (final r in _rungs) ...[
            if (r != _rungs.first) const TextSpan(text: '  '),
            TextSpan(
              text: _name(r),
              style: TextStyle(
                color: r == rung
                    ? EditorTheme.black
                    : EditorTheme.of(context).disabledInk,
                fontWeight: r == rung ? FontWeight.w600 : FontWeight.w400,
              ),
            ),
          ],
        ],
      ),
      style: const TextStyle(
        fontSize: EditorMetrics.s12,
        fontFeatures: [FontFeature.tabularFigures()],
      ),
    ),
  );
}

/// How a track tells its amount, by what the number is for.
enum TrackStyle {
  /// So much of the reach, from where it rests.
  fill,

  /// What lies above this passes: the far side is lit.
  level,

  /// Whole steps, one mark each.
  steps,

  /// A length along a ruler: fill under ticks.
  ruler,
}

/// The amount behind a bounded number, in the family's hue, with a tick
/// where the rest point is.
class _TrackPainter extends CustomPainter {
  final EditorTheme colors;

  const _TrackPainter({
    this.colors = EditorTheme.chromatic,
    required this.value,
    required this.min,
    required this.max,
    required this.tint,
    required this.style,
    this.rest,
  });
  final double value, min, max;
  final double? rest;
  final Color tint;
  final TrackStyle style;
  double _x(double v, double w) =>
      ((v - min) / (max - min)).clamp(0.0, 1.0) * w;
  @override
  void paint(Canvas canvas, Size size) {
    final wash = Paint()..color = tint.withValues(alpha: .28);
    final b = _x(value, size.width);
    switch (style) {
      case TrackStyle.fill:
      case TrackStyle.ruler:
        final zero = rest != null && rest! > min && rest! < max ? rest! : min;
        final a = _x(zero, size.width);
        canvas.drawRect(
          Rect.fromLTRB(math.min(a, b), 0, math.max(a, b), size.height),
          wash,
        );
        if (style == TrackStyle.ruler) {
          final tick = Paint()..color = tint.withValues(alpha: .5);
          for (var i = 1; i < 8; i++) {
            final x = size.width * i / 8;
            canvas.drawLine(
              Offset(x, size.height - (i.isEven ? 5 : 3)),
              Offset(x, size.height),
              tick,
            );
          }
        }
      case TrackStyle.level:
        canvas.drawRect(Rect.fromLTRB(b, 0, size.width, size.height), wash);
        canvas.drawLine(
          Offset(b, 0),
          Offset(b, size.height),
          Paint()
            ..color = tint
            ..strokeWidth = 1.5,
        );
      case TrackStyle.steps:
        final span = (max - min).round().clamp(1, 12);
        final cell = size.width / span;
        final lit = (value - min).round().clamp(0, span);
        for (var i = 0; i < span; i++) {
          canvas.drawRect(
            Rect.fromLTRB(
              i * cell + 1,
              size.height - 4,
              (i + 1) * cell - 1,
              size.height - 1,
            ),
            i < lit ? (Paint()..color = tint) : wash,
          );
        }
    }
    if (rest != null) {
      final x = _x(rest!, size.width);
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.height),
        Paint()
          ..color = colors.border
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(_TrackPainter old) =>
      colors != old.colors ||
      old.value != value ||
      old.min != min ||
      old.max != max ||
      old.rest != rest ||
      old.tint != tint ||
      old.style != style;
}
