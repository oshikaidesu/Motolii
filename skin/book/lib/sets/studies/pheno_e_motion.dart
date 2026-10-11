part of 'pheno_e.dart';

// 5 Spin and 6 Dough: a spinning top that you flick and a piece of dough that you pull.

const _sparams = <PParam>[
  PParam('spin.speed', 'Speed', -720, 720, 90, '°/s'),
  PParam('spin.friction', 'Friction', 0, 100, 15, '%'),
];

class _SL {
  _SL(this.cx, this.gy, this.r, this.rx);
  final double cx, gy, r, rx;
  double get ry => rx * .13;
}

_SL _slay(Size s) {
  final big = s.height >= 150;
  final r = big ? 46.0 : 30.0;
  return _SL(s.width / 2, s.height - (big ? 52 : 32), r, math.min(s.width * .36, r * 3.4));
}

class SpinModel extends PhenoModel {
  SpinModel() : super(_sparams, const {'spin.speed': 240, 'spin.friction': 18}) {
    omega = 240;
  }

  double omega = 0, phase = .6, prec = 0, _vx = 0;
  final Stopwatch _sw = Stopwatch()..start();
  int _lastMs = 0;
  int _zone = 0;

  @override
  String get caption => 'spin';
  @override
  String get story => 'Spin: Speed (°/s, the sign is the direction) and Friction (%)';

  double get _lean => _cl(1 - omega.abs() / 140);

  @override
  Object? saveExtra() => omega;
  @override
  void loadExtra(Object? x) => omega = x as double;
  @override
  String extraSig() => omega.toStringAsFixed(2);
  @override
  void resetExtra() => omega = v['spin.speed']!;

  @override
  bool get alive => omega.abs() > .4 || active != null || _lean > 0 && prec % math.pi != 0;

  @override
  void tick(double dt) {
    final holding = active == 0;
    if (holding) {
      omega *= math.exp(-dt * 9);
    } else {
      omega *= math.exp(-v['spin.friction']! * .04 * dt);
      if (!hover) omega *= math.exp(-.7 * dt);
    }
    if (omega.abs() < .4 && !holding) omega = 0;
    phase += omega * dt * math.pi / 180;
    final s = _lean;
    if (omega.abs() > 4) {
      prec += (4 + 10 * s) * dt;
    } else {
      final tgt = (prec / math.pi).roundToDouble() * math.pi;
      prec += (tgt - prec) * math.min(1, dt * 6);
      if ((tgt - prec).abs() < .002) prec = tgt;
    }
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final l = _slay(s);
    if (p.dx > l.cx - l.r - 10 && p.dx < l.cx + l.r + 10 && p.dy > l.gy - l.r * 2.0 - 10 && p.dy < l.gy + 4) return 0;
    if ((p.dx - l.cx).abs() < l.rx + 8 && (p.dy - l.gy).abs() < l.ry + 12) return 1;
    return null;
  }

  @override
  void onDown(int zone, Offset p, Size s) {
    _zone = zone;
    _vx = 0;
    _lastMs = _sw.elapsedMilliseconds;
  }

  @override
  void onMove(Offset p, Offset delta, Size s, bool fine) {
    final l = _slay(s);
    if (_zone == 1) {
      add('spin.friction', delta.dx * 100 / (l.rx * 1.6) * (fine ? .1 : 1));
      return;
    }
    final now = _sw.elapsedMilliseconds, dt = math.max(4, now - _lastMs) / 1000;
    _lastMs = now;
    _vx = _lerp(_vx, delta.dx / dt, .5);
    omega = _lerp(omega, _cl(_vx / l.r * 57.3, -1500, 1500), .6);
  }

  @override
  void onUp(Offset vel, bool moved) {
    if (_zone == 1) return;
    final l = _slay(_lastSize);
    if (!moved) {
      omega = v['spin.speed']!.abs() < 20 ? 90 : v['spin.speed']!;
      return;
    }
    final fl = vel.dx / l.r * 57.3;
    if (fl.abs() > 40) omega = _cl(fl * .7, -1500, 1500);
    set('spin.speed', omega.roundToDouble());
    omega = _cl(omega, -720, 720);
  }

  Size _lastSize = const Size(320, 200);

  /// A top: a straight cone up to the shoulder at 62 % of its height, then a rounded dome.
  double _w(double t, double r) {
    if (t < .62) return r * math.pow(t / .62, .92).toDouble();
    final u = (t - .62) / .38;
    return r * math.sqrt(math.max(0, 1 - u * u * .78));
  }

  @override
  void paintMini(Canvas c, Size s, int? hot) {
    _lastSize = s;
    final l = _slay(s), h = l.r * 1.55;
    // the ground: friction is how rough it is
    c.drawOval(Rect.fromCenter(center: Offset(l.cx, l.gy), width: l.rx * 2, height: l.ry * 2), _ln(hot == 1 ? (active == 1 ? _hotInk : N.g76) : N.g38));
    var seed = 77;
    double rnd() {
      seed = (seed * 1103515245 + 12345) & 0x7fffffff;
      return seed / 0x7fffffff;
    }

    final n = 4 + (d('spin.friction') * .5).round();
    for (var i = 0; i < n; i++) {
      final x = (rnd() * 2 - 1) * l.rx * .86, y = (rnd() * 2 - 1) * l.ry * .7;
      if ((x / l.rx).abs() < .0 || (x * x) / (l.rx * l.rx) + (y * y) / (l.ry * l.ry) > .8) continue;
      c.drawLine(Offset(l.cx + x, l.gy + y), Offset(l.cx + x + 2.5, l.gy + y), _ln(N.g56.withValues(alpha: .7)));
    }
    final sg = _lean, amp = .025 + .5 * math.pow(sg, 1.4).toDouble();
    c.save();
    c.translate(l.cx, l.gy);
    c.rotate(amp * math.cos(prec));
    // body
    final left = <Offset>[], right = <Offset>[];
    for (var i = 0; i <= 20; i++) {
      final t = i / 20, w = _w(t, l.r);
      left.add(Offset(-w, -t * h));
      right.add(Offset(w, -t * h));
    }
    final capW = _w(1, l.r), capRy = capW * .2;
    final body = Path()..moveTo(left.first.dx, left.first.dy);
    for (final q in left.skip(1)) {
      body.lineTo(q.dx, q.dy);
    }
    body.arcToPoint(Offset(capW, -h), radius: Radius.elliptical(capW, capRy), clockwise: true);
    for (final q in right.reversed.skip(1)) {
      body.lineTo(q.dx, q.dy);
    }
    body.close();
    c.drawLine(Offset(0, -h), Offset(0, -h - l.r * .4), _ln(N.g76, 2));
    c.drawPath(body, _fl(N.g13));
    // two latitude arcs make it round
    for (final t in [.3, .62]) {
      final w = _w(t, l.r);
      c.drawArc(Rect.fromCenter(center: Offset(0, -t * h), width: w * 2, height: w * .4), 0, math.pi, false, _ln(N.g38));
    }
    // meridians: they slide across, so the spin is seen
    final mark = Role.of(N.g100, Role.mode);
    void meridian(double phi, Color col, double wd, double alpha) {
      final cs = math.cos(phi);
      if (cs <= 0) return;
      final pth = Path();
      for (var i = 1; i <= 16; i++) {
        final t = .06 + .94 * i / 16, w = _w(t, l.r);
        final q = Offset(w * math.sin(phi), -t * h + .2 * w * cs);
        i == 1 ? pth.moveTo(q.dx, q.dy) : pth.lineTo(q.dx, q.dy);
      }
      c.drawPath(pth, _ln(col.withValues(alpha: alpha * (.25 + .75 * cs)), wd));
    }

    for (var k = 1; k < 6; k++) {
      meridian(phase + k * math.pi / 3, N.g76, 1, .8);
    }
    final trail = _cl(omega.abs() / 720);
    for (var j = 3; j >= 1; j--) {
      meridian(phase - _sgn(omega) * trail * j * .13, mark, 1, .22 / j);
    }
    meridian(phase, mark, 1.5, 1);
    c.drawPath(body, _ln(N.g76));
    c.restore();
    if (hot == 0 || active == 0) {
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(l.cx - l.r - 6, l.gy - h - 8, l.cx + l.r + 6, l.gy + 2), const Radius.circular(8)), _ln(active == 0 ? _hotInk : N.g95.withValues(alpha: .4)));
    }
  }

  @override
  List<Edge> edges(Size s) => [
        _cap('spin'),
        Edge('${v['spin.speed']! >= 0 ? '+' : ''}${v['spin.speed']!.round()}°/s', left: 7, bottom: 6, hot: changed('spin.speed')),
        Edge('${v['spin.friction']!.round()}% friction', right: 7, bottom: 6, hot: changed('spin.friction')),
      ];
}

// ---------------------------------------------------------------------------------------------------------------------
const _dparams = <PParam>[
  PParam('scale.x', 'Scale X', 10, 200, 100, '%'),
  PParam('scale.y', 'Scale Y', 10, 200, 100, '%'),
  PParam('scale.link', 'Link', 0, 1, 1, ''),
];

class DoughModel extends PhenoModel {
  DoughModel() : super(_dparams, const {'scale.x': 128, 'scale.y': 88}) {
    springK = 700;
    springZ = .45;
    var s = 5;
    double rnd() {
      s = (s * 1103515245 + 12345) & 0x7fffffff;
      return s / 0x7fffffff;
    }

    while (_grain.length < 46) {
      final q = Offset(rnd() * 2 - 1, rnd() * 2 - 1);
      if (math.pow(q.dx.abs(), 2.6) + math.pow(q.dy.abs(), 2.6) < .72) _grain.add(q);
    }
  }

  final List<Offset> _grain = [];
  int _z = 8;
  Offset _p0 = Offset.zero;
  double _sx0 = 100, _sy0 = 100, _rxA = 1, _ryA = 1, _ruA = 1, _rxP = 1, _ryP = 1, _ruP = 1;

  @override
  String get caption => 'dough';
  @override
  String get story => 'Dough: Scale X, Scale Y, Link (corner pull keeps proportion, edge pull stretches one axis)';

  ({double bx, double by}) _base(Size s) {
    final by = (s.height / 2 - 12) / 2.0;
    return (bx: math.min(by * 1.5, (s.width / 2 - 16) / 2.0), by: by);
  }

  Offset _ctr(Size s) => Offset(s.width / 2, s.height / 2 - 3);

  (double, double) _half(Size s) {
    final b = _base(s), cue = (_tanh(sp['scale.x']!.v * .35) - _tanh(sp['scale.y']!.v * .35)) * .05;
    return (b.bx * d('scale.x') / 100 * (1 - cue), b.by * d('scale.y') / 100 * (1 + cue));
  }

  @override
  bool get alive => false;

  @override
  int? zoneAt(Offset p, Size s) {
    final (hx, hy) = _half(s);
    final q = p - _ctr(s);
    final corners = [Offset(-hx, -hy) * _c45, Offset(hx, -hy) * _c45, Offset(hx, hy) * _c45, Offset(-hx, hy) * _c45];
    var best = -1;
    var bd = 14.0;
    for (var i = 0; i < 4; i++) {
      final dd = (q - corners[i]).distance;
      if (dd < bd) {
        bd = dd;
        best = i;
      }
    }
    if (best >= 0) return best;
    final cand = <int, double>{};
    if (q.dx.abs() <= hx * .62) {
      cand[4] = (q.dy + hy).abs();
      cand[6] = (q.dy - hy).abs();
    }
    if (q.dy.abs() <= hy * .62) {
      cand[5] = (q.dx - hx).abs();
      cand[7] = (q.dx + hx).abs();
    }
    int? e;
    var ed = 12.0;
    cand.forEach((k, dd) {
      if (dd < ed) {
        ed = dd;
        e = k;
      }
    });
    if (e != null) return e;
    return (q.dx.abs() < hx && q.dy.abs() < hy) ? 8 : null;
  }

  @override
  void onDown(int zone, Offset p, Size s) {
    _z = zone;
    _p0 = p;
    _sx0 = v['scale.x']!;
    _sy0 = v['scale.y']!;
    _rxA = _ryA = _ruA = _rxP = _ryP = _ruP = 1;
  }

  double _acc(double raw, double prev, double acc, double f) => acc + (raw - prev) * f;

  @override
  void onMove(Offset p, Offset delta, Size s, bool fine) {
    if (_z == 8) return;
    final c = _ctr(s), f = fine ? .1 : 1.0, a = _p0 - c, b = p - c;
    final rx = a.dx.abs() < 4 ? 1.0 : b.dx / a.dx, ry = a.dy.abs() < 4 ? 1.0 : b.dy / a.dy;
    final ru = (a.dx * b.dx + a.dy * b.dy) / math.max(16, a.dx * a.dx + a.dy * a.dy);
    if (_z <= 3) {
      if (v['scale.link']! > .5) {
        _ruA = _acc(ru, _ruP, _ruA, f);
        set('scale.x', _sx0 * _ruA);
        set('scale.y', _sy0 * _ruA);
      } else {
        _rxA = _acc(rx, _rxP, _rxA, f);
        _ryA = _acc(ry, _ryP, _ryA, f);
        set('scale.x', _sx0 * _rxA);
        set('scale.y', _sy0 * _ryA);
      }
    } else if (_z == 4 || _z == 6) {
      _ryA = _acc(ry, _ryP, _ryA, f);
      set('scale.y', _sy0 * _ryA);
    } else {
      _rxA = _acc(rx, _rxP, _rxA, f);
      set('scale.x', _sx0 * _rxA);
    }
    _rxP = rx;
    _ryP = ry;
    _ruP = ru;
  }

  @override
  void onUp(Offset vel, bool moved) {
    if (!moved && _z == 8) {
      set('scale.link', v['scale.link']! > .5 ? 0 : 1);
      sp['scale.x']!.v -= .9;
      sp['scale.y']!.v += .9;
    }
  }

  static const _n = 2.6, _c45 = .766;
  static double _lump(double t) => 1 + .035 * math.sin(3 * t + 1.1) + .025 * math.sin(5 * t + .4);

  Offset _edgePt(double hx, double hy, double t, [bool lumpy = true]) {
    final cs = math.cos(t), sn = math.sin(t), k = lumpy ? _lump(t) : 1.0;
    return Offset(hx * k * _sgn(cs) * math.pow(cs.abs(), 2 / _n), hy * k * _sgn(sn) * math.pow(sn.abs(), 2 / _n));
  }

  Path _contour(double hx, double hy, [double? from, double? to]) {
    final p = Path();
    final a = from ?? 0, b = to ?? math.pi * 2, steps = ((b - a) / (math.pi * 2) * 72).ceil().clamp(2, 72);
    for (var i = 0; i <= steps; i++) {
      final q = _edgePt(hx, hy, a + (b - a) * i / steps);
      i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    return p;
  }

  @override
  void paintMini(Canvas c, Size s, int? hot) {
    final b = _base(s), o = _ctr(s), linked = v['scale.link']! > .5;
    final (hx, hy) = _half(s);
    final act = active != null;
    c.save();
    c.translate(o.dx, o.dy);
    // it rests on a table: a faint shadow, and the 100 % outline as a dashed ghost
    c.drawOval(Rect.fromCenter(center: Offset(0, hy + 7), width: hx * 1.7, height: 7), _fl(N.g00.withValues(alpha: .35)));
    _dashed(c, _contour(b.bx, b.by)..close(), _ln(N.g38), on: 2, off: 3);
    final dg = [for (final k in [5, 7, 1, 3]) _edgePt(hx, hy, k * math.pi / 4)];
    if (linked) {
      final p = _ln(N.g38.withValues(alpha: .7));
      c.drawLine(dg[0] + const Offset(-8, -8), dg[2] + const Offset(8, 8), p);
      c.drawLine(dg[1] + const Offset(8, -8), dg[3] + const Offset(-8, 8), p);
    }
    final body = _contour(hx, hy)..close();
    c.drawPath(body, _fl(N.g13));
    c.save();
    c.clipPath(body);
    c.drawCircle(Offset(-hx * .4, -hy * .5), math.max(hx, hy) * .9, Paint()..shader = ui.Gradient.radial(Offset(-hx * .4, -hy * .5), math.max(hx, hy) * .9, [N.g95.withValues(alpha: .07), N.g95.withValues(alpha: 0)]));
    c.drawRect(Rect.fromLTRB(-hx, hy * .55, hx, hy), _fl(N.g00.withValues(alpha: .18)));
    for (final g in _grain) {
      c.drawCircle(Offset(g.dx * hx, g.dy * hy), .8, _fl(N.g56.withValues(alpha: .6)));
    }
    c.restore();
    c.drawPath(body, _ln(N.g76));
    // the pull points sit on the dough: four diagonal ticks (corners), the four mid-edges light along the skin when touched
    for (var i = 0; i < 4; i++) {
      final q = dg[i], dir = q / q.distance, on = hot == i;
      c.drawLine(q + dir * (on ? 3 : 4), q + dir * (on ? 11 : 8), _ln(on ? (act ? _hotInk : N.g95) : N.g56, on ? 1.5 : 1));
    }
    if (hot != null && hot >= 4 && hot <= 7) {
      final centre = [-math.pi / 2, 0.0, math.pi / 2, math.pi][hot - 4];
      c.drawPath(_contour(hx, hy, centre - .75, centre + .75), _ln(act ? _hotInk : N.g95.withValues(alpha: .75), 1.5));
    }
    if (hot == 8) c.drawCircle(Offset.zero, 5, _ln(act ? _hotInk : N.g95.withValues(alpha: .5)));
    c.restore();
  }

  @override
  List<Edge> edges(Size s) => [
        _cap('dough'),
        Edge('x ${v['scale.x']!.round()}%  y ${v['scale.y']!.round()}%', left: 7, bottom: 6, hot: changed('scale.x') || changed('scale.y')),
        Edge(v['scale.link']! > .5 ? 'link on' : 'link off', right: 7, bottom: 6, hot: v['scale.link']! > .5),
      ];
}

double _tanh(double x) {
  final e = math.exp(2 * x.clamp(-20, 20));
  return (e - 1) / (e + 1);
}
