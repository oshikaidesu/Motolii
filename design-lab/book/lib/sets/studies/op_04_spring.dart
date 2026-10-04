// OP 04 Spring: 20 OP-1 style panels. Values: 0 STIFF (blue), 1 DAMP (green), 2 MASS (red). Release replays the ring-out.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'op_04_kit.dart';

const _w = K.white;
double _ls(OpV v, [double p = 3.4]) => v.since % p;
final _dimW = al(K.white, .35);

void _coil(Canvas c, Offset a, Offset b, int turns, double r, Color col, [double w = 1]) {
  final d = b - a, n = Offset(-d.dy, d.dx) / d.distance;
  fn(c, turns * 10, (u) {
    final ph = u * turns * 2 * math.pi;
    return a + d * u + d / d.distance * (math.cos(ph) * r * .45) + n * (math.sin(ph) * r);
  }, col, w);
}

void _zig(Canvas c, Offset a, Offset b, int n, double r, Color col, [double w = 1]) {
  final d = b - a, nn = Offset(-d.dy, d.dx) / d.distance;
  pl(c, [a, for (var i = 1; i < n * 2; i++) a + d * (i / (n * 2)) + nn * (i.isOdd ? r : -r), b], col, w);
}

// 1 Coil: the OP-1 spring, TURNS = stiffness, DAMP below, the weight on the end.
void _turns(Canvas c, Size s, OpV v, double t) {
  final x = ring(v, t);
  final turns = 6 + (v[0] * 12).round();
  ln(c, const Offset(14, 44), const Offset(14, 76), _w, 1.2);
  ln(c, const Offset(8, 44), const Offset(20, 44), _w, 1.2);
  final end = Offset(100 + 22 * x, 60);
  _coil(c, const Offset(14, 60), end, turns, 9, pc(v, 0), 1);
  final r = 4 + 7 * v[2];
  ci(c, end + Offset(r, 0), r, pc(v, 2), 1.2);
  dl(c, Offset(146, 30), const Offset(146, 90), pc(v, 1), 3.4, .7);
  dot(c, const Offset(146, 30), 1.8, pc(v, 1));
  dot(c, Offset(146, lr(90, 30, v[1])), 2.4, pc(v, 1));
  lb(c, 'TURNS', const Offset(78, 10), pc(v, 0), ax: .5, h: 7);
  lb(c, 'DAMP', const Offset(78, 100), pc(v, 1), ax: .5, h: 7);
  nm(c, turns.toString().padLeft(2, '0'), const Offset(30, 92), 14, pc(v, 0));
}

// 2 Diving board: the board is the spring, the diver is the mass.
void _board(Canvas c, Size s, OpV v, double t) {
  final x = ring(v, t, 2.6);
  pl(c, const [Offset(0, 60), Offset(30, 60), Offset(30, 120)], _dimW, .9);
  final defl = 16 * x * (.6 + .6 * v[2]);
  fn(c, 20, (u) => Offset(30 + 90 * u, 60 + defl * u * u), _w, 1.6 - v[0] * .4 + .4 * v[0]);
  final tip = Offset(120, 60 + defl);
  final up = math.max(0.0, -x) * 30;
  final f = tip - Offset(0, up);
  final hs = 1 + v[2] * .5;
  ln(c, f, f + Offset(0, -14 * hs), _w);
  ci(c, f + Offset(0, -18 * hs), 3.5 * hs, pc(v, 2));
  pl(c, [f + Offset(-6, -18 * hs), f + Offset(0, -10 * hs), f + Offset(6, -18 * hs)], _w, .9);
  ln(c, f, f + const Offset(-4, 0), _w);
  for (var k = 0; k < 3; k++) {
    fn(c, 30, (u) => Offset(u * 156, 104 + k * 5 + math.sin(u * 20 + t * 2 + k) * 1.2), al(K.blue, .5 - k * .12), .8);
  }
  rd(c, v, 0, 'STIFF', const Offset(6, 6), h: 16);
  rd(c, v, 1, 'DAMP', const Offset(56, 6), h: 16);
  rd(c, v, 2, 'MASS', const Offset(150, 6), h: 16, ax: 1);
}

// 3 Jack in the box: pull to wind; the head boings out of an isometric box.
void _jack(Canvas c, Size s, OpV v, double t) {
  const o = Offset(56, 92);
  const sc = 10.0;
  isoBox(c, o, 0, 0, 0, 3, 3, 3, sc, _w, 1);
  pl(c, [iso(o, 0, 0, 3, sc), iso(o, -1.6, 0, 5.6, sc), iso(o, -1.6, 3, 5.6, sc), iso(o, 0, 3, 3, sc)], _dimW, .9);
  final x = ring(v, t, 2.8);
  final h = 4.2 - 3.4 * x;
  final base = iso(o, 1.5, 1.5, 3, sc), top = iso(o, 1.5, 1.5, 3 + h, sc);
  if ((top - base).distance > 2) _zig(c, base, top, 7, 5, pc(v, 0), 1);
  final hr = 6 + 5 * v[2];
  final hd = top - Offset(0, hr);
  ci(c, hd, hr, _w, 1.2);
  dot(c, hd + Offset(-hr * .35, -hr * .2), 1, _w);
  dot(c, hd + Offset(hr * .35, -hr * .2), 1, _w);
  c.drawArc(Rect.fromCircle(center: hd + Offset(0, hr * .1), radius: hr * .5), .3, math.pi - .6, false, st(_w, .9));
  pl(c, [hd + Offset(-hr * .8, -hr * .6), hd + Offset(0, -hr * 2), hd + Offset(hr * .8, -hr * .6)], pc(v, 2), 1);
  rd(c, v, 0, 'WIND', const Offset(110, 30), h: 14);
  rd(c, v, 1, 'DAMP', const Offset(110, 62), h: 14);
  lb(c, 'HEAD ${d2(v[2])}', const Offset(110, 94), pc(v, 2));
}

// 4 Kangaroo: hops decay with damping, heavier roo hops lower and slower.
void _roo(Canvas c, Size s, OpV v, double t) {
  final x = ring(v, t, 3);
  final hop = x.abs() * 40;
  final gx = 48.0 + ((_ls(v) % 3) * 8);
  ln(c, const Offset(4, 104), const Offset(152, 104), K.dim);
  for (var k = 0; k < 12; k++) {
    final px = 48.0 + k * 3.0;
    dot(c, Offset(px, 104 - spv(k * .1, v).abs() * 40), .6, _dimW);
  }
  final squash = hop < 4 ? 4.0 : 0.0;
  final b = Offset(gx, 84 - hop + squash);
  c.drawOval(Rect.fromCenter(center: b, width: 22, height: 14 - squash / 2), st(_w, 1.1));
  final hd = b + const Offset(12, -14);
  ci(c, hd, 4.5, _w);
  ln(c, b + const Offset(7, -6), hd + const Offset(-2, 4), _w);
  pl(c, [hd + const Offset(-2, -4), hd + const Offset(-4, -11), hd + const Offset(0, -5)], _w, .9);
  dot(c, hd + const Offset(2, -1), .9, _w);
  fn(c, 10, (u) => b + Offset(-10 - u * 18, 2 + u * u * 10), _w, 1);
  pl(c, [b + const Offset(-4, 6), b + Offset(-2, 13 - squash), b + Offset(8, 14 - squash)], _w);
  ln(c, b + const Offset(8, 2), b + const Offset(11, 8), _w, .8);
  rd(c, v, 0, 'SPRING', const Offset(150, 8), h: 16, ax: 1);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(6, 8), pc(v, 1));
  lb(c, 'ROO ${d2(v[2])}', const Offset(6, 18), pc(v, 2));
}

// 5 Car suspension: rub for a rough road; the body floats on its springs.
void _car(Canvas c, Size s, OpV v, double t) {
  final rough = v.energy;
  final x = ring(v, t, 3.4);
  for (var k = 0; k < 2; k++) {
    final wx = 46 + k * 64.0;
    final bump = rough * math.sin(t * 40 + k * 2) * 3;
    final wc = Offset(wx, 92 - bump);
    ci(c, wc, 9, _w, 1.2);
    ci(c, wc, 3, _w, .8);
    for (var j = 0; j < 5; j++) {
      ln(c, pol(wc, 3, j * 1.256 + t * 6), pol(wc, 8, j * 1.256 + t * 6), _dimW, .6);
    }
    final by = 70 + x * 8 * (.5 + v[2]) + rough * math.sin(t * 23 + k) * 2;
    _zig(c, wc + const Offset(0, -9), Offset(wx, by + 2), 4, 3, pc(v, 0), .9);
  }
  final by = 70 + x * 8 * (.5 + v[2]);
  pl(c, [Offset(26, by), Offset(26, by - 12), Offset(54, by - 14), Offset(66, by - 28), Offset(102, by - 28), Offset(116, by - 14), Offset(134, by - 12), Offset(134, by), Offset(26, by)], _w, 1.2);
  pl(c, [Offset(70, by - 25), Offset(84, by - 25), Offset(84, by - 15), Offset(60, by - 15)], _dimW, .8);
  pl(c, [Offset(88, by - 25), Offset(100, by - 25), Offset(110, by - 15), Offset(88, by - 15)], _dimW, .8);
  fn(c, 60, (u) => Offset(u * 156, 101 + (rough > .02 ? math.sin(u * 60 + t * 30) * rough * 2 : 0)), _dimW, .9);
  lb(c, 'STIFF ${d2(v[0])}', const Offset(6, 8), pc(v, 0));
  lb(c, 'DAMP ${d2(v[1])}', const Offset(6, 18), pc(v, 1));
  lb(c, 'LOAD ${d2(v[2])}', const Offset(6, 28), pc(v, 2));
  lb(c, 'RUB = ROAD', const Offset(150, 110), K.grey, ax: 1, h: 5);
}

// 6 Ruler twang: overhang length is stiffness; ghosts show the blur of the ring.
void _ruler(Canvas c, Size s, OpV v, double t) {
  const o = Offset(10, 78);
  const sc = 9.0;
  isoBox(c, o, 0, 0, 0, 6, 4, 2.2, sc, _dimW, .8);
  final len = 92 - 50 * v[0];
  final clamp = iso(o, 5.4, 1.6, 2.2, sc);
  final x = ring(v, t, 2.4);
  final env = v.press ? 1.0 : math.exp(-_ls(v) * (.4 + 4 * v[1]));
  for (var g = -2; g <= 2; g++) {
    final a = g * .14 * env;
    final tip = clamp + Offset(len * math.cos(a), len * math.sin(a) * .9);
    ln(c, clamp, tip, al(K.white, .12), 1);
  }
  final a = x * .35;
  final tip = clamp + Offset(len * math.cos(a), len * math.sin(a) * .9);
  ln(c, clamp, tip, _w, 1.6);
  for (var k = 1; k < 8; k++) {
    final p = ol(clamp, tip, k / 8);
    ln(c, p, p + Offset(0, -2.5 - (k.isEven ? 1.5 : 0)), _dimW, .6);
  }
  rc(c, Rect.fromCenter(center: clamp + const Offset(-6, -4), width: 10, height: 6), _w, .9);
  if (env > .05) {
    for (var k = 0; k < 3; k++) {
      c.drawArc(Rect.fromCircle(center: tip, radius: 6.0 + k * 4), -.5, 1, false, st(pc(v, 2, env), .8));
    }
  }
  rd(c, v, 0, 'SHORT', const Offset(6, 6), h: 16);
  rd(c, v, 1, 'DAMP', const Offset(56, 6), h: 16);
  rd(c, v, 2, 'TIP', const Offset(150, 6), h: 16, ax: 1);
}

// 7 Step response: three large numerals lead, the ring-out underneath with its dotted envelope.
void _resp(Canvas c, Size s, OpV v, double t) {
  rd(c, v, 0, 'STIFF', const Offset(8, 6), h: 22);
  rd(c, v, 1, 'DAMP', const Offset(58, 6), h: 22);
  rd(c, v, 2, 'MASS', const Offset(108, 6), h: 22);
  const y0 = 84.0, amp = 24.0;
  ln(c, const Offset(8, y0), const Offset(148, y0), K.dim);
  fn(c, 80, (u) => Offset(8 + 140 * u, y0 - amp * spv(u * 2.5, v)), _w, 1.1);
  final k = 30 + 370 * v[0], m = .4 + 3.6 * v[2], d = .4 + 26 * v[1] * v[1];
  final zw = d / (2 * m);
  fn(c, 30, (u) => Offset(8 + 140 * u, y0 - amp * math.exp(-zw * u * 2.5)), pc(v, 1, .6), .7);
  fn(c, 30, (u) => Offset(8 + 140 * u, y0 + amp * math.exp(-zw * u * 2.5)), pc(v, 1, .6), .7);
  final wd = math.sqrt(math.max(0.0, k / m - zw * zw));
  if (wd > 0) {
    for (var j = 1; j < 12; j++) {
      final tau = j * math.pi / wd;
      if (tau > 2.5) break;
      final p = Offset(8 + 140 * tau / 2.5, y0 - amp * spv(tau, v));
      dl(c, Offset(p.dx, y0), p, K.dim, 2.6, .5);
    }
  }
  final tau = _ls(v) % 2.5;
  dot(c, Offset(8 + 140 * tau / 2.5, y0 - amp * spv(tau, v)), 2.4, _w);
}

// 8 Bobblehead: drag the head; heavier head = mass.
void _bobble(Canvas c, Size s, OpV v, double t) {
  final x = ring(v, t, 3);
  pl(c, const [Offset(10, 112), Offset(40, 100), Offset(116, 100), Offset(146, 112)], _dimW, .9);
  c.drawArc(Rect.fromCenter(center: const Offset(78, 100), width: 50, height: 30), math.pi, math.pi, false, st(_w, 1.1));
  const neck = Offset(78, 85);
  final ang = -math.pi / 2 + x * .7 + (v.press ? (v.at.dx - 78) / 160 : 0);
  final hr = 12 + 10 * v[2];
  final top = pol(neck, 12, ang);
  _zig(c, neck, top, 5, 3, pc(v, 0), .9);
  final hc = pol(neck, 12 + hr, ang);
  ci(c, hc, hr, _w, 1.2);
  final rt = ang + math.pi / 2;
  dot(c, hc + Offset(math.cos(rt), math.sin(rt)) * hr * .35 + Offset(math.cos(ang), math.sin(ang)) * hr * .15, 1.3, _w);
  dot(c, hc - Offset(math.cos(rt), math.sin(rt)) * hr * .35 + Offset(math.cos(ang), math.sin(ang)) * hr * .15, 1.3, _w);
  c.drawArc(Rect.fromCircle(center: hc, radius: hr * .55), ang + math.pi - .7, 1.4, false, st(_w, .9));
  for (var k = 0; k < 6; k++) {
    ln(c, hc + Offset(math.cos(ang), math.sin(ang)) * hr, hc + Offset(math.cos(ang + (k - 2.5) * .25), math.sin(ang + (k - 2.5) * .25)) * (hr + 5), pc(v, 2, .8), .8);
  }
  lb(c, 'NECK ${d2(v[0])}', const Offset(6, 8), pc(v, 0));
  lb(c, 'DAMP ${d2(v[1])}', const Offset(6, 18), pc(v, 1));
  lb(c, 'HEAD', const Offset(150, 8), pc(v, 2), ax: 1);
  nm(c, d2(v[2]), const Offset(150, 18), 18, pc(v, 2), ax: 1);
}

// 9 Bungee: from a truss bridge into the canyon.
void _bungee(Canvas c, Size s, OpV v, double t) {
  ln(c, const Offset(0, 14), const Offset(156, 14), _w);
  ln(c, const Offset(0, 24), const Offset(156, 24), _w);
  for (var k = 0; k < 13; k++) {
    ln(c, Offset(k * 13.0, 14), Offset(k * 13.0 + 13, 24), _dimW, .7);
    ln(c, Offset(k * 13.0, 24), Offset(k * 13.0 + 13, 14), _dimW, .7);
  }
  pl(c, const [Offset(0, 60), Offset(14, 80), Offset(10, 120)], K.dim);
  pl(c, const [Offset(156, 50), Offset(140, 76), Offset(146, 120)], K.dim);
  final x = ring(v, t, 3.6);
  final rest = 60 + 20 * v[2];
  final y = rest - x * (rest - 26);
  const ax = 70.0;
  final p = Offset(ax, y);
  if (y > 30) _zig(c, const Offset(ax, 24), p, 6 + (v[0] * 6).round(), 2.5 - v[0], pc(v, 0), .9);
  ln(c, p, p + const Offset(0, 14), _w);
  ci(c, p + const Offset(0, 18), 3.5, _w);
  pl(c, [p + const Offset(-6, 12), p + const Offset(0, 6), p + const Offset(6, 12)], _w, .9);
  pl(c, [p + const Offset(-4, -4), p, p + const Offset(4, -4)], _w, .9);
  fn(c, 30, (u) => Offset(u * 156, 112 + math.sin(u * 14 + t) * 1.2), al(K.blue, .5), .8);
  rd(c, v, 1, 'DAMP', const Offset(108, 40), h: 14);
  rd(c, v, 2, 'BODY', const Offset(108, 74), h: 14);
  lb(c, 'CORD ${d2(v[0])}', const Offset(6, 32), pc(v, 0));
}

// 10 Gravity well: a planet bobbing in an isometric rubber sheet. Pinch for mass.
void _well(Canvas c, Size s, OpV v, double t) {
  const o = Offset(78, 40);
  const sc = 8.0;
  final x = ring(v, t, 3.4);
  final depth = (2 + 4 * v[2]) * (1 - x * .8);
  double zf(double a, double b) => -depth * math.exp(-((a - 4) * (a - 4) + (b - 4) * (b - 4)) / 4);
  for (var i = 0; i <= 8; i++) {
    fn(c, 16, (u) => iso(o, i.toDouble(), u * 8, zf(i.toDouble(), u * 8), sc), i == 4 ? al(K.white, .6) : K.dim, .8);
    fn(c, 16, (u) => iso(o, u * 8, i.toDouble(), zf(u * 8, i.toDouble()), sc), i == 4 ? al(K.white, .6) : K.dim, .8);
  }
  final r = 4 + 7 * v[2];
  final pc0 = iso(o, 4, 4, zf(4, 4), sc) - Offset(0, r);
  ci(c, pc0, r, pc(v, 2), 1.2);
  c.drawOval(Rect.fromCenter(center: pc0, width: r * 3.2, height: r * .7), st(_w, .7));
  for (var k = 0; k < 14; k++) {
    dot(c, Offset((k * 41.0) % 156, (k * 17.0) % 30 + 4), .6, _dimW);
  }
  lb(c, 'SHEET ${d2(v[0])}', const Offset(6, 104), pc(v, 0), h: 5);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(6, 112), pc(v, 1), h: 5);
  lb(c, 'MASS', const Offset(150, 6), pc(v, 2), ax: 1);
  nm(c, d2(v[2]), const Offset(150, 14), 16, pc(v, 2), ax: 1);
}

// 11 Doorstop: flick the coil on the wall, boing.
void _doorstop(Canvas c, Size s, OpV v, double t) {
  ln(c, const Offset(22, 0), const Offset(22, 120), _w, 1.2);
  ln(c, const Offset(22, 96), const Offset(156, 96), K.dim);
  rc(c, const Rect.fromLTWH(22, 56, 6, 10), _w, .9);
  final x = ring(v, t, 2.6);
  final ang = x * .9;
  final len = 80 - 20 * v[0];
  const root = Offset(28, 61);
  final tip = root + Offset(len * math.cos(ang), len * math.sin(ang));
  final ctrl = root + Offset(len * .5, 0);
  Offset bez(double u) => root * ((1 - u) * (1 - u)) + ctrl * (2 * u * (1 - u)) + tip * (u * u);
  final turns = 14 + (v[0] * 10).round();
  fn(c, turns * 6, (u) {
    final p = bez(u), q = bez(math.min(1, u + .01));
    final d = q - p;
    final n = d.distance == 0 ? const Offset(0, 1) : Offset(-d.dy, d.dx) / d.distance;
    return p + n * math.sin(u * turns * 2 * math.pi) * 3.2;
  }, pc(v, 0), .8);
  final rr = 5 + 4 * v[2];
  ci(c, tip + Offset(rr * math.cos(ang), rr * math.sin(ang)), rr, pc(v, 2), 1.2);
  final env = v.press ? 0.0 : math.exp(-_ls(v) * (.4 + 3 * v[1]));
  if (env > .1) {
    final s0 = 'BOING';
    for (var k = 0; k < s0.length; k++) {
      lb(c, s0[k], Offset(96 + k * 9.0, 22 + math.sin(t * 18 + k) * 4 * env), al(K.white, env), h: 9);
    }
  }
  lb(c, 'COIL ${d2(v[0])}', const Offset(30, 104), pc(v, 0), h: 5);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(30, 112), pc(v, 1), h: 5);
  lb(c, 'TIP ${d2(v[2])}', const Offset(100, 104), pc(v, 2), h: 5);
}

// 12 SPRING letters: the word stretches up off its baseline and rings, letter after letter.
void _word(Canvas c, Size s, OpV v, double t) {
  const word = 'SPRING';
  ln(c, const Offset(8, 92), const Offset(148, 92), K.dim);
  for (var k = 0; k < word.length; k++) {
    final x = v.press ? 1.0 : spv((_ls(v) % 3.4) - k * .07, v);
    final h = 26 * (1 + .9 * x);
    c.save();
    c.translate(12 + k * 23.0, 92);
    c.scale(1, h / 26);
    tx(c, word[k], const Offset(0, -26), 26, x < -.05 ? pc(v, 2) : K.white, w: 1.2);
    c.restore();
  }
  lb(c, 'STIFF ${d2(v[0])}', const Offset(8, 104), pc(v, 0), h: 5);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(58, 104), pc(v, 1), h: 5);
  lb(c, 'MASS ${d2(v[2])}', const Offset(108, 104), pc(v, 2), h: 5);
}

// 13 Phase spiral: position against velocity, curling to rest. Spin it for damping.
void _phase(Canvas c, Size s, OpV v, double t) {
  const o = Offset(78, 62);
  dl(c, const Offset(14, 62), const Offset(142, 62), K.dim, 4);
  dl(c, const Offset(78, 8), const Offset(78, 116), K.dim, 4);
  lb(c, 'X', const Offset(144, 54), K.grey, h: 5);
  lb(c, 'V', const Offset(82, 8), K.grey, h: 5);
  final k = 30 + 370 * v[0], m = .4 + 3.6 * v[2];
  final w = math.sqrt(k / m);
  Offset at(double tau) {
    final x = spv(tau, v), x2 = spv(tau + .004, v);
    return o + Offset(x * 56, (x2 - x) / .004 / w * 44);
  }

  fn(c, 160, (u) => at(u * 4), _w, 1);
  final tau = _ls(v) % 4;
  final p = at(tau);
  dot(c, p, 2.6, pc(v, 2));
  ci(c, p, 5, pc(v, 2), .8);
  dot(c, o + const Offset(56, 0), 1.8, _w);
  rd(c, v, 1, 'DAMP', const Offset(6, 6), h: 14);
  lb(c, 'STIFF ${d2(v[0])}', const Offset(150, 104), pc(v, 0), ax: 1, h: 5);
  lb(c, 'MASS ${d2(v[2])}', const Offset(150, 112), pc(v, 2), ax: 1, h: 5);
}

// 14 Frog on a lily pad: flick the frog on, the pad sinks and rings in the water.
void _frog(Canvas c, Size s, OpV v, double t) {
  final x = ring(v, t, 3);
  final sink = -x * 6 * (.4 + v[2]);
  const pc0 = Offset(78, 86);
  for (var k = 0; k < 3; k++) {
    final r = ((_ls(v) * 30 + k * 18) % 54) + 20;
    c.drawOval(Rect.fromCenter(center: pc0 + Offset(0, sink), width: r * 2, height: r * .5), st(al(K.blue, (1 - (r - 20) / 54) * .7), .8));
  }
  final pad = Rect.fromCenter(center: pc0 + Offset(0, sink), width: 60, height: 14);
  c.drawArc(pad, .25, math.pi * 2 - .5, false, st(K.green, 1.1));
  ln(c, pad.center, pad.center + const Offset(28, 3), K.green, .8);
  final f = pc0 + Offset(0, sink - 4);
  final fs = 1 + .5 * v[2];
  c.drawArc(Rect.fromCenter(center: f, width: 26 * fs, height: 18 * fs), math.pi, math.pi, false, st(_w, 1.1));
  ci(c, f + Offset(-6 * fs, -9 * fs), 3.2 * fs, _w);
  ci(c, f + Offset(6 * fs, -9 * fs), 3.2 * fs, _w);
  dot(c, f + Offset(-6 * fs, -9 * fs), 1, _w);
  dot(c, f + Offset(6 * fs, -9 * fs), 1, _w);
  c.drawArc(Rect.fromCenter(center: f + Offset(0, -4 * fs), width: 12 * fs, height: 5 * fs), .2, math.pi - .4, false, st(_w, .8));
  pl(c, [f + Offset(-13 * fs, 0), f + Offset(-17 * fs, 3), f + Offset(-11 * fs, 3)], _w, .9);
  pl(c, [f + Offset(13 * fs, 0), f + Offset(17 * fs, 3), f + Offset(11 * fs, 3)], _w, .9);
  rd(c, v, 0, 'PAD', const Offset(6, 6), h: 14);
  rd(c, v, 1, 'DAMP', const Offset(56, 6), h: 14);
  rd(c, v, 2, 'FROG', const Offset(150, 6), h: 14, ax: 1);
}

// 15 Slinky: rings walking down isometric stairs, jiggling after each landing.
void _slinky(Canvas c, Size s, OpV v, double t) {
  const o = Offset(20, 30);
  const sc = 9.0;
  for (var k = 0; k < 4; k++) {
    final z = 6 - k * 2.0;
    pl(c, [iso(o, k * 3.0, 0, z, sc), iso(o, k * 3.0 + 3, 0, z, sc), iso(o, k * 3.0 + 3, 0, z - 2, sc)], _dimW, .9);
    pl(c, [iso(o, k * 3.0, 3, z, sc), iso(o, k * 3.0 + 3, 3, z, sc), iso(o, k * 3.0 + 3, 3, z - 2, sc)], K.dim, .7);
    ln(c, iso(o, k * 3.0, 0, z, sc), iso(o, k * 3.0, 3, z, sc), K.dim, .7);
  }
  const per = 1.8;
  final cyc = (t / per).floor() % 3, ph = (t % per) / per;
  final a = iso(o, cyc * 3.0 + 1.5, 1.5, 6 - cyc * 2.0, sc), b = iso(o, cyc * 3.0 + 4.5, 1.5, 4 - cyc * 2.0, sc);
  final land = math.max(0.0, ph - .55) / .45;
  final jig = ph > .55 ? spv(land * 2, v) : 0.0;
  const n = 14;
  for (var i = 0; i < n; i++) {
    final u = i / (n - 1);
    final s0 = ph < .55 ? u * cl(ph / .55) : lr(u, 1, cl((ph - .55) / .2));
    final arch = math.sin(s0 * math.pi) * (24 + 10 * (1 - v[0]));
    final p = ol(a, b, s0) - Offset(0, arch) + Offset(0, jig * (1 - u) * 5);
    c.drawOval(Rect.fromCenter(center: p, width: 12 + 4 * v[2], height: 4), st(i == n - 1 ? pc(v, 2) : al(K.white, .5 + .5 * u), .9));
  }
  rd(c, v, 0, 'STIFF', const Offset(110, 50), h: 12);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(110, 82), pc(v, 1), h: 5);
  lb(c, 'MASS ${d2(v[2])}', const Offset(110, 90), pc(v, 2), h: 5);
}

// 16 Kitchen scale: the reading itself rings before it settles. Drag sideways to add weight.
void _scale(Canvas c, Size s, OpV v, double t) {
  final x = ring(v, t, 3.2);
  final grams = (v[2] * 900 + 100) * (1 - x);
  nm(c, grams.abs().clamp(0, 1999).round().toString().padLeft(3, '0'), const Offset(150, 10), 34, x < -.02 ? pc(v, 2) : K.white, ax: 1);
  lb(c, 'G', const Offset(150, 48), K.grey, ax: 1);
  final dip = (1 - x) * 6 * (.3 + v[2]);
  pl(c, [Offset(14, 74 + dip), Offset(70, 74 + dip)], _w, 1.2);
  ln(c, Offset(42, 74 + dip), const Offset(42, 90), _dimW);
  pl(c, const [Offset(10, 104), Offset(18, 90), Offset(66, 90), Offset(74, 104)], _w, 1.1, true);
  _zig(c, const Offset(42, 92), Offset(42, 78 + dip), 3, 3, pc(v, 0), .8);
  final count = 1 + (v[2] * 5).round();
  for (var k = 0; k < count; k++) {
    final bx = 22 + (k % 3) * 14.0, by = 74 + dip - 8 - (k ~/ 3) * 9.0;
    rc(c, Rect.fromLTWH(bx, by, 12, 8), pc(v, 2), .9);
  }
  lb(c, 'SPRING ${d2(v[0])}', const Offset(84, 92), pc(v, 0), h: 5);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(84, 100), pc(v, 1), h: 5);
}

// 17 Tuning fork: rub to strike; stiffness is pitch, the decaying tone runs off to the right.
void _fork(Canvas c, Size s, OpV v, double t) {
  final env = v.press ? 1.0 : math.exp(-_ls(v) * (.3 + 3 * v[1]) / (.4 + v[2]));
  final f = 8 + 40 * v[0];
  final wob = math.sin(t * f) * 3 * env;
  pl(c, [Offset(24 - wob, 18), const Offset(24, 64), const Offset(30, 72), const Offset(36, 64), Offset(36 + wob, 18)], _w, 1.4);
  ln(c, const Offset(30, 72), const Offset(30, 108), _w, 1.4);
  for (var k = 1; k < 4; k++) {
    if (env < .05) break;
    c.drawArc(Rect.fromCircle(center: const Offset(30, 40), radius: 12.0 + k * 6), -.6, 1.2, false, st(pc(v, 0, env * (1 - k * .25)), .9));
  }
  final cyc = 3 + 22 * v[0];
  fn(c, 160, (u) {
    final e = math.exp(-u * 3 * (.3 + 3 * v[1]) / (.4 + v[2]));
    return Offset(56 + 94 * u, 62 + math.sin(u * cyc * 2 * math.pi - t * 8) * 18 * e * env);
  }, _w, 1);
  dl(c, const Offset(56, 40), const Offset(150, 40), K.dim, 4);
  dl(c, const Offset(56, 84), const Offset(150, 84), K.dim, 4);
  nm(c, (f * 10).round().toString(), const Offset(150, 92), 18, pc(v, 0), ax: 1);
  lb(c, 'HZ', const Offset(150, 112), K.grey, ax: 1, h: 5);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(56, 104), pc(v, 1), h: 5);
  lb(c, 'MASS ${d2(v[2])}', const Offset(56, 112), pc(v, 2), h: 5);
}

// 18 Seismograph: flick to shake the table; the drum records the ring-out.
void _seismo(Canvas c, Size s, OpV v, double t) {
  const px = 118.0, y0 = 66.0;
  for (var k = 0; k < 6; k++) {
    final x = (k * 26 - (t * 20) % 26);
    ln(c, Offset(x, 30), Offset(x, 102), K.dim, .5);
  }
  ln(c, const Offset(0, 30), const Offset(px + 8, 30), _dimW, .8);
  ln(c, const Offset(0, 102), const Offset(px + 8, 102), _dimW, .8);
  c.drawOval(const Rect.fromLTWH(px + 4, 28, 12, 76), st(_w));
  fn(c, 90, (u) {
    final x = u * px;
    final tau = _ls(v) - (px - x) / 40;
    final y = tau < 0 ? 0.0 : spv(tau, v) * (1 - math.min(1, tau / 30));
    return Offset(x, y0 - y * 30);
  }, _w, 1);
  final yn = y0 - spv(_ls(v), v) * 30;
  ln(c, Offset(px, yn), const Offset(150, 18), pc(v, 0), 1);
  dot(c, Offset(px, yn), 2, pc(v, 2));
  dot(c, const Offset(150, 18), 2, _w);
  lb(c, 'STIFF ${d2(v[0])}', const Offset(6, 8), pc(v, 0));
  lb(c, 'DAMP ${d2(v[1])}', const Offset(6, 18), pc(v, 1));
  lb(c, 'PEN ${d2(v[2])}', const Offset(6, 108), pc(v, 2));
}

// 19 Slingshot: pull the pouch, let go; the band rings, the stone flies.
void _sling(Canvas c, Size s, OpV v, double t) {
  const l = Offset(40, 46), r = Offset(64, 46), base = Offset(52, 70);
  pl(c, [l, const Offset(44, 62), base, const Offset(60, 62), r], _w, 1.4);
  ln(c, base, const Offset(52, 110), _w, 1.4);
  Offset pouch;
  if (v.press) {
    pouch = v.at;
  } else {
    final x = spv(_ls(v), v);
    pouch = const Offset(52, 46) + Offset(-x * 24 * (.4 + v[0]), x * 6);
  }
  pl(c, [l, pouch, r], pc(v, 0), 1);
  c.drawOval(Rect.fromCenter(center: pouch, width: 8, height: 5), st(_w, .9));
  if (!v.press) {
    final ft = _ls(v);
    final sx = 52 + ft * 160 * (.4 + v[0]), sy = 46 - ft * 50 + ft * ft * 60;
    if (sx < 160) dot(c, Offset(sx, sy), 2.4 + v[2] * 1.4, pc(v, 2));
  }
  rd(c, v, 0, 'BAND', const Offset(150, 6), h: 16, ax: 1);
  lb(c, 'DAMP ${d2(v[1])}', const Offset(96, 92), pc(v, 1), h: 5);
  lb(c, 'STONE ${d2(v[2])}', const Offset(96, 100), pc(v, 2), h: 5);
}

// 20 Jelly world: everything in the panel is on springs; rub and a wave runs through it.
void _jelly(Canvas c, Size s, OpV v, double t) {
  const nx = 13, ny = 10;
  final hit = v.at;
  final pts = List<Offset>.generate(nx * ny, (i) {
    final p = Offset(6 + (i % nx) * 12.0, 6 + (i ~/ nx) * 12.0);
    final d = p - hit;
    final dist = d.distance;
    final dir = dist == 0 ? Offset.zero : d / dist;
    if (v.press) return p + dir * 24 * math.exp(-dist / 30);
    final tau = _ls(v) - dist / 90;
    final x = tau < 0 ? 0.0 : spv(tau, v);
    return p + dir * x * 24 * math.exp(-dist / 60);
  });
  for (var j = 0; j < ny; j++) {
    pl(c, [for (var i = 0; i < nx; i++) pts[j * nx + i]], al(K.white, .55), .8);
  }
  for (var i = 0; i < nx; i++) {
    pl(c, [for (var j = 0; j < ny; j++) pts[j * nx + i]], al(K.white, .25), .7);
  }
  ci(c, hit, 4, pc(v, 2));
  final d = (1 - v[1]) * 99;
  nm(c, d.round().toString().padLeft(2, '0'), const Offset(150, 84), 28, pc(v, 1), ax: 1);
  lb(c, 'WOBBLE', const Offset(150, 74), pc(v, 1), ax: 1, h: 5);
}

final springPanels = <OpPanel>[
  OpPanel('OP-1 coil: turns / damp', 'diagram · mechanism · sensible', _turns),
  OpPanel('diving board', 'landscape+figure · effect · drawing', _board, g: const OpFlick(2, 0)),
  OpPanel('jack in the box', 'isometric · effect · playful', _jack, g: const OpPull(0, 1)),
  OpPanel('kangaroo hops', 'animal · effect · drawing', _roo),
  OpPanel('car suspension', 'vehicle · effect · rub the road', _car, g: const OpRub(0, 1)),
  OpPanel('ruler twang on a desk', 'instrument · effect · drawing', _ruler, g: const OpFlick(1, 0)),
  OpPanel('step response', 'diagram · mechanism · numeral', _resp),
  OpPanel('bobblehead', 'character · effect · drawing', _bobble, g: const OpDrag(0, 2)),
  OpPanel('bungee off a truss', 'landscape · effect · drawing', _bungee, g: const OpPull(2, 1)),
  OpPanel('gravity well', 'cosmic+isometric · effect', _well, g: const OpPinch(2, 0)),
  OpPanel('doorstop boing', 'object · effect · playful', _doorstop, g: const OpFlick(1, 0)),
  OpPanel('SPRING stretches', 'typographic · effect · type', _word),
  OpPanel('phase spiral', 'diagram · mechanism · pushed', _phase, g: const OpSpin(1, Offset(.5, .5), 0)),
  OpPanel('frog on a lily pad', 'animal · effect · drawing', _frog, g: const OpFlick(2, 0)),
  OpPanel('slinky on stairs', 'isometric · effect · drawing', _slinky),
  OpPanel('scale reading rings', 'machine · effect · numeral', _scale, g: const OpDrag(2, 0)),
  OpPanel('tuning fork', 'instrument · effect+sound', _fork, g: const OpRub(0, 1)),
  OpPanel('seismograph drum', 'machine · history · drawing', _seismo, g: const OpFlick(1, 0)),
  OpPanel('slingshot', 'object · effect · pull', _sling, g: const OpPull(0, 1)),
  OpPanel('jelly world', 'field · 振り切れた', _jelly, g: const OpRub(1, 0), init: const [.35, .15, .4]),
];
