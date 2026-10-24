// Part of op_04_ease.dart (cut there by size, as it was).
part of 'op_04_ease.dart';

// 16 Door slam: flick it shut; OVER bangs it past the frame and back.
void _door(Canvas c, Size s, OpV v, double t) {
  const o = Offset(70, 92);
  const sc = 12.0;
  isoGrid(c, o, 4, 4, sc, K.dim);
  pl(c, [iso(o, 0, 0, 0, sc), iso(o, 0, 0, 5, sc), iso(o, 0, 4, 5, sc), iso(o, 0, 4, 0, sc)], _dimW, .9);
  pl(c, [iso(o, 0, 0, 5, sc), iso(o, -1.5, 0, 5, sc)], _dimW, .9);
  final e = ezv(trip(t, 2.2), v);
  final th = (1 - e) * 1.4;
  Offset dp(double r, double z) => iso(o, r * math.sin(th), r * math.cos(th) * 1, z, sc);
  final col = e > 1.0 ? pc(v, 2) : K.white;
  pl(c, [dp(0, 0), dp(3.6, 0), dp(3.6, 4.6), dp(0, 4.6)], col, 1.3, true);
  ci(c, dp(3.0, 2.4), 1.6, col, .9);
  for (var k = 0; k < 4; k++) {
    final r = 1.0 + k * .8;
    final a0 = 1.4, a1 = th;
    if ((a0 - a1).abs() > .05) {
      fn(c, 8, (q) => iso(o, r * math.sin(lr(a0, a1, q)), r * math.cos(lr(a0, a1, q)), 0, sc), al(K.white, .15), .6);
    }
  }
  if (e > 1.0) {
    final hit = iso(o, 0, 3.6, 4.6, sc);
    for (var k = 0; k < 4; k++) {
      ln(c, pol(hit, 6, -.6 - k * .5), pol(hit, 12, -.6 - k * .5), pc(v, 2));
    }
    lb(c, 'SLAM', hit + const Offset(4, -22), pc(v, 2));
  }
  rd(c, v, 0, 'IN', const Offset(110, 8), h: 12);
  rd(c, v, 1, 'OUT', const Offset(110, 40), h: 12);
}

// 17 Family: ten overshoots stacked in isometric depth; the bright one is yours.
void _family(Canvas c, Size s, OpV v, double t) {
  const n = 10;
  final cols = [K.blue, K.purple, K.green, K.white, K.red];
  final mine = (v[2] * (n - 1)).round();
  for (var k = n - 1; k >= 0; k--) {
    final off = Offset(k * 5.0, -k * 4.0);
    final ov = k / (n - 1);
    final col = Color.lerp(cols[(k * 4) ~/ (n - 1)], cols[math.min(4, (k * 4) ~/ (n - 1) + 1)], ((k * 4) / (n - 1)) % 1)!;
    fn(c, 30, (x) => Offset(10 + 96 * x, 104 - 56 * ez(x, v[0], v[1], ov)) + off, k == mine ? K.white : al(col, .55), k == mine ? 1.5 : .8);
  }
  for (var k = 0; k < n; k++) {
    ln(c, Offset(10 + k * 5.0, 104 - k * 4.0), Offset(106 + k * 5.0, 104 - k * 4.0), al(K.dim, .7), .5);
  }
  final u = trip(t);
  final off = Offset(mine * 5.0, -mine * 4.0);
  ci(c, Offset(10 + 96 * u, 104 - 56 * ezv(u, v)) + off, 3, _w);
  lb(c, 'MAX', const Offset(8, 8), K.grey, h: 5);
  lb(c, 'MIN', const Offset(8, 110), K.grey, h: 5);
  lb(c, 'OVER', const Offset(8, 22), pc(v, 2));
  nm(c, d2(v[2]), const Offset(8, 32), 24, pc(v, 2));
  lb(c, 'IN ${d2(v[0])}  OUT ${d2(v[1])}', const Offset(150, 110), _w, ax: 1, h: 5);
}

// 18 Hare and snail: the snail is linear, the hare is your ease; who reaches the flag how.
void _hare(Canvas c, Size s, OpV v, double t) {
  final u = trip(t, 3.2);
  ln(c, const Offset(8, 56), const Offset(150, 56), K.dim);
  ln(c, const Offset(8, 104), const Offset(150, 104), K.dim);
  ln(c, const Offset(132, 20), const Offset(132, 106), _dimW, .8);
  for (var k = 0; k < 6; k++) {
    rc(c, Rect.fromLTWH(133, 20 + k * 4.0, 4, 4), k.isEven ? _w : K.dim, .6);
  }
  // snail
  final sx = 20 + 100 * u;
  ci(c, Offset(sx, 49), 6, _dimW);
  fn(c, 20, (q) => Offset(sx, 49) + Offset(math.cos(q * 9), math.sin(q * 9)) * (6 * (1 - q)), _dimW, .7);
  pl(c, [Offset(sx - 7, 55), Offset(sx + 10, 55), Offset(sx + 12, 50)], _dimW, .9);
  ln(c, Offset(sx + 11, 50), Offset(sx + 13, 43), _dimW, .7);
  // hare
  final e = ezv(u, v);
  final hx = 20 + 112 * e * (100 / 112);
  final hop = (math.sin(u * 40).abs()) * 6 * (ezv(u + .02, v) - e).abs() * 20;
  final b = Offset(hx, 94 - hop);
  c.drawOval(Rect.fromCenter(center: b, width: 22, height: 12), st(e > 1.0 ? pc(v, 2) : K.white, 1.1));
  final hd = b + const Offset(12, -7);
  ci(c, hd, 4, _w);
  ln(c, hd + const Offset(-1, -3), hd + const Offset(-6, -16), _w, .9);
  ln(c, hd + const Offset(1, -3), hd + const Offset(0, -16), _w, .9);
  dot(c, hd + const Offset(2, -1), .8, _w);
  ci(c, b + const Offset(-12, -2), 2, _w, .8);
  ln(c, b + const Offset(6, 5), b + const Offset(10, 10), _w, .9);
  ln(c, b + const Offset(-6, 5), b + const Offset(-12, 10), _w, .9);
  lb(c, 'LINEAR', const Offset(8, 10), K.grey, h: 5);
  lb(c, 'IN ${d2(v[0])}', const Offset(8, 110), pc(v, 0), h: 5);
  lb(c, 'OUT ${d2(v[1])}', const Offset(46, 110), pc(v, 1), h: 5);
  lb(c, 'OVER ${d2(v[2])}', const Offset(86, 110), pc(v, 2), h: 5);
}

// 19 Typewriter: flick the return lever; the carriage eases home and bounces on the bell.
void _type(Canvas c, Size s, OpV v, double t) {
  final e = ezv(trip(t, 2.6), v);
  final cx = 104 - 70 * e;
  ln(c, const Offset(6, 84), const Offset(150, 84), _dimW);
  pl(c, const [Offset(14, 84), Offset(24, 104), Offset(132, 104), Offset(142, 84)], _w);
  for (var k = 0; k < 8; k++) {
    ci(c, Offset(36 + k * 11.0, 94), 3, K.dim, .8);
  }
  final r = RRect.fromRectAndRadius(Rect.fromLTWH(cx - 38, 52, 76, 14), const Radius.circular(7));
  c.drawRRect(r, st(_w, 1.2));
  ci(c, Offset(cx - 42, 59), 4, _w);
  ci(c, Offset(cx + 42, 59), 4, _w);
  pl(c, [Offset(cx - 46, 52), Offset(cx - 56, 40), Offset(cx - 62, 40)], pc(v, 1), 1.1);
  rc(c, Rect.fromLTWH(cx - 30, 22, 60, 30), _dimW, .8);
  for (var k = 0; k < 3; k++) {
    final len = k < 2 ? 50.0 : 50 * (1 - cl(e));
    ln(c, Offset(cx - 26, 28 + k * 6.0), Offset(cx - 26 + len, 28 + k * 6.0), al(K.white, .5), .7);
  }
  const bell = Offset(140, 40);
  final ring = e > .99 ? 1.0 : 0.0;
  c.drawArc(Rect.fromCircle(center: bell, radius: 6), math.pi, math.pi, false, st(ring > 0 ? pc(v, 2) : K.white, 1));
  ln(c, bell + const Offset(-6, 0), bell + const Offset(6, 0), _w, .9);
  if (ring > 0) {
    for (var k = 0; k < 3; k++) {
      ln(c, pol(bell, 9, -2.4 + k * .8), pol(bell, 13, -2.4 + k * .8), pc(v, 2), .9);
    }
    lb(c, 'DING', bell + const Offset(-6, -20), pc(v, 2), h: 5);
  }
  lb(c, 'IN ${d2(v[0])}', const Offset(6, 8), pc(v, 0), h: 5);
  lb(c, 'OUT ${d2(v[1])}', const Offset(6, 16), pc(v, 1), h: 5);
  lb(c, 'OVER ${d2(v[2])}', const Offset(6, 112), pc(v, 2), h: 5);
}

// 20 Off the chart: overshoot pushed to the extreme, the curve leaves the panel and the number takes over.
void _offChart(Canvas c, Size s, OpV v, double t) {
  const om = 6.0;
  final pct = (v[2] * om * 100 * .75).round();
  nm(c, '$pct%', const Offset(150, 62), 46, pc(v, 2, .9), ax: 1);
  Offset at(double x, double y) => Offset(8 + 140 * x, 110 - 70 * y);
  dl(c, at(0, 1), at(1, 1), _dimW);
  fn(c, 60, (x) => at(x, ez(x, v[0], v[1], v[2], om)), _w, 1.3);
  var pk = 0.0, px = 0.0;
  for (var i = 0; i <= 60; i++) {
    final y = ez(i / 60, v[0], v[1], v[2], om);
    if (y > pk) {
      pk = y;
      px = i / 60;
    }
  }
  final top = at(px, pk);
  if (top.dy < 2) {
    final x = top.dx;
    pl(c, [Offset(x - 9, 0), Offset(x - 4, 5), Offset(x - 1, 2), Offset(x + 3, 8), Offset(x + 6, 3), Offset(x + 11, 0)], pc(v, 2), 1);
  }
  final u = trip(t, 2.2);
  ci(c, at(u, ez(u, v[0], v[1], v[2], om)), 3, _w);
  lb(c, 'IN ${d2(v[0])}  OUT ${d2(v[1])}', const Offset(8, 8), _w, h: 5);
  lb(c, 'RUB HARDER', const Offset(8, 16), K.grey, h: 5);
}

final easePanels = <OpPanel>[
  OpPanel('dashboard to the stop line', 'vehicle · effect · sensible', _road),
  OpPanel('OP-1 envelope', 'diagram · mechanism · sensible', _env),
  OpPanel('ski slope with kicker', 'landscape · effect · drawing', _ski),
  OpPanel('onion-skin frames', 'diagram · sample box · drawing', _onion, g: const OpRub(2, 0)),
  OpPanel('lift between floors', 'isometric · effect · numeral', _lift),
  OpPanel('boxer: wind, snap, reach', 'character · effect · drawing', _boxer, g: const OpFlick(2, 0)),
  OpPanel('monkey drummer', 'character · effect · playful', _monkey, g: const OpDrag(1, 0)),
  OpPanel('letters land', 'typographic · effect · type', _letters, g: const OpPinch(0, 1)),
  OpPanel('rocket into orbit', 'vehicle · effect · numeral', _rocket),
  OpPanel('sunrise arc', 'landscape · effect · drawing', _sun, g: const OpSpin(0, Offset(.5, .73), 1)),
  OpPanel('crate on a conveyor', 'isometric · effect · diagram', _crate),
  OpPanel('Marey timetable', 'diagram · mechanism · draw it', _marey, g: const OpDraw(_fitEase)),
  OpPanel('cat pounce', 'animal · effect · playful', _cat, g: const OpPull(2, 0)),
  OpPanel('dot-matrix lane', 'Max device · mechanism', _matrix, g: const OpDrag(0, 2)),
  OpPanel('Kepler spokes', 'cosmic · time spokes', _kepler, g: const OpSpin(0, Offset(.5, .5), 1)),
  OpPanel('door slam', 'isometric · effect · drawing', _door, g: const OpFlick(2, 1)),
  OpPanel('family in depth', 'diagram · all at once · pushed', _family, g: const OpPinch(2, 0)),
  OpPanel('hare vs snail', 'animal · vs linear · drawing', _hare),
  OpPanel('typewriter return', 'machine · effect · drawing', _type, g: const OpFlick(1, 0)),
  OpPanel('off the chart', 'type · 振り切れた', _offChart, g: const OpRub(2, 0), init: const [.3, .6, .4]),
];
