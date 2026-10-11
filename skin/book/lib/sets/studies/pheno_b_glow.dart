// B1 Glow star: radius + intensity + threshold are ONE small star. The centre of the star is the part that is bright enough to glow (its edge IS the threshold),
// the halo's rim is the radius, the diffraction spikes are the intensity (pull the top tip up or down). Dragging the star acts on the star.
part of 'pheno_b.dart';

class GlowValues extends PhenoValues {
  GlowValues()
      : super(const [
          PhSpec('radius', 0, 64, 24, unit: 'px'),
          PhSpec('intensity', 0, 3, 1, prefix: '×', decimals: 2),
          PhSpec('threshold', 0, 1, .6, decimals: 2),
        ]);
  double get radius => this['radius'];
  double get intensity => this['intensity'];
  double get threshold => this['threshold'];
}

(PhenoValues, PhenoSim Function(PhenoValues)) _makeGlow() => (GlowValues(), (v) => GlowSim(v));

class GlowSim extends PhenoSim {
  GlowSim(super.v);
  final _rad = Spr(24), _int = Spr(1), _thr = Spr(.6);
  double _a0 = 0, _v0 = 0;

  double _r0(Size s) => math.min(s.width, s.height) * .17;
  double _k(Size s) => (math.min(s.width, s.height) * .47 - _r0(s)) / 64;
  double _spike(Size s, double i) => _r0(s) * (1 + .4 * i);
  Offset _c(Size s) => s.center(Offset.zero);

  @override
  String? zoneAt(Offset p, Size s) {
    final c = _c(s), r0 = _r0(s), d = (p - c).distance, tip = _spike(s, v['intensity']);
    if ((p - (c - Offset(0, tip))).distance < 12 || (p - (c + Offset(0, tip))).distance < 12) return 'intensity';
    if (d < r0 * .9) return 'threshold';
    if (d < r0 + v['radius'] * _k(s) + 12) return 'radius';
    return null;
  }

  /// The pointer, expressed in the value the zone edits.
  double _a(String z, Offset p, Size s) {
    final c = _c(s), r0 = _r0(s);
    return switch (z) {
      'threshold' => 1 - (p - c).distance / r0,
      'radius' => ((p - c).distance - r0) / _k(s),
      _ => ((p.dy - c.dy).abs() - r0) / (r0 * .4),
    };
  }

  @override
  void down(String zone, Offset p, Size s) {
    _a0 = _a(zone, p, s);
    _v0 = v[zone];
  }

  @override
  void drag(String zone, Offset p, Offset delta, Size s, {required bool fine, required bool alt}) =>
      v.set(zone, _fine(_a(zone, p, s), _a0, _v0, fine));

  @override
  List<String> resetIds(String? zone) => zone == null ? v.ids.toList() : [zone];

  @override
  bool tick(double dt, PhCtx x, Size s) {
    _rad.t = v['radius'];
    _int.t = v['intensity'];
    _thr.t = v['threshold'];
    for (final p in [_rad, _int, _thr]) {
      p.step(dt);
    }
    return _rad.moving(.02) || _int.moving() || _thr.moving() || x.hover;
  }

  @override
  List<PhRead> readout() => [for (final id in ['radius', 'intensity', 'threshold']) PhRead(v.fmt(id), {id}, v.changed(id))];

  @override
  void paint(Canvas c, Size s, PhCtx x) {
    _lattice(c, s);
    final ctr = _c(s), r0 = _r0(s), k = _k(s), warm = _light;
    final breath = x.hover ? 1 + .018 * math.sin(x.time * 2.4) : 1.0;
    final hr = math.max(r0 + _rad.x.clamp(0, 70) * k, r0 + 1) * breath;
    final lit = (1 - _thr.x).clamp(0.0, 1.0).toDouble(), inten = _int.x.clamp(0.0, 3.4).toDouble();
    final gain = (inten * math.pow(lit, .6) * .34).clamp(0.0, 1.0).toDouble();

    // the halo: what the lit part throws into the air
    c.drawCircle(
      ctr,
      hr,
      Paint()
        ..shader = ui.Gradient.radial(ctr, hr, [_al(warm, gain), _al(warm, gain * .32), _al(warm, 0)], [0, (r0 / hr).clamp(.05, .95), 1]),
    );

    // diffraction spikes: the vertical one is the intensity handle
    final twinkle = x.hover ? 1 + .08 * math.sin(x.time * 3.1) : 1.0;
    void spike(Offset dir, double len, double w, double a) {
      if (len < 1) return;
      final n = Offset(-dir.dy, dir.dx);
      final a0 = ctr - dir * len, a1 = ctr + dir * len;
      final path = Path()
        ..moveTo(a1.dx, a1.dy)
        ..lineTo(ctr.dx + n.dx * w, ctr.dy + n.dy * w)
        ..lineTo(a0.dx, a0.dy)
        ..lineTo(ctr.dx - n.dx * w, ctr.dy - n.dy * w)
        ..close();
      c.drawPath(path, Paint()..shader = ui.Gradient.linear(a0, a1, [_al(warm, 0), _al(warm, a), _al(warm, 0)], [0, .5, 1]));
    }

    final len = _spike(s, inten);
    final sa = (.35 + .5 * math.pow(lit, .5)) * twinkle;
    spike(const Offset(0, 1), len, 1.7, sa);
    spike(const Offset(1, 0), len * .62, 1.5, sa * .8);
    final dg = math.sqrt1_2;
    spike(Offset(dg, dg), len * .34, 1.0, sa * .45);
    spike(Offset(dg, -dg), len * .34, 1.0, sa * .45);

    // the star itself: dim facets; what is above the threshold is lit
    final star = Path();
    for (var i = 0; i < 16; i++) {
      final a = -math.pi / 2 + i * math.pi / 8, r = i.isEven ? r0 : r0 * .46;
      final pt = ctr + Offset(math.cos(a), math.sin(a)) * r;
      i == 0 ? star.moveTo(pt.dx, pt.dy) : star.lineTo(pt.dx, pt.dy);
    }
    star.close();
    c.drawPath(star, Paint()..shader = ui.Gradient.radial(ctr, r0, [N.g56, N.g20], [0, 1]));
    for (var i = 0; i < 8; i++) {
      final a = -math.pi / 2 + i * math.pi / 4;
      c.drawLine(ctr, ctr + Offset(math.cos(a), math.sin(a)) * r0, _hair(_al(N.g100, .07)));
    }
    c.drawPath(star, _hair(N.g44));
    final core = r0 * lit;
    if (core > .5) {
      c.save();
      c.clipPath(star);
      c.drawCircle(ctr, core, Paint()..shader = ui.Gradient.radial(ctr, core, [N.g100, _al(warm, .78)], [.2, 1]));
      c.restore();
      c.drawCircle(ctr, core, _hair(_al(N.g100, .38)));
    }

    // hover: the one grabbable part lights up with a 1 px mark
    final h = x.hot;
    if (h != null) {
      final mark = _hair(Role.selected);
      switch (h) {
        case 'threshold':
          c.drawCircle(ctr, math.max(core, 3), mark);
        case 'radius':
          c.drawCircle(ctr, hr, mark);
        case 'intensity':
          c.drawCircle(ctr - Offset(0, len), 3.5, mark);
          c.drawCircle(ctr + Offset(0, len), 3.5, mark);
      }
    }
  }
}
