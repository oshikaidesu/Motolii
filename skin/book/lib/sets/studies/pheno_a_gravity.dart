part of 'pheno_a.dart';

/// 6. Attractor / Gravity: strength + direction = a small well with orbiting dust. Swing the well around its home (direction), pull its outer contour (strength).
class _GravitySpec extends PhenoSpec {
  const _GravitySpec();
  @override
  String get name => '6 Gravity';
  @override
  String get word => 'gravity';
  @override
  String get caption => 'Gravity: strength + direction. Silhouette: three spiral arms of dust turning around a bright speck, a hairline back to home.';
  @override
  double get rowHeight => 104;
  @override
  List<PhenoParam> get params => const [
        PhenoParam('strength', 'Strength', 0, 100, 40, unit: '%'),
        PhenoParam('direction', 'Direction', 0, 360, 270, unit: '°'),
      ];
  @override
  PhenoSim newSim() => _GravitySim();

  static Offset home(Size s) => Offset(_cr(s).center.dx, _cr(s).top + _cr(s).height * .42);
  static double rh(Size s) => math.min(_cr(s).height * .2, _cr(s).width * .2);
  static double rimMax(Size s) => _cr(s).height * .44;
  /// 0 = right, counter-clockwise (screen y is flipped).
  static Offset well(Size s, double deg) {
    final a = deg * math.pi / 180;
    return home(s) + Offset(math.cos(a), -math.sin(a)) * rh(s);
  }

  static double rim(Size s, double strength) => rimMax(s) * (.25 + .75 * math.sqrt(strength / 100));

  @override
  String? zoneAt(Size s, Offset p, PhenoValues v) {
    final w = well(s, v['direction']);
    if ((p - w).distance < 14) return 'well';
    if (((p - w).distance - rim(s, v['strength'])).abs() < 12) return 'rim';
    return null;
  }

  @override
  Map<String, double> drag(String zone, Size s, Offset p0, Offset p, PhenoValues v0) {
    if (zone == 'well') {
      final h = home(s);
      double ang(Offset q) => math.atan2(-(q.dy - h.dy), q.dx - h.dx) * 180 / math.pi;
      return {'direction': ((v0['direction'] + ang(p) - ang(p0)) % 360 + 360) % 360};
    }
    final w = well(s, v0['direction']);
    final nr = (rim(s, v0['strength']) + (p - w).distance - (p0 - w).distance) / rimMax(s);
    final f = ((nr - .25) / .75).clamp(0.0, 1.0);
    return {'strength': 100 * f * f};
  }

  @override
  List<String> zoneParams(String zone) => zone == 'well' ? ['direction'] : ['strength'];
}

class _GravitySim extends PhenoSim {
  static const n = 42, arms = 3;
  final pos = List<Offset>.filled(n, Offset.zero), prev = List<Offset>.filled(n, Offset.zero);
  final rho = List<double>.generate(n, (i) => .12 + .88 * math.sqrt(_hash(6, i)));
  double turn = 0;
  bool first = true;

  /// How tightly the arms wind: a stronger well curls them in more.
  static double twist(double strength) => 1.6 + 2.4 * strength / 100;

  /// A point on arm [k] at fraction [u] (0 at the well, 1 at the rim).
  static Offset armPoint(Offset w, double rim, double tw, double turn, int k, double u) {
    final a = turn + k * math.pi * 2 / arms + tw * u;
    return w + Offset(math.cos(a), -math.sin(a)) * (rim * u);
  }

  Offset target(Size s, PhenoValues v, int i) {
    final w = _GravitySpec.well(s, v['direction']), rim = _GravitySpec.rim(s, v['strength']);
    final k = i % arms, jitter = (_hash(8, i) - .5) * .5;
    return armPoint(w, rim, twist(v['strength']), turn + jitter, k, rho[i]);
  }

  @override
  bool step(double dt, Size s, PhenoCtx c) {
    // The whole spiral turns only while the pointer is near, faster for a stronger well.
    if (c.live) turn += dt * (.25 + .9 * c.v['strength'] / 100);
    var m = false;
    for (var i = 0; i < n; i++) {
      final tg = target(s, c.v, i);
      if (first) pos[i] = tg;
      prev[i] = pos[i];
      // A soft follow: when the well is swung, the dust trails after it like a comet's tail.
      pos[i] = Offset.lerp(pos[i], tg, math.min(1, dt * 7))!;
      if ((tg - pos[i]).distance > .06) m = true;
    }
    first = false;
    return m;
  }

  @override
  void paint(Canvas cv, Size s, PhenoCtx c) {
    final v = c.v, h = _GravitySpec.home(s), w = _GravitySpec.well(s, v['direction']), rim = _GravitySpec.rim(s, v['strength']), tw = twist(v['strength']);
    final rm = _mark(c, 'rim');
    // The arms: three hairlines winding out of the well.
    for (var k = 0; k < arms; k++) {
      final path = Path();
      for (var q = 0; q <= 40; q++) {
        final pt = armPoint(w, rim, tw, turn, k, q / 40);
        q == 0 ? path.moveTo(pt.dx, pt.dy) : path.lineTo(pt.dx, pt.dy);
      }
      cv.drawPath(path, _line(N.g26));
      final tip = armPoint(w, rim, tw, turn, k, 1);
      cv.drawCircle(tip, 1.8, _fill(rm ?? N.g56));
      if (rm != null) cv.drawCircle(tip, 6, _line(rm));
    }
    // The tether: a hairline from home to the well.
    cv.drawLine(h, w, _line(N.g26));
    cv.drawCircle(h, 1.5, _fill(N.g44));
    final p = Paint();
    for (var i = 0; i < n; i++) {
      final a = (.55 + .4 * (1 - rho[i])).clamp(0.0, 1.0);
      if (c.live) cv.drawLine(pos[i], pos[i] + (pos[i] - prev[i]) * -3, _line(N.g63.withValues(alpha: .4 * a)));
      p.color = N.g76.withValues(alpha: a);
      cv.drawCircle(pos[i], 1.1 + .6 * _hash(7, i), p);
    }
    // The well: a bright speck in the attach hue, with a ring when it can be grabbed.
    final wm = _mark(c, 'well');
    cv.drawCircle(w, 2.8, _fill(Role.linkedFor(Fam.attach)));
    if (wm != null) cv.drawCircle(w, 10, _line(wm));
  }
}
