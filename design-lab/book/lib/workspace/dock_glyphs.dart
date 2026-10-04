// The dock's painted marks: shelves and panes (on their tabs), effect families, things you can create, files. One stroke weight, filled where a pop poster would fill.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

enum DockG {
  media,
  effects,
  fonts,
  colors,
  files,
  create,
  search,
  close,
  chevron,
  grid,
  list,
  sort,
  filter,
  plus,
  light,
  blur,
  distort,
  colour,
  stylize,
  copies,
  paths,
  text,
  particles,
  masks,
  time,
  shape,
  camera,
  nullObj,
  solid,
  adjust,
  group,
  rect,
  ellipse,
  polygon,
  star,
  line,
  pen,
  radial,
  folder,
  comp,
  image,
  audio,
  video,
  font,
  doc,
  link,
  stage,
  inspector,
  timeline,
  desk,
}

class DockGlyph extends StatelessWidget {
  const DockGlyph(this.g, {super.key, this.size = 14, required this.color});
  final DockG g;
  final double size;
  final Color color;
  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: size,
    child: CustomPaint(painter: _P(g, color)),
  );
}

class _P extends CustomPainter {
  const _P(this.g, this.c);
  final DockG g;
  final Color c;

  @override
  void paint(Canvas cv, Size s) {
    final w = s.width;
    final st = Paint()
      ..color = c
      ..style = PaintingStyle.stroke
      ..strokeWidth = math.max(1.2, w * .1)
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    final fl = Paint()..color = c;
    Offset p(double x, double y) => Offset(x * w, y * w);
    Rect r(double l, double t, double rw, double rh) => Rect.fromLTWH(l * w, t * w, rw * w, rh * w);
    RRect rr(double l, double t, double rw, double rh, [double rad = .1]) => RRect.fromRectAndRadius(r(l, t, rw, rh), Radius.circular(rad * w));
    Path poly(List<Offset> pts) => Path()..addPolygon(pts.map((o) => p(o.dx, o.dy)).toList(), true);
    Path ngon(int n, double rad, {double inner = 0, double rot = -math.pi / 2}) {
      final pts = <Offset>[];
      final k = inner > 0 ? n * 2 : n;
      for (var i = 0; i < k; i++) {
        final a = rot + i * 2 * math.pi / k, rv = inner > 0 && i.isOdd ? inner : rad;
        pts.add(Offset(.5 + math.cos(a) * rv, .5 + math.sin(a) * rv));
      }
      return poly(pts);
    }

    switch (g) {
      case DockG.media || DockG.video:
        cv.drawRRect(rr(.1, .2, .8, .6, .12), st);
        cv.drawPath(poly(const [Offset(.42, .36), Offset(.64, .5), Offset(.42, .64)]), fl);
      case DockG.effects:
        cv.drawPath(ngon(4, .42, inner: .13, rot: -math.pi / 2), fl);
        cv.drawCircle(p(.84, .18), w * .07, fl);
      case DockG.fonts || DockG.font:
        cv.drawPath(
          Path()
            ..moveTo(w * .14, w * .84)
            ..lineTo(w * .4, w * .16)
            ..lineTo(w * .66, w * .84),
          st,
        );
        cv.drawLine(p(.24, .6), p(.56, .6), st);
        cv.drawCircle(p(.8, .7), w * .13, st);
      case DockG.colors:
        cv.drawCircle(p(.38, .38), w * .22, fl);
        cv.drawCircle(p(.62, .38), w * .22, st);
        cv.drawCircle(p(.5, .64), w * .22, st);
      case DockG.files || DockG.folder:
        cv.drawPath(
          Path()
            ..moveTo(w * .1, w * .24)
            ..lineTo(w * .4, w * .24)
            ..lineTo(w * .5, w * .34)
            ..lineTo(w * .9, w * .34)
            ..lineTo(w * .9, w * .8)
            ..lineTo(w * .1, w * .8)
            ..close(),
          g == DockG.folder ? fl : st,
        );
      case DockG.create || DockG.plus:
        if (g == DockG.create) cv.drawRRect(rr(.12, .12, .76, .76, .18), st);
        final k = g == DockG.create ? .3 : .2;
        cv.drawLine(p(.5, k), p(.5, 1 - k), st);
        cv.drawLine(p(k, .5), p(1 - k, .5), st);
      case DockG.search:
        cv.drawCircle(p(.43, .43), w * .26, st);
        cv.drawLine(p(.63, .63), p(.86, .86), st);
      case DockG.close:
        cv.drawLine(p(.28, .28), p(.72, .72), st);
        cv.drawLine(p(.72, .28), p(.28, .72), st);
      case DockG.chevron:
        cv.drawPath(poly(const [Offset(.34, .22), Offset(.72, .5), Offset(.34, .78)]), fl);
      case DockG.stage:
        cv.drawRRect(rr(.08, .2, .84, .6, .08), st);
        cv.drawCircle(p(.5, .5), w * .1, fl);
      case DockG.inspector:
        for (final (i, x) in const [.34, .66, .46].indexed) {
          cv.drawLine(p(.12, .24 + i * .26), p(.88, .24 + i * .26), st);
          cv.drawCircle(p(x, .24 + i * .26), w * .1, fl);
        }
      case DockG.timeline:
        cv.drawRRect(rr(.08, .2, .46, .14, .04), fl);
        cv.drawRRect(rr(.26, .43, .5, .14, .04), fl);
        cv.drawRRect(rr(.16, .66, .38, .14, .04), fl);
        cv.drawLine(p(.86, .12), p(.86, .88), st);
      case DockG.desk:
        cv.drawRRect(rr(.12, .12, .34, .34, .08), fl);
        cv.drawRRect(rr(.54, .12, .34, .34, .08), st);
        cv.drawRRect(rr(.12, .54, .34, .34, .08), st);
        cv.drawRRect(rr(.54, .54, .34, .34, .08), st);
      case DockG.grid:
        for (var i = 0; i < 2; i++) {
          for (var j = 0; j < 2; j++) {
            cv.drawRRect(rr(.14 + i * .4, .14 + j * .4, .32, .32, .06), fl);
          }
        }
      case DockG.list:
        for (var i = 0; i < 3; i++) {
          cv.drawRRect(rr(.12, .18 + i * .26, .14, .14, .04), fl);
          cv.drawLine(p(.38, .25 + i * .26), p(.88, .25 + i * .26), st);
        }
      case DockG.sort:
        for (var i = 0; i < 3; i++) {
          cv.drawLine(p(.16, .26 + i * .24), p(.84 - i * .22, .26 + i * .24), st);
        }
      case DockG.filter:
        for (var i = 0; i < 3; i++) {
          cv.drawLine(p(.14 + i * .14, .26 + i * .24), p(.86 - i * .14, .26 + i * .24), st);
        }
      case DockG.light:
        cv.drawCircle(p(.5, .5), w * .18, fl);
        for (var i = 0; i < 8; i++) {
          final a = i * math.pi / 4;
          cv.drawLine(p(.5 + math.cos(a) * .3, .5 + math.sin(a) * .3), p(.5 + math.cos(a) * .44, .5 + math.sin(a) * .44), st);
        }
      case DockG.blur:
        cv.drawCircle(p(.5, .5), w * .14, fl);
        cv.drawCircle(p(.5, .5), w * .38, st..strokeWidth = math.max(1, w * .06));
        cv.drawCircle(p(.5, .5), w * .26, st);
      case DockG.distort:
        for (var j = 0; j < 2; j++) {
          final path = Path()..moveTo(w * .1, w * (.36 + j * .3));
          for (var x = .1; x <= .9; x += .05) {
            path.lineTo(w * x, w * (.36 + j * .3 + math.sin((x - .1) * 2 * math.pi * 1.25) * .1));
          }
          cv.drawPath(path, st);
        }
      case DockG.colour:
        cv.drawCircle(p(.5, .5), w * .36, st);
        cv.drawArc(r(.14, .14, .72, .72), -math.pi / 2, math.pi, true, fl);
      case DockG.stylize:
        for (var i = 0; i < 3; i++) {
          for (var j = 0; j < 3; j++) {
            if ((i + j).isEven) cv.drawRect(r(.14 + i * .24, .14 + j * .24, .24, .24), fl);
          }
        }
        cv.drawRect(r(.14, .14, .72, .72), st..strokeWidth = math.max(1, w * .06));
      case DockG.copies:
        cv.drawRRect(rr(.12, .12, .46, .46), st);
        cv.drawRRect(rr(.27, .27, .46, .46), st);
        cv.drawRRect(rr(.42, .42, .46, .46), fl);
      case DockG.paths:
        cv.drawPath(
          Path()
            ..moveTo(w * .14, w * .8)
            ..cubicTo(w * .2, w * .2, w * .8, w * .8, w * .86, w * .2),
          st,
        );
        cv.drawCircle(p(.14, .8), w * .09, fl);
        cv.drawCircle(p(.86, .2), w * .09, fl);
      case DockG.text:
        cv.drawLine(p(.18, .2), p(.82, .2), st);
        cv.drawLine(p(.5, .2), p(.5, .84), st);
      case DockG.particles:
        const dots = [(.2, .3, .08), (.46, .18, .06), (.74, .3, .1), (.3, .62, .1), (.6, .56, .07), (.82, .74, .06), (.5, .84, .08)];
        for (final (x, y, rad) in dots) {
          cv.drawCircle(p(x, y), w * rad, fl);
        }
      case DockG.masks:
        cv.drawRRect(rr(.12, .12, .76, .76, .12), st);
        cv.drawCircle(p(.5, .5), w * .22, fl);
      case DockG.time:
        cv.drawCircle(p(.5, .5), w * .36, st);
        cv.drawLine(p(.5, .5), p(.5, .28), st);
        cv.drawLine(p(.5, .5), p(.66, .58), st);
      case DockG.shape:
        cv.drawRect(r(.12, .36, .46, .46), fl);
        cv.drawCircle(p(.64, .38), w * .24, st);
      case DockG.camera:
        cv.drawRRect(rr(.08, .3, .6, .44, .1), fl);
        cv.drawPath(poly(const [Offset(.72, .52), Offset(.92, .34), Offset(.92, .7)]), fl);
      case DockG.nullObj:
        cv.drawRect(r(.18, .18, .64, .64), st);
        cv.drawLine(p(.5, .3), p(.5, .7), st);
        cv.drawLine(p(.3, .5), p(.7, .5), st);
      case DockG.solid:
        cv.drawRect(r(.16, .16, .68, .68), fl);
      case DockG.adjust:
        cv.drawRect(r(.16, .16, .68, .68), st);
        cv.drawPath(poly(const [Offset(.16, .84), Offset(.84, .16), Offset(.84, .84)]), fl);
      case DockG.group:
        cv.drawRRect(rr(.1, .28, .62, .5, .08), st);
        cv.drawRRect(rr(.28, .16, .62, .5, .08), fl);
      case DockG.rect:
        cv.drawRRect(rr(.14, .22, .72, .56, .06), fl);
      case DockG.ellipse:
        cv.drawOval(r(.12, .2, .76, .6), fl);
      case DockG.polygon:
        cv.drawPath(ngon(6, .4, rot: 0), fl);
      case DockG.star:
        cv.drawPath(ngon(5, .44, inner: .19), fl);
      case DockG.line:
        cv.drawLine(p(.16, .84), p(.84, .16), st);
        cv.drawCircle(p(.16, .84), w * .08, fl);
        cv.drawCircle(p(.84, .16), w * .08, fl);
      case DockG.pen:
        cv.drawPath(poly(const [Offset(.5, .1), Offset(.78, .48), Offset(.58, .9), Offset(.42, .9), Offset(.22, .48)]), fl);
      case DockG.radial:
        for (var i = 0; i < 8; i++) {
          final a = i * math.pi / 4;
          cv.drawCircle(p(.5 + math.cos(a) * .34, .5 + math.sin(a) * .34), w * .08, fl);
        }
      case DockG.comp:
        cv.drawRect(r(.1, .22, .8, .56), st);
        for (var i = 0; i < 4; i++) {
          cv.drawRect(r(.18 + i * .18, .3, .1, .08), fl);
          cv.drawRect(r(.18 + i * .18, .62, .1, .08), fl);
        }
      case DockG.image:
        cv.drawRect(r(.1, .2, .8, .6), st);
        cv.drawPath(poly(const [Offset(.18, .74), Offset(.42, .44), Offset(.6, .64), Offset(.7, .54), Offset(.84, .74)]), fl);
        cv.drawCircle(p(.68, .36), w * .07, fl);
      case DockG.audio:
        const hs = [.2, .5, .34, .7, .44, .26];
        for (var i = 0; i < hs.length; i++) {
          final x = .14 + i * .144;
          cv.drawLine(p(x, .5 - hs[i] / 2), p(x, .5 + hs[i] / 2), st);
        }
      case DockG.doc:
        cv.drawPath(poly(const [Offset(.22, .1), Offset(.6, .1), Offset(.8, .3), Offset(.8, .9), Offset(.22, .9)]), st);
        cv.drawLine(p(.34, .52), p(.68, .52), st);
        cv.drawLine(p(.34, .7), p(.6, .7), st);
      case DockG.link:
        cv.drawRRect(rr(.08, .34, .5, .32, .16), st);
        cv.drawRRect(rr(.42, .34, .5, .32, .16), st);
    }
  }

  @override
  bool shouldRepaint(_P o) => o.g != g || o.c != c;
}
