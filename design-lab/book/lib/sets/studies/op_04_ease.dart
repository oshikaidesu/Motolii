// OP 04 Easing: 20 OP-1 style panels. Values: 0 IN (blue, slow start), 1 OUT (green, slow arrival), 2 OVER (red, flies past and lands).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'op_04_kit.dart';

const _w = K.white;
final _dimW = al(K.white, .35);

// 1 Dashboard: you ease a move so the ride feels like braking into a stop line.
void _road(Canvas c, Size s, OpV v, double t) {
  const hy = 46.0;
  double y(double z) => hy + 74 * z * z;
  double hw(double z) => 3 + 72 * z * z;
  final u = trip(t, 3), d = ezv(u, v);
  // skyline
  final sk = <Offset>[const Offset(46, hy)];
  var x = 46.0;
  for (final h in [6.0, 12, 4, 16, 9, 20, 7, 11, 5]) {
    sk..add(Offset(x, hy - h))..add(Offset(x + 7, hy - h));
    x += 7;
    sk.add(Offset(x, hy));
  }
  pl(c, sk, _dimW, .8);
  ln(c, const Offset(0, hy), Offset(s.width, hy), K.dim);
  c.save();
  c.clipRect(const Rect.fromLTWH(0, 0, 156, 96));
  ln(c, Offset(78 - hw(0), hy), Offset(78 - hw(1), 120), _w);
  ln(c, Offset(78 + hw(0), hy), Offset(78 + hw(1), 120), _w);
  for (var k = 0; k < 7; k++) {
    final z0 = ((k / 7 + d * 1.6) % 1), z1 = z0 + .05;
    if (z1 > 1) continue;
    ln(c, Offset(78, y(z0)), Offset(78, y(z1)), al(K.white, .4 + .6 * z0));
  }
  final zl = cl(.15 + .8 * d, 0, 1.2);
  if (zl < 1.05) ln(c, Offset(78 - hw(zl) * .9, y(zl)), Offset(78 + hw(zl) * .9, y(zl)), d > 1.0 ? pc(v, 2) : _w, 1.6);
  c.restore();
  // dashboard
  pl(c, const [Offset(0, 100), Offset(40, 96), Offset(116, 96), Offset(156, 100)], _w);
  final spd = ((ezv(u + .01, v) - d) / .01 * 30).abs();
  nm(c, spd.clamp(0.0, 99.0).round().toString().padLeft(2, '0'), const Offset(78, 101), 13, _w, ax: .5);
  rd(c, v, 0, 'IN', const Offset(6, 6), h: 18);
  rd(c, v, 1, 'OUT', const Offset(150, 6), h: 18, ax: 1);
  lb(c, 'OVER ${d2(v[2])}', const Offset(150, 108), pc(v, 2), ax: 1, h: 5);
}

// 2 Envelope: the curve itself, OP-1 envelope style, handles as coloured dots.
void _env(Canvas c, Size s, OpV v, double t) {
  const x0 = 14.0, x1 = 142.0, y0 = 104.0, y1 = 44.0;
  Offset at(double x, double y) => Offset(lr(x0, x1, x), lr(y0, y1, y));
  ln(c, const Offset(x0, y0), const Offset(x1, y0), K.dim);
  dl(c, Offset(x0, y1), Offset(x1, y1), K.dim);
  final p1 = at(.02 + .9 * v[0], 0), p2 = at(.98 - .9 * v[1], 1 + 1.1 * v[2]);
  dl(c, at(0, 0), p1, pc(v, 0, .6));
  dl(c, at(1, 1), p2, pc(v, 1, .6));
  dl(c, Offset(p1.dx, y0), p1, K.dim);
  dl(c, Offset(p2.dx, y0), p2, K.dim);
  fn(c, 48, (x) => at(x, ezv(x, v)), _w, 1.2);
  dot(c, p1, 2.6, pc(v, 0));
  dot(c, p2, 2.6, pc(v, 1));
  var pk = 0.0, px = 1.0;
  for (var i = 0; i <= 40; i++) {
    final y = ezv(i / 40, v);
    if (y > pk) {
      pk = y;
      px = i / 40;
    }
  }
  if (pk > 1.01) {
    dl(c, at(px, 1), at(px, pk), pc(v, 2));
    dot(c, at(px, pk), 2.2, pc(v, 2));
  }
  dot(c, at(0, 0), 2, _w);
  dot(c, at(1, 1), 2, _w);
  final u = trip(t);
  ci(c, at(u, ezv(u, v)), 3.4, _w);
  rd(c, v, 0, 'IN', const Offset(8, 6), h: 14);
  rd(c, v, 1, 'OUT', const Offset(40, 6), h: 14);
  rd(c, v, 2, 'OVER', const Offset(74, 6), h: 14);
}

// 3 Ski jump: the slope is the curve, and OVER is the kicker lip at the bottom.
void _ski(Canvas c, Size s, OpV v, double t) {
  Offset hill(double x) => Offset(8 + 140 * x, 26 + 70 * ezv(x, v));
  fn(c, 40, hill, _w, 1.2);
  for (var k = 0; k < 5; k++) {
    final b = Offset(18 + k * 9.0, 30 + (k % 2) * 4);
    pl(c, [b + const Offset(-4, 0), b + const Offset(0, -10), b + const Offset(4, 0)], al(K.green, .5), .8, true);
  }
  ln(c, const Offset(140, 96), const Offset(140, 74), _w);
  pl(c, const [Offset(140, 74), Offset(150, 78), Offset(140, 82)], pc(v, 2), 1);
  final u = trip(t, 2.4);
  final p = hill(u), q = hill(math.min(1, u + .02));
  final dir = (q - p).direction, nrm = dir - math.pi / 2;
  ln(c, pol(p, 7, dir + math.pi), pol(p, 7, dir), K.blue, 1.4);
  final hip = pol(p, 7, nrm), head = pol(hip, 6, nrm - .5);
  ln(c, p, hip, _w);
  ln(c, hip, head, _w);
  ci(c, pol(head, 2.4, nrm - .5), 2.4, _w);
  ln(c, hip + const Offset(0, -3), hip + const Offset(-8, 2), _w, .8);
  for (var k = 0; k < 14; k++) {
    final sx = (k * 37.0 + t * 6) % 156, sy = (k * 23.0 + t * (10 + k)) % 120;
    dot(c, Offset(sx, sy), .7, _dimW);
  }
  lb(c, 'IN ${d2(v[0])}', const Offset(100, 8), pc(v, 0));
  lb(c, 'OUT ${d2(v[1])}', const Offset(100, 18), pc(v, 1));
  lb(c, 'KICK ${d2(v[2])}', const Offset(100, 28), pc(v, 2));
}

// 4 Onion skin: twelve ghost frames of one box; the spacing is the ease. Lines tie each time tick to its position.
void _onion(Canvas c, Size s, OpV v, double t) {
  const n = 12;
  for (var k = 0; k < n; k++) {
    final x = 16 + 112 * ezv(k / (n - 1), v);
    final tx0 = 16 + 124 * k / (n - 1);
    dl(c, Offset(tx0, 100), Offset(x + 6, 62), al(K.white, .25), 4, .45);
    ln(c, Offset(tx0, 98), Offset(tx0, 102), K.dim);
    rc(c, Rect.fromLTWH(x, 44, 14, 14), al(K.white, .18 + .5 * k / n), .8);
  }
  ln(c, const Offset(16, 100), const Offset(140, 100), K.dim);
  final u = trip(t);
  final x = 16 + 112 * ezv(u, v);
  rc(c, Rect.fromLTWH(x, 44, 14, 14), _w, 1.3);
  dot(c, Offset(16 + 124 * u, 100), 2, _w);
  lb(c, 'TIME', const Offset(16, 106), K.grey, h: 5);
  lb(c, 'SPACE', const Offset(16, 34), K.grey, h: 5);
  lb(c, 'IN', const Offset(70, 8), pc(v, 0));
  nm(c, d2(v[0]), const Offset(70, 16), 12, pc(v, 0));
  lb(c, 'OUT', const Offset(96, 8), pc(v, 1));
  nm(c, d2(v[1]), const Offset(96, 16), 12, pc(v, 1));
  lb(c, 'OVER', const Offset(122, 8), pc(v, 2));
  nm(c, d2(v[2]), const Offset(122, 16), 12, pc(v, 2));
}

// 5 Elevator: a lift that eases between floors; the floor number is the hero.
void _lift(Canvas c, Size s, OpV v, double t) {
  const o = Offset(40, 108);
  const sc = 9.0;
  final u = trip(t, 3), y = ezv(u, v);
  isoBox(c, o, 0, 0, 0, 3, 3, 10, sc, _dimW, .8);
  for (var f = 1; f < 5; f++) {
    final z = f * 2.0;
    pl(c, [iso(o, 0, 0, z, sc), iso(o, 3, 0, z, sc), iso(o, 3, 3, z, sc)], K.dim, .8);
  }
  final z = .2 + y * 6;
  isoBox(c, o, .4, .4, z, 2.2, 2.2, 2.4, sc, y > 1.0 ? pc(v, 2) : _w, 1.2);
  ln(c, iso(o, 1.5, 1.5, z + 2.4, sc), iso(o, 1.5, 1.5, 10, sc), _dimW, .7);
  final floor = 2 + (y * 6).round();
  nm(c, floor.toString().padLeft(2, '0'), const Offset(148, 20), 40, _w, ax: 1);
  lb(c, 'FLOOR', const Offset(148, 10), K.grey, ax: 1, h: 5);
  pl(c, [const Offset(96, 74), const Offset(100, 70), const Offset(104, 74)], _w, .9);
  lb(c, 'IN ${d2(v[0])}', const Offset(96, 82), pc(v, 0));
  lb(c, 'OUT ${d2(v[1])}', const Offset(96, 92), pc(v, 1));
  lb(c, 'OVER ${d2(v[2])}', const Offset(96, 102), pc(v, 2));
}

// 6 Boxer: wind-up (IN), snap (OUT) and how far the glove reaches past the pad (OVER).
void _boxer(Canvas c, Size s, OpV v, double t) {
  final u = trip(t, 1.6), e = ezv(u, v);
  const head = Offset(44, 40);
  ci(c, head, 9, _w);
  ln(c, const Offset(40, 38), const Offset(48, 38), _w, .8);
  dot(c, const Offset(48, 41), 1, _w);
  pl(c, const [Offset(44, 49), Offset(40, 76), Offset(30, 104)], _w);
  pl(c, const [Offset(40, 76), Offset(50, 104)], _w);
  pl(c, const [Offset(42, 56), Offset(54, 60), Offset(58, 50)], _w);
  ci(c, const Offset(59, 47), 4, _w);
  final sh = const Offset(44, 56), fist = Offset(56 + 66 * e - 8 * (1 - e) * v[0], 54);
  final elbow = Offset(lr(sh.dx, fist.dx, .5), 62 - 8 * (1 - e));
  pl(c, [sh, elbow, fist], _w);
  ci(c, fist, 6, e > 1.0 ? pc(v, 2) : _w, 1.3);
  ln(c, fist + const Offset(-2, -3), fist + const Offset(2, -3), _w, .8);
  ln(c, const Offset(134, 10), const Offset(134, 38), _dimW);
  rc(c, const Rect.fromLTWH(126, 38, 16, 30), _w);
  if (e > .98) {
    for (var k = 0; k < 5; k++) {
      final a = -1.2 + k * .6;
      ln(c, pol(const Offset(134, 54), 14, a + math.pi), pol(const Offset(134, 54), 20, a + math.pi), pc(v, 2));
    }
  }
  lb(c, 'WIND ${d2(v[0])}', const Offset(74, 84), pc(v, 0));
  lb(c, 'SNAP ${d2(v[1])}', const Offset(74, 94), pc(v, 1));
  lb(c, 'REACH ${d2(v[2])}', const Offset(74, 104), pc(v, 2));
}

// 7 Monkey drummer: the stick falls with the ease and buries into the skin with OVER.
void _monkey(Canvas c, Size s, OpV v, double t) {
  const hd = Offset(54, 30);
  ci(c, hd, 11, _w);
  ci(c, hd + const Offset(-12, 0), 4, _w);
  ci(c, hd + const Offset(12, 0), 4, _w);
  c.drawOval(Rect.fromCenter(center: hd + const Offset(0, 4), width: 12, height: 8), st(_w, .9));
  dot(c, hd + const Offset(-4, -3), 1.1, _w);
  dot(c, hd + const Offset(4, -3), 1.1, _w);
  pl(c, [hd + const Offset(-4, 13), const Offset(44, 62)], _w);
  pl(c, [hd + const Offset(4, 13), const Offset(64, 62)], _w);
  final u = trip(t, 1.2), e = ezv(u, v);
  const hand = Offset(78, 50);
  pl(c, [const Offset(62, 46), hand], _w);
  final ang = lr(-1.9, .55, e);
  final tip = pol(hand, 34, ang);
  ln(c, hand, tip, K.blue, 1.4);
  dot(c, tip, 1.6, K.blue);
  final dent = math.max(0.0, e - 1) * 14;
  const dc = Offset(108, 82);
  c.drawOval(Rect.fromCenter(center: dc, width: 50, height: 14 - dent), st(e > 1 ? pc(v, 2) : _w, 1.2));
  ln(c, dc + const Offset(-25, 0), dc + const Offset(-25, 24), _w);
  ln(c, dc + const Offset(25, 0), dc + const Offset(25, 24), _w);
  c.drawArc(Rect.fromCenter(center: dc + const Offset(0, 24), width: 50, height: 14), 0, math.pi, false, st(_w));
  for (var k = 0; k < 6; k++) {
    ln(c, dc + Offset(-25 + k * 10, 6), dc + Offset(-20 + k * 10, 30), K.dim, .7);
  }
  if (e > .95) {
    for (var k = 0; k < 4; k++) {
      final a = -2.4 + k * .5;
      ln(c, pol(dc, 30, a), pol(dc, 36 + dent, a), pc(v, 2));
    }
  }
  lb(c, 'IN ${d2(v[0])}', const Offset(6, 92), pc(v, 0));
  lb(c, 'OUT ${d2(v[1])}', const Offset(6, 102), pc(v, 1));
  lb(c, 'OVER ${d2(v[2])}', const Offset(6, 112), pc(v, 2), h: 5);
}

// 8 Letter drop: the word lands letter by letter; type is the drawing.
void _letters(Canvas c, Size s, OpV v, double t) {
  const word = 'EASE';
  ln(c, const Offset(10, 86), const Offset(146, 86), K.dim);
  for (var k = 0; k < word.length; k++) {
    final e = ezv(trip(t, 2.8, -k * .14), v);
    final y = lr(-40, 50, e);
    tx(c, word[k], Offset(20 + k * 32.0, y), 36, k == 3 && e > 1 ? pc(v, 2) : _w, w: 1.2);
    dl(c, Offset(29 + k * 32.0, 4), Offset(29 + k * 32.0, math.max(4, y - 2)), al(K.white, .18), 4, .4);
  }
  rd(c, v, 0, 'IN', const Offset(10, 92), h: 10);
  rd(c, v, 1, 'OUT', const Offset(64, 92), h: 10);
  rd(c, v, 2, 'OVER', const Offset(146, 92), h: 10, ax: 1);
}

// 9 Rocket: lift-off (IN), coast into orbit (OUT), overshoot the orbit line (OVER).
void _rocket(Canvas c, Size s, OpV v, double t) {
  for (var k = 0; k < 18; k++) {
    dot(c, Offset((k * 53.0) % 150 + 3, (k * 29.0) % 60 + 4), .6, _dimW);
  }
  final orbit = Path()..addArc(const Rect.fromLTWH(-60, 18, 276, 140), math.pi * 1.1, math.pi * .8);
  c.drawPath(orbit, st(K.dim));
  dl(c, const Offset(20, 30), const Offset(100, 30), pc(v, 1, .6));
  ln(c, const Offset(8, 110), const Offset(148, 110), _w);
  pl(c, const [Offset(46, 110), Offset(50, 96), Offset(56, 96), Offset(60, 110)], _dimW, .8);
  final u = trip(t, 3), e = ezv(u, v);
  final y = 96 - 66 * e;
  const x = 60.0;
  final body = [Offset(x - 5, y), Offset(x - 5, y - 18), Offset(x, y - 26), Offset(x + 5, y - 18), Offset(x + 5, y)];
  pl(c, body, e > 1 ? pc(v, 2) : _w, 1.2, true);
  ci(c, Offset(x, y - 14), 2, _w, .8);
  pl(c, [Offset(x - 5, y - 4), Offset(x - 9, y + 2), Offset(x - 5, y)], _w, .9);
  pl(c, [Offset(x + 5, y - 4), Offset(x + 9, y + 2), Offset(x + 5, y)], _w, .9);
  final spd = (ezv(u + .02, v) - e) / .02;
  for (var k = 0; k < (spd.abs() * 5).clamp(0, 10).round(); k++) {
    dot(c, Offset(x + math.sin(k * 2.1 + t * 20) * 2.5, y + 4 + k * 3.2), 1 - k * .06, pc(v, 0, 1 - k * .08));
  }
  nm(c, (e * 100).clamp(0, 199).round().toString().padLeft(2, '0'), const Offset(148, 40), 26, _w, ax: 1);
  lb(c, 'ALT', const Offset(148, 30), K.grey, ax: 1, h: 5);
  lb(c, 'IN ${d2(v[0])}', const Offset(96, 76), pc(v, 0));
  lb(c, 'OUT ${d2(v[1])}', const Offset(96, 86), pc(v, 1));
  lb(c, 'OVER ${d2(v[2])}', const Offset(96, 96), pc(v, 2));
}

// 10 Sunrise: the sun eases along its arc; spin the sky.
void _sun(Canvas c, Size s, OpV v, double t) {
  const o = Offset(78, 88);
  dl(c, const Offset(8, 88), const Offset(148, 88), K.dim);
  c.drawArc(Rect.fromCircle(center: o, radius: 62), math.pi, math.pi, false, st(K.dim, .7));
  pl(c, const [Offset(0, 96), Offset(30, 84), Offset(52, 92), Offset(84, 80), Offset(118, 94), Offset(156, 86)], _w);
  for (var k = 0; k < 4; k++) {
    ln(c, Offset(10.0 + k * 34, 104 + k % 2 * 6), Offset(30.0 + k * 34, 104 + k % 2 * 6), K.dim, .8);
  }
  final e = ezv(trip(t, 3.4), v);
  final a = math.pi + math.pi * e;
  final sun = pol(o, 62, a);
  final col = e > 1 ? pc(v, 2) : K.white;
  ci(c, sun, 9, col, 1.2);
  for (var k = 0; k < 10; k++) {
    final r = k * math.pi / 5 + t * .4;
    ln(c, pol(sun, 12, r), pol(sun, 16, r), col, .9);
  }
  for (var k = 0; k <= 8; k++) {
    final p = pol(o, 62, math.pi + math.pi * ezv(k / 8, v));
    ln(c, pol(p, 3, math.pi + math.pi * ezv(k / 8, v)), pol(p, -3, math.pi + math.pi * ezv(k / 8, v)), _dimW, .8);
  }
  rd(c, v, 0, 'DAWN', const Offset(6, 6), h: 14);
  rd(c, v, 1, 'DUSK', const Offset(150, 6), h: 14, ax: 1);
  lb(c, 'OVER ${d2(v[2])}', const Offset(78, 112), pc(v, 2), ax: .5, h: 5);
}

// 11 Conveyor: an isometric crate slides to its mark; OVER slams the end stop.
void _crate(Canvas c, Size s, OpV v, double t) {
  const o = Offset(20, 66);
  const sc = 11.0;
  isoGrid(c, o, 8, 2, sc, K.dim);
  final e = ezv(trip(t, 2.6), v);
  final x = .2 + 5.6 * e;
  final wall = [iso(o, 8, 0, 0, sc), iso(o, 8, 0, 2.4, sc), iso(o, 8, 2, 2.4, sc), iso(o, 8, 2, 0, sc)];
  pl(c, wall, e > 1.0 ? pc(v, 2) : _w, 1.1, true);
  dl(c, iso(o, 6, 0, 0, sc), iso(o, 6, 2, 0, sc), pc(v, 1));
  for (var k = 1; k < 4; k++) {
    final sp = (ezv(trip(t, 2.6) + .01, v) - e) * 60;
    if (sp.abs() > .2) ln(c, iso(o, x - k * .5, .3 + k * .45, .6, sc), iso(o, x - k * .5 - sp.abs() * .4, .3 + k * .45, .6, sc), _dimW, .7);
  }
  isoBox(c, o, x, .4, 0, 1.6, 1.2, 1.4, sc, _w, 1.2);
  ln(c, iso(o, x, .4, 1.4, sc), iso(o, x + 1.6, 1.6, 1.4, sc), _dimW, .7);
  rd(c, v, 0, 'IN', const Offset(6, 6), h: 14);
  rd(c, v, 1, 'OUT', const Offset(36, 6), h: 14);
  rd(c, v, 2, 'OVER', const Offset(68, 6), h: 14);
}

// 12 Marey timetable: draw how a train should travel; stations up the side, time across.
void _fitEase(OpV v) {
  final tr = v.trail;
  final a = tr.first, b = tr.last;
  if ((b.dx - a.dx).abs() < .05) return;
  double yAt(double fx) {
    var best = tr.first;
    for (final p in tr) {
      if (((p.dx - a.dx) / (b.dx - a.dx) - fx).abs() < ((best.dx - a.dx) / (b.dx - a.dx) - fx).abs()) best = p;
    }
    return (best.dy - a.dy) / (b.dy - a.dy);
  }

  var mx = 0.0;
  for (final p in tr) {
    mx = math.max(mx, (p.dy - a.dy) / (b.dy - a.dy));
  }
  v.set(0, (.25 - yAt(.25)) / .25 * 1.2 + .15);
  v.set(1, (yAt(.75) - .75) / .25 * 1.2 + .15);
  v.set(2, (mx - 1) / .35);
  v.hot = -1;
}

void _marey(Canvas c, Size s, OpV v, double t) {
  const x0 = 20.0, x1 = 148.0, y0 = 104.0, y1 = 16.0;
  const st0 = ['A', 'B', 'C', 'D', 'E'];
  for (var k = 0; k < 5; k++) {
    final y = lr(y0, y1, k / 4);
    dl(c, Offset(x0, y), Offset(x1, y), K.dim, 4);
    lb(c, st0[k], Offset(8, y - 3), K.grey, h: 5);
  }
  for (var k = 0; k < 3; k++) {
    final sh = k * .28 - .2;
    fn(c, 40, (x) => Offset(lr(x0, x1, cl(x * .55 + sh + .2, 0, 1)), lr(y0, y1, ezv(x, v))), al(K.white, .22), .8);
  }
  fn(c, 48, (x) => Offset(lr(x0, x1, x), lr(y0, y1, ezv(x, v))), _w, 1.2);
  final u = trip(t);
  final p = Offset(lr(x0, x1, u), lr(y0, y1, ezv(u, v)));
  ci(c, p, 3, _w);
  dl(c, Offset(p.dx, y0), p, al(K.white, .4));
  if (v.trail.length > 1) {
    pl(c, [for (final q in v.trail) Offset(q.dx * s.width, q.dy * s.height)], pc(v, 2, .8), 1);
  }
  lb(c, 'DRAW A TRIP', const Offset(148, 110), K.grey, ax: 1, h: 5);
  lb(c, '${d2(v[0])} ${d2(v[1])} ${d2(v[2])}', const Offset(22, 110), _w, h: 5);
}

// 13 Cat pounce: pull back to load the pounce; the cat skids past the mouse with OVER.
void _cat(Canvas c, Size s, OpV v, double t) {
  final u = trip(t, 2.4), e = ezv(u, v);
  ln(c, const Offset(6, 100), const Offset(150, 100), K.dim);
  const tgt = Offset(120, 96);
  ci(c, tgt, 3, _w, .9);
  ln(c, tgt + const Offset(3, 0), tgt + const Offset(10, -2), _w, .7);
  final x = 26 + 94 * e;
  final hop = math.sin(cl(e) * math.pi) * 34;
  final b = Offset(x, 90 - hop);
  final crouch = u < .02 ? 1.0 : 0.0;
  // body: an arc back, head, ears, tail, legs
  c.drawArc(Rect.fromCenter(center: b, width: 30, height: 18 - crouch * 6), math.pi, math.pi, false, st(_w, 1.1));
  final hd = b + Offset(17, -6 + crouch * 3);
  ci(c, hd, 5, _w);
  pl(c, [hd + const Offset(-4, -3), hd + const Offset(-3, -9), hd + const Offset(0, -5)], _w, .9);
  pl(c, [hd + const Offset(1, -5), hd + const Offset(4, -10), hd + const Offset(5, -2)], _w, .9);
  dot(c, hd + const Offset(2, -1), .9, _w);
  final tl = b + const Offset(-15, 0);
  fn(c, 10, (q) => tl + Offset(-q * 14, -math.sin(q * 3 + t * 3) * 6 - q * 8), _w, .9);
  final legA = hop > 2 ? 10.0 : 4.0;
  ln(c, b + const Offset(-12, 0), b + Offset(-12 - legA, 10), _w);
  ln(c, b + const Offset(12, 0), b + Offset(12 + legA, 10), _w);
  if (e > 1.0) {
    for (var k = 0; k < 3; k++) {
      ln(c, Offset(x - 20 - k * 6, 100 + k * 2.0), Offset(x - 8 - k * 6, 100 + k * 2.0), pc(v, 2));
    }
  }
  if (v.press && v.trail.isNotEmpty) {
    ln(c, Offset(v.trail.first.dx * s.width, v.trail.first.dy * s.height), v.at, pc(v, 2), .9);
  }
  lb(c, 'CROUCH ${d2(v[0])}', const Offset(6, 8), pc(v, 0));
  lb(c, 'LAND ${d2(v[1])}', const Offset(6, 18), pc(v, 1));
  lb(c, 'SKID', const Offset(150, 8), pc(v, 2), ax: 1);
  nm(c, d2(v[2]), const Offset(150, 18), 20, pc(v, 2), ax: 1);
}

// 14 Dot matrix: a Max-style display, the curve lit on a grid of dots.
void _matrix(Canvas c, Size s, OpV v, double t) {
  const cols = 24, rows = 14;
  const x0 = 12.0, y0 = 12.0, dx = 5.6, dy = 5.6;
  final u = trip(t);
  final cur = (u * (cols - 1)).round();
  for (var i = 0; i < cols; i++) {
    final y = ezv(i / (cols - 1), v);
    final r = ((1 - y / 1.3) * (rows - 1)).round();
    for (var j = 0; j < rows; j++) {
      final p = Offset(x0 + i * dx, y0 + j * dy);
      if (j == r) {
        dot(c, p, 1.5, i == cur ? (y > 1 ? pc(v, 2) : K.white) : (y > 1.0 ? pc(v, 2, .8) : al(K.white, .75)));
      } else {
        dot(c, p, .5, K.dim);
      }
    }
    if (i == cur) ln(c, Offset(x0 + i * dx, y0 - 4), Offset(x0 + i * dx, y0 + rows * dy - 2), al(K.white, .3), .6);
  }
  final ry = y0 + (1 - 1 / 1.3) * (rows - 1) * dy;
  ln(c, Offset(x0 - 6, ry), Offset(x0 - 2, ry), pc(v, 1));
  rc(c, const Rect.fromLTWH(12, 98, 14, 9), pc(v, 0), .9);
  lb(c, d2(v[0]), const Offset(30, 100), pc(v, 0), h: 5);
  rc(c, const Rect.fromLTWH(56, 98, 14, 9), pc(v, 1), .9);
  lb(c, d2(v[1]), const Offset(74, 100), pc(v, 1), h: 5);
  rc(c, const Rect.fromLTWH(100, 98, 14, 9), pc(v, 2), .9);
  lb(c, d2(v[2]), const Offset(118, 100), pc(v, 2), h: 5);
}

// 15 Kepler: equal time, unequal angle. The spokes are equal time steps; spin the orbit.
void _kepler(Canvas c, Size s, OpV v, double t) {
  const o = Offset(78, 62);
  c.drawOval(Rect.fromCenter(center: o, width: 128, height: 76), st(K.dim, .8));
  const sun = Offset(52, 62);
  ci(c, sun, 5, _w);
  for (var k = 0; k < 8; k++) {
    ln(c, pol(sun, 7, k * math.pi / 4 + t), pol(sun, 9.5, k * math.pi / 4 + t), _w, .8);
  }
  Offset at(double e) => Offset(o.dx + 64 * math.cos(math.pi * 2 * e + math.pi), o.dy + 38 * math.sin(math.pi * 2 * e + math.pi));
  for (var k = 0; k < 12; k++) {
    final e = ezv(k / 12, v);
    dl(c, sun, at(e), k.isEven ? pc(v, 0, .5) : pc(v, 1, .5), 4, .5);
    dot(c, at(e), 1.2, _dimW);
  }
  final e = ezv(trip(t, 3.2), v);
  final p = at(e);
  ci(c, p, 4, e > 1 ? pc(v, 2) : K.white, 1.2);
  c.drawOval(Rect.fromCenter(center: p, width: 14, height: 3), st(_w, .7));
  lb(c, 'PERI', const Offset(6, 6), pc(v, 0));
  nm(c, d2(v[0]), const Offset(6, 14), 12, pc(v, 0));
  lb(c, 'APO', const Offset(150, 6), pc(v, 1), ax: 1);
  nm(c, d2(v[1]), const Offset(150, 14), 12, pc(v, 1), ax: 1);
  lb(c, 'OVER ${d2(v[2])}', const Offset(150, 108), pc(v, 2), ax: 1, h: 5);
}

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
