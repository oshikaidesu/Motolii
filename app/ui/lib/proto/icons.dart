// Hand-drawn glyphs for the prototype. Each is a small CustomPainter.
import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'tokens.dart';

enum Glyph {
  text, shape, image, camera, repeater, grid, circle, spiral,
  scatter, alongPath, stagger, face, follow, attach,
  blur, glow, color, composite, distort, stylize,
  arrow, move, rect, ellipse, pen, type, crop,
  play, stop, record, fullscreen, pin, folder, search, lock, plus, chevronDown, chevronRight, diamond, fit, fit2, preset, eye, lockSmall, keyframe, star,
}

class Icon1 extends StatelessWidget {
  const Icon1(this.glyph, {super.key, this.size = 22, this.color = P.text, this.stroke = 1.6});
  final Glyph glyph;
  final double size, stroke;
  final Color color;
  @override
  Widget build(BuildContext context) => CustomPaint(size: Size.square(size), painter: _GlyphPainter(glyph, color, stroke));
}

class _GlyphPainter extends CustomPainter {
  _GlyphPainter(this.g, this.c, this.sw);
  final Glyph g;
  final Color c;
  final double sw;
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = sw..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final f = Paint()..color = c..style = PaintingStyle.fill;
    final w = s.width, h = s.height, cx = w / 2, cy = h / 2;
    Rect r(double inset) => Rect.fromLTWH(inset, inset, w - inset * 2, h - inset * 2);
    switch (g) {
      case Glyph.text:
      case Glyph.type:
        cv.drawLine(Offset(w * .2, h * .2), Offset(w * .8, h * .2), p..strokeWidth = sw * 1.2);
        cv.drawLine(Offset(cx, h * .2), Offset(cx, h * .85), p);
      case Glyph.shape:
        cv.drawRect(r(w * .2), f);
      case Glyph.rect:
        cv.drawRect(r(w * .22), p);
      case Glyph.image:
        cv.drawRRect(RRect.fromRectAndRadius(r(w * .15), const Radius.circular(1.5)), p);
        final path = Path()..moveTo(w * .2, h * .75)..lineTo(w * .42, h * .48)..lineTo(w * .56, h * .62)..lineTo(w * .66, h * .52)..lineTo(w * .82, h * .75)..close();
        cv.drawPath(path, f);
        cv.drawCircle(Offset(w * .66, h * .34), w * .07, f);
      case Glyph.camera:
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .12, h * .3, w * .76, h * .5), const Radius.circular(2)), p);
        cv.drawRect(Rect.fromLTWH(w * .36, h * .2, w * .28, h * .1), p);
        cv.drawCircle(Offset(cx, h * .55), w * .14, p);
      case Glyph.repeater:
        for (final o in [Offset(.3, .3), Offset(.7, .3), Offset(.3, .7), Offset(.7, .7)]) {
          cv.drawRect(Rect.fromCenter(center: Offset(o.dx * w, o.dy * h), width: w * .16, height: w * .16), f);
        }
        cv.drawRect(Rect.fromCenter(center: Offset(cx, cy), width: w * .12, height: w * .12), f..color = c.withValues(alpha: .5));
      case Glyph.grid:
        for (var i = 0; i < 3; i++) {
          for (var j = 0; j < 3; j++) {
            cv.drawRect(Rect.fromCenter(center: Offset(w * (.25 + i * .25), h * (.25 + j * .25)), width: w * .14, height: w * .14), f);
          }
        }
      case Glyph.circle:
        cv.drawCircle(Offset(cx, cy), w * .34, p);
      case Glyph.ellipse:
        cv.drawCircle(Offset(cx, cy), w * .3, p);
      case Glyph.spiral:
        cv.drawCircle(Offset(cx, cy), w * .34, p);
        cv.drawCircle(Offset(cx, cy), w * .2, p);
        cv.drawCircle(Offset(cx, cy), w * .07, f);
      case Glyph.scatter:
        for (final o in [Offset(.3, .3), Offset(.62, .22), Offset(.75, .55), Offset(.4, .6), Offset(.55, .78), Offset(.22, .74)]) {
          cv.drawCircle(Offset(o.dx * w, o.dy * h), w * .06, f);
        }
        cv.drawLine(Offset(w * .3, h * .3), Offset(w * .75, h * .55), p..strokeWidth = 1);
        cv.drawLine(Offset(w * .4, h * .6), Offset(w * .55, h * .78), p);
      case Glyph.alongPath:
        final path = Path()..moveTo(w * .2, h * .75)..cubicTo(w * .15, h * .15, w * .85, h * .15, w * .8, h * .75);
        cv.drawPath(path, p..strokeWidth = sw * 1.3);
        cv.drawCircle(Offset(w * .2, h * .75), w * .08, f);
      case Glyph.stagger:
        cv.drawLine(Offset(w * .2, h * .8), Offset(w * .8, h * .2), p..strokeWidth = sw * 1.2);
        cv.drawCircle(Offset(w * .2, h * .8), w * .09, f);
        cv.drawCircle(Offset(w * .5, h * .5), w * .09, f);
        cv.drawCircle(Offset(w * .8, h * .2), w * .09, f);
      case Glyph.face:
        cv.drawCircle(Offset(cx, cy), w * .3, p..strokeWidth = sw * 1.3);
      case Glyph.follow:
        cv.drawLine(Offset(w * .12, cy), Offset(w * .48, cy), p);
        cv.drawLine(Offset(w * .4, cy - h * .12), Offset(w * .5, cy), p);
        cv.drawLine(Offset(w * .4, cy + h * .12), Offset(w * .5, cy), p);
        cv.drawLine(Offset(w * .56, cy), Offset(w * .88, cy), p);
        cv.drawLine(Offset(w * .8, cy - h * .12), Offset(w * .9, cy), p);
        cv.drawLine(Offset(w * .8, cy + h * .12), Offset(w * .9, cy), p);
      case Glyph.attach:
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(w * .38, h * .62), width: w * .34, height: h * .2), Radius.circular(h * .1)), p);
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(w * .62, h * .38), width: w * .34, height: h * .2), Radius.circular(h * .1)), p);
      case Glyph.blur:
        cv.drawCircle(Offset(cx, cy), w * .3, p);
      case Glyph.glow:
        cv.drawRect(r(w * .25), f);
      case Glyph.color:
        cv.drawCircle(Offset(cx, cy), w * .3, p);
        cv.drawPath(Path()..addArc(Rect.fromCircle(center: Offset(cx, cy), radius: w * .3), -math.pi / 2, math.pi)..lineTo(cx, cy)..close(), f);
      case Glyph.composite:
        cv.drawRect(Rect.fromLTWH(w * .18, h * .18, w * .45, h * .45), p);
        cv.drawRect(Rect.fromLTWH(w * .38, h * .38, w * .45, h * .45), f);
      case Glyph.distort:
        final path = Path()..moveTo(w * .1, cy)..cubicTo(w * .3, h * .1, w * .4, h * .9, cx, cy)..cubicTo(w * .6, h * .1, w * .7, h * .9, w * .9, cy);
        cv.drawPath(path, p);
      case Glyph.stylize:
        cv.drawPath(Path()..moveTo(cx, h * .15)..lineTo(w * .85, h * .82)..lineTo(w * .15, h * .82)..close(), p);
      case Glyph.arrow:
        cv.drawPath(Path()..moveTo(w * .3, h * .18)..lineTo(w * .3, h * .78)..lineTo(w * .45, h * .63)..lineTo(w * .56, h * .86)..lineTo(w * .64, h * .82)..lineTo(w * .53, h * .6)..lineTo(w * .74, h * .6)..close(), f);
      case Glyph.move:
        cv.drawLine(Offset(cx, h * .12), Offset(cx, h * .88), p);
        cv.drawLine(Offset(w * .12, cy), Offset(w * .88, cy), p);
        for (final d in [0, 1, 2, 3]) {
          cv.save();
          cv.translate(cx, cy);
          cv.rotate(d * math.pi / 2);
          cv.drawLine(Offset(0, -h * .38), Offset(-w * .1, -h * .28), p);
          cv.drawLine(Offset(0, -h * .38), Offset(w * .1, -h * .28), p);
          cv.restore();
        }
      case Glyph.pen:
        cv.drawLine(Offset(w * .25, h * .75), Offset(w * .72, h * .28), p..strokeWidth = sw * 1.5);
        cv.drawLine(Offset(w * .25, h * .75), Offset(w * .2, h * .82), p);
      case Glyph.crop:
        cv.drawPath(Path()..moveTo(w * .3, h * .1)..lineTo(w * .3, h * .7)..lineTo(w * .9, h * .7), p);
        cv.drawPath(Path()..moveTo(w * .1, h * .3)..lineTo(w * .7, h * .3)..lineTo(w * .7, h * .9), p);
      case Glyph.play:
        cv.drawPath(Path()..moveTo(w * .3, h * .2)..lineTo(w * .78, cy)..lineTo(w * .3, h * .8)..close(), f);
      case Glyph.stop:
        cv.drawRect(r(w * .28), f);
      case Glyph.record:
        cv.drawCircle(Offset(cx, cy), w * .24, f);
      case Glyph.fullscreen:
      case Glyph.fit:
        for (final q in [0, 1, 2, 3]) {
          cv.save();
          cv.translate(cx, cy);
          cv.rotate(q * math.pi / 2);
          cv.drawPath(Path()..moveTo(-w * .34, -h * .14)..lineTo(-w * .34, -h * .34)..lineTo(-w * .14, -h * .34), p);
          cv.restore();
        }
      case Glyph.fit2:
        for (final q in [0, 1, 2, 3]) {
          cv.save();
          cv.translate(cx, cy);
          cv.rotate(q * math.pi / 2);
          cv.drawPath(Path()..moveTo(-w * .34, -h * .14)..lineTo(-w * .34, -h * .34)..lineTo(-w * .14, -h * .34), p);
          cv.restore();
        }
        cv.drawRect(r(w * .38), p);
      case Glyph.pin:
        cv.drawPath(Path()..moveTo(cx, h * .15)..lineTo(w * .75, h * .8)..lineTo(cx, h * .65)..lineTo(w * .25, h * .8)..close(), f);
      case Glyph.folder:
        cv.drawPath(Path()..moveTo(w * .12, h * .3)..lineTo(w * .4, h * .3)..lineTo(w * .48, h * .38)..lineTo(w * .88, h * .38)..lineTo(w * .88, h * .78)..lineTo(w * .12, h * .78)..close(), p);
      case Glyph.search:
        cv.drawCircle(Offset(w * .42, h * .42), w * .24, p);
        cv.drawLine(Offset(w * .6, h * .6), Offset(w * .84, h * .84), p);
      case Glyph.lock:
      case Glyph.lockSmall:
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .22, h * .45, w * .56, h * .4), const Radius.circular(1.5)), f);
        cv.drawPath(Path()..addArc(Rect.fromLTWH(w * .32, h * .18, w * .36, h * .5), math.pi, math.pi), p);
      case Glyph.plus:
        cv.drawLine(Offset(cx, h * .2), Offset(cx, h * .8), p);
        cv.drawLine(Offset(w * .2, cy), Offset(w * .8, cy), p);
      case Glyph.chevronDown:
        cv.drawPath(Path()..moveTo(w * .25, h * .38)..lineTo(cx, h * .62)..lineTo(w * .75, h * .38), p);
      case Glyph.chevronRight:
        cv.drawPath(Path()..moveTo(w * .3, h * .2)..lineTo(w * .75, cy)..lineTo(w * .3, h * .8)..close(), f);
      case Glyph.diamond:
      case Glyph.keyframe:
        cv.save();
        cv.translate(cx, cy);
        cv.rotate(math.pi / 4);
        cv.drawRect(Rect.fromCenter(center: Offset.zero, width: w * .42, height: w * .42), p);
        cv.restore();
      case Glyph.preset:
        cv.drawRect(Rect.fromLTWH(w * .15, h * .2, w * .7, h * .6), p);
        cv.drawLine(Offset(w * .15, h * .4), Offset(w * .85, h * .4), p);
      case Glyph.star:
        final path = Path();
        for (var i = 0; i < 10; i++) {
          final rr = i.isEven ? w * .42 : w * .18;
          final a = -math.pi / 2 + i * math.pi / 5;
          final o = Offset(cx + math.cos(a) * rr, cy + math.sin(a) * rr);
          i == 0 ? path.moveTo(o.dx, o.dy) : path.lineTo(o.dx, o.dy);
        }
        cv.drawPath(path..close(), f);
      case Glyph.eye:
        cv.drawPath(Path()..moveTo(w * .1, cy)..quadraticBezierTo(cx, h * .1, w * .9, cy)..quadraticBezierTo(cx, h * .9, w * .1, cy), p);
        cv.drawCircle(Offset(cx, cy), w * .12, f);
    }
  }
  @override
  bool shouldRepaint(_GlyphPainter o) => o.g != g || o.c != c;
}
