// Scatter sheet: amount / spread / seed. The reason to touch it: "throw copies around at random: more of them, further out, a different throw".
// Default mapping: drag up = amount, drag right = spread, tap or flick = a new seed (a different arrangement).
part of 'pop_01.dart';

const _scatterSpecs = <PopSpec>[
  PopSpec('Sowing seeds', 'nature · flat · top-down · throw', _sSow, bg: _K.cream),
  PopSpec('Dice on felt', 'toy · bold · iso · tap', _sDice, bg: Color(0xFF16305E)),
  PopSpec('Confetti cannon', 'toy · neon · 2D · flick', _sConfetti),
  PopSpec('Donut sprinkles', 'food · bold · light · drag', _sDonut, bg: _K.cream),
  PopSpec('Dandelion', 'nature · crisp · 2D · blow', _sDandelion, bg: _K.ink1),
  PopSpec('Galaxy', 'cosmic · neon · 2D · spin', _sGalaxy, bg: Color(0xFF0E0E1A)),
  PopSpec('Billiard break', 'physics · bold · top-down · flick', _sPool, bg: Color(0xFF14284F)),
  PopSpec('Fish school', 'creature · flat · 2D · drag', _sFish, bg: Color(0xFF102A36)),
  PopSpec('Paint splat', 'material · neon · light · flick', _sSplat, bg: _K.cream),
  PopSpec('Rain radar', 'weather · crisp · map · drag', _sRadar),
  PopSpec('Constellation', 'cosmic · crisp · 2D · tap', _sConst, bg: Color(0xFF1D1836)),
  PopSpec('Tossed crates', 'toy · bold · iso · throw', _sCrates, bg: _K.ink1),
  PopSpec('Bees round a hive', 'creature · bold · light · drag', _sBees, bg: _K.cream),
  PopSpec('Popcorn pan', 'food · wild · top-down · rub', _sPopcorn, bg: _K.red),
  PopSpec('Map pins', 'landscape · crisp · map · drag', _sPins, bg: _K.cream),
  PopSpec('Darts', 'toy · bold · 2D · throw', _sDarts, bg: _K.ink1),
  PopSpec('Snow globe', 'weather · bold · 3D · shake', _sGlobe),
  PopSpec('Photo shards', 'result · crisp · 2D · drag', _sShards, bg: _K.ink1),
  PopSpec('Leaves in wind', 'nature · flat · 2D · drag', _sLeaves, bg: Color(0xFF232634)),
  PopSpec('Sheep on a hill', 'character · flat · top-down · tap', _sSheep, bg: _K.lime),
];

int _amt(PV v, [int lo = 3, int hi = 40]) => lo + (v.b * (hi - lo)).round();

void _sSow(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = v.pt(s), n = _amt(v, 4, 40), sp = 6 + v.a * 62;
  for (var x = -h; x < w; x += 11) {
    c.drawLine(Offset(x, h), Offset(x + h, 0), _s(_K.orange, 2, .18));
  }
  final pts = [for (var i = 0; i < n; i++) o + _rnd2(v.seed, i) * sp]..sort((p, q) => p.dy.compareTo(q.dy));
  for (var i = 0; i < pts.length; i++) {
    final p = pts[i], a = _h(i * 3 + v.seed) * .8 - .4;
    c.drawLine(p, p + _dir(-math.pi / 2 + a) * 6, _s(const Color(0xFF3F7A2A), 1.4));
    final top = p + _dir(-math.pi / 2 + a) * 6;
    for (final sgn in [-1.0, 1.0]) {
      c.save();
      c.translate(top.dx, top.dy);
      c.rotate(a + sgn * .8);
      c.drawOval(Rect.fromLTWH(sgn > 0 ? 0 : -7, -2, 7, 4), _f(_K.lime));
      c.restore();
    }
    c.drawCircle(p + const Offset(0, 1), 1.6, _f(const Color(0xFF6A4A30)));
  }
  final pouch = RRect.fromRectAndRadius(Rect.fromCenter(center: o + const Offset(0, -3), width: 14, height: 13), const Radius.circular(5));
  c.drawRRect(pouch, _f(_K.orange));
  c.drawRRect(pouch, _s(_K.ink0, 1.4));
  c.drawLine(o + const Offset(-4, -9), o + const Offset(4, -9), _s(_K.ink0, 2));
}

void _sDice(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h * .54), n = 1 + (v.b * 6).round(), sp = 22 + v.a * 80;
  c.drawRRect(RRect.fromLTRBR(6, 6, w - 6, h - 6, const Radius.circular(8)), _s(_K.cyan, 1.5, .35));
  final dice = [
    for (var i = 0; i < n; i++)
      Offset((o + _rnd2(v.seed, i) * sp).dx.clamp(20.0, w - 20), (o + _rnd2(v.seed, i) * sp * .7).dy.clamp(22.0, h - 14))
  ]..sort((p, q) => p.dy.compareTo(q.dy));
  const k = 12.0;
  final ex = const Offset(.866, .5) * k, ey = const Offset(-.866, .5) * k, ez = const Offset(0, -1) * k;
  for (var i = 0; i < dice.length; i++) {
    final b = dice[i];
    final p0 = b, p1 = b + ex, p2 = b + ex + ey, p3 = b + ey;
    c.drawOval(Rect.fromCenter(center: b + ey * .5 + ex * .5 + const Offset(0, 3), width: k * 2.2, height: k * 1.1), _f(_K.ink0, .45));
    c.drawPath(_polyPath([p3, p2, p2 + ez, p3 + ez]), _f(_K.orange));
    c.drawPath(_polyPath([p1, p2, p2 + ez, p1 + ez]), _f(_K.red));
    c.drawPath(_polyPath([p0 + ez, p1 + ez, p2 + ez, p3 + ez]), _f(_K.cream));
    final face = 1 + (_hs(v.seed, i + 40) * 6).floor(), ctr = b + ez + (ex + ey) * .5;
    final spots = switch (face) {
      1 => [Offset.zero],
      2 => [const Offset(-.5, -.5), const Offset(.5, .5)],
      3 => [const Offset(-.5, -.5), Offset.zero, const Offset(.5, .5)],
      4 => [const Offset(-.5, -.5), const Offset(.5, -.5), const Offset(-.5, .5), const Offset(.5, .5)],
      5 => [const Offset(-.5, -.5), const Offset(.5, -.5), Offset.zero, const Offset(-.5, .5), const Offset(.5, .5)],
      _ => [const Offset(-.5, -.5), const Offset(.5, -.5), const Offset(-.5, 0), const Offset(.5, 0), const Offset(-.5, .5), const Offset(.5, .5)],
    };
    for (final sp2 in spots) {
      c.drawCircle(ctr + ex * (sp2.dx * .6) + ey * (sp2.dy * .6), 1.4, _f(_K.ink0));
    }
  }
}

void _sConfetti(Canvas c, Size s, PV v) {
  final h = s.height, base = Offset(20, h - 18), aim = -.85, n = _amt(v, 6, 60), cone = .12 + v.a * 1.3;
  final ph = (v.t % 2.4) / 2.4;
  for (var i = 0; i < n; i++) {
    final d = aim + (_hs(v.seed, i) - .5) * cone, dist = 30 + _hs(v.seed, i + 300) * 110;
    final p = base + _dir(d) * dist * _eo(ph * 1.4) + Offset(0, 46 * ph * ph);
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(v.t * (2 + _h(i) * 6) + i);
    c.drawRect(Rect.fromCenter(center: Offset.zero, width: 5, height: 2.6), _f(_K.vivid[i % 7]));
    c.restore();
  }
  c.save();
  c.translate(base.dx, base.dy);
  c.rotate(aim);
  c.drawRRect(RRect.fromLTRBR(-6, -7, 24, 7, const Radius.circular(4)), _f(_K.violet));
  c.drawRect(const Rect.fromLTRB(14, -7, 18, 7), _f(_K.yellow));
  c.restore();
  c.drawCircle(base + const Offset(-2, 6), 6, _f(_K.orange));
  c.drawCircle(base + const Offset(-2, 6), 2, _f(_K.ink0));
}

void _sDonut(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h / 2), n = _amt(v, 4, 56), arc = .4 + v.a * (math.pi * 2 - .4);
  c.drawCircle(o + const Offset(2, 4), 44, _f(_K.ink0, .12));
  final dough = Path()
    ..fillType = PathFillType.evenOdd
    ..addOval(Rect.fromCircle(center: o, radius: 44))
    ..addOval(Rect.fromCircle(center: o, radius: 13));
  c.drawPath(dough, _f(_K.orange));
  final ice = Path()..fillType = PathFillType.evenOdd;
  final pts = [for (var i = 0; i < 40; i++) o + _dir(i * math.pi / 20) * (37 + 3 * math.sin(i * 1.7))];
  ice.addPath(_polyPath(pts), Offset.zero);
  ice.addOval(Rect.fromCircle(center: o, radius: 17));
  c.drawPath(ice, _f(_K.pink));
  final jig = v.energy * 2;
  for (var i = 0; i < n; i++) {
    final a = -math.pi / 2 + (_hs(v.seed, i) - .5) * arc, r = 21 + _hs(v.seed, i + 77) * 13;
    final p = o + _dir(a) * r + Offset(math.sin(v.t * 30 + i) * jig, math.cos(v.t * 27 + i) * jig), d = _dir(_hs(v.seed, i + 9) * math.pi);
    c.drawLine(p - d * 2.4, p + d * 2.4, _s(const [_K.yellow, _K.cyan, _K.lime, _K.white, _K.violet][i % 5], 2.2));
  }
}

void _sDandelion(Canvas c, Size s, PV v) {
  final h = s.height, head = Offset(36, h * .46), m = 28, gone = math.min(m, (v.b * m + v.energy * m).round());
  final stem = Path()
    ..moveTo(30, h)
    ..quadraticBezierTo(24, h * .75, head.dx, head.dy);
  c.drawPath(stem, _s(_K.lime, 2.4));
  void seed(Offset base, Offset tip, double a) {
    c.drawLine(base, tip, _s(_K.cream, 1, .8));
    for (var j = -2; j <= 2; j++) {
      c.drawLine(tip, tip + _dir(a + j * .35) * 4, _s(_K.white, .9));
    }
  }

  for (var i = gone; i < m; i++) {
    final a = i * math.pi * 2 / m;
    seed(head + _dir(a) * 3, head + _dir(a) * 15, a);
  }
  for (var i = 0; i < gone; i++) {
    final dist = 18 + _hs(v.seed, i) * (14 + v.a * 100), lift = (_hs(v.seed, i + 50) - .6) * (12 + v.a * 70);
    final p = head + Offset(dist, lift + math.sin(v.t * 1.3 + i) * 3);
    final a = -math.pi / 2 + math.sin(v.t + i) * .4;
    seed(p - _dir(a) * 8, p, a);
  }
  c.drawCircle(head, 3.5, _f(_K.orange));
}

void _sGalaxy(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h / 2), n = _amt(v, 40, 260), rot = v.spin * .6 + v.t * .12 + v.seed * 1.7;
  _glow(c, o, 22, _K.yellow, .55);
  for (var i = 0; i < n; i++) {
    final r = math.pow(_hs(v.seed, i), .8) * (14 + v.a * 90), arm = i.isEven ? 0 : math.pi;
    final a = arm + r * .07 + rot + (_hs(v.seed, i + 500) - .5) * (.3 + v.a * 1.1);
    final p = o + Offset(math.cos(a) * r, math.sin(a) * r * .62);
    final col = r < 12 ? _K.yellow : (i % 3 == 0 ? _K.pink : (i % 3 == 1 ? _K.cyan : _K.violet));
    c.drawCircle(p, 1 + _h(i) * 1.6, _f(col));
  }
  c.drawCircle(o, 3.5, _f(_K.white));
}

void _sPool(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, n = _amt(v, 3, 15), rack = Offset(w * .56, h / 2);
  c.drawRRect(RRect.fromLTRBR(6, 6, w - 6, h - 6, const Radius.circular(6)), _s(_K.cream, 3));
  for (final p in [const Offset(9, 9), Offset(w / 2, 7), Offset(w - 9, 9), Offset(9, h - 9), Offset(w / 2, h - 7), Offset(w - 9, h - 9)]) {
    c.drawCircle(p, 5, _f(_K.ink0));
  }
  final cue = Offset(w * .2 + v.a * 8, h / 2);
  c.drawLine(Offset(w * .06, h / 2), cue, _s(_K.cyan, 1.2, .5));
  final balls = <Offset>[];
  var row = 0, k = 0;
  while (balls.length < n) {
    final idx = balls.length;
    final home = rack + Offset(row * 8.0, (k - row / 2) * 9.2);
    final kick = _rnd2(v.seed, idx) * (v.a * 64);
    final p = home + kick;
    balls.add(Offset(p.dx.clamp(14.0, w - 14), p.dy.clamp(14.0, h - 14)));
    if (++k > row) {
      row++;
      k = 0;
    }
  }
  for (var i = 0; i < balls.length; i++) {
    final col = _K.vivid[i % 7];
    c.drawCircle(balls[i], 4.6, _f(col));
    if (i >= 7) c.drawCircle(balls[i], 2.2, _f(_K.white));
  }
  c.drawCircle(cue, 4.6, _f(_K.white));
}

void _sFish(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, n = _amt(v, 3, 28), o = Offset(w / 2 + math.sin(v.t * .5) * 10, h / 2 + math.sin(v.t * .8) * 4);
  for (var i = 0; i < 5; i++) {
    final x = (i * 37 + v.t * 6) % w;
    c.drawLine(Offset(x, 0), Offset(x - 22, h), _s(_K.cyan, 6, .05));
  }
  for (var i = 0; i < n; i++) {
    final p = o + Offset(_rnd2(v.seed, i).dx * (8 + v.a * 64), _rnd2(v.seed, i).dy * (5 + v.a * 40)) + Offset(math.sin(v.t * 2 + i) * 2, math.cos(v.t * 1.7 + i) * 1.5);
    final col = i % 3 == 0 ? _K.yellow : _K.cyan, wag = math.sin(v.t * 9 + i) * 1.5;
    c.drawPath(_polyPath([p + const Offset(-4, 0), p + Offset(-9, -3.5 + wag), p + Offset(-9, 3.5 + wag)]), _f(col));
    c.drawOval(Rect.fromCenter(center: p, width: 11, height: 5.5), _f(col));
    c.drawCircle(p + const Offset(3, -.6), .9, _f(_K.ink0));
  }
}

void _sSplat(Canvas c, Size s, PV v) {
  final o = v.pt(s), n = _amt(v, 3, 34), sp = 4 + v.a * 56;
  final blob = [for (var i = 0; i < 18; i++) o + _dir(i * math.pi / 9) * (10 + _hs(v.seed, i + 200) * 7)];
  final bp = Path()..moveTo((blob[0] + blob[1]).dx / 2, (blob[0] + blob[1]).dy / 2);
  for (var i = 1; i <= 18; i++) {
    final a = blob[i % 18], b = blob[(i + 1) % 18];
    bp.quadraticBezierTo(a.dx, a.dy, (a.dx + b.dx) / 2, (a.dy + b.dy) / 2);
  }
  for (var i = 0; i < n; i++) {
    final d = _dir(_hs(v.seed, i) * math.pi * 2), dist = 14 + _hs(v.seed, i + 60) * sp, r = 1.2 + (1 - dist / (14 + sp)) * 4;
    final p = o + d * dist, col = i % 4 == 0 ? _K.violet : _K.pink;
    if (dist > 22) c.drawLine(p - d * (dist * .25), p, _s(col, r * .6));
    c.drawCircle(p, r, _f(col));
  }
  c.drawPath(bp, _f(_K.pink));
  c.drawCircle(o + const Offset(-4, -4), 3, _f(_K.white, .6));
}

void _sRadar(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w * .52, h * .5), n = _amt(v, 6, 80), sp = 10 + v.a * 46;
  c.drawPath(_polyPath([Offset(0, h * .2), Offset(w * .3, h * .3), Offset(w * .45, h * .7), Offset(w * .8, h * .62), Offset(w, h * .8), Offset(w, h), Offset(0, h)]), _f(_K.lime, .14));
  for (var r = 20.0; r < 90; r += 20) {
    c.drawCircle(o, r, _s(_K.lime, 1, .18));
  }
  c.drawCircle(o, sp, _f(_K.violet, .22));
  for (var i = 0; i < n; i++) {
    final p = o + _rnd2(v.seed, i) * sp, ph = (v.t * 1.1 + _hs(v.seed, i + 33)) % 1;
    c.drawCircle(p, 1 + ph * 5, _s(_K.cyan, 1.2, 1 - ph));
    c.drawCircle(p, 1, _f(_K.cyan));
  }
  final sweep = v.t * 1.5;
  c.drawLine(o, o + _dir(sweep) * 70, _s(_K.lime, 1.4, .6));
}

void _sConst(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h / 2), n = _amt(v, 4, 18);
  final pts = [
    for (var i = 0; i < n; i++)
      Offset((o + _rnd2(v.seed, i) * (14 + v.a * 70)).dx.clamp(8.0, w - 8), (o + _rnd2(v.seed, i) * (10 + v.a * 46)).dy.clamp(8.0, h - 8))
  ];
  for (var i = 1; i < n; i++) {
    var best = 0;
    for (var j = 1; j < i; j++) {
      if ((pts[j] - pts[i]).distance < (pts[best] - pts[i]).distance) best = j;
    }
    c.drawLine(pts[i], pts[best], _s(_K.violet, 1.2, .9));
  }
  for (var i = 0; i < n; i++) {
    final tw = .7 + .3 * math.sin(v.t * 3 + i * 1.3);
    if (i % 4 == 0) {
      _glow(c, pts[i], 8, _K.yellow, .5 * tw);
      c.drawPath(_star(pts[i], 5 * tw, 1.4, 4), _f(_K.yellow));
    } else {
      c.drawCircle(pts[i], 2.4, _f(_K.cream));
    }
  }
}

void _sCrates(Canvas c, Size s, PV v) {
  final w = s.width, iso = _Iso(Offset(w / 2, 14), 13), n = _amt(v, 1, 14);
  c.drawPath(_polyPath([iso.p(0, 0), iso.p(6, 0), iso.p(6, 6), iso.p(0, 6)]), _f(_K.dim));
  final cr = <(double, double)>[];
  for (var i = 0; i < n; i++) {
    final d = _rnd2(v.seed, i) * (.3 + v.a * 4.4);
    cr.add(((2.6 + d.dx).clamp(0.0, 5.2), (2.6 + d.dy).clamp(0.0, 5.2)));
  }
  cr.sort((p, q) => (p.$1 + p.$2).compareTo(q.$1 + q.$2));
  final stack = <(double, double, double)>[];
  for (final (x, y) in cr) {
    var z = 0.0;
    for (final (x2, y2, z2) in stack) {
      if ((x - x2).abs() < .55 && (y - y2).abs() < .55) z = math.max(z, z2 + .8);
    }
    stack.add((x, y, z));
  }
  stack.sort((p, q) => (p.$3 * 10 + p.$1 + p.$2).compareTo(q.$3 * 10 + q.$1 + q.$2));
  for (final (x, y, z) in stack) {
    iso.box(c, x, y, .8, .8, z, z + .8, _K.yellow, _K.orange, const Color(0xFFD9542A));
    c.drawLine(iso.p(x + .1, y + .4, z + .8), iso.p(x + .7, y + .4, z + .8), _s(_K.orange, 1.2));
  }
}

void _sBees(Canvas c, Size s, PV v) {
  final h = s.height, hive = Offset(46, h * .56), n = _amt(v, 2, 18);
  for (var i = 0; i < 4; i++) {
    final wd = 30.0 - (i - 1.5).abs() * 7;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hive + Offset(0, (i - 1.5) * 9), width: wd, height: 10), const Radius.circular(5)), _f(_K.orange));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hive + Offset(0, (i - 1.5) * 9), width: wd, height: 10), const Radius.circular(5)), _s(_K.ink0, 1.2));
  }
  c.drawLine(hive - const Offset(0, 22), hive - const Offset(0, 30), _s(_K.ink0, 1.6));
  c.drawCircle(hive + const Offset(0, 6), 3, _f(_K.ink0));
  for (var i = 0; i < n; i++) {
    final r = 16 + _hs(v.seed, i) * (6 + v.a * 90), a = _hs(v.seed, i + 20) * math.pi * 2 + v.t * (.5 + _hs(v.seed, i + 40)) * (i.isEven ? 1 : -1);
    final p = hive + Offset(math.cos(a) * r, math.sin(a) * r * .55);
    for (var k = 1; k <= 3; k++) {
      final q = hive + Offset(math.cos(a - k * .12 * (i.isEven ? 1 : -1)) * r, math.sin(a - k * .12 * (i.isEven ? 1 : -1)) * r * .55);
      c.drawCircle(q, .8, _f(_K.ink0, .3));
    }
    c.drawOval(Rect.fromCenter(center: p + const Offset(-1, -3.4), width: 4, height: 5), _f(_K.white, .9));
    c.drawOval(Rect.fromCenter(center: p, width: 8, height: 5.5), _f(_K.yellow));
    c.drawOval(Rect.fromCenter(center: p, width: 8, height: 5.5), _s(_K.ink0, 1));
    c.drawLine(p + const Offset(-.6, -2.5), p + const Offset(-.6, 2.5), _s(_K.ink0, 1.2));
  }
}

void _sPopcorn(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h / 2), n = _amt(v, 6, 36), popped = _cl(.25 + v.energy * 1.1);
  c.drawLine(o + const Offset(30, 18), o + const Offset(70, 46), _s(_K.ink0, 7));
  c.drawCircle(o, 34, _f(_K.ink0));
  c.drawCircle(o, 34, _s(const Color(0xFF3A3A42), 3));
  for (var i = 0; i < n; i++) {
    final home = o + _rnd2(v.seed, i) * 24;
    if (_hs(v.seed, i + 90) < popped) {
      final shake = v.energy * 2 * math.sin(v.t * 25 + i);
      final p = home + _dir(_hs(v.seed, i + 3) * math.pi * 2) * (6 + _hs(v.seed, i + 4) * v.a * 60) + Offset(shake, -shake);
      for (var j = 0; j < 3; j++) {
        c.drawCircle(p + _dir(j * 2.1 + i) * 2.4, 3.2, _f(_K.cream));
      }
      c.drawCircle(p, 1.4, _f(_K.yellow));
    } else {
      c.drawOval(Rect.fromCenter(center: home, width: 4, height: 3), _f(_K.yellow));
    }
  }
}

void _sPins(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w * .5, h * .55), n = _amt(v, 2, 22), sp = 6 + v.a * 62;
  c.drawPath(_polyPath([Offset(w * .62, 0), Offset(w * .55, h * .4), Offset(w * .7, h * .7), Offset(w * .64, h)], close: false), _s(_K.cyan, 7, .6));
  c.drawOval(Rect.fromLTWH(8, 8, 40, 26), _f(_K.lime, .7));
  for (var x = 14.0; x < w; x += 24) {
    c.drawLine(Offset(x, 0), Offset(x - 8, h), _s(_K.ink1, 1.6, .18));
  }
  for (var y = 12.0; y < h; y += 22) {
    c.drawLine(Offset(0, y), Offset(w, y + 6), _s(_K.ink1, 1.6, .18));
  }
  final pins = [for (var i = 0; i < n; i++) (o + Offset(_rnd2(v.seed, i).dx * sp * 1.2, _rnd2(v.seed, i).dy * sp * .7), i)]..sort((p, q) => p.$1.dy.compareTo(q.$1.dy));
  for (final (p, i) in pins) {
    final col = const [_K.red, _K.blue, _K.violet, _K.orange][i % 4];
    c.drawOval(Rect.fromCenter(center: p, width: 7, height: 2.6), _f(_K.ink0, .25));
    c.drawPath(_polyPath([p, p + const Offset(-3.6, -8), p + const Offset(3.6, -8)]), _f(col));
    c.drawCircle(p + const Offset(0, -10), 4.6, _f(col));
    c.drawCircle(p + const Offset(0, -10), 1.7, _f(_K.white));
  }
}

void _sDarts(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w * .44, h / 2), n = _amt(v, 1, 12), sp = 2 + v.a * 40;
  c.drawCircle(o, 46, _f(_K.ink0));
  for (var i = 0; i < 20; i++) {
    final p = Path()
      ..moveTo(o.dx, o.dy)
      ..arcTo(Rect.fromCircle(center: o, radius: 40), i * math.pi / 10 - math.pi / 20, math.pi / 10, false)
      ..close();
    c.drawPath(p, _f(i.isEven ? _K.cream : const Color(0xFF34343C)));
  }
  c.drawCircle(o, 38, _s(_K.red, 4));
  c.drawCircle(o, 22, _s(_K.lime, 3.5));
  c.drawCircle(o, 6, _f(_K.lime));
  c.drawCircle(o, 3, _f(_K.red));
  for (var i = 0; i < n; i++) {
    final hit = o + _rnd2(v.seed, i) * sp, tail = hit + const Offset(13, -11), col = _K.vivid[(i + 1) % 6];
    c.drawLine(hit, tail, _s(_K.white, 1.6));
    c.drawPath(_polyPath([tail, tail + const Offset(9, -1), tail + const Offset(3, -5)]), _f(col));
    c.drawPath(_polyPath([tail, tail + const Offset(1, -9), tail + const Offset(5, -3)]), _f(_mix(col, _K.ink0, .3)));
    c.drawCircle(hit, 1.4, _f(_K.white));
  }
}

void _sGlobe(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h * .45), R = 40.0, n = _amt(v, 8, 70), mixUp = _cl(v.energy * 1.3 + .12);
  c.drawPath(_polyPath([Offset(o.dx - 30, o.dy + 32), Offset(o.dx + 30, o.dy + 32), Offset(o.dx + 38, h - 6), Offset(o.dx - 38, h - 6)]), _f(_K.red));
  c.drawRect(Rect.fromLTRB(o.dx - 34, h - 16, o.dx + 34, h - 12), _f(_K.yellow));
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: o, radius: R)));
  c.drawCircle(o, R, _f(_K.blue, .45));
  c.drawOval(Rect.fromCenter(center: o + const Offset(0, 34), width: 90, height: 26), _f(_K.cream));
  c.drawRect(Rect.fromCenter(center: o + const Offset(-6, 16), width: 16, height: 12), _f(_K.violet));
  c.drawPath(_polyPath([o + const Offset(-16, 10), o + const Offset(4, 10), o + const Offset(-6, 1)]), _f(_K.red));
  c.drawPath(_polyPath([o + const Offset(14, 22), o + const Offset(26, 22), o + const Offset(20, 2)]), _f(_K.lime));
  for (var i = 0; i < n; i++) {
    final rest = o + Offset((_hs(v.seed, i) - .5) * 70, 26 + _hs(v.seed, i + 8) * 6);
    final fly = o + _rnd2(v.seed, i + 100) * (10 + v.a * 30) + Offset(math.sin(v.t + i) * 3, ((v.t * 6 + i * 7) % 20) - 10);
    c.drawCircle(Offset.lerp(rest, fly, mixUp)!, 1.3, _f(_K.white));
  }
  c.restore();
  c.drawCircle(o, R, _s(_K.cream, 1.6, .7));
  c.drawArc(Rect.fromCircle(center: o, radius: R - 6), -2.6, .9, false, _s(_K.white, 2.4, .6));
}

void _sShards(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, pic = Rect.fromCenter(center: Offset(w / 2, h / 2), width: 96, height: 66);
  const cols = 5, rows = 4;
  final tw = pic.width / cols, th = pic.height / rows;
  c.drawRect(pic.inflate(2), _s(_K.cream, 1, .25));
  for (var j = 0; j < rows; j++) {
    for (var i = 0; i < cols; i++) {
      final id = j * cols + i, tile = Rect.fromLTWH(pic.left + i * tw, pic.top + j * th, tw, th);
      final moved = _hs(v.seed, id) < v.b;
      final d = moved ? _rnd2(v.seed, id + 30) * (6 + v.a * 40) : Offset.zero, rot = moved ? (_hs(v.seed, id + 60) - .5) * v.a * 1.2 : 0.0;
      c.save();
      c.translate(tile.center.dx + d.dx, tile.center.dy + d.dy);
      c.rotate(rot);
      c.translate(-tile.center.dx, -tile.center.dy);
      c.clipRect(tile);
      _pic(c, pic);
      c.restore();
    }
  }
}

void _sLeaves(Canvas c, Size s, PV v) {
  final h = s.height, crown = Offset(38, h * .4), m = 34, gone = (v.b * m).round();
  c.drawLine(Offset(crown.dx, h), crown, _s(const Color(0xFF8A5638), 6));
  for (final (d, r) in [(const Offset(-12, 4), 16.0), (const Offset(10, 2), 17.0), (const Offset(0, -12), 18.0)]) {
    c.drawCircle(crown + d, r, _f(const Color(0xFF3F7A2A)));
  }
  void leaf(Offset p, double a, Color col) {
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(a);
    c.drawOval(Rect.fromCenter(center: Offset.zero, width: 8, height: 4.4), _f(col));
    c.restore();
  }

  const cols = [_K.lime, _K.yellow, _K.orange, _K.red];
  for (var i = gone; i < m; i++) {
    leaf(crown + _rnd2(3, i) * 22, i * 1.1, cols[i % 2]);
  }
  for (var i = 0; i < gone; i++) {
    final dist = 26 + _hs(v.seed, i) * (10 + v.a * 120);
    final p = crown + Offset(dist + math.sin(v.t * 1.4 + i) * 3, (_hs(v.seed, i + 40) - .45) * (14 + v.a * 60) + math.sin(v.t * 2 + i * 2) * 4);
    leaf(p, v.t * (1 + _h(i)) + i, cols[i % 4]);
  }
  for (var i = 0; i < 3; i++) {
    final y = h * (.25 + i * .25), x = 70 + (v.t * 30 + i * 40) % 90;
    c.drawArc(Rect.fromLTWH(x - 20, y, 26, 10), math.pi, math.pi * .8, false, _s(_K.cream, 1.3, .35));
  }
}

void _sSheep(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w / 2, h / 2), n = _amt(v, 1, 16);
  c.drawCircle(o + const Offset(28, -30), 46, _f(const Color(0xFF8ACF55)));
  final sheep = [
    for (var i = 0; i < n; i++)
      Offset((o.dx + _rnd2(v.seed, i).dx * (10 + v.a * 66)).clamp(10.0, w - 10), (o.dy + _rnd2(v.seed, i).dy * (8 + v.a * 46)).clamp(10.0, h - 8))
  ]..sort((p, q) => p.dy.compareTo(q.dy));
  for (var i = 0; i < sheep.length; i++) {
    final p = sheep[i], face = _h(i + v.seed) > .5 ? 1.0 : -1.0, bob = math.sin(v.t * 3 + i) * .6;
    c.drawLine(p + const Offset(-3, 3), p + const Offset(-3, 7), _s(_K.ink0, 1.6));
    c.drawLine(p + const Offset(3, 3), p + const Offset(3, 7), _s(_K.ink0, 1.6));
    for (final d in [const Offset(-4, 0), const Offset(0, -2), const Offset(4, 0), const Offset(0, 2)]) {
      c.drawCircle(p + d + Offset(0, bob), 4.4, _f(_K.white));
    }
    c.drawOval(Rect.fromCenter(center: p + Offset(face * 7, -1 + bob), width: 5.4, height: 4.4), _f(_K.ink0));
  }
  c.drawLine(o + const Offset(0, -10), o + const Offset(0, 6), _s(_K.ink0, 3));
  c.drawCircle(o + const Offset(0, -13), 3.5, _f(_K.orange));
  final crook = Path()
    ..moveTo(o.dx + 6, o.dy + 8)
    ..lineTo(o.dx + 6, o.dy - 14)
    ..arcToPoint(Offset(o.dx + 12, o.dy - 14), radius: const Radius.circular(3));
  c.drawPath(crook, _s(_K.ink0, 1.6));
}
