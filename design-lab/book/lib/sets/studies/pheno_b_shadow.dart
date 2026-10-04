// B3 Shadow and sun: direction + distance + softness become a small block, its shadow, and a sun you carry around it. Where the sun is decides the direction and the length
// (a low, far sun throws a long shadow); the sun's aura is the softness (a bigger sun makes a softer edge) and you grow it by pulling its rim.
part of 'pheno_b.dart';

class ShadowValues extends PhenoValues {
  ShadowValues()
      : super(const [
          PhSpec('direction', 0, 360, 135, unit: '°'),
          PhSpec('distance', 0, 48, 12, unit: 'px'),
          PhSpec('softness', 0, 32, 8, unit: 'px'),
        ]);
  double get direction => this['direction'];
  double get distance => this['distance'];
  double get softness => this['softness'];
}

(PhenoValues, PhenoSim Function(PhenoValues)) _makeShadow() => (ShadowValues(), (v) => ShadowSim(v));

class ShadowSim extends PhenoSim {
  ShadowSim(super.v);
  final _ox = Spr(0), _oy = Spr(0), _sg = Spr(8);
  bool _seeded = false;
  Offset _grab = Offset.zero, _sun0 = Offset.zero, _p0 = Offset.zero;
  double _a0 = 0, _v0 = 0;
  final _v0s = <String, double>{};

  double _s(Size s) => math.min(s.width, s.height) / 200;
  double _u(Size s) => math.max(.7, _s(s));
  Offset _c(Size s) => Offset(s.width * .5, s.height * .52);
  double _rMin(Size s) => 34 * _s(s);
  double _rMax(Size s) => math.min(s.width, s.height) * .44;
  double _k(Size s) => (_rMax(s) - _rMin(s)) / 48;
  Offset _sunAt(Size s, double dir, double dist) {
    final a = (dir + 180) * math.pi / 180;
    return _c(s) + Offset(math.cos(a), math.sin(a)) * (_rMin(s) + dist * _k(s));
  }

  double _ring(Size s, double soft) => (14 + soft * .45) * _u(s);

  @override
  String? zoneAt(Offset p, Size s) {
    final sun = _sunAt(s, v['direction'], v['distance']), d = (p - sun).distance;
    if (d < 11) return 'sun';
    if ((d - _ring(s, v['softness'])).abs() < 8) return 'softness';
    return null;
  }

  @override
  void down(String zone, Offset p, Size s) {
    _sun0 = _sunAt(s, v['direction'], v['distance']);
    _p0 = p;
    _grab = p - _sun0;
    _v0s
      ..['direction'] = v['direction']
      ..['distance'] = v['distance'];
    _a0 = ((p - _sun0).distance - 14 * _u(s)) / (.45 * _u(s));
    _v0 = v['softness'];
  }

  @override
  void drag(String zone, Offset p, Offset delta, Size s, {required bool fine, required bool alt}) {
    if (zone == 'sun') {
      final target = fine ? _sun0 + (p - _p0) * .1 : p - _grab;
      final q = target - _c(s), r = q.distance;
      if (r > 4) v.set('direction', ((math.atan2(q.dy, q.dx) * 180 / math.pi + 180) % 360 + 360) % 360);
      v.set('distance', (r - _rMin(s)) / _k(s));
    } else {
      final sun = _sunAt(s, _v0s['direction']!, _v0s['distance']!);
      final a = ((p - sun).distance - 14 * _u(s)) / (.45 * _u(s));
      v.set('softness', _fine(a, _a0, _v0, fine));
    }
  }

  @override
  List<String> resetIds(String? zone) => switch (zone) {
        'sun' => ['direction', 'distance'],
        'softness' => ['softness'],
        _ => v.ids.toList(),
      };

  @override
  bool tick(double dt, PhCtx x, Size s) {
    final a = v['direction'] * math.pi / 180, d = v['distance'] * _s(s);
    if (!_seeded) {
      _ox.x = math.cos(a) * d;
      _oy.x = math.sin(a) * d;
      _sg.x = v['softness'];
      _seeded = true;
    }
    _ox.t = math.cos(a) * d;
    _oy.t = math.sin(a) * d;
    _sg.t = v['softness'];
    for (final p in [_ox, _oy, _sg]) {
      p.step(dt, k: 520, c: 26);
    }
    return _ox.moving(.03) || _oy.moving(.03) || _sg.moving(.03) || x.hover;
  }

  @override
  List<PhRead> readout() => [
        PhRead(v.fmt('direction'), const {'sun'}, v.changed('direction')),
        PhRead(v.fmt('distance'), const {'sun'}, v.changed('distance')),
        PhRead(v.fmt('softness'), const {'softness'}, v.changed('softness')),
      ];

  @override
  void paint(Canvas c, Size s, PhCtx x) {
    final k = _s(s), u = _u(s), ctr = _c(s), warm = _light;
    // the floor: a dotted tile
    final ground = RRect.fromRectAndRadius(Rect.fromLTWH(6, 6, s.width - 12, s.height - 12), const Radius.circular(4));
    c.drawRRect(ground, _fill(N.g13));
    c.save();
    c.clipRRect(ground);
    _lattice(c, s, a: .9);
    final half = 17 * k;
    final shape = RRect.fromRectAndRadius(Rect.fromCenter(center: ctr, width: half * 2, height: half * 2), Radius.circular(4 * k));
    final grow = 1 + v['distance'] * .004;
    // contact shadow, then the cast one
    c.drawRRect(shape.shift(Offset(0, 1.2 * k)), Paint()..color = _al(N.g00, .5)..maskFilter = MaskFilter.blur(BlurStyle.normal, 1.6 * k));
    final sigma = math.max(.05, _sg.x.clamp(0, 36) * .5 * k);
    final cast = RRect.fromRectAndRadius(Rect.fromCenter(center: ctr + Offset(_ox.x, _oy.x), width: half * 2 * grow, height: half * 2 * grow), Radius.circular(4 * k));
    c.drawRRect(cast, Paint()..color = _al(N.g00, .82)..maskFilter = MaskFilter.blur(BlurStyle.normal, sigma));
    c.restore();
    c.drawRRect(ground, _hair(N.g20));

    // the block: a raised top, lit on the side that faces the sun
    final sun = _sunAt(s, v['direction'], v['distance']);
    final toSun = (sun - ctr) / (sun - ctr).distance;
    c.drawRRect(shape, Paint()..shader = ui.Gradient.linear(shape.outerRect.topLeft, shape.outerRect.bottomRight, [N.g44, N.g26]));
    c.drawRRect(shape, _hair(N.g63));
    c.drawRRect(shape.deflate(4 * k), _hair(_al(N.g100, .07)));
    c.drawCircle(ctr, 2 * k, _fill(N.g20));
    for (final e in [(const Offset(0, -1), 0), (const Offset(0, 1), 1), (const Offset(-1, 0), 2), (const Offset(1, 0), 3)]) {
      final w = e.$1.dx * toSun.dx + e.$1.dy * toSun.dy;
      if (w < .25) continue;
      final pa = _al(warm, (w * .95).clamp(0.0, 1.0).toDouble());
      final r = shape.outerRect.deflate(1.2);
      switch (e.$2) {
        case 0:
          c.drawLine(r.topLeft + Offset(3 * k, 0), r.topRight - Offset(3 * k, 0), _hair(pa));
        case 1:
          c.drawLine(r.bottomLeft + Offset(3 * k, 0), r.bottomRight - Offset(3 * k, 0), _hair(pa));
        case 2:
          c.drawLine(r.topLeft + Offset(0, 3 * k), r.bottomLeft - Offset(0, 3 * k), _hair(pa));
        default:
          c.drawLine(r.topRight + Offset(0, 3 * k), r.bottomRight - Offset(0, 3 * k), _hair(pa));
      }
    }

    // the sun: its aura is the softness; the tether to the block is faint
    final ring = _ring(s, _sg.x.clamp(0, 36));
    c.drawLine(ctr + toSun * (half * 1.5), sun - toSun * (5 * u), _hair(_al(N.g63, .22)));
    c.drawCircle(sun, ring, Paint()..shader = ui.Gradient.radial(sun, ring, [_al(warm, .34), _al(warm, 0)]));
    final spin = x.hover ? x.time * .35 : 0.0;
    for (var i = 0; i < 8; i++) {
      final a = spin + i * math.pi / 4, d = Offset(math.cos(a), math.sin(a));
      c.drawLine(sun + d * 6.2 * u, sun + d * (i.isEven ? 9 : 8) * u, _hair(_al(warm, .8)));
    }
    c.drawCircle(sun, 3.8 * u, _fill(warm));
    if (x.hot == 'sun') c.drawCircle(sun, 11.5 * u, _hair(Role.selected));
    if (x.hot == 'softness') c.drawCircle(sun, ring, _hair(Role.selected));
  }
}
