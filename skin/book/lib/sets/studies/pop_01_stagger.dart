// Stagger sheet: offset / direction / order. The reason to touch it: "things arrive one after another, from this side, in this order".
// Shared mapping: drag up/down = the gap between arrivals, drag left/right = which way it runs, tap = the order (along, centre-out, edges-in, random).
part of 'pop_01.dart';

const _staggerSpecs = <PopSpec>[
  PopSpec('Dominoes', 'physics · bold · 2D · drag · tap', _tDomino, bg: _K.ink1),
  PopSpec('Stadium wave', 'character · bold · 2D · drag', _tWave),
  PopSpec('Hand bells', 'instrument · flat · light · drag', _tBells, bg: _K.cream),
  PopSpec('Ducklings', 'creature · flat · top-down · drag', _tDucks, bg: Color(0xFF173A66)),
  PopSpec('Train pulls in', 'machine · bold · 2D · drag', _tTrain, bg: _K.ink1),
  PopSpec('Rocket launches', 'cosmic · neon · 2D · drag', _tRockets, bg: Color(0xFF0E0E1A)),
  PopSpec('Toaster row', 'food · bold · light · tap', _tToast, bg: _K.cream),
  PopSpec('Dealt cards', 'toy · crisp · 2D · flick', _tCards, bg: Color(0xFF2A1F4A)),
  PopSpec('Puddle drops', 'weather · crisp · top-down · drag', _tPuddle, bg: Color(0xFF16213A)),
  PopSpec('Firework chain', 'festive · neon · 2D · drag', _tFireworks),
  PopSpec('Rising pillars', 'machine · bold · iso · spin', _tPillars, bg: _K.ink1, a: .75),
  PopSpec('Sunflowers turn', 'nature · flat · 2D · drag', _tSunflowers, bg: _K.blue),
  PopSpec('Corridor lights', 'machine · crisp · 3D · drag', _tCorridor),
  PopSpec('Bubble wrap', 'material · wild · 2D · tap', _tBubbles, bg: _K.cyan),
  PopSpec('Dropped balls', 'physics · neon · 2D · drag', _tBalls),
  PopSpec('Stamped path', 'toy · bold · light · paint', _tStamps, bg: _K.cream),
  PopSpec('Hop queue', 'character · bold · 2D · tap', _tQueue, bg: _K.ink1),
  PopSpec('Frogs on pads', 'nature · flat · top-down · tap', _tFrogs, bg: Color(0xFF123C4A), b: .35),
  PopSpec('Gear train', 'machine · crisp · 2D · drag', _tGears),
  PopSpec('Photo strips', 'result · crisp · 2D · drag', _tStrips, bg: _K.ink1),
];

void _tDomino(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, gy = h * .8;
  const n = 10;
  final rk = _ranks(v, n);
  c.drawLine(Offset(6, gy), Offset(w - 6, gy), _s(_K.cream, 1.6, .35));
  for (var i = 0; i < n; i++) {
    final x = 14 + i * (w - 28) / (n - 1);
    final next = i + 1 < n ? rk[i + 1] : -1, prev = i > 0 ? rk[i - 1] : -1;
    final sign = next > rk[i] || (next < 0 && prev < rk[i]) ? 1.0 : -1.0;
    final p = _eo(_st(v, rk[i], n, dur: .35));
    c.save();
    c.translate(x + 3 * sign, gy);
    c.rotate(p * 1.15 * sign);
    final r = sign > 0 ? const Rect.fromLTWH(-7, -34, 7, 34) : const Rect.fromLTWH(0, -34, 7, 34);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(1.5)), _f(_mix(_K.cream, _K.orange, p)));
    c.drawLine(Offset(r.left + 1, r.center.dy), Offset(r.right - 1, r.center.dy), _s(_K.ink0, 1));
    c.drawCircle(Offset(r.center.dx, r.top + 8), 1.3, _f(_K.ink0));
    c.drawCircle(Offset(r.center.dx, r.bottom - 8), 1.3, _f(_K.ink0));
    c.restore();
  }
}

void _tWave(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 9;
  final rk = _ranks(v, n);
  for (var row = 0; row < 3; row++) {
    final y = h * (.42 + row * .22), sc = .75 + row * .13;
    c.drawRect(Rect.fromLTRB(0, y + 4 * sc, w, y + 14 * sc), _f(row.isEven ? _K.dim : _K.dim2));
    for (var i = 0; i < n; i++) {
      final x = 10 + i * (w - 20) / (n - 1) + (row.isOdd ? 6 : 0);
      if (x > w - 4) continue;
      final up = math.sin(math.pi * _st(v, rk[i], n, dur: .55)) * 10 * sc;
      final col = const [_K.orange, _K.cyan, _K.pink][row];
      final body = Offset(x, y - up);
      if (up > 2) {
        c.drawLine(body + Offset(-3 * sc, -3), body + Offset(-6 * sc, -12 * sc), _s(_K.cream, 1.6));
        c.drawLine(body + Offset(3 * sc, -3), body + Offset(6 * sc, -12 * sc), _s(_K.cream, 1.6));
      }
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: body + Offset(0, 2 * sc), width: 9 * sc, height: 10 * sc), Radius.circular(3 * sc)), _f(col));
      c.drawCircle(body + Offset(0, -6 * sc), 3.6 * sc, _f(_K.cream));
    }
  }
}

void _tBells(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 6;
  final rk = _ranks(v, n);
  c.drawLine(Offset(8, 14), Offset(w - 8, 14), _s(_K.ink1, 3));
  const cols = [_K.yellow, _K.orange, _K.pink, _K.violet, _K.cyan, _K.lime];
  for (var i = 0; i < n; i++) {
    final x = 18 + i * (w - 36) / (n - 1), sz = 1.15 - i * .07, p = _st(v, rk[i], n, dur: .9);
    final ang = math.sin(p * math.pi * 4) * (1 - p) * .55;
    final top = Offset(x, 14), len = 26.0 * sz;
    c.save();
    c.translate(top.dx, top.dy);
    c.rotate(ang);
    c.drawLine(Offset.zero, Offset(0, len * .5), _s(_K.ink1, 1.4));
    final b = Path()
      ..moveTo(-5 * sz, len * .5)
      ..quadraticBezierTo(-6 * sz, len * 1.1, -12 * sz, len * 1.25)
      ..lineTo(12 * sz, len * 1.25)
      ..quadraticBezierTo(6 * sz, len * 1.1, 5 * sz, len * .5)
      ..close();
    c.drawPath(b, _f(cols[i]));
    c.drawPath(b, _s(_K.ink0, 1.4));
    c.drawCircle(Offset(-ang * 14, len * 1.32), 3 * sz, _f(_K.ink0));
    c.restore();
    if (p > 0 && p < 1) {
      for (var k = 0; k < 2; k++) {
        final rr = 10 + (p + k * .3) * 26;
        c.drawArc(Rect.fromCircle(center: Offset(x, 14 + len * 1.1), radius: rr), -.5, 1.0, false, _s(cols[i], 1.6, (1 - p) * .9));
        c.drawArc(Rect.fromCircle(center: Offset(x, 14 + len * 1.1), radius: rr), math.pi - .5, 1.0, false, _s(cols[i], 1.6, (1 - p) * .9));
      }
    }
  }
  c.drawLine(Offset(8, h - 10), Offset(w - 8, h - 10), _s(_K.ink1, 1, .15));
}

void _tDucks(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 7;
  final rk = _ranks(v, n);
  Offset path(double u) => Offset(-20 + u * (w + 10), h * .55 + math.sin(u * 7) * 16);
  for (var i = 0; i < 6; i++) {
    final p = Offset(_h(i * 4) * w, _h(i * 4 + 1) * h);
    c.drawArc(Rect.fromCenter(center: p, width: 16, height: 8), math.pi, math.pi, false, _s(_K.cyan, 1.2, .25));
  }
  for (var i = 0; i < n; i++) {
    final slot = .92 - i * .12, u = slot * _eo(_st(v, rk[i], n, dur: .9)), p = path(u);
    if (p.dx < -8) continue;
    final big = i == 0, r = big ? 8.5 : 5.6, d = path(u + .01) - p, a = d.direction;
    c.drawArc(Rect.fromCircle(center: p - _dir(a) * (r + 2), radius: r + 4), a + math.pi - .6, 1.2, false, _s(_K.white, 1.2, .35));
    c.drawCircle(p, r, _f(big ? _K.cream : _K.yellow));
    final head = p + _dir(a) * r * .9;
    c.drawCircle(head, r * .6, _f(big ? _K.white : _K.yellow));
    c.drawPath(_polyPath([head + _dir(a - .5) * r * .45, head + _dir(a) * r * 1.15, head + _dir(a + .5) * r * .45]), _f(_K.orange));
  }
}

void _tTrain(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, ty = h * .7;
  const n = 5;
  final rk = _ranks(v, n), side = v.a < .5 ? -1.0 : 1.0;
  c.drawRect(Rect.fromLTRB(w * .1, ty + 9, w * .9, ty + 14), _f(_K.dim2));
  c.drawLine(Offset(0, ty + 7), Offset(w, ty + 7), _s(_K.cream, 1.4, .6));
  for (var x = 4.0; x < w; x += 9) {
    c.drawLine(Offset(x, ty + 6), Offset(x, ty + 9), _s(_K.cream, 1.4, .3));
  }
  const cols = [_K.orange, _K.cyan, _K.yellow, _K.pink, _K.violet];
  for (var i = 0; i < n; i++) {
    final slot = 10 + i * 28.0, q = _eo(_st(v, rk[i], n, dur: .7)), x = slot + (1 - q) * (w + 40) * side;
    final r = Rect.fromLTWH(x, ty - 18, 25, 20);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), _f(cols[i]));
    if (i == 0) {
      c.drawRect(Rect.fromLTWH(x + 4, ty - 26, 5, 8), _f(_K.orange));
      c.drawRect(Rect.fromLTWH(x + 12, ty - 14, 9, 6), _f(_K.ink0));
    } else {
      for (var k = 0; k < 2; k++) {
        c.drawRect(Rect.fromLTWH(x + 4 + k * 10, ty - 14, 7, 6), _f(_K.ink0, .7));
      }
    }
    for (final dx in [6.0, 19.0]) {
      c.drawCircle(Offset(x + dx, ty + 3), 3.4, _f(_K.ink0));
      c.drawCircle(Offset(x + dx, ty + 3), 1.2, _f(_K.cream));
    }
  }
}

void _tRockets(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 5;
  final rk = _ranks(v, n);
  for (var i = 0; i < 20; i++) {
    c.drawCircle(Offset(_h(i * 3) * w, _h(i * 3 + 1) * h * .8), .8, _f(_K.white, .6));
  }
  c.drawRect(Rect.fromLTRB(0, h - 10, w, h), _f(_K.dim));
  for (var i = 0; i < n; i++) {
    final x = 18 + i * (w - 36) / (n - 1), p = _st(v, rk[i], n, dur: .8), y = h - 12 - p * p * (h + 30);
    if (p > 0) {
      for (var k = 0; k < 3; k++) {
        c.drawCircle(Offset(x + (k - 1) * 6, h - 11), 4 + p * 4, _f(_K.cream, (1 - p) * .7));
      }
      final fl = 8 + math.sin(v.t * 40 + i) * 2;
      c.drawPath(_polyPath([Offset(x - 4, y), Offset(x + 4, y), Offset(x, y + fl + 6)]), _f(_K.orange));
      c.drawPath(_polyPath([Offset(x - 2, y), Offset(x + 2, y), Offset(x, y + fl)]), _f(_K.yellow));
    }
    c.drawPath(_polyPath([Offset(x - 5, y - 2), Offset(x - 9, y + 2), Offset(x - 5, y - 8)]), _f(_K.violet));
    c.drawPath(_polyPath([Offset(x + 5, y - 2), Offset(x + 9, y + 2), Offset(x + 5, y - 8)]), _f(_K.violet));
    c.drawRRect(RRect.fromLTRBR(x - 5, y - 22, x + 5, y, const Radius.circular(2)), _f(_K.cream));
    c.drawPath(_polyPath([Offset(x - 5, y - 22), Offset(x + 5, y - 22), Offset(x, y - 30)]), _f(_K.red));
    c.drawCircle(Offset(x, y - 14), 2, _f(_K.cyan));
  }
}

void _tToast(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 4;
  final rk = _ranks(v, n);
  const cols = [_K.cyan, _K.pink, _K.lime, _K.violet];
  for (var i = 0; i < n; i++) {
    final x = 6 + i * (w - 12) / n, bw = (w - 12) / n - 6, p = _st(v, rk[i], n, dur: .7);
    final lift = p < 1 ? math.sin(p * math.pi) * 30 + p * 8 : 8.0;
    final top = h * .5;
    final toast = RRect.fromLTRBR(x + 5, top - lift - 4, x + bw - 5, top - lift + 22, const Radius.circular(5));
    c.drawRRect(toast, _f(_K.orange));
    c.drawRRect(toast.deflate(2.5), _f(_K.yellow));
    c.drawOval(Rect.fromLTRB(x + 1, h - 12, x + bw - 1, h - 6), _f(_K.ink0, .15));
    final body = RRect.fromLTRBR(x, top, x + bw, h - 10, const Radius.circular(7));
    c.drawRRect(body, _f(cols[i]));
    c.drawRRect(body, _s(_K.ink0, 1.6));
    c.drawLine(Offset(x + 6, top + 1), Offset(x + bw - 6, top + 1), _s(_K.ink0, 3));
    c.drawRect(Rect.fromLTWH(x + bw - 3, top + 10 + (p < 1 && p > 0 ? 0 : 12), 5, 4), _f(_K.ink0));
  }
}

void _tCards(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, deck = Offset(w / 2, h - 16), pivot = Offset(w / 2, h + 50);
  const n = 7;
  final rk = _ranks(v, n);
  for (var k = 3; k >= 0; k--) {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: deck + Offset(k * .8, -k * 1.2), width: 18, height: 24), const Radius.circular(3)), _f(k == 0 ? _K.violet : _K.cream));
  }
  for (var i = 0; i < n; i++) {
    final p = _bo(_st(v, rk[i], n, dur: .45));
    if (p <= 0) continue;
    final a = -.75 + i * 1.5 / (n - 1), target = pivot + _dir(-math.pi / 2 + a) * 100;
    final pos = Offset.lerp(deck, target, p)!;
    c.save();
    c.translate(pos.dx, pos.dy);
    c.rotate(a * p);
    final r = Rect.fromCenter(center: Offset.zero, width: 20, height: 28);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), _f(_K.cream));
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), _s(_K.ink0, 1));
    final red = i.isEven, mark = red ? _K.red : _K.ink0;
    if (i % 3 == 0) {
      c.drawPath(_polyPath([const Offset(0, -6), const Offset(5, 0), const Offset(0, 6), const Offset(-5, 0)]), _f(mark));
    } else {
      c.drawCircle(Offset.zero, 4.5, _f(mark));
    }
    c.restore();
  }
}

void _tPuddle(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 8;
  final rk = _ranks(v, n);
  c.drawOval(Rect.fromLTRB(4, 8, w - 4, h - 8), _f(_K.blue, .35));
  for (var i = 0; i < n; i++) {
    final u = i / (n - 1), o = Offset(16 + u * (w - 32), h / 2 + math.sin(u * math.pi * 2) * 22);
    final p = _st(v, rk[i], n, dur: .9);
    if (p <= 0) {
      c.drawCircle(o, 1.2, _f(_K.cyan, .3));
      continue;
    }
    for (var k = 0; k < 3; k++) {
      final q = p - k * .18;
      if (q <= 0 || q >= 1) continue;
      c.drawCircle(o, 2 + q * 18, _s(_K.cyan, 1.8 - k * .4, 1 - q));
    }
    if (p < .25) c.drawCircle(o, 3, _f(_K.white));
  }
}

void _tFireworks(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 6;
  final rk = _ranks(v, n);
  for (var i = 0; i < 8; i++) {
    final x = i * w / 8;
    c.drawRect(Rect.fromLTRB(x + 1, h - 10 - _h(i * 5) * 22, x + w / 8 - 1, h), _f(_K.ink1));
  }
  for (var i = 0; i < n; i++) {
    final x = 14 + i * (w - 28) / (n - 1), top = Offset(x, h * (.22 + .2 * _h(i * 9))), p = _st(v, rk[i], n, dur: 1.1);
    if (p <= 0 || p >= 1) continue;
    final col = _K.vivid[i % 6];
    if (p < .3) {
      final y = h - (h - top.dy) * _eo(p / .3);
      c.drawLine(Offset(x, y), Offset(x, y + 10), _s(_K.cream, 1.6));
      continue;
    }
    final q = (p - .3) / .7, r = 4 + _eo(q) * 22;
    for (var k = 0; k < 12; k++) {
      final d = _dir(k * math.pi / 6);
      c.drawLine(top + d * r * .55, top + d * r + Offset(0, q * q * 6), _s(col, 2, 1 - q * q));
    }
    c.drawCircle(top, 3 * (1 - q), _f(_K.white));
  }
}

void _tPillars(Canvas c, Size s, PV v) {
  final w = s.width, iso = _Iso(Offset(w / 2, 46), 12.5);
  const g = 5;
  final ang = v.spin + math.pi / 4, d = _dir(ang);
  final keys = <double>[];
  for (var y = 0; y < g; y++) {
    for (var x = 0; x < g; x++) {
      keys.add((x * d.dx + y * d.dy));
    }
  }
  final mn = keys.reduce(math.min), mx = keys.reduce(math.max);
  final rk = _ranks(v, g * g, [for (final k in keys) (k - mn) / (mx - mn + 1e-6)]);
  for (var sum = 0; sum <= 2 * (g - 1); sum++) {
    for (var x = 0; x < g; x++) {
      final y = sum - x;
      if (y < 0 || y >= g) continue;
      final i = y * g + x, p = _bo(_st(v, rk[i], g * g, dur: .5) * 1.0), z = .2 + p * 3.0;
      iso.box(c, x * 1.0 + .08, y * 1.0 + .08, .84, .84, 0, z, _mix(_K.cyan, _K.yellow, p), _K.violet, const Color(0xFF5A44C8));
    }
  }
  final arrow = Offset(w - 22, 22);
  c.drawLine(arrow - Offset(d.dx * .866 - d.dy * .866, d.dx * .5 + d.dy * .5) * 9, arrow + Offset(d.dx * .866 - d.dy * .866, d.dx * .5 + d.dy * .5) * 9, _s(_K.cream, 2, .7));
  c.drawCircle(arrow + Offset(d.dx * .866 - d.dy * .866, d.dx * .5 + d.dy * .5) * 9, 2.6, _f(_K.cream));
}

void _tSunflowers(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const n = 6;
  final rk = _ranks(v, n);
  c.drawCircle(Offset(w - 18, 18), 11, _f(_K.yellow));
  for (var k = 0; k < 8; k++) {
    final d = _dir(k * math.pi / 4 + v.t * .3);
    c.drawLine(Offset(w - 18, 18) + d * 14, Offset(w - 18, 18) + d * 19, _s(_K.yellow, 2));
  }
  c.drawRect(Rect.fromLTRB(0, h - 12, w, h), _f(_K.lime, .5));
  for (var i = 0; i < n; i++) {
    final x = 14 + i * (w - 40) / (n - 1), y = h * .52 + (i % 2) * 8, p = _eo(_st(v, rk[i], n, dur: .6));
    c.drawLine(Offset(x, h), Offset(x, y), _s(const Color(0xFF3F7A2A), 2.4));
    c.drawOval(Rect.fromCenter(center: Offset(x + 5, h - 22), width: 10, height: 4), _f(_K.lime));
    final sx = .28 + .72 * p;
    c.save();
    c.translate(x + (1 - p) * -3, y);
    c.scale(sx, 1);
    for (var k = 0; k < 12; k++) {
      c.drawOval(Rect.fromCenter(center: _dir(k * math.pi / 6) * 9, width: 8, height: 8), _f(_K.yellow));
    }
    c.drawCircle(Offset.zero, 6.5, _f(const Color(0xFF6A3A1E)));
    c.restore();
  }
}

void _tCorridor(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, vp = Offset(w / 2, h * .48);
  const n = 6;
  final rk = _ranks(v, n);
  for (final corner in [Offset.zero, Offset(w, 0), Offset(0, h), Offset(w, h)]) {
    c.drawLine(corner, vp + (corner - vp) * .12, _s(_K.dim2, 1.2));
  }
  c.drawRect(Rect.fromCenter(center: vp, width: w * .12, height: h * .12), _s(_K.dim2, 1.2));
  for (var i = n - 1; i >= 0; i--) {
    final sc = 1 / (1 + i * .55), p = _st(v, rk[i], n, dur: .3);
    final cy = vp.dy - (vp.dy - 4) * sc, fy = vp.dy + (h - vp.dy - 4) * sc, lw = 34 * sc;
    if (p > 0) {
      c.drawPath(_polyPath([Offset(vp.dx - lw / 2, cy + 3 * sc), Offset(vp.dx + lw / 2, cy + 3 * sc), Offset(vp.dx + lw, fy), Offset(vp.dx - lw, fy)]), _f(_K.yellow, .12 * p));
      c.save();
      c.translate(vp.dx, fy);
      c.scale(1, .3);
      _glow(c, Offset.zero, lw * 1.3, _K.yellow, .6 * p);
      c.restore();
    }
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(vp.dx, cy + 1.5 * sc), width: lw, height: 4 * sc + 1), const Radius.circular(2)), _f(p > 0 ? _mix(_K.orange, _K.white, p) : _K.dim2));
  }
}

void _tBubbles(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  const cols = 7, rows = 5, n = cols * rows;
  final keys = [for (var y = 0; y < rows; y++) for (var x = 0; x < cols; x++) ((y.isEven ? x : cols - 1 - x) + y * cols) / (n - 1)];
  final rk = _ranks(v, n, keys);
  final px = (w - 10) / cols, py = (h - 8) / rows;
  for (var y = 0; y < rows; y++) {
    for (var x = 0; x < cols; x++) {
      final i = y * cols + x, o = Offset(5 + px * (x + .5) + (y.isOdd ? px * .25 : -px * .25), 4 + py * (y + .5)), p = _st(v, rk[i], n, dur: .25);
      final r = py * .42;
      if (p < .5) {
        c.drawCircle(o, r, _f(_K.white, .55));
        c.drawCircle(o, r, _s(_K.white, 1.4));
        c.drawCircle(o + Offset(-r * .35, -r * .35), r * .22, _f(_K.white));
      } else {
        if (p < 1) c.drawPath(_star(o, r * 1.6 * (p - .4), r * .5, 6), _f(_K.yellow));
        c.drawCircle(o, r * .9, _s(_K.blue, 1.2, .6));
      }
    }
  }
}

void _tBalls(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, fy = h - 10;
  const n = 6;
  final rk = _ranks(v, n);
  c.drawLine(Offset(4, fy + 5), Offset(w - 4, fy + 5), _s(_K.cream, 2));
  for (var i = 0; i < n; i++) {
    final x = 16 + i * (w - 32) / (n - 1), p = _st(v, rk[i], n, dur: 1.1), col = _K.vivid[i % 6];
    if (p <= 0) {
      c.drawCircle(Offset(x, 8), 3, _s(col, 1.2, .6));
      continue;
    }
    final bounce = (math.cos(math.pi * (.5 + p * 3.5))).abs() * math.pow(1 - p, 1.4) * (fy - 14);
    final y = fy - bounce, squash = bounce < 4 && p < .95 ? .7 : 1.0;
    c.drawOval(Rect.fromCenter(center: Offset(x, fy + 4), width: 12 * (1 - bounce / fy), height: 2.4), _f(_K.white, .2));
    c.drawOval(Rect.fromCenter(center: Offset(x, y - 5 * squash), width: 11 / squash, height: 11 * squash), _f(col));
  }
}

void _tStamps(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height;
  final pts = <Offset>[];
  if (v.trail.whereType<Offset>().length < 4) {
    for (var x = 12.0; x <= w - 12; x += 3) {
      pts.add(Offset(x, h / 2 + math.sin(x * .05) * 30));
    }
  } else {
    pts.addAll(v.trail.whereType<Offset>());
  }
  final pathP = _polyPath(pts, close: false);
  c.drawPath(pathP, _s(_K.ink1, 1.4, .2));
  final len = <double>[0];
  for (var i = 1; i < pts.length; i++) {
    len.add(len.last + (pts[i] - pts[i - 1]).distance);
  }
  const n = 9;
  final rk = _ranks(v, n);
  for (var i = 0; i < n; i++) {
    final at = len.last * i / (n - 1);
    var j = 0;
    while (j < len.length - 1 && len[j + 1] < at) {
      j++;
    }
    final o = pts[j], p = _bo(_st(v, rk[i], n, dur: .35));
    if (p <= 0) continue;
    c.drawPath(_star(o, 9 * p, 4 * p, 5, -math.pi / 2 + (1 - p) * 2), _f(_K.vivid[i % 6] == _K.white ? _K.red : _K.vivid[i % 6]));
    c.drawPath(_star(o, 9 * p, 4 * p, 5, -math.pi / 2 + (1 - p) * 2), _s(_K.ink0, 1.2));
  }
}

void _tQueue(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, gy = h * .8;
  const n = 6;
  final rk = _ranks(v, n);
  c.drawLine(Offset(4, gy), Offset(w - 4, gy), _s(_K.cream, 1.4, .3));
  var hopX = -1.0;
  for (var i = 0; i < n; i++) {
    final p = _st(v, rk[i], n, dur: .45);
    if (p > 0 && p < 1) hopX = 14 + i * (w - 28) / (n - 1);
  }
  for (var i = 0; i < n; i++) {
    final x = 14 + i * (w - 28) / (n - 1), p = _st(v, rk[i], n, dur: .45), up = math.sin(p * math.pi) * 24;
    final sq = p > 0 && p < .15 ? .8 : 1.0, col = _K.vivid[i % 6] == _K.white ? _K.red : _K.vivid[i % 6];
    final body = Rect.fromLTRB(x - 9 / sq, gy - 24 * sq - up, x + 9 / sq, gy - up);
    c.drawRRect(RRect.fromRectAndRadius(body, const Radius.circular(9)), _f(col));
    final look = hopX < 0 ? Offset.zero : Offset(((hopX - x) / 30).clamp(-1.5, 1.5), up > 2 ? -1.5 : 0);
    for (final ex in [-3.6, 3.6]) {
      final e = Offset(x + ex, body.top + 8);
      c.drawCircle(e, 3, _f(_K.white));
      c.drawCircle(e + look, 1.5, _f(_K.ink0));
    }
    if (up > 2) c.drawArc(Rect.fromCenter(center: Offset(x, body.top + 15), width: 6, height: 4), 0, math.pi, false, _s(_K.ink0, 1.2));
  }
}

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
