// Glyphs drawn to the reference forms (handoff 1.6 icon boxes). 24-unit design box.
// Weights are deliberately unequal: outline glyphs 1.4-1.7, filled masses, small dots.
import 'dart:math' as math;
import 'package:flutter/widgets.dart';

enum HG {
  text, shape, image, camera, repeater, grid, circle, spiral,
  scatter, alongPath, stagger, face, follow, attach,
  blur, glow, color, composite, distort, stylize,
  arrow, move, rect, ellipse, pen, type, crop,
  pie, kebab, grid4, list, search, fit, corners, play, pin, folder, lock, plus, star, chevronDown, cross, triangle, headphones, diamond, power, preset, eye, eyeOff, solo,
}

class HgPainter extends CustomPainter {
  HgPainter(this.g, this.c, this.bg, [this.a = 1]);
  final HG g;
  final Color c, bg;
  final double a;
  @override
  void paint(Canvas cv, Size s) {
    cv.scale(s.width / 24);
    final col = c.withValues(alpha: a * c.a);
    Paint fill([Color? k]) => Paint()..color = k ?? col;
    Paint line(double w, [Color? k]) => Paint()..color = k ?? col..style = PaintingStyle.stroke..strokeWidth = w..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    RRect rr(double x, double y, double w, double h, double r) => RRect.fromRectAndRadius(Rect.fromLTWH(x, y, w, h), Radius.circular(r));
    const o = Offset.new;
    switch (g) {
      case HG.text:
      case HG.type:
        final w = g == HG.text ? 1.4 : 1.7;
        cv.drawLine(o(5, 5.5), o(19, 5.5), line(w)); cv.drawLine(o(12, 5.5), o(12, 19.5), line(w));
      case HG.shape:
        cv.drawRRect(rr(3, 3, 18, 18, 2.6), fill());
      case HG.image:
        cv.drawRRect(rr(2.5, 4, 19, 16, 1.6), fill());
        final m = fill(Color.lerp(col, bg, .7));
        cv.drawPath(Path()..moveTo(4.5, 17.5)..lineTo(9, 11)..lineTo(12, 14.8)..lineTo(14.6, 11.6)..lineTo(19.5, 17.5)..close(), m);
        cv.drawCircle(o(16.5, 8.2), 1.2, m);
      case HG.camera:
        cv.drawRRect(rr(8, 4.6, 8, 4, 1), fill());
        cv.drawRRect(rr(2, 7, 20, 13.5, 2.2), fill());
        cv.drawCircle(o(12, 13.7), 4.2, line(1.7, bg));
        cv.drawCircle(o(18.6, 9.8), .8, fill(bg));
      case HG.repeater:
        for (final p in [o(5, 5), o(19, 5), o(5, 19), o(19, 19)]) { cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: p, width: 4.8, height: 4.8), const Radius.circular(1)), fill()); }
        cv.drawCircle(o(12, 5), 1.5, fill()); cv.drawCircle(o(12, 19), 1.5, fill());
        cv.drawCircle(o(5, 12), 1.8, line(1.2)); cv.drawCircle(o(19, 12), 1.8, line(1.2));
        cv.drawCircle(o(12, 12), 1.3, fill(col.withValues(alpha: .35 * col.a)));
      case HG.grid:
        for (final x in [5.0, 12.0, 19.0]) { for (final y in [5.0, 12.0, 19.0]) { cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: o(x, y), width: 5, height: 5), const Radius.circular(.9)), fill()); } }
      case HG.circle:
        cv.drawCircle(o(12, 12), 9, line(1.5));
      case HG.spiral:
        cv.drawCircle(o(12, 12), 9, line(1.4)); cv.drawCircle(o(12, 12), 5.6, line(1.4)); cv.drawCircle(o(12, 12), 2, fill());
      case HG.scatter:
        cv.drawLine(o(7, 18.3), o(11.6, 12.6), line(1)); cv.drawLine(o(11.6, 12.6), o(18.4, 17.6), line(1));
        cv.drawCircle(o(7, 18.3), 1.9, fill()); cv.drawCircle(o(11.6, 12.6), 2.9, fill()); cv.drawCircle(o(18.2, 6.6), 2.2, fill()); cv.drawCircle(o(18.6, 17.8), 2.2, fill());
      case HG.alongPath:
        final r = Rect.fromCircle(center: o(12, 12), radius: 7.8);
        cv.drawArc(r, 112 * math.pi / 180, 316 * math.pi / 180, false, line(1.8));
        cv.drawCircle(o(12 + 7.8 * math.cos(112 * math.pi / 180), 12 + 7.8 * math.sin(112 * math.pi / 180)), 1.5, fill());
        cv.drawCircle(o(12 + 7.8 * math.cos(68 * math.pi / 180), 12 + 7.8 * math.sin(68 * math.pi / 180)), 1.5, fill());
      case HG.stagger:
        cv.drawLine(o(5.5, 18.5), o(18.5, 5.5), line(1.6));
        cv.drawCircle(o(5.5, 18.5), 2.5, fill()); cv.drawCircle(o(18.5, 5.5), 2.5, fill()); cv.drawCircle(o(12, 12), 1.7, fill());
      case HG.face:
        for (var i = 0; i < 16; i++) { cv.drawArc(Rect.fromCircle(center: o(12, 12), radius: 8.6), i * math.pi / 8 + .05, math.pi / 8 * .82, false, line(1.9)); }
      case HG.follow:
        void arrow(double x0, double x1, double al) {
          final p = line(1.6, col.withValues(alpha: al * col.a));
          cv.drawLine(o(x0, 12), o(x1, 12), p); cv.drawLine(o(x1 - 3, 9), o(x1, 12), p); cv.drawLine(o(x1 - 3, 15), o(x1, 12), p);
        }
        arrow(3, 10, 1); arrow(12.5, 19.5, 1); arrow(29, 36, .15);
      case HG.attach:
        for (final p in [o(8.8, 15.2), o(15.2, 8.8)]) {
          cv.save(); cv.translate(p.dx, p.dy); cv.rotate(-math.pi / 4);
          cv.drawRRect(rr(-7, -3.6, 14, 7.2, 3.6), line(1.7));
          cv.restore();
        }
      case HG.blur:
        cv.drawCircle(o(12, 12), 9, line(1.7));
      case HG.glow:
        cv.drawRRect(rr(3.5, 3.5, 17, 17, 1.4), fill());
      case HG.color:
        cv.drawCircle(o(12, 12), 9, line(1.5));
        cv.drawPath(Path()..moveTo(12, 3.4)..arcTo(Rect.fromCircle(center: o(12, 12), radius: 8.6), -math.pi / 2, -math.pi, false)..close(), fill());
      case HG.composite:
        cv.drawRRect(rr(3, 3, 13, 13, 1.6), fill());
        cv.drawRRect(rr(8.5, 8.5, 12.5, 12.5, 1.6), fill(bg));
        cv.drawRRect(rr(8.5, 8.5, 12.5, 12.5, 1.6), line(1.5));
        cv.drawRRect(rr(11.5, 11.5, 6.6, 6.6, 1), line(1.2, Color.lerp(col, bg, .55)));
      case HG.distort:
        cv.drawPath(Path()..moveTo(3.5, 15.5)..cubicTo(4.5, 6, 10.5, 5.5, 12, 12)..cubicTo(13, 18.5, 18.5, 18.5, 20.5, 12.5), line(1.7));
        cv.drawLine(o(3.2, 19), o(4.4, 17.6), line(1.2));
      case HG.stylize:
        cv.drawPath(Path()..moveTo(12, 2.7)..lineTo(21.4, 19.3)..lineTo(2.6, 19.3)..close(), line(1.5));
        cv.drawCircle(o(12, 14.5), 1.2, fill());
      case HG.arrow:
        cv.drawPath(Path()..moveTo(6, 3)..lineTo(6, 19.5)..lineTo(10, 15.6)..lineTo(12.9, 21.6)..lineTo(15.4, 20.4)..lineTo(12.6, 14.6)..lineTo(18.4, 14.4)..close(), fill());
      case HG.move:
        cv.drawLine(o(12, 4.5), o(12, 19.5), line(1.5)); cv.drawLine(o(4.5, 12), o(19.5, 12), line(1.5));
        for (final a2 in [0.0, 1.0, 2.0, 3.0]) {
          cv.save(); cv.translate(12, 12); cv.rotate(a2 * math.pi / 2);
          cv.drawPath(Path()..moveTo(0, -10)..lineTo(-2.6, -6.6)..lineTo(2.6, -6.6)..close(), fill());
          cv.restore();
        }
      case HG.rect:
        cv.drawRRect(rr(5, 5, 14, 14, .6), line(1.7));
      case HG.ellipse:
        cv.drawCircle(o(12, 12), 7.4, line(1.6));
      case HG.pen:
        cv.drawPath(Path()..moveTo(4.5, 19.2)..lineTo(7.4, 13.2)..lineTo(16.4, 4.2)..lineTo(19.6, 7.4)..lineTo(10.6, 16.4)..close(), line(1.4));
        cv.drawLine(o(6.5, 16.5), o(9.2, 18.4), line(1.2)); cv.drawCircle(o(19, 19), .9, fill());
      case HG.crop:
        cv.drawPath(Path()..moveTo(7, 3)..lineTo(7, 17)..lineTo(21, 17), line(1.7)); cv.drawPath(Path()..moveTo(3, 7)..lineTo(17, 7)..lineTo(17, 21), line(1.7));
      case HG.search:
        cv.drawCircle(o(10.4, 10.4), 5.8, line(1.6)); cv.drawLine(o(14.8, 14.8), o(20, 20), line(1.8));
      case HG.fit:
        for (final q in [0, 1, 2, 3]) {
          cv.save(); cv.translate(12, 12); cv.rotate(q * math.pi / 2);
          cv.drawPath(Path()..moveTo(-8.5, -4)..lineTo(-8.5, -8.5)..lineTo(-4, -8.5), line(1.6)); cv.restore();
        }
        cv.drawCircle(o(12, 12), 1.7, fill());
      case HG.corners:
        for (final q in [0, 1, 2, 3]) {
          cv.save(); cv.translate(12, 12); cv.rotate(q * math.pi / 2);
          cv.drawPath(Path()..moveTo(-8.5, -3.6)..lineTo(-8.5, -8.5)..lineTo(-3.6, -8.5), line(1.6));
          cv.drawLine(o(-8, -8), o(-4.6, -4.6), line(1.2)); cv.restore();
        }
      case HG.pie:
        cv.drawCircle(o(12, 12), 8.6, line(1.7));
        cv.drawPath(Path()..moveTo(12, 12)..lineTo(12, 3.4)..arcTo(Rect.fromCircle(center: o(12, 12), radius: 8.6), -math.pi / 2, math.pi * .62, false)..close(), fill());
      case HG.kebab:
        for (final y in [5.0, 12.0, 19.0]) { cv.drawCircle(o(12, y), 1.7, fill()); }
      case HG.grid4:
        for (final x in [4.5, 13.0]) { for (final y in [4.5, 13.0]) { cv.drawRRect(rr(x, y, 6.5, 6.5, 1.2), fill()); } }
      case HG.list:
        for (final y in [5.0, 10.6, 16.2]) { cv.drawRRect(rr(4, y, 16, 3, 1), fill()); }
      case HG.play:
        cv.drawPath(Path()..moveTo(7.2, 3.5)..lineTo(20.6, 12)..lineTo(7.2, 20.5)..close(), fill());
      case HG.pin:
        cv.drawPath(Path()..moveTo(12, 3.4)..lineTo(19.5, 18.8)..lineTo(12, 15)..lineTo(4.5, 18.8)..close(), fill());
      case HG.folder:
        cv.drawPath(Path()..moveTo(3.5, 6.5)..lineTo(9.5, 6.5)..lineTo(11.5, 9)..lineTo(20.5, 9)..lineTo(20.5, 19.5)..lineTo(3.5, 19.5)..close(), line(1.5));
      case HG.lock:
        cv.drawRRect(rr(5.2, 11, 13.6, 10, 2), fill());
        cv.drawPath(Path()..moveTo(8.2, 11)..lineTo(8.2, 8.4)..arcTo(Rect.fromCircle(center: o(12, 8.4), radius: 3.8), math.pi, math.pi, false)..lineTo(15.8, 11), line(2));
        cv.drawCircle(o(12, 15.6), 1.3, fill(bg));
      case HG.plus:
        cv.drawLine(o(12, 4), o(12, 20), line(1.6)); cv.drawLine(o(4, 12), o(20, 12), line(1.6));
      case HG.star:
        final p = Path();
        for (var i = 0; i < 10; i++) {
          final r = i.isEven ? 9.4 : 4.1; final an = -math.pi / 2 + i * math.pi / 5;
          final q = o(12 + math.cos(an) * r, 12.6 + math.sin(an) * r);
          i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
        }
        cv.drawPath(p..close(), fill());
      case HG.chevronDown:
        cv.drawPath(Path()..moveTo(6, 9)..lineTo(12, 15)..lineTo(18, 9), line(1.9));
      case HG.cross:
        cv.drawLine(o(6.5, 6.5), o(17.5, 17.5), line(1.7)); cv.drawLine(o(17.5, 6.5), o(6.5, 17.5), line(1.7));
      case HG.triangle:
        cv.drawPath(Path()..moveTo(12, 3.8)..lineTo(20, 18.2)..lineTo(4, 18.2)..close(), line(1.5));
      case HG.headphones:
        cv.drawArc(Rect.fromCircle(center: o(12, 13), radius: 6.6), math.pi * 1.04, math.pi * .92, false, line(1.7));
        cv.drawRRect(rr(4, 12.4, 3.8, 7, 1.4), fill()); cv.drawRRect(rr(16.2, 12.4, 3.8, 7, 1.4), fill());
      case HG.diamond:
        cv.save(); cv.translate(12, 12); cv.rotate(math.pi / 4);
        cv.drawRRect(rr(-5.6, -5.6, 11.2, 11.2, 1.2), line(1.6)); cv.restore();
      case HG.power:
        cv.drawCircle(o(12, 12.4), 6.4, line(1.5)); cv.drawLine(o(12, 6.8), o(12, 12.4), line(1.5));
      case HG.eye || HG.eyeOff:
        // an almond with its pupil; off is the same eye struck through
        cv.drawPath(Path()..moveTo(2.5, 12)..quadraticBezierTo(12, 3, 21.5, 12)..quadraticBezierTo(12, 21, 2.5, 12)..close(), line(1.7));
        cv.drawCircle(o(12, 12), 3, fill());
        if (g == HG.eyeOff) {
          cv.drawLine(o(4.5, 20), o(19.5, 4), line(3.4, bg));
          cv.drawLine(o(4.5, 20), o(19.5, 4), line(1.7));
        }
      case HG.solo:
        // a target: the one heard/seen alone
        cv.drawCircle(o(12, 12), 7.4, line(1.7));
        cv.drawCircle(o(12, 12), 3, fill());
      case HG.preset:
        cv.drawRRect(rr(2.5, 5, 19, 14, 3.4), fill());
        cv.drawCircle(o(8.6, 11.4), 1.5, fill(bg)); cv.drawCircle(o(15.4, 11.4), 1.5, fill(bg)); cv.drawLine(o(9, 15), o(15, 15), line(1.2, bg));
    }
  }
  @override
  bool shouldRepaint(HgPainter o) => o.g != g || o.c != c || o.a != a;
}
