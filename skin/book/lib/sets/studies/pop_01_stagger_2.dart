// Part of pop_01_stagger.dart (cut there by size, as it was).
part of 'pop_01.dart';

void _tFrogs(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h / 2);
  const n = 9;
  final pads = [for (var i = 0; i < n; i++) i == 0 ? o : o + Offset(math.cos(i * 2.4) * (18 + i * 5.5) * 1.3, math.sin(i * 2.4) * (12 + i * 3.5))];
  final dmax = pads.map((p) => (p - o).distance).reduce(math.max);
  final rk = _ranks(v, n, [for (final p in pads) (p - o).distance / dmax]);
  for (var i = 0; i < n; i++) {
    final p = pads[i], q = _st(v, rk[i], n, dur: .5);
    if (q > 0 && q < 1) c.drawCircle(p, 10 + q * 12, _s(_K.cyan, 1.4, 1 - q));
    final pad = Path()
      ..moveTo(p.dx, p.dy)
      ..arcTo(Rect.fromCircle(center: p, radius: 10), .18, math.pi * 2 - .36, false)
      ..close();
    c.drawPath(pad, _f(_K.lime));
    for (var k = 1; k < 5; k++) {
      c.drawLine(p, p + _dir(.18 + k * (math.pi * 2 - .36) / 5) * 8, _s(const Color(0xFF6FBF3F), 1));
    }
  }
  for (var i = 0; i < n; i++) {
    final p = pads[i], q = _st(v, rk[i], n, dur: .5);
    if (q <= 0) continue;
    final from = p + const Offset(0, -40), pos = Offset.lerp(from, p, _eo(q))! - Offset(0, math.sin(q * math.pi) * 10), sc = .6 + .4 * _bo(q);
    c.drawOval(Rect.fromCenter(center: pos, width: 12 * sc, height: 9 * sc), _f(_K.pink));
    for (final ex in [-3.0, 3.0]) {
      c.drawCircle(pos + Offset(ex * sc, -4 * sc), 2.4 * sc, _f(_K.white));
      c.drawCircle(pos + Offset(ex * sc, -4.4 * sc), 1.1 * sc, _f(_K.ink0));
    }
  }
}

void _gear(Canvas c, Offset o, double r, int teeth, double ang, Color col) {
  final pts = <Offset>[];
  for (var k = 0; k < teeth; k++) {
    final a = ang + k * math.pi * 2 / teeth, step = math.pi * 2 / teeth;
    pts.addAll([o + _dir(a - step * .25) * r, o + _dir(a - step * .15) * (r + 4), o + _dir(a + step * .15) * (r + 4), o + _dir(a + step * .25) * r]);
  }
  c.drawPath(_polyPath(pts), _f(col));
  c.drawCircle(o, r * .35, _f(_K.ink0));
  c.drawLine(o, o + _dir(ang) * r * .75, _s(_K.ink0, 2));
}

void _tGears(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 5;
  final rk = _ranks(v, n);
  final gears = [(Offset(18, h * .3), 12.0, 8), (Offset(44, h * .55), 15.0, 10), (Offset(w * .5, h * .3), 12.0, 8), (Offset(w * .68, h * .62), 15.0, 10), (Offset(w - 22, h * .32), 12.0, 8)];
  for (var i = 0; i < n; i++) {
    final (o, r, t) = gears[i];
    final p = _st(v, rk[i], n, dur: .6), ang = _eo(p) * math.pi * (i.isEven ? 1 : -1) + (i.isOdd ? math.pi / t : 0);
    _gear(c, o, r, t, ang, p > 0 ? _K.vivid[i % 6] : _K.dim2);
    if (p > 0 && p < 1) c.drawPath(_star(o + _dir(-1) * (r + 8), 5 * (1 - p), 1.2, 4), _f(_K.white));
  }
}

void _tStrips(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, pic = Rect.fromCenter(center: Offset(w / 2, h / 2), width: 108, height: 72);
  const n = 6;
  final rk = _ranks(v, n), sw = pic.width / n;
  c.drawRect(pic, _s(_K.cream, 1, .25));
  for (var i = 0; i < n; i++) {
    final p = _eo(_st(v, rk[i], n, dur: .5));
    if (p <= 0) continue;
    final strip = Rect.fromLTWH(pic.left + i * sw, pic.top, sw, pic.height);
    c.save();
    c.translate(0, (1 - p) * 70);
    c.clipRect(strip);
    _pic(c, pic);
    c.restore();
  }
}
