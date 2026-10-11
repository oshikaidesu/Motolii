// Line-drawn figures shared by both sheets (cow, monkey drummer, fish, chameleon, parrot, boxer, walker). Outline only.
part of 'op_08.dart';

Offset _qb(Offset a, Offset k, Offset b, double u) => a * ((1 - u) * (1 - u)) + k * (2 * u * (1 - u)) + b * (u * u);

/// Cow facing left; [p] = left foot on the ground, [sc] = body length.
void _cow(Canvas c, Offset p, double sc, Color col, double ph) {
  Offset u(double x, double y) => p + Offset(x * sc, (y - .62) * sc);
  final hb = math.sin(ph) * .025;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(u(.24, .16), u(.86, .44)), Radius.circular(sc * .07)), _s(col, 1));
  _pl(c, [u(.26, .2), u(.13, .15 + hb), u(.03, .29 + hb), u(.06, .36 + hb), u(.16, .35 + hb), u(.25, .3)], col, w: 1);
  _ln(c, u(.13, .15 + hb), u(.1, .07 + hb), col, 1);
  _ln(c, u(.17, .16 + hb), u(.21, .08 + hb), col, 1);
  _ln(c, u(.2, .19 + hb), u(.27, .14 + hb), col, 1);
  _dot(c, u(.12, .23 + hb), sc * .018, col);
  _dot(c, u(.05, .32 + hb), sc * .012, col);
  final sw = math.sin(ph * .7) * .02;
  for (final x in [.3, .38, .74, .82]) {
    _ln(c, u(x, .44), u(x + (x < .5 ? sw : -sw), .62), col, 1);
  }
  _pl(c, [u(.86, .2), u(.91, .3 + sw), u(.93, .4)], col, w: 1);
  _dot(c, u(.93, .41), sc * .016, col);
  _ring(c, u(.6, .27), sc * .055, col, 1);
  c.drawArc(Rect.fromCircle(center: u(.68, .44), radius: sc * .04), 0, math.pi, false, _s(col, 1));
}

/// Monkey on a drum. [hit] 0..1 where 0 = stick on the head.
void _monkey(Canvas c, Offset p, double r, Color col, double hit, Color spark) {
  _ring(c, p, r, col, 1);
  _ring(c, p + Offset(-r * 1.1, -r * .1), r * .38, col, 1);
  _ring(c, p + Offset(r * 1.1, -r * .1), r * .38, col, 1);
  c.drawOval(Rect.fromCenter(center: p + Offset(0, r * .3), width: r * 1.2, height: r * .9), _s(col, 1));
  _dot(c, p + Offset(-r * .32, -r * .25), .9, col);
  _dot(c, p + Offset(r * .32, -r * .25), .9, col);
  final sh = p + Offset(0, r * 1.5), hip = p + Offset(0, r * 2.6);
  _ln(c, p + Offset(0, r), hip, col, 1);
  final drum = p + Offset(0, r * 3.4);
  final lift = math.sin(math.min(1.0, hit * 1.6) * math.pi * .5);
  for (final sd in [-1.0, 1.0]) {
    final hand = sh + Offset(sd * r * 1.3, -r * .2 - lift * r * 1.2);
    _ln(c, sh, hand, col, 1);
    final tip = hand + Offset(sd * -r * .5, r * (1.1 - lift * 2.2));
    _ln(c, hand, tip, col, 1);
  }
  c.drawOval(Rect.fromCenter(center: drum, width: r * 3.6, height: r * 1.1), _s(hit < .12 ? spark : col, 1));
  _ln(c, drum + Offset(-r * 1.8, 0), drum + Offset(-r * 1.8, r * 1.5), col, 1);
  _ln(c, drum + Offset(r * 1.8, 0), drum + Offset(r * 1.8, r * 1.5), col, 1);
  c.drawArc(Rect.fromCenter(center: drum + Offset(0, r * 1.5), width: r * 3.6, height: r * 1.1), 0, math.pi, false, _s(col, 1));
  if (hit < .12) {
    for (var k = 0; k < 5; k++) {
      final a = -math.pi * (.15 + k * .175);
      final d = Offset(math.cos(a), math.sin(a));
      _ln(c, drum + d * r * 2.1, drum + d * r * 2.8, spark, 1);
    }
  }
}

/// Fish facing along [ang]; [p] = centre, [l] = length.
void _fish(Canvas c, Offset p, double l, double ang, Color col, double wig) {
  c.save();
  c.translate(p.dx, p.dy);
  c.rotate(ang);
  final body = Path()
    ..moveTo(l * .5, 0)
    ..quadraticBezierTo(0, -l * .3, -l * .3, 0)
    ..quadraticBezierTo(0, l * .3, l * .5, 0);
  c.drawPath(body, _s(col, 1));
  final tw = math.sin(wig) * l * .08;
  _pl(c, [Offset(-l * .3, 0), Offset(-l * .52, -l * .2 + tw), Offset(-l * .52, l * .2 + tw)], col, w: 1, close: true);
  _dot(c, Offset(l * .26, -l * .04), .9, col);
  c.drawArc(Rect.fromCircle(center: Offset(l * .1, 0), radius: l * .12), -1, 2, false, _s(col, 1));
  c.restore();
}

/// Chameleon on a branch; striped in [stripe]. [o] top-left, [sc] scale (unit box 1 x .65).
void _chameleon(Canvas c, Offset o, double sc, Color line, Color stripe, double eye, double tongue) {
  Offset u(double x, double y) => o + Offset(x * sc, y * sc);
  final t0 = u(.26, .42), tk = u(.46, -.02), t1 = u(.74, .2);
  final b0 = u(.28, .5), bk = u(.5, .56), b1 = u(.74, .42);
  for (var i = 1; i < 12; i++) {
    final v = .08 + i * .075;
    _ln(c, _qb(t0, tk, t1, v), _qb(b0, bk, b1, v), stripe, 1.3);
  }
  c.drawPath(
      Path()
        ..moveTo(t0.dx, t0.dy)
        ..quadraticBezierTo(tk.dx, tk.dy, t1.dx, t1.dy),
      _s(line, 1));
  c.drawPath(
      Path()
        ..moveTo(b0.dx, b0.dy)
        ..quadraticBezierTo(bk.dx, bk.dy, b1.dx, b1.dy),
      _s(line, 1));
  _pl(c, [t1, u(.8, .1), u(.86, .24), u(.97, .31), u(.74, .42)], line, w: 1);
  final ec = u(.82, .29);
  _ring(c, ec, sc * .045, line, 1);
  _dot(c, ec + Offset(math.cos(eye), math.sin(eye)) * sc * .02, 1.1, line);
  if (tongue > 0) {
    final tip = u(.97 + tongue * .4, .34);
    _ln(c, u(.96, .33), tip, _rd, 1);
    _ring(c, tip, 2, _rd, 1);
  }
  final tail = <Offset>[];
  for (var i = 0; i <= 40; i++) {
    final th = i / 40 * math.pi * 3.2, r = .13 * (1 - i / 40 * .8);
    tail.add(u(.17 + math.cos(th - .5) * r, .5 + math.sin(th - .5) * r * .9));
  }
  _pl(c, [t0, ...tail], line, w: 1);
  _ln(c, b0, tail[7], line, 1);
  _pl(c, [u(.4, .52), u(.36, .63), u(.42, .64)], line, w: 1);
  _pl(c, [u(.64, .48), u(.68, .62), u(.74, .63)], line, w: 1);
  _ln(c, u(.02, .64), u(.98, .64), _dg, 1.4);
}

/// Parrot facing right; feathers graded between [h0] and [h1] (hues). [o] top-left, [sc] unit scale.
void _parrot(Canvas c, Offset o, double sc, Color line, double h0, double h1, double sat, double sway) {
  Offset u(double x, double y) => o + Offset(x * sc, y * sc);
  Color g(double v) => _hc(h0 + (h1 - h0) * v, sat);
  for (var f = 0; f < 5; f++) {
    final a = u(.46 + f * .02, .6), b = u(.24 + f * .06 + sway * .03, 1.02 - f * .015);
    for (var k = 0; k < 6; k++) {
      _ln(c, Offset.lerp(a, b, k / 6)!, Offset.lerp(a, b, (k + 1) / 6)!, g(k / 5), 1.2);
    }
  }
  _ring(c, u(.56, .18), sc * .1, line, 1);
  _pl(c, [u(.64, .13), u(.74, .17), u(.71, .27), u(.65, .23)], line, w: 1);
  _ring(c, u(.57, .16), sc * .022, line, 1);
  c.drawPath(
      Path()
        ..moveTo(u(.47, .24).dx, u(.47, .24).dy)
        ..quadraticBezierTo(u(.38, .45).dx, u(.38, .45).dy, u(.46, .62).dx, u(.46, .62).dy)
        ..lineTo(u(.58, .62).dx, u(.58, .62).dy)
        ..quadraticBezierTo(u(.68, .44).dx, u(.68, .44).dy, u(.63, .25).dx, u(.63, .25).dy),
      _s(line, 1));
  for (var k = 0; k < 4; k++) {
    final v = k / 3;
    c.drawPath(
        Path()
          ..moveTo(u(.5, .32 + k * .06).dx, u(.5, .32 + k * .06).dy)
          ..quadraticBezierTo(u(.6, .36 + k * .06).dx, u(.6, .36 + k * .06).dy, u(.55, .46 + k * .05).dx,
              u(.55, .46 + k * .05).dy),
        _s(g(v), 1.2));
  }
  _ln(c, u(.48, .62), u(.47, .67), line, 1);
  _ln(c, u(.56, .62), u(.57, .67), line, 1);
}

/// Boxer facing right; [ext] 0..1 punch reach. Gloves stroked in [glove].
void _boxer(Canvas c, Offset p, double r, Color line, Color glove, double ext, double bob) {
  final hd = p + Offset(0, bob);
  _ring(c, hd, r, line, 1);
  _ln(c, hd + Offset(-r * .9, -r * .2), hd + Offset(r * .9, -r * .35), line, 1);
  _dot(c, hd + Offset(r * .45, -r * .05), .9, line);
  final sh = p + Offset(-r * .2, r * 1.6), hip = p + Offset(-r * .5, r * 4.2);
  _pl(c, [sh + Offset(-r * 1.1, 0), sh + Offset(r * 1.1, 0), hip + Offset(r * .8, 0), hip + Offset(-r * .8, 0)], line,
      w: 1, close: true);
  _pl(c, [hip + Offset(-r * .5, 0), hip + Offset(-r * 1.3, r * 2.4), hip + Offset(-r * .7, r * 2.5)], line, w: 1);
  _pl(c, [hip + Offset(r * .5, 0), hip + Offset(r * 1.3, r * 2.4), hip + Offset(r * 1.9, r * 2.5)], line, w: 1);
  final guard = sh + Offset(r * 1.4, -r * 1.2);
  final elbow = sh + Offset(r * 1.2, r * .9);
  _pl(c, [sh + Offset(r * .9, 0), elbow, guard], line, w: 1);
  _ring(c, guard, r * .75, glove, 1.2);
  final reach = sh + Offset(r * (1.6 + ext * 4.2), -r * .5 * (1 - ext));
  _pl(c, [sh + Offset(-r * .7, 0), sh + Offset(r * (.4 + ext * 1.8), r * (1 - ext)), reach], line, w: 1);
  _ring(c, reach, r * .85, glove, 1.2);
  _ln(c, reach + Offset(-r * .3, -r * .4), reach + Offset(-r * .3, r * .4), glove, 1);
}

/// Stick walker with feet at [p].
void _walker(Canvas c, Offset p, double h, Color col, double ph) {
  final hip = p + Offset(0, -h * .45), neck = p + Offset(0, -h * .85);
  final sw = math.sin(ph) * h * .22;
  _ln(c, hip, p + Offset(sw, 0), col, 1);
  _ln(c, hip, p + Offset(-sw, 0), col, 1);
  _ln(c, hip, neck, col, 1);
  _ln(c, neck + Offset(0, h * .08), neck + Offset(-sw * .8, h * .38), col, 1);
  _ln(c, neck + Offset(0, h * .08), neck + Offset(sw * .8, h * .38), col, 1);
  _ring(c, neck + Offset(0, -h * .12), h * .12, col, 1);
}
