// Phenomenon D 1 and 2: Easing (a ball travels, you reshape its journey) and Spring (a ball in a vial of oil, you flick it).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'pheno_d_kit.dart';

// ---- 1. Easing: the 4 numbers of a cubic-bezier are the shape of a journey -----------------------------------------------------------

const easeSpecs = [
  PhSpec('easing.x1', 0, 1, .25),
  PhSpec('easing.y1', -.5, 1.5, .1),
  PhSpec('easing.x2', 0, 1, .25),
  PhSpec('easing.y2', -.5, 1.5, 1),
];

double _bz(double a, double b, double u) => 3 * a * u * (1 - u) * (1 - u) + 3 * b * u * u * (1 - u) + u * u * u;

/// Progress (0..1, may overshoot) at time fraction [t] of a CSS-style cubic-bezier(x1, y1, x2, y2).
double easeAt(double t, double x1, double y1, double x2, double y2) {
  var lo = 0.0, hi = 1.0;
  for (var i = 0; i < 22; i++) {
    final u = (lo + hi) / 2;
    if (_bz(x1, x2, u) < t) {
      lo = u;
    } else {
      hi = u;
    }
  }
  return _bz(y1, y2, (lo + hi) / 2);
}

class EaseMini extends PhMini {
  EaseMini(super.doc);
  static const _cycle = 1.9, _run = 1.1;
  double _clock = .6; // seconds into the cycle; only moves while the pointer is on it

  @override
  String get cap => 'ease';

  @override
  List<PhR> readout() => [for (final s in easeSpecs) (doc.show(s.id), doc.changed(s.id))];

  Rect _plot(Size s) {
    final a = art(s);
    return Rect.fromLTWH(a.left, a.top, a.width * .6, a.height);
  }

  Offset _pt(Rect p, double x, double y) => Offset(p.left + 5 + x * (p.width - 10), p.bottom - 4 - (y + .5) / 2 * (p.height - 8));

  Offset _h(Rect p, int i) => _pt(p, doc['easing.x${i + 1}'], doc['easing.y${i + 1}']);

  double _prog(double t) => easeAt(t, doc['easing.x1'], doc['easing.y1'], doc['easing.x2'], doc['easing.y2']);

  @override
  int? zoneAt(Offset p, Size s) {
    final pl = _plot(s);
    final d0 = (p - _h(pl, 0)).distance, d1 = (p - _h(pl, 1)).distance;
    if (d0 <= 13 && d0 <= d1) return 0;
    if (d1 <= 13) return 1;
    for (var i = 0; i <= 30; i++) {
      final t = i / 30;
      if ((p - _pt(pl, t, _prog(t))).distance <= 9) return 2;
    }
    return null;
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final pl = _plot(s), k = fine ? .1 : 1.0;
    final dx = d.dx * k / (pl.width - 10), dy = -d.dy * k * 2 / (pl.height - 8);
    if (z == 0 || z == 2) {
      doc.set('easing.x1', doc['easing.x1'] + dx);
      doc.set('easing.y1', doc['easing.y1'] + dy);
    }
    if (z == 1 || z == 2) {
      doc.set('easing.x2', doc['easing.x2'] + dx);
      doc.set('easing.y2', doc['easing.y2'] + dy);
    }
  }

  @override
  void reset(int? z) => switch (z) {
        0 => doc.resetIds(['easing.x1', 'easing.y1']),
        1 => doc.resetIds(['easing.x2', 'easing.y2']),
        _ => super.reset(z),
      };

  @override
  bool step(double dt, Size s) {
    if (view.hover || view.grab != null) _clock = (_clock + dt) % _cycle;
    return false;
  }

  @override
  void paint(Canvas c, Size s) {
    final pl = _plot(s), a = art(s);
    final t = phClamp(_clock / _run, 0, 1), p = _prog(t);
    final hot = view.grab ?? view.hot;
    // the journey: start and end rails, the straight diagonal as the ghost of "linear"
    final p0 = _pt(pl, 0, 0), p3 = _pt(pl, 1, 1), c1 = _h(pl, 0), c2 = _h(pl, 1);
    final rail = phStroke(N.g20);
    c.drawLine(Offset(pl.left, p0.dy), Offset(pl.right, p0.dy), rail);
    c.drawLine(Offset(pl.left, p3.dy), Offset(pl.right, p3.dy), rail);
    phDash(c, p0, p3, phStroke(N.g26), on: 2, off: 3);
    final tan = phStroke(N.g38);
    c.drawLine(p0, c1, tan);
    c.drawLine(p3, c2, tan);
    final curve = Path()
      ..moveTo(p0.dx, p0.dy)
      ..cubicTo(c1.dx, c1.dy, c2.dx, c2.dy, p3.dx, p3.dy);
    c.drawPath(curve, phStroke(hot == 2 ? phHot : N.g95, hot == 2 ? 2 : 1.5));
    // the clock rides the curve: a hairline playhead and the dot
    final dot = _pt(pl, t, p);
    c.drawLine(Offset(dot.dx, p0.dy), Offset(dot.dx, p3.dy), phStroke(N.g26.withValues(alpha: .7)));
    c.drawCircle(dot, 3, phFill(N.g95));
    // the handles
    for (var i = 0; i < 2; i++) {
      final h = i == 0 ? c1 : c2;
      c.drawCircle(h, 2.6, phFill(N.g76));
      if (hot == i) {
        c.drawCircle(h, view.grab == i ? 6.5 : 5, phStroke(phHot));
        if (view.grab == i) c.drawCircle(h, 6.5, phFill(phHot.withValues(alpha: .18)));
      }
    }
    // the ball: the same eased progress travelling an arch; equal-time beads show the spacing (bunched = slow there)
    final z = Rect.fromLTRB(pl.right + 16, a.top, a.right, a.bottom);
    final w = math.max(z.width - 8, 20.0), sag = math.min(z.height * .3, w * .2);
    final r = (w * w / 4 + sag * sag) / (2 * sag), apex = z.center.dy - sag / 2 - 2;
    final ctr = Offset(z.center.dx, apex + r), al = math.asin(math.min(1.0, w / 2 / r));
    final a0 = -math.pi / 2 - al, sweep = 2 * al;
    Offset on(double q) => ctr + Offset(math.cos(a0 + sweep * q), math.sin(a0 + sweep * q)) * r;
    c.drawArc(Rect.fromCircle(center: ctr, radius: r), a0, sweep, false, phStroke(N.g20));
    for (var k = 0; k <= 8; k++) {
      c.drawCircle(on(_prog(k / 8)), 1, phFill(N.g56.withValues(alpha: .8)));
    }
    final moving = t > 0 && t < 1;
    for (var k = 3; k >= 1 && moving; k--) {
      c.drawCircle(on(_prog(phClamp(t - k * .035, 0, 1))), 3.2 - k * .5, phFill(phAcc(Fam.stagger).withValues(alpha: .24 - k * .05)));
    }
    c.drawCircle(on(p), 4, phFill(phAcc(Fam.stagger)));
  }
}

// ---- 2. Spring: stiffness and damping are a ball in a vial of oil ------------------------------------------------------------------

const springSpecs = [
  PhSpec('spring.stiffness', 40, 600, 220, digits: 0),
  PhSpec('spring.damping', .5, 60, 14, digits: 1),
];

class SpringMini extends PhMini {
  SpringMini(super.doc);
  double x = 0, v = 0; // ball offset from rest (px, down is positive) and velocity
  final List<double> _hist = []; // samples since the last release, 1/240 s apart... see _hStep
  bool _rec = false;
  double _acc = 0, _vPtr = 0, _lastY = 0, _t = 0;
  final _sw = Stopwatch();
  static const _hStep = 1 / 60, _span = 1.6;

  double get _k => doc['spring.stiffness'];
  double get _c => doc['spring.damping'];

  @override
  String get cap => 'spring';

  @override
  List<PhR> readout() => [
        (doc.show('spring.stiffness'), doc.changed('spring.stiffness')),
        (doc.show('spring.damping'), doc.changed('spring.damping')),
        ('z ${(_c / (2 * math.sqrt(_k))).toStringAsFixed(2).replaceFirst('0.', '.')}', false),
      ];

  ({Rect a, double xc, double y0, double r, double maxD, Rect tr}) _g(Size s) {
    final a = art(s), r = phClamp(a.height * .09, 5.5, 9.0);
    final xc = a.left + 32;
    return (a: a, xc: xc, y0: a.top + a.height * .52, r: r, maxD: a.height * .4 - r, tr: Rect.fromLTRB(xc + 42, a.top, a.right, a.bottom));
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final g = _g(s), ball = Offset(g.xc, g.y0 + x);
    if ((p - ball).distance <= g.r + 9) return 0;
    final dx = (p.dx - g.xc).abs();
    if (p.dy < g.a.top - 4 || p.dy > g.a.bottom + 4) return null;
    if (dx <= 12 && p.dy < ball.dy - g.r) return 1; // the coil
    if (dx > 12 && dx <= 32) return 2; // the oil
    return null;
  }

  @override
  void down(int z, Offset p, Size s) {
    if (z != 0) return;
    _rec = false;
    _hist.clear();
    _lastY = p.dy;
    _vPtr = 0;
    _sw..reset()..start();
    v = 0;
    x = phClamp(p.dy - _g(s).y0, -_g(s).maxD, _g(s).maxD);
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final g = _g(s), k = fine ? .1 : 1.0;
    if (z == 0) {
      x = phClamp(p.dy - g.y0, -g.maxD, g.maxD);
      final dt = math.max(_sw.elapsedMicroseconds / 1e6, .002);
      _sw..reset()..start();
      _vPtr = phLerp(_vPtr, (p.dy - _lastY) / dt, .5);
      _lastY = p.dy;
    } else if (z == 1) {
      doc.set('spring.stiffness', math.exp(math.log(_k) - d.dy * k / g.a.height * 2.7));
    } else {
      doc.set('spring.damping', math.exp(math.log(_c) - d.dy * k / g.a.height * 4.8));
    }
  }

  @override
  void up(int z) {
    if (z != 0) return;
    final stale = _sw.elapsedMilliseconds > 90;
    v = phClamp(stale ? 0 : _vPtr, -1100, 1100);
    _hist
      ..clear()
      ..add(x);
    _acc = 0;
    _rec = true;
  }

  @override
  void reset(int? z) => switch (z) {
        1 => doc.resetIds(['spring.stiffness']),
        2 => doc.resetIds(['spring.damping']),
        _ => doc.resetIds(['spring.stiffness', 'spring.damping']),
      };

  @override
  bool step(double dt, Size s) {
    _t += dt; // the oil bubbles and the ball's idle bob keep running
    if (view.grab == 0) return true;
    final n = math.max(1, (dt / .004).ceil()), h = dt / n;
    for (var i = 0; i < n; i++) {
      v += (-_k * x - _c * v) * h;
      x += v * h;
    }
    if (_rec) {
      _acc += dt;
      while (_acc >= _hStep && _hist.length < _span / _hStep) {
        _acc -= _hStep;
        _hist.add(x);
      }
    }
    if (x.abs() <= .05 && v.abs() <= .5) {
      x = 0;
      v = 0;
      _rec = false;
    }
    return true;
  }

  List<double> _ring(double d0) {
    // the response a flick of d0 px would make with today's numbers
    final out = <double>[];
    var px = d0, pv = 0.0;
    const h = 1 / 240;
    for (var i = 0; i <= _span / h; i++) {
      if (i % 4 == 0) out.add(px);
      pv += (-_k * px - _c * pv) * h;
      px += pv * h;
    }
    return out;
  }

  @override
  void paint(Canvas cv, Size s) {
    final g = _g(s), hot = view.grab ?? view.hot, ball = Offset(g.xc, g.y0 + x + (x == 0 && v == 0 ? math.sin(_t * 2.2) * .9 : 0));
    final top = g.a.top, bot = g.a.bottom;
    // the vial: a hairline tube; the oil is a few tiny bubbles drifting up (more of them = more damping)
    final tube = RRect.fromLTRBR(g.xc - 32, top - 3, g.xc + 32, bot + 3, const Radius.circular(10));
    cv.drawRRect(tube, phStroke(hot == 2 ? phHot : N.g26));
    final nc = ((math.log(_c) - math.log(.5)) / (math.log(60) - math.log(.5))).clamp(0.0, 1.0);
    final nb = 3 + (nc * 9).round(), bub = N.g44.withValues(alpha: hot == 2 ? .95 : .6);
    for (var i = 0; i < nb; i++) {
      final hx = (math.sin(i * 12.9898) * 43758.5453).abs() % 1, side = i.isEven ? -1 : 1;
      final bx = g.xc + side * (16 + hx * 11) + math.sin(_t * 1.3 + i) * 1.2;
      final fy = ((i * .618 + _t * (.05 + .05 * hx)) % 1), by = bot - 4 - fy * (bot - top - 8);
      cv.drawCircle(Offset(bx, by), .8 + hx * .7, phStroke(bub));
    }
    // rest line and the ring: the faint curve is today's numbers, the bright one is what the ball just did
    phDash(cv, Offset(g.xc + 33, g.y0), Offset(g.tr.right, g.y0), phStroke(N.g20));
    final ring = _ring(g.maxD * .7);
    final dxT = g.tr.width / (ring.length - 1);
    final fp = Path();
    for (var i = 0; i < ring.length; i++) {
      final q = Offset(g.tr.left + i * dxT, g.y0 + ring[i]);
      i == 0 ? fp.moveTo(q.dx, q.dy) : fp.lineTo(q.dx, q.dy);
    }
    cv.drawPath(fp, phStroke(N.g38));
    if (_hist.length > 1) {
      final lp = Path();
      for (var i = 0; i < _hist.length; i++) {
        final q = Offset(g.tr.left + i * (g.tr.width / (_span / _hStep)), g.y0 + _hist[i]);
        i == 0 ? lp.moveTo(q.dx, q.dy) : lp.lineTo(q.dx, q.dy);
      }
      cv.drawPath(lp, phStroke(N.g95, 1.25));
    }
    // the coil: stiffer = more, finer turns
    final nk = ((math.log(_k) - math.log(40)) / (math.log(600) - math.log(40))).clamp(0.0, 1.0);
    final turns = 5 + (nk * 8).round(), amp = phLerp(9, 4.5, nk);
    final y1 = ball.dy - g.r - 1, coil = Path()..moveTo(g.xc, top);
    cv.drawLine(Offset(g.xc - 9, top), Offset(g.xc + 9, top), phStroke(N.g56));
    final y2 = top + 3, y3 = y1 - 3, seg = (y3 - y2) / (turns * 2);
    coil.lineTo(g.xc, y2);
    for (var i = 0; i < turns * 2; i++) {
      coil.lineTo(g.xc + (i.isEven ? amp : -amp), y2 + seg * (i + .5));
    }
    coil.lineTo(g.xc, y3);
    coil.lineTo(g.xc, y1);
    cv.drawPath(coil, phStroke(hot == 1 ? phHot : N.g76, hot == 1 ? 1.5 : 1));
    // the ball
    cv.drawCircle(ball, g.r, phFill(phAcc(Fam.follow)));
    if (hot == 0) cv.drawCircle(ball, g.r + (view.grab == 0 ? 5 : 4), phStroke(phHot));
  }
}
