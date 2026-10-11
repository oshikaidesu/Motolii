// World A 5 and 6: Along Path (a string of beads between two pennants) and Scale distribution (a stream of soap bubbles swelling from small to large).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_a_kit.dart';

// ---- 5. Along Path: a necklace between two pennants ------------------------------------------------------------------------------------------

const alongSpecs = [
  WaSpec('ap.start', 0, 100, 0, unit: '%', digits: 0),
  WaSpec('ap.end', 0, 100, 100, unit: '%', digits: 0),
  WaSpec('ap.sp', 0, 1, .5),
  WaSpec('ap.bias', .4, 2.5, 1),
];

class AlongWorld extends WaWorld {
  AlongWorld(super.doc);

  Size? _ps;
  final List<Offset> _pts = [];
  final List<double> _cum = [];

  @override
  List<List<String>> get zoneIds => const [['ap.start'], ['ap.end'], ['ap.sp'], ['ap.bias']];

  static int _count(double sp) => (18 - 14 * sp).round().clamp(4, 18);

  void _path(Size s) {
    if (_ps == s) return;
    _ps = s;
    final a = art(s).deflate(6);
    final p0 = Offset(a.left, a.top + a.height * .78), p1 = Offset(a.left + a.width * .3, a.top - a.height * .22);
    final p2 = Offset(a.left + a.width * .64, a.bottom + a.height * .22), p3 = Offset(a.right, a.top + a.height * .22);
    _pts.clear();
    _cum.clear();
    const m = 200;
    var acc = 0.0;
    for (var i = 0; i <= m; i++) {
      final t = i / m, u = 1 - t;
      final q = p0 * (u * u * u) + p1 * (3 * u * u * t) + p2 * (3 * u * t * t) + p3 * (t * t * t);
      if (i > 0) acc += (q - _pts.last).distance;
      _pts.add(q);
      _cum.add(acc);
    }
    for (var i = 0; i <= m; i++) {
      _cum[i] /= acc;
    }
  }

  /// A point at arc fraction [f] (0..1) and its unit tangent.
  (Offset, Offset) _at(double f) {
    final ff = waClamp(f, 0, 1);
    var lo = 0, hi = _cum.length - 1;
    while (hi - lo > 1) {
      final mid = (lo + hi) >> 1;
      if (_cum[mid] < ff) {
        lo = mid;
      } else {
        hi = mid;
      }
    }
    final span = _cum[hi] - _cum[lo], k = span < 1e-9 ? 0.0 : (ff - _cum[lo]) / span;
    return (Offset.lerp(_pts[lo], _pts[hi], k)!, waUnit(_pts[hi] - _pts[lo]));
  }

  /// Arc fraction (0..1) of the path point nearest to [p].
  double _near(Offset p) {
    var bi = 0, bd = 1e18;
    for (var i = 0; i < _pts.length; i++) {
      final d = (_pts[i] - p).distanceSquared;
      if (d < bd) {
        bd = d;
        bi = i;
      }
    }
    return _cum[bi];
  }

  Offset _flagTop(double f) {
    final (p, t) = _at(f);
    var n = Offset(-t.dy, t.dx);
    if (n.dy > 0) n = -n;
    return p + n * 13;
  }

  double _sf(WaCtx x, String id) => x.v(id) / 100;
  double _bead(int k, int n, WaCtx x) {
    final st = _sf(x, 'ap.start'), en = _sf(x, 'ap.end');
    final u = math.pow(k / (n - 1), x.v('ap.bias'));
    return st + (en - st) * u + .007 * math.sin(x.t * .9 + k * .4);
  }

  int _mid(int n) => math.max(2, (n - 1) ~/ 2);

  @override
  int? zoneAt(Offset p, Size s, WaCtx x) {
    _path(s);
    final n = _count(x.v('ap.sp'));
    return waPick([
      (0, (p - _flagTop(_sf(x, 'ap.start'))).distance, 14),
      (1, (p - _flagTop(_sf(x, 'ap.end'))).distance, 14),
      (2, (p - _at(_bead(1, n, x)).$1).distance, 13),
      (3, (p - _at(_bead(_mid(n), n, x)).$1).distance, 13),
    ]);
  }

  @override
  Offset? anchor(int z, Offset ptr, Size s, WaCtx x) {
    _path(s);
    final n = _count(x.v('ap.sp'));
    return switch (z) {
      0 => _flagTop(_sf(x, 'ap.start')),
      1 => _flagTop(_sf(x, 'ap.end')),
      2 => _at(_bead(1, n, x)).$1,
      _ => _at(_bead(_mid(n), n, x)).$1,
    };
  }

  @override
  void drag(int z, Offset vp, Size s, WaCtx x) {
    _path(s);
    final f = _near(vp), st = _sf(x, 'ap.start'), en = _sf(x, 'ap.end'), bias = x.v('ap.bias');
    switch (z) {
      case 0:
        set('ap.start', math.min(f, en - .02) * 100);
      case 1:
        set('ap.end', math.max(f, st + .02) * 100);
      case 2:
        final u = waClamp((f - st) / math.max(en - st, .02), .03, .98);
        final n = 1 + math.pow(u, -1 / bias);
        set('ap.sp', (18 - n) / 14);
      default:
        final n = _count(x.v('ap.sp')), k = _mid(n);
        final u = waClamp((f - st) / math.max(en - st, .02), .03, .98);
        set('ap.bias', math.log(u) / math.log(k / (n - 1)));
    }
  }

  @override
  List<String> readout(WaCtx x) => [x.doc.spec('ap.start').fmt(x.v('ap.start')), x.doc.spec('ap.end').fmt(x.v('ap.end')), x.doc.spec('ap.sp').fmt(x.v('ap.sp')), x.doc.spec('ap.bias').fmt(x.v('ap.bias'))];

  void _flag(Canvas cv, int z, double f, int dir, WaCtx x) {
    final (p, t) = _at(f);
    final top = _flagTop(f), nrm = waUnit(top - p);
    final pt = x.mark(z, 1);
    cv.drawLine(p, top, pt);
    final tip = top - nrm * 3 + t * (dir * 9.0);
    cv.drawPath(
        Path()
          ..moveTo(top.dx, top.dy)
          ..lineTo(tip.dx, tip.dy)
          ..lineTo(top.dx - nrm.dx * 6, top.dy - nrm.dy * 6),
        pt);
    if (x.hl(z)) cv.drawCircle(top, 10, waStroke(waHot.withValues(alpha: .35)));
  }

  @override
  void paint(Canvas cv, Size s, WaCtx x) {
    _path(s);
    final st = _sf(x, 'ap.start'), en = _sf(x, 'ap.end'), n = _count(x.v('ap.sp'));
    final path = Path()..moveTo(_pts.first.dx, _pts.first.dy);
    for (final q in _pts.skip(1)) {
      path.lineTo(q.dx, q.dy);
    }
    cv.drawPath(path, waStroke(N.g26));
    final live = Path();
    var first = true;
    for (var i = 0; i < _pts.length; i++) {
      if (_cum[i] < st || _cum[i] > en) continue;
      first ? live.moveTo(_pts[i].dx, _pts[i].dy) : live.lineTo(_pts[i].dx, _pts[i].dy);
      first = false;
    }
    cv.drawPath(live, waStroke(N.g44));
    final k2 = _mid(n);
    for (var k = 0; k < n; k++) {
      final p = _at(_bead(k, n, x)).$1;
      final held = k == 1 ? 2 : (k == k2 ? 3 : -1);
      cv.drawCircle(p, 2.5, waFill(N.g76));
      if (held >= 0) cv.drawCircle(p, x.hl(held) ? 6.5 : 5.2, x.mark(held, 1));
    }
    _flag(cv, 0, st, 1, x);
    _flag(cv, 1, en, -1, x);
  }
}

// ---- 6. Scale distribution: soap bubbles swelling along a stream ----------------------------------------------------------------------------

const scaleSpecs = [
  WaSpec('sd.min', 0, 200, 40, unit: '%', digits: 0),
  WaSpec('sd.max', 0, 200, 100, unit: '%', digits: 0),
  WaSpec('sd.curve', .4, 2.5, 1),
  WaSpec('sd.jit', 0, 1, 0),
];

class ScaleWorld extends WaWorld {
  ScaleWorld(super.doc);

  static const _n = 7;
  double _j0 = 0, _spAbs = 0, _rho0 = 0;

  @override
  List<List<String>> get zoneIds => const [['sd.min'], ['sd.max'], ['sd.curve'], ['sd.jit']];

  /// Sizes above 100 % are drawn soft-limited (100 + 90 (1 - e^(-(p-100)/60))): 200 % is 173 % of the unit, so the biggest bubble still fits.
  static double _show(double pct) => pct <= 100 ? pct : 100 + 90 * (1 - math.exp(-(pct - 100) / 60));
  static double _unshow(double rho) => rho <= 100 ? rho : 100 - 60 * math.log(1 - waClamp((rho - 100) / 90, 0, .99));
  static const _maxShow = 1.8;

  // geometry cached per frame size: unit radius [r100] and the path box inset so the largest bubble stays inside the drawing area
  Size? _gs;
  double _r = 10;
  Rect _bx = Rect.zero;
  void _geo(Size s) {
    if (_gs == s) return;
    _gs = s;
    final a = art(s);
    var r = a.height * .15;
    for (var i = 0; i < 6; i++) {
      final inset = _maxShow * r + 3;
      final b = Rect.fromLTRB(a.left + inset, a.top + inset, a.right - inset, a.bottom - inset);
      _bx = b.width < 8 || b.height < 2 ? Rect.fromCenter(center: a.center, width: math.max(b.width, 8), height: math.max(b.height, 2)) : b;
      final len = (Offset(_bx.width, _bx.height * .76)).distance;
      r = math.min(a.height * .15, len / (_n - 1) * .5);
    }
    _r = r;
  }

  Rect _box(Size s) {
    _geo(s);
    return _bx;
  }

  Offset _p0(Size s) => Offset(_box(s).left, _box(s).bottom - _box(s).height * .12);
  Offset _p1(Size s) => Offset(_box(s).right, _box(s).top + _box(s).height * .12);
  Offset _ctr(int k, Size s) => Offset.lerp(_p0(s), _p1(s), k / (_n - 1))!;
  double _r100(Size s) {
    _geo(s);
    return _r;
  }

  /// Un-jittered size (percent) of bubble [k].
  double _pct(int k, WaCtx x) => x.v('sd.min') + (x.v('sd.max') - x.v('sd.min')) * math.pow(k / (_n - 1), x.v('sd.curve'));
  double _rad(int k, Size s, WaCtx x) => math.max(1.0, _show(_pct(k, x)) / 100 * _r100(s));

  @override
  int? zoneAt(Offset p, Size s, WaCtx x) {
    double th(int k) => math.max(_rad(k, s, x) + 8, 14);
    final z = waPick([
      (0, (p - _ctr(0, s)).distance, th(0)),
      (1, (p - _ctr(_n - 1, s)).distance, th(_n - 1)),
      (2, (p - _ctr(3, s)).distance, th(3)),
    ]);
    return z ?? (art(s).inflate(8).contains(p) ? 3 : null);
  }

  double _perp(Offset p, Size s) {
    final d = waUnit(_p1(s) - _p0(s));
    final q = p - _p0(s);
    return q.dx * -d.dy + q.dy * d.dx;
  }

  @override
  Offset? anchor(int z, Offset ptr, Size s, WaCtx x) {
    if (z == 3) return ptr;
    final k = switch (z) { 0 => 0, 1 => _n - 1, _ => 3 };
    final c = _ctr(k, s);
    return c + waUnit(ptr - c) * _rad(k, s, x);
  }

  @override
  void begin(int z, Offset vp, Size s, WaCtx x) {
    _j0 = doc['sd.jit'];
    _spAbs = _perp(vp, s).abs();
    if (z < 3) _rho0 = (vp - _ctr(const [0, _n - 1, 3][z], s)).distance / _r100(s) * 100;
  }

  @override
  void drag(int z, Offset vp, Size s, WaCtx x) {
    final r100 = _r100(s);
    // gain .65 on the radial pull: a short drag moves the size, it does not jump to the end of the range
    double pct(int k) => _unshow(_rho0 + ((vp - _ctr(k, s)).distance / r100 * 100 - _rho0) * .65);
    switch (z) {
      case 0:
        set('sd.min', pct(0));
      case 1:
        set('sd.max', pct(_n - 1));
      case 2:
        final mn = doc['sd.min'], mx = doc['sd.max'];
        if ((mx - mn).abs() < 6) return;
        final u = waClamp((pct(3) - mn) / (mx - mn), .03, .97);
        set('sd.curve', math.log(u) / math.log(.5));
      default:
        set('sd.jit', _j0 + (_perp(vp, s).abs() - _spAbs) / (art(s).height * .35));
    }
  }

  @override
  List<String> readout(WaCtx x) => [x.doc.spec('sd.min').fmt(x.v('sd.min')), x.doc.spec('sd.max').fmt(x.v('sd.max')), '×${x.doc.spec('sd.curve').fmt(x.v('sd.curve'))}', x.doc.spec('sd.jit').fmt(x.v('sd.jit'))];

  @override
  void paint(Canvas cv, Size s, WaCtx x) {
    final jit = x.v('sd.jit'), r100 = _r100(s);
    for (var k = 0; k < _n; k++) {
      final c = _ctr(k, s);
      final base = _rad(k, s, x);
      final w = 1 + jit * .45 * ((waRnd(k * 7 + 3) * 2 - 1) + .35 * math.sin(x.t * 1.4 + k * 1.7));
      final breath = 1 + .02 * math.sin(x.t * 1.1 + k * .5);
      final r = waClamp(base * w * breath, 1.0, r100 * _maxShow);
      if (jit > .02) cv.drawCircle(c, base, waStroke(N.g26));
      cv.drawCircle(c, r, waStroke(N.g76.withValues(alpha: .92)));
      if (r > 6) waArc(cv, c, r * .72, math.pi * 1.08, math.pi * 1.38, waStroke(N.g44.withValues(alpha: .8)));
      final z = k == 0 ? 0 : (k == _n - 1 ? 1 : (k == 3 ? 2 : -1));
      if (z >= 0) {
        final a = -math.pi * .33;
        waArc(cv, c, r + 2.5, a - .3, a + .3, x.mark(z, 1.5));
        if (x.hl(z)) cv.drawCircle(c, r + 7, waStroke(waHot.withValues(alpha: .28)));
      }
    }
    // D: a little orange fizz above the stream; it scatters wider as the sizes get less even
    final d = waUnit(_p1(s) - _p0(s)), nv = Offset(-d.dy, d.dx);
    for (var i = 0; i < 6; i++) {
      final f = ((i * .17 + x.t * .05) % 1.0), side = waRnd(i + 11) < .5 ? -1.0 : 1.0;
      final off = side * (r100 * .9 + 4 + waRnd(i * 3 + 1) * (5 + jit * 28));
      final q0 = Offset.lerp(_p0(s), _p1(s), f)! + nv * off - Offset(0, math.sin(x.t * .9 + i) * 1.4), a = art(s).deflate(2);
      final q = Offset(waClamp(q0.dx, a.left, a.right), waClamp(q0.dy, a.top, a.bottom));
      cv.drawCircle(q, x.hl(3) ? 1.3 : .95, waFill(x.col(3).withValues(alpha: .85)));
    }
  }
}
