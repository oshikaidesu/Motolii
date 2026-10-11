part of 'inspector_parts.dart';

/// What a card holds, as one bold shape: the card reads by its sticker before its word.
enum CardMark {
  transform,
  anchor,
  layer,
  text,
  fill,
  blend,
  blur,
  glow,
  colour,
  distort,
  shader,
  repeater,
  mirror,
  path,
  solid,
  textAnim,
  mask,
  clock,
  layout,
  particles,
  camera,
  track,
  motionBlur,
  overlay,
  link,
}

/// A solid tone tile with a dark mark knocked onto it, sticker style.
class ToneBadge extends StatelessWidget {
  const ToneBadge(this.mark, this.tone, {super.key, this.dim = false, this.size = 16});
  final CardMark mark;
  final Color tone;
  final bool dim;
  final double size;
  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    decoration: BoxDecoration(color: dim ? Grey.g26 : tone, borderRadius: BorderRadius.circular(size * .32)),
    child: CustomPaint(painter: _MarkPaint(mark, dim ? Grey.g56 : Pop.onAccent)),
  );
}

class _MarkPaint extends CustomPainter {
  const _MarkPaint(this.mark, this.ink);
  final CardMark mark;
  final Color ink;

  @override
  void paint(Canvas c, Size s) {
    final u = s.width / 20, o = s.center(Offset.zero);
    final fill = Paint()..color = ink;
    Paint stroke(double w) => Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = w * u
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    Offset p(double x, double y) => o + Offset(x, y) * u;
    switch (mark) {
      case CardMark.transform:
        final st = stroke(1.8);
        c.drawLine(p(-5.5, 0), p(5.5, 0), st);
        c.drawLine(p(0, -5.5), p(0, 5.5), st);
        for (final (dx, dy) in const [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)]) {
          final tip = p(dx * 6, dy * 6);
          c.drawPath(
            Path()
              ..moveTo(tip.dx, tip.dy)
              ..lineTo(tip.dx - (dx * 3 + dy * 2.4) * u, tip.dy - (dy * 3 + dx * 2.4) * u)
              ..lineTo(tip.dx - (dx * 3 - dy * 2.4) * u, tip.dy - (dy * 3 - dx * 2.4) * u)
              ..close(),
            fill,
          );
        }
      case CardMark.anchor:
        c.drawCircle(o, 5.5 * u, stroke(1.8));
        c.drawCircle(o, 2 * u, fill);
      case CardMark.layer:
        Path dia(double y) => Path()
          ..moveTo(p(0, y - 3.2).dx, p(0, y - 3.2).dy)
          ..lineTo(p(6, y).dx, p(6, y).dy)
          ..lineTo(p(0, y + 3.2).dx, p(0, y + 3.2).dy)
          ..lineTo(p(-6, y).dx, p(-6, y).dy)
          ..close();
        c.drawPath(dia(2.2), stroke(1.5));
        c.drawPath(dia(-1.8), fill);
      case CardMark.text:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(-5.5, -5.5), p(5.5, -2.5)), Radius.circular(1 * u)), fill);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(-1.5, -5.5), p(1.5, 5.5)), Radius.circular(1 * u)), fill);
      case CardMark.fill:
        c.drawPath(
          Path()
            ..moveTo(p(0, -6.5).dx, p(0, -6.5).dy)
            ..cubicTo(p(3, -2.5).dx, p(3, -2.5).dy, p(5, 0).dx, p(5, 0).dy, p(5, 2).dx, p(5, 2).dy)
            ..arcToPoint(p(-5, 2), radius: Radius.circular(5 * u))
            ..cubicTo(p(-5, 0).dx, p(-5, 0).dy, p(-3, -2.5).dx, p(-3, -2.5).dy, p(0, -6.5).dx, p(0, -6.5).dy)
            ..close(),
          fill,
        );
      case CardMark.blend:
        c.drawCircle(p(-2.2, 0), 4.2 * u, fill);
        c.drawCircle(p(2.6, 0), 4.2 * u, stroke(1.5));
      case CardMark.blur:
        c.drawCircle(o, 2.6 * u, fill);
        c.drawCircle(o, 4.6 * u, Paint()..color = ink.withValues(alpha: .45));
        c.drawCircle(o, 6.6 * u, Paint()..color = ink.withValues(alpha: .18));
      case CardMark.glow:
        c.drawCircle(o, 2.8 * u, fill);
        final st = stroke(1.6);
        for (var i = 0; i < 8; i++) {
          final a = i * math.pi / 4;
          c.drawLine(o + Offset(math.cos(a), math.sin(a)) * 4.8 * u, o + Offset(math.cos(a), math.sin(a)) * 6.6 * u, st);
        }
      case CardMark.colour:
        final r = Rect.fromCircle(center: o, radius: 6 * u);
        for (var i = 0; i < 3; i++) {
          c.drawArc(r, -math.pi / 2 + i * 2 * math.pi / 3, 2 * math.pi / 3 - .35, true, Paint()..color = ink.withValues(alpha: 1 - i * .3));
        }
      case CardMark.shader:
        final hex = Path();
        for (var i = 0; i < 6; i++) {
          final a = math.pi / 6 + i * math.pi / 3, q = o + Offset(math.cos(a), math.sin(a)) * 6.2 * u;
          i == 0 ? hex.moveTo(q.dx, q.dy) : hex.lineTo(q.dx, q.dy);
        }
        c.drawPath(hex..close(), stroke(1.6));
        c.drawCircle(o, 2 * u, fill);
      case CardMark.distort:
        final path = Path()..moveTo(p(-6.5, 0).dx, p(-6.5, 0).dy);
        for (var x = -6.5; x <= 6.5; x += .5) {
          path.lineTo(p(x, math.sin(x * .95) * 3.6).dx, p(x, math.sin(x * .95) * 3.6).dy);
        }
        c.drawPath(path, stroke(2));
      case CardMark.repeater:
        for (var i = 0; i < 3; i++) {
          c.drawRRect(
            RRect.fromRectAndRadius(Rect.fromCenter(center: p(-4.5 + i * 4.5, 0), width: 3.4 * u, height: 3.4 * u), Radius.circular(.8 * u)),
            Paint()..color = ink.withValues(alpha: 1 - i * .3),
          );
        }
      case CardMark.mirror:
        c.drawLine(p(0, -6.5), p(0, 6.5), stroke(1.2));
        c.drawPath(
          Path()
            ..moveTo(p(-1.6, -4.5).dx, p(-1.6, -4.5).dy)
            ..lineTo(p(-6, 4).dx, p(-6, 4).dy)
            ..lineTo(p(-1.6, 4).dx, p(-1.6, 4).dy)
            ..close(),
          fill,
        );
        c.drawPath(
          Path()
            ..moveTo(p(1.6, -4.5).dx, p(1.6, -4.5).dy)
            ..lineTo(p(6, 4).dx, p(6, 4).dy)
            ..lineTo(p(1.6, 4).dx, p(1.6, 4).dy)
            ..close(),
          stroke(1.3),
        );
      case CardMark.path:
        c.drawPath(
          Path()
            ..moveTo(p(-6, 4).dx, p(-6, 4).dy)
            ..cubicTo(p(-3, -7).dx, p(-3, -7).dy, p(3, 7).dx, p(3, 7).dy, p(6, -4).dx, p(6, -4).dy),
          stroke(1.8),
        );
        c.drawCircle(p(-6, 4), 1.8 * u, fill);
        c.drawCircle(p(6, -4), 1.8 * u, fill);
      case CardMark.solid:
        c.drawRect(Rect.fromPoints(p(-5.5, -2), p(2, 5.5)), fill);
        c.drawPath(
          Path()
            ..moveTo(p(-5.5, -2).dx, p(-5.5, -2).dy)
            ..lineTo(p(-2, -5.5).dx, p(-2, -5.5).dy)
            ..lineTo(p(5.5, -5.5).dx, p(5.5, -5.5).dy)
            ..lineTo(p(5.5, 2).dx, p(5.5, 2).dy)
            ..lineTo(p(2, 5.5).dx, p(2, 5.5).dy),
          stroke(1.3),
        );
      case CardMark.textAnim:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(-6, -5), p(-1, -2.4)), Radius.circular(.8 * u)), fill);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(-4.8, -5), p(-2.2, 5)), Radius.circular(.8 * u)), fill);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(1, -1), p(6, 1.6)), Radius.circular(.8 * u)), Paint()..color = ink.withValues(alpha: .5));
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(2.2, -1), p(4.8, 6)), Radius.circular(.8 * u)), Paint()..color = ink.withValues(alpha: .5));
      case CardMark.mask:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: o, width: 13 * u, height: 13 * u), Radius.circular(2.4 * u)), stroke(1.4));
        c.drawCircle(o, 3.6 * u, fill);
      case CardMark.clock:
        c.drawCircle(o, 6 * u, stroke(1.6));
        c.drawLine(o, p(0, -3.8), stroke(1.6));
        c.drawLine(o, p(2.8, 1.2), stroke(1.6));
      case CardMark.layout:
        for (final r in [Rect.fromPoints(p(-6, -6), p(-.8, -.8)), Rect.fromPoints(p(.8, -6), p(6, -.8)), Rect.fromPoints(p(-6, .8), p(6, 6))]) {
          c.drawRRect(RRect.fromRectAndRadius(r, Radius.circular(1 * u)), fill);
        }
      case CardMark.particles:
        for (final (x, y, r) in const [(-4.0, -3.0, 1.8), (2.5, -4.5, 1.3), (4.5, 1.5, 2.0), (-1.5, 2.5, 1.4), (-5.0, 4.5, 1.0), (1.0, 5.5, 1.0)]) {
          c.drawCircle(p(x, y), r * u, fill);
        }
      case CardMark.camera:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(-6.5, -3.5), p(2.5, 4)), Radius.circular(1.4 * u)), fill);
        c.drawPath(
          Path()
            ..moveTo(p(3.5, 0).dx, p(3.5, 0).dy)
            ..lineTo(p(6.5, -3).dx, p(6.5, -3).dy)
            ..lineTo(p(6.5, 3.5).dx, p(6.5, 3.5).dy)
            ..close(),
          fill,
        );
      case CardMark.track:
        c.drawRect(Rect.fromCenter(center: o, width: 9 * u, height: 9 * u), stroke(1.4));
        for (final (dx, dy) in const [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)]) {
          c.drawLine(p(dx * 4.5, dy * 4.5), p(dx * 7, dy * 7), stroke(1.4));
        }
        c.drawCircle(o, 1.6 * u, fill);
      case CardMark.motionBlur:
        c.drawCircle(p(3, 0), 3.6 * u, fill);
        for (var i = 0; i < 3; i++) {
          c.drawLine(p(-7 + i * 1.2, -2.6 + i * 2.6), p(-1.5, -2.6 + i * 2.6), stroke(1.3)..color = ink.withValues(alpha: .4 + i * .2));
        }
      case CardMark.overlay:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(-6, -6), p(2, 2)), Radius.circular(1.4 * u)), stroke(1.4));
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(p(-2, -2), p(6, 6)), Radius.circular(1.4 * u)), fill);
      case CardMark.link:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: p(-2.6, 0), width: 8 * u, height: 5 * u), Radius.circular(2.5 * u)), stroke(1.6));
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: p(2.6, 0), width: 8 * u, height: 5 * u), Radius.circular(2.5 * u)), stroke(1.6));
    }
  }

  @override
  bool shouldRepaint(_MarkPaint o) => o.mark != mark || o.ink != ink;
}

/// One summary value as a tag, so a folded card reads as a row of chips rather than a sentence. A colour value brings its swatch.
class BriefChip extends StatelessWidget {
  const BriefChip(this.text, {super.key, this.dim = false});
  final String text;
  final bool dim;

  static TextStyle get _style => T.label().copyWith(fontWeight: FontWeight.w600, height: 1);
  static Color? swatch(String t) => RegExp(r'^#[0-9A-Fa-f]{6}$').hasMatch(t) ? Color(int.parse(t.substring(1), radix: 16) | 0xFF000000) : null;

  /// The chip's laid-out width, so a row can drop the chips that would not fit whole.
  static double width(String t) =>
      (TextPainter(
        text: TextSpan(text: t, style: _style),
        maxLines: 1,
        textDirection: TextDirection.ltr,
      )..layout()).width +
      12 +
      (swatch(t) != null ? 12 : 0);

  @override
  Widget build(BuildContext context) {
    final sw = swatch(text);
    return Container(
      height: 16,
      padding: const EdgeInsets.symmetric(horizontal: 6),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: dim ? Grey.g15 : Grey.g20, borderRadius: BorderRadius.circular(8)),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (sw != null) ...[
            Container(
              width: 8,
              height: 8,
              decoration: BoxDecoration(color: sw, shape: BoxShape.circle),
            ),
            const SizedBox(width: 4),
          ],
          Flexible(
            child: Text(
              text,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: _style.copyWith(color: dim ? Grey.g56 : Grey.g95),
            ),
          ),
        ],
      ),
    );
  }
}
