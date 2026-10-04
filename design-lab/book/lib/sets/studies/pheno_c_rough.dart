part of 'pheno_c.dart';

// Roughness: a glossy bead. Rubbing sideways scuffs it (grain pops in, the highlight spreads, the reflected light fans out); wiping up and down polishes it.
class _Dust {
  _Dust(this.p, this.v, this.life, this.glint);
  Offset p, v;
  double life;
  final bool glint;
}

class _RoughToy extends _Toy {
  _RoughToy() : super(PcDoc(const [PcParam('rough', 'Roughness', 0, 1, .25, dec: 2)])) {
    _rd = doc['rough'];
    for (var i = 0; i < _n; i++) {
      _alpha[i] = i / _n < _rd ? 1 : 0;
    }
  }
  static const _n = 420;
  late final List<Offset> _g = List.generate(_n, (i) {
    final a = _hash(i * 3 + 1) * math.pi * 2, r = math.sqrt(_hash(i * 3 + 2));
    return Offset(math.cos(a) * r, math.sin(a) * r);
  });
  final List<double> _alpha = List.filled(_n, 0);
  final _lx = _Sp(0), _ly = _Sp(0);
  final List<_Dust> _dust = [];
  late double _rd;
  int _seed = 0;

  Offset _c(Size s) => Offset(s.width * .40, s.height * .58);
  double _r(Size s) => math.min(s.height * .40, s.width * .22);

  @override
  String? zoneAt(Offset p, Size s) => (p - _c(s)).distance < _r(s) + 12 ? 'bead' : null;

  @override
  List<String> readout() => [doc.fmt('rough')];

  @override
  bool get busy => super.busy || _dust.isNotEmpty || (_rd - doc['rough']).abs() > .002 || !_lx.at(0) || !_ly.at(0) || _alpha.indexed.any((e) => (e.$2 - (e.$1 / _n < doc['rough'] ? 1 : 0)).abs() > .01);

  @override
  void drag(Offset p, Offset d, Size s, bool fine) {
    final k = (fine ? .1 : 1) / (_r(s) * 7);
    doc.set('rough', doc['rough'] + (d.dx.abs() - d.dy.abs()) * k);
    _lx.v = (_lx.v + d.dx * .3).clamp(-3.0, 3.0);
    _ly.v = (_ly.v + d.dy * .3).clamp(-3.0, 3.0);
    final dist = d.distance;
    if (dist < .5) return;
    if (d.dx.abs() >= d.dy.abs()) {
      for (var i = 0; i < (dist > 5 ? 2 : 1); i++) {
        _seed++;
        final r1 = _hash(_seed), r2 = _hash(_seed + 977);
        _dust.add(_Dust(p, Offset(-d.dx.sign * (18 + 50 * r1), -26 * r2 + 10), .5, false));
      }
    } else {
      _dust.add(_Dust(p, Offset(0, d.dy.sign * 10), .26, true));
    }
  }

  @override
  void step(double dt) {
    _rd += (doc['rough'] - _rd) * math.min(1, dt * 14);
    for (var i = 0; i < _n; i++) {
      final t = i / _n < doc['rough'] ? 1.0 : 0.0;
      _alpha[i] += (t - _alpha[i]) * math.min(1, dt * 12);
    }
    _dust.removeWhere((d) => (d.life -= dt) <= 0);
    for (final d in _dust) {
      d.p += d.v * dt;
      if (!d.glint) d.v = Offset(d.v.dx * .9, d.v.dy + 120 * dt);
    }
    _lx.to(0, dt, k: 240, d: 11);
    _ly.to(0, dt, k: 240, d: 11);
  }

  @override
  void paint(Canvas z, Size s) {
    final c = _c(s), r = _r(s), rd = _rd;
    final lit = hover != null || pressed;
    z.drawOval(Rect.fromCenter(center: c + Offset(0, r * 1.02), width: r * 1.7, height: r * .22), _fl(_a(N.g00, .5))..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3));

    final hl = c + Offset(.42 + _lx.x, -.42 + _ly.x) * r;
    final nrm = (hl - c).direction;
    z.save();
    z.clipPath(Path()..addOval(Rect.fromCircle(center: c, radius: r)));
    z.drawCircle(c, r, _fl(N.g15));
    z.drawCircle(c, r, Paint()..shader = ui.Gradient.radial(hl, r * 1.55, [N.g56, N.g26, N.g10], [0, .5, 1]));
    for (var i = 0; i < _n; i++) {
      final al = _alpha[i];
      if (al < .02) continue;
      final q = c + _g[i] * r * .97;
      final bright = i.isEven;
      z.drawRect(Rect.fromCenter(center: q, width: 1.3, height: 1.3), _fl(bright ? _a(N.g95, .5 * al) : _a(N.g00, .55 * al)));
    }
    final rr = r * (.09 + .22 * rd);
    z.drawCircle(hl, rr, _fl(_a(N.g100, .95 - .5 * rd))..maskFilter = MaskFilter.blur(BlurStyle.normal, .4 + rd * r * .1));
    z.restore();
    z.drawCircle(c, r, _ln(pressed ? _act : (lit ? _a(_hot, .7) : N.g44)));

    // the light that leaves the bead: one mirror ray when polished, a fan when rough
    final surf = c + Offset.fromDirection(nrm, r);
    final inc = nrm + .75;
    final dashEnd = surf + Offset.fromDirection(inc, r * .95);
    for (var i = 0; i < 4; i++) {
      final a0 = surf + (dashEnd - surf) * (i / 4 + .05), a1 = surf + (dashEnd - surf) * (i / 4 + .17);
      z.drawLine(a1, a0, _ln(_a(N.g63, .6)));
    }
    final mid = nrm - .75 * (1 - rd);
    final spread = .15 + rd * 1.0;
    final len = r * .95;
    z.drawLine(surf, surf + Offset.fromDirection(mid, len), _ln(_a(N.g91, 1 - rd * .6)));
    for (var k = 1; k <= 3; k++) {
      final w = (rd * 4 - (k - 1) * 1.1).clamp(0.0, 1.0);
      if (w < .02) continue;
      for (final sg in [-1, 1]) {
        final l = len * (1 - .35 * _hash(k * 7 + (sg > 0 ? 1 : 0)) * rd);
        z.drawLine(surf, surf + Offset.fromDirection(mid + sg * spread * k / 3, l), _ln(_a(N.g76, .55 * w)));
      }
    }

    for (final d in _dust) {
      final f = (d.life / (d.glint ? .26 : .5)).clamp(0.0, 1.0);
      if (d.glint) {
        z.drawLine(d.p + const Offset(-3, 0), d.p + const Offset(3, 0), _ln(_a(N.g100, .7 * f)));
      } else {
        z.drawRect(Rect.fromCenter(center: d.p, width: 1.4, height: 1.4), _fl(_a(N.g76, f)));
      }
    }
  }
}
