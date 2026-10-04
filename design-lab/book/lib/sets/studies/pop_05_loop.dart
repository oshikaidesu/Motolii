// Pop 05 sheet 2, Loop: a = length (how big a chunk repeats), b = offset (where the chunk starts), tap = mode (once / cycle / ping-pong).
// Colour = function: the repeating chunk wears the mode colour: once yellow, cycle cyan, ping-pong pink.
import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'pop_05_kit.dart';

const _pi2 = math.pi * 2;
Color mc(PopP p) => switch (p.m) { 0 => kYellow, 1 => kCyan, _ => kPink };
double len(PopP p) => .15 + p.a * .8;
double lu(PopP p) {
  final per = .8 + p.a * 2.2, x = p.t / per;
  return switch (p.m) { 0 => math.min(frac(x / 2.2) * 2.2, 1.0), 1 => frac(x), _ => tri(x / 2) };
}

/// Position in the whole journey (0..1, wraps): start at the offset, run through the chunk.
double jpos(PopP p) => frac(p.b + len(p) * lu(p));

List<PopSpec> loopPanels() => [
      PopSpec('Racetrack lap', 'toy · top-down · spin+drag · tap mode', G.spinB, _track, a: .5, b: .1, modes: 3, m: 1),
      PopSpec('Hamster wheel', 'creature · flat · drag', G.hv, _hamster, a: .6, b: .0, modes: 3, m: 1),
      PopSpec('Ouroboros', 'creature · bold · spin', G.spinB, _snake, a: .7, b: .2, modes: 3, m: 1),
      PopSpec('Ping-pong table', 'toy · isometric · drag', G.hv, _pingpong, a: .6, b: .2, modes: 3, m: 2),
      PopSpec('Yo-yo', 'toy · light · drag v', G.v, _yoyo, a: .6, modes: 3, m: 2),
      PopSpec('Ferris wheel', 'landscape · flat · spin', G.spinB, _ferris, a: .4, b: .0, modes: 3, m: 1),
      PopSpec('Neon infinity', 'cosmic · neon wild · spin', G.spinB, _infinity, a: .45, b: .1, modes: 3, m: 1),
      PopSpec('Planet orbit', 'cosmic · flat · spin', G.spinB, _orbit, a: .5, b: .15, modes: 3, m: 1),
      PopSpec('Boomerang', 'physics · bold · throw', G.h, _boomerang, a: .6, modes: 3, m: 2),
      PopSpec('Tape loop pegs', 'machine · crisp · stretch', G.hv, _pegs, a: .5, b: .3, modes: 3, m: 1),
      PopSpec('Candy bracelet', 'food · light · spin · result', G.spinB, _candy, a: .3, b: .0, modes: 3, m: 1),
      PopSpec('Carousel', 'toy · isometric · drag', G.hv, _carousel, a: .5, b: .2, modes: 3, m: 2),
      PopSpec('Swing loop-the-loop', 'physics · flat · push', G.v, _swing, a: .5, modes: 3, m: 2),
      PopSpec('Water cycle', 'weather · light · spin', G.spinB, _water, a: .6, b: .0, modes: 3, m: 1),
      PopSpec('Wallpaper stamp', 'text-art · crisp · paint · result', G.hv, _stamp, a: .4, b: .0, modes: 3, m: 1),
      PopSpec('Spirograph', 'instrument · neon · spin', G.spinB, _spiro, a: .4, b: .0, modes: 3, m: 1),
      PopSpec('Toy train tunnel', 'toy · isometric · spin', G.spinB, _train, a: .5, b: .25, modes: 3, m: 1),
      PopSpec('Washer drum', 'machine · bold · drag', G.h, _washer, a: .5, modes: 3, m: 2),
      PopSpec('Slinky coil', 'material · pseudo3D · stretch', G.hv, _slinky, a: .4, b: .0, modes: 3, m: 1),
      PopSpec('Seasons tree', 'nature · light · drag · result', G.hv, _seasons, a: .5, b: .0, modes: 3, m: 1),
    ];

/// Draws the loop chunk along a closed parametric path as a thick coloured arc.
void chunkPath(Canvas c, Offset Function(double u) f, PopP p, double w) {
  final path = Path();
  const n = 48;
  for (var i = 0; i <= n; i++) {
    final q = f(frac(p.b + len(p) * i / n));
    i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
  }
  c.drawPath(path, st(al(mc(p), .9), w));
  c.drawCircle(f(p.b), w * .8, fl(kWhite));
}

void fullPath(Canvas c, Offset Function(double u) f, Paint pt) {
  final path = Path();
  for (var i = 0; i <= 60; i++) {
    final q = f(i / 60);
    i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
  }
  c.drawPath(path, pt);
}

void _track(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF3E8E3A));
  final ctr = s.center(Offset.zero);
  Offset f(double u) => pol(ctr, 56, u * _pi2 - math.pi / 2, .7);
  fullPath(c, f, st(const Color(0xFF3A3A40), 16));
  chunkPath(c, f, p, 6);
  for (var k = 0; k < 24; k++) {
    c.drawCircle(f(k / 24), 1, fl(al(kWhite, .5)));
  }
  final u = jpos(p), q = f(u), q2 = f(u + .01);
  c.save();
  c.translate(q.dx, q.dy);
  c.rotate((q2 - q).direction);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 16, height: 9), const Radius.circular(3)), fl(kRed));
  c.drawRect(Rect.fromCenter(center: const Offset(2, 0), width: 5, height: 7), fl(kInk));
  c.restore();
}

void _hamster(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final ctr = Offset(s.width / 2, s.height * .5);
  const r = 44.0;
  c.drawLine(ctr, Offset(ctr.dx - 26, s.height - 4), st(kCream, 4));
  c.drawLine(ctr, Offset(ctr.dx + 26, s.height - 4), st(kCream, 4));
  final rot = (p.b + len(p) * lu(p)) * _pi2;
  c.drawCircle(ctr, r, st(const Color(0xFF55555C), 6));
  c.drawArc(Rect.fromCircle(center: ctr, radius: r), rot, len(p) * _pi2, false, st(mc(p), 6));
  for (var k = 0; k < 12; k++) {
    c.drawLine(ctr, pol(ctr, r, rot + k * _pi2 / 12), st(al(kCream, .18), 1));
  }
  c.drawCircle(ctr, 4, fl(kCream));
  final hb = Offset(ctr.dx, ctr.dy + r - 15), step = math.sin(p.t * 14);
  c.drawOval(Rect.fromCenter(center: hb, width: 30, height: 18), fl(kOrange));
  c.drawCircle(hb + const Offset(12, -5), 7, fl(kOrange));
  c.drawCircle(hb + const Offset(14, -7), 1.6, fl(kInk));
  c.drawCircle(hb + const Offset(8, -12), 3, fl(kPink));
  c.drawLine(hb + const Offset(-8, 8), hb + Offset(-8 + step * 4, 13), st(kOrange, 3));
  c.drawLine(hb + const Offset(8, 8), hb + Offset(8 - step * 4, 13), st(kOrange, 3));
}

void _snake(Canvas c, Size s, PopP p) {
  bg(c, s, kLime);
  final ctr = s.center(Offset.zero);
  final head = (p.b + len(p) * lu(p)) * _pi2 - math.pi / 2;
  final body = len(p) * _pi2 * .95;
  const n = 30;
  for (var i = n; i >= 0; i--) {
    final a = head - body * i / n;
    final wob = math.sin(i * .9 + p.t * 5) * 3;
    c.drawCircle(pol(ctr, 40 + wob, a), 9 - i * .15, fl(i.isEven ? kInk : const Color(0xFF2D6A2A)));
  }
  final hp = pol(ctr, 40, head);
  c.drawCircle(hp, 11, fl(kInk));
  c.drawCircle(hp + Offset(math.cos(head + 1.6) * 4, math.sin(head + 1.6) * 4), 3, fl(kYellow));
  c.drawLine(pol(hp, 10, head + math.pi / 2), pol(hp, 17, head + math.pi / 2), st(kRed, 2));
  c.drawCircle(pol(ctr, 40, p.b * _pi2 - math.pi / 2), 4, fl(mc(p)));
  c.drawCircle(pol(ctr, 40, p.b * _pi2 - math.pi / 2), 4, st(kInk, 1.5));
}

void _pingpong(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  const ix = Offset(.9, .44), iy = Offset(-.6, .5);
  const o = Offset(38, 30);
  Offset iso(double x, double y) => o + ix * x + iy * y;
  final table = poly([iso(0, 0), iso(110, 0), iso(110, 60), iso(0, 60)]);
  c.drawPath(table.shift(const Offset(0, 6)), fl(const Color(0xFF0B1A4A)));
  c.drawPath(table, fl(kBlue));
  c.drawLine(iso(0, 30), iso(110, 30), st(kWhite, 1.2));
  c.drawLine(iso(55, -4), iso(55, 64), st(kWhite, 3));
  final x0 = 6 + p.b * 50, x1 = math.min(x0 + len(p) * 100, 106.0);
  c.drawLine(iso(x0, 50), iso(x1, 50), st(mc(p), 4));
  final u = lu(p), x = lerp(x0, x1, u), hop = math.sin(frac(u * 2) * math.pi).abs() * 18;
  c.drawOval(Rect.fromCenter(center: iso(x, 30), width: 8, height: 4), fl(al(Colors.black, .5)));
  c.drawCircle(iso(x, 30) - Offset(0, hop + 4), 4.5, fl(kOrange));
  for (final xx in [x0, x1]) {
    c.drawCircle(iso(xx, 30) - const Offset(0, 14), 7, fl(kRed));
  }
}

void _yoyo(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final hand = Offset(s.width / 2, 12), u = p.m == 2 ? lu(p) : (p.m == 1 ? lu(p) : lu(p));
  final drop = 14 + len(p) * 80 * u;
  final y = hand + Offset(0, drop);
  c.drawCircle(hand + Offset(0, 14 + len(p) * 80), 14, st(mc(p) == kYellow ? kOrange : mc(p), 2.5));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hand - const Offset(0, 4), width: 24, height: 14), const Radius.circular(7)), fl(const Color(0xFFE9B48A)));
  c.drawLine(hand, y, st(kInk, 1.5));
  final spin = p.t * 9;
  c.drawCircle(y, 14, fl(kRed));
  c.drawCircle(y, 14, st(kInk, 2));
  for (var k = 0; k < 3; k++) {
    c.drawLine(y, pol(y, 12, spin + k * _pi2 / 3), st(kWhite, 2));
  }
  c.drawCircle(y, 3, fl(kInk));
}

void _ferris(Canvas c, Size s, PopP p) {
  vgrad(c, Offset.zero & s, kBlue, kViolet);
  final ctr = Offset(s.width / 2, s.height * .46);
  const r = 40.0;
  c.drawLine(ctr, Offset(ctr.dx - 26, s.height), st(kCream, 3));
  c.drawLine(ctr, Offset(ctr.dx + 26, s.height), st(kCream, 3));
  c.drawCircle(ctr, r, st(kCream, 2));
  final rot = (p.b + len(p) * lu(p)) * _pi2;
  const n = 10;
  final lit = (len(p) * n).round();
  for (var k = 0; k < n; k++) {
    final a = rot + k * _pi2 / n, q = pol(ctr, r, a);
    c.drawLine(ctr, q, st(al(kCream, .4), 1));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: q + const Offset(0, 6), width: 12, height: 10), const Radius.circular(3)), fl(k < lit ? mc(p) : kP1));
  }
  c.drawCircle(ctr, 4, fl(kYellow));
}

void _infinity(Canvas c, Size s, PopP p) {
  bg(c, s, Colors.black);
  final ctr = s.center(Offset.zero);
  Offset f(double u) {
    final t = u * _pi2, d = 1 + math.sin(t) * math.sin(t);
    return ctr + Offset(62 * math.cos(t) / d, 62 * math.sin(t) * math.cos(t) / d);
  }

  fullPath(c, f, st(al(kViolet, .35), 10));
  fullPath(c, f, st(kViolet, 1.5));
  chunkPath(c, f, p, 5);
  final q = f(jpos(p));
  c.drawCircle(q, 10, fl(al(kWhite, .25)));
  c.drawCircle(q, 5, fl(kWhite));
}

void _orbit(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF0E1230));
  final ctr = s.center(Offset.zero);
  Offset f(double u) => pol(ctr, 60, u * _pi2, .48);
  fullPath(c, f, st(al(kCream, .25), 1));
  chunkPath(c, f, p, 3);
  final q = f(jpos(p));
  final behind = math.sin(jpos(p) * _pi2) < 0;
  void planet() {
    c.drawCircle(q, 8, fl(kLime));
    c.drawCircle(q + const Offset(-2, -2), 3, fl(al(kWhite, .5)));
  }

  if (behind) planet();
  c.drawCircle(ctr, 15, fl(kYellow));
  c.drawCircle(ctr, 19, st(al(kOrange, .6), 3));
  if (!behind) planet();
  final flag = f(p.b);
  c.drawLine(flag, flag - const Offset(0, 14), st(kWhite, 1.5));
  c.drawPath(poly([flag - const Offset(0, 14), flag - const Offset(-8, 11), flag - const Offset(0, 8)]), fl(mc(p)));
}

void _boomerang(Canvas c, Size s, PopP p) {
  bg(c, s, kOrange);
  final start = Offset(16, s.height - 22), far = 20 + len(p) * 120;
  Offset f(double u) {
    final t = u * math.pi;
    return start + Offset(math.sin(t) * far * .9, -math.sin(t * 2) * 26 - math.sin(t) * 22);
  }

  final path = Path();
  for (var i = 0; i <= 40; i++) {
    final q = f(i / 40 * (p.m == 0 ? .5 : 1));
    i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
  }
  c.drawPath(path, st(al(kInk, .35), 2));
  double u = p.m == 0 ? lu(p) * .5 : (p.m == 1 ? lu(p) : lu(p));
  final q = f(u);
  final y = p.m == 0 && lu(p) >= 1 ? s.height - 12.0 : q.dy;
  c.save();
  c.translate(q.dx, y);
  c.rotate(p.t * 12);
  c.drawPath(poly([const Offset(-12, 6), const Offset(0, -8), const Offset(12, 6), const Offset(8, 8), const Offset(0, -2), const Offset(-8, 8)]), fl(kYellow));
  c.drawPath(poly([const Offset(-12, 6), const Offset(0, -8), const Offset(12, 6), const Offset(8, 8), const Offset(0, -2), const Offset(-8, 8)]), st(kInk, 1.6));
  c.restore();
  c.drawCircle(start, 7, fl(kInk));
  c.drawLine(Offset(start.dx + far * .9, s.height - 6), Offset(start.dx + far * .9, s.height - 16), st(mc(p) == kYellow ? kInk : mc(p), 3));
}

void _pegs(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final cy = s.height / 2, w = 24 + len(p) * 90, x0 = (s.width - w) / 2;
  const r = 22.0;
  final belt = RRect.fromRectAndRadius(Rect.fromLTWH(x0 - r, cy - r, w + 2 * r, 2 * r), const Radius.circular(r));
  c.drawRRect(belt, st(const Color(0xFF6B4A2A), 7));
  final per = 2 * w + 2 * math.pi * r;
  Offset at(double d) {
    d = frac(d / per) * per;
    if (d < w) return Offset(x0 + d, cy - r);
    d -= w;
    if (d < math.pi * r) return pol(Offset(x0 + w, cy), r, -math.pi / 2 + d / r);
    d -= math.pi * r;
    if (d < w) return Offset(x0 + w - d, cy + r);
    d -= w;
    return pol(Offset(x0, cy), r, math.pi / 2 + d / r);
  }

  final run = p.t * 40 * (p.m == 2 ? (tri(p.t * .25) * 2 - 1) : 1);
  for (var k = 0; k < 10; k++) {
    c.drawCircle(at(run + k * per / 10), 1.5, fl(al(kCream, .5)));
  }
  final sp = at(run + p.b * per);
  c.drawCircle(sp, 5, fl(mc(p)));
  for (final x in [x0, x0 + w]) {
    c.drawCircle(Offset(x, cy), 12, fl(kCream));
    c.drawCircle(Offset(x, cy), 4, fl(kInk));
  }
}

void _candy(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final ctr = s.center(Offset.zero);
  const pal = [kPink, kYellow, kCyan, kLime, kViolet, kOrange];
  final period = 2 + (p.a * 4).round();
  const n = 18;
  final rot = p.b * _pi2 + p.t * .3 * (p.m == 2 ? math.sin(p.t * .8).sign : 1) * (p.m == 0 ? 0 : 1);
  c.drawCircle(ctr, 44, st(al(kInk, .3), 1.5));
  for (var k = 0; k < n; k++) {
    final q = pol(ctr, 44, rot + k * _pi2 / n);
    final col = pal[k % period];
    c.drawCircle(q, 7, fl(col));
    c.drawCircle(q, 7, st(kInk, 1.6));
    if (k % period == 0) c.drawCircle(q, 2.4, fl(kWhite));
  }
  final a0 = rot - math.pi / n, a1 = _pi2 / n * period;
  c.drawArc(Rect.fromCircle(center: ctr, radius: 30), a0, a1, false, st(mc(p) == kYellow ? kOrange : mc(p), 3));
}

void _carousel(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final ctr = Offset(s.width / 2, s.height * .62);
  c.drawOval(Rect.fromCenter(center: ctr + const Offset(0, 8), width: 130, height: 38), fl(kRed));
  c.drawOval(Rect.fromCenter(center: ctr, width: 130, height: 38), fl(kCream));
  c.drawPath(poly([Offset(ctr.dx - 64, 28), Offset(ctr.dx, 6), Offset(ctr.dx + 64, 28)]), fl(kRed));
  final rot = p.t * .7 + p.b * _pi2;
  final items = <(double, Offset, int)>[];
  for (var k = 0; k < 6; k++) {
    final a = rot + k * _pi2 / 6;
    items.add((math.sin(a), pol(ctr, 52, a, .3), k));
  }
  items.sort((x, y) => x.$1.compareTo(y.$1));
  for (final (z, q, k) in items) {
    final bob = (p.m == 1 ? frac(p.t / (.6 + len(p) * 1.6) + k / 6) : (p.m == 2 ? tri(p.t / (.6 + len(p) * 1.6) + k / 6) : 0.0)) * len(p) * 22;
    final top = Offset(q.dx, 28 + 2 * z);
    c.drawLine(top, q, st(kYellow, 1.5));
    final hq = q - Offset(0, 12 + bob);
    final sc = .8 + z * .2;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hq, width: 18 * sc, height: 9 * sc), const Radius.circular(4)), fl(k.isEven ? mc(p) : kWhite));
    c.drawCircle(hq + Offset(9 * sc, -5 * sc), 4 * sc, fl(k.isEven ? mc(p) : kWhite));
  }
}

void _swing(Canvas c, Size s, PopP p) {
  bg(c, s, kCyan);
  final pivot = Offset(s.width / 2, 22);
  c.drawLine(Offset(10, 22), Offset(s.width - 10, 22), st(kInk, 4));
  c.drawLine(Offset(20, 22), Offset(6, s.height), st(kInk, 4));
  c.drawLine(Offset(s.width - 20, 22), Offset(s.width - 6, s.height), st(kInk, 4));
  final amp = .3 + len(p) * 1.1;
  final a = switch (p.m) {
    0 => amp * math.exp(-frac(p.t / 6) * 4) * math.sin(p.t * 3),
    1 => p.t * 2.6,
    _ => amp * math.sin(p.t * 3),
  };
  final seat = pivot + Offset(math.sin(a), math.cos(a)) * 60;
  c.drawArc(Rect.fromCircle(center: pivot, radius: 60), math.pi / 2 - (p.m == 1 ? math.pi : amp), p.m == 1 ? _pi2 : amp * 2, false, st(al(mc(p) == kCyan ? kWhite : mc(p), .9), 3));
  c.drawLine(pivot, seat, st(kInk, 2));
  c.drawRect(Rect.fromCenter(center: seat, width: 18, height: 5), fl(kInk));
  final kid = seat - Offset(math.sin(a), math.cos(a)) * 12;
  c.drawCircle(kid, 8, fl(kOrange));
  c.drawCircle(kid - Offset(math.sin(a), math.cos(a)) * 13, 6, fl(kOrange));
}

void _water(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final ctr = s.center(Offset.zero);
  c.drawRect(Rect.fromLTWH(0, s.height - 24, s.width, 24), fl(kBlue));
  for (final (x, r) in [(ctr.dx - 14, 12.0), (ctr.dx + 2, 16.0), (ctr.dx + 18, 11.0)]) {
    c.drawCircle(Offset(x, 22), r, fl(const Color(0xFFB9C6D6)));
  }
  Offset f(double u) => pol(ctr + const Offset(0, 6), 44, u * _pi2 - math.pi / 2, .8);
  chunkPath(c, f, p, 4);
  final q = f(jpos(p));
  final up = math.cos(jpos(p) * _pi2 - math.pi / 2) < 0;
  if (up) {
    c.drawCircle(q, 6, st(kBlue, 2));
  } else {
    final drop = Path()
      ..moveTo(q.dx, q.dy - 9)
      ..quadraticBezierTo(q.dx + 7, q.dy + 2, q.dx, q.dy + 5)
      ..quadraticBezierTo(q.dx - 7, q.dy + 2, q.dx, q.dy - 9);
    c.drawPath(drop, fl(kBlue));
  }
  c.drawCircle(Offset(s.width - 18, 16), 8, fl(kYellow));
}

void _stamp(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final per = 16 + p.a * 40, off = p.b * per;
  final head = 6 + frac(p.t * .18) * (s.width + 10);
  for (var row = 0; row < 3; row++) {
    final y = 26.0 + row * 34;
    for (var x = -per + off; x < s.width + per; x += per) {
      if (row == 1 && x > head) continue;
      final shown = row != 1 || x <= head;
      if (!shown) continue;
      final col = row == 1 ? mc(p) : al(kCream, .3);
      final star = <Offset>[];
      for (var k = 0; k < 10; k++) {
        star.add(pol(Offset(x, y), k.isEven ? 9 : 4, -math.pi / 2 + k * math.pi / 5));
      }
      c.drawPath(poly(star), fl(col));
      c.drawCircle(Offset(x + per / 2, y), 2.5, fl(col));
    }
  }
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(head, 60), width: 14, height: 24), const Radius.circular(3)), fl(kOrange));
  c.drawRect(Rect.fromCenter(center: Offset(head, 46), width: 6, height: 8), fl(kOrange));
}

void _spiro(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF14082A));
  final ctr = s.center(Offset.zero);
  final k = 3 + (p.a * 7).round();
  const R = 54.0;
  final r = R / k, d = r * 1.9;
  Offset f(double t) {
    t += p.b * _pi2;
    return ctr + Offset((R - r) * math.cos(t) + d * math.cos((R - r) / r * t), (R - r) * math.sin(t) - d * math.sin((R - r) / r * t)) * .95;
  }

  final u = p.m == 1 ? lu(p) : (p.m == 0 ? lu(p) : lu(p));
  final path = Path();
  const n = 220;
  final end = (u * n).round().clamp(1, n);
  for (var i = 0; i <= n; i++) {
    final q = f(i / n * _pi2);
    i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
  }
  c.drawPath(path, st(al(kViolet, .45), 1.4));
  final drawn = Path();
  for (var i = 0; i <= end; i++) {
    final q = f(i / n * _pi2);
    i == 0 ? drawn.moveTo(q.dx, q.dy) : drawn.lineTo(q.dx, q.dy);
  }
  c.drawPath(drawn, st(mc(p), 2.6));
  c.drawCircle(f(end / n * _pi2), 4, fl(kWhite));
}

void _train(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF3E8E3A));
  final ctr = s.center(const Offset(0, 4));
  Offset f(double u) => pol(ctr, 60, u * _pi2, .5);
  fullPath(c, f, st(const Color(0xFF8A5A3A), 10));
  fullPath(c, f, st(const Color(0xFFCCCCCC), 1.5));
  chunkPath(c, f, p, 3);
  final head = jpos(p);
  for (var k = 3; k >= 0; k--) {
    final q = f(head - k * .05);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: q - const Offset(0, 5), width: 14, height: 10), const Radius.circular(2)),
        fl(k == 0 ? kRed : (k.isEven ? kYellow : kBlue)));
  }
  final tn = f(p.b + .5);
  c.drawPath(Path()..addArc(Rect.fromCenter(center: tn, width: 30, height: 30), math.pi, math.pi), fl(const Color(0xFF6B6B70)));
  c.drawPath(Path()..addArc(Rect.fromCenter(center: tn, width: 18, height: 20), math.pi, math.pi), fl(kInk));
}

void _washer(Canvas c, Size s, PopP p) {
  bg(c, s, kWhite);
  final ctr = Offset(s.width / 2, s.height * .56);
  c.drawRect(Rect.fromLTWH(10, 6, s.width - 20, 12), fl(const Color(0xFFDDDDDD)));
  c.drawCircle(Offset(s.width - 24, 12), 4, fl(mc(p) == kYellow ? kOrange : mc(p)));
  c.drawCircle(ctr, 44, fl(const Color(0xFFBBBBBB)));
  c.drawCircle(ctr, 38, fl(kBlue));
  final amp = len(p) * math.pi * 1.6;
  final rot = switch (p.m) { 0 => math.min(p.t % 5, 1.5) * 3, 1 => p.t * 3, _ => math.sin(p.t * 2) * amp };
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: 38)));
  for (var k = 0; k < 5; k++) {
    final q = pol(ctr, 22, rot + k * 1.25);
    c.save();
    c.translate(q.dx, q.dy);
    c.rotate(rot * 1.4 + k);
    final col = [kPink, kYellow, kLime, kOrange, kViolet][k];
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-5, -9, 9, 14), const Radius.circular(2)), fl(col));
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-5, 2, 14, 7), const Radius.circular(3)), fl(col));
    c.restore();
  }
  c.restore();
  c.drawCircle(ctr, 38, st(kInk, 3));
  c.drawArc(Rect.fromCircle(center: ctr, radius: 46), -math.pi / 2 - amp / 2, amp, false, st(mc(p) == kYellow ? kOrange : mc(p), 3));
}

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
