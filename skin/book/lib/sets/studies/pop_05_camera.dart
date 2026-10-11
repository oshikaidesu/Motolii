// Pop 05 sheet 3, Camera: distance (get closer / further), orbit (circle around the subject), focal length (wide and bendy vs tele and flat).
// Colour = function: distance orange, orbit cyan, focal violet; the subject is always yellow.
import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'pop_05_kit.dart';

const _pi2 = math.pi * 2;

List<PopSpec> cameraPanels() => [
      PopSpec('Dolly rails', 'machine · top-down · drag · distance', G.h, _dolly, a: .5),
      PopSpec('Satellite', 'cosmic · flat · spin · orbit+distance', G.spinB, _satellite, a: .5, b: .1),
      PopSpec('View cone forest', 'landscape · top-down · drag · focal', G.v, _cone, a: .4),
      PopSpec('Vertigo street', 'landscape · pseudo3D · drag · result', G.h, _vertigo, a: .5),
      PopSpec('Turning cube', 'toy · isometric · spin · result', G.spinB, _cube, a: .5, b: .1),
      PopSpec('Telescope moon', 'instrument · bold · pull · focal', G.h, _telescope, a: .4),
      PopSpec('Curious eye', 'creature · bold · drag v · distance', G.v, _eye, a: .5),
      PopSpec('Lighthouse', 'landscape · neon · spin · orbit+focal', G.spinB, _lighthouse, a: .4, b: .15),
      PopSpec('Fisheye grid', 'text-art · neon wild · pinch · focal', G.v, _fisheye, a: .3),
      PopSpec('Drone altitude', 'weather · top-down · drag v', G.v, _drone, a: .5),
      PopSpec('Crop finder', 'material · light · drag · result', G.h, _crop, a: .5),
      PopSpec('Turntable figure', 'character · isometric · spin', G.spinB, _turntable, a: .5, b: .0),
      PopSpec('Railway vanish', 'landscape · flat · drag · focal', G.h, _rails, a: .4),
      PopSpec('Binocular bird', 'nature · bold · drag · focal', G.h, _binocs, a: .4),
      PopSpec('Paper layers', 'material · pseudo3D · drag', G.hv, _paper, a: .5, b: .5),
      PopSpec('Powers of ten', 'cosmic · neon · drag v · distance', G.v, _powers, a: .3),
      PopSpec('Kite string', 'weather · light · drag · distance', G.hv, _kite, a: .5, b: .5),
      PopSpec('Circle the house', 'map · top-down · spin · orbit', G.spinB, _house, a: .5, b: .6),
      PopSpec('Synth horizon', 'cosmic · neon wild · drag', G.hv, _synth, a: .5, b: .5),
      PopSpec('Lens face', 'character · light · drag · focal', G.h, _face, a: .3),
    ];

void _subject(Canvas c, Offset o, double r) {
  final pts = <Offset>[];
  for (var k = 0; k < 10; k++) {
    pts.add(pol(o, k.isEven ? r : r * .45, -math.pi / 2 + k * math.pi / 5));
  }
  c.drawPath(poly(pts), fl(kYellow));
  c.drawPath(poly(pts), st(kInk, 1.4));
}

void _camIcon(Canvas c, Offset o, double ang, Color col, [double k = 1]) {
  c.save();
  c.translate(o.dx, o.dy);
  c.rotate(ang);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 16 * k, height: 12 * k), Radius.circular(2 * k)), fl(col));
  c.drawPath(poly([Offset(8 * k, -3 * k), Offset(13 * k, -6 * k), Offset(13 * k, 6 * k), Offset(8 * k, 3 * k)]), fl(col));
  c.drawCircle(Offset(-2 * k, 0), 2.5 * k, fl(kInk));
  c.restore();
}

void _dolly(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final y = s.height / 2, sub = Offset(s.width - 24, y);
  for (var x = 6.0; x < s.width - 40; x += 9) {
    c.drawLine(Offset(x, y - 12), Offset(x, y + 12), st(const Color(0xFF6B4A2A), 3));
  }
  c.drawLine(Offset(4, y - 9), Offset(s.width - 40, y - 9), st(kCream, 1.5));
  c.drawLine(Offset(4, y + 9), Offset(s.width - 40, y + 9), st(kCream, 1.5));
  final cx = lerp(s.width - 52, 16, p.a), cam = Offset(cx, y);
  final reach = sub.dx - cx;
  c.drawPath(poly([cam, Offset(sub.dx + 8, y - reach * .45), Offset(sub.dx + 8, y + reach * .45)]), fl(al(kOrange, .22)));
  _subject(c, sub, 13);
  _camIcon(c, cam, 0, kOrange);
}

void _satellite(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF0E1230));
  final ctr = s.center(Offset.zero), r = 24 + p.a * 32;
  c.drawCircle(ctr, r, st(al(kCyan, .3), 1));
  c.drawCircle(ctr, 18, fl(kBlue));
  c.drawPath(Path()..addOval(Rect.fromCircle(center: ctr + const Offset(-5, -4), radius: 8)), fl(kLime));
  _subject(c, ctr + const Offset(6, 6), 5);
  final a = p.b * _pi2 + p.t * .25, q = pol(ctr, r, a);
  c.drawLine(q, ctr, st(al(kOrange, .5), 1));
  c.drawRect(Rect.fromCenter(center: q, width: 8, height: 8), fl(kCream));
  c.drawLine(pol(q, 12, a + math.pi / 2), pol(q, 12, a - math.pi / 2), st(kCyan, 4));
}

void _cone(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF2D5A2A));
  final cam = Offset(s.width / 2, s.height - 10), half = lerp(.95, .12, p.a);
  final trees = [for (var k = 0; k < 16; k++) Offset(10 + (k * 47 % 136).toDouble(), 10 + (k * 31 % 78).toDouble())];
  c.drawPath(poly([cam, pol(cam, 160, -math.pi / 2 - half), pol(cam, 160, -math.pi / 2 + half)]), fl(al(kViolet, .35)));
  for (final t in trees) {
    final ang = (t - cam).direction + math.pi / 2;
    final inside = ang.abs() < half;
    final r = inside ? 6 + (1 - half) * 4 : 5.0;
    c.drawCircle(t, r, fl(inside ? kLime : const Color(0xFF3E7A3A)));
  }
  _subject(c, Offset(s.width / 2, 34), 9);
  _camIcon(c, cam, -math.pi / 2, kViolet);
}

void _vertigo(Canvas c, Size s, PopP p) {
  vgrad(c, Offset.zero & s, kPink, kOrange);
  final k = lerp(.5, 1.8, p.a), base = s.height - 14, cx = s.width / 2;
  for (var i = -3; i <= 3; i++) {
    if (i == 0) continue;
    final w = 18 * k, h = (36 + (i.abs() % 2) * 18) * k;
    final x = cx + i * w * 1.1;
    c.drawRect(Rect.fromLTWH(x - w / 2, base - h, w, h), fl(i.isEven ? kViolet : kBlue));
    c.drawRect(Rect.fromLTWH(x - w / 4, base - h + 6 * k, w / 2, 4 * k), fl(kYellow));
  }
  c.drawRect(Rect.fromLTWH(0, base, s.width, 14), fl(kInk));
  c.drawCircle(Offset(cx, base - 38), 8, fl(kCream));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, base - 16), width: 18, height: 30), const Radius.circular(6)), fl(kYellow));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, base - 16), width: 18, height: 30), const Radius.circular(6)), st(kInk, 1.5));
}

void _cube(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final ctr = s.center(const Offset(0, 6)), yaw = p.b * _pi2 + p.t * .2, sz = 26 + (1 - p.a) * 14;
  c.drawOval(Rect.fromCenter(center: ctr + Offset(0, sz * .9), width: sz * 3, height: sz * .8), fl(const Color(0xFF2A2A2E)));
  Offset pr(double x, double y, double z) {
    final xr = x * math.cos(yaw) - z * math.sin(yaw), zr = x * math.sin(yaw) + z * math.cos(yaw);
    return ctr + Offset(xr * sz, y * sz + zr * sz * .35);
  }

  final faces = <(double, List<Offset>, Color)>[];
  const cols = [kOrange, kCyan, kPink, kLime];
  for (var f = 0; f < 4; f++) {
    final a0 = f * math.pi / 2;
    final x0 = math.cos(a0), z0 = math.sin(a0), x1 = math.cos(a0 + math.pi / 2), z1 = math.sin(a0 + math.pi / 2);
    final nz = (x0 + x1) * math.sin(yaw) + (z0 + z1) * math.cos(yaw);
    faces.add((nz, [pr(x0, -.7, z0), pr(x1, -.7, z1), pr(x1, .7, z1), pr(x0, .7, z0)], cols[f]));
  }
  faces.sort((x, y) => x.$1.compareTo(y.$1));
  for (final (_, pts, col) in faces.skip(2)) {
    c.drawPath(poly(pts), fl(col));
    c.drawPath(poly(pts), st(kInk, 1.6));
  }
  final top = [for (var f = 0; f < 4; f++) pr(math.cos(f * math.pi / 2), -.7, math.sin(f * math.pi / 2))];
  c.drawPath(poly(top), fl(kYellow));
  c.drawPath(poly(top), st(kInk, 1.6));
}

void _telescope(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF101830));
  final view = Offset(s.width - 44, 46), vr = 34.0;
  c.drawCircle(view, vr, fl(Colors.black));
  final moon = 6 + p.a * 40;
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: view, radius: vr)));
  c.drawCircle(view + const Offset(4, 4), moon, fl(kCream));
  c.drawCircle(view + Offset(4 - moon * .3, 4 - moon * .2), moon * .18, fl(const Color(0xFFD9CDB8)));
  c.drawCircle(view + Offset(4 + moon * .3, 4 + moon * .3), moon * .12, fl(const Color(0xFFD9CDB8)));
  c.restore();
  c.drawCircle(view, vr, st(kViolet, 3));
  final t0 = Offset(14, s.height - 18), dir = const Offset(.82, -.57), ext = 30 + p.a * 40;
  for (var k = 0; k < 3; k++) {
    final a0 = t0 + dir * (k * ext / 3), a1 = t0 + dir * ((k + 1) * ext / 3 + 6);
    c.drawLine(a0, a1, st(k == 0 ? kViolet : (k == 1 ? const Color(0xFF7A5FE0) : const Color(0xFFB9A6FF)), 14 - k * 3.0));
  }
  c.drawLine(t0 + const Offset(6, 4), t0 + const Offset(0, 18), st(kCream, 2));
  c.drawLine(t0 + const Offset(6, 4), t0 + const Offset(16, 18), st(kCream, 2));
}

void _eye(Canvas c, Size s, PopP p) {
  bg(c, s, kLime);
  final near = 1 - p.a, r = 12 + near * 34, ctr = Offset(s.width / 2, s.height * .52);
  c.drawCircle(ctr, r, fl(kWhite));
  c.drawCircle(ctr, r, st(kInk, 2 + near * 2));
  final look = Offset(math.sin(p.t * .9) * r * .25, math.cos(p.t * .7) * r * .1);
  c.drawCircle(ctr + look, r * .48, fl(kBlue));
  c.drawCircle(ctr + look, r * .22, fl(kInk));
  c.drawCircle(ctr + look - Offset(r * .14, r * .14), r * .08, fl(kWhite));
  final blink = frac(p.t * .25) > .96 ? 1.0 : 0.0;
  if (blink > 0) c.drawRect(Rect.fromCenter(center: ctr, width: r * 2.2, height: r * 2.2), fl(kLime));
  for (var k = -2; k <= 2; k++) {
    final a = -math.pi / 2 + k * .32;
    c.drawLine(pol(ctr, r + 2, a), pol(ctr, r + 6 + near * 8, a), st(kInk, 2));
  }
}

void _lighthouse(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF0A1028));
  final top = Offset(s.width / 2, 46), half = lerp(.55, .06, p.a), dir = p.b * _pi2 + p.t * .5;
  final reach = 140.0;
  final beam = poly([top, pol(top, reach, dir - half, .22), pol(top, reach, dir + half, .22)]);
  c.drawPath(beam, fl(al(kYellow, .4 + (1 - half) * .4)));
  c.drawRect(Rect.fromLTWH(0, s.height - 26, s.width, 26), fl(kBlue));
  for (var k = 0; k < 5; k++) {
    final x = frac(k * .23 + p.t * .03) * s.width;
    c.drawLine(Offset(x, s.height - 16 + (k % 2) * 6), Offset(x + 10, s.height - 16 + (k % 2) * 6), st(kCyan, 1.5));
  }
  c.drawPath(poly([Offset(top.dx - 9, s.height - 24), Offset(top.dx - 6, top.dy + 6), Offset(top.dx + 6, top.dy + 6), Offset(top.dx + 9, s.height - 24)]), fl(kCream));
  c.drawRect(Rect.fromLTWH(top.dx - 8, top.dy + 22, 16, 7), fl(kRed));
  c.drawRect(Rect.fromLTWH(top.dx - 8, top.dy + 44, 16, 7), fl(kRed));
  c.drawCircle(top, 7, fl(kYellow));
}

void _fisheye(Canvas c, Size s, PopP p) {
  bg(c, s, Colors.black);
  final ctr = s.center(Offset.zero), k = lerp(1.6, 0, p.a);
  Offset warp(Offset q) {
    final d = q - ctr, r = d.distance / 80;
    final f = 1 / (1 + k * r * r * .6);
    return ctr + d * f * (1 + k * .35);
  }

  for (var i = -8; i <= 8; i++) {
    final h = Path(), v = Path();
    for (var j = 0; j <= 30; j++) {
      final t = -90 + j * 6.0;
      final a = warp(ctr + Offset(t, i * 9.0)), b = warp(ctr + Offset(i * 11.0, t));
      j == 0 ? h.moveTo(a.dx, a.dy) : h.lineTo(a.dx, a.dy);
      j == 0 ? v.moveTo(b.dx, b.dy) : v.lineTo(b.dx, b.dy);
    }
    c.drawPath(h, st(al(kPink, .8), 1.2));
    c.drawPath(v, st(al(kCyan, .8), 1.2));
  }
  _subject(c, ctr, 10 + k * 6);
}

void _drone(Canvas c, Size s, PopP p) {
  final z = lerp(2.2, .7, p.a);
  bg(c, s, const Color(0xFF6FBF4A));
  final ctr = s.center(Offset.zero);
  c.save();
  c.translate(ctr.dx, ctr.dy);
  c.scale(z);
  c.drawPath(Path()..addOval(const Rect.fromLTWH(-60, 10, 70, 40)), fl(kBlue));
  c.drawRect(const Rect.fromLTWH(-80, -6, 160, 8), fl(const Color(0xFFD9CDB8)));
  for (final (x, y) in [(20.0, -30.0), (40.0, -20.0), (-40.0, -28.0), (30.0, 24.0)]) {
    c.drawCircle(Offset(x, y), 7, fl(const Color(0xFF2D6A2A)));
  }
  c.drawRect(const Rect.fromLTWH(-14, -26, 14, 12), fl(kRed));
  c.restore();
  final lift = 6 + p.a * 22;
  c.drawOval(Rect.fromCenter(center: ctr + Offset(lift * .6, lift), width: 28, height: 12), fl(al(kInk, .35)));
  for (final o in [const Offset(-12, -12), const Offset(12, -12), const Offset(-12, 12), const Offset(12, 12)]) {
    c.drawLine(ctr, ctr + o, st(kInk, 3));
    c.drawOval(Rect.fromCenter(center: ctr + o, width: 14, height: 4 + 2 * math.sin(p.t * 30).abs()), fl(al(kWhite, .8)));
  }
  c.drawCircle(ctr, 6, fl(kOrange));
}

void _crop(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final img = Rect.fromLTWH(10, 10, s.width - 20, s.height - 20);
  c.drawRect(img, fl(kCyan));
  c.drawPath(poly([img.bottomLeft, Offset(img.left + 40, img.top + 50), Offset(img.left + 64, img.top + 70), Offset(img.left + 100, img.top + 30), img.bottomRight]), fl(kLime));
  final sub = Offset(img.left + 100, img.top + 54);
  _subject(c, sub, 10);
  final w = lerp(26, img.width, p.a), h = w * .66;
  final r = Rect.fromCenter(center: Offset.lerp(sub, img.center, p.a)!, width: w, height: h);
  final outside = Path()
    ..fillType = PathFillType.evenOdd
    ..addRect(img)
    ..addRect(r);
  c.drawPath(outside, fl(al(kInk, .6)));
  final L = math.min(10.0, w / 3);
  for (final (q, dx, dy) in [(r.topLeft, 1.0, 1.0), (r.topRight, -1.0, 1.0), (r.bottomLeft, 1.0, -1.0), (r.bottomRight, -1.0, -1.0)]) {
    c.drawLine(q, q + Offset(dx * L, 0), st(kOrange, 3));
    c.drawLine(q, q + Offset(0, dy * L), st(kOrange, 3));
  }
}

void _turntable(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final ctr = Offset(s.width / 2, s.height * .7), yaw = p.b * _pi2 + p.t * .3;
  c.drawOval(Rect.fromCenter(center: ctr + const Offset(0, 6), width: 110, height: 34), fl(kInk));
  c.drawOval(Rect.fromCenter(center: ctr, width: 110, height: 34), fl(kCyan));
  for (var k = 0; k < 8; k++) {
    c.drawCircle(pol(ctr, 50, yaw + k * _pi2 / 8, .3), 2, fl(kInk));
  }
  final face = math.cos(yaw), side = math.sin(yaw);
  final head = ctr - const Offset(0, 54);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: ctr - const Offset(0, 24), width: 30, height: 40), const Radius.circular(10)), fl(kOrange));
  c.drawCircle(head, 16, fl(kOrange));
  if (face > -.2) {
    for (final ex in [-6.0, 6.0]) {
      c.drawCircle(head + Offset(ex * face + side * 8, -2), 3, fl(kInk));
    }
    c.drawArc(Rect.fromCenter(center: head + Offset(side * 8, 5), width: 10 * face.abs() + 2, height: 6), 0, math.pi, false, st(kInk, 2));
  } else {
    c.drawArc(Rect.fromCenter(center: head, width: 32, height: 32), math.pi * 1.1, math.pi * .8, false, st(const Color(0xFFB3501F), 5));
  }
  _camIcon(c, Offset(s.width - 18, 20), math.pi * .75, kCyan, .8);
}

void _rails(Canvas c, Size s, PopP p) {
  vgrad(c, Offset.zero & s, kBlue, kCyan);
  final hy = lerp(30, 64, p.a), vx = s.width / 2;
  c.drawRect(Rect.fromLTRB(0, hy, s.width, s.height), fl(const Color(0xFF6FBF4A)));
  final spread = lerp(160, 50, p.a);
  final l0 = Offset(vx - spread / 2, s.height), r0 = Offset(vx + spread / 2, s.height), v = Offset(vx, hy);
  c.drawPath(poly([l0, v, r0]), fl(const Color(0xFFB79A6A)));
  for (var k = 1; k < 12; k++) {
    final t = math.pow(k / 12, 1.8).toDouble();
    final y = lerp(s.height, hy, 1 - t);
    final w = spread * t;
    c.drawLine(Offset(vx - w * .55, y), Offset(vx + w * .55, y), st(const Color(0xFF6B4A2A), 1 + t * 3));
  }
  c.drawLine(Offset(vx - spread * .35, s.height), v, st(kInk, 2.5));
  c.drawLine(Offset(vx + spread * .35, s.height), v, st(kInk, 2.5));
  c.drawCircle(Offset(vx + 34, hy - 12), 8, fl(kYellow));
}

void _binocs(Canvas c, Size s, PopP p) {
  bg(c, s, kInk);
  final m = lerp(.8, 3.4, p.a);
  final cl = Offset(s.width / 2 - 30, s.height / 2), cr = Offset(s.width / 2 + 30, s.height / 2);
  final clip = Path()
    ..addOval(Rect.fromCircle(center: cl, radius: 38))
    ..addOval(Rect.fromCircle(center: cr, radius: 38));
  c.save();
  c.clipPath(clip);
  vgrad(c, Offset.zero & s, kCyan, kLime);
  final bird = s.center(Offset(math.sin(p.t * .8) * 6, math.cos(p.t * .6) * 3));
  c.save();
  c.translate(bird.dx, bird.dy);
  c.scale(m);
  c.drawLine(const Offset(-8, 10), const Offset(10, 10), st(const Color(0xFF6B4A2A), 2));
  c.drawOval(Rect.fromCenter(center: Offset.zero, width: 14, height: 11), fl(kRed));
  c.drawCircle(const Offset(5, -5), 4.5, fl(kRed));
  c.drawCircle(const Offset(6, -6), 1.1, fl(kInk));
  c.drawPath(poly([const Offset(9, -5), const Offset(13, -4), const Offset(9, -3)]), fl(kYellow));
  final flap = math.sin(p.t * 10) * 3;
  c.drawPath(poly([const Offset(-3, -1), Offset(-9, -6 - flap), const Offset(2, -2)]), fl(const Color(0xFFB02E2E)));
  c.restore();
  c.restore();
  c.drawPath(clip, st(kViolet, 5));
}

void _paper(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final push = 1 - p.a, sway = (p.b - .5) * 2;
  const cols = [Color(0xFFB9A6FF), kViolet, kPink, kOrange];
  for (var l = 0; l < 4; l++) {
    final depth = (l + 1) / 4, k = 1 + push * depth * .6;
    final dx = sway * depth * 22, base = s.height * (.45 + l * .14);
    final path = Path()..moveTo(-20, s.height + 4);
    for (var i = 0; i <= 12; i++) {
      final x = -20 + i * 17.0;
      final y = base - (math.sin(i * 1.3 + l * 2) * .5 + .5) * 22 * k;
      path.lineTo(s.width / 2 + (x - s.width / 2) * k + dx, y);
    }
    path
      ..lineTo(s.width + 20, s.height + 4)
      ..close();
    c.drawPath(path.shift(const Offset(0, 3)), fl(al(kInk, .25)));
    c.drawPath(path, fl(cols[l]));
    if (l == 2) _subject(c, Offset(s.width / 2 + dx, base - 22 * k), 7 * k);
  }
}

void _powers(Canvas c, Size s, PopP p) {
  bg(c, s, Colors.black);
  final ctr = s.center(Offset.zero), z = (1 - p.a) * 3 + p.t * .15;
  const cols = [kPink, kViolet, kCyan, kLime, kYellow, kOrange];
  for (var k = 6; k >= 0; k--) {
    final lvl = k - frac(z);
    final r = 6 * math.pow(2.4, lvl).toDouble();
    if (r > 200) continue;
    final col = cols[(k + z.floor()) % cols.length];
    c.drawCircle(ctr, r, fl(al(col, .9)));
    c.drawCircle(ctr, r, st(Colors.black, 2));
    c.drawCircle(pol(ctr, r * .62, k * 1.7), r * .12, fl(Colors.black));
  }
}

void _kite(Canvas c, Size s, PopP p) {
  vgrad(c, Offset.zero & s, kCyan, kCream);
  final hand = Offset(16, s.height - 10), L = 40 + p.a * 110;
  final ang = -math.pi / 4 - (p.b - .5) * .9 + math.sin(p.t * 1.3) * .06;
  final kite = Offset(hand.dx + math.cos(ang) * L * .8, hand.dy + math.sin(ang) * L * .8);
  final kp = Offset(kite.dx.clamp(14, s.width - 14), kite.dy.clamp(12, s.height - 20));
  final sag = Offset.lerp(hand, kp, .5)! + Offset(0, 10 + p.a * 10);
  c.drawPath(Path()
    ..moveTo(hand.dx, hand.dy)
    ..quadraticBezierTo(sag.dx, sag.dy, kp.dx, kp.dy), st(kOrange, 1.5));
  final k = 1.3 - p.a * .6;
  final body = [kp + Offset(0, -14 * k), kp + Offset(10 * k, 0), kp + Offset(0, 18 * k), kp + Offset(-10 * k, 0)];
  c.drawPath(poly(body), fl(kRed));
  c.drawLine(body[0], body[2], st(kInk, 1.2));
  c.drawLine(body[1], body[3], st(kInk, 1.2));
  final tail = Path()..moveTo(body[2].dx, body[2].dy);
  for (var i = 1; i <= 4; i++) {
    final q = body[2] + Offset(math.sin(p.t * 4 + i) * 5, i * 7.0 * k);
    tail.lineTo(q.dx, q.dy);
    c.drawPath(poly([q + const Offset(-3, 0), q + const Offset(0, -3), q + const Offset(3, 0), q + const Offset(0, 3)]), fl(kYellow));
  }
  c.drawPath(tail, st(kInk, 1));
  c.drawCircle(hand, 6, fl(const Color(0xFFE9B48A)));
}

void _house(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF4FA048));
  final ctr = s.center(Offset.zero);
  for (var k = 0; k < 10; k++) {
    c.drawCircle(pol(ctr, 50 + (k % 2) * 4, k * _pi2 / 10 + .2, .82), 7, fl(const Color(0xFF2D6A2A)));
  }
  c.drawRect(Rect.fromCenter(center: ctr, width: 26, height: 22), fl(kCream));
  c.drawPath(poly([ctr + const Offset(-15, -11), ctr + const Offset(0, -2), ctr + const Offset(15, -11), ctr + const Offset(0, -20)]), fl(kRed));
  c.drawRect(Rect.fromCenter(center: ctr + const Offset(0, 6), width: 6, height: 10), fl(kYellow));
  final a = p.b * _pi2, cam = pol(ctr, 36 + p.a * 20, a, .82);
  c.drawArc(Rect.fromCenter(center: ctr, width: (36 + p.a * 20) * 2, height: (36 + p.a * 20) * 1.64), a - .9, .9, false, st(kCyan, 3));
  final look = (ctr - cam).direction;
  c.drawPath(poly([cam, pol(cam, 40, look - .35), pol(cam, 40, look + .35)]), fl(al(kCyan, .35)));
  _camIcon(c, cam, look, kCyan, .9);
}

void _synth(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF14082A));
  final hy = s.height * lerp(.28, .5, p.a), vx = s.width / 2 + (p.b - .5) * 80;
  c.drawCircle(Offset(s.width / 2, hy), 26, fl(kOrange));
  for (var k = 0; k < 4; k++) {
    c.drawRect(Rect.fromLTWH(s.width / 2 - 28, hy - 18 + k * 6.0, 56, 2 + k * .6), fl(const Color(0xFF14082A)));
  }
  c.drawRect(Rect.fromLTRB(0, hy, s.width, s.height), fl(const Color(0xFF14082A)));
  final fov = lerp(3.2, .8, p.a);
  for (var i = -10; i <= 10; i++) {
    c.drawLine(Offset(vx, hy), Offset(s.width / 2 + i * 16 * fov, s.height + 60), st(kPink, 1.2));
  }
  for (var k = 0; k < 8; k++) {
    final t = frac(k / 8 + p.t * .25);
    final y = hy + math.pow(t, 2.4) * (s.height - hy);
    c.drawLine(Offset(0, y), Offset(s.width, y), st(al(kCyan, t + .1), 1.2));
  }
  c.drawLine(Offset(0, hy), Offset(s.width, hy), st(kPink, 2));
}

void _face(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final w = 1 - p.a, ctr = Offset(s.width / 2, s.height * .52);
  final fw = lerp(66, 52, w), fh = lerp(84, 90, w);
  c.drawOval(Rect.fromCenter(center: ctr, width: fw, height: fh), fl(kOrange));
  c.drawOval(Rect.fromCenter(center: ctr, width: fw, height: fh), st(kInk, 2.4));
  final ew = lerp(14, 10, w), es = lerp(14, 11, w);
  for (final sx in [-1.0, 1.0]) {
    final e = ctr + Offset(sx * es, -10);
    c.drawOval(Rect.fromCenter(center: e, width: ew, height: ew * .8), fl(kWhite));
    c.drawCircle(e, 2.6, fl(kInk));
  }
  final nose = lerp(8, 30, w * w);
  c.drawOval(Rect.fromCenter(center: ctr + const Offset(0, 6), width: nose, height: nose * .8), fl(kRed));
  c.drawOval(Rect.fromCenter(center: ctr + const Offset(0, 6), width: nose, height: nose * .8), st(kInk, 1.6));
  final ear = lerp(9, 2, w);
  for (final sx in [-1.0, 1.0]) {
    c.drawOval(Rect.fromCenter(center: ctr + Offset(sx * (fw / 2 + ear / 2 - 2), -2), width: ear, height: 18), fl(kOrange));
  }
  c.drawArc(Rect.fromCenter(center: ctr + Offset(0, 22 + nose * .1), width: 22, height: 10), 0, math.pi, false, st(kInk, 2.4));
}
