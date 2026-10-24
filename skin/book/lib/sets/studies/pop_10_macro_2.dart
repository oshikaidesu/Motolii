// Part of pop_10_macro.dart (cut there by size, as it was).
part of 'pop_10.dart';

void _tree(Canvas c, Size s, _S st) {
  _vgrad(c, Offset.zero & s, _mix(_bl, _k0, .5), _k1);
  c.drawOval(Rect.fromCenter(center: const Offset(78, 110), width: 120, height: 14), _f(_mix(_li, _k0, .3)));
  final depth = 3 + ((1 - st.y) * 5).round(), seed = st.taps + st.flings;
  _branch(c, const Offset(78, 108), -math.pi / 2, 16 + depth * 2.0, depth, seed, .25 + st.x * 1.4, st.t, 1);
}

void _bouncyStep(_S st, double dt) {
  final b = st.b;
  if (b.isEmpty) {
    for (var i = 0; i < 9; i++) {
      b.addAll([20 + _rn(i) * 116, 20 + _rn(i + 9) * 60, 0, 0]);
    }
    b.add(-9);
  }
  if (b[36] != st.rel) {
    b[36] = st.rel;
    for (var i = 0; i < 9; i++) {
      b[i * 4 + 2] += st.vel.dx * .6 + (_rn(i + st.ends * 11) - .5) * 300;
      b[i * 4 + 3] += st.vel.dy * .6 - 150 - _rn(i + st.ends * 5) * 200;
    }
  }
  for (var i = 0; i < 9; i++) {
    final k = i * 4, r = 6 + (i % 3) * 2.5;
    if (st.down && st.p != null) {
      final d = Offset(b[k], b[k + 1]) - st.p!;
      if (d.distance < 24 && d.distance > 0) {
        b[k + 2] += d.dx / d.distance * 900 * dt;
        b[k + 3] += d.dy / d.distance * 900 * dt;
      }
    }
    b[k + 3] += 420 * dt;
    b[k] += b[k + 2] * dt;
    b[k + 1] += b[k + 3] * dt;
    if (b[k] < 6 + r || b[k] > 150 - r) {
      b[k] = _cl(b[k], 6 + r, 150 - r);
      b[k + 2] *= -.75;
    }
    if (b[k + 1] > 110 - r) {
      b[k + 1] = 110 - r;
      b[k + 3] *= -.6;
      b[k + 2] *= .96;
    }
    if (b[k + 1] < 6 + r) {
      b[k + 1] = 6 + r;
      b[k + 3] *= -.6;
    }
  }
}

void _bouncy(Canvas c, Size s, _S st) {
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(4, 4, 148, 108), const Radius.circular(6)), _s(_al(_cr, .4), 2));
  if (st.b.length < 37) return;
  for (var i = 0; i < 9; i++) {
    final k = i * 4, r = 6 + (i % 3) * 2.5, p = Offset(st.b[k], st.b[k + 1]);
    final squash = _cl(1 - (st.b[k + 3].abs() < 60 && p.dy > 104 - r ? .15 : 0), .8, 1);
    c.drawOval(Rect.fromCenter(center: p, width: r * 2 / squash, height: r * 2 * squash), _f(_pop[(i + st.ends) % 8]));
    c.drawCircle(p + Offset(-r * .35, -r * .35), r * .25, _f(_al(_wh, .6)));
  }
}

// Figures for 4 pieces (circle, triangle, triangle, square): (cx, cy, radius, rot).
const _figs = [
  [(78.0, 46.0, 15.0, 0.0), (66.0, 31.0, 9.0, -.3), (90.0, 31.0, 9.0, .3), (78.0, 82.0, 22.0, 0.0)],
  [(78.0, 84.0, 6.0, 0.0), (78.0, 46.0, 30.0, 0.0), (102.0, 34.0, 8.0, 0.0), (78.0, 80.0, 28.0, 0.0)],
  [(70.0, 60.0, 24.0, 0.0), (108.0, 60.0, 17.0, -math.pi / 2), (66.0, 34.0, 10.0, 0.0), (60.0, 54.0, 4.0, .6)],
];

void _tangram(Canvas c, Size s, _S st) {
  _bg(c, s, _bl);
  final u = st.x * 2, k = u.floor().clamp(0, 1), f = _ss(u - k);
  final a = _figs[k], b = _figs[k + 1];
  final cols = [_ye, _pk, _cy, _or];
  for (final i in [3, 0, 1, 2]) {
    final x = a[i].$1 + (b[i].$1 - a[i].$1) * f, y = a[i].$2 + (b[i].$2 - a[i].$2) * f;
    final r = a[i].$3 + (b[i].$3 - a[i].$3) * f, rot = a[i].$4 + (b[i].$4 - a[i].$4) * f + math.sin(f * math.pi) * 1.5;
    final o = Offset(x, y);
    final path = switch (i) {
      0 => Path()..addOval(Rect.fromCircle(center: o, radius: r)),
      3 => _ngon(o, r * 1.2, 4, rot + math.pi / 4),
      _ => _ngon(o, r, 3, rot),
    };
    c.drawPath(path.shift(const Offset(2, 3)), _f(_al(_k0, .3)));
    c.drawPath(path, _f(cols[i]));
  }
  final ey = Offset(a[0].$1 + (b[0].$1 - a[0].$1) * f, a[0].$2 + (b[0].$2 - a[0].$2) * f);
  if (u < .5) {
    c.drawCircle(ey + const Offset(-5, -2), 1.8, _f(_k0));
    c.drawCircle(ey + const Offset(5, -2), 1.8, _f(_k0));
  } else if (u > 1.5) {
    c.drawCircle(ey + const Offset(-10, -5), 2.2, _f(_k0));
  }
}

const _leaves = [(24.0, _pk), (60.0, _ye), (96.0, _cy), (132.0, _vi)];

void _chameleon(Canvas c, Size s, _S st) {
  _bg(c, s, _mix(_li, _k0, .7));
  c.drawPath(
      Path()
        ..moveTo(0, 80)
        ..quadraticBezierTo(78, 70, 156, 84),
      _s(_mix(_or, _k0, .35), 6));
  for (final (x, col) in _leaves) {
    final o = Offset(x, 92), sw = math.sin(st.t * 1.4 + x) * .15;
    c.save();
    c.translate(x, 78);
    c.rotate(sw);
    c.translate(-x, -78);
    c.drawPath(
        Path()
          ..moveTo(o.dx, o.dy - 14)
          ..quadraticBezierTo(o.dx + 13, o.dy, o.dx, o.dy + 18)
          ..quadraticBezierTo(o.dx - 13, o.dy, o.dx, o.dy - 14)
          ..close(),
        _f(col));
    c.drawLine(Offset(o.dx, o.dy - 12), Offset(o.dx, o.dy + 15), _s(_al(_k0, .25), 1));
    c.restore();
  }
  final cx = 16 + st.x * 124;
  double r = 0, g = 0, b = 0, tot = 0;
  for (final (x, col) in _leaves) {
    final w = 1 / ((x - cx) * (x - cx) + 30);
    tot += w;
    r += col.r * w;
    g += col.g * w;
    b += col.b * w;
  }
  final target = Color.from(alpha: 1, red: r / tot, green: g / tot, blue: b / tot);
  if (st.b.isEmpty) st.b.addAll([target.r, target.g, target.b]);
  st.b[0] += (target.r - st.b[0]) * .08;
  st.b[1] += (target.g - st.b[1]) * .08;
  st.b[2] += (target.b - st.b[2]) * .08;
  final col = Color.from(alpha: 1, red: st.b[0], green: st.b[1], blue: st.b[2]), dark = _mix(col, _k0, .35);
  final y = 62.0 + math.sin(st.t * 3) * .8;
  final tail = Path();
  for (var i = 0; i <= 30; i++) {
    final a = i * .3, rr = 10 * (1 - i / 34);
    final q = Offset(cx - 22, y + 6) + Offset(-math.cos(a) * rr, math.sin(a) * rr);
    i == 0 ? tail.moveTo(cx - 14, y) : tail.lineTo(q.dx, q.dy);
  }
  c.drawPath(tail, _s(col, 4));
  for (final dx in [-8.0, 8.0]) {
    c.drawLine(Offset(cx + dx, y + 6), Offset(cx + dx + 3, y + 13), _s(dark, 3));
  }
  c.drawOval(Rect.fromCenter(center: Offset(cx, y), width: 34, height: 18), _f(col));
  for (var i = 0; i < 3; i++) {
    c.drawLine(Offset(cx - 8 + i * 7.0, y - 8), Offset(cx - 10 + i * 7.0, y + 2), _s(dark, 2));
  }
  c.drawPath(_poly([Offset(cx + 12, y - 8), Offset(cx + 28, y + 1), Offset(cx + 12, y + 7)]), _f(col));
  c.drawCircle(Offset(cx + 14, y - 3), 5, _f(dark));
  c.drawCircle(Offset(cx + 15 + math.sin(st.t * 2) * 1.2, y - 3), 2, _f(_k0));
}

void _fireworks(Canvas c, Size s, _S st) {
  for (var i = 0; i < 10; i++) {
    final h = 12 + _rn(i + 40) * 22, w = 10 + _rn(i + 50) * 8.0;
    c.drawRect(Rect.fromLTWH(i * 16.0, _ch - h, w + 6, h), _f(_k1));
  }
  final amp = .6 + (1 - st.y) * .8;
  for (var j = 0; j < 2; j++) {
    final ph = st.t / 2.4 + j * .5;
    final seed = 1000 + ph.floor() * 2 + j;
    _burst(c, Offset(40 + _rn(seed) * 76, 26 + _rn(seed + 1) * 26), _wr(ph) * 2.4, seed, amp * .8);
  }
  if (st.rel > 0 && st.t - st.rel < 2.4) {
    final o = st.p == null ? const Offset(78, 40) : Offset(st.p!.dx, math.min(st.p!.dy, 70));
    _burst(c, o, st.t - st.rel, st.ends + st.taps, amp);
  }
}

void _burst(Canvas c, Offset o, double since, int seed, double amp) {
  if (since < .35) {
    final q = Offset(o.dx, _ch - (_ch - o.dy) * since / .35);
    c.drawLine(q, q + const Offset(0, 10), _s(_ye, 2));
    return;
  }
  final u = since - .35, rays = 8 + seed % 9, fade = _cl(1.9 - u);
  final c1 = _pop[seed % 8], c2 = _pop[(seed * 3 + 1) % 8];
  final rr = 46 * amp * (1 - math.exp(-u * 3.2));
  for (var i = 0; i < rays; i++) {
    final a = i * 2 * math.pi / rays + _rn(seed) * 3;
    for (var k = 0; k < 4; k++) {
      final q = o + _pol(a, rr * (1 - k * .12)) + Offset(0, u * u * 10);
      c.drawCircle(q, (2.6 - k * .5) * fade + .2, _f(_al(k.isEven ? c1 : c2, fade * (1 - k * .2))));
    }
  }
  if (seed % 2 == 0) c.drawCircle(o, rr * .5, _s(_al(c2, fade), 2));
  c.drawCircle(o, 5 * fade, _f(_al(_wh, fade)));
}

void _mosaicImg(Canvas c) {
  _vgrad(c, const Rect.fromLTWH(0, 0, _cw, 70), _ye, _or);
  c.drawCircle(const Offset(78, 58), 24, _f(_rd));
  c.drawPath(_poly(const [Offset(-10, 82), Offset(40, 36), Offset(84, 82)]), _f(_vi));
  c.drawPath(_poly(const [Offset(60, 82), Offset(116, 30), Offset(170, 82)]), _f(_pk));
  c.drawRect(const Rect.fromLTWH(0, 80, _cw, 36), _f(_cy));
  for (var i = 0; i < 4; i++) {
    c.drawLine(Offset(20 + i * 34.0, 92 + (i % 2) * 8.0), Offset(38 + i * 34.0, 92 + (i % 2) * 8.0), _s(_wh, 2));
  }
}

void _mosaic(Canvas c, Size s, _S st) {
  if (st.b.isEmpty) st.b.addAll([0, 0]);
  if (st.b[1] != st.taps) {
    st.b[1] = st.taps.toDouble();
    st.b[0] = st.dist;
  }
  final amt = _ss((st.dist - st.b[0]) / 260), seed = st.ends;
  const tw = 26.0, th = 29.0;
  for (var i = 0; i < 6; i++) {
    for (var j = 0; j < 4; j++) {
      final k = i * 4 + j, r = Rect.fromLTWH(i * tw, j * th, tw, th);
      final src = ((_rn(k + seed * 31) * 24).floor());
      final si = src ~/ 4, sj = src % 4;
      final from = Offset(si * tw, sj * th) - r.topLeft;
      c.save();
      c.translate(r.center.dx, r.center.dy);
      c.rotate(amt * (_rn(k + 9 + seed) - .5) * 1.2);
      c.translate(-r.center.dx, -r.center.dy);
      c.clipRect(r.deflate(1));
      c.translate(-from.dx * amt, -from.dy * amt);
      _mosaicImg(c);
      c.restore();
    }
  }
}

void _donut(Canvas c, Size s, _S st) {
  const cc = Offset(78, 58);
  c.drawCircle(cc + const Offset(2, 4), 54, _f(_al(_k0, .12)));
  c.drawCircle(cc, 54, _f(_wh));
  final ring = Path()
    ..addOval(Rect.fromCircle(center: cc, radius: 44))
    ..addOval(Rect.fromCircle(center: cc, radius: 14))
    ..fillType = PathFillType.evenOdd;
  c.drawPath(ring, _f(_mix(_or, _ye, .45)));
  final glaze = Path()..fillType = PathFillType.evenOdd;
  final g = Path();
  for (var i = 0; i <= 60; i++) {
    final a = i / 60 * 2 * math.pi, r = 38 + 3 * math.sin(a * 7) + 2 * math.sin(a * 3);
    final q = cc + _pol(a, r);
    i == 0 ? g.moveTo(q.dx, q.dy) : g.lineTo(q.dx, q.dy);
  }
  glaze
    ..addPath(g..close(), Offset.zero)
    ..addOval(Rect.fromCircle(center: cc, radius: 18));
  c.drawPath(glaze, _f(_h(.95 + st.x * .6, .85, .62)));
  final seed = (st.dist / 140).floor() + st.taps * 7;
  for (var i = 0; i < 30; i++) {
    final q = i + seed * 41, a = _rn(q) * 2 * math.pi, r = 21 + _rn(q + 1) * 15;
    final p = cc + _pol(a, r), d = _pol(_rn(q + 2) * math.pi, 3);
    c.drawLine(p - d, p + d, _s(_pop[(q * 3) % 8 == 4 ? 2 : (q * 3) % 8], 2.6));
  }
}
