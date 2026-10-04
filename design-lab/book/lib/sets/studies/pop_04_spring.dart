// Pop 04 sheet 2: Spring (stiffness / damping / mass) x20. The reason you touch a spring: "how snappy, how many bounces before it rests,
// how heavy it feels". Every panel shows something knocked loose that rings back to rest; grabbing holds it, letting go releases it.
// Values: [0] stiffness, [1] damping, [2] mass (drawn as the size or weight of the thing where it can be).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'pop_04_kit.dart';

double _x(PopV p) => spring(ring(p), p[0], p[1], p[2]);
double _m(PopV p) => .7 + .7 * p[2];

/// A coil from [a] to [b]: [n] turns, [w] wide; thicker wire for a stiffer spring.
void _coil(Canvas c, Offset a, Offset b, int n, double w, Paint paint) {
  final d = b - a, u = d / d.distance, nrm = Offset(-u.dy, u.dx);
  final pts = <Offset>[a];
  for (var i = 0; i < n * 2; i++) {
    pts.add(a + d * ((i + .5) / (n * 2)) + nrm * (i.isEven ? w : -w));
  }
  pts.add(b);
  c.drawPath(poly(pts, close: false), paint);
}

const _wood = Color(0xFFC98A4B);

final List<PopPanel> springPanels = [
  PopPanel('Diving board', 'sport · bold flat · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Rect.fromLTWH(0, s.height - 26, s.width, 26), pf(Pop.cyan));
    for (var x = 6.0; x < s.width; x += 22) {
      c.drawLine(Offset(x, s.height - 18), Offset(x + 10, s.height - 18), ps(Pop.white, 2));
    }
    c.drawRect(Rect.fromLTWH(0, 40, 18, s.height - 66), pf(Pop.panel2));
    final x = _x(p), m = _m(p), tip = Offset(s.width - 34, 46 + x * 20 * m);
    final board = Path()
      ..moveTo(14, 44)
      ..quadraticBezierTo(70, 44, tip.dx, tip.dy);
    c.drawPath(board, ps(Pop.yellow, 3 + 4 * p[0]));
    final r = 9 * m;
    rr(c, Rect.fromCenter(center: tip + Offset(0, -15 * m), width: 13 * m, height: 28 * m), 5, pf(Pop.pink));
    c.drawCircle(tip + Offset(0, -29 * m - r), r, pf(Pop.orange));
    c.drawLine(tip + Offset(-6 * m, -26 * m), tip + Offset(-15 * m, -40 * m), ps(Pop.pink, 4));
    c.drawLine(tip + Offset(6 * m, -26 * m), tip + Offset(15 * m, -40 * m), ps(Pop.pink, 4));
  }),
  PopPanel('Jack in the box', 'toy · bold flat · 2D · drag', G.drag, (c, s, p) {
    final x = _x(p), m = _m(p), base = Offset(s.width / 2, s.height - 34);
    final head = base + Offset(math.sin(p.t * 3) * x * 8, -40 + x * 28);
    _coil(c, base, head + const Offset(0, 10), 6, 9, ps(Pop.cream, 1.5 + 3 * p[0]));
    final r = 12 * m;
    c.drawCircle(head, r, pf(Pop.white));
    c.drawCircle(head + Offset(0, r * .2), r * .3, pf(Pop.red));
    c.drawCircle(head + Offset(-r * .4, -r * .3), 2, pf(Pop.ink));
    c.drawCircle(head + Offset(r * .4, -r * .3), 2, pf(Pop.ink));
    c.drawPath(poly([head + Offset(-r, -r * .6), head + Offset(0, -r * 2.2), head + Offset(r, -r * .6)]), pf(Pop.violet));
    rr(c, Rect.fromLTWH(base.dx - 30, base.dy - 2, 60, 36), 3, pf(Pop.blue));
    c.drawPath(poly([Offset(base.dx - 30, base.dy - 2), Offset(base.dx - 48, base.dy - 22), Offset(base.dx - 30, base.dy - 12)]), pf(const Color(0xFF2448CC)));
    c.drawCircle(Offset(base.dx, base.dy + 16), 6, pf(Pop.yellow));
  }),
  PopPanel('Bobblehead', 'character · bold flat · 2D · flick', G.flick, (c, s, p) {
    final x = _x(p), m = _m(p), neck = Offset(s.width / 2, s.height - 44);
    rr(c, Rect.fromLTWH(neck.dx - 20, neck.dy + 4, 40, 40), 10, pf(Pop.blue));
    c.drawLine(neck, neck + const Offset(0, 8), ps(Pop.cream, 3));
    c.save();
    c.translate(neck.dx, neck.dy);
    c.rotate(x * .7);
    final r = 22 * m;
    c.drawCircle(Offset(0, -r), r, pf(Pop.yellow));
    c.drawCircle(Offset(-r * .35, -r * 1.1), r * .14, pf(Pop.ink));
    c.drawCircle(Offset(r * .35, -r * 1.1), r * .14, pf(Pop.ink));
    c.drawArc(Rect.fromCenter(center: Offset(0, -r * .75), width: r * .7, height: r * .45), .2, math.pi - .4, false, ps(Pop.ink, 2.5));
    c.drawArc(Rect.fromCenter(center: Offset(0, -r * 1.55), width: r * 1.9, height: r * 1.2), math.pi, math.pi, true, pf(Pop.orange));
    c.restore();
  }),
  PopPanel('Doorstop twang', 'object · crisp · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Rect.fromLTWH(0, 0, 18, s.height), pf(Pop.panel2));
    final x = _x(p), base = Offset(18, s.height / 2);
    for (var k = 3; k >= 0; k--) {
      final xx = spring(ring(p) - k * .025, p[0], p[1], p[2]);
      final tip = base + Offset(math.cos(xx * .9) * 100, math.sin(xx * .9) * 100);
      final bend = ol(base, tip, .5) + Offset(0, xx * 12);
      final path = Path()
        ..moveTo(base.dx, base.dy)
        ..quadraticBezierTo(bend.dx, bend.dy, tip.dx, tip.dy);
      c.drawPath(path, ps(pa(k == 0 ? Pop.cream : Pop.cyan, k == 0 ? 1 : .25), k == 0 ? 2 + 3 * p[0] : 2));
      if (k == 0) c.drawCircle(tip, 7 * _m(p), pf(Pop.red));
    }
    if (x.abs() < .01) c.drawCircle(base + const Offset(100, 0), 11, ps(pa(Pop.cream, .2), 1));
  }),
  PopPanel('Bungee', 'landscape · bold flat · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF8FD3FF)));
    c.drawPath(poly([Offset(0, s.height), const Offset(0, 40), const Offset(30, 34), Offset(40, s.height)]), pf(Pop.violet));
    c.drawPath(poly([Offset(s.width, s.height), Offset(s.width, 40), Offset(s.width - 34, 30), Offset(s.width - 44, s.height)]), pf(Pop.violet));
    c.drawRect(Rect.fromLTWH(0, 18, s.width, 7), pf(Pop.red));
    c.drawRect(Rect.fromLTWH(0, s.height - 14, s.width, 14), pf(Pop.blue));
    final x = _x(p), m = _m(p), top = Offset(s.width / 2, 25);
    final y = 48 - x * 22;
    final body = Offset(s.width / 2, y);
    _coil(c, top, body, 7, 2 + 4 * (1 - p[0]), ps(Pop.yellow, 2));
    rr(c, Rect.fromCenter(center: body + Offset(0, 10 * m), width: 14 * m, height: 22 * m), 5, pf(Pop.lime));
    c.drawCircle(body + Offset(0, 28 * m), 9 * m, pf(Pop.orange));
    c.drawLine(body + Offset(-6 * m, 14 * m), body + Offset(-16 * m, 26 * m), ps(Pop.lime, 4));
    c.drawLine(body + Offset(6 * m, 14 * m), body + Offset(16 * m, 26 * m), ps(Pop.lime, 4));
  }),
  PopPanel('Bubble pop-in', 'UI · crisp · 2D · pinch', G.pinch, (c, s, p) {
    final sc = 1 - _x(p);
    final ctr = Offset(s.width / 2, s.height / 2 - 4);
    c.save();
    c.translate(ctr.dx - 30, ctr.dy + 24);
    c.scale(math.max(0, sc));
    c.translate(-(ctr.dx - 30), -(ctr.dy + 24));
    rr(c, Rect.fromCenter(center: ctr, width: 76 * (.8 + .3 * p[2]), height: 44), 16, pf(Pop.lime));
    c.drawPath(poly([ctr + const Offset(-36, 16), ctr + const Offset(-30, 34), ctr + const Offset(-16, 20)]), pf(Pop.lime));
    for (var k = -1; k <= 1; k++) {
      c.drawCircle(ctr + Offset(k * 16.0, 0), 5, pf(Pop.ink));
    }
    c.restore();
  }),
  PopPanel('Pothole car', 'machine · bold flat · 2D · drag', G.drag, (c, s, p) {
    final ground = s.height - 24;
    c.drawRect(Rect.fromLTWH(0, ground, s.width, 24), pf(Pop.ink));
    c.drawLine(Offset(0, ground), Offset(s.width, ground), ps(Pop.yellow, 2));
    final x = _x(p), m = _m(p), cx = s.width / 2;
    final bodyY = ground - 30 - x * 14;
    for (final wx in [cx - 30, cx + 30]) {
      _coil(c, Offset(wx, ground - 12), Offset(wx, bodyY + 12), 3, 5, ps(Pop.cream, 1.5 + 2.5 * p[0]));
    }
    rr(c, Rect.fromLTWH(cx - 48, bodyY - 4 * m, 96, 18 + 4 * m), 8, pf(Pop.cyan));
    rr(c, Rect.fromLTWH(cx - 26, bodyY - 20 * m, 50, 20 * m), 8, pf(Pop.cyan));
    rr(c, Rect.fromLTWH(cx - 20, bodyY - 16 * m, 18, 13 * m), 3, pf(Pop.white));
    rr(c, Rect.fromLTWH(cx + 2, bodyY - 16 * m, 17, 13 * m), 3, pf(Pop.white));
    for (final wx in [cx - 30, cx + 30]) {
      c.drawCircle(Offset(wx, ground - 10), 11, pf(Pop.panel2));
      c.drawCircle(Offset(wx, ground - 10), 4, pf(Pop.cream));
    }
  }, light: true),
  PopPanel('Fishing bobber', 'nature · bold flat · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF13506B)));
    final x = _x(p), m = _m(p), surf = s.height / 2 + 6, ctr = Offset(s.width / 2, surf + x * 16);
    final age = cl(ring(p) / 3);
    for (var k = 0; k < 3; k++) {
      final r = 14 + (age * 3 + k * .3) * 26;
      c.drawOval(Rect.fromCenter(center: Offset(s.width / 2, surf), width: r * 2, height: r * .5), ps(pa(Pop.cyan, (1 - age) * .8), 2));
    }
    c.drawLine(Offset(ctr.dx, ctr.dy - 14 * m), Offset(s.width - 10, 0), ps(pa(Pop.cream, .6), 1));
    c.drawCircle(ctr, 9 * m, pf(Pop.red));
    c.save();
    c.clipRect(Rect.fromLTWH(0, ctr.dy, s.width, 30));
    c.drawCircle(ctr, 9 * m, pf(Pop.white));
    c.restore();
    c.drawRect(Rect.fromLTWH(0, surf, s.width, s.height - surf), pf(pa(Pop.blue, .55)));
    c.drawRect(Rect.fromLTWH(ctr.dx - 1.5, ctr.dy - 14 * m - 6, 3, 8), pf(Pop.yellow));
  }),
  PopPanel('Plucked string', 'instrument · neon · 2D · rub', G.rub, (c, s, p) {
    final x = _x(p), y = s.height / 2, l = 12.0, r = s.width - 12;
    c.drawCircle(Offset(l, y), 5, pf(Pop.violet));
    c.drawCircle(Offset(r, y), 5, pf(Pop.violet));
    for (var k = 3; k >= 0; k--) {
      final xx = spring(ring(p) - k * .012, p[0], p[1], p[2]) * (1 + .5 * p[2]);
      final path = Path()..moveTo(l, y);
      for (var i = 1; i <= 30; i++) {
        final u = i / 30;
        path.lineTo(lr(l, r, u), y + math.sin(u * math.pi) * xx * 38);
      }
      c.drawPath(path, ps(k == 0 ? Pop.pink : pa(Pop.pink, .2), k == 0 ? 2 + 2 * p[0] : 4));
    }
    if (x.abs() > .3) c.drawCircle(Offset(s.width / 2, y), 3, pf(Pop.white));
  }),
  PopPanel('Hanging crate', 'weight · bold · isometric · drag', G.drag, (c, s, p) {
    final x = _x(p), m = _m(p), top = Offset(s.width / 2 + 6, 6);
    c.drawRect(Rect.fromLTWH(top.dx - 24, 0, 48, 6), pf(Pop.panel2));
    final bot = top + Offset(0, 36 + x * 20);
    _coil(c, top, bot, 7, 7, ps(Pop.cyan, 1.5 + 3 * p[0]));
    final w = 34 * m;
    final dx = Offset(w * .87, w * .5) * .7, dy = Offset(-w * .87, w * .5) * .7, up = Offset(0, -w * .9);
    final b = bot;
    c.drawPath(poly([b, b + dx, b + dx + dy, b + dy]), pf(Pop.yellow));
    c.drawPath(poly([b + dy, b + dy + dx, b + dy + dx - up, b + dy - up]), pf(Pop.orange));
    c.drawPath(poly([b + dx, b + dx - up, b + dx + dy - up, b + dx + dy]), pf(Pop.red));
    c.drawOval(Rect.fromCenter(center: Offset(top.dx, s.height - 8), width: 50 * (1.2 - x * .3), height: 8), pf(pa(Pop.ink, .6)));
  }),
  PopPanel('Slingshot', 'toy · bold flat · 2D · throw', G.flick, (c, s, p) {
    final x = _x(p), a = Offset(32, 26), b = Offset(s.width - 32, 26);
    c.drawPath(poly([Offset(s.width / 2 - 6, s.height), Offset(s.width / 2 - 6, 70), a + const Offset(-4, 0), a + const Offset(4, 0), Offset(s.width / 2, 62), b + const Offset(-4, 0), b + const Offset(4, 0), Offset(s.width / 2 + 6, 70), Offset(s.width / 2 + 6, s.height)]), pf(_wood));
    final mid = Offset(s.width / 2, 26 + x * 52 * _m(p));
    c.drawPath(poly([a, mid, b], close: false), ps(Pop.red, 2 + 3 * p[0]));
    rr(c, Rect.fromCenter(center: mid, width: 18, height: 9), 3, pf(Pop.ink));
    c.drawCircle(mid - Offset(0, 9 * _m(p)), 9 * _m(p), pf(Pop.lime));
  }, light: true),
  PopPanel('Bird on a branch', 'nature · bold flat · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFFFFE7B8)));
    final x = _x(p), m = _m(p), tip = Offset(s.width - 18, 58 + x * 26 * m);
    final branch = Path()
      ..moveTo(-4, 52)
      ..quadraticBezierTo(70, 52, tip.dx, tip.dy);
    c.drawPath(branch, ps(const Color(0xFF6B3E1E), 5 + 3 * p[0]));
    for (final (u, dy) in [(.35, -1.0), (.7, 1.0)]) {
      final q = Offset(lr(0, tip.dx, u), lr(52, tip.dy, u * u));
      c.drawOval(Rect.fromCenter(center: q + Offset(4, dy * 8), width: 18, height: 9), pf(Pop.lime));
    }
    final bird = Offset(lr(0, tip.dx, .82), lr(52, tip.dy, .67)) - Offset(0, 11 * m);
    c.drawOval(Rect.fromCenter(center: bird, width: 26 * m, height: 20 * m), pf(Pop.blue));
    c.drawCircle(bird + Offset(9 * m, -8 * m), 7 * m, pf(Pop.blue));
    c.drawCircle(bird + Offset(11 * m, -9 * m), 1.8, pf(Pop.white));
    c.drawPath(poly([bird + Offset(15 * m, -9 * m), bird + Offset(22 * m, -7 * m), bird + Offset(15 * m, -5 * m)]), pf(Pop.orange));
  }, light: true),
  PopPanel('UFO landing', 'cosmic · neon · 2D · drag', G.drag, (c, s, p) {
    c.drawRect(Rect.fromLTWH(0, s.height - 18, s.width, 18), pf(const Color(0xFF3B2C5E)));
    for (var i = 0; i < 18; i++) {
      c.drawCircle(Offset((i * 37.0) % s.width, (i * 23.0) % (s.height - 30)), .9, pf(Pop.cream));
    }
    final x = _x(p), m = _m(p), ground = s.height - 18, body = Offset(s.width / 2, ground - 34 + x * 16);
    for (final dx in [-26.0, 26.0]) {
      _coil(c, body + Offset(dx * .6, 4), Offset(body.dx + dx, ground - 2), 3, 3, ps(Pop.cream, 1.5 + 2 * p[0]));
      c.drawLine(Offset(body.dx + dx - 5, ground - 1), Offset(body.dx + dx + 5, ground - 1), ps(Pop.cream, 3));
    }
    c.drawPath(poly([body + const Offset(-10, 6), body + const Offset(10, 6), Offset(body.dx + 26, ground), Offset(body.dx - 26, ground)]), pf(pa(Pop.lime, .18)));
    c.drawOval(Rect.fromCenter(center: body + Offset(0, -10 * m), width: 30 * m, height: 26 * m), pf(Pop.cyan));
    c.drawOval(Rect.fromCenter(center: body, width: 70 * m, height: 18 * m), pf(Pop.violet));
    for (var k = -2; k <= 2; k++) {
      c.drawCircle(body + Offset(k * 12.0 * m, 1), 2.5, pf(k.isEven ? Pop.yellow : Pop.pink));
    }
  }),
  PopPanel('Pixel boing', '8-bit · bold flat · 2D · flick', G.flick, (c, s, p) {
    const px = 8.0;
    for (var y = 0.0; y < s.height; y += px) {
      for (var xx = 0.0; xx < s.width; xx += px) {
        if (((xx + y) / px).round().isEven) c.drawRect(Rect.fromLTWH(xx, y, px, px), pf(const Color(0xFF232327)));
      }
    }
    final floor = s.height - 2 * px;
    c.drawRect(Rect.fromLTWH(0, floor, s.width, 2 * px), pf(Pop.lime));
    final x = _x(p), m = _m(p);
    final h = ((-x).clamp(-1.0, 1.0) * 6 + 2).round().clamp(0, 6);
    final squash = x > 0 ? 0 : (-x * 2).round().clamp(0, 2);
    final w = (4 * m).round() + squash, hh = (4 * m).round() - squash + 1;
    final left = (s.width / 2 / px).floor() - (w / 2).floor();
    final top = floor / px - hh - (6 - h);
    for (var i = 0; i < w; i++) {
      for (var j = 0; j < hh; j++) {
        c.drawRect(Rect.fromLTWH((left + i) * px, (top + j - (x > 0 ? x * 7 : 0).round()) * px, px - 1, px - 1), pf(j == 0 ? Pop.yellow : Pop.orange));
      }
    }
  }),
  PopPanel('Pinball plunger', 'machine · neon · 2D · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF1A1033)));
    final x = _x(p), lane = Rect.fromLTWH(s.width - 44, 6, 32, s.height - 12);
    rr(c, lane, 14, ps(Pop.pink, 3));
    final py = s.height - 30 + x * 18;
    _coil(c, Offset(lane.center.dx, py + 6), Offset(lane.center.dx, s.height - 8), 5, 8, ps(Pop.cream, 1.5 + 3 * p[0]));
    rr(c, Rect.fromCenter(center: Offset(lane.center.dx, py), width: 22, height: 10), 3, pf(Pop.red));
    c.drawCircle(Offset(lane.center.dx, py - 14 - (x < 0 ? -x * 70 : 0)), 9 * _m(p), pf(Pop.cream));
    for (final (o, col) in [(const Offset(34, 34), Pop.yellow), (const Offset(70, 70), Pop.cyan), (const Offset(28, 86), Pop.lime)]) {
      c.drawCircle(o, 12, pf(col));
      c.drawCircle(o, 6, pf(const Color(0xFF1A1033)));
    }
  }),
  PopPanel('Cat on a cushion', 'creature · bold flat · 2D · drag', G.drag, (c, s, p) {
    final x = _x(p), m = _m(p), dip = x * 10 * m;
    final cy = s.height - 34;
    final cushion = Path()
      ..moveTo(10, cy)
      ..quadraticBezierTo(s.width / 2, cy + dip * 1.6 - 6, s.width - 10, cy)
      ..lineTo(s.width - 14, s.height - 10)
      ..quadraticBezierTo(s.width / 2, s.height - 4, 14, s.height - 10)
      ..close();
    c.drawPath(cushion, pf(Pop.pink));
    c.drawCircle(Offset(s.width / 2, cy + 10 + dip * .6), 3, pf(pa(Pop.ink, .4)));
    final body = Offset(s.width / 2, cy - 10 + dip);
    c.drawOval(Rect.fromCenter(center: body, width: 50 * m, height: 26 * m), pf(Pop.ink));
    final head = body + Offset(22 * m, -12 * m);
    c.drawCircle(head, 11 * m, pf(Pop.ink));
    c.drawPath(poly([head + Offset(-9 * m, -5 * m), head + Offset(-6 * m, -17 * m), head + Offset(-1 * m, -9 * m)]), pf(Pop.ink));
    c.drawPath(poly([head + Offset(1 * m, -9 * m), head + Offset(7 * m, -17 * m), head + Offset(9 * m, -5 * m)]), pf(Pop.ink));
    c.drawCircle(head + Offset(-3 * m, -1), 2, pf(Pop.yellow));
    c.drawCircle(head + Offset(4 * m, -1), 2, pf(Pop.yellow));
    final tail = Path()
      ..moveTo(body.dx - 22 * m, body.dy)
      ..quadraticBezierTo(body.dx - 40 * m, body.dy - 20 - x * 14, body.dx - 30 * m, body.dy - 30);
    c.drawPath(tail, ps(Pop.ink, 4));
  }, light: true),
  PopPanel('Rope bridge', 'landscape · bold flat · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF2B4D3A)));
    c.drawRect(Rect.fromLTWH(0, 50, 22, s.height), pf(Pop.violet));
    c.drawRect(Rect.fromLTWH(s.width - 22, 50, 22, s.height), pf(Pop.violet));
    final x = _x(p), m = _m(p), sag = 10 + x * 24 * m;
    Offset at(double u) => Offset(lr(22, s.width - 22, u), 50 + math.sin(u * math.pi) * sag);
    for (var i = 0; i <= 10; i++) {
      final q = at(i / 10);
      c.drawLine(q + const Offset(0, -2), q + const Offset(0, 4), ps(_wood, 6));
    }
    c.drawPath(poly([for (var i = 0; i <= 20; i++) at(i / 20) + const Offset(0, -18)], close: false), ps(Pop.cream, 1.5 + 2 * p[0]));
    final g = at(.5);
    c.drawCircle(g + Offset(0, -10 - 6 * m), 5 * m, pf(Pop.yellow));
    rr(c, Rect.fromCenter(center: g + Offset(0, -5 * m), width: 8 * m, height: 12 * m), 3, pf(Pop.orange));
    for (var i = 0; i < 5; i++) {
      c.drawLine(Offset(32.0 + i * 24, s.height), Offset(40.0 + i * 24, s.height - 14), ps(pa(Pop.cyan, .35), 2));
    }
  }),
  PopPanel('Robot antenna', 'character · crisp · 2D · flick', G.flick, (c, s, p) {
    final head = Rect.fromCenter(center: Offset(s.width / 2, s.height - 30), width: 68, height: 50);
    final base = head.topCenter;
    final x = _x(p);
    final tip = base + Offset(x * 40 * _m(p), -46 + x.abs() * 8);
    final ant = Path()
      ..moveTo(base.dx, base.dy)
      ..quadraticBezierTo(base.dx, base.dy - 26, tip.dx, tip.dy);
    c.drawPath(ant, ps(Pop.cream, 1.5 + 3 * p[0]));
    c.drawCircle(tip, 6 * _m(p), pf(Pop.red));
    c.drawCircle(tip + const Offset(-2, -2), 2, pf(Pop.white));
    rr(c, head, 12, pf(Pop.cyan));
    rr(c, head.deflate(8).translate(0, -2), 6, pf(Pop.ink));
    final look = x * 4;
    c.drawCircle(head.center + Offset(-12 + look, -4), 5, pf(Pop.lime));
    c.drawCircle(head.center + Offset(12 + look, -4), 5, pf(Pop.lime));
    c.drawLine(head.center + const Offset(-8, 9), head.center + const Offset(8, 9), ps(Pop.lime, 2));
  }),
  PopPanel('Lantern sway', 'weather · bold flat · 2D · blow', G.rub, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF14213D)));
    c.drawLine(const Offset(0, 14), Offset(s.width, 22), ps(Pop.cream, 1.5));
    final x = _x(p), m = _m(p);
    for (var k = 0; k < 3; k++) {
      final top = Offset(30.0 + k * 48, 15.5 + k * 3.0);
      final a = x * (.6 - k * .12) * (2 - p[2]);
      final ctr = top + Offset(math.sin(a), math.cos(a)) * (40 + k * 6.0);
      c.drawLine(top, ctr, ps(Pop.cream, 1));
      c.drawCircle(ctr + Offset(0, 4), 26 * m, pf(pa(Pop.yellow, .2)));
      c.drawOval(Rect.fromCenter(center: ctr + Offset(0, 6 * m), width: 22 * m, height: 26 * m), pf(k == 1 ? Pop.red : Pop.orange));
      c.drawRect(Rect.fromCenter(center: ctr + Offset(0, -6 * m), width: 12 * m, height: 4), pf(Pop.ink));
    }
    if (p.touch != null) {
      for (var k = 0; k < 3; k++) {
        c.drawLine(Offset(4, 60.0 + k * 12), Offset(26, 58.0 + k * 12), ps(pa(Pop.cyan, .7), 2));
      }
    }
  }),
  PopPanel('Woofer', 'speaker · neon · top-down · rub', G.rub, (c, s, p) {
    final x = _x(p), ctr = s.center(Offset.zero);
    for (var k = 0; k < 4; k++) {
      final age = (ring(p) * (1.5 + p[0] * 3) + k * .25) % 1;
      c.drawCircle(ctr, 44 + age * 40, ps(pa(Pop.lime, (1 - age) * x.abs() * 1.5), 2));
    }
    c.drawCircle(ctr, 46, pf(Pop.panel2));
    c.drawCircle(ctr, 40 + x * 4, pf(const Color(0xFF111114)));
    c.drawCircle(ctr, 30 + x * 6, ps(pa(Pop.lime, .8), 3));
    c.drawCircle(ctr, (14 + x * 8) * _m(p), pf(Pop.lime));
    c.drawCircle(ctr - const Offset(3, 3), (5 + x * 3) * _m(p), pf(pa(Pop.white, .6)));
    for (var k = 0; k < 4; k++) {
      c.drawCircle(polar(ctr, 52, k * math.pi / 2 + math.pi / 4), 3, pf(Pop.cream));
    }
  }),
];
