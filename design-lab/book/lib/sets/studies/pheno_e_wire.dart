part of 'pheno_e.dart';

// 3 React-to and 4 Echo: a wire between two small phenomena, and a comet that leaves a fading trail.

const _rparams = <PParam>[
  PParam('react.inLo', 'Input low', 0, 127, 0, ' vel'),
  PParam('react.inHi', 'Input high', 0, 127, 127, ' vel'),
  PParam('react.outLo', 'Output low', 0, 100, 0, '%'),
  PParam('react.outHi', 'Output high', 0, 100, 100, '%'),
  PParam('react.curve', 'Curve', -100, 100, 0, ''),
];

class _RL {
  _RL(this.s, this.t, this.r, this.amp);
  final Offset s, t;
  final double r, amp;
  Offset get a => s + Offset(r + 5, 0);
  Offset get b => t - Offset(r + 5, 0);
  Offset wire(double u, double gamma) => Offset(_lerp(a.dx, b.dx, u), s.dy + amp * (u - math.pow(u, gamma)));
}

_RL _rlay(Size s) {
  final big = s.height >= 150, cy = s.height / 2 - (big ? 10 : 6), r = big ? 46.0 : 28.0;
  return _RL(Offset(big ? 64 : 46, cy), Offset(s.width - (big ? 64 : 46), cy), r, big ? 46 : 30);
}

class ReactModel extends PhenoModel {
  ReactModel() : super(_rparams, const {'react.inLo': 24, 'react.inHi': 100, 'react.outLo': 12, 'react.outHi': 90, 'react.curve': 40}) {
    springK = 700;
    springZ = .5;
    outS.t = outS.x = _out(_rest) / 100;
  }

  static const _rest = 80.0, _demo = [96.0, 38.0, 118.0, 66.0, 22.0, 104.0, 58.0];
  final Spr outS = Spr(0);
  double? _tapT;
  double _vel = 0, _demoT = .25;
  int _demoI = 0, _which = 0;
  Offset? _hp;

  @override
  String get caption => 'react';
  @override
  String get story => 'React-to: Input low/high (velocity), Output low/high (glow %), Curve';

  double get _gamma => math.pow(4, v['react.curve']! / 100).toDouble();

  double _out(double vin) {
    final lo = v['react.inLo']!, hi = math.max(lo + 1, v['react.inHi']!);
    final f = math.pow(_cl((vin - lo) / (hi - lo)), _gamma).toDouble();
    return _lerp(v['react.outLo']!, v['react.outHi']!, f);
  }

  double _env(double t) => t < 0 ? 0 : (t < .08 ? t / .08 : math.exp(-(t - .08) / .32));
  double get _vNow => _tapT == null ? _rest : _rest + (_vel - _rest) * _env(_tapT!);
  double get _vArrive => _tapT == null ? _rest : _rest + (_vel - _rest) * _env(_tapT! - .28);

  void _tap(double vel) {
    _vel = vel;
    _tapT = 0;
  }

  @override
  bool get alive => hover || _tapT != null || !outS.still;

  @override
  void tick(double dt) {
    if (hover && active == null) {
      _demoT -= dt;
      if (_demoT <= 0) {
        _tap(_demo[_demoI++ % _demo.length]);
        _demoT = 1.5;
      }
    }
    if (_tapT != null) {
      _tapT = _tapT! + dt;
      if (_tapT! > 1.8) _tapT = null;
    }
    outS.t = _out(_vArrive) / 100;
    outS.step(dt, k: 700, z: .5);
  }

  @override
  void onHover(Offset? p, Size s) => _hp = p;

  @override
  int? zoneAt(Offset p, Size s) {
    final l = _rlay(s);
    if ((p - l.s).distance < l.r + 8) return 0;
    if ((p - l.t).distance < l.r + 8) return 1;
    for (var k = 4; k <= 12; k++) {
      if ((p - l.wire(k / 16, _gamma)).distance < 13) return 2;
    }
    return null;
  }

  int _nearest(Offset p, Offset c, double r1, double r2) {
    final d = (p - c).distance, a = (d - r1).abs(), b = (d - r2).abs();
    if ((a - b).abs() < .5) return d > r1 ? 1 : 0;
    return a <= b ? 0 : 1;
  }

  (double, double) _ringsS(_RL l) => (l.r * v['react.inLo']! / 127, l.r * v['react.inHi']! / 127);
  (double, double) _ringsT(_RL l) => (l.r * v['react.outLo']! / 100, l.r * v['react.outHi']! / 100);

  @override
  void onDown(int zone, Offset p, Size s) {
    final l = _rlay(s);
    _tapAt = p;
    if (zone == 0) {
      final (a, b) = _ringsS(l);
      _which = _nearest(p, l.s, a, b);
    } else if (zone == 1) {
      final (a, b) = _ringsT(l);
      _which = _nearest(p, l.t, a, b);
    }
  }

  @override
  void onMove(Offset p, Offset delta, Size s, bool fine) {
    final l = _rlay(s), k = fine ? .1 : 1.0;
    final z = active;
    if (z == 2) {
      // ~70 px of drag spans the whole range, and the last fifth of it slows down (a soft limit)
      final cur = v['react.curve']!, soft = 1 - .75 * math.pow(cur.abs() / 100, 3) * (_sgn(delta.dy) == _sgn(cur) ? 1 : 0);
      add('react.curve', delta.dy * 100 / 70 * k * soft);
      return;
    }
    final c = z == 0 ? l.s : l.t;
    final dd = (p - c).distance - (p - delta - c).distance;
    if (z == 0) {
      final dv = dd / l.r * 127 * k;
      if (_which == 0) {
        add('react.inLo', dv);
        if (v['react.inLo']! > v['react.inHi']! - 1) set('react.inLo', v['react.inHi']! - 1);
      } else {
        add('react.inHi', dv);
        if (v['react.inHi']! < v['react.inLo']! + 1) set('react.inHi', v['react.inLo']! + 1);
      }
    } else {
      final dv = dd / l.r * 100 * k;
      if (_which == 0) {
        add('react.outLo', dv);
        if (v['react.outLo']! > v['react.outHi']!) set('react.outLo', v['react.outHi']!);
      } else {
        add('react.outHi', dv);
        if (v['react.outHi']! < v['react.outLo']!) set('react.outHi', v['react.outLo']!);
      }
    }
  }

  Offset? _tapAt;
  @override
  void onUp(Offset vel, bool moved) {
    if (!moved && active == 0 && _tapAt != null) {
      final l = _rlay(_lastSize);
      _tap(_cl((_tapAt! - l.s).distance / l.r * 127, 12, 127));
    }
  }

  Size _lastSize = const Size(320, 200);

  @override
  void paintMini(Canvas c, Size s, int? hot) {
    _lastSize = s;
    final l = _rlay(s), g = _gamma, wireInk = Role.of(N.g76, Role.linked);
    // the wire is the curve: it sags where the output stays low
    final path = Path()..moveTo(l.a.dx, l.a.dy);
    for (var k = 1; k <= 28; k++) {
      final q = l.wire(k / 28, g);
      path.lineTo(q.dx, q.dy);
    }
    // the range ends: the two faint dotted wires the curve can never pass
    for (final e in const [-100.0, 100.0]) {
      final gp = Path()..moveTo(l.a.dx, l.a.dy);
      for (var k = 1; k <= 20; k++) {
        final q = l.wire(k / 20, math.pow(4, e / 100).toDouble());
        gp.lineTo(q.dx, q.dy);
      }
      _dashed(c, gp, _ln(N.g26), on: 1, off: 3.5);
    }
    c.drawPath(path, _ln(wireInk.withValues(alpha: .85)));
    c.drawCircle(l.a, 2, _fl(wireInk));
    c.drawCircle(l.b, 2, _fl(wireInk));
    if (hot == 2) {
      c.drawCircle(l.wire(.5, g), 5, _ln(active == 2 ? _hotInk : N.g95.withValues(alpha: .5)));
    }
    // source: a held pressure that grows past the first ring and tops out at the second
    final (sa, sb) = _ringsS(l);
    final rs = l.r * _vNow / 127;
    c.drawCircle(l.s, rs, _fl(N.g56.withValues(alpha: .22)));
    c.drawCircle(l.s, rs, _ln(N.g76));
    c.drawCircle(l.s + Offset(-rs * .3, -rs * .3), math.max(.8, rs * .12), _fl(N.g95.withValues(alpha: .5)));
    if (_tapT != null && _tapT! < .5) {
      final q = _tapT! / .5;
      c.drawArc(Rect.fromCircle(center: l.s, radius: l.r * _vel / 127 + 12 * Mo.ease.transform(q)), -.9, 1.8, false, _ln(N.g76.withValues(alpha: .5 * (1 - q))));
    }
    // the two window marks are arcs on the side that faces the wire (like a signal), so they never read as a dial
    void ring(Offset ctr, double r, bool on, bool act, double facing) {
      final col = on ? (act ? _hotInk : N.g95.withValues(alpha: .85)) : N.g56;
      if (r < 2) {
        c.drawCircle(ctr, 1.5, _fl(col));
        return;
      }
      final start = facing > 0 ? -1.0 : math.pi - 1.0;
      _dashed(c, Path()..addArc(Rect.fromCircle(center: ctr, radius: r), start, 2.0), _ln(col), on: 2, off: 2.5);
    }

    final sNearI = active == 0 ? _which : (hot == 0 && _hp != null ? _nearest(_hp!, l.s, sa, sb) : -1);
    ring(l.s, sa, sNearI == 0, active == 0, 1);
    ring(l.s, sb, sNearI == 1, active == 0, 1);
    // target: an orb that answers; its two rings are the output window
    final (ta, tb) = _ringsT(l);
    final out = outS.x;
    final ro = math.max(0.0, l.r * out);
    c.drawCircle(l.t, l.r * (.9 + .5 * out), Paint()..shader = ui.Gradient.radial(l.t, l.r * (.9 + .5 * out), [N.g95.withValues(alpha: .16 * out), N.g95.withValues(alpha: 0)], [.3, 1]));
    c.drawCircle(l.t, ro, _fl(N.g63.withValues(alpha: .24)));
    c.drawCircle(l.t, ro, _ln(N.g91));
    final tNearI = active == 1 ? _which : (hot == 1 && _hp != null ? _nearest(_hp!, l.t, ta, tb) : -1);
    ring(l.t, ta, tNearI == 0, active == 1, -1);
    ring(l.t, tb, tNearI == 1, active == 1, -1);
    // the bead that carries a tap along the wire
    if (_tapT != null && _tapT! < .3) {
      final u = Mo.ease.transform(_cl(_tapT! / .28));
      c.drawCircle(l.wire(u, g), 1.5 + _vel / 127 * 2.5, _fl(wireInk));
    }
  }

  @override
  List<Edge> edges(Size s) => [
        _cap('react'),
        Edge('${v['react.inLo']!.round()}–${v['react.inHi']!.round()} vel', left: 7, bottom: 6, hot: changed('react.inLo') || changed('react.inHi')),
        Edge('curve ${v['react.curve']! >= 0 ? '+' : ''}${v['react.curve']!.clamp(-100, 100).round()}${v['react.curve']!.abs() >= 99.5 ? ' max' : ''}', left: s.width / 2 - 24, bottom: 6, hot: changed('react.curve')),
        Edge('${v['react.outLo']!.round()}–${v['react.outHi']!.round()}% glow', right: 7, bottom: 6, hot: changed('react.outLo') || changed('react.outHi')),
      ];
}

// ---------------------------------------------------------------------------------------------------------------------
const _eparams = <PParam>[
  PParam('echo.feedback', 'Feedback', 1, 12, 6, ' echoes'),
  PParam('echo.spacing', 'Spacing', 1, 10, 4, ' fr'),
  PParam('echo.decay', 'Decay', 20, 100, 70, '%'),
];

class EchoModel extends PhenoModel {
  EchoModel() : super(_eparams, const {'echo.feedback': 7, 'echo.spacing': 4, 'echo.decay': 72}) {
    springK = 900;
    springZ = .55;
  }

  static const _dRad = .036;
  double _u = 1.0, _spd = 0;
  int? _near;

  @override
  String get caption => 'echo';
  @override
  String get story => 'Echo: Feedback (echoes), Spacing (frames), Decay (%)';

  Offset _pt(Size s, double u) {
    final big = s.height >= 150, cy = s.height / 2 - (big ? 10 : 6);
    final ax = s.width * (big ? .36 : .34), ay = big ? 54.0 : 26.0;
    return Offset(s.width / 2 + ax * math.sin(u), cy + ay * math.sin(2 * u));
  }

  Offset _ghost(Size s, int k) => _pt(s, _u - k * d('echo.spacing') * _dRad);

  @override
  bool get alive => hover || _spd.abs() > .01;

  @override
  void tick(double dt) {
    final go = hover && active == null && _near == null;
    _spd += ((go ? 1.3 : 0) - _spd) * math.min(1, dt * 6);
    if (_spd.abs() < .005 && !go) _spd = 0;
    _u += _spd * dt;
  }

  @override
  void onHover(Offset? p, Size s) {
    _near = p == null ? null : zoneAt(p, s);
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final n = math.max(1, d('echo.feedback').round());
    final tail = _ghost(s, n), gap = _ghost(s, 1);
    final dt = (p - tail).distance, dg = (p - gap).distance;
    if (dt < 13 && dt <= dg) return 2;
    if (dg < 13) return 1;
    return null;
  }

  @override
  void onMove(Offset p, Offset delta, Size s, bool fine) {
    final k = fine ? .1 : 1.0;
    final head = _pt(s, _u);
    if (active == 1) {
      final g = _ghost(s, 1), aw = g - head, len = aw.distance;
      if (len > 1) {
        final proj = (delta.dx * aw.dx + delta.dy * aw.dy) / len;
        add('echo.spacing', proj * v['echo.spacing']! / math.max(len, 8) * k);
      }
    } else if (active == 2) {
      final n = math.max(1, d('echo.feedback').round());
      final g = _ghost(s, n), aw = g - head, len = aw.distance;
      if (len > 1) {
        final proj = (delta.dx * aw.dx + delta.dy * aw.dy) / len;
        add('echo.feedback', proj * v['echo.feedback']! / math.max(len, 8) * k * 1.2);
      }
      add('echo.decay', -delta.dy * 80 / 60 * k);
    }
  }

  @override
  void onUp(Offset vel, bool moved) {
    if (moved) {
      set('echo.feedback', v['echo.feedback']!.roundToDouble());
      set('echo.spacing', v['echo.spacing']!.roundToDouble());
    }
  }

  @override
  void paintMini(Canvas c, Size s, int? hot) {
    final fb = d('echo.feedback'), dec = d('echo.decay') / 100, n = math.max(1, fb.ceil());
    final track = Path()..moveTo(_pt(s, 0).dx, _pt(s, 0).dy);
    for (var i = 1; i <= 90; i++) {
      final q = _pt(s, i / 90 * math.pi * 2);
      track.lineTo(q.dx, q.dy);
    }
    _dashed(c, track, _ln(N.g26), on: 1, off: 4);
    final head = _pt(s, _u), pts = <Offset>[head];
    final al = <double>[1];
    for (var k = 1; k <= n; k++) {
      pts.add(_ghost(s, k));
      al.add(math.pow(dec, k).toDouble() * _cl(fb - k + 1));
    }
    for (var k = pts.length - 1; k >= 1; k--) {
      c.drawLine(pts[k - 1], pts[k], _ln(N.g76.withValues(alpha: .5 * al[k])));
    }
    for (var k = pts.length - 1; k >= 1; k--) {
      c.drawCircle(pts[k], 3.6 * (1 - .4 * k / n), _fl(N.g95.withValues(alpha: .85 * al[k])));
    }
    final hc = Role.of(N.g100, Fam.follow.c);
    c.drawCircle(head, 4.4, _fl(hc));
    c.drawCircle(head, 6.5, _ln(hc.withValues(alpha: .35)));
    final tail = pts[math.max(1, fb.round()).clamp(1, pts.length - 1)];
    if (hot == 1) _hotRing(c, pts[1], 8, on: active == 1);
    if (hot == 2) _hotRing(c, tail, 9, on: active == 2);
  }

  @override
  List<Edge> edges(Size s) => [
        _cap('echo'),
        Edge('${v['echo.feedback']!.round()} echoes  ${v['echo.spacing']!.round()} fr  ${v['echo.decay']!.round()}%', left: 7, bottom: 6, hot: changed('echo.feedback') || changed('echo.spacing') || changed('echo.decay')),
      ];
}
