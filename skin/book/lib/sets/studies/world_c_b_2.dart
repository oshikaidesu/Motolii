// Part of world_c_b.dart (cut there by size, as it was).
part of 'world_c_b.dart';

/// A: the umbra edge of the shadow (distance, pull along the shadow). B: the little sun (angle, carry it around). C: the penumbra edge
/// (softness, move it out / in). D: the hatching inside the shadow (opacity, up / down).
class ShadowWorld extends WWorld {
  ShadowWorld(super.doc);
  @override
  String get title => 'Shadow';
  @override
  String get silhouette => 'sundial';

  Offset _c(Size s) => Offset(s.width / 2, s.height / 2 - 4);
  double _half(Size s) => wcClamp(s.height * .14, 10, 28);
  double _offOf(Size s, double d) => s.height * .26 * math.sqrt(d / 100);
  double _spOf(double soft) => 3 + soft / 100 * 34;
  Offset get _dirS {
    final a = v('shadow.angle') * math.pi / 180;
    return Offset(math.sin(a), -math.cos(a));
  }

  double get _breath => 1 + .012 * math.sin(t * 1.3);
  Offset _sc(Size s) => _c(s) + _dirS * (_offOf(s, v('shadow.distance')) * _breath);
  Offset _sun(Size s) => _c(s) - _dirS * (math.min(s.width, s.height) * .40);

  static double _sd(Offset q, double h, double r) {
    final d = Offset(q.dx.abs() - (h - r), q.dy.abs() - (h - r));
    return Offset(math.max(d.dx, 0), math.max(d.dy, 0)).distance + math.min(math.max(d.dx, d.dy), 0) - r;
  }

  @override
  int? zoneAt(Offset p, Size s) {
    if ((p - _sun(s)).distance <= 14) return 1;
    final sd = _sd(p - _sc(s), _half(s), 3), sp = _spOf(v('shadow.softness')) * (1 + .03 * math.sin(t * 1.1));
    final da = sd.abs(), dc = (sd - sp).abs();
    if (da <= 10 && da <= dc) return 0;
    if (dc <= 10) return 2;
    if (sd < -8) return 3;
    return null;
  }

  @override
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start) {
    final c = _c(s);
    switch (z) {
      case 0:
        final o = _offOf(s, start['shadow.distance']!) + acc.dx * _dirS.dx + acc.dy * _dirS.dy;
        final f = math.max(0.0, o) / (s.height * .26);
        doc.set('shadow.distance', 100 * f * f);
      case 1:
        doc.set('shadow.angle', start['shadow.angle']! + angDelta(c, p) * 180 / math.pi);
      case 2:
        final sc = _sc(s);
        final dsd = _sd(p - sc, _half(s), 3) - _sd(drag0 - sc, _half(s), 3);
        doc.set('shadow.softness', (_spOf(start['shadow.softness']!) + dsd - 3) / 34 * 100);
      case _:
        doc.set('shadow.opacity', start['shadow.opacity']! - acc.dy / 80 * 100);
    }
  }

  @override
  void paint(Canvas cv, Size s) {
    final c = _c(s), h = _half(s), sc = _sc(s), op = v('shadow.opacity') / 100;
    final sp = _spOf(v('shadow.softness')) * (1 + .03 * math.sin(t * 1.1));
    // the ground: a quiet field of dots (the shadow swallows them), no fill
    final gd = <Offset>[];
    for (double y = 7; y < s.height; y += 12) {
      for (double x = 7; x < s.width; x += 12) {
        gd.add(Offset(x, y));
      }
    }
    cv.drawPoints(ui.PointMode.points, gd, Paint()
      ..color = N.g26.withValues(alpha: .55)
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round);
    // the shadow: a blurred dark shape
    final rr = RRect.fromRectAndRadius(Rect.fromCenter(center: sc, width: h * 2, height: h * 2), const Radius.circular(3));
    cv.drawRRect(rr, Paint()
      ..color = N.g00.withValues(alpha: op)
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, math.max(.5, sp * .45)));
    // D: hatching inside the shadow, denser when darker
    cv.save();
    cv.clipRRect(rr);
    final gap = 3 + (1 - op) * 9;
    final hz = zc(3).withValues(alpha: lit(3) ? .9 : .5);
    for (double u = -h * 2; u <= h * 2; u += gap) {
      cv.drawLine(sc + Offset(u - h, h), sc + Offset(u + h, -h), wcStroke(hz, zw(3)));
    }
    cv.restore();
    // A: the umbra edge, C: the penumbra edge
    cv.drawRRect(rr, wcStroke(zc(0), zw(0)));
    final outer = rr.inflate(sp);
    final path = Path()..addRRect(outer);
    for (final m in path.computeMetrics()) {
      for (double d = 0; d < m.length; d += 6) {
        cv.drawPath(m.extractPath(d, math.min(d + 3, m.length)), wcStroke(zc(2).withValues(alpha: lit(2) ? 1 : .7), zw(2)));
      }
    }
    // the object that casts it
    final obj = RRect.fromRectAndRadius(Rect.fromCenter(center: c, width: h * 2, height: h * 2), const Radius.circular(3));
    cv.drawRRect(obj, wcFill(N.g26));
    cv.drawRRect(obj, wcStroke(N.g76));
    // B: the sun, with a hairline of light toward the object
    final sun = _sun(s), toObj = (c - sun) / (c - sun).distance;
    for (double d = 12; d < (c - sun).distance - h * 1.6; d += 7) {
      cv.drawLine(sun + toObj * d, sun + toObj * (d + 3), wcStroke(N.g26));
    }
    final sz = zc(1);
    cv.drawCircle(sun, 3.4, wcFill(sz));
    for (var i = 0; i < 8; i++) {
      final u = wcDir(i * math.pi / 4 + t * .2);
      cv.drawLine(sun + u * 6, sun + u * (lit(1) ? 10 : 8.5), wcStroke(sz, zw(1)));
    }
    ring(cv, sun, 1, 13);
  }
}

final waveDef = WorldDef(16, 'Wave Warp', 'flag', waveSpecs, WaveWorld.new);
final noiseDef = WorldDef(17, 'Fractal Noise', 'contour map', noiseSpecs, NoiseWorld.new);
final shadowDef = WorldDef(18, 'Shadow', 'sundial', shadowSpecs, ShadowWorld.new);
