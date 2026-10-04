part of 'pheno_a.dart';

/// 5. Falloff: strength + shape + size + curve = a field of dots, a region you reshape, and the falloff profile drawn under it in the same picture.
/// Pull the region's edge (size), push inside it (strength), bend the profile (curve), pick one of four small region sketches (shape).
class _FalloffSpec extends PhenoSpec {
  const _FalloffSpec();
  @override
  String get name => '5 Falloff';
  @override
  String get word => 'falloff';
  @override
  String get caption => 'Falloff: strength + shape + size + curve. Silhouette: a field of dots that swell inside a region outline, a thin profile curve underneath.';
  @override
  double get rowHeight => 128;
  @override
  List<PhenoParam> get params => const [
        PhenoParam('strength', 'Strength', 0, 100, 100, unit: '%'),
        PhenoParam('shape', 'Shape', 0, 3, 0),
        PhenoParam('size', 'Size', 10, 300, 140, unit: 'px'),
        PhenoParam('curve', 'Curve', -1, 1, 0, digits: 2),
      ];
  @override
  PhenoSim newSim() => _FalloffSim();
  @override
  String fmt(PhenoParam p, double v) => switch (p.id) {
        'shape' => shapes[v.round()],
        'curve' => '^${math.pow(2, 2 * v).toStringAsFixed(2)}',
        _ => p.fmt(v),
      };

  static const shapes = ['circle', 'rect', 'linear', 'sweep'];

  static Rect area(Size s) => _cr(s);
  static Offset centre(Size s) => area(s).center;
  /// Screen length of the region's "radius" (Size 300 maps to about half the field's height, so the default region sits whole inside the row).
  static double unitPx(Size s) => area(s).height * .52;
  static double rs(Size s, PhenoValues v) => v['size'] / 300 * unitPx(s);
  static double expo(PhenoValues v) => math.pow(2, 2 * v['curve']).toDouble();
  static double profile(PhenoValues v, double t) => v['strength'] / 100 * math.pow(1 - t, expo(v));

  /// 0 at the region's heart, 1 at its edge, beyond = outside. Per shape.
  static double t(Size s, PhenoValues v, Offset p) {
    final c = centre(s), r = rs(s, v), d = p - c;
    switch (v['shape'].round()) {
      case 0:
        return d.distance / r;
      case 1:
        return math.max(d.dx.abs() / (r * 1.5), d.dy.abs() / (r * .95));
      case 2:
        return math.max(0, (p.dx - (c.dx - r * 1.6)) / (r * 3.2));
      default:
        return math.atan2(d.dx, -d.dy).abs() / math.max(.15, v['size'] / 300 * math.pi);
    }
  }

  static double infl(Size s, PhenoValues v, Offset p) {
    final tt = t(s, v, p);
    return tt >= 1 ? 0 : profile(v, tt);
  }

  /// Pixels from the region's edge, per shape (the edge is a drawn line, so the distance is to that line).
  static double edgeDist(Size s, PhenoValues v, Offset p) {
    final c = centre(s), r = rs(s, v), d = p - c;
    switch (v['shape'].round()) {
      case 0:
        return (d.distance - r).abs();
      case 1:
        return (t(s, v, p) - 1).abs() * r * .95;
      case 2:
        return (p.dx - (c.dx + r * 1.6)).abs();
      default:
        return (math.atan2(d.dx, -d.dy).abs() - math.max(.15, v['size'] / 300 * math.pi)).abs() * d.distance;
    }
  }

  /// A measure that grows with Size, in the same screen units as [rs].
  static double measure(Size s, PhenoValues v, Offset p) {
    final c = centre(s), d = p - c;
    switch (v['shape'].round()) {
      case 0:
        return d.distance;
      case 1:
        return d.dx.abs() / 1.5;
      case 2:
        return d.dx / 1.6;
      default:
        return math.atan2(d.dx, -d.dy).abs() / math.pi * unitPx(s);
    }
  }

  static Offset edgeHandle(Size s, PhenoValues v) {
    final h = _rawEdge(s, v), a = area(s).deflate(5);
    return Offset(h.dx.clamp(a.left, a.right), h.dy.clamp(a.top, a.bottom));
  }

  static Offset _rawEdge(Size s, PhenoValues v) {
    final c = centre(s), r = rs(s, v);
    return switch (v['shape'].round()) {
      0 => c + Offset(r, 0),
      1 => c + Offset(r * 1.5, 0),
      2 => c + Offset(r * 1.6, 0),
      _ => c + Offset(math.sin(math.max(.15, v['size'] / 300 * math.pi)), -math.cos(math.max(.15, v['size'] / 300 * math.pi))) * (unitPx(s) * .9),
    };
  }

  /// Where the profile has fallen to half (0 heart .. 1 edge): the curve shows as how the contour lines crowd.
  static double tHalf(PhenoValues v) => 1 - math.pow(.5, 1 / expo(v)).toDouble();

  /// A point on the contour at [t], on the heart's left side (the edge handle is on its right).
  static Offset onContour(Size s, PhenoValues v, double t) {
    final c = centre(s), r = rs(s, v);
    switch (v['shape'].round()) {
      case 0:
        return c + Offset(-r * t, 0);
      case 1:
        return c + Offset(-r * 1.5 * t, 0);
      case 2:
        return Offset(c.dx - r * 1.6 + r * 3.2 * t, c.dy + r * .55);
      default:
        final a = math.max(.15, v['size'] / 300 * math.pi) * t;
        return c + Offset(-math.sin(a), -math.cos(a)) * (unitPx(s) * .9);
    }
  }

  static Offset curveHandle(Size s, PhenoValues v) => onContour(s, v, tHalf(v));
  static const trayPitch = 26.0;
  static Rect tray(Size s, int i) => Rect.fromCenter(center: Offset(_cr(s).right - 13 - (3 - i) * trayPitch, _cr(s).top + 11), width: trayPitch, height: 24);

  @override
  String? zoneAt(Size s, Offset p, PhenoValues v) {
    for (var i = 0; i < 4; i++) {
      if (tray(s, i).contains(p)) return 'shape$i';
    }
    if ((p - curveHandle(s, v)).distance < 13) return 'curve';
    final a = area(s).inflate(6);
    if (a.contains(p)) {
      if (edgeDist(s, v, p) < 11) return 'edge';
      if (t(s, v, p) < 1) return 'core';
    }
    return null;
  }

  @override
  Map<String, double> drag(String zone, Size s, Offset p0, Offset p, PhenoValues v0) {
    switch (zone) {
      case 'edge':
        return {'size': v0['size'] + (measure(s, v0, p) - measure(s, v0, p0)) * 300 / unitPx(s)};
      case 'core':
        return {'strength': v0['strength'] + (p0.dy - p.dy) / (area(s).height * .8) * 100};
      case 'curve':
        // The bead rides the contour where the profile is half: move it in or out and the exponent follows (eased, never to the extremes).
        final tq = (tHalf(v0) + (t(s, v0, p) - t(s, v0, p0)) * .8).clamp(.08, .92);
        final ex = math.log(.5) / math.log(1 - tq);
        return {'curve': (math.log(ex) / math.ln2 / 2).clamp(-1.0, 1.0)};
      default:
        return {};
    }
  }

  @override
  Map<String, double>? tap(String zone, PhenoValues v) => zone.startsWith('shape') ? {'shape': double.parse(zone.substring(5))} : null;

  @override
  List<String> zoneParams(String zone) => switch (zone) { 'edge' => ['size'], 'core' => ['strength'], 'curve' => ['curve'], _ => ['shape'] };
}

class _FalloffSim extends PhenoSim {
  List<_Sp> dots = [];
  List<Offset> pts = [];
  Size? lastSize;
  bool first = true;

  void layout(Size s) {
    if (lastSize == s) return;
    lastSize = s;
    final a = _FalloffSpec.area(s), sp = (a.height / 8).clamp(9.0, 14.0);
    final nc = (a.width / sp).floor(), nr = (a.height / sp).floor();
    pts = [
      for (var j = 0; j < nr; j++)
        for (var i = 0; i < nc; i++) Offset(a.center.dx + (i - (nc - 1) / 2) * sp, a.center.dy + (j - (nr - 1) / 2) * sp),
    ].where((p) => !Rect.fromLTRB(_cr(s).right - 112, _cr(s).top - 4, _cr(s).right + 4, _cr(s).top + 26).contains(p)).toList(); // keep the shape sketches clear
    dots = List.generate(pts.length, (_) => _Sp());
    first = true;
  }

  @override
  bool step(double dt, Size s, PhenoCtx c) {
    layout(s);
    final cen = _FalloffSpec.centre(s), w = _FalloffSpec.area(s).width;
    for (var i = 0; i < pts.length; i++) {
      final tg = _FalloffSpec.infl(s, c.v, pts[i]);
      if (first) dots[i].x = tg;
      // Far dots answer a little later (softer springs): a change travels outward from the heart, with a small settle.
      dots[i].step(tg, dt, k: 300 / (1 + 1.4 * (pts[i] - cen).distance / w), z: .55);
    }
    first = false;
    return true; // contours breathe at rest
  }

  /// The region's outline scaled to [t] (1 = the edge), clipped to the field by the caller.
  void outline(Canvas cv, Size s, PhenoValues v, double t, Paint ep, {Paint? back}) {
    final a = _FalloffSpec.area(s), cen = _FalloffSpec.centre(s), r = _FalloffSpec.rs(s, v) * t;
    switch (v['shape'].round()) {
      case 0:
        cv.drawCircle(cen, r, ep);
      case 1:
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: cen, width: r * 3, height: r * 1.9), Radius.circular(3 * t)), ep);
      case 2:
        final x = cen.dx - _FalloffSpec.rs(s, v) * 1.6 + _FalloffSpec.rs(s, v) * 3.2 * t;
        cv.drawLine(Offset(x, a.top), Offset(x, a.bottom), ep);
      default:
        final half = math.max(.15, v['size'] / 300 * math.pi) * t, len = a.height * 2;
        for (final sg in const [-1, 1]) {
          cv.drawLine(cen, cen + Offset(math.sin(half * sg), -math.cos(half * sg)) * len, ep);
        }
    }
  }

  @override
  void paint(Canvas cv, Size s, PhenoCtx c) {
    layout(s);
    final v = c.v, a = _FalloffSpec.area(s), cen = _FalloffSpec.centre(s), shape = v['shape'].round();
    final p = Paint();
    for (var i = 0; i < pts.length; i++) {
      final f = dots[i].x.clamp(0.0, 1.0);
      p.color = Color.lerp(N.g38, N.g95, f)!.withValues(alpha: .9);
      cv.drawCircle(pts[i], .9 + 2.3 * f, p);
    }
    cv.save();
    cv.clipRect(a.inflate(2));
    // Terrain: contour lines of the falloff, at influence 25 / 50 / 75 %. Where they crowd, the profile is steep (the curve, seen as a landscape).
    final st = v['strength'] / 100;
    for (final lv in const [.25, .5, .75]) {
      if (lv >= st) continue;
      final t = 1 - math.pow(lv / st, 1 / _FalloffSpec.expo(v)).toDouble();
      final wob = 1 + .025 * math.sin(c.t * 1.4 - lv * 5);
      outline(cv, s, v, (t * wob).clamp(0.02, 1.0), _line(N.g38.withValues(alpha: .35 + .5 * lv)));
    }
    // The region's edge.
    final em = _mark(c, 'edge');
    outline(cv, s, v, 1, _line(em ?? N.g76.withValues(alpha: .8)));
    if (shape == 2) outline(cv, s, v, 0, _line(N.g44));
    cv.restore();
    final eh = _FalloffSpec.edgeHandle(s, v);
    cv.drawPath(Path()..moveTo(eh.dx, eh.dy - 4.5)..lineTo(eh.dx + 4.5, eh.dy)..lineTo(eh.dx, eh.dy + 4.5)..lineTo(eh.dx - 4.5, eh.dy)..close(), _fill(N.g95));
    // The bead on the half-way contour: ride it in or out to bend the profile.
    final ch = _FalloffSpec.curveHandle(s, v), cm = _mark(c, 'curve');
    cv.drawCircle(ch, 2.8, _fill(N.g76));
    if (cm != null) cv.drawCircle(ch, 8, _line(cm));
    final km = _mark(c, 'core');
    if (km != null) cv.drawCircle(cen, 5, _line(km));
    // Four small region sketches: the shape. Quiet, and secondary to the field.
    for (var i = 0; i < 4; i++) {
      final b = _FalloffSpec.tray(s, i).center, on = i == shape, tm = _mark(c, 'shape$i');
      final col = on ? N.g95 : (tm ?? N.g56);
      switch (i) {
        case 0:
          cv.drawCircle(b, 6, _line(col));
        case 1:
          cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: b, width: 15, height: 10), const Radius.circular(2)), _line(col));
        case 2:
          for (var k = 0; k < 5; k++) {
            cv.drawLine(b + Offset(-6 + k * 3, -6), b + Offset(-6 + k * 3, 6 - k * 1.6 - 1), _line(col));
          }
        default:
          cv.drawPath(Path()..moveTo(b.dx, b.dy + 6)..lineTo(b.dx - 6.5, b.dy - 6)..moveTo(b.dx, b.dy + 6)..lineTo(b.dx + 6.5, b.dy - 6), _line(col));
      }
      if (on) cv.drawLine(b + const Offset(-5, 10), b + const Offset(5, 10), _line(_hot));
    }
  }
}
