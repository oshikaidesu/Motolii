part of 'op_01.dart';

// Scatter: amount = blue, spread = green, seed = red. White is the subject being scattered.

const _scatterSpecs = <_S>[
  _S('seed sower', 'character|effect|flick|drawing|1', _G.flick, _s01, a: .45, b: .5),
  _S('dandelion', 'plant|effect|rub|drawing|2', _G.rub, _s02, a: .4, b: .5),
  _S('choke target', 'diagram|mechanism|drag|diagram|1', _G.drag, _s03, a: .45, b: .45),
  _S('galaxy arms', 'cosmic|effect|spin|drawing|3', _G.spin, _s04, a: .5, b: .5),
  _S('iso dice', 'isometric|mechanism|flick|drawing|3', _G.flick, _s05, a: .5, b: .55),
  _S('galton board', 'machine|mechanism|drag|diagram|2', _G.drag, _s06, a: .5, b: .5),
  _S('startled pigeons', 'animal|reason|drag|drawing|2', _G.drag, _s07, a: .4, b: .55),
  _S('spray path', 'tool|effect|draw|drawing|2', _G.draw, _s08, a: .5, b: .5),
  _S('seed numeral', 'typographic|value|drag|numeral|1', _G.drag, _s09, a: .3, b: .5, c: .5),
  _S('signal flow', 'diagram|mechanism|drag|diagram|1', _G.drag, _s10, a: .5, b: .5),
  _S('bell plot', 'device|mechanism|pinch|diagram|2', _G.pinch, _s11, a: .5, b: .5),
  _S('strike', 'sport|effect|flick|drawing|3', _G.flick, _s12, a: .6, b: .6),
  _S('confetti cannon', 'machine|pushed|flick|drawing|4', _G.flick, _s13, a: .7, b: .6),
  _S('night windows', 'landscape|effect|rub|drawing|2', _G.rub, _s14, a: .45, b: .6),
  _S('rain on pond', 'landscape|effect|drag|drawing|2', _G.drag, _s15, a: .5, b: .55),
  _S('bee swarm', 'animal|effect|spin|drawing|3', _G.spin, _s16, a: .5, b: .5),
  _S('iso placement', 'isometric|mechanism|drag|diagram|2', _G.drag, _s17, a: .45, b: .55),
  _S('loose letters', 'typographic|effect on sample|drag|drawing|2', _G.drag, _s18, a: .45, b: .35),
  _S('constellation', 'cosmic|reason|pinch|diagram|3', _G.pinch, _s19, a: .45, b: .6),
  _S('big bang', 'cosmic|pushed|rub|drawing|5', _G.rub, _s20, a: .75, b: .9),
];

int _seed(_P p) => (p.c * 100).round();

// 1 seed sower: a figure throws seeds in an arc; how many = amount, how wide = spread; double-tap throws a new hand.
void _s01(Canvas c, _P p, double t) {
  const g = 104.0;
  _ln(c, const Offset(0, g), const Offset(_pw, g), _dim, 1);
  final sw = math.sin(t * 2.2);
  _stick(c, const Offset(22, g), 44, _white, arm: .35 + sw * .12, lean: .15);
  c.drawArc(Rect.fromCenter(center: const Offset(16, g - 26), width: 12, height: 14), 0, math.pi, false, _s(_white, 1));
  final n = 4 + (p.a * 40).round(), sd = _seed(p);
  const hand = Offset(36, 52);
  for (var i = 0; i < n; i++) {
    final ang = -.5 + (_h(i, sd) - .5) * (.2 + p.b * 1.6);
    final v = 40 + _h(i, sd + 1) * 50;
    final ph = ((t * .45 + _h(i, sd + 2)) % 1);
    final x = hand.dx + math.cos(ang) * v * ph * 2.2, y = hand.dy + math.sin(ang) * v * ph * 2 + 90 * ph * ph;
    if (y >= g) {
      final lx = hand.dx + math.cos(ang) * v * 2.2 * .8;
      _pl(c, [Offset(lx - 2, g - 4), Offset(lx, g), Offset(lx + 2, g - 4)], _green, .8);
      continue;
    }
    _dot(c, Offset(x, y), 1.3, i.isEven ? _blue : _white);
  }
  _lb(c, 'SEEDS', const Offset(112, 8), _blue);
  _num(c, '${n.clamp(0, 99)}', const Offset(112, 16), 22, _blue);
}

// 2 dandelion: rub (blow) to release seeds; spread = how far the wind carries them.
void _s02(Canvas c, _P p, double t) {
  const head = Offset(42, 52);
  final stem = Path()
    ..moveTo(36, 116)
    ..quadraticBezierTo(30, 84, head.dx, head.dy + 4);
  c.drawPath(stem, _s(_white, 1.1));
  _pl(c, const [Offset(34, 104), Offset(22, 96), Offset(28, 104)], _white, 1);
  _ring(c, head, 3, _white, 1);
  const n = 22;
  final gone = (p.a * n).round(), sd = _seed(p);
  for (var i = 0; i < n; i++) {
    final a = i * 2 * math.pi / n;
    final u = Offset(math.cos(a), math.sin(a));
    if (_h(i, sd) * n >= n - gone) {
      final ph = (t * .25 + _h(i, sd + 1)) % 1;
      final q = head + u * 16 + Offset(ph * (40 + p.b * 90), -ph * 30 + math.sin(ph * 9 + i) * 5 + (_h(i, sd + 2) - .5) * p.b * 70 * ph);
      _ln(c, q, q + const Offset(-3, 4), _white, .8);
      for (final d in [-.6, 0.0, .6]) {
        _ln(c, q, q + Offset(math.cos(-math.pi / 2 + d), math.sin(-math.pi / 2 + d)) * 4, _green, .7);
      }
    } else {
      _ln(c, head + u * 3, head + u * 16, _a(_white, .8), .7);
      _ring(c, head + u * 17, 1.4, _green, .7);
    }
  }
  for (var k = 0; k < 3; k++) {
    final y = 30.0 + k * 16, x0 = (t * 30 + k * 40) % 200 - 40;
    _dash(c, Offset(x0, y), Offset(x0 + 24 + p.b * 20, y), _a(_blue, .7), on: 4, off: 3);
  }
  _lb(c, 'WIND', const Offset(112, 102), _green);
  _num(c, _nn(p.b), const Offset(112, 84), 18, _green);
}

// 3 choke target: pellets as crosses; spread is the choke, amount the pellet count.
void _s03(Canvas c, _P p, double t) {
  const o = Offset(66, 60);
  for (var k = 1; k <= 5; k++) {
    _ring(c, o, k * 10.0, k == 5 ? _white : _a(_white, .35), .9);
  }
  _ln(c, const Offset(66, 4), const Offset(66, 116), _dim, .8);
  _ln(c, const Offset(10, 60), const Offset(122, 60), _dim, .8);
  _ring(c, o, 6 + p.b * 44, _green, 1);
  final n = 3 + (p.a * 30).round(), sd = _seed(p);
  final rec = (t % 3) / 3;
  for (var i = 0; i < n; i++) {
    if (i / n > rec * 2) continue;
    final r = math.sqrt(_h(i, sd)) * (6 + p.b * 44), a = _h(i, sd + 1) * 2 * math.pi;
    final q = o + Offset(math.cos(a), math.sin(a)) * r;
    _ln(c, q - const Offset(2, 2), q + const Offset(2, 2), _blue, 1);
    _ln(c, q - const Offset(2, -2), q + const Offset(2, -2), _blue, 1);
  }
  _lb(c, 'CHOKE', const Offset(126, 10), _green);
  _num(c, _nn(1 - p.b), const Offset(126, 18), 18, _green);
  _lb(c, 'SEED', const Offset(126, 92), _red);
  _lb(c, '$sd', const Offset(126, 102), _red, 8);
}

// 4 galaxy arms: stars along spiral arms, arm thickness = spread, count = amount.
void _s04(Canvas c, _P p, double t) {
  const o = Offset(78, 60);
  final n = 30 + (p.a * 160).round(), sd = _seed(p), rot = t * .15 + p.ang * .3;
  for (var i = 0; i < n; i++) {
    final arm = i % 3, u = _h(i, sd);
    final r = 4 + u * 50;
    final a = arm * 2 * math.pi / 3 + u * 4.2 + rot + (_h(i, sd + 1) - .5) * p.b * 1.6;
    final q = o + Offset(math.cos(a) * r, math.sin(a) * r * .62);
    _dot(c, q, .5 + (1 - u) * .9, u < .2 ? _white : (arm == 0 ? _blue : arm == 1 ? _green : _purple));
  }
  for (var arm = 0; arm < 3; arm++) {
    _pl(c, [for (var u = 0.0; u <= 1; u += .05) o + Offset(math.cos(arm * 2 * math.pi / 3 + u * 4.2 + rot) * (4 + u * 50), math.sin(arm * 2 * math.pi / 3 + u * 4.2 + rot) * (4 + u * 50) * .62)], _a(_dim, .9), .7);
  }
  _ring(c, o, 3, _white, 1);
  _lb(c, 'STARS ${n.clamp(0, 999)}', const Offset(8, 8), _blue, 7);
  _lb(c, 'ARM ${_nn(p.b)}', const Offset(8, 104), _green, 7);
}

// 5 iso dice: wire dice bounce out from the hand; amount = how many, spread = how far they roll; top face shows the seed.
void _s05(Canvas c, _P p, double t) {
  const o = Offset(78, 30);
  _isoGrid(c, o, 8, 8, _dim);
  final n = 1 + (p.a * 7).round(), sd = _seed(p);
  final cyc = (t % 3.2) / 3.2;
  for (var i = 0; i < n; i++) {
    final tx = 32 + (_h(i, sd) - .5) * p.b * 60, ty = 32 + (_h(i, sd + 1) - .5) * p.b * 60;
    final k = _ease(_cl(cyc * 1.6 - i * .05));
    final x = 32 + (tx - 32) * k - 4, y = 32 + (ty - 32) * k - 4;
    final z = (math.sin(k * math.pi * 3).abs() * (1 - k)) * 18;
    _isoBox(c, o, x, y, z, 8, 8, 8, i == 0 ? _white : _blue, .9);
    final face = 1 + (_h(i, sd + 3) * 6).floor();
    final top = _iso(o, x + 4, y + 4, z + 8);
    for (var d = 0; d < face; d++) {
      final ang = d * 2 * math.pi / face;
      _dot(c, face == 1 ? top : top + Offset(math.cos(ang) * 2.4, math.sin(ang) * 1.3), .8, _red);
    }
  }
  _lb(c, 'ROLL ${_nn(p.b)}', const Offset(8, 104), _green, 7);
  _num(c, '$n', const Offset(8, 6), 26, _blue);
}

// 6 galton board: balls fall through pegs; spread = peg gap, the bins draw the distribution.
void _s06(Canvas c, _P p, double t) {
  const top = Offset(78, 10), rows = 7;
  final gap = 5 + p.b * 6;
  for (var r = 0; r < rows; r++) {
    for (var k = 0; k <= r; k++) {
      _dot(c, top + Offset((k - r / 2) * gap * 2, 10 + r * 9.0), 1, _white);
    }
  }
  final n = 3 + (p.a * 14).round(), sd = _seed(p);
  final bins = List<int>.filled(rows + 1, 0);
  for (var i = 0; i < 60; i++) {
    var k = 0;
    for (var r = 0; r < rows; r++) {
      if (_h(i * 13 + r, sd) > .5) k++;
    }
    bins[k]++;
  }
  for (var i = 0; i < n; i++) {
    final ph = (t * .5 + i / n) % 1;
    final pts = <Offset>[top];
    var x = 0.0;
    for (var r = 0; r < rows; r++) {
      x += _h(i * 31 + r, sd + 5) > .5 ? .5 : -.5;
      pts.add(top + Offset(x * gap * 2, 14 + r * 9.0));
    }
    final idx = (ph * (pts.length - 1)).floor();
    final q = Offset.lerp(pts[idx], pts[math.min(idx + 1, pts.length - 1)], (ph * (pts.length - 1)) % 1)!;
    if (i == 0) _pl(c, pts, _a(_blue, .4), .7);
    _ring(c, q, 1.8, _blue, 1);
  }
  const by = 112.0;
  _ln(c, Offset(top.dx - (rows / 2 + .5) * gap * 2, by), Offset(top.dx + (rows / 2 + .5) * gap * 2, by), _dim, 1);
  final hist = <Offset>[];
  for (var k = 0; k <= rows; k++) {
    final x = top.dx + (k - rows / 2) * gap * 2;
    _ln(c, Offset(x - gap, by), Offset(x - gap, by - 6), _dim, .7);
    hist.add(Offset(x, by - 4 - bins[k] * 1.3));
  }
  _pl(c, hist, _green, 1);
  for (final q in hist) {
    _dot(c, q, 1.2, _green);
  }
}

// 7 startled pigeons: the reason you scatter -- break a tidy flock. Spread pushes them outward and they take off.
void _s07(Canvas c, _P p, double t) {
  const o = Offset(78, 80), g = 98.0;
  _ln(c, const Offset(0, g), const Offset(_pw, g), _dim, 1);
  for (var k = 0; k < 5; k++) {
    _dot(c, o + Offset((k - 2) * 3.0, 16), .9, _red);
  }
  final n = 3 + (p.a * 8).round(), sd = _seed(p);
  for (var i = 0; i < n; i++) {
    final a = -math.pi * (.08 + .84 * _h(i, sd));
    final d = 10 + _h(i, sd + 1) * 16 + p.b * 50;
    final fly = p.b > .35 + _h(i, sd + 2) * .5;
    final home = Offset(16 + (n == 1 ? .5 : i / (n - 1)) * 124, g - 6);
    final q = fly ? home + Offset(math.cos(a) * d * .8, math.sin(a) * d * .9 - 6) : home;
    final dir = fly ? (math.cos(a) >= 0 ? 1.0 : -1.0) : (_h(i, sd + 5) > .5 ? 1.0 : -1.0);
    c.save();
    c.translate(q.dx, q.dy);
    c.scale(1.6);
    if (fly) {
      final fl = math.sin(t * 14 + i * 2) * 4;
      _pl(c, [Offset(-6, -fl), Offset.zero, Offset(6, -fl)], _white, .8);
      _dot(c, Offset(dir * 1.5, -.5), .9, _white);
    } else {
      c.drawArc(Rect.fromCenter(center: Offset.zero, width: 12, height: 8), math.pi, math.pi, false, _s(_white, .8));
      _ln(c, Offset(-6 * dir, 0), Offset(6 * dir, 0), _white, .8);
      _ring(c, Offset(6 * dir, -4), 2, _white, .7);
      _ln(c, Offset(8 * dir, -4), Offset(10 * dir, -3), _green, .8);
      _ln(c, const Offset(-1, 0), const Offset(-1, 4), _dim, .7);
      _ln(c, const Offset(1, 0), const Offset(1, 4), _dim, .7);
    }
    c.restore();
  }
  _lb(c, 'SCARE', const Offset(8, 8), _green);
  _num(c, _nn(p.b), const Offset(8, 16), 22, _green);
}

// 8 spray path: draw a stroke; the can sprays dots around it within the spread.
void _s08(Canvas c, _P p, double t) {
  final path = p.pts.length > 2
      ? p.pts
      : [for (var i = 0; i <= 20; i++) Offset(20 + i * 5.6, 64 + math.sin(i * .45) * 24)];
  _pl(c, path, _a(_white, .35), .8);
  final n = 40 + (p.a * 240).round(), sd = _seed(p), sp = 2 + p.b * 22;
  for (var i = 0; i < n; i++) {
    final q = path[(_h(i, sd) * path.length).floor() % path.length];
    final r = math.pow(_h(i, sd + 1), 1.6) * sp, a = _h(i, sd + 2) * 2 * math.pi;
    _dot(c, q + Offset(math.cos(a) * r, math.sin(a) * r), .7, r < sp * .5 ? _blue : _a(_green, .8));
  }
  final tip = p.touch ?? path.last;
  final can = tip + const Offset(10, -26);
  c.drawRect(Rect.fromLTWH(can.dx, can.dy, 10, 18), _s(_white, 1));
  c.drawRect(Rect.fromLTWH(can.dx + 3, can.dy - 4, 4, 4), _s(_white, 1));
  _ln(c, can + const Offset(3, -3), can + const Offset(-1, -3), _white, 1);
  _lb(c, 'SPRAY ${_nn(p.b)}', const Offset(8, 8), _green, 7);
}

// 9 seed numeral: the seed is the hero number; the strip under it previews the same twelve dots re-thrown.
void _s09(Canvas c, _P p, double t) {
  final sd = (p.a * 99).round();
  _lb(c, 'SEED', const Offset(10, 10), _red);
  _num(c, sd.toString().padLeft(2, '0'), const Offset(8, 18), 58, _red);
  _lb(c, 'AMT', const Offset(104, 10), _blue, 7);
  _num(c, _nn(p.b), const Offset(104, 18), 20, _blue);
  _lb(c, 'SPR', const Offset(104, 44), _green, 7);
  _num(c, _nn(p.c), const Offset(104, 52), 20, _green);
  final n = 3 + (p.b * 12).round();
  for (var i = 0; i < 15; i++) {
    final base = Offset(12 + i * 9.0, 100);
    _dotted(c, base + const Offset(0, -6), base + const Offset(0, 6), _dim, 3);
    if (i >= n) continue;
    final q = base + Offset((_h(i, sd) - .5) * p.c * 14, (_h(i, sd + 1) - .5) * p.c * 18);
    _ring(c, q, 1.8, _white, 1);
  }
}

// 10 signal flow: one shape -> amount amp -> spread fan -> seed die -> copies wheel.
void _s10(Canvas c, _P p, double t) {
  const y = 50.0;
  c.drawRect(const Rect.fromLTWH(6, y - 10, 20, 20), _s(_white, 1));
  _pl(c, const [Offset(16, y - 5), Offset(21, y + 4), Offset(11, y + 4)], _white, .9, true);
  _pl(c, const [Offset(34, y - 11), Offset(54, y), Offset(34, y + 11)], _blue, 1.1, true);
  _lb(c, _nn(p.a), const Offset(36, y - 4), _blue, 8);
  final sp = .1 + p.b * .7;
  for (var k = -2; k <= 2; k++) {
    final a = k / 2 * sp;
    _ln(c, const Offset(60, y), Offset(60 + math.cos(a) * 26, y + math.sin(a) * 26), _green, .9);
  }
  _isoBox(c, const Offset(104, y - 6), 0, 0, 0, 10, 10, 10, _red, .9);
  _dot(c, _iso(const Offset(104, y - 6), 5, 5, 10), 1, _red);
  const w = Offset(136, y);
  _ring(c, w, 12, _white, 1);
  final n = 2 + (p.a * 10).round(), sd = _seed(p);
  for (var i = 0; i < n; i++) {
    final a = _h(i, sd) * 2 * math.pi, r = 2 + _h(i, sd + 1) * 8 * (.3 + p.b * .7);
    _dot(c, w + Offset(math.cos(a), math.sin(a)) * r, 1.1, _blue);
  }
  for (final (a, b) in const [(26.0, 34.0), (54.0, 60.0), (90.0, 96.0), (114.0, 124.0)]) {
    _ln(c, Offset(a, y), Offset(b, y), _dim, 1);
  }
  final px = 6 + ((t * 40) % 140);
  _dot(c, Offset(px, y + 22), 1.4, _white);
  _dotted(c, const Offset(6, y + 22), const Offset(148, y + 22), _dim, 4);
  for (final (s, x, col) in const [('SRC', 16.0, _white), ('AMT', 44.0, _blue), ('SPRD', 74.0, _green), ('SEED', 104.0, _red), ('OUT', 136.0, _white)]) {
    _lb(c, s, Offset(x, y + 30), col, 6, .5);
  }
}

// 11 bell plot (Max device): points with a gaussian envelope; dotted verticals at +-sigma.
void _s11(Canvas c, _P p, double t) {
  const x0 = 10.0, x1 = 146.0, cx = 78.0, base = 96.0;
  _ln(c, const Offset(x0, base), const Offset(x1, base), _dim, 1);
  final sg = 4 + p.b * 40;
  _pl(c, [for (var x = x0; x <= x1; x += 2) Offset(x, base - 60 * math.exp(-math.pow((x - cx) / sg, 2) / 2))], _green, 1);
  _dotted(c, Offset(cx - sg, base), Offset(cx - sg, base - 60 * math.exp(-.5)), _green);
  _dotted(c, Offset(cx + sg, base), Offset(cx + sg, base - 60 * math.exp(-.5)), _green);
  _dotted(c, const Offset(cx, base), const Offset(cx, 30), _dim);
  final n = 6 + (p.a * 60).round(), sd = _seed(p);
  for (var i = 0; i < n; i++) {
    final u1 = math.max(1e-4, _h(i, sd)), u2 = _h(i, sd + 1);
    final z = math.sqrt(-2 * math.log(u1)) * math.cos(2 * math.pi * u2);
    final x = cx + z * sg;
    if (x < x0 || x > x1) continue;
    final yy = base - 4 - _h(i, sd + 2) * 50 * math.exp(-z * z / 2) - math.sin(t * 2 + i) * .8;
    _dot(c, Offset(x, yy), 1.1, _blue);
  }
  _lb(c, 'σ', Offset(cx + sg + 2, base - 44), _green, 8);
  _lb(c, 'N ${n.clamp(0, 99)}', const Offset(x0, 8), _blue, 7);
  _lb(c, 'SEED $sd', const Offset(x1, 104), _red, 7, 1);
}

// 12 strike: flick the ball; the pins fly as far as the spread lets them.
void _s12(Canvas c, _P p, double t) {
  const vp = Offset(78, 20);
  _ln(c, const Offset(30, 118), vp + const Offset(-14, 0), _dim, 1);
  _ln(c, const Offset(126, 118), vp + const Offset(14, 0), _dim, 1);
  final cyc = (t % 3) / 3;
  final hit = _cl((cyc - .35) / .5);
  final by = 112 - _ease(_cl(cyc / .35)) * 74;
  if (cyc < .4) {
    final r = 7 - (112 - by) / 74 * 4;
    _ring(c, Offset(78, by), r, _white, 1.1);
    _dot(c, Offset(78 - r * .3, by - r * .3), .8, _white);
  }
  final n = 3 + (p.a * 7).round(), sd = _seed(p);
  for (var i = 0; i < n; i++) {
    final row = i < 1 ? 0 : (i < 3 ? 1 : (i < 6 ? 2 : 3));
    final col = i - [0, 1, 3, 6][row];
    final home = vp + Offset((col - row / 2) * 14, 22 + row * 8.0);
    final ang = (_h(i, sd) - .5) * 2 * math.pi;
    final d = hit * (10 + _h(i, sd + 1) * 50) * p.b;
    final q = home + Offset(math.cos(ang) * d * 1.3, math.sin(ang) * d * .6 - math.sin(hit * math.pi) * 14 * p.b);
    final rot = hit * (_h(i, sd + 2) - .5) * 6 * p.b;
    c.save();
    c.translate(q.dx, q.dy);
    c.rotate(rot);
    c.scale(1.5);
    _ring(c, const Offset(0, -7), 1.8, _white, .9);
    _pl(c, const [Offset(-1.2, -5), Offset(-2.5, 0), Offset(-1.5, 3), Offset(1.5, 3), Offset(2.5, 0), Offset(1.2, -5)], _white, .9);
    _ln(c, const Offset(-1.6, -3), const Offset(1.6, -3), _red, .8);
    c.restore();
  }
  _lb(c, 'PINS', const Offset(8, 8), _blue);
  _num(c, '$n', const Offset(8, 16), 22, _blue);
}

// 13 confetti cannon (pushed): the panel fills with tumbling strips; more = more, spread = cone.
void _s13(Canvas c, _P p, double t) {
  const m = Offset(18, 102);
  c.save();
  c.translate(m.dx, m.dy);
  c.rotate(-.75);
  c.drawRRect(RRect.fromLTRBR(0, -6, 28, 6, const Radius.circular(2)), _s(_white, 1.1));
  _ln(c, const Offset(28, -7), const Offset(28, 7), _white, 1.4);
  c.restore();
  _ring(c, m + const Offset(4, 8), 6, _white, 1);
  _ln(c, m + const Offset(4, 8), m + const Offset(8, 4), _white, .8);
  final n = 30 + (p.a * 230).round(), sd = _seed(p);
  const cols = [_blue, _green, _white, _red, _purple];
  for (var i = 0; i < n; i++) {
    final ph = (t * .35 + _h(i, sd)) % 1;
    final ang = -.75 + (_h(i, sd + 1) - .5) * (.2 + p.b * 1.8);
    final v = 70 + _h(i, sd + 2) * 120;
    final q = m + const Offset(20, -20) + Offset(math.cos(ang) * v * ph, math.sin(ang) * v * ph + 110 * ph * ph);
    final rot = t * (3 + _h(i, sd + 3) * 6) + i;
    final u = Offset(math.cos(rot), math.sin(rot)) * 2.5;
    _ln(c, q - u, q + u, cols[i % 5], 1.1);
  }
  _num(c, '${(n * 3).clamp(0, 999)}', Offset(_pw - 6, 6), 26, _a(_blue, .9), 1);
}

// 14 night windows: rub across the skyline; lit windows are scattered across the buildings within the spread.
void _s14(Canvas c, _P p, double t) {
  const g = 112.0;
  final hs = [52.0, 74.0, 40.0, 88.0, 60.0, 46.0, 70.0];
  final ws = [20.0, 22.0, 18.0, 22.0, 20.0, 18.0, 22.0];
  var x = 4.0;
  final sd = _seed(p);
  const cx = 78.0;
  for (var b = 0; b < hs.length; b++) {
    final top = g - hs[b];
    _pl(c, [Offset(x, g), Offset(x, top), Offset(x + ws[b] - 2, top), Offset(x + ws[b] - 2, g)], _white, 1);
    if (b == 3) _ln(c, Offset(x + 10, top), Offset(x + 10, top - 10), _white, .9);
    for (var wy = top + 6; wy < g - 6; wy += 7) {
      for (var wx = x + 3; wx < x + ws[b] - 6; wx += 6) {
        final i = (wx * 7 + wy * 13).toInt();
        final inRange = ((wx - cx).abs() / 80) < p.b;
        final lit = inRange && _h(i, sd) < p.a * .8 && math.sin(t * .3 + _h(i, 5) * 20) > -.8;
        if (lit) {
          c.drawRect(Rect.fromLTWH(wx, wy, 2.6, 3), _f(_h(i, 2) > .5 ? _green : _blue));
        } else {
          c.drawRect(Rect.fromLTWH(wx, wy, 2.6, 3), _s(_dim, .5));
        }
      }
    }
    x += ws[b];
  }
  c.drawArc(Rect.fromCircle(center: const Offset(130, 18), radius: 7), -1.2, 3.6, false, _s(_white, 1));
  _lb(c, 'LIT ${_nn(p.a)}', const Offset(6, 8), _blue, 7);
}

// 15 rain on pond: ripples appear at scattered spots inside the spread ring.
void _s15(Canvas c, _P p, double t) {
  const o = Offset(78, 76);
  c.drawOval(Rect.fromCenter(center: o, width: 140, height: 56), _s(_white, 1));
  c.drawOval(Rect.fromCenter(center: o, width: 20 + p.b * 120, height: 8 + p.b * 48), _s(_a(_green, .7), .8));
  final n = 3 + (p.a * 18).round(), sd = _seed(p);
  for (var i = 0; i < n; i++) {
    final ph = (t * .5 + _h(i, sd)) % 1;
    final cyc = ((t * .5 + _h(i, sd)) / 1).floor();
    final a = _h(i + cyc * 97, sd + 1) * 2 * math.pi, r = math.sqrt(_h(i + cyc * 97, sd + 2));
    final q = o + Offset(math.cos(a) * r * (10 + p.b * 60), math.sin(a) * r * (4 + p.b * 24));
    if (ph < .3) {
      final y = q.dy - (1 - ph / .3) * 60;
      _ln(c, Offset(q.dx, y - 5), Offset(q.dx, y), _blue, 1);
    } else {
      final k = (ph - .3) / .7;
      for (var j = 0; j < 2; j++) {
        final kk = _cl(k - j * .25);
        if (kk <= 0) continue;
        c.drawOval(Rect.fromCenter(center: q, width: kk * 22, height: kk * 8), _s(_a(_white, 1 - kk), .9));
      }
    }
  }
  _lb(c, 'DROPS', const Offset(8, 8), _blue);
  _num(c, '$n', const Offset(8, 16), 22, _blue);
}

// 16 bee swarm: bees loop around the hive; spread = swarm radius, amount = bees. Spin to stir them.
void _s16(Canvas c, _P p, double t) {
  const h = Offset(78, 86);
  for (var k = 0; k < 4; k++) {
    final w = 30.0 - k * 6;
    c.drawArc(Rect.fromCenter(center: h + Offset(0, -k * 7.0), width: w, height: 14), math.pi, math.pi, false, _s(_white, 1));
    _ln(c, h + Offset(-w / 2, -k * 7.0), h + Offset(w / 2, -k * 7.0), _white, 1);
  }
  c.drawArc(Rect.fromCenter(center: h + const Offset(0, 2), width: 7, height: 7), math.pi, math.pi, false, _s(_white, 1));
  _ln(c, const Offset(40, 94), const Offset(116, 94), _dim, 1);
  final n = 3 + (p.a * 22).round(), sd = _seed(p);
  for (var i = 0; i < n; i++) {
    final r = 10 + _h(i, sd) * (8 + p.b * 60);
    final sp = (.6 + _h(i, sd + 1)) * (_h(i, sd + 2) > .5 ? 1 : -1);
    final a = t * sp + p.ang + _h(i, sd + 3) * 6.28;
    final q = h + Offset(0, -26) + Offset(math.cos(a) * r * 1.2, math.sin(a) * r * .55 + math.sin(t * 6 + i) * 2);
    _dot(c, q, 1.3, _green);
    final fl = math.sin(t * 20 + i).abs() * 2 + 1;
    _ring(c, q + Offset(-1.5, -fl), 1.6, _a(_white, .7), .6);
    _ring(c, q + Offset(1.5, -fl), 1.6, _a(_white, .7), .6);
  }
  _lb(c, 'BEES', const Offset(8, 8), _blue);
  _num(c, '$n', const Offset(8, 16), 22, _blue);
}

// 17 iso placement: wire cubes dropped on tiles inside the spread radius.
void _s17(Canvas c, _P p, double t) {
  const o = Offset(78, 24);
  const n = 8, cs = 9.0;
  _isoGrid(c, o, n, cs, _dim);
  final sd = _seed(p), cnt = 1 + (p.a * 16).round(), rad = 1 + p.b * 4.5;
  final used = <int>{};
  var placed = 0;
  for (var i = 0; i < 200 && placed < cnt; i++) {
    final gx = (_h(i, sd) * n).floor(), gy = (_h(i, sd + 1) * n).floor();
    if (math.sqrt(math.pow(gx - 3.5, 2) + math.pow(gy - 3.5, 2)) > rad) continue;
    if (!used.add(gx * n + gy)) continue;
    placed++;
    final drop = math.max(0.0, math.sin(t * 1.2 + i) * 3);
    _isoBox(c, o, gx * cs + 1.5, gy * cs + 1.5, drop, 6, 6, 6 + _h(i, sd + 4) * 6, placed == 1 ? _white : _blue, .9);
  }
  final rp = Path();
  for (var i = 0; i <= 36; i++) {
    final a = i * math.pi / 18;
    final q = _iso(o, 36 + math.cos(a) * rad * cs, 36 + math.sin(a) * rad * cs, 0);
    i == 0 ? rp.moveTo(q.dx, q.dy) : rp.lineTo(q.dx, q.dy);
  }
  c.drawPath(rp, _s(_green, .9));
  _lb(c, 'N $placed', const Offset(8, 8), _blue, 7);
  _lb(c, 'SEED $sd', const Offset(8, 104), _red, 7);
}

// 18 loose letters: a sample word, each letter thrown off its place; dotted ghosts mark home.
void _s18(Canvas c, _P p, double t) {
  const word = 'SCATTER';
  final sd = _seed(p);
  var x = 12.0;
  for (var i = 0; i < word.length; i++) {
    final tp = _tp(word[i], 26, _white, FontWeight.w200, 0, true);
    final home = Offset(x, 46);
    final moved = _h(i, sd + 7) < p.a;
    if (moved) {
      final off = Offset((_h(i, sd) - .5) * p.b * 90, (_h(i, sd + 1) - .5) * p.b * 80) + Offset(math.sin(t + i) * 1.5, math.cos(t * 1.2 + i) * 1.5);
      final rot = (_h(i, sd + 2) - .5) * p.b * 3;
      _tp(word[i], 26, _dim, FontWeight.w200, 0, true).paint(c, home);
      _dotted(c, home + Offset(tp.width / 2, tp.height / 2), home + off + Offset(tp.width / 2, tp.height / 2), _a(_green, .6), 4);
      c.save();
      c.translate(home.dx + off.dx + tp.width / 2, home.dy + off.dy + tp.height / 2);
      c.rotate(rot);
      _tp(word[i], 26, _blue, FontWeight.w200, 0, true).paint(c, Offset(-tp.width / 2, -tp.height / 2));
      c.restore();
    } else {
      tp.paint(c, home);
    }
    x += tp.width + 2;
  }
  _lb(c, 'LOOSE ${_nn(p.a)}', const Offset(8, 8), _blue, 7);
  _lb(c, 'THROW ${_nn(p.b)}', const Offset(8, 106), _green, 7);
}

// 19 constellation: a new seed draws a new sky and names it; why you scatter -- to get a pattern that looks found, not placed.
void _s19(Canvas c, _P p, double t) {
  final sd = _seed(p), n = 4 + (p.a * 10).round();
  final field = 20 + p.b * 50;
  final stars = [
    for (var i = 0; i < n; i++) Offset(78 + (_h(i, sd) - .5) * field * 2, 54 + (_h(i, sd + 1) - .5) * field * 1.3),
  ];
  for (var i = 0; i < 30; i++) {
    _dot(c, Offset(_h(i, 77) * _pw, _h(i, 78) * _ph), .5, _dim);
  }
  final order = <int>[0];
  final left = List<int>.generate(n - 1, (i) => i + 1);
  while (left.isNotEmpty) {
    final last = stars[order.last];
    left.sort((a, b) => (stars[a] - last).distance.compareTo((stars[b] - last).distance));
    order.add(left.removeAt(0));
  }
  _pl(c, [for (final i in order) stars[i]], _a(_green, .8), .8);
  for (var i = 0; i < n; i++) {
    final tw = 1.4 + math.sin(t * 2 + i) * .5;
    _dot(c, stars[i], tw * .7, _white);
    if (_h(i, sd + 3) > .7) {
      _ln(c, stars[i] - Offset(tw + 2, 0), stars[i] + Offset(tw + 2, 0), _white, .6);
      _ln(c, stars[i] - Offset(0, tw + 2), stars[i] + Offset(0, tw + 2), _white, .6);
    }
  }
  const names = ['LYRA', 'URSA', 'VELA', 'GRUS', 'PYXIS', 'MUSCA'];
  _lb(c, names[sd % names.length], const Offset(8, 100), _red);
  _lb(c, sd.toString().padLeft(2, '0'), const Offset(44, 100), _red, 9);
  _num(c, '$n', Offset(_pw - 8, 6), 22, _blue, 1);
}

// 20 big bang (pushed): everything leaves the centre; streaks run off the frame and the counter goes to 999.
void _s20(Canvas c, _P p, double t) {
  const o = Offset(78, 60);
  final n = 40 + (p.a * 400).round(), sd = _seed(p);
  final ph = (t * .3) % 1;
  for (var i = 0; i < n; i++) {
    final a = _h(i, sd) * 2 * math.pi + (p.b > .5 ? math.sin(i * .3) * p.b : 0);
    final sp = .3 + _h(i, sd + 1) * (.7 + p.b * 2);
    final d = ((ph + _h(i, sd + 2)) % 1) * 200 * sp;
    final u = Offset(math.cos(a), math.sin(a));
    final col = [_blue, _green, _white, _red][i % 4];
    _ln(c, o + u * d, o + u * (d + 2 + d * .12), _a(col, .9), .8);
  }
  _ring(c, o, 3 + (1 - ph) * 4, _white, 1.2);
  _num(c, '${math.min(999, n * 2)}', Offset(_pw - 6, 6), 30, _white, 1);
  _lb(c, 'PARTICLES', Offset(_pw - 6, 38), _blue, 6.5, 1);
}
