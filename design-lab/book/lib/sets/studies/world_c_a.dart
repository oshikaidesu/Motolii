// World C, worlds 13-15: Gaussian Blur (a bleeding bokeh orb), Glow (a star over a waterline of sparks), Echo (a comet that leaves copies).
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_c_kit.dart';

// ---- 13. Gaussian Blur: the out-of-focus light ----------------------------------------------------------------------------------------

const blurSpecs = [
  WSpec('blur.radius', 'Radius', 0, 200, 10, unit: 'px'),
  WSpec('blur.aperture', 'Aperture', 3, 8, 6, integer: true),
  WSpec('blur.direction', 'Direction', 0, 360, 0, unit: '°', wrap: true),
  WSpec('blur.mix', 'Mix', 0, 100, 100, unit: '%'),
];

/// A: the orb's rim (radius). B: the blade corners (aperture, twist along the rim). C: the end of the smear (direction, swing around).
/// D: the crisp original light at the middle (mix, pull up / down).
class BlurWorld extends WWorld {
  BlurWorld(super.doc);
  @override
  String get title => 'Gaussian Blur';
  @override
  String get silhouette => 'smeared light';

  double _rMax(Size s) => math.min(s.width, s.height) * .34;
  double _rMin(Size s) => math.max(14, s.height * .1);
  double _rOf(Size s, double radius) => math.max(_rMin(s), _rMax(s) * (.45 + .55 * math.sqrt(radius / 200)));
  Offset _c(Size s) => Offset(s.width / 2, s.height / 2 - 4);
  double get _rot => -math.pi / 2 + .03 * math.sin(t * .5);

  /// Radius of a unit-circumradius n-gon at angle [th] from a corner; n may be fractional (two polygons blended).
  static double _poly(double th, int n) {
    final sec = 2 * math.pi / n;
    final a = (th % sec + sec) % sec;
    return math.cos(math.pi / n) / math.cos(a - math.pi / n);
  }

  static double polyF(double th, double n) {
    final lo = n.floor().clamp(3, 8), hi = (lo + 1).clamp(3, 8);
    return wcLerp(_poly(th, lo), _poly(th, hi), lo == hi ? 0 : n - lo);
  }

  Path _contour(Offset c, double r, double n, {Offset shift = Offset.zero}) {
    final p = Path();
    const m = 120;
    for (var i = 0; i <= m; i++) {
      final th = i / m * 2 * math.pi;
      final q = c + shift + wcDir(th + _rot) * (r * polyF(th, n));
      i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    return p..close();
  }

  double _breath() => 1 + .015 * math.sin(t * 2 * math.pi / 4.5);
  double get _dir => v('blur.direction') * math.pi / 180;

  Offset _tip(Size s) {
    final c = _c(s), r = _rOf(s, v('blur.radius')) * _breath();
    return c + wcDir(_dir) * (r * .28 + r * polyF(_dir - _rot, v('blur.aperture')));
  }

  List<Offset> _corners(Size s) {
    final c = _c(s), r = _rOf(s, v('blur.radius')) * _breath(), n = v('blur.aperture'), k = n.round();
    return [for (var i = 0; i < k; i++) c + wcDir(_rot + 2 * math.pi * i / k) * (r * polyF(2 * math.pi * i / k, n))];
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final c = _c(s), r = _rOf(s, v('blur.radius')) * _breath();
    if ((p - c).distance <= 12) return 3;
    if ((p - _tip(s)).distance <= 14) return 2;
    for (final q in _corners(s)) {
      if ((p - q).distance <= 14) return 1;
    }
    final d = p - c;
    final th = ((d.direction - _rot) % (2 * math.pi) + 2 * math.pi) % (2 * math.pi);
    if ((d.distance - r * polyF(th, v('blur.aperture'))).abs() <= 12) return 0;
    return null;
  }

  @override
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start) {
    final c = _c(s);
    switch (z) {
      case 0:
        final r0 = _rOf(s, start['blur.radius']!);
        final rn = math.max(0.0, r0 + (p - c).distance - (drag0 - c).distance);
        final f = math.max(0.0, (rn / _rMax(s) - .45) / .55);
        doc.set('blur.radius', 200 * f * f);
      case 1:
        // any move of a vertex counts: across it turns the blades, along it (outward) adds one
        final u = wcDir((drag0 - c).direction), tg = Offset(-u.dy, u.dx), r = _rOf(s, start['blur.radius']!);
        doc.set('blur.aperture', start['blur.aperture']! + (acc.dx * tg.dx + acc.dy * tg.dy) / (r * .5) + (acc.dx * u.dx + acc.dy * u.dy) / 18);
      case 2:
        doc.set('blur.direction', start['blur.direction']! + angDelta(c, p) * 180 / math.pi);
      case _:
        doc.set('blur.mix', start['blur.mix']! - acc.dy / 70 * 100);
    }
  }

  @override
  void paint(Canvas cv, Size s) {
    final c = _c(s), r = _rOf(s, v('blur.radius')) * _breath(), n = v('blur.aperture'), mix = v('blur.mix') / 100;
    final a = .25 + .75 * mix;
    final dirV = wcDir(_dir);
    // the smear: ghosts of the rim trailing toward the direction
    for (var k = 5; k >= 1; k--) {
      cv.drawPath(_contour(c, r, n, shift: dirV * (r * .12 * k)), wcStroke(N.g56.withValues(alpha: .30 * (1 - k / 6) * a)));
    }
    // the disc: faint fill and nested hairlines (a bokeh disc is bright at its rim)
    cv.drawPath(_contour(c, r, n), wcFill(N.g95.withValues(alpha: .05 * a)));
    for (var k = 1; k <= 3; k++) {
      final tw = 1 + .14 * math.sin(t * 1.7 + k * 1.9);
      cv.drawPath(_contour(c, r * (1 - k * .15), n), wcStroke((k == 1 ? N.g44 : N.g26).withValues(alpha: (.7 * a * tw).clamp(0, 1))));
    }
    // A: the rim
    cv.drawPath(_contour(c, r, n), wcStroke(zc(0).withValues(alpha: a.clamp(.45, 1)), zw(0)));
    // B: the blade corners
    for (final q in _corners(s)) {
      cv.drawCircle(q, lit(1) ? 2.6 : 2.0, wcFill(zc(1)));
    }
    if (lit(1)) ring(cv, _corners(s).first, 1);
    // C: the end of the smear: a short crosswise tick and a dot
    final tip = _tip(s), nrm = Offset(-dirV.dy, dirV.dx);
    cv.drawLine(tip - nrm * 5, tip + nrm * 5, wcStroke(zc(2), zw(2)));
    cv.drawCircle(tip, 1.6, wcFill(zc(2)));
    ring(cv, tip, 2);
    // D: the crisp original: a dot and four ticks
    final ca = (.6 + .4 * (1 - mix)).clamp(0.0, 1.0);
    final dc = zc(3).withValues(alpha: ca);
    cv.drawCircle(c, 2.2, wcFill(dc));
    for (var i = 0; i < 4; i++) {
      final u = wcDir(i * math.pi / 2 + math.pi / 4);
      cv.drawLine(c + u * 5, c + u * (8 + (lit(3) ? 2 : 0)), wcStroke(dc, zw(3)));
    }
    ring(cv, c, 3, 12);
  }
}

// ---- 14. Glow: a star and the sparks that ignite ---------------------------------------------------------------------------------------

const glowSpecs = [
  WSpec('glow.radius', 'Radius', 0, 300, 40, unit: 'px'),
  WSpec('glow.intensity', 'Intensity', 0, 300, 100, unit: '%'),
  WSpec('glow.threshold', 'Threshold', 0, 100, 60, unit: '%'),
  WSpec('glow.tint', 'Tint', 0, 100, 0, unit: '%'),
];

/// A: the star's arm tips (radius). B: the core (intensity, pull up / down). C: the waterline of sparks (threshold, up / down).
/// D: the corona of dashes (tint toward orange, turn it around the star).
class GlowWorld extends WWorld {
  GlowWorld(super.doc);
  @override
  String get title => 'Glow';
  @override
  String get silhouette => 'star';

  Offset _c(Size s) => Offset(s.width / 2, s.height * .44);
  double _lMax(Size s) => math.min(s.width, s.height) * .42;
  double _lOf(Size s, double radius) => math.max(18.0, _lMax(s) * math.pow(radius / 300, .4));
  double _top(Size s) => 16;
  double _bot(Size s) => s.height - 16;
  double _yThr(Size s, double thr) => wcLerp(_bot(s), _top(s), thr / 100);
  double _wob() => .02 * math.sin(t * .6);
  double _coreR() => 2.5 + 2.2 * (v('glow.intensity') / 100);
  double _rho(double l) => math.max(21, l * .45);

  /// The sparks: fixed positions, each with its own brightness 0..1 (hash of the index).
  static final _sparks = [
    for (var i = 0; i < 24; i++) (x: .07 + .86 * ((i * 0.6180339 + .13) % 1.0), b: ((i * 0.7548776 + .31) % 1.0)),
  ];

  Offset _spark(Size s, int i) => Offset(_sparks[i].x * s.width, wcLerp(_bot(s), _top(s), _sparks[i].b));

  double _waterY(Size s, double x) => _yThr(s, v('glow.threshold')) + 1.2 * math.sin(x * .05 + t * 1.2);

  List<Offset> _tips(Size s) {
    final c = _c(s), l = _lOf(s, v('glow.radius'));
    return [for (var i = 0; i < 4; i++) c + wcDir(i * math.pi / 2 - math.pi / 2 + _wob()) * l];
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final c = _c(s), l = _lOf(s, v('glow.radius'));
    final d = (p - c).distance;
    if (d <= _coreR() + 8) return 1;
    for (final q in _tips(s)) {
      if ((p - q).distance <= 14) return 0;
    }
    if ((d - _rho(l)).abs() <= 10) return 3;
    if ((p.dy - _waterY(s, p.dx)).abs() <= 12) return 2;
    return null;
  }

  @override
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start) {
    final c = _c(s);
    switch (z) {
      case 0:
        final l0 = _lOf(s, start['glow.radius']!);
        // soft: the tip follows at 0.6 of the pointer and the value grows as a 2.5 power, so a short drag stays gentle
        final ln = math.max(0.0, l0 + .6 * ((p - c).distance - (drag0 - c).distance));
        doc.set('glow.radius', 300 * math.pow(ln / _lMax(s), 2.5).toDouble());
      case 1:
        doc.set('glow.intensity', start['glow.intensity']! - acc.dy / 90 * 300);
      case 2:
        doc.set('glow.threshold', start['glow.threshold']! - acc.dy / (_bot(s) - _top(s)) * 100);
      case _:
        doc.set('glow.tint', start['glow.tint']! + angDelta(c, p) / (1.5 * math.pi) * 100);
    }
  }

  @override
  void paint(Canvas cv, Size s) {
    final c = _c(s), l = _lOf(s, v('glow.radius')), inten = v('glow.intensity') / 100, tint = v('glow.tint') / 100;
    final thr = v('glow.threshold');
    final tone = Color.lerp(N.g76, Fam.follow.c, tint)!;
    // C: the waterline (wavy, dashed) and the sparks: above it they ignite and carry a small halo
    final wl = Path();
    for (double x = 0; x <= s.width; x += 3) {
      final y = _waterY(s, x);
      x == 0 ? wl.moveTo(x, y) : wl.lineTo(x, y);
    }
    final lc = zc(2);
    for (final m in wl.computeMetrics()) {
      for (double d = 0; d < m.length; d += 8) {
        cv.drawPath(m.extractPath(d, math.min(d + 4, m.length)), wcStroke(lc.withValues(alpha: lit(2) ? 1 : .75), zw(2)));
      }
    }
    for (var i = 0; i < _sparks.length; i++) {
      final q = _spark(s, i), on = _sparks[i].b * 100 > thr;
      final tw = 1 + .25 * math.sin(t * 2.1 + i * 1.7);
      if (on) {
        cv.drawCircle(q, 1.2 + _sparks[i].b * 1.2, wcFill(N.g95.withValues(alpha: (.9 * math.min(1.0, .5 + inten * .5) * tw).clamp(0, 1))));
        cv.drawCircle(q, 3 + l * .08, wcStroke(tone.withValues(alpha: .22 * math.min(1.0, inten))));
      } else {
        cv.drawCircle(q, 1, wcFill(N.g38));
      }
    }
    // the halo rings of the star
    for (var k = 0; k < 3; k++) {
      cv.drawCircle(c, l * (.30 + .28 * k), wcStroke(tone.withValues(alpha: (.16 - .045 * k) * math.min(1.0, .4 + inten * .6))));
    }
    // the arms (fade to nothing at the tip)
    for (var i = 0; i < 8; i++) {
      final main = i.isEven, ang = i * math.pi / 4 - math.pi / 2 + _wob();
      final len = main ? l : l * .5;
      final tipP = c + wcDir(ang) * len;
      final a = (.85 * math.min(1.0, .35 + inten * .65)).clamp(0.0, 1.0);
      final sh = ui.Gradient.linear(c, tipP, [N.g95.withValues(alpha: a), tone.withValues(alpha: 0)]);
      cv.drawLine(c, tipP, Paint()
        ..shader = sh
        ..strokeWidth = main ? 1.2 : 1
        ..strokeCap = StrokeCap.round);
    }
    // A: the tips
    for (final q in _tips(s)) {
      final nrm = (q - c) / (q - c).distance;
      final perp = Offset(-nrm.dy, nrm.dx);
      cv.drawLine(q - perp * 4, q + perp * 4, wcStroke(zc(0), zw(0)));
    }
    if (lit(0)) ring(cv, _tips(s).first, 0);
    // D: the corona of dashes, turned by the tint
    final rho = _rho(l), turn = tint * 1.5 * math.pi;
    const pat = [5.0, 2.0, 3.0, 2.0, 6.0, 2.0, 3.0, 2.0, 5.0, 2.0];
    final dz = zc(3).withValues(alpha: .4 + .5 * tint);
    for (var i = 0; i < pat.length; i++) {
      final ang = turn + i * 2 * math.pi / pat.length - math.pi / 2;
      cv.drawLine(c + wcDir(ang) * rho, c + wcDir(ang) * (rho + pat[i]), wcStroke(dz, zw(3)));
    }
    // B: the core with a green ring
    cv.drawCircle(c, _coreR(), wcFill(N.g95.withValues(alpha: (.6 + .4 * math.min(1.0, inten / 2)).clamp(0, 1))));
    cv.drawCircle(c, _coreR() + 3 + (lit(1) ? 1 : 0), wcStroke(zc(1), zw(1)));
  }
}

// ---- 15. Echo: a point that leaves copies of itself ------------------------------------------------------------------------------------

const echoSpecs = [
  WSpec('echo.count', 'Count', 1, 12, 4, integer: true),
  WSpec('echo.delay', 'Delay', 0, 1, .08, unit: 'ms', scale: 1000),
  WSpec('echo.decay', 'Decay', 0, 100, 60, unit: '%'),
  WSpec('echo.mix', 'Mix', 0, 100, 100, unit: '%'),
];

/// A: the last copy (count, pull it along the tail). B: the first copy (delay, pull it along). C: the two hairlines hugging the copies
/// (decay, move them across). D: the original (mix, up / down).
class EchoWorld extends WWorld {
  EchoWorld(super.doc);
  @override
  String get title => 'Echo';
  @override
  String get silhouette => 'comet';

  Offset _c(Size s) => Offset(s.width / 2, s.height / 2 - 4);
  static const _w = 2 * math.pi / 2.4, _slow = 2.2;

  Offset _pos(Size s, double tau) => _c(s) + Offset(s.width * .33 * math.sin(_w * tau), s.height * .26 * math.sin(2 * _w * tau));

  double get _q => 1 - .65 * v('echo.decay') / 100;
  double _rad(int k) => 5.5 * (.22 + .78 * math.pow(_q, k * .4));

  /// Copy k (0 = the original) at its place on the figure-eight.
  Offset _cp(Size s, int k) => _pos(s, t - k * v('echo.delay') * _slow);

  int get _n => v('echo.count').ceil().clamp(1, 12);

  Offset _normalAt(Size s, int k) {
    final a = _cp(s, math.max(0, k - 1)), b = _cp(s, math.min(_n, k + 1));
    final d = b - a;
    if (d.distance < .01) return const Offset(0, -1);
    final u = d / d.distance;
    return Offset(-u.dy, u.dx);
  }

  late Offset _u0;
  late Offset _n0;
  late double _side;

  Offset _unit(Offset d, Offset fallback) => d.distance < 2 ? fallback : d / d.distance;

  @override
  int? zoneAt(Offset p, Size s) {
    int? best;
    var bd = 1e9;
    void cand(int z, double d, double tol) {
      if (d <= tol && d < bd) {
        best = z;
        bd = d;
      }
    }

    cand(3, (p - _cp(s, 0)).distance, 13);
    cand(0, (p - _cp(s, _n)).distance, 13);
    cand(1, (p - _cp(s, 1)).distance, 13);
    for (var side = -1; side <= 1; side += 2) {
      for (var k = 0; k < _n; k++) {
        final a = _cp(s, k) + _normalAt(s, k) * (side * _rad(k)), b = _cp(s, k + 1) + _normalAt(s, k + 1) * (side * _rad(k + 1));
        cand(2, _segDist(p, a, b), 11);
      }
    }
    return best;
  }

  static double _segDist(Offset p, Offset a, Offset b) {
    final ab = b - a, l2 = ab.dx * ab.dx + ab.dy * ab.dy;
    if (l2 < 1e-6) return (p - a).distance;
    final t = (((p - a).dx * ab.dx + (p - a).dy * ab.dy) / l2).clamp(0.0, 1.0);
    return (p - (a + ab * t)).distance;
  }

  @override
  void onDown(int z, Offset p, Size s) {
    final h = _cp(s, 0), l = _cp(s, _n), e1 = _cp(s, 1);
    _u0 = switch (z) {
      0 => _unit(l - h, const Offset(1, 0)),
      1 => _unit(e1 - h, _unit(l - h, const Offset(1, 0))),
      _ => const Offset(1, 0),
    };
    final tail = _unit(l - h, const Offset(1, 0));
    _n0 = Offset(-tail.dy, tail.dx);
    _side = ((p - h).dx * _n0.dx + (p - h).dy * _n0.dy) >= 0 ? 1 : -1;
  }

  @override
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start) {
    switch (z) {
      case 0:
        final l = (_cp(s, _n) - _cp(s, 0)).distance;
        final step = math.max(16.0, l / start['echo.count']!);
        doc.set('echo.count', start['echo.count']! + (acc.dx * _u0.dx + acc.dy * _u0.dy) / step);
      case 1:
        doc.set('echo.delay', start['echo.delay']! + (acc.dx * _u0.dx + acc.dy * _u0.dy) / 220);
      case 2:
        final pr = (acc.dx * _n0.dx + acc.dy * _n0.dy) * _side;
        doc.set('echo.decay', start['echo.decay']! - pr / 70 * 100);
      case _:
        doc.set('echo.mix', start['echo.mix']! - acc.dy / 70 * 100);
    }
  }

  @override
  void paint(Canvas cv, Size s) {
    final mix = v('echo.mix') / 100, cnt = v('echo.count'), n = _n;
    double fade(int k) => k == n ? (cnt - (n - 1)).clamp(0.0, 1.0) : 1.0;
    // C: two hairlines hugging the copies
    final wc = zc(2).withValues(alpha: (.25 + .4 * mix).clamp(0, 1));
    for (var side = -1; side <= 1; side += 2) {
      final path = Path();
      for (var k = 0; k <= n; k++) {
        final q = _cp(s, k) + _normalAt(s, k) * (side * (_rad(k) + 1.5));
        k == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
      }
      cv.drawPath(path, wcStroke(wc, zw(2)));
    }
    // the path the original runs (a faint trace of the figure it draws)
    final trace = Path();
    for (var i = 0; i <= 120; i++) {
      final q = _pos(s, i / 120 * 2.4);
      i == 0 ? trace.moveTo(q.dx, q.dy) : trace.lineTo(q.dx, q.dy);
    }
    cv.drawPath(trace, wcStroke(N.g20.withValues(alpha: .8)));
    // the copies
    for (var k = n; k >= 1; k--) {
      final a = (math.pow(_q, k * .6).toDouble() * mix * fade(k)).clamp(0.0, 1.0);
      cv.drawCircle(_cp(s, k), _rad(k), wcFill(N.g91.withValues(alpha: (.15 + .85 * a))));
      cv.drawCircle(_cp(s, k), _rad(k), wcStroke(N.g63.withValues(alpha: .15 + .6 * a)));
    }
    final last = _cp(s, n), first = _cp(s, 1), head = _cp(s, 0);
    // A: ring on the last copy; B: a green stitch from the original to the first copy; D: ring on the original
    cv.drawCircle(last, _rad(n) + 4, wcStroke(zc(0), zw(0)));
    ring(cv, last, 0, _rad(n) + 8);
    cv.drawLine(head, first, wcStroke(zc(1), zw(1)));
    cv.drawCircle(first, 2, wcFill(zc(1)));
    ring(cv, first, 1, 8);
    cv.drawCircle(head, 5.5, wcFill(N.g95));
    cv.drawCircle(head, 9, wcStroke(zc(3), zw(3)));
    ring(cv, head, 3, 12);
  }
}

final blurDef = WorldDef(13, 'Gaussian Blur', 'smeared light', blurSpecs, BlurWorld.new);
final glowDef = WorldDef(14, 'Glow', 'star', glowSpecs, GlowWorld.new);
final echoDef = WorldDef(15, 'Echo', 'comet', echoSpecs, EchoWorld.new);
