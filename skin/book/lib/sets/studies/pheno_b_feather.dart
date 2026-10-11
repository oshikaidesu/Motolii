// B6 Mask feather: softness + expansion are the EDGE of a pale pebble-shaped matte, and you handle the edge like a brush: rub along it and it frays soft (the more you rub, the softer),
// push it out or pull it in and the matte grows or shrinks. Hold Alt and rub to harden it again. Tiny flakes fly off the rim while you rub.
part of 'pheno_b.dart';

class FeatherValues extends PhenoValues {
  FeatherValues()
      : super(const [
          PhSpec('feather', 0, 100, 0, unit: 'px'),
          PhSpec('expansion', -50, 50, 0, unit: 'px'),
        ]);
  double get feather => this['feather'];
  double get expansion => this['expansion'];
}

(PhenoValues, PhenoSim Function(PhenoValues)) _makeFeather() => (FeatherValues(), (v) => FeatherSim(v));

class _Flake {
  _Flake(this.p, this.vel);
  Offset p, vel;
  double life = 1;
}

class FeatherSim extends PhenoSim {
  FeatherSim(super.v);
  final _fe = Spr(0), _ex = Spr(0);
  final _flakes = <_Flake>[];
  final _rnd = math.Random(7);
  bool _seeded = false;
  Offset _brush = Offset.zero;
  double _acc = 0, _ex0 = 0, _gap0 = 0, _idle = 0;

  double _k(Size s) => math.min(s.width, s.height) / 200;
  double _ek(Size s) => _k(s) * .6;
  Offset _c(Size s) => s.center(Offset.zero);
  double _r0(Size s) => math.min(s.width, s.height) * .26;
  double _base(double th, double r0) => r0 * (1 + .10 * math.cos(3 * th + .4) + .05 * math.cos(5 * th + 1.3) + .03 * math.cos(2 * th));
  double _edge(double th, Size s, double ex) => math.max(_base(th, _r0(s)) + ex * _ek(s), 4);

  /// Positive inside the hard edge.
  double _sdf(Offset p, Size s, double ex) {
    final d = p - _c(s);
    return _edge(math.atan2(d.dy, d.dx), s, ex) - d.distance;
  }

  @override
  String? zoneAt(Offset p, Size s) {
    final band = 12 + v['feather'] * .2 * _k(s) * 1.4;
    return _sdf(p, s, v['expansion']).abs() < band ? 'edge' : null;
  }

  @override
  void down(String zone, Offset p, Size s) {
    _ex0 = v['expansion'];
    _acc = 0;
    final d = p - _c(s);
    _gap0 = d.distance - _edge(math.atan2(d.dy, d.dx), s, _ex0);
  }

  @override
  void drag(String zone, Offset p, Offset delta, Size s, {required bool fine, required bool alt}) {
    final d = p - _c(s);
    if (d.distance < 1) return;
    final n = d / d.distance, dn = delta.dx * n.dx + delta.dy * n.dy;
    final tan = delta - n * dn;
    var lt = tan.distance;
    // pushing along the normal is "move the edge", rubbing across it is "fray the edge"
    if (lt <= .6 * dn.abs()) lt = 0;
    final k = fine ? .1 : 1.0;
    // the edge sticks to the pointer: it sits where it was grabbed, so rubbing along the rim does not push it
    final ex = (d.distance - _gap0 - _base(math.atan2(d.dy, d.dx), _r0(s))) / _ek(s);
    v.set('expansion', _fine(ex, _ex0, _ex0, fine));
    if (lt > 0) {
      v.set('feather', v['feather'] + (alt ? -1 : 1) * lt * .5 * k / math.max(_k(s), .4));
      _acc += lt;
      while (_acc > 9) {
        _acc -= 9;
        final ang = _rnd.nextDouble() * 2 * math.pi;
        _flakes.add(_Flake(p + Offset(_rnd.nextDouble() * 6 - 3, _rnd.nextDouble() * 6 - 3), (n * (10 + _rnd.nextDouble() * 22)) + Offset(math.cos(ang), math.sin(ang)) * 8));
      }
    }
    _brush = p;
  }

  @override
  List<String> resetIds(String? zone) => v.ids.toList();

  @override
  bool tick(double dt, PhCtx x, Size s) {
    if (!_seeded) {
      _fe.x = v['feather'];
      _ex.x = v['expansion'];
      _brush = _c(s);
      _seeded = true;
    }
    _fe.t = v['feather'];
    _ex.t = v['expansion'];
    _fe.step(dt, k: 520, c: 34);
    _ex.step(dt, k: 520, c: 22);
    if (x.ptr != null) _brush += (x.ptr! - _brush) * (1 - math.exp(-dt * 28));
    for (final f in _flakes) {
      f.p += f.vel * dt;
      f.vel *= math.exp(-dt * 4.5);
      f.life -= dt / .75;
    }
    _flakes.removeWhere((f) => f.life <= 0);
    // idle life: a ghost brush rubs slowly round the rim and now and then sheds a flake
    if (!x.hover && !x.dragging) {
      _idle += dt;
      if (_idle > .5) {
        _idle = 0;
        final th = x.time * .6, d = Offset(math.cos(th), math.sin(th));
        _flakes.add(_Flake(_c(s) + d * _edge(th, s, _ex.x), d * (8 + _rnd.nextDouble() * 8) + Offset(_rnd.nextDouble() * 6 - 3, _rnd.nextDouble() * 6 - 3)));
      }
    }
    return true;
  }

  @override
  List<PhRead> readout() => [
        PhRead(v.fmt('feather'), const {'edge'}, v.changed('feather')),
        PhRead(v.fmt('expansion'), const {'edge'}, v.changed('expansion')),
      ];

  Path _blob(Size s, double ex) {
    final c = _c(s), path = Path();
    for (var i = 0; i < 160; i++) {
      final th = i * 2 * math.pi / 160, r = _edge(th, s, ex), pt = c + Offset(math.cos(th), math.sin(th)) * r;
      i == 0 ? path.moveTo(pt.dx, pt.dy) : path.lineTo(pt.dx, pt.dy);
    }
    return path..close();
  }

  @override
  void paint(Canvas c, Size s, PhCtx x) {
    _lattice(c, s);
    final ex = _ex.x.clamp(-55, 55).toDouble(), sigma = math.max(.01, _fe.x.clamp(0, 110) * .2 * _k(s) * .8);
    final blob = _blob(s, ex);
    // the matte: a calm grey plate; its rim is blurred by the feather
    c.drawPath(blob, Paint()..color = _al(N.g63, .2)..maskFilter = MaskFilter.blur(BlurStyle.normal, sigma));
    // where the edge IS (the 50 % contour): a crisp hair while hard, a dotted hair once frayed
    final hard = (1 - _fe.x / 30).clamp(0.0, 1.0).toDouble();
    if (hard > 0) c.drawPath(blob, _hair(_al(N.g76, .55 * hard)));
    final metrics = blob.computeMetrics().toList();
    if (_fe.x > .6) {
      for (final m in metrics) {
        for (var d = 0.0; d < m.length; d += 7) {
          c.drawPath(m.extractPath(d, math.min(d + 3, m.length)), _hair(_al(N.g76, .45 * (1 - hard))));
        }
      }
    }
    // idle: a ghost rub has already frayed a stretch of the rim, so soft and hard sit side by side
    if (!x.dragging && metrics.isNotEmpty) {
      final m = metrics.first, th = x.time * .6, len = m.length;
      final f = ((th / (2 * math.pi)) % 1.0), w = .07 * len;
      final c0 = f * len, p0 = m.getTangentForOffset(c0)?.position;
      for (var i = 0; i < 3; i++) {
        final o = (c0 - (i + 1) * w * .7 + len) % len, e = o + w;
        final seg = e <= len ? m.extractPath(o, e) : (Path()..addPath(m.extractPath(o, len), Offset.zero)..addPath(m.extractPath(0, e - len), Offset.zero));
        c.drawPath(seg, Paint()..style = PaintingStyle.stroke..strokeWidth = 8 - i * 1.5..color = _al(N.g76, .26 - i * .07)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3));
      }
      if (p0 != null && !x.hover) {
        c.drawCircle(p0, 6, _hair(_al(N.g63, .5)));
        c.drawCircle(p0, 1.1, _fill(_al(N.g95, .8)));
      }
    }
    // a faint ring where the matte began (so push and pull show their distance)
    if (ex.abs() > 1.5) c.drawPath(_blob(s, 0), _hair(_al(N.g56, .28)));

    final hot = x.hot == 'edge';
    if (hot) c.drawPath(blob, _hair(Role.selected));
    // the brush: a small ring that follows the pointer with a little lag
    if (x.hover && (hot || x.dragging)) {
      c.drawCircle(_brush, 11, _hair(_al(hot || x.dragging ? Role.selected : N.g63, .9)));
      c.drawCircle(_brush, 1.4, _fill(Role.selected));
    }
    for (final f in _flakes) {
      c.drawCircle(f.p, .9, _fill(_al(N.g100, f.life * .9)));
    }
  }
}
