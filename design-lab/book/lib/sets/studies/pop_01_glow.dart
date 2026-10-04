// Glow sheet: radius / intensity / threshold. The reason to touch it: "only the bright parts light up, and the light spills around them".
// Default mapping: drag up = more things pass the threshold (or more intensity), drag right = the light spills further.
part of 'pop_01.dart';

const _glowSpecs = <PopSpec>[
  PopSpec('Night city', 'landscape · crisp · drag · result', _gCity),
  PopSpec('Firefly jar', 'creature · bold · 2D · drag', _gJar, bg: _K.ink1),
  PopSpec('Neon sign', 'machine · neon · 2D · drag', _gNeon),
  PopSpec('Hot bulb', 'machine · bold · 2D · drag', _gBulb, bg: _K.ink1),
  PopSpec('Peaks over clouds', 'landscape · bold · 3D · drag', _gPeaks, bg: Color(0xFF241C3A)),
  PopSpec('Grey snail', 'text-art · crisp · light · drag', _gSnail, bg: _K.cream),
  PopSpec('Jellyfish', 'creature · neon · 2D · drag', _gJelly, bg: Color(0xFF14142A)),
  PopSpec('Campfire', 'nature · flat · top-down · rub', _gFire, bg: Color(0xFF1F2A1E), b: .5),
  PopSpec('Lighthouse', 'machine · bold · map · spin', _gLighthouse, bg: Color(0xFF13203F)),
  PopSpec('Disco ball', 'toy · neon · 3D · spin', _gDisco),
  PopSpec('Glow stick', 'toy · neon · 2D · rub', _gStick, bg: _K.ink1, b: .55),
  PopSpec('Star magnitudes', 'cosmic · crisp · 2D · drag', _gStars, bg: Color(0xFF0E0E1A)),
  PopSpec('Lamp-lit block', 'landscape · bold · iso · drag', _gIsoBlock, bg: _K.ink1),
  PopSpec('Lava lamp', 'material · bold · 2D · drag', _gLava),
  PopSpec('Glow mushrooms', 'nature · flat · 2D · drag', _gShrooms, bg: Color(0xFF1A1A2E)),
  PopSpec('LED smiley', 'machine · crisp · drag · result', _gLed),
  PopSpec('Blow the candles', 'food · bold · light · blow', _gCake, bg: _K.cream),
  PopSpec('Paint with light', 'neon · wild · 2D · paint', _gPaint, a: .45, b: .8),
  PopSpec('Thermal camera', 'instrument · neon · drag · result', _gThermal, bg: Color(0xFF0D1030)),
  PopSpec('Moon halo', 'weather · crisp · 2D · drag', _gMoon, bg: Color(0xFF1E2238)),
];

double _thr(PV v) => 1 - v.b;
double _rad(PV v, [double lo = 3, double hi = 24]) => lo + v.a * (hi - lo);

void _gCity(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, thr = _thr(v), r = _rad(v, 3, 22);
  c.drawCircle(Offset(w * .84, h * .18), 7, _f(_K.cream));
  c.drawCircle(Offset(w * .84 + 3, h * .18 - 2), 6, _f(const Color(0xFF1C1C1E)));
  const xs = [0.0, .15, .28, .44, .56, .71, .85, 1.0];
  const hs = [.55, .8, .48, .92, .6, .76, .44];
  final wins = <(Offset, double)>[];
  for (var b = 0; b < 7; b++) {
    final x0 = xs[b] * w + 1, x1 = xs[b + 1] * w - 1, top = h * (1 - hs[b] * .82);
    c.drawRect(Rect.fromLTRB(x0, top, x1, h), _f(b.isEven ? const Color(0xFF33333B) : const Color(0xFF2B2B33)));
    final cols = ((x1 - x0 - 4) / 7).floor().clamp(1, 4), rows = ((h - top - 8) / 9).floor();
    final gx = (x1 - x0) / cols;
    for (var y = 0; y < rows; y++) {
      for (var x = 0; x < cols; x++) {
        wins.add((Offset(x0 + gx * (x + .5), top + 7 + y * 9), _h(b * 97 + y * 7 + x)));
      }
    }
  }
  for (final (o, br) in wins) {
    if (br > thr) _glow(c, o, r, _K.yellow, .45);
  }
  for (final (o, br) in wins) {
    final lit = br > thr;
    c.drawRect(Rect.fromCenter(center: o, width: 3.2, height: 4.2), _f(lit ? (br > .9 ? _K.white : _K.yellow) : const Color(0xFF45454E)));
  }
}

void _gJar(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, cx = w / 2, thr = _thr(v), r = _rad(v, 4, 22);
  final jar = RRect.fromLTRBR(cx - 32, h * .24, cx + 32, h - 8, const Radius.circular(16));
  c.drawRRect(jar, _f(_K.lime, .07));
  final flies = <(Offset, double)>[];
  for (var i = 0; i < 12; i++) {
    final o = Offset(cx + math.sin(v.t * .6 + i * 2.1) * 22, h * .32 + h * .27 * (1 + .9 * math.sin(v.t * .45 + i * 1.3)));
    flies.add((o, .5 + .5 * math.sin(v.t * 1.7 + i * 2.4)));
  }
  c.save();
  c.clipRRect(jar);
  for (final (o, br) in flies) {
    if (br > thr) _glow(c, o, r, _K.lime, .75 * (br - thr + .3));
  }
  c.restore();
  for (final (o, br) in flies) {
    final lit = br > thr;
    c.drawCircle(o, lit ? 2.8 : 2, _f(lit ? _K.lime : _K.dim2));
    if (lit) c.drawCircle(o, 1.2, _f(_K.white));
  }
  c.drawRRect(jar, _s(_K.cream, 2));
  c.drawRRect(RRect.fromLTRBR(cx - 26, h * .13, cx + 26, h * .24, const Radius.circular(3)), _f(_K.orange));
  c.drawLine(Offset(cx + 20, h * .34), Offset(cx + 20, h * .6), _s(_K.white, 2, .5));
}

void _neon(Canvas c, Path p, Color col, double r, double k) {
  for (var j = 3; j >= 1; j--) {
    c.drawPath(p, _s(col, 3 + r * j / 3 * 1.6, k * .16));
  }
  c.drawPath(p, _s(col, 3.4, .35 + .65 * k));
  c.drawPath(p, _s(_mix(col, _K.white, .3 + .7 * k), 1.3));
}

void _gNeon(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, r = _rad(v, 2, 18);
  var k = .15 + v.b * .85;
  if (_h((v.t * 9).floor()) < .05) k *= .35;
  for (var y = 8.0; y < h; y += 12) {
    for (var x = (y ~/ 12).isEven ? 0.0 : 12.0; x < w; x += 24) {
      c.drawRRect(RRect.fromLTRBR(x + 1, y, x + 23, y + 10, const Radius.circular(1)), _f(const Color(0xFF26262B)));
    }
  }
  final hc = Offset(w * .36, h * .5), q = 24.0;
  final heart = Path()
    ..moveTo(hc.dx, hc.dy + q * .9)
    ..cubicTo(hc.dx - q * 1.4, hc.dy + q * .05, hc.dx - q * .95, hc.dy - q * 1.05, hc.dx, hc.dy - q * .35)
    ..cubicTo(hc.dx + q * .95, hc.dy - q * 1.05, hc.dx + q * 1.4, hc.dy + q * .05, hc.dx, hc.dy + q * .9);
  final bolt = _polyPath([Offset(w * .76, h * .14), Offset(w * .66, h * .52), Offset(w * .76, h * .5), Offset(w * .7, h * .86), Offset(w * .87, h * .42), Offset(w * .77, h * .44), Offset(w * .84, h * .14)]);
  _neon(c, heart, _K.pink, r, k);
  _neon(c, bolt, _K.cyan, r, k * (_h((v.t * 7).floor() + 5) < .04 ? .3 : 1));
}

void _gBulb(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h * .42), r0 = 21.0, k = v.b;
  _glow(c, o, r0 + 6 + v.a * 34, _K.yellow, .2 + .6 * k);
  final len = 4 + v.a * 22;
  for (var i = 0; i < 12; i++) {
    final d = _dir(i * math.pi / 6 + .26);
    if (d.dy > .75) continue;
    c.drawLine(o + d * (r0 + 5), o + d * (r0 + 5 + len), _s(_K.yellow, 2.6, .25 + .75 * k));
  }
  c.drawCircle(o, r0, _f(_mix(const Color(0xFF3A3A30), _K.yellow, k * .55)));
  c.drawCircle(o, r0, _s(_K.cream, 2));
  final heat = k < .5 ? _mix(_K.red, _K.orange, k * 2) : _mix(_K.orange, _K.white, (k - .5) * 2);
  final fil = _polyPath([for (var i = 0; i <= 8; i++) Offset(o.dx - 10 + i * 2.5, o.dy + (i.isEven ? -3 : 3))], close: false);
  c.drawPath(fil, _s(heat, 2.2));
  c.drawLine(Offset(o.dx - 10, o.dy), Offset(o.dx - 6, o.dy + r0 - 2), _s(_K.cream, 1.2, .6));
  c.drawLine(Offset(o.dx + 10, o.dy), Offset(o.dx + 6, o.dy + r0 - 2), _s(_K.cream, 1.2, .6));
  for (var i = 0; i < 3; i++) {
    c.drawRRect(RRect.fromLTRBR(o.dx - 11 + i, o.dy + r0 + i * 6, o.dx + 11 - i, o.dy + r0 + 5 + i * 6, const Radius.circular(2)), _f(i.isEven ? _K.dim2 : _K.tag));
  }
}

void _gPeaks(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, cloud = h * (.42 + v.b * .45), r = _rad(v, 4, 24);
  c.drawCircle(Offset(w * .16, h * .22), 9, _f(_K.orange));
  final back = [Offset(0, h * .5), Offset(w * .14, h * .2), Offset(w * .3, h * .55), Offset(w * .5, h * .12), Offset(w * .66, h * .5), Offset(w * .84, h * .28), Offset(w, h * .52)];
  final mid = [Offset(0, h * .8), Offset(w * .22, h * .4), Offset(w * .4, h * .78), Offset(w * .6, h * .36), Offset(w * .78, h * .75), Offset(w * .92, h * .58), Offset(w, h * .7)];
  for (final (pts, col) in [(back, _K.violet), (mid, const Color(0xFF5B48B8))]) {
    c.drawPath(_polyPath([...pts, Offset(w, h), Offset(0, h)]), _f(col));
    for (var i = 1; i < pts.length - 1; i += 2) {
      final pk = pts[i];
      if (pk.dy >= cloud) continue;
      _glow(c, pk, r, _K.orange, .9);
      c.drawPath(_polyPath([pk, pk + const Offset(7, 9), pk + const Offset(-7, 9)]), _f(_K.yellow));
    }
  }
  final sea = Path()..moveTo(0, h);
  for (var x = 0.0; x <= w + 1; x += 13) {
    sea.lineTo(x, cloud + 2 + 3 * math.sin(x * .3 + v.t * 1.2));
    sea.quadraticBezierTo(x + 6.5, cloud - 6 + 2 * math.sin(x + v.t), x + 13, cloud + 2 + 3 * math.sin((x + 13) * .3 + v.t * 1.2));
  }
  sea
    ..lineTo(w, h)
    ..close();
  c.drawPath(sea, _f(_K.cream));
}

void _gSnail(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, thr = _thr(v), o = Offset(w / 2, h / 2);
  c.drawRRect(RRect.fromLTRBR(10, 8, w - 10, h - 8, const Radius.circular(8)), _f(_K.ink0));
  const n = 24;
  final pts = <(Offset, double)>[];
  for (var i = 0; i < n; i++) {
    final g = i / (n - 1), a = i * .58 + v.t * .15, rr = 46 - i * 1.7;
    pts.add((o + Offset(math.cos(a) * rr * 1.35, math.sin(a) * rr * .95), g));
  }
  for (final (p, g) in pts) {
    if (g > thr) _glow(c, p, 3 + v.a * 16, _K.yellow, .85);
  }
  for (final (p, g) in pts) {
    final lit = g > thr;
    c.drawCircle(p, 3.2, _f(lit ? _mix(_K.yellow, _K.white, (g - thr) * 3) : Color.lerp(const Color(0xFF2E2E34), const Color(0xFFB8B2A8), g)!));
  }
}

void _gJelly(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, k = v.b, r = _rad(v, 3, 18);
  final o = Offset(w / 2, h * .36 + math.sin(v.t * 1.4) * 3);
  for (var i = 0; i < 5; i++) {
    final x0 = o.dx - 18 + i * 9.0;
    final t = Path()..moveTo(x0, o.dy + 4);
    for (var y = 0.0; y <= 50; y += 5) {
      t.lineTo(x0 + math.sin(v.t * 2 + i + y * .12) * (2 + y * .1), o.dy + 4 + y);
    }
    c.drawPath(t, _s(i.isEven ? _K.pink : _K.violet, 1.6, .8));
  }
  final squeeze = 1 + .06 * math.sin(v.t * 2.8);
  final bell = Path()
    ..moveTo(o.dx - 28 * squeeze, o.dy + 4)
    ..cubicTo(o.dx - 30 * squeeze, o.dy - 30, o.dx + 30 * squeeze, o.dy - 30, o.dx + 28 * squeeze, o.dy + 4);
  for (var i = 0; i < 6; i++) {
    bell.quadraticBezierTo(o.dx + 28 * squeeze - (i + .5) * 9.33 * squeeze, o.dy + 9, o.dx + 28 * squeeze - (i + 1) * 9.33 * squeeze, o.dy + 4);
  }
  bell.close();
  final spots = [for (var i = 0; i < 6; i++) o + Offset(-17 + i * 6.8, -8 - 7 * math.sin((i + .5) / 6 * math.pi))];
  for (var i = 0; i < 6; i++) {
    _glow(c, spots[i], r, _K.pink, k * (.5 + .5 * math.sin(v.t * 3 + i)));
  }
  c.drawPath(bell, _f(_K.violet, .35 + k * .4));
  c.drawPath(bell, _s(_K.pink, 2));
  for (final p in spots) {
    c.drawCircle(p, 2.2, _f(_mix(_K.dim2, _K.white, k * 1.4)));
  }
  for (var i = 0; i < 4; i++) {
    final y = h - ((v.t * 14 + i * 31) % (h + 10));
    c.drawCircle(Offset(w * (.15 + .22 * i), y), 1.8 + i % 2, _s(_K.cyan, 1, .5));
  }
}

void _gFire(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h / 2);
  final k = _cl(.2 + v.b * .6 + v.energy * .8), pool = 18 + v.a * 34 + k * 14;
  _glow(c, o, pool, _K.orange, .35 + .45 * k);
  for (var i = 0; i < 7; i++) {
    final a = i * math.pi * 2 / 7 + .3, p = o + _dir(a) * (36 + (i % 2) * 6);
    final lit = (p - o).distance < pool * .85;
    c.drawCircle(p, 5.5, _f(lit ? _K.cream : const Color(0xFF3A4038)));
    c.drawCircle(p + _dir(a + math.pi) * 2, 2.6, _f(lit ? _K.orange : const Color(0xFF2C322B)));
  }
  for (var i = 0; i < 8; i++) {
    c.drawCircle(o + _dir(i * math.pi / 4) * 13, 3.4, _f(_K.dim2));
  }
  c.drawLine(o + const Offset(-11, -6), o + const Offset(11, 6), _s(const Color(0xFF7A4A30), 5));
  c.drawLine(o + const Offset(-11, 6), o + const Offset(11, -6), _s(const Color(0xFF8A5638), 5));
  final fl = 4 + k * 8 + math.sin(v.t * 13) * 1.2;
  c.drawPath(_star(o, fl, fl * .45, 6, v.t * 2), _f(_K.orange));
  c.drawPath(_star(o, fl * .6, fl * .3, 5, -v.t * 3), _f(_K.yellow));
  for (var i = 0; i < 4; i++) {
    final ph = (v.t * .7 + i * .25) % 1;
    c.drawCircle(o + Offset(math.sin(i * 3 + v.t) * 6, -ph * 28 * (.4 + k)), 1.3, _f(_K.yellow, (1 - ph) * k));
  }
}

void _gLighthouse(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, lh = Offset(w * .3, h * .58), len = 34 + v.a * 90, k = .2 + v.b * .8;
  for (var i = 0; i < 9; i++) {
    final p = Offset(_h(i * 3) * w, _h(i * 3 + 1) * h);
    c.drawArc(Rect.fromCenter(center: p, width: 10, height: 6), math.pi, math.pi, false, _s(_K.cyan, 1.2, .3));
  }
  final land = _polyPath([Offset(0, h * .4), Offset(w * .22, h * .44), Offset(w * .38, h * .62), Offset(w * .3, h * .82), Offset(w * .12, h), Offset(0, h)]);
  c.drawPath(land, _f(_K.lime));
  final ang = v.spin + v.t * .55, ha = .24;
  final beam = Path()
    ..moveTo(lh.dx, lh.dy)
    ..arcTo(Rect.fromCircle(center: lh, radius: len), ang - ha, ha * 2, false)
    ..close();
  c.drawPath(beam, Paint()..shader = ui.Gradient.radial(lh, len, [_al(_K.yellow, k * .9), _al(_K.yellow, 0)]));
  for (var i = 0; i < 5; i++) {
    final b = Offset(w * (.45 + .5 * _h(i * 7 + 2)), h * (.1 + .8 * _h(i * 7 + 3)));
    final d = b - lh;
    var da = (d.direction - ang) % (math.pi * 2);
    if (da > math.pi) da -= math.pi * 2;
    final lit = da.abs() < ha && d.distance < len;
    if (lit) _glow(c, b, 12, _K.yellow, k);
    c.drawPath(_polyPath([b + const Offset(-6, -1), b + const Offset(6, -1), b + const Offset(4, 3), b + const Offset(-4, 3)]), _f(lit ? _K.yellow : _K.cream, lit ? 1 : .35));
    c.drawLine(b + const Offset(0, -1), b + const Offset(0, -7), _s(lit ? _K.white : _K.cream, 1.2, lit ? 1 : .35));
  }
  c.drawCircle(lh, 6, _f(_K.white));
  c.drawCircle(lh, 6, _s(_K.red, 2.4));
  c.drawCircle(lh, 2.2, _f(_K.yellow));
}

void _gDisco(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h * .46), R = 31.0, thr = 1 - v.b * .85, rot = v.spin + v.t * .35;
  c.drawLine(Offset(o.dx, 0), Offset(o.dx, o.dy - R), _s(_K.cream, 1.2, .6));
  final tiles = <(Offset, double, double, double, int)>[];
  for (var j = 0; j < 8; j++) {
    final lat = -math.pi / 2 + (j + .5) * math.pi / 8;
    for (var k = 0; k < 14; k++) {
      final lon = rot + k * math.pi * 2 / 14, z = math.cos(lat) * math.cos(lon);
      if (z <= 0.05) continue;
      final p = o + Offset(R * math.cos(lat) * math.sin(lon), R * math.sin(lat));
      final br = _cl(z * (.35 + .75 * _h(j * 31 + k)) + (lat < 0 ? .1 : 0));
      tiles.add((p, br, 13.5 * math.cos(lat) * math.cos(lon) + .6, z, j * 14 + k));
    }
  }
  for (final (_, br, _, _, id) in tiles) {
    if (br <= thr) continue;
    final sp = Offset(8 + _h(id * 5) * (w - 16), 6 + _h(id * 5 + 1) * (h - 12));
    if ((sp - o).distance < R + 4) continue;
    final col = _K.vivid[id % 6];
    _glow(c, sp, 3 + v.a * 12, col, .7);
    c.drawCircle(sp, 1.6, _f(col));
  }
  c.drawCircle(o, R, _f(const Color(0xFF26262C)));
  for (final (p, br, tw, _, _) in tiles) {
    final lit = br > thr;
    c.drawRect(Rect.fromCenter(center: p, width: math.max(tw, 1), height: 10.5), _f(lit ? _K.white : _mix(const Color(0xFF3A3A44), _K.violet, br)));
  }
  for (final (p, br, _, _, _) in tiles) {
    if (br > thr && br > .9) c.drawPath(_star(p, 5, 1, 4, v.t), _f(_K.white));
  }
  c.drawCircle(o, R, _s(_K.cream, 1.4, .7));
}

void _gStick(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, k = _cl(v.energy + v.b * .8);
  final a = Offset(w * .18, h * .78), m = Offset(w * .3, h * .12), b = Offset(w * .84, h * .32);
  Offset at(double t) => a * ((1 - t) * (1 - t)) + m * (2 * t * (1 - t)) + b * (t * t);
  for (var i = 0; i <= 10; i++) {
    _glow(c, at(i / 10), 8 + v.a * 22, _K.lime, k * .32);
  }
  final p = Path()
    ..moveTo(a.dx, a.dy)
    ..quadraticBezierTo(m.dx, m.dy, b.dx, b.dy);
  c.drawPath(p, _s(_mix(const Color(0xFF3C4A34), _K.lime, k), 11));
  c.drawPath(p, _s(_mix(const Color(0xFF4A5A40), _K.white, k), 3.2));
  c.drawCircle(a, 6, _f(_K.cream));
  c.drawCircle(b, 6, _f(_K.cream));
  if (v.energy > .1) {
    for (var i = 0; i < 5; i++) {
      final q = at(_h((v.t * 6).floor() * 5 + i)) + Offset(_h(i + (v.t * 6).floor()) * 20 - 10, -8 - _h(i * 3) * 10);
      c.drawPath(_star(q, 4 * v.energy + 1, 1, 4), _f(_K.lime));
    }
  }
}

void _gStars(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, thr = 1 - v.b * .9;
  for (var i = 0; i < 48; i++) {
    final p = Offset(_h(i * 2) * w, _h(i * 2 + 1) * h), m = math.pow(_h(i * 11 + 3), 1.6).toDouble();
    final col = i % 5 == 0 ? _K.cyan : (i % 7 == 0 ? _K.yellow : _K.white);
    if (m > thr) {
      final L = 3 + (m - thr + .2) * v.a * 26;
      _glow(c, p, L * .7, col, .55);
      c.drawLine(p - Offset(L, 0), p + Offset(L, 0), _s(col, 1.1));
      c.drawLine(p - Offset(0, L), p + Offset(0, L), _s(col, 1.1));
      c.drawCircle(p, 2, _f(_K.white));
    } else {
      c.drawCircle(p, .7 + m * 1.2, _f(col, .3 + m * .7));
    }
  }
}

void _gIsoBlock(Canvas c, Size s, PV v) {
  final w = s.width, iso = _Iso(Offset(w / 2, 26), 15.5), thr = _thr(v), r = _rad(v, 6, 30);
  c.drawPath(_polyPath([iso.p(0, 0), iso.p(5, 0), iso.p(5, 5), iso.p(0, 5)]), _f(_K.dim));
  c.drawPath(_polyPath([iso.p(2, 0), iso.p(3, 0), iso.p(3, 5), iso.p(2, 5)]), _f(const Color(0xFF222226)));
  c.drawPath(_polyPath([iso.p(0, 2), iso.p(5, 2), iso.p(5, 3), iso.p(0, 3)]), _f(const Color(0xFF222226)));
  const lamps = [(2.5, .7), (2.5, 1.6), (2.5, 3.6), (2.5, 4.5), (.6, 2.5), (1.6, 2.5), (3.5, 2.5), (4.4, 2.5)];
  for (var i = 0; i < lamps.length; i++) {
    if (_h(i * 13 + 4) <= thr) continue;
    final g = iso.p(lamps[i].$1, lamps[i].$2);
    c.save();
    c.translate(g.dx, g.dy);
    c.scale(1, .5);
    _glow(c, Offset.zero, r, _K.yellow, .85);
    c.restore();
  }
  const blds = [(0.2, 0.2, 1.6), (3.2, 0.2, 2.4), (0.2, 3.2, 1.1), (3.2, 3.2, 1.7)];
  for (final (x, y, z) in blds) {
    iso.box(c, x, y, 1.6, 1.6, 0, z, const Color(0xFF6E5BD0), _K.violet, const Color(0xFF4636A0));
  }
  for (var i = 0; i < lamps.length; i++) {
    final lit = _h(i * 13 + 4) > thr, g = iso.p(lamps[i].$1, lamps[i].$2), top = g - const Offset(0, 12);
    c.drawLine(g, top, _s(_K.cream, 1.4, .8));
    if (lit) _glow(c, top, 7, _K.yellow, 1);
    c.drawCircle(top, 2.4, _f(lit ? _K.yellow : _K.dim2));
  }
}

void _gLava(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, cx = w / 2, thr = _thr(v);
  final glass = Path()
    ..moveTo(cx - 12, 14)
    ..lineTo(cx + 12, 14)
    ..lineTo(cx + 24, h * .66)
    ..lineTo(cx + 15, h - 22)
    ..lineTo(cx - 15, h - 22)
    ..lineTo(cx - 24, h * .66)
    ..close();
  c.drawPath(glass, _f(_K.violet, .3));
  c.save();
  c.clipPath(glass);
  for (var i = 0; i < 6; i++) {
    final ph = (v.t * (.05 + .02 * i) + i / 6) % 1, y = (h - 26) - (math.sin(ph * math.pi * 2) * .5 + .5) * (h - 46);
    final heat = 1 - (y - 14) / (h - 40), p = Offset(cx + math.sin(v.t * .5 + i * 2) * 9, y), r = 5.0 + (i % 3) * 2.2;
    if (heat > thr) _glow(c, p, r + 4 + v.a * 22, _K.yellow, .7);
    c.drawCircle(p, r, _f(_mix(_K.pink, heat > thr ? _K.yellow : _K.orange, heat)));
  }
  c.restore();
  c.drawPath(glass, _s(_K.cream, 1.6, .7));
  c.drawPath(_polyPath([Offset(cx - 9, 4), Offset(cx + 9, 4), Offset(cx + 12, 14), Offset(cx - 12, 14)]), _f(_K.orange));
  c.drawPath(_polyPath([Offset(cx - 15, h - 22), Offset(cx + 15, h - 22), Offset(cx + 22, h - 4), Offset(cx - 22, h - 4)]), _f(_K.orange));
}

void _gShrooms(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, gy = h * .84, thr = _thr(v), r = _rad(v, 4, 22);
  c.drawRect(Rect.fromLTRB(0, gy, w, h), _f(const Color(0xFF2C2440)));
  const xs = [.1, .24, .36, .52, .66, .8, .92];
  final caps = <(Offset, double, double)>[];
  for (var i = 0; i < xs.length; i++) {
    final hh = 18 + _h(i * 9) * 46, cw = 9 + _h(i * 9 + 1) * 10;
    caps.add((Offset(xs[i] * w, gy - hh), cw, _h(i * 17 + 2)));
    c.drawLine(Offset(xs[i] * w, gy), Offset(xs[i] * w + math.sin(i.toDouble()) * 3, gy - hh), _s(_K.cream, 3.5, .85));
  }
  for (final (p, cw, br) in caps) {
    if (br > thr) _glow(c, p, r + cw * .6, _K.cyan, .8);
  }
  for (final (p, cw, br) in caps) {
    final lit = br > thr;
    c.drawArc(Rect.fromCenter(center: p + const Offset(0, 3), width: cw * 2, height: cw * 1.6), math.pi, math.pi, true, _f(lit ? _K.cyan : const Color(0xFF3E3A58)));
    if (lit) {
      c.drawCircle(p + Offset(-cw * .35, -cw * .2), 1.6, _f(_K.white));
      c.drawCircle(p + Offset(cw * .3, -cw * .35), 1.3, _f(_K.white));
    }
  }
  for (var i = 0; i < 14; i++) {
    final x = _h(i * 5 + 1) * w;
    c.drawLine(Offset(x, gy + 1), Offset(x + 2, gy - 5), _s(_K.lime, 1.2, .7));
  }
}

void _gLed(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, thr = _thr(v);
  const cols = 15, rows = 11;
  final pitch = math.min((w - 12) / cols, (h - 10) / rows), x0 = (w - pitch * (cols - 1)) / 2, y0 = (h - pitch * (rows - 1)) / 2;
  double val(int x, int y) {
    final dx = x - 7.0, dy = y - 5.0, d = math.sqrt(dx * dx + dy * dy);
    if ((dx.abs() - 2.5).abs() < .6 && (dy + 1.6).abs() < 1.1) return 1;
    if (dy > 0.6 && dy < 3.4 && (d - 3.2).abs() < .65) return .9;
    if ((d - 4.8).abs() < .6) return .7;
    if (d < 4.6) return .42 + .08 * _h(x * 19 + y);
    return .06 + .18 * _h(x * 31 + y * 7);
  }

  final leds = [for (var y = 0; y < rows; y++) for (var x = 0; x < cols; x++) (Offset(x0 + x * pitch, y0 + y * pitch), val(x, y))];
  for (final (o, b) in leds) {
    if (b > thr) _glow(c, o, 2 + v.a * 9, _K.orange, .55);
  }
  for (final (o, b) in leds) {
    c.drawCircle(o, pitch * .34, _f(b > thr ? _mix(_K.orange, _K.yellow, (b - .6) * 2.5) : const Color(0xFF33302E)));
  }
}

void _gCake(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, blow = _cl(v.energy * 1.15), r = _rad(v, 4, 22);
  c.drawLine(Offset(14, h - 12), Offset(w - 14, h - 12), _s(_K.ink1, 3));
  c.drawRRect(RRect.fromLTRBR(26, h * .55, w - 26, h - 14, const Radius.circular(6)), _f(_K.orange));
  final drip = Path()..moveTo(26, h * .55 + 10);
  for (var i = 0; i < 7; i++) {
    final x = 26 + (w - 52) * i / 7, dx = (w - 52) / 7;
    drip.quadraticBezierTo(x + dx / 2, h * .55 + 18 + (i % 2) * 6, x + dx, h * .55 + 10);
  }
  drip
    ..lineTo(w - 26, h * .55 + 2)
    ..quadraticBezierTo(w - 26, h * .55 - 4, w - 32, h * .55 - 4)
    ..lineTo(32, h * .55 - 4)
    ..quadraticBezierTo(26, h * .55 - 4, 26, h * .55 + 2)
    ..close();
  c.drawPath(drip, _f(_K.pink));
  const candleCols = [_K.cyan, _K.violet, _K.lime, _K.cyan, _K.violet];
  for (var i = 0; i < 5; i++) {
    final x = 40 + (w - 80) * i / 4, top = h * .32;
    c.drawRRect(RRect.fromLTRBR(x - 3, top, x + 3, h * .55 - 2, const Radius.circular(1.5)), _f(candleCols[i]));
    final out = blow * (1 + _h(i * 3) * .5) > .9;
    final lean = blow * 9 * (.7 + .3 * math.sin(v.t * 20 + i)), size = (1 - blow * .75) * (1 + .1 * math.sin(v.t * 14 + i * 2)) * (.6 + v.b * .6);
    final fb = Offset(x, top - 2), tip = fb + Offset(lean, -14 * size);
    if (out) {
      c.drawLine(fb, fb + Offset(lean * .5, -8), _s(_K.tag, 1.4, .6));
      continue;
    }
    _glow(c, fb + Offset(lean * .4, -6 * size), r * (.5 + v.b * .7), _K.orange, .55);
    final fl = Path()
      ..moveTo(tip.dx, tip.dy)
      ..quadraticBezierTo(fb.dx + 6 * size, fb.dy - 3, fb.dx, fb.dy + 1)
      ..quadraticBezierTo(fb.dx - 6 * size, fb.dy - 3, tip.dx, tip.dy);
    c.drawPath(fl, _f(_K.yellow));
    c.drawCircle(fb + Offset(lean * .2, -3 * size), 1.8 * size, _f(_K.orange));
  }
}

void _gPaint(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, k = .2 + v.b * .8, r = _rad(v, 3, 20);
  final path = Path();
  if (v.trail.isEmpty) {
    path.moveTo(16, h * .6);
    for (var x = 16.0; x <= w - 16; x += 4) {
      path.lineTo(x, h * .5 + math.sin(x * .07) * 22 + math.sin(x * .19) * 6);
    }
  } else {
    var pen = false;
    for (final q in v.trail) {
      if (q == null) {
        pen = false;
      } else if (!pen) {
        path.moveTo(q.dx, q.dy);
        pen = true;
      } else {
        path.lineTo(q.dx, q.dy);
      }
    }
  }
  final col = _mix(_K.pink, _K.violet, .5 + .5 * math.sin(v.t));
  c.drawPath(path, _s(col, 3 + r * 1.4, k * .1));
  c.drawPath(path, _s(col, 3 + r * .7, k * .2));
  c.drawPath(path, _s(col, 4, .5 + .5 * k));
  c.drawPath(path, _s(_K.white, 1.4, .4 + .6 * k));
  final end = v.trail.isEmpty ? Offset(w - 16, h * .5 + math.sin((w - 16) * .07) * 22 + math.sin((w - 16) * .19) * 6) : v.trail.last ?? s.center(Offset.zero);
  c.drawPath(_star(end, 6 + 2 * math.sin(v.t * 6), 1.5, 4, v.t), _f(_K.white, k));
}

void _gThermal(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, thr = _thr(v), o = Offset(w / 2, h * .34);
  const layers = [(14.0, Color(0xFF2E5BFF), .1), (10.0, Color(0xFF9B7BFF), .3), (6.5, Color(0xFFFF5DA2), .5), (3.5, Color(0xFFFF7A3D), .7), (1.0, Color(0xFFFFD23F), .9)];
  for (final (g, col, lvl) in layers) {
    final hot = lvl > thr;
    final paint = _f(hot ? _mix(col, _K.white, .35) : col);
    c.drawCircle(o, 11 + g, paint);
    c.drawRRect(RRect.fromLTRBR(o.dx - 22 - g, o.dy + 16 - g * .4, o.dx + 22 + g, h + 20, Radius.circular(14 + g)), paint);
  }
  final k = _cl((1 - thr) * 1.4 - .2);
  for (final p in [o, o + const Offset(0, 30)]) {
    _glow(c, p, 8 + v.a * 30, _K.white, k * .8);
  }
  final br = _s(_K.cream, 2);
  for (final (x, y, sx, sy) in [(8.0, 8.0, 1.0, 1.0), (w - 8, 8.0, -1.0, 1.0), (8.0, h - 8, 1.0, -1.0), (w - 8, h - 8, -1.0, -1.0)]) {
    c.drawLine(Offset(x, y), Offset(x + 10 * sx, y), br);
    c.drawLine(Offset(x, y), Offset(x, y + 10 * sy), br);
  }
  c.drawCircle(Offset(w - 16, 16), 3, _f(_K.red, .5 + .5 * math.sin(v.t * 5)));
}

void _gMoon(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w * .5, h * .5), k = .2 + v.b * .8, ring = 17 + v.a * 30;
  for (var i = 0; i < 16; i++) {
    c.drawCircle(Offset(_h(i * 2 + 9) * w, _h(i * 2 + 10) * h), .9, _f(_K.white, .5));
  }
  _glow(c, o, ring + 8, _K.cream, k * .35);
  c.drawCircle(o, ring + 2, _s(_K.cyan, 2.5, k * .6));
  c.drawCircle(o, ring, _s(_K.cream, 2, k * .8));
  c.drawCircle(o, ring - 2.5, _s(_K.red, 2, k * .5));
  _glow(c, o, 18, _K.white, .3 + k * .4);
  c.drawCircle(o, 10, _f(_K.cream));
  c.drawCircle(o + const Offset(-3, -2), 2.2, _f(const Color(0xFFD9CFC0)));
  c.drawCircle(o + const Offset(3, 3), 1.6, _f(const Color(0xFFD9CFC0)));
  for (var i = 0; i < 3; i++) {
    final x = (v.t * (6 + i * 3) + i * 60) % (w + 80) - 40, y = h * (.28 + i * .25);
    for (var j = 0; j < 3; j++) {
      c.drawOval(Rect.fromCenter(center: Offset(x + j * 11, y + (j == 1 ? -3 : 0)), width: 22, height: 9), _f(_K.cream, .14));
    }
  }
}
