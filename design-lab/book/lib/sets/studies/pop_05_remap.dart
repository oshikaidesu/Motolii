// Pop 05 sheet 1, Time remap: one value per panel, speed -2..+3 where 0 is hold and below 0 is reverse.
// Colour = function: forward cyan, fast orange, hold yellow, reverse pink.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

import 'pop_05_kit.dart';

double sp(PopP p) {
  final v = p.a * 5 - 2;
  return v.abs() < .2 ? 0 : v;
}

Color spc(double v) => v == 0 ? kYellow : (v < 0 ? kPink : (v > 1.6 ? kOrange : kCyan));

List<PopSpec> remapPanels() => [
      PopSpec('Ghost runner', 'physics · crisp · flick · result', G.h, _runner, a: .6),
      PopSpec('Hourglass flip', 'material · light · drag v · mechanism', G.v, _hourglass, a: 0.3),
      PopSpec('Strobe bounce', 'physics · flat · drag · result', G.h, _strobe, a: 0.5),
      PopSpec('Scratch vinyl', 'instrument · pseudo3D · spin', G.spin, _vinyl, a: 0.9),
      PopSpec('Flipbook riffle', 'toy · light · flick · result', G.h, _flipbook, a: .6),
      PopSpec('Paper boat river', 'landscape · top-down · drag', G.h, _boat, a: .6),
      PopSpec('Melting clock', 'material · neon · drag v', G.v, _melt, a: 0.4),
      PopSpec('Footprints', 'creature · top-down · drag v', G.v, _feet, a: 0.3),
      PopSpec('Film conveyor', 'machine · isometric · drag', G.h, _conveyor, a: .6),
      PopSpec('Spiral galaxy', 'cosmic · neon · spin', G.spin, _galaxy, a: 0.3),
      PopSpec('Freeze ray', 'weather · bold · drag v · result', G.v, _freeze, a: 0.4),
      PopSpec('Rubber time', 'material · light · stretch · result', G.h, _rubber, a: 0.4),
      PopSpec('Chevron river', 'text-art · crisp · drag', G.h, _chevrons, a: 0.9),
      PopSpec('Sun race', 'landscape · flat · drag · result', G.h, _sunrace, a: 0.3),
      PopSpec('Soda fizz', 'food · light · drag v', G.v, _soda, a: .6),
      PopSpec('Sleepy blob', 'character · bold · drag', G.h, _blob, a: 0.92),
      PopSpec('Cassette reels', 'machine · pseudo3D · drag', G.h, _cassette, a: .6),
      PopSpec('Held cards', 'toy · isometric · stack · result', G.v, _cards, a: 0.4),
      PopSpec('Rain up', 'weather · flat · drag v', G.v, _rain, a: 0.3),
      PopSpec('Warp tunnel', 'cosmic · neon wild · pseudo3D', G.v, _warp, a: 0.95),
    ];

void _runner(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final v = sp(p), col = spc(v), gy = s.height * .66;
  c.drawRect(Rect.fromLTWH(0, gy + 16, s.width, s.height), fl(kP1));
  final x = 20 + frac(p.ph * .35) * (s.width - 40);
  final dir = v < 0 ? -1.0 : 1.0, gap = 4 + v.abs() * 8;
  for (var i = 6; i >= 1; i--) {
    c.drawCircle(Offset(x - dir * gap * i, gy), 15 - i * 1.2, fl(al(col, .55 - i * .07)));
  }
  final sq = 1 + v.abs() * .15;
  c.drawOval(Rect.fromCenter(center: Offset(x, gy), width: 30 * sq, height: 30 / sq), fl(col));
  c.drawCircle(Offset(x + dir * 6, gy - 4), 3, fl(kInk));
  if (v == 0) {
    c.drawRect(Rect.fromLTWH(x - 8, gy - 44, 5, 16), fl(kYellow));
    c.drawRect(Rect.fromLTWH(x + 3, gy - 44, 5, 16), fl(kYellow));
  } else {
    for (var k = 0; k < 3; k++) {
      final y = gy - 10 + k * 10.0;
      c.drawLine(Offset(x - dir * (22 + k * 4), y), Offset(x - dir * (22 + k * 4 + 8 + v.abs() * 8), y), st(kCream, 2));
    }
  }
}

void _hourglass(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final v = sp(p), ctr = s.center(Offset.zero);
  c.save();
  c.translate(ctr.dx, ctr.dy);
  if (v < 0) c.rotate(math.pi);
  final w = 30.0, h = 44.0;
  final top = poly([Offset(-w, -h), Offset(w, -h), const Offset(3, -2), const Offset(-3, -2)]);
  final bot = poly([const Offset(-3, 2), const Offset(3, 2), Offset(w, h), Offset(-w, h)]);
  final u = v == 0 ? .5 : frac(p.ph * .08 * v.sign);
  // sand top: shrinks; bottom: grows
  final tf = (1 - u), tl = -2 - (h - 2) * tf;
  c.save();
  c.clipPath(top);
  c.drawRect(Rect.fromLTRB(-w, tl, w, 0), fl(kOrange));
  c.restore();
  c.save();
  c.clipPath(bot);
  final mound = Path()
    ..moveTo(-w, h)
    ..lineTo(w, h)
    ..lineTo(w, h - h * u * .7)
    ..quadraticBezierTo(0, h - h * u * 1.5, -w, h - h * u * .7)
    ..close();
  c.drawPath(mound, fl(kOrange));
  c.restore();
  if (v != 0) {
    c.drawLine(const Offset(0, 0), Offset(0, h - h * u * 1.1), st(kOrange, .8 + v.abs() * .9));
  } else {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 22, height: 7), const Radius.circular(3)), fl(kYellow));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 22, height: 7), const Radius.circular(3)), st(kInk, 1.5));
  }
  c.drawPath(top, st(kInk, 2.2));
  c.drawPath(bot, st(kInk, 2.2));
  c.drawLine(Offset(-w - 6, -h - 2), Offset(w + 6, -h - 2), st(kInk, 4));
  c.drawLine(Offset(-w - 6, h + 2), Offset(w + 6, h + 2), st(kInk, 4));
  c.restore();
}

void _strobe(Canvas c, Size s, PopP p) {
  bg(c, s);
  final v = sp(p), col = spc(v), base = s.height - 18;
  c.drawRect(Rect.fromLTWH(0, base + 8, s.width, 10), fl(kCream));
  double yAt(double x) => base - (math.sin((x - 4) / 50 * math.pi).abs()) * 72;
  final dx = v == 0 ? 0.0 : 5 + v.abs() * 8;
  final head = 10 + frac(p.ph * .1) * (s.width - 20);
  if (dx == 0) {
    c.drawCircle(Offset(head, yAt(head)), 8, fl(kYellow));
    c.drawCircle(Offset(head, yAt(head)), 14, st(kYellow, 2));
    return;
  }
  final arc = Path()..moveTo(4, yAt(4));
  for (var x = 4.0; x <= s.width - 4; x += 2) {
    arc.lineTo(x, yAt(x));
  }
  c.drawPath(arc, st(al(kCream, .18), 1.5));
  for (var i = 9; i >= 0; i--) {
    final x = 10 + frac((head - (v < 0 ? -1 : 1) * i * dx - 10) / (s.width - 20)) * (s.width - 20);
    final o = math.max(.15, 1 - i * .09);
    c.drawCircle(Offset(x, yAt(x)), i == 0 ? 8 : 5.5, i == 0 ? fl(col) : fl(al(col, o * .7)));
  }
}

void _vinyl(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final v = sp(p), ctr = Offset(s.width * .44, s.height * .55);
  const sy = .55;
  c.drawOval(Rect.fromCenter(center: ctr + const Offset(0, 6), width: 116, height: 116 * sy), fl(const Color(0xFF0C0C0D)));
  c.drawOval(Rect.fromCenter(center: ctr, width: 116, height: 116 * sy), fl(const Color(0xFF151517)));
  for (var r = 22.0; r < 56; r += 5) {
    c.drawOval(Rect.fromCenter(center: ctr, width: r * 2, height: r * 2 * sy), st(al(kCream, .12), 1));
  }
  final rot = p.ph * 2.2;
  c.drawArc(Rect.fromCenter(center: ctr, width: 92, height: 92 * sy), rot, .9, false, st(al(spc(v), .9), 2.5));
  c.drawOval(Rect.fromCenter(center: ctr, width: 34, height: 34 * sy), fl(v < 0 ? kPink : kOrange));
  c.drawLine(ctr, pol(ctr, 16, rot, sy), st(kCream, 2.4));
  c.drawCircle(ctr, 2, fl(kInk));
  final pivot = Offset(s.width - 18, 16);
  c.drawCircle(pivot, 6, fl(kCream));
  c.drawLine(pivot, Offset(ctr.dx + 34, ctr.dy - 6), st(kCream, 3));
  c.drawRect(Rect.fromCenter(center: Offset(ctr.dx + 34, ctr.dy - 6), width: 9, height: 6), fl(kCream));
}

void _flipbook(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final v = sp(p), fan = (v.abs() * 3).round().clamp(0, 9);
  final origin = Offset(28, s.height - 18);
  for (var i = fan; i >= 1; i--) {
    c.save();
    c.translate(origin.dx, origin.dy);
    c.rotate((v < 0 ? 1 : -1) * i * .07);
    c.drawRect(const Rect.fromLTWH(0, -76, 100, 76), fl(i.isEven ? kWhite : const Color(0xFFEDE2D0)));
    c.drawRect(const Rect.fromLTWH(0, -76, 100, 76), st(al(kInk, .5), 1));
    c.restore();
  }
  c.drawRect(Rect.fromLTWH(origin.dx, origin.dy - 76, 100, 76), fl(kWhite));
  c.drawRect(Rect.fromLTWH(origin.dx, origin.dy - 76, 100, 76), st(kInk, 2));
  // stick figure in a running pose
  final f = v == 0 ? 0.0 : p.ph * 2.4;
  final hip = Offset(origin.dx + 50, origin.dy - 30), sw = math.sin(f) * .8;
  final ink = st(v == 0 ? kOrange : (v < 0 ? kPink : kInk), 3);
  c.drawCircle(hip - const Offset(0, 30), 7, ink);
  c.drawLine(hip, hip - const Offset(0, 22), ink);
  c.drawLine(hip, hip + Offset(math.sin(sw) * 18, math.cos(sw) * 18), ink);
  c.drawLine(hip, hip + Offset(-math.sin(sw) * 18, math.cos(sw) * 18), ink);
  final sh = hip - const Offset(0, 18);
  c.drawLine(sh, sh + Offset(-math.sin(sw) * 14, math.cos(sw) * 12), ink);
  c.drawLine(sh, sh + Offset(math.sin(sw) * 14, math.cos(sw) * 12), ink);
}

void _boat(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF6FBF4A));
  final v = sp(p);
  final river = Path()
    ..moveTo(0, 30)
    ..cubicTo(50, 10, 90, 70, s.width, 40)
    ..lineTo(s.width, 92)
    ..cubicTo(90, 120, 50, 64, 0, 86)
    ..close();
  c.drawPath(river, fl(kBlue));
  for (var k = 0; k < 9; k++) {
    final u = frac(k / 9 + p.ph * .08);
    final x = u * s.width, y = 58 + math.sin(u * 6.3 + k) * 6 + (k.isEven ? -10 : 12) + math.sin(u * math.pi * 2) * -6;
    final d = v < 0 ? -1.0 : 1.0;
    c.drawLine(Offset(x - d * 5, y - 3), Offset(x, y), st(al(kCyan, .9), 2));
    c.drawLine(Offset(x - d * 5, y + 3), Offset(x, y), st(al(kCyan, .9), 2));
  }
  final u = frac(.3 + p.ph * .08), bx = u * s.width, by = 58 + math.sin(u * math.pi * 2) * -6;
  final d = v < 0 ? -1.0 : 1.0;
  c.drawPath(poly([Offset(bx - d * 12, by - 6), Offset(bx + d * 14, by), Offset(bx - d * 12, by + 6)]), fl(kWhite));
  c.drawPath(poly([Offset(bx - d * 6, by), Offset(bx + d * 4, by - 9), Offset(bx + d * 2, by)]), fl(v == 0 ? kYellow : kOrange));
  if (v == 0) c.drawCircle(Offset(bx, by), 15, st(kYellow, 2));
}

void _melt(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final v = sp(p), ctr = Offset(s.width / 2, s.height * .42);
  final melt = v == 0 ? 1.0 : (v < 0 ? .3 : (1.4 - v) / 1.4).clamp(0.0, 1.0);
  c.drawRect(Rect.fromLTWH(0, s.height * .62, s.width, 4), fl(kViolet));
  final face = Path();
  for (var i = 0; i <= 40; i++) {
    final a = i / 40 * math.pi * 2, x = math.cos(a), y = math.sin(a);
    final droop = y > 0 ? y * y * melt * 34 * (1 - x.abs() * .4) : 0.0;
    final pt = ctr + Offset(x * 34, y * 34 + droop);
    i == 0 ? face.moveTo(pt.dx, pt.dy) : face.lineTo(pt.dx, pt.dy);
  }
  face.close();
  c.drawPath(face, fl(kYellow));
  c.drawPath(face, st(kOrange, 2.5));
  for (var k = 0; k < 3; k++) {
    final x = ctr.dx - 14 + k * 14.0, len = melt * (10 + k * 7 + 4 * math.sin(p.t * 1.4 + k));
    c.drawLine(Offset(x, ctr.dy + 34 + melt * 30), Offset(x, ctr.dy + 34 + melt * 30 + len), st(kYellow, 5));
  }
  final hand = v < 0 ? kPink : kInk;
  c.drawLine(ctr, pol(ctr, 24, p.ph * 1.5 - math.pi / 2), st(hand, 3));
  c.drawLine(ctr, pol(ctr, 15, p.ph * .125 - math.pi / 2), st(hand, 4));
  c.drawCircle(ctr, 3, fl(hand));
  if (v > 1.6) {
    for (var k = 0; k < 3; k++) {
      c.drawLine(Offset(ctr.dx + 42, ctr.dy - 14 + k * 12.0), Offset(ctr.dx + 56, ctr.dy - 14 + k * 12.0), st(kOrange, 2.5));
    }
  }
}

void _feet(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFFE9C98A));
  final v = sp(p), stride = 16 + v.abs() * 9, d = v < 0 ? 1.0 : -1.0;
  void foot(Offset o, double alpha, bool left) {
    c.save();
    c.translate(o.dx, o.dy);
    if (d > 0) c.rotate(math.pi);
    final col = al(v == 0 ? kOrange : (v < 0 ? kPink : kInk), alpha);
    c.drawOval(Rect.fromCenter(center: Offset.zero, width: 10, height: 16), fl(col));
    c.drawOval(Rect.fromCenter(center: const Offset(0, 11), width: 8, height: 7), fl(col));
    for (var t = 0; t < 4; t++) {
      c.drawCircle(Offset(-4.5 + t * 3.0 + (left ? -.5 : .5), -10.5 - (t == 0 || t == 3 ? 0 : 1.2)), 1.6, fl(col));
    }
    c.restore();
  }

  if (v == 0) {
    foot(Offset(s.width / 2 - 8, s.height / 2), 1, true);
    foot(Offset(s.width / 2 + 8, s.height / 2), 1, false);
    c.drawCircle(s.center(Offset.zero), 26, st(kYellow, 3));
    return;
  }
  final shift = frac(p.ph * .5 * v.sign.abs()) * stride * 2;
  for (var i = -1; i < 9; i++) {
    final y = d < 0 ? s.height + 10 - i * stride - shift : -10 + i * stride + shift;
    final age = i / 8;
    foot(Offset(s.width / 2 + (i.isEven ? -9 : 9), y), (1 - age).clamp(.15, 1), i.isEven);
  }
}

void _conveyor(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final v = sp(p);
  const ix = Offset(.866, .5), iy = Offset(-.866, .5);
  final o = Offset(30, 22);
  Offset iso(double x, double y) => o + ix * x + iy * y;
  final belt = poly([iso(0, -14), iso(120, -14), iso(120, 14), iso(0, 14)]);
  c.drawPath(belt.shift(const Offset(0, 8)), fl(const Color(0xFF0E0E10)));
  c.drawPath(belt, fl(const Color(0xFF3A3A40)));
  for (var k = 0; k < 7; k++) {
    final x = frac(k / 7 + p.ph * .05) * 120;
    if (x < 4 || x > 104) continue;
    final fr = poly([iso(x, -11), iso(x + 14, -11), iso(x + 14, 11), iso(x, 11)]);
    c.drawPath(fr, fl(k.isEven ? kCyan : kLime));
    c.drawPath(fr, st(kInk, 1.4));
    final mid = iso(x + 7, 0);
    c.drawCircle(mid - const Offset(0, 0), 2.6, fl(kInk));
  }
  for (final x in [0.0, 120.0]) {
    final g = iso(x, 0) + const Offset(0, 6);
    c.drawOval(Rect.fromCenter(center: g, width: 22, height: 12), fl(spc(v)));
    c.drawLine(g, g + Offset(math.cos(p.ph * 3) * 9, math.sin(p.ph * 3) * 5), st(kInk, 2));
  }
}

void _galaxy(Canvas c, Size s, PopP p) {
  bg(c, s, const Color(0xFF120C24));
  final v = sp(p), ctr = s.center(Offset.zero);
  for (var arm = 0; arm < 3; arm++) {
    for (var i = 0; i < 26; i++) {
      final r = 4 + i * 2.1, a = arm * math.pi * 2 / 3 + i * .23 + p.ph * .9;
      final col = i < 8 ? kWhite : (arm == 0 ? kPink : (arm == 1 ? kViolet : kCyan));
      c.drawCircle(pol(ctr, r, a, .7), 2.4 - i * .06, fl(al(col, 1 - i / 34)));
    }
  }
  c.drawCircle(ctr, 6, fl(v == 0 ? kYellow : kWhite));
  for (var k = 0; k < 18; k++) {
    c.drawCircle(Offset((k * 47 % 150) + 3, (k * 29 % 112) + 4), .9, fl(al(kCream, .6)));
  }
  final dir = v < 0 ? -1 : 1;
  if (v != 0) {
    final tip = pol(ctr, 50, -math.pi / 2 + dir * .5, .7);
    c.drawArc(Rect.fromCenter(center: ctr, width: 100, height: 70), -math.pi / 2, dir * .5, false, st(spc(v), 2.5));
    c.drawCircle(tip, 3.5, fl(spc(v)));
  }
}

void _freeze(Canvas c, Size s, PopP p) {
  bg(c, s, kBlue);
  final v = sp(p), gy = s.height - 14, x = s.width / 2;
  c.drawRect(Rect.fromLTWH(0, gy, s.width, 14), fl(kInk));
  final h = v == 0 ? 34.0 : (math.sin(p.t * (2 + v.abs() * 2.2)).abs()) * (50 - v.abs() * 4);
  final y = gy - 12 - h;
  if (v == 0) {
    final r = Rect.fromCenter(center: Offset(x, y), width: 46, height: 46);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(6)), fl(al(kCyan, .55)));
  }
  c.drawCircle(Offset(x, y), 12, fl(v < 0 ? kPink : kOrange));
  c.drawCircle(Offset(x - 4, y - 4), 3, fl(al(kWhite, .7)));
  if (v == 0) {
    final r = Rect.fromCenter(center: Offset(x, y), width: 46, height: 46);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(6)), st(kWhite, 2));
    c.drawLine(r.topLeft + const Offset(6, 12), r.topLeft + const Offset(12, 6), st(kWhite, 2));
    for (var k = 0; k < 5; k++) {
      final a = k * 1.256 + .3;
      final o = pol(Offset(x, y), 34, a);
      c.drawLine(o - const Offset(4, 0), o + const Offset(4, 0), st(kWhite, 1.5));
      c.drawLine(o - const Offset(0, 4), o + const Offset(0, 4), st(kWhite, 1.5));
    }
  } else {
    c.drawOval(Rect.fromCenter(center: Offset(x, gy - 2), width: 26 - h * .2, height: 5), fl(al(kInk, .5)));
  }
}

void _rubber(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final v = sp(p);
  final k = v == 0 ? 2.6 : (1 / (.4 + v.abs() * .5)).clamp(.55, 2.2);
  final w = (s.width - 40) * k / 2.2, cx = s.width / 2, cy = s.height / 2;
  final r = Rect.fromCenter(center: Offset(cx, cy), width: math.min(w + 20, s.width - 6), height: 64 / math.sqrt(k));
  c.save();
  c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(8)));
  vgrad(c, r, kCyan, kWhite);
  c.save();
  c.translate(cx, r.bottom);
  c.scale(r.width / 100, 1);
  c.drawPath(poly([const Offset(-50, 0), const Offset(-20, -30), const Offset(-4, -16), const Offset(18, -40), const Offset(50, 0)]), fl(kLime));
  c.drawCircle(Offset(v < 0 ? -26 : 26, -46), 9, fl(v < 0 ? kPink : kOrange));
  c.restore();
  c.restore();
  c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(8)), st(kInk, 2.4));
  if (k > 1.2) {
    for (var i = 0; i < 4; i++) {
      final x = r.left + r.width * (.2 + i * .2);
      c.drawLine(Offset(x - 2, r.top + 4), Offset(x + 2, r.top + 12), st(al(kInk, .35), 1.2));
    }
  }
  for (final side in [-1.0, 1.0]) {
    final e = Offset(side < 0 ? r.left : r.right, cy);
    c.drawCircle(e, 6, fl(v == 0 ? kYellow : kInk));
  }
}

void _chevrons(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final v = sp(p), col = spc(v);
  if (v == 0) {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: s.center(const Offset(-12, 0)), width: 14, height: 48), const Radius.circular(2)), fl(kYellow));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: s.center(const Offset(12, 0)), width: 14, height: 48), const Radius.circular(2)), fl(kYellow));
    return;
  }
  final n = (v.abs() * 1.4).ceil().clamp(1, 4), d = v < 0 ? -1.0 : 1.0;
  for (var row = 0; row < 3; row++) {
    final y = 30.0 + row * 30;
    for (var k = 0; k < 6; k++) {
      final x0 = frac(k / 6 + d * p.ph * .1 + row * .13) * (s.width + 30) - 15;
      for (var j = 0; j < n; j++) {
        final x = x0 + d * j * 6;
        c.drawPath(poly([Offset(x - d * 5, y - 8), Offset(x + d * 3, y), Offset(x - d * 5, y + 8)], close: false), st(al(col, row == 1 ? 1 : .55), 3));
      }
    }
  }
}

void _sunrace(Canvas c, Size s, PopP p) {
  final v = sp(p);
  vgrad(c, Offset.zero & s, v < 0 ? const Color(0xFF6B2A6E) : kBlue, v < 0 ? kPink : kCyan);
  Offset sun(double u) => Offset(14 + u * (s.width - 28), s.height * .78 - math.sin(u * math.pi) * 70);
  final u = frac(p.ph * .06);
  if (v != 0) {
    for (var i = 1; i < 6; i++) {
      final g = (u - v.sign * i * .025 * v.abs()).clamp(0.0, 1.0);
      c.drawCircle(sun(g), 9, fl(al(kYellow, .38 - i * .06)));
    }
  } else {
    for (var k = 0; k < 8; k++) {
      final a = k * math.pi / 4;
      c.drawLine(pol(sun(u), 15, a), pol(sun(u), 22, a), st(kYellow, 2.5));
    }
  }
  c.drawCircle(sun(u), 11, fl(kYellow));
  final hills = Path()
    ..moveTo(0, s.height)
    ..lineTo(0, 92)
    ..quadraticBezierTo(40, 70, 78, 92)
    ..quadraticBezierTo(116, 74, s.width, 90)
    ..lineTo(s.width, s.height)
    ..close();
  c.drawPath(hills, fl(kLime));
  c.drawPath(Path()..addOval(Rect.fromLTWH(100, 96, 70, 40)), fl(const Color(0xFF6FBF4A)));
}

void _soda(Canvas c, Size s, PopP p) {
  bg(c, s, kCream);
  final v = sp(p);
  final glass = poly([const Offset(46, 12), const Offset(110, 12), const Offset(102, 110), const Offset(54, 110)]);
  c.save();
  c.clipPath(glass);
  c.drawRect(const Rect.fromLTRB(0, 30, 160, 120), fl(kOrange));
  for (var k = 0; k < 14; k++) {
    final x = 56 + (k * 37 % 44).toDouble();
    final u = v == 0 ? (k * .37 % 1) : frac(k * .19 + p.ph * .25);
    final y = v < 0 ? 34 + u * 74 : 108 - u * 74;
    final r = 1.6 + (k % 3) * 1.1 + (v > 0 ? u * 1.4 : 0);
    c.drawCircle(Offset(x, y), r, v == 0 ? fl(kYellow) : st(kWhite, 1.6));
  }
  c.restore();
  c.drawPath(glass, st(kInk, 2.6));
  c.drawLine(const Offset(90, 2), const Offset(84, 60), st(v < 0 ? kPink : kRed, 5));
}

void _blob(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final v = sp(p), still = v == 0;
  final bounce = still ? 0.0 : math.sin(p.t * (2 + v.abs() * 3)).abs() * (6 + v.abs() * 3);
  final ctr = Offset(s.width / 2, s.height * .62 - bounce);
  final body = still ? const Color(0xFF8E8A84) : (v < 0 ? kPink : (v > 1.6 ? kOrange : kLime));
  c.drawOval(Rect.fromCenter(center: Offset(s.width / 2, s.height * .9), width: 50 - bounce, height: 6), fl(al(kInk, .5)));
  c.drawOval(Rect.fromCenter(center: ctr, width: 64 + (still ? 4 : 0), height: 54 - (still ? 4 : 0)), fl(body));
  final look = v < 0 ? -6.0 : (still ? 0.0 : 5.0);
  for (final ex in [-12.0, 12.0]) {
    final e = ctr + Offset(ex, -6);
    c.drawCircle(e, 8, fl(kWhite));
    final open = still ? 0.0 : (v.abs() < 1.1 ? .45 : 1.0);
    c.drawCircle(e + Offset(look * .5, 0), 3.8, fl(kInk));
    if (open < 1) c.drawRect(Rect.fromLTRB(e.dx - 9, e.dy - 9, e.dx + 9, e.dy - 9 + 18 * (1 - open)), fl(body));
    c.drawLine(Offset(e.dx - 8, e.dy - 9 + 18 * (1 - open)), Offset(e.dx + 8, e.dy - 9 + 18 * (1 - open)), st(kInk, 1.6));
  }
  if (v > 1.6) {
    c.drawPath(poly([ctr + const Offset(28, -26), ctr + const Offset(33, -16), ctr + const Offset(23, -16)]), fl(kCyan));
    for (var k = 0; k < 3; k++) {
      c.drawLine(ctr + Offset(-40, -8 + k * 9.0), ctr + Offset(-56, -8 + k * 9.0), st(kCream, 2));
    }
  }
  if (still) {
    c.drawRect(Rect.fromCenter(center: ctr + const Offset(0, 36), width: 76, height: 8), fl(kYellow));
  }
}

void _cassette(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final v = sp(p);
  final body = RRect.fromRectAndRadius(Rect.fromLTWH(14, 18, s.width - 28, 82), const Radius.circular(8));
  c.drawRRect(body.shift(const Offset(4, 5)), fl(Colors.black));
  c.drawRRect(body, fl(kViolet));
  c.drawRect(Rect.fromLTWH(26, 28, s.width - 52, 14), fl(kCream));
  final win = RRect.fromRectAndRadius(Rect.fromLTWH(30, 48, s.width - 60, 34), const Radius.circular(17));
  c.drawRRect(win, fl(kInk));
  final u = .5 + .45 * math.sin(p.ph * .3);
  final l = Offset(win.left + 18, win.center.dy), r = Offset(win.right - 18, win.center.dy);
  c.drawCircle(l, 7 + 9 * (1 - u), fl(const Color(0xFF6B4A2A)));
  c.drawCircle(r, 7 + 9 * u, fl(const Color(0xFF6B4A2A)));
  for (final (o, sgn) in [(l, 1.0), (r, 1.0)]) {
    c.drawCircle(o, 7, fl(kWhite));
    for (var k = 0; k < 3; k++) {
      final a = p.ph * 3 * sgn + k * math.pi * 2 / 3;
      c.drawLine(pol(o, 2, a), pol(o, 6, a), st(spc(v), 2));
    }
  }
  c.drawPath(poly([Offset(48, 100), Offset(56, 90), Offset(s.width - 56, 90), Offset(s.width - 48, 100)]), fl(const Color(0xFF7A5FE0)));
}

void _cards(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final v = sp(p);
  final gap = v == 0 ? 4.0 : 9 + v.abs() * 2.5;
  const n = 6;
  for (var i = n - 1; i >= 0; i--) {
    final o = Offset(24 + i * gap, 30 + i * gap * .55);
    final r = Rect.fromLTWH(o.dx, o.dy, 64, 46);
    c.drawRRect(RRect.fromRectAndRadius(r.shift(const Offset(2, 3)), const Radius.circular(4)), fl(al(Colors.black, .4)));
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), fl(kCream));
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), st(kInk, 1.4));
    final step = v == 0 ? .5 : frac(p.ph * .15 - i * .12 * v.abs().clamp(.3, 3) * v.sign);
    final bp = Offset(r.left + 10 + step * 44, r.bottom - 8 - math.sin(step * math.pi) * 26);
    c.drawCircle(bp, 6, fl(v == 0 ? kYellow : (v < 0 ? kPink : kOrange)));
    c.drawLine(Offset(r.left + 4, r.bottom - 3), Offset(r.right - 4, r.bottom - 3), st(kInk, 1.4));
  }
  if (v == 0) {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(18, 24, 98, 80), const Radius.circular(8)), st(kYellow, 2.5));
  }
}

void _rain(Canvas c, Size s, PopP p) {
  vgrad(c, Offset.zero & s, const Color(0xFF2E3A5C), kP0);
  final v = sp(p), len = 3 + v.abs() * 7;
  for (var k = 0; k < 22; k++) {
    final x = 8 + (k * 53 % 140).toDouble();
    final u = v == 0 ? (k * .41 % 1) : frac(k * .23 + p.ph * .5);
    final y = v < 0 ? s.height - u * (s.height - 30) : 30 + u * (s.height - 30);
    if (v == 0) {
      c.drawCircle(Offset(x, y), 2.4, fl(kYellow));
    } else {
      final d = v < 0 ? 1 : -1;
      c.drawLine(Offset(x, y), Offset(x - 1.5, y + d * len), st(v < 0 ? kPink : kCyan, 2));
    }
  }
  for (final (x, r) in [(40.0, 18.0), (66.0, 24.0), (96.0, 20.0), (120.0, 15.0)]) {
    c.drawCircle(Offset(x, 16), r, fl(kCream));
  }
}

void _warp(Canvas c, Size s, PopP p) {
  bg(c, s, Colors.black);
  final v = sp(p), ctr = s.center(Offset.zero);
  for (var k = 0; k < 9; k++) {
    final u = frac(k / 9 + p.ph * .12);
    final r = math.pow(u, 2.2).toDouble() * 110 + 2;
    final col = k.isEven ? kPink : kCyan;
    c.drawRect(Rect.fromCenter(center: ctr, width: r * 1.5, height: r), st(al(v == 0 ? kYellow : col, u * 1.4), 1 + u * 3));
  }
  for (var k = 0; k < 12; k++) {
    final a = k * math.pi / 6 + .2;
    c.drawLine(pol(ctr, 14, a), pol(ctr, 90, a), Paint()
      ..shader = ui.Gradient.linear(ctr, pol(ctr, 90, a), [al(kViolet, 0), al(kViolet, .6)])
      ..strokeWidth = 1);
  }
  c.drawCircle(ctr, 4, fl(spc(v)));
}
