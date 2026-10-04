part of 'pheno_a.dart';

/// 2. Stagger: offset + direction + range = a fan of cards. Pull the tip of the fan open (offset), slide the hinge (direction), pull more cards out of the deck (range).
class _StaggerSpec extends PhenoSpec {
  const _StaggerSpec();
  @override
  String get name => '2 Stagger';
  @override
  String get word => 'stagger';
  @override
  String get caption => 'Stagger: offset + direction + range. Silhouette: a hand of playing cards fanned from one hinge, each with its own small moving line.';
  @override
  double get rowHeight => 112;
  @override
  List<PhenoParam> get params => const [
        PhenoParam('offset', 'Offset', 0, 24, 4, unit: 'f', digits: 1),
        PhenoParam('direction', 'Direction', 0, 1, .5, digits: 2),
        PhenoParam('range', 'Range', 2, 12, 7, prefix: '×'),
      ];
  @override
  PhenoSim newSim() => _StaggerSim();

  static const maxN = 12;
  static const degPerF = 1.4 * math.pi / 180;

  static int n(PhenoValues v) => v['range'].round();
  /// Angle between neighbouring cards; capped so the whole fan never opens past 150 degrees.
  static double per(PhenoValues v) => math.min(v['offset'] * degPerF, 150 * math.pi / 180 / (n(v) - 1));
  /// The card that stays upright (0 offset): direction 0 = the first, 1 = the last, between = from the middle out.
  static double lead(PhenoValues v) => v['direction'] * (n(v) - 1);
  static int far(PhenoValues v) => lead(v) < (n(v) - 1) / 2 ? n(v) - 1 : 0;
  static double len(Size s) => (_cr(s).height - 6).clamp(30.0, 150.0);
  static Offset pivot(Size s, PhenoValues v) => Offset(_cr(s).center.dx + (v['direction'] - .5) * _cr(s).width * .5, _cr(s).bottom - 2);
  static Offset tip(Size s, PhenoValues v) {
    final a = (far(v) - lead(v)) * per(v), p = pivot(s, v), l = len(s);
    return p + Offset(math.sin(a), -math.cos(a)) * l;
  }

  @override
  String? zoneAt(Size s, Offset p, PhenoValues v) {
    if ((p - pivot(s, v)).distance < 14) return 'pivot';
    if ((p - tip(s, v)).distance < 16) return 'tip';
    if (_cr(s).inflate(10).contains(p)) return 'cards';
    return null;
  }

  @override
  Map<String, double> drag(String zone, Size s, Offset p0, Offset p, PhenoValues v0) {
    switch (zone) {
      case 'pivot':
        return {'direction': v0['direction'] + (p.dx - p0.dx) / (_cr(s).width * .5)};
      case 'tip':
        final k = far(v0) - lead(v0), pv = pivot(s, v0);
        double ang(Offset q) => math.atan2(q.dx - pv.dx, -(q.dy - pv.dy));
        final a = k * per(v0) + ang(p) - ang(p0);
        return {'offset': math.max(0, a / k) / degPerF};
      default:
        return {'range': v0['range'] + ((p0.dy - p.dy) / (len(s) * .16)).round()};
    }
  }

  @override
  List<String> zoneParams(String zone) => switch (zone) { 'pivot' => ['direction'], 'tip' => ['offset'], _ => ['range'] };
}

class _StaggerSim extends PhenoSim {
  final ang = List.generate(_StaggerSpec.maxN, (_) => _Sp()), pre = List.generate(_StaggerSpec.maxN, (_) => _Sp());
  double phase = .6;
  bool first = true;

  @override
  bool step(double dt, Size s, PhenoCtx c) {
    final v = c.v, n = _StaggerSpec.n(v), per = _StaggerSpec.per(v), lead = _StaggerSpec.lead(v);
    if (c.live) phase += dt;
    var moving = false;
    for (var i = 0; i < _StaggerSpec.maxN; i++) {
      final tg = i < n ? (i - lead) * per : 0.0;
      if (first) {
        ang[i].x = tg;
        pre[i].x = i < n ? 1 : 0;
      }
      // Each card chases the one before it a little later: the fan opens with a slight overshoot.
      moving |= ang[i].step(tg, dt, k: 190 - i * 8, z: .55);
      moving |= pre[i].step(i < n ? 1 : 0, dt, k: 260, z: 1);
    }
    first = false;
    return moving;
  }

  @override
  void paint(Canvas cv, Size s, PhenoCtx c) {
    final v = c.v, n = _StaggerSpec.n(v), pv = _StaggerSpec.pivot(s, v), l = _StaggerSpec.len(s), w = l * .44;
    final cm = _mark(c, 'cards');
    for (var i = 0; i < _StaggerSpec.maxN; i++) {
      final al = pre[i].x.clamp(0.0, 1.0);
      if (al < .02) continue;
      cv.save();
      cv.translate(pv.dx, pv.dy);
      cv.rotate(ang[i].x);
      final rr = RRect.fromRectAndRadius(Rect.fromLTWH(-w / 2, -l, w, l), const Radius.circular(3));
      cv.drawRRect(rr, _fill(Color.lerp(N.g10, N.g20, i / (_StaggerSpec.maxN - 1))!.withValues(alpha: al)));
      cv.drawRRect(rr.deflate(.5), _line((cm ?? N.g44).withValues(alpha: (cm == null ? 1 : .9) * al)));
      // The card's own small line travels up its face, later for later cards: the stagger itself, in miniature.
      final f = (phase * .5 - i * v['offset'] * .02) % 1.0, y = -l * (.1 + .78 * (f < 0 ? f + 1 : f));
      cv.drawLine(Offset(-w / 2 + 4, y), Offset(w / 2 - 4, y), _line((c.live ? Role.linkedFor(Fam.stagger) : N.g38).withValues(alpha: .9 * al)));
      cv.restore();
    }
    final pm = _mark(c, 'pivot'), tm = _mark(c, 'tip');
    cv.drawCircle(pv, 2.5, _fill(N.g76));
    cv.drawCircle(pv, 5, _line(N.g44));
    if (pm != null) cv.drawCircle(pv, 9, _line(pm));
    final tp = _StaggerSpec.tip(s, v);
    if (tm != null) cv.drawCircle(tp, 7, _line(tm));
    if (n > 0 && c.live && tm == null) cv.drawCircle(tp, 2, _fill(N.g76.withValues(alpha: .7)));
  }
}
