// A small line-icon set, drawn: one stroke weight, one size grid, g63 idle / g95 active. Placeholders (empty squares) are what the critique flagged, so every tool and tile has a real mark.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../tokens.dart';

enum G { select, move, rect, ellipse, pen, text, image, camera, repeater, grid, circle, spiral, shape, effect, media }

class Glyph extends StatelessWidget {
  const Glyph(this.kind, {super.key, this.size = 18, this.color = N.g63});
  final G kind;
  final double size;
  final Color color;
  @override
  Widget build(BuildContext context) => SizedBox.square(dimension: size, child: CustomPaint(painter: _GlyphPainter(kind, color)));
}

class _GlyphPainter extends CustomPainter {
  const _GlyphPainter(this.kind, this.color);
  final G kind;
  final Color color;
  @override
  void paint(Canvas c, Size s) {
    final w = s.width, st = Paint()..color = color..style = PaintingStyle.stroke..strokeWidth = w * .08..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round, fl = Paint()..color = color;
    Offset p(double x, double y) => Offset(x * w, y * w);
    switch (kind) {
      case G.select:
        c.drawPath(Path()..moveTo(w * .26, w * .14)..lineTo(w * .26, w * .82)..lineTo(w * .44, w * .66)..lineTo(w * .58, w * .9)..lineTo(w * .68, w * .85)..lineTo(w * .55, w * .62)..lineTo(w * .78, w * .6)..close(), st);
      case G.move:
        c.drawLine(p(.5, .12), p(.5, .88), st);
        c.drawLine(p(.12, .5), p(.88, .5), st);
        for (final d in const [Offset(.5, .12), Offset(.5, .88), Offset(.12, .5), Offset(.88, .5)]) {
          final dx = d.dx == .5 ? 0.0 : (d.dx < .5 ? .09 : -.09), dy = d.dy == .5 ? 0.0 : (d.dy < .5 ? .09 : -.09);
          c.drawLine(p(d.dx, d.dy), p(d.dx + (dx == 0 ? .08 : dx), d.dy + (dy == 0 ? .08 : dy)), st);
          c.drawLine(p(d.dx, d.dy), p(d.dx + (dx == 0 ? -.08 : dx), d.dy + (dy == 0 ? -.08 : dy)), st);
        }
      case G.rect || G.shape:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .18, w * .22, w * .64, w * .56), Radius.circular(w * .08)), st);
      case G.ellipse || G.circle:
        c.drawOval(Rect.fromLTWH(w * .16, w * .2, w * .68, w * .6), st);
      case G.pen:
        c.drawPath(Path()..moveTo(w * .5, w * .1)..lineTo(w * .78, w * .46)..lineTo(w * .56, w * .9)..lineTo(w * .44, w * .9)..lineTo(w * .22, w * .46)..close(), st);
        c.drawCircle(p(.5, .52), w * .06, fl);
        c.drawLine(p(.5, .58), p(.5, .9), st);
      case G.text:
        c.drawLine(p(.22, .2), p(.78, .2), st);
        c.drawLine(p(.5, .2), p(.5, .84), st);
        c.drawLine(p(.38, .84), p(.62, .84), st);
      case G.image:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .14, w * .2, w * .72, w * .6), Radius.circular(w * .08)), st);
        c.drawPath(Path()..moveTo(w * .2, w * .72)..lineTo(w * .42, w * .46)..lineTo(w * .58, w * .62)..lineTo(w * .68, w * .52)..lineTo(w * .82, w * .7), st);
        c.drawCircle(p(.66, .36), w * .05, fl);
      case G.camera:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .12, w * .3, w * .76, w * .5), Radius.circular(w * .08)), st);
        c.drawCircle(p(.5, .55), w * .15, st);
        c.drawLine(p(.34, .3), p(.4, .2), st);
        c.drawLine(p(.4, .2), p(.6, .2), st);
        c.drawLine(p(.6, .2), p(.66, .3), st);
      case G.repeater:
        for (var i = 0; i < 3; i++) {
          for (var j = 0; j < 3; j++) {
            c.drawCircle(p(.24 + i * .26, .24 + j * .26), w * .05, fl);
          }
        }
      case G.grid:
        for (var i = 0; i < 2; i++) {
          c.drawLine(p(.38 + i * .24, .16), p(.38 + i * .24, .84), st);
          c.drawLine(p(.16, .38 + i * .24), p(.84, .38 + i * .24), st);
        }
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .16, w * .16, w * .68, w * .68), Radius.circular(w * .06)), st);
      case G.spiral:
        final path = Path()..moveTo(w * .5, w * .5);
        for (var a = 0.0; a < 12.5; a += .2) {
          final r = w * .04 * a / 1.6;
          path.lineTo(w * .5 + r * (a == 0 ? 0 : 1) * math.cos(a), w * .5 + r * math.sin(a));
        }
        c.drawPath(path, st);
      case G.effect:
        c.drawCircle(p(.5, .5), w * .3, st);
        c.drawCircle(p(.5, .5), w * .14, st);
        for (var i = 0; i < 4; i++) {
          c.drawLine(p(.5 + (i.isEven ? 0 : (i == 1 ? .36 : -.36)), .5 + (i.isEven ? (i == 0 ? -.36 : .36) : 0)), p(.5 + (i.isEven ? 0 : (i == 1 ? .44 : -.44)), .5 + (i.isEven ? (i == 0 ? -.44 : .44) : 0)), st);
        }
      case G.media:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .14, w * .2, w * .72, w * .6), Radius.circular(w * .08)), st);
        c.drawPath(Path()..moveTo(w * .44, w * .38)..lineTo(w * .64, w * .5)..lineTo(w * .44, w * .62)..close(), fl);
    }
  }

  @override
  bool shouldRepaint(_GlyphPainter o) => o.kind != kind || o.color != color;
}
