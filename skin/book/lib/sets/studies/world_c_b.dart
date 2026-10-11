// World C, worlds 16-18: Wave Warp (a mesh of threads with a wave passing), Fractal Noise (a contour map), Shadow (a sundial).
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_c_kit.dart';

// ---- 16. Wave Warp: a mesh that a wave runs through -----------------------------------------------------------------------------------

const waveSpecs = [
  WSpec('wave.amplitude', 'Amplitude', 0, 100, 20, unit: 'px'),
  WSpec('wave.wavelength', 'Wavelength', 10, 400, 120, unit: 'px'),
  WSpec('wave.phase', 'Phase', 0, 360, 0, unit: '°', wrap: true),
  WSpec('wave.direction', 'Direction', 0, 360, 0, unit: '°', wrap: true),
];

/// A: the blue thread that shows the wave's height (pull a crest up / down). B: two green beads on neighbouring crests (wavelength, stretch
/// the pair). C: the white cork that rides the wave (phase, slide the wave past it). D: the orange crest lines (direction, swing around).
class WaveWorld extends WWorld {
  WaveWorld(super.doc);
  @override
  String get title => 'Wave Warp';
  @override
  String get silhouette => 'flag';

  Offset _c(Size s) => Offset(s.width / 2, s.height / 2 - 4);
  double _lam(Size s, double l) => 18 + s.width * .85 * math.pow((l - 10) / 390, .8);
  double _lamInv(Size s, double d) => 10 + 390 * math.pow(math.max(0.0, (d - 18) / (s.width * .85)), 1 / .8).toDouble();
  double _amp(Size s, double a) => s.height * .26 * math.sqrt(a / 100);
  double _ampInv(Size s, double d) => 100 * math.pow(d / (s.height * .26), 2).toDouble();
  Offset get _d => wcDir(v('wave.direction') * math.pi / 180);
  Offset get _n => Offset(-_d.dy, _d.dx);

  /// The wave phase on screen: the parameter plus the idle flow.
  double get _phi => v('wave.phase') * math.pi / 180 - t * 1.6;

  Offset _warp(Size s, Offset p) {
    final c = _c(s), k = 2 * math.pi / _lam(s, v('wave.wavelength')), a = _amp(s, v('wave.amplitude'));
    final sd = (p - c).dx * _d.dx + (p - c).dy * _d.dy;
    return p + _n * (a * math.sin(k * sd - _phi));
  }

  Offset _thread(Size s, double sd) => _warp(s, _c(s) + _d * sd);

  /// Crest positions along the axis nearest the middle: the first at or after -quarter wavelength.
  double _crest(Size s, int n) {
    final lam = _lam(s, v('wave.wavelength')), k = 2 * math.pi / lam;
    final s0 = (math.pi / 2 + _phi) / k;
    final m = ((-lam * .25 - s0) / lam).ceil();
    return s0 + (m + n) * lam;
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final c = _c(s), lam = _lam(s, v('wave.wavelength'));
    final sc = _cork(s);
    if ((p - sc).distance <= 13) return 2;
    if ((p - _thread(s, _crest(s, 0))).distance <= 13 || (p - _thread(s, _crest(s, 1))).distance <= 13) return 1;
    final L = s.width * .75;
    var best = 1e9;
    for (double sd = -L; sd <= L; sd += 6) {
      best = math.min(best, (p - _thread(s, sd)).distance);
    }
    if (best <= 10) return 0;
    final sd = (p - c).dx * _d.dx + (p - c).dy * _d.dy;
    final m = ((sd - _crest(s, 0)) / lam).round();
    if ((sd - (_crest(s, 0) + m * lam)).abs() <= 11) return 3;
    return null;
  }

  Offset _cork(Size s) => _thread(s, -s.width * .28);

  @override
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start) {
    final c = _c(s);
    final along = acc.dx * _d.dx + acc.dy * _d.dy, across = acc.dx * _n.dx + acc.dy * _n.dy;
    switch (z) {
      case 0:
        final a0 = _amp(s, start['wave.amplitude']!);
        // the thread is pulled toward the pointer: its height follows the pointer across the axis
        doc.set('wave.amplitude', _ampInv(s, math.max(0.0, a0 + across * _sideA)));
      case 1:
        final l0 = _lam(s, start['wave.wavelength']!);
        doc.set('wave.wavelength', _lamInv(s, math.max(18.0, l0 + along * _sideB)));
      case 2:
        final k = 2 * math.pi / _lam(s, start['wave.wavelength']!);
        doc.set('wave.phase', start['wave.phase']! + along * k * 180 / math.pi);
      case _:
        doc.set('wave.direction', start['wave.direction']! + angDelta(c, p) * 180 / math.pi);
    }
  }

  double _sideA = 1, _sideB = 1;

  @override
  void onDown(int z, Offset p, Size s) {
    final c = _c(s);
    if (z == 0) {
      // which side of the axis the pointer is on decides the sign: pull the thread's bulge toward the pointer
      final k = 2 * math.pi / _lam(s, v('wave.wavelength'));
      final sd = (p - c).dx * _d.dx + (p - c).dy * _d.dy, off = (p - c).dx * _n.dx + (p - c).dy * _n.dy;
      final h = math.sin(k * sd - _phi);
      _sideA = (off * h) >= 0 ? 1 : -1;
    } else if (z == 1) {
      final sd = (p - c).dx * _d.dx + (p - c).dy * _d.dy;
      _sideB = (sd - _crest(s, 1)).abs() <= (sd - _crest(s, 0)).abs() ? 1 : -1;
    }
  }

  @override
  void paint(Canvas cv, Size s) {
    final c = _c(s), lam = _lam(s, v('wave.wavelength'));
    final rows = (s.height / 24).floor().clamp(3, 8), cols = (s.width / 30).floor().clamp(6, 11);
    final mesh = wcStroke(N.g26);
    // D: crest lines (straight wavefronts)
    final dz = zc(3).withValues(alpha: lit(3) ? .85 : .42);
    final dd = _d, nn = _n, half = s.width;
    final m0 = ((-s.width * .75 - _crest(s, 0)) / lam).ceil();
    for (var m = m0; _crest(s, 0) + m * lam < s.width * .75; m++) {
      final sd = _crest(s, 0) + m * lam;
      cv.drawLine(c + dd * sd - nn * half, c + dd * sd + nn * half, wcStroke(dz, zw(3)));
    }
    // the mesh
    for (var i = 1; i <= rows; i++) {
      final y = s.height * i / (rows + 1);
      final path = Path();
      for (double x = -6; x <= s.width + 6; x += 5) {
        final q = _warp(s, Offset(x, y));
        x == -6 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
      }
      cv.drawPath(path, mesh);
    }
    for (var i = 1; i <= cols; i++) {
      final x = s.width * i / (cols + 1);
      final path = Path();
      for (double y = -6; y <= s.height + 6; y += 5) {
        final q = _warp(s, Offset(x, y));
        y == -6 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
      }
      cv.drawPath(path, mesh);
    }
    // A: the blue thread along the travel axis
    final th = Path();
    final L = s.width * .75;
    var first = true;
    for (double sd = -L; sd <= L; sd += 4) {
      final q = _thread(s, sd);
      first ? th.moveTo(q.dx, q.dy) : th.lineTo(q.dx, q.dy);
      first = false;
    }
    cv.drawPath(th, wcStroke(zc(0), zw(0)));
    // B: two beads on neighbouring crests, a dotted hairline between
    final b0 = _thread(s, _crest(s, 0)), b1 = _thread(s, _crest(s, 1));
    final bz = zc(1);
    for (double u = 0; u < 1; u += .08) {
      cv.drawLine(Offset.lerp(b0, b1, u)!, Offset.lerp(b0, b1, math.min(1, u + .04))!, wcStroke(bz.withValues(alpha: .6)));
    }
    cv.drawCircle(b0, lit(1) ? 2.8 : 2.2, wcFill(bz));
    cv.drawCircle(b1, lit(1) ? 2.8 : 2.2, wcFill(bz));
    ring(cv, b1, 1);
    // C: the cork on the wave
    final ck = _cork(s);
    cv.drawCircle(ck, 3, wcFill(zc(2)));
    ring(cv, ck, 2, 8);
  }
}

// ---- 17. Fractal Noise: a contour map -------------------------------------------------------------------------------------------------

const noiseSpecs = [
  WSpec('noise.scale', 'Scale', 1, 1000, 200, unit: 'px'),
  WSpec('noise.contrast', 'Contrast', 0, 300, 100, unit: '%'),
  WSpec('noise.evolution', 'Evolution', 0, 360, 0, unit: '°/s'),
  WSpec('noise.complexity', 'Complexity', 1, 8, 3, integer: true),
];

int _hash(int x, int y, int o) {
  var n = ((x * 73856093) ^ (y * 19349663) ^ (o * 83492791)) & 0x7fffffff;
  n = ((n >> 13) ^ n) & 0x7fffffff;
  n = (n * 48271) % 2147483647;
  n = (n * 48271) % 2147483647;
  return n;
}

double _vnoise(double x, double y, int o) {
  final ix = x.floor(), iy = y.floor();
  var fx = x - ix, fy = y - iy;
  fx = fx * fx * (3 - 2 * fx);
  fy = fy * fy * (3 - 2 * fy);
  double h(int a, int b) => _hash(ix + a, iy + b, o) / 2147483647.0;
  return wcLerp(wcLerp(h(0, 0), h(1, 0), fx), wcLerp(h(0, 1), h(1, 1), fx), fy);
}

/// A: the four corners of one lattice cell (scale, pull them apart). B: the green contour (contrast, up / down). C: four motes that orbit
/// at the rate of the evolution (right = faster). D: the finest lattice of dots (complexity, up / down).
class NoiseWorld extends WWorld {
  NoiseWorld(super.doc);
  @override
  String get title => 'Fractal Noise';
  @override
  String get silhouette => 'contour map';

  static const _cell = 5.0;
  static const _levels = [.2, .3, .4, .5, .6, .7, .8];
  static const _greenLevel = 4;
  double _psi = 0, _prevT = 0;

  Offset _c(Size s) => Offset(s.width / 2, s.height / 2 - 4);
  double _pOf(Size s, double scale) => 14 + s.height * .75 * math.pow(scale / 1000, .6);
  double _scaleOf(Size s, double p) => 1000 * math.pow(math.max(0.0, (p - 14) / (s.height * .75)), 1 / .6).toDouble();
  Offset _off(int i) => Offset(math.cos(_psi + 2.1 * i), math.sin(_psi + 2.1 * i)) * 1.3;
  static const _motes = [Offset(14, -34), Offset(36, -42), Offset(50, -26), Offset(26, -20)];
  List<Offset> _moteBase(Size s) => [for (final m in _motes) Offset(m.dx, s.height + m.dy * (s.height < 110 ? .6 : 1))];

  @override
  void step(double dt) {
    final dT = t - _prevT;
    _prevT = t;
    _psi += (v('noise.evolution') * math.pi / 180 * .25 + .07) * dT;
  }

  double _field(Size s, double x, double y) {
    final c = _c(s), p = _pOf(s, v('noise.scale')), cx = v('noise.complexity');
    var sum = 0.0, wsum = 0.0;
    for (var i = 0; i < 8; i++) {
      final w = wcClamp(cx - i, 0, 1);
      if (w <= 0) break;
      final f = math.pow(2, i).toDouble(), o = _off(i);
      final a = math.pow(.5, i) * w;
      sum += a * _vnoise((x - c.dx) / p * f + o.dx, (y - c.dy) / p * f + o.dy, i);
      wsum += a;
    }
    final base = sum / wsum;
    return .5 + (base - .5) * 1.7 * v('noise.contrast') / 100;
  }

  late List<(Offset, Offset)> _greenSegs = const [];

  /// The coarsest octave whose cell is small enough to sit inside the box: its corners are the blue handles (the ruler of Scale).
  int _lock = -1;
  int _ruler(Size s) {
    if (grab == 0 && _lock >= 0) return _lock;
    final p = _pOf(s, v('noise.scale')), lim = math.min(s.width, s.height) * .40;
    var m = 0;
    while (m < 7 && p / math.pow(2, m) > lim) {
      m++;
    }
    return m;
  }

  /// Corners of the lattice cell under the middle, at the ruler octave (index a * 2 + b).
  List<Offset> _corners(Size s) {
    final m = _ruler(s), c = _c(s), p = _pOf(s, v('noise.scale')) / math.pow(2, m), o = _off(m);
    final i0 = o.dx.floor(), j0 = o.dy.floor();
    return [for (final a in [0, 1]) for (final b in [0, 1]) Offset(c.dx + (i0 + a - o.dx) * p, c.dy + (j0 + b - o.dy) * p)];
  }

  Offset _opp = Offset.zero;
  @override
  void onDown(int z, Offset p, Size s) {
    if (z != 0) return;
    _lock = -1;
    _lock = _ruler(s);
    final cs = _corners(s);
    var bi = 0;
    for (var i = 1; i < 4; i++) {
      if ((p - cs[i]).distance < (p - cs[bi]).distance) bi = i;
    }
    _opp = cs[3 - bi];
  }

  int get _top => math.max(0, v('noise.complexity').round() - 1);

  /// The finest octave whose dots are at least 6 px apart.
  int _finest(Size s) {
    final p = _pOf(s, v('noise.scale'));
    var m = _top;
    while (m > 0 && p / math.pow(2, m) < 6) {
      m--;
    }
    return m;
  }

  Offset _nearestLattice(Size s, Offset q, int oct) {
    final c = _c(s), p = _pOf(s, v('noise.scale')) / math.pow(2, oct), o = _off(oct);
    final i = ((q.dx - c.dx) / p + o.dx).round(), j = ((q.dy - c.dy) / p + o.dy).round();
    return Offset(c.dx + (i - o.dx) * p, c.dy + (j - o.dy) * p);
  }

  @override
  int? zoneAt(Offset p, Size s) {
    for (final m in _moteBase(s)) {
      if ((p - m).distance <= 13) return 2;
    }
    for (final q in _corners(s)) {
      if ((p - q).distance <= 14) return 0;
    }
    for (final g in _greenSegs) {
      if (_segDist(p, g.$1, g.$2) <= 9) return 1;
    }
    if ((p - _nearestLattice(s, p, _finest(s))).distance <= 9) return 3;
    return null;
  }

  static double _segDist(Offset p, Offset a, Offset b) {
    final ab = b - a, l2 = ab.dx * ab.dx + ab.dy * ab.dy;
    if (l2 < 1e-6) return (p - a).distance;
    final t = (((p - a).dx * ab.dx + (p - a).dy * ab.dy) / l2).clamp(0.0, 1.0);
    return (p - (a + ab * t)).distance;
  }

  @override
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start) {
    switch (z) {
      case 0:
        // blue corners drive Scale: the cell grows by the ratio of the stretched diagonal (stable, no jumps)
        final p0 = _pOf(s, start['noise.scale']!);
        final r = (p - _opp).distance / math.max(8.0, (drag0 - _opp).distance);
        doc.set('noise.scale', _scaleOf(s, math.max(14.01, p0 * r)));
      case 1:
        doc.set('noise.contrast', start['noise.contrast']! - acc.dy / 70 * 200);
      case 2:
        doc.set('noise.evolution', start['noise.evolution']! + acc.dx / 90 * 360);
      case _:
        doc.set('noise.complexity', start['noise.complexity']! - acc.dy / 22);
    }
  }

  @override
  void paint(Canvas cv, Size s) {
    final cols = (s.width / _cell).ceil(), rows = (s.height / _cell).ceil();
    final f = List<double>.filled((cols + 1) * (rows + 1), 0);
    for (var j = 0; j <= rows; j++) {
      for (var i = 0; i <= cols; i++) {
        f[j * (cols + 1) + i] = _field(s, i * _cell, j * _cell);
      }
    }
    final grey = Path(), green = Path();
    final gs = <(Offset, Offset)>[];
    for (var li = 0; li < _levels.length; li++) {
      final L = _levels[li];
      final target = li == _greenLevel ? green : grey;
      for (var j = 0; j < rows; j++) {
        for (var i = 0; i < cols; i++) {
          final tl = f[j * (cols + 1) + i], tr = f[j * (cols + 1) + i + 1], br = f[(j + 1) * (cols + 1) + i + 1], bl = f[(j + 1) * (cols + 1) + i];
          final idx = (tl > L ? 8 : 0) | (tr > L ? 4 : 0) | (br > L ? 2 : 0) | (bl > L ? 1 : 0);
          if (idx == 0 || idx == 15) continue;
          final x = i * _cell, y = j * _cell;
          double lerpT(double a, double b) => (L - a) / (b - a);
          Offset top() => Offset(x + _cell * lerpT(tl, tr), y);
          Offset right() => Offset(x + _cell, y + _cell * lerpT(tr, br));
          Offset bottom() => Offset(x + _cell * lerpT(bl, br), y + _cell);
          Offset left() => Offset(x, y + _cell * lerpT(tl, bl));
          void seg(Offset a, Offset b) {
            target.moveTo(a.dx, a.dy);
            target.lineTo(b.dx, b.dy);
            if (li == _greenLevel) gs.add((a, b));
          }

          final centre = (tl + tr + br + bl) / 4 > L;
          switch (idx) {
            case 1 || 14:
              seg(left(), bottom());
            case 2 || 13:
              seg(bottom(), right());
            case 3 || 12:
              seg(left(), right());
            case 4 || 11:
              seg(top(), right());
            case 6 || 9:
              seg(top(), bottom());
            case 7 || 8:
              seg(top(), left());
            case 5:
              if (centre) {
                seg(left(), top());
                seg(bottom(), right());
              } else {
                seg(left(), bottom());
                seg(top(), right());
              }
            case 10:
              if (centre) {
                seg(top(), right());
                seg(left(), bottom());
              } else {
                seg(left(), top());
                seg(bottom(), right());
              }
          }
        }
      }
    }
    _greenSegs = gs;
    cv.drawPath(grey, wcStroke(N.g38.withValues(alpha: grab == null ? 1 : .6), .8));
    cv.drawPath(green, wcStroke(zc(1), zw(1)));

    // the lattices: octave 0 faint, finer octaves in orange, the finest the brightest (D)
    final c = _c(s), p0 = _pOf(s, v('noise.scale')), fin = _finest(s);
    final dots = <Offset>[];
    for (var o = 0; o <= fin; o++) {
      final pp = p0 / math.pow(2, o), off = _off(o);
      if (pp < 6 && o > 0) break;
      dots.clear();
      final i0 = (((0 - c.dx) / pp) + off.dx).floor(), i1 = (((s.width - c.dx) / pp) + off.dx).ceil();
      final j0 = (((0 - c.dy) / pp) + off.dy).floor(), j1 = (((s.height - c.dy) / pp) + off.dy).ceil();
      for (var j = j0; j <= j1; j++) {
        for (var i = i0; i <= i1; i++) {
          dots.add(Offset(c.dx + (i - off.dx) * pp, c.dy + (j - off.dy) * pp));
        }
      }
      final col = o == 0 && fin > 0 ? N.g44.withValues(alpha: .45) : zc(3).withValues(alpha: o == fin ? (lit(3) ? .95 : .55) : .22);
      cv.drawPoints(ui.PointMode.points, dots, Paint()
        ..color = col
        ..strokeWidth = o == fin && lit(3) ? 2.2 : 1.6
        ..strokeCap = StrokeCap.round);
    }
    // A: one cell's corners
    final cs = _corners(s);
    for (final q in cs) {
      cv.drawCircle(q, lit(0) ? 3 : 2.4, wcFill(zc(0)));
    }
    if (lit(0)) ring(cv, cs.first, 0);
    // C: the motes
    for (var i = 0; i < _motes.length; i++) {
      final b = _moteBase(s)[i];
      final q = b + Offset(math.cos(_psi * 1.0 + i * 1.7), math.sin(_psi * 1.0 + i * 1.7)) * 5;
      cv.drawCircle(b, 5.5, wcFill(N.g07.withValues(alpha: .85)));
      cv.drawCircle(b, 5.5, wcStroke(N.g26));
      cv.drawCircle(q, 1.7, wcFill(zc(2)));
      ring(cv, b, 2, 10);
    }
  }
}

// ---- 18. Shadow: a sundial ------------------------------------------------------------------------------------------------------------

const shadowSpecs = [
  WSpec('shadow.distance', 'Distance', 0, 100, 10, unit: 'px'),
  WSpec('shadow.angle', 'Angle', 0, 360, 135, unit: '°', wrap: true),
  WSpec('shadow.softness', 'Softness', 0, 100, 12, unit: 'px'),
  WSpec('shadow.opacity', 'Opacity', 0, 100, 50, unit: '%'),
];

/// A: the umbra edge of the shadow (distance, pull along the shadow). B: the little sun (angle, carry it around). C: the penumbra edge
/// (softness, move it out / in). D: the hatching inside the shadow (opacity, up / down).
class ShadowWorld extends WWorld {
  ShadowWorld(super.doc);
  @override
  String get title => 'Shadow';
  @override
  String get silhouette => 'sundial';

  Offset _c(Size s) => Offset(s.width / 2, s.height / 2 - 4);
  double _half(Size s) => wcClamp(s.height * .14, 10, 28);
  double _offOf(Size s, double d) => s.height * .26 * math.sqrt(d / 100);
  double _spOf(double soft) => 3 + soft / 100 * 34;
  Offset get _dirS {
    final a = v('shadow.angle') * math.pi / 180;
    return Offset(math.sin(a), -math.cos(a));
  }

  double get _breath => 1 + .012 * math.sin(t * 1.3);
  Offset _sc(Size s) => _c(s) + _dirS * (_offOf(s, v('shadow.distance')) * _breath);
  Offset _sun(Size s) => _c(s) - _dirS * (math.min(s.width, s.height) * .40);

  static double _sd(Offset q, double h, double r) {
    final d = Offset(q.dx.abs() - (h - r), q.dy.abs() - (h - r));
    return Offset(math.max(d.dx, 0), math.max(d.dy, 0)).distance + math.min(math.max(d.dx, d.dy), 0) - r;
  }

  @override
  int? zoneAt(Offset p, Size s) {
    if ((p - _sun(s)).distance <= 14) return 1;
    final sd = _sd(p - _sc(s), _half(s), 3), sp = _spOf(v('shadow.softness')) * (1 + .03 * math.sin(t * 1.1));
    final da = sd.abs(), dc = (sd - sp).abs();
    if (da <= 10 && da <= dc) return 0;
    if (dc <= 10) return 2;
    if (sd < -8) return 3;
    return null;
  }

  @override
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start) {
    final c = _c(s);
    switch (z) {
      case 0:
        final o = _offOf(s, start['shadow.distance']!) + acc.dx * _dirS.dx + acc.dy * _dirS.dy;
        final f = math.max(0.0, o) / (s.height * .26);
        doc.set('shadow.distance', 100 * f * f);
      case 1:
        doc.set('shadow.angle', start['shadow.angle']! + angDelta(c, p) * 180 / math.pi);
      case 2:
        final sc = _sc(s);
        final dsd = _sd(p - sc, _half(s), 3) - _sd(drag0 - sc, _half(s), 3);
        doc.set('shadow.softness', (_spOf(start['shadow.softness']!) + dsd - 3) / 34 * 100);
      case _:
        doc.set('shadow.opacity', start['shadow.opacity']! - acc.dy / 80 * 100);
    }
  }

  @override
  void paint(Canvas cv, Size s) {
    final c = _c(s), h = _half(s), sc = _sc(s), op = v('shadow.opacity') / 100;
    final sp = _spOf(v('shadow.softness')) * (1 + .03 * math.sin(t * 1.1));
    // the ground: a quiet field of dots (the shadow swallows them), no fill
    final gd = <Offset>[];
    for (double y = 7; y < s.height; y += 12) {
      for (double x = 7; x < s.width; x += 12) {
        gd.add(Offset(x, y));
      }
    }
    cv.drawPoints(ui.PointMode.points, gd, Paint()
      ..color = N.g26.withValues(alpha: .55)
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round);
    // the shadow: a blurred dark shape
    final rr = RRect.fromRectAndRadius(Rect.fromCenter(center: sc, width: h * 2, height: h * 2), const Radius.circular(3));
    cv.drawRRect(rr, Paint()
      ..color = N.g00.withValues(alpha: op)
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, math.max(.5, sp * .45)));
    // D: hatching inside the shadow, denser when darker
    cv.save();
    cv.clipRRect(rr);
    final gap = 3 + (1 - op) * 9;
    final hz = zc(3).withValues(alpha: lit(3) ? .9 : .5);
    for (double u = -h * 2; u <= h * 2; u += gap) {
      cv.drawLine(sc + Offset(u - h, h), sc + Offset(u + h, -h), wcStroke(hz, zw(3)));
    }
    cv.restore();
    // A: the umbra edge, C: the penumbra edge
    cv.drawRRect(rr, wcStroke(zc(0), zw(0)));
    final outer = rr.inflate(sp);
    final path = Path()..addRRect(outer);
    for (final m in path.computeMetrics()) {
      for (double d = 0; d < m.length; d += 6) {
        cv.drawPath(m.extractPath(d, math.min(d + 3, m.length)), wcStroke(zc(2).withValues(alpha: lit(2) ? 1 : .7), zw(2)));
      }
    }
    // the object that casts it
    final obj = RRect.fromRectAndRadius(Rect.fromCenter(center: c, width: h * 2, height: h * 2), const Radius.circular(3));
    cv.drawRRect(obj, wcFill(N.g26));
    cv.drawRRect(obj, wcStroke(N.g76));
    // B: the sun, with a hairline of light toward the object
    final sun = _sun(s), toObj = (c - sun) / (c - sun).distance;
    for (double d = 12; d < (c - sun).distance - h * 1.6; d += 7) {
      cv.drawLine(sun + toObj * d, sun + toObj * (d + 3), wcStroke(N.g26));
    }
    final sz = zc(1);
    cv.drawCircle(sun, 3.4, wcFill(sz));
    for (var i = 0; i < 8; i++) {
      final u = wcDir(i * math.pi / 4 + t * .2);
      cv.drawLine(sun + u * 6, sun + u * (lit(1) ? 10 : 8.5), wcStroke(sz, zw(1)));
    }
    ring(cv, sun, 1, 13);
  }
}

final waveDef = WorldDef(16, 'Wave Warp', 'flag', waveSpecs, WaveWorld.new);
final noiseDef = WorldDef(17, 'Fractal Noise', 'contour map', noiseSpecs, NoiseWorld.new);
final shadowDef = WorldDef(18, 'Shadow', 'sundial', shadowSpecs, ShadowWorld.new);
