// World A 1 and 2: Falloff (a pond: the effect fades with distance like ripples) and Direction / Attract (grass leaning in the wind and toward a point).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_a_kit.dart';

// ---- 1. Falloff: ripples on a pond -------------------------------------------------------------------------------------------------------

const falloffSpecs = [
  WaSpec('fo.r', 0, 100, 50, unit: '%', digits: 0),
  WaSpec('fo.curve', .4, 4, 1.6),
  WaSpec('fo.s', 0, 100, 100, unit: '%', digits: 0),
  WaSpec('fo.cx', 0, 100, 50, digits: 0),
  WaSpec('fo.cy', 0, 100, 50, digits: 0),
];

class FalloffWorld extends WaWorld {
  FalloffWorld(super.doc);

  /// The pond is seen at a slant: every ripple is squashed to this height ratio, so it reads as water, not as a dial.
  static const _k = .72;
  static Offset _pp(Offset c, double r, double th) => Offset(c.dx + r * math.cos(th), c.dy + r * _k * math.sin(th));
  static double _dd(Offset p, Offset c) => Offset(p.dx - c.dx, (p.dy - c.dy) / _k).distance;

  @override
  List<List<String>> get zoneIds => const [['fo.r'], ['fo.curve'], ['fo.s'], ['fo.cx', 'fo.cy']];

  double _u(Size s) => math.min(art(s).width, art(s).height) * 1.1;
  Offset _c(Size s, WaCtx x) {
    final a = art(s);
    return Offset(a.left + x.v('fo.cx') / 100 * a.width, a.top + x.v('fo.cy') / 100 * a.height);
  }

  double _r(Size s, WaCtx x) => x.v('fo.r') / 100 * _u(s);
  double _rm(Size s, WaCtx x) => _r(s, x) * math.pow(.5, x.v('fo.curve'));
  double _lift(Size s, WaCtx x) => 4 + x.v('fo.s') / 100 * art(s).height * .26;
  Offset _drop(Size s, WaCtx x) => _c(s, x) - Offset(0, _lift(s, x));

  @override
  int? zoneAt(Offset p, Size s, WaCtx x) {
    final c = _c(s, x), d = _dd(p, c);
    final z = waPick([
      (2, (p - _drop(s, x)).distance, 14),
      (0, (d - _r(s, x)).abs(), 13),
      (1, (d - _rm(s, x)).abs(), 13),
    ]);
    return z ?? (art(s).inflate(8).contains(p) ? 3 : null);
  }

  @override
  Offset? anchor(int z, Offset ptr, Size s, WaCtx x) {
    final c = _c(s, x), th = math.atan2((ptr.dy - c.dy) / _k, ptr.dx - c.dx);
    return switch (z) { 0 => _pp(c, _r(s, x), th), 1 => _pp(c, _rm(s, x), th), 2 => _drop(s, x), _ => c };
  }

  @override
  void drag(int z, Offset vp, Size s, WaCtx x) {
    final a = art(s), c = _c(s, x);
    switch (z) {
      case 0:
        set('fo.r', _dd(vp, c) / _u(s) * 100);
      case 1:
        final r = _r(s, x);
        if (r < 4) return;
        final q = waClamp(_dd(vp, c) / r, .02, .985);
        set('fo.curve', math.log(q) / math.log(.5));
      case 2:
        set('fo.s', (c.dy - vp.dy - 4) / (a.height * .26) * 100);
      default:
        set('fo.cx', (vp.dx - a.left) / a.width * 100);
        set('fo.cy', (vp.dy - a.top) / a.height * 100);
    }
  }

  @override
  List<String> readout(WaCtx x) => [x.doc.spec('fo.r').fmt(x.v('fo.r')), '×${x.doc.spec('fo.curve').fmt(x.v('fo.curve'))}', x.doc.spec('fo.s').fmt(x.v('fo.s')), '${x.v('fo.cx').round()},${x.v('fo.cy').round()}'];

  @override
  void paint(Canvas cv, Size s, WaCtx x) {
    final c = _c(s, x), r = _r(s, x), str = x.v('fo.s') / 100, curve = x.v('fo.curve');
    const n = 6;
    final phase = (x.t / 4) % 1.0; // the ripples leave the centre once every 4 s
    for (var k = 0; k < n; k++) {
      final u = (k + phase) / n;
      final rr = r * math.pow(u, curve);
      if (rr < 1.5) continue;
      final a = str * math.pow(1 - u, 1.25) * waSmooth(u * 12) * .95;
      final p = Path();
      const seg = 56;
      for (var i = 0; i <= seg; i++) {
        final th = i / seg * math.pi * 2;
        final w = 1 + .014 * waNoise(th * 3 + x.t * .55, k + 1.0 + (k + phase).floorToDouble());
        final pt = _pp(c, rr * w, th);
        i == 0 ? p.moveTo(pt.dx, pt.dy) : p.lineTo(pt.dx, pt.dy);
      }
      cv.drawPath(p, waStroke(N.g76.withValues(alpha: waClamp(a, 0, 1))));
    }
    // A: three short ticks on the outer edge where the effect has faded to nothing
    final rimA = x.mark(0, 1);
    for (var i = 0; i < 3; i++) {
      final th = -.55 + i * math.pi * 2 / 3;
      cv.drawLine(_pp(c, r - 4.5, th), _pp(c, r + 4.5, th), rimA);
    }
    // B: three short arcs on the middle ripple (where the curve bends the spacing)
    final rm = _rm(s, x);
    if (rm > 5) {
      final pb = x.mark(1, 1.4);
      for (var i = 0; i < 3; i++) {
        final th = .5 + i * math.pi * 2 / 3;
        final arc = Path();
        for (var j = 0; j <= 6; j++) {
          final q = _pp(c, rm, th - .15 + .3 * j / 6);
          j == 0 ? arc.moveTo(q.dx, q.dy) : arc.lineTo(q.dx, q.dy);
        }
        cv.drawPath(arc, pb);
      }
    }
    // D: a tiny orange ring where the first stone fell; the whole water is its handle
    cv.drawOval(Rect.fromCenter(center: c, width: 11, height: 11 * _k), x.mark(3, 1));
    // C: the drop (strength = how high it falls from); a teardrop, never a handle
    final d = _drop(s, x) + Offset(0, math.sin(x.t * 1.3) * .8);
    final drop = Path()
      ..moveTo(d.dx, d.dy - 4.5)
      ..quadraticBezierTo(d.dx + 1.2, d.dy - 1.2, d.dx + 2.6, d.dy + .9)
      ..arcToPoint(Offset(d.dx - 2.6, d.dy + .9), radius: const Radius.circular(2.7), clockwise: true)
      ..quadraticBezierTo(d.dx - 1.2, d.dy - 1.2, d.dx, d.dy - 4.5)
      ..close();
    cv.drawPath(drop, waFill(x.col(2).withValues(alpha: x.hl(2) ? 1 : .9)));
    if (x.hl(2)) cv.drawCircle(d, 8, waStroke(waHot.withValues(alpha: .5)));
  }
}

// ---- 2. Direction / Attract: grass leaning in the wind and toward a point -------------------------------------------------------------------

const directionSpecs = [
  WaSpec('di.a', 0, 360, 0, unit: '°', digits: 0),
  WaSpec('di.pull', 0, 1, .55),
  WaSpec('di.s', 0, 100, 80, unit: '%', digits: 0),
  WaSpec('di.tx', 0, 100, 64, digits: 0),
  WaSpec('di.ty', 0, 100, 36, digits: 0),
];

class _Blade {
  _Blade(this.root, this.h);
  final Offset root;
  final int h;
  Offset tip = Offset.zero, dir = const Offset(1, 0);
}

class DirectionWorld extends WaWorld {
  DirectionWorld(super.doc);

  Size? _bs;
  List<_Blade> _blades = [];
  double _lmax = 20;
  Offset _gRoot = Offset.zero, _gDir = const Offset(1, 0);

  @override
  List<List<String>> get zoneIds => const [['di.a'], ['di.pull'], ['di.s'], ['di.tx', 'di.ty']];

  double _short(Size s) => math.min(art(s).width, art(s).height);
  Offset _t(Size s, WaCtx x) {
    final a = art(s);
    return Offset(a.left + x.v('di.tx') / 100 * a.width, a.top + x.v('di.ty') / 100 * a.height);
  }

  double _rp(Size s, WaCtx x) => 10 + x.v('di.pull') * _short(s) * .5;

  Offset _pa(Size s, WaCtx x) {
    final a = art(s), th = x.v('di.a') / 180 * math.pi;
    return a.center + Offset(math.cos(th + math.pi), math.sin(th + math.pi)) * (_short(s) * .42);
  }

  void _grow(Size s) {
    if (_bs == s) return;
    _bs = s;
    final a = art(s);
    final cols = math.max(4, (a.width / 30).round()), rows = math.max(2, (a.height / 26).round());
    final cw = a.width / cols, rh = a.height / rows;
    _lmax = math.min(cw, rh) * 1.1;
    _blades = [
      for (var j = 0; j < rows; j++)
        for (var i = 0; i < cols; i++)
          _Blade(Offset(a.left + (i + .5 + (waRnd(j * 31 + i) - .5) * .5) * cw, a.top + (j + .5 + (waRnd(j * 17 + i * 5 + 3) - .5) * .5) * rh), j * 31 + i),
    ];
  }

  /// Place every blade for the shown values.
  void _lay(Size s, WaCtx x) {
    _grow(s);
    final th = x.v('di.a') / 180 * math.pi, wv = Offset(math.cos(th), math.sin(th));
    final t = _t(s, x), pull = x.v('di.pull'), len = _lmax * x.v('di.s') / 100, u = _short(s);
    for (final b in _blades) {
      final d = t - b.root, dist = d.distance;
      final infl = pull * math.exp(-dist / (u * .9));
      var dir = waUnit(wv * (1 - infl) + waUnit(d) * infl);
      final gust = math.sin(b.root.dx * wv.dx / 55 + b.root.dy * wv.dy / 55 - x.t * 1.4 + b.h * .05);
      final ang = .09 * gust + .05 * math.sin(x.t * 1.1 + b.h);
      dir = Offset(dir.dx * math.cos(ang) - dir.dy * math.sin(ang), dir.dx * math.sin(ang) + dir.dy * math.cos(ang));
      b.dir = dir;
      b.tip = b.root + dir * len;
    }
  }

  @override
  int? zoneAt(Offset p, Size s, WaCtx x) {
    _lay(s, x);
    final t = _t(s, x), d = (p - t).distance;
    var bt = 99.0;
    for (final b in _blades) {
      bt = math.min(bt, (p - b.tip).distance);
    }
    return waPick([
      (3, d, 11),
      (1, (d - _rp(s, x)).abs(), 12),
      (2, bt, 12),
      (0, (p - _pa(s, x)).distance, 17),
    ]);
  }

  @override
  Offset? anchor(int z, Offset ptr, Size s, WaCtx x) {
    _lay(s, x);
    final t = _t(s, x);
    switch (z) {
      case 0:
        return _pa(s, x);
      case 1:
        return t + waUnit(ptr - t) * _rp(s, x);
      case 2:
        _Blade? best;
        for (final b in _blades) {
          if (best == null || (ptr - b.tip).distance < (ptr - best.tip).distance) best = b;
        }
        _gRoot = best!.root;
        _gDir = best.dir;
        return best.tip;
      default:
        return t;
    }
  }

  @override
  void drag(int z, Offset vp, Size s, WaCtx x) {
    final a = art(s);
    switch (z) {
      case 0:
        final v = a.center - vp;
        set('di.a', (math.atan2(v.dy, v.dx) * 180 / math.pi + 360) % 360);
      case 1:
        set('di.pull', ((vp - _t(s, x)).distance - 10) / (_short(s) * .5));
      case 2:
        final l = (vp - _gRoot).dx * _gDir.dx + (vp - _gRoot).dy * _gDir.dy;
        set('di.s', l / _lmax * 100);
      default:
        set('di.tx', (vp.dx - a.left) / a.width * 100);
        set('di.ty', (vp.dy - a.top) / a.height * 100);
    }
  }

  @override
  List<String> readout(WaCtx x) => ['${x.v('di.a').round()}°', x.doc.spec('di.pull').fmt(x.v('di.pull')), '${x.v('di.s').round()}', '${x.v('di.tx').round()},${x.v('di.ty').round()}'];

  @override
  void paint(Canvas cv, Size s, WaCtx x) {
    _lay(s, x);
    final stroke = waStroke(N.g56.withValues(alpha: x.grab == null ? .9 : .7));
    final tipPaint = waFill(x.col(2).withValues(alpha: x.hl(2) ? 1 : .88));
    for (final b in _blades) {
      final n = Offset(-b.dir.dy, b.dir.dx) * (_lmax * .1 * (b.h.isEven ? 1 : -1));
      final p = Path()
        ..moveTo(b.root.dx, b.root.dy)
        ..quadraticBezierTo((b.root.dx + b.tip.dx) / 2 + n.dx, (b.root.dy + b.tip.dy) / 2 + n.dy, b.tip.dx, b.tip.dy);
      cv.drawPath(p, stroke);
      cv.drawCircle(b.root, 1, waFill(N.g38));
      cv.drawCircle(b.tip, x.hl(2) ? 2.1 : 1.6, tipPaint);
    }
    // A: three short breeze streaks upwind, drifting downwind
    final th = x.v('di.a') / 180 * math.pi, wv = Offset(math.cos(th), math.sin(th)), nv = Offset(-wv.dy, wv.dx);
    final pa = _pa(s, x), drift = ((x.t * .45) % 1.0) * 8 - 4, pw = x.mark(0, 1);
    final gust = Path()
      ..moveTo(pa.dx - wv.dx * 9 - nv.dx * 3, pa.dy - wv.dy * 9 - nv.dy * 3)
      ..lineTo(pa.dx + wv.dx * (6 + drift) - nv.dx * 3, pa.dy + wv.dy * (6 + drift) - nv.dy * 3)
      ..quadraticBezierTo(pa.dx + wv.dx * (12 + drift) - nv.dx * 3 + nv.dx * 1, pa.dy + wv.dy * (12 + drift) - nv.dy * 3 + nv.dy * 1, pa.dx + wv.dx * (9 + drift) + nv.dx * 2, pa.dy + wv.dy * (9 + drift) + nv.dy * 2);
    cv.drawPath(gust, pw);
    cv.drawLine(pa - wv * 4 + nv * 5 + wv * drift, pa + wv * 2 + nv * 5 + wv * drift, pw);
    // B: four green arcs round the target, their radius is the pull
    final t = _t(s, x), rp = _rp(s, x), pb = x.mark(1, 1.3), spin = x.t * .25;
    for (var i = 0; i < 4; i++) {
      final a0 = spin + i * math.pi / 2;
      waArc(cv, t, rp, a0, a0 + .5, pb);
    }
    // D: the target, a small orange seed
    cv.drawCircle(t, x.hl(3) ? 3.4 : 2.8, waFill(x.col(3)));
    cv.drawCircle(t, 6, waStroke(x.col(3).withValues(alpha: .5)));
  }
}
