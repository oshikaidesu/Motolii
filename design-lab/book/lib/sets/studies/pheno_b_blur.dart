// B5 Blur bokeh: radius + aperture blades + direction (+ streak) are ONE point of light and the shape it smears into.
// Pull the light outward and it opens into a polygon of the lens's aperture (radius); pull an end of it and it stretches into a streak along that line (direction + streak);
// sweep around its rim and the aperture closes or opens one blade at a time (the corner count).
part of 'pheno_b.dart';

class BlurValues extends PhenoValues {
  BlurValues()
      : super(const [
          PhSpec('radius', 0, 40, 22, unit: 'px'),
          PhSpec('blades', 0, 8, 6, unit: '-gon'),
          PhSpec('angle', 0, 180, 0, unit: '°'),
          PhSpec('streak', 0, 1, 0, decimals: 2),
        ]);
  double get radius => this['radius'];
  int get blades => this['blades'].round();
  double get angle => this['angle'];
  double get streak => this['streak'];

  /// 0 is a round aperture; 3..8 are polygons.
  static const stops = [0, 3, 4, 5, 6, 7, 8];
  @override
  String fmt(String id) => id == 'blades' ? (blades == 0 ? 'round' : '$blades-gon') : (id == 'streak' ? '${(this[id] * 100).round()}%' : super.fmt(id));
}

(PhenoValues, PhenoSim Function(PhenoValues)) _makeBlur() => (BlurValues(), (v) => BlurSim(v));

class BlurSim extends PhenoSim {
  BlurSim(super.v);
  final _r = Spr(22), _st = Spr(0), _pop = Spr(1);
  double _ang = 0, _fade = 1, _a0 = 0, _v0 = 0, _phi = 0, _sweep = 0;
  int _n = 6, _prevN = 6, _idx0 = 4;
  bool _seeded = false;

  double _k(Size s) => math.min(s.width, s.height) / 150;
  Offset _c(Size s) => s.center(Offset.zero);
  double _stretch(double st) => 1 + 4 * st;
  double _rpx(Size s, double r) => math.max(r * _k(s), 3.0);

  /// Distance from the centre to the polygon's edge in local direction [phi] (vertex at phi 0).
  double _rb(int n, double r, double phi) {
    if (n == 0) return r;
    final seg = 2 * math.pi / n;
    final m = ((phi % seg) + seg) % seg;
    return r * math.cos(math.pi / n) / math.cos(m - math.pi / n);
  }

  @override
  String? zoneAt(Offset p, Size s) {
    final c = _c(s), r = _rpx(s, v['radius']), st = math.min(_stretch(v['streak']), s.width * .46 / r).clamp(1.0, 6.0).toDouble(), a = v['angle'] * math.pi / 180, n = (v as BlurValues).blades;
    final dir = Offset(math.cos(a), math.sin(a)), len = r * st, d = p - c;
    if (d.distance > 10 && r > 10) {
      if ((p - (c + dir * len)).distance < 12 || (p - (c - dir * len)).distance < 12) return 'tip';
    }
    // into the polygon's own frame
    final lx = d.dx * math.cos(-a) - d.dy * math.sin(-a), ly = d.dx * math.sin(-a) + d.dy * math.cos(-a);
    final pl = Offset(lx / st, ly), sdf = pl.distance - _rb(n, r, math.atan2(pl.dy, pl.dx));
    if (r > 14 && sdf.abs() < 5.5) return 'rim';
    if (d.distance < 14 || sdf < 8) return 'body';
    return null;
  }

  @override
  void down(String zone, Offset p, Size s) {
    final c = _c(s), d = p - c;
    switch (zone) {
      case 'body':
        _a0 = d.distance / _k(s);
        _v0 = v['radius'];
      case 'tip':
        _a0 = _streakOf(p, s);
        _v0 = v['streak'];
        _ang0 = v['angle'];
        _angP0 = _angOf(p, s);
      case 'rim':
        _phi = math.atan2(d.dy, d.dx);
        _sweep = 0;
        _idx0 = _idxOf((v as BlurValues).blades);
    }
  }

  double _ang0 = 0, _angP0 = 0;
  int _idxOf(int b) => BlurValues.stops.indexOf(b).clamp(0, BlurValues.stops.length - 1);
  double _angOf(Offset p, Size s) {
    final d = p - _c(s);
    return ((math.atan2(d.dy, d.dx) * 180 / math.pi) % 180 + 180) % 180;
  }

  double _streakOf(Offset p, Size s) => ((p - _c(s)).distance / _rpx(s, v['radius']) - 1) / 4;

  @override
  void drag(String zone, Offset p, Offset delta, Size s, {required bool fine, required bool alt}) {
    switch (zone) {
      case 'body':
        v.set('radius', _fine((p - _c(s)).distance / _k(s), _a0, _v0, fine));
      case 'tip':
        v.set('streak', _fine(_streakOf(p, s), _a0, _v0, fine));
        final a = _angOf(p, s);
        final dd = ((a - _angP0 + 90) % 180 + 180) % 180 - 90;
        v.set('angle', (((fine ? _ang0 + dd * .1 : a) % 180) + 180) % 180);
      case 'rim':
        final d = p - _c(s), phi = math.atan2(d.dy, d.dx);
        _sweep += _wrapPi(phi - _phi);
        _phi = phi;
        final i = (_idx0 + (_sweep / (fine ? 2 * math.pi / 3 : 2 * math.pi / 9)).round()).clamp(0, BlurValues.stops.length - 1);
        v.set('blades', BlurValues.stops[i].toDouble());
    }
  }

  @override
  List<String> resetIds(String? zone) => switch (zone) {
        'body' => ['radius'],
        'tip' => ['angle', 'streak'],
        'rim' => ['blades'],
        _ => v.ids.toList(),
      };

  @override
  bool tick(double dt, PhCtx x, Size s) {
    final b = (v as BlurValues).blades;
    if (!_seeded) {
      _r.x = v['radius'];
      _st.x = v['streak'];
      _ang = v['angle'];
      _n = _prevN = b;
      _seeded = true;
    }
    if (b != _n) {
      _prevN = _n;
      _n = b;
      _fade = 0;
      _pop.x = 1.09;
    }
    _fade = math.min(1, _fade + dt / .16);
    _r.t = v['radius'];
    _st.t = v['streak'];
    _pop.t = 1;
    _r.step(dt, k: 560, c: 27);
    _st.step(dt, k: 560, c: 27);
    _pop.step(dt, k: 700, c: 24);
    final diff = ((v['angle'] - _ang + 90) % 180 + 180) % 180 - 90;
    _ang += diff * (1 - math.exp(-dt * 20));
    return true; // the idle shimmer never rests
  }

  @override
  List<PhRead> readout() => [
        PhRead(v.fmt('radius'), const {'body'}, v.changed('radius')),
        PhRead(v.fmt('blades'), const {'rim'}, v.changed('blades')),
        PhRead(v.fmt('angle'), const {'tip'}, v.changed('angle')),
        PhRead(v.fmt('streak'), const {'tip'}, v.changed('streak')),
      ];

  Path _poly(int n, double r, double st, double ang, Offset c) {
    final m = n == 0 ? 56 : n, path = Path();
    final ca = math.cos(ang), sa = math.sin(ang);
    for (var i = 0; i < m; i++) {
      final th = i * 2 * math.pi / m, lx = r * math.cos(th) * st, ly = r * math.sin(th);
      final pt = c + Offset(lx * ca - ly * sa, lx * sa + ly * ca);
      i == 0 ? path.moveTo(pt.dx, pt.dy) : path.lineTo(pt.dx, pt.dy);
    }
    return path..close();
  }

  @override
  void paint(Canvas c, Size s, PhCtx x) {
    _lattice(c, s);
    final shim = x.dragging ? 0.0 : 1.0;
    final ctr = _c(s) + Offset(math.sin(x.time * .7) * 2.2, math.cos(x.time * .9) * 1.6) * shim, light = _light;
    final r = _rpx(s, _r.x.clamp(0, 44)) * _pop.x, st = _stretch(_st.x.clamp(0, 1.1)), a = _ang * math.pi / 180;
    final maxLen = s.width * .46;
    final stc = math.min(st, maxLen / r).clamp(1.0, 6.0).toDouble();
    final amt = (_r.x / 6).clamp(0.0, 1.0).toDouble(); // the smear fades in from a pinpoint

    void shape(int n, double alpha) {
      final path = _poly(n, r, stc, a, ctr);
      c.drawPath(path, _fill(_al(light, .075 * alpha * amt)));
      c.drawPath(path, Paint()..style = PaintingStyle.stroke..strokeWidth = 3..color = _al(light, .16 * alpha * amt)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2.4));
      c.drawPath(path, _hair(_al(light, .62 * alpha * amt)));
    }

    // a few far lights out of focus, each breathing at its own pace
    for (var i = 0; i < 5; i++) {
      final bp = Offset(s.width * (.12 + .19 * i + .03 * math.sin(i * 2.1)), s.height * (i.isEven ? .22 : .78) + 4 * math.cos(i * 1.7));
      final br = 2.2 + 1.8 * (.5 + .5 * math.sin(x.time * (.8 + .17 * i) + i * 1.9));
      c.drawCircle(bp, br, _hair(_al(N.g63, .22)));
    }
    // the smear trail: ghosts of the aperture left behind along the line of the pull, drifting and fading
    {
      final dir = Offset(math.cos(a), math.sin(a));
      for (var i = 4; i >= 1; i--) {
        final f = i / 4, off = dir * (-(r * .55 * i) + math.sin(x.time * .8 + i) * 2.0 * shim);
        var gp = ctr + off;
        gp = Offset(gp.dx.clamp(r * .5, s.width - r * .5).toDouble(), gp.dy.clamp(r * .5, s.height - r * .5).toDouble());
        final gpath = _poly(_n, r * (1 - .1 * f), stc, a, gp);
        c.drawPath(gpath, _fill(_al(light, .05 * (1 - f * .6) * amt)));
        c.drawPath(gpath, _hair(_al(light, .34 * (1 - f * .8) * amt)));
      }
    }
    if (_fade < 1) shape(_prevN, 1 - _fade);
    shape(_n, _fade);

    // the streak: a thin line of light along the axis (anamorphic flare), only as the light is pulled
    final stk = ((stc - 1) / 3).clamp(0.0, 1.0).toDouble();
    if (stk > .02) {
      final dir = Offset(math.cos(a), math.sin(a)), len = r * stc * 1.08;
      c.drawLine(
          ctr - dir * len,
          ctr + dir * len,
          Paint()
            ..strokeWidth = 1.2
            ..shader = ui.Gradient.linear(ctr - dir * len, ctr + dir * len, [_al(light, 0), _al(light, .6 * stk), _al(light, 0)], [0, .5, 1]));
    }
    // the point of light itself
    final tw = 1 + .14 * math.sin(x.time * 2.3) + .07 * math.sin(x.time * 5.1);
    c.drawCircle(ctr, 5.5 * tw, _fill(_al(light, .22)));
    c.drawCircle(ctr, 2.1 * tw, _fill(N.g100));

    // hover marks
    final h = x.hot;
    if (h == 'body') c.drawCircle(ctr, 11, _hair(Role.selected));
    if (h == 'rim') c.drawPath(_poly(_n, r, stc, a, ctr), _hair(Role.selected));
    if (h == 'tip' || (h == null && x.hover && r > 10)) {
      final dir = Offset(math.cos(a), math.sin(a)), mk = _hair(h == 'tip' ? Role.selected : _al(N.g63, .5));
      c.drawCircle(ctr + dir * r * stc, 4, mk);
      c.drawCircle(ctr - dir * r * stc, 4, mk);
    }
  }
}
