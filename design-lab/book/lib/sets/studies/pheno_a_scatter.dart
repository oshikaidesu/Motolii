part of 'pheno_a.dart';

/// 1. Scatter: amount + spread + seed = a small flock. Pinch / spread the flock (spread), pull dots out of its heart (amount), tumble the little die (seed).
class _ScatterSpec extends PhenoSpec {
  const _ScatterSpec();
  @override
  String get name => '1 Scatter';
  @override
  String get word => 'scatter';
  @override
  String get caption => 'Scatter: amount + spread + seed. Silhouette: a loose flock of dots with a tiny hollow heart and a small die.';
  @override
  double get rowHeight => 104;
  @override
  List<PhenoParam> get params => const [
        PhenoParam('amount', 'Amount', 1, 200, 36),
        PhenoParam('spread', 'Spread', 0, 400, 170, unit: 'px'),
        PhenoParam('seed', 'Seed', 0, 999, 7, prefix: '#'),
      ];
  @override
  PhenoSim newSim() => _ScatterSim();

  /// The flock lives in an ellipse that fills the well (wide rows give dots room, so they stay distinct).
  static Offset rmax(Size s) => Offset(_cr(s).width * .5 - 3, _cr(s).height * .5 - 3);

  /// 0..1 reach of the flock: spread 400 fills the well.
  static double reach(double spread) => math.sqrt(spread / 400);
  static Offset rs(Size s, double spread) => rmax(s) * reach(spread);
  static double ndist(Size s, Offset p) {
    final d = p - _cr(s).center, m = rmax(s);
    return math.sqrt(d.dx * d.dx / (m.dx * m.dx) + d.dy * d.dy / (m.dy * m.dy));
  }

  static Offset die(Size s) => Offset(_cr(s).right - 9, _cr(s).top + 9);

  @override
  String? zoneAt(Size s, Offset p, PhenoValues v) {
    if ((p - die(s)).distance < 20) return 'seed';
    if ((p - _cr(s).center).distance < 14) return 'heart';
    if (_cr(s).inflate(10).contains(p)) return 'cloud';
    return null;
  }

  @override
  Map<String, double> drag(String zone, Size s, Offset p0, Offset p, PhenoValues v0) {
    switch (zone) {
      case 'cloud':
        // Pinch / spread: the flock's edge follows the pointer's change of (normalised) distance from the centre; gently eased, never to zero.
        final f = (reach(v0['spread']) + (ndist(s, p) - ndist(s, p0)) * .8).clamp(.12, 1.0);
        return {'spread': 400 * f * f};
      case 'heart':
        // Multiplicative: 30 px of pull doubles / halves the flock, so a modest drag stays modest.
        return {'amount': v0['amount'] * math.pow(2, (p0.dy - p.dy) / 30)};
      default:
        return {'seed': v0['seed'] + ((p.dx - p0.dx) / 14).round()};
    }
  }

  @override
  Map<String, double>? tap(String zone, PhenoValues v) => zone == 'seed' ? {'seed': v['seed'] + 1} : null;

  @override
  List<String> zoneParams(String zone) => switch (zone) { 'cloud' => ['spread'], 'heart' => ['amount'], _ => ['seed'] };
}

class _ScatterSim extends PhenoSim {
  static const maxN = 200;
  final pos = List<Offset>.filled(maxN, Offset.zero), vel = List<Offset>.filled(maxN, Offset.zero);
  final pr = List<double>.filled(maxN, 0);
  final tumble = _Sp();
  bool first = true;

  static Offset unit(int seed, int i) {
    final a = _hash(seed + 1, i * 2) * math.pi * 2, r = math.sqrt(_hash(seed + 1, i * 2 + 1));
    return Offset(math.cos(a) * r, math.sin(a) * r);
  }

  @override
  bool step(double dt, Size s, PhenoCtx c) {
    final n = c.v['amount'].round(), seed = c.v['seed'].round(), r = _ScatterSpec.rs(s, c.v['spread']);
    var moving = tumble.step(seed * 2.4, dt, k: 90, z: .4);
    for (var i = 0; i < maxN; i++) {
      final on = i < n;
      var tg = Offset(unit(seed, i).dx * r.dx, unit(seed, i).dy * r.dy);
      if (c.live) tg += Offset(math.sin(c.t * 1.3 + i * 1.7), math.cos(c.t * 1.1 + i * 2.3)) * .7;
      if (first && on) {
        pos[i] = tg;
        pr[i] = 1;
      }
      pr[i] += ((on ? 1 : 0) - pr[i]) * math.min(1, dt * 10);
      if (!on && pr[i] < .02) {
        pr[i] = 0;
        pos[i] = Offset.zero; // a dot not in the flock waits in its heart, ready to be pulled out
        vel[i] = Offset.zero;
        continue;
      }
      // Underdamped: a dot settles with a small overshoot, a flock that "lands".
      vel[i] += ((tg - pos[i]) * 130 - vel[i] * 11) * dt;
      pos[i] += vel[i] * dt;
      if ((tg - pos[i]).distance > .05 || vel[i].distance > .5 || (pr[i] - (on ? 1 : 0)).abs() > .01) moving = true;
    }
    first = false;
    return moving;
  }

  @override
  void paint(Canvas cv, Size s, PhenoCtx c) {
    final cr = _cr(s), ctr = cr.center, r = _ScatterSpec.rs(s, c.v['spread']);
    final cm = _mark(c, 'cloud');
    if (cm != null) {
      // What is grabbable: the flock's edge, as a hairline in the family colour.
      cv.drawOval(Rect.fromCenter(center: ctr, width: r.dx * 2, height: r.dy * 2), _line((c.active == 'cloud' ? _hot : Role.linkedFor(Fam.scatter)).withValues(alpha: .6)));
    }
    final p = Paint();
    for (var i = 0; i < maxN; i++) {
      if (pr[i] < .02) continue;
      final a = pr[i].clamp(0.0, 1.0);
      p.color = N.g76.withValues(alpha: .85 * a);
      cv.drawCircle(ctr + pos[i], .8 + 1.0 * _hash(3, i), p);
    }
    // The heart: where dots are pulled out of.
    final hm = _mark(c, 'heart');
    cv.drawCircle(ctr, 3.5, _line(N.g63));
    if (hm != null) cv.drawCircle(ctr, 8, _line(hm));
    // The die: a tiny rounded square with three pips, it tumbles when the seed changes.
    final d = _ScatterSpec.die(s), dm = _mark(c, 'seed');
    cv.save();
    cv.translate(d.dx, d.dy);
    cv.rotate(tumble.x);
    cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 11, height: 11), const Radius.circular(2.5)), _line(N.g56));
    for (final o in const [Offset(-2.6, -2.6), Offset(0, 0), Offset(2.6, 2.6)]) {
      cv.drawCircle(o, 1, _fill(N.g76));
    }
    cv.restore();
    if (dm != null) cv.drawCircle(d, 11, _line(dm));
  }
}
