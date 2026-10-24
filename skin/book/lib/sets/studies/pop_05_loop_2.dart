// Part of pop_05_loop.dart (cut there by size, as it was).
part of 'pop_05_loop.dart';

void _slinky(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final coils = 3 + (p.a * 7).round();
  final x0 = 16.0, x1 = s.width - 16;
  final pts = <Offset>[];
  const steps = 260;
  for (var i = 0; i <= steps; i++) {
    final u = i / steps, a = u * coils * _pi2 + p.b * _pi2;
    pts.add(Offset(lerp(x0, x1, u) + math.cos(a) * 6, s.height / 2 + math.sin(a) * 26 - math.sin(u * math.pi) * 10));
  }
  for (var i = 0; i < steps; i++) {
    final a = (i / steps) * coils * _pi2 + p.b * _pi2;
    final front = math.cos(a) > 0;
    c.drawLine(pts[i], pts[i + 1], st(front ? kCyan : al(kCyan, .35), front ? 3 : 2));
  }
  final per = 1 / coils;
  final u = (lu(p) * per + p.b * per).clamp(0.0, 1.0);
  for (var lap = 0; lap < coils; lap++) {
    final q = pts[((u + lap * per) * steps).round().clamp(0, steps)];
    c.drawCircle(q, lap == 0 ? 6 : 3, fl(lap == 0 ? mc(p) : al(mc(p), .5)));
  }
}

void _seasons(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  const seasons = [kLime, Color(0xFF3E9E3A), kOrange, kWhite];
  final q = jpos(p) * 4, i = q.floor() % 4, fr = q - q.floor();
  final col = Color.lerp(seasons[i], seasons[(i + 1) % 4], fr > .7 ? (fr - .7) / .3 : 0)!;
  c.drawRect(Rect.fromLTWH(0, s.height - 18, s.width, 18), fl(i == 3 ? kWhite : const Color(0xFF6FBF4A)));
  c.drawRect(Rect.fromLTWH(s.width / 2 - 6, 54, 12, 50), fl(const Color(0xFF6B4A2A)));
  for (final (o, r) in [(const Offset(0, -6), 30.0), (const Offset(-22, 8), 20.0), (const Offset(22, 8), 20.0)]) {
    c.drawCircle(Offset(s.width / 2, 46) + o, r, fl(col));
    c.drawCircle(Offset(s.width / 2, 46) + o, r, st(kInk, 2));
  }
  if (i == 2) {
    for (var k = 0; k < 4; k++) {
      final y = 70 + frac(p.t * .5 + k * .25) * 34;
      c.drawOval(Rect.fromCenter(center: Offset(40 + k * 24.0, y), width: 7, height: 4), fl(kOrange));
    }
  }
  for (var k = 0; k < 4; k++) {
    final d = Offset(22 + k * 37.0, 12);
    final kk = frac((k + .5) / 4 - p.b);
    final inChunk = kk <= len(p) + .01;
    c.drawCircle(d, 6, fl(seasons[k]));
    c.drawCircle(d, 6, st(kInk, 1.2));
    if (inChunk) c.drawCircle(d, 9.5, st(mc(p) == kYellow ? kOrange : mc(p), 2.5));
  }
}
