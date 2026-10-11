// Pop 04 sheet 3: Wiggle (frequency / amplitude / smoothness) x20. The reason you touch wiggle: "make it alive, nervous, handheld,
// buzzing, drifting". Every panel shows something that jitters; values: [0] frequency (how fast), [1] amplitude (how far), [2] smoothness
// (1 = one soft drift, 0 = jagged layered jitter).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'pop_04_kit.dart';

double _w(PopV p, double seed, [double dt = 0]) => wig(p.t - dt, p[0], p[2], seed) * p[1];
Offset _w2(PopV p, double seed, [double dt = 0]) => Offset(_w(p, seed, dt), _w(p, seed + 50, dt));

/// The same wiggle laid out along space instead of time ([u] in 0..1 across the panel).
double _ws(PopV p, double u, double seed) => wig(u * 3 + p.t * .3, p[0], p[2], seed) * p[1];

final List<PopPanel> wigglePanels = [
  PopPanel('Handheld cam', 'camera · crisp · 2D · rub', G.rub, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(Pop.cyan));
    c.drawPath(poly([Offset(0, s.height), Offset(0, 70), const Offset(50, 46), const Offset(96, 74), Offset(s.width, 54), Offset(s.width, s.height)]), pf(Pop.lime));
    c.drawCircle(const Offset(112, 26), 12, pf(Pop.yellow));
    final j = _w2(p, 1) * 16, rot = _w(p, 7) * .12;
    c.save();
    c.translate(s.width / 2 + j.dx, s.height / 2 + j.dy);
    c.rotate(rot);
    final r = Rect.fromCenter(center: Offset.zero, width: 92, height: 62);
    for (final (a, dx, dy) in [(r.topLeft, 1.0, 1.0), (r.topRight, -1.0, 1.0), (r.bottomLeft, 1.0, -1.0), (r.bottomRight, -1.0, -1.0)]) {
      c.drawPath(poly([a + Offset(dx * 14, 0), a, a + Offset(0, dy * 14)], close: false), ps(Pop.white, 3));
    }
    c.drawCircle(Offset(r.left + 10, r.top + 10), 3.5, pf(Pop.red));
    c.drawLine(const Offset(-6, 0), const Offset(6, 0), ps(Pop.white, 1.5));
    c.drawLine(const Offset(0, -6), const Offset(0, 6), ps(Pop.white, 1.5));
    c.restore();
  }),
  PopPanel('Firefly jar', 'nature · neon · 2D · rub', G.rub, (c, s, p) {
    final jar = Rect.fromLTWH(s.width / 2 - 40, 16, 80, s.height - 24);
    rr(c, jar, 16, pf(const Color(0xFF15251C)));
    rr(c, jar, 16, ps(pa(Pop.cream, .5), 2));
    rr(c, Rect.fromLTWH(jar.left + 10, 8, jar.width - 20, 10), 3, pf(Pop.orange));
    for (var k = 0; k < 9; k++) {
      final home = Offset(jar.left + 16 + (k * .618 % 1) * (jar.width - 32), jar.top + 16 + (k + .5) / 9 * (jar.height - 32));
      final pos = home + _w2(p, k * 3.3) * 30;
      for (var j = 3; j >= 1; j--) {
        c.drawCircle(home + _w2(p, k * 3.3, j * .05) * 28, 1.5, pf(pa(Pop.lime, .3)));
      }
      c.drawCircle(pos, 7, pf(pa(Pop.yellow, .22)));
      c.drawCircle(pos, 3, pf(Pop.yellow));
    }
  }),
  PopPanel('Jellyfish', 'creature · bold flat · 2D · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF241A4A)));
    final top = Offset(s.width / 2, 34 + _w(p, 9) * 6);
    for (var k = 0; k < 6; k++) {
      final x0 = top.dx - 25 + k * 10.0;
      final pts = <Offset>[];
      for (var i = 0; i <= 10; i++) {
        pts.add(Offset(x0 + _w(p, k * 1.7, i * .06) * 16 * (i / 10), top.dy + 6 + i * 6.5));
      }
      c.drawPath(poly(pts, close: false), ps(k.isEven ? Pop.pink : Pop.violet, 3));
    }
    c.drawArc(Rect.fromCenter(center: top + const Offset(0, 8), width: 66, height: 52), math.pi, math.pi, true, pf(Pop.pink));
    c.drawCircle(top + const Offset(-10, -4), 3, pf(Pop.ink));
    c.drawCircle(top + const Offset(10, -4), 3, pf(Pop.ink));
  }),
  PopPanel('Quake city', 'city · bold · isometric · rub', G.rub, (c, s, p) {
    c.drawPath(poly([Offset(s.width / 2, 56), Offset(s.width - 6, 84), Offset(s.width / 2, 112), const Offset(6, 84)]), pf(Pop.panel2));
    final blds = [(const Offset(52, 78), 40.0, Pop.cyan), (const Offset(96, 78), 30.0, Pop.pink), (const Offset(74, 92), 52.0, Pop.yellow)];
    for (var k = 0; k < 3; k++) {
      final (o, h, col) = blds[k];
      final j = _w2(p, k * 4.0) * 7;
      final b = o + Offset(j.dx, j.dy * .3);
      final dx = const Offset(14, 8), dy = const Offset(-14, 8), up = Offset(0, -h);
      c.drawPath(poly([b + up, b + up + dx, b + up + dx + dy, b + up + dy]), pf(Color.lerp(col, Pop.white, .35)!));
      c.drawPath(poly([b + dy, b + dy + up, b + dy + dx + up, b + dy + dx]), pf(col));
      c.drawPath(poly([b + dx, b + dx + up, b + dx + dy + up, b + dx + dy]), pf(Color.lerp(col, Pop.ink, .35)!));
      for (var f = 1; f < h ~/ 12; f++) {
        c.drawLine(b + dy + Offset(3, -f * 12.0 + 2), b + dy + dx + Offset(-3, -f * 12.0 - 2), ps(pa(Pop.ink, .4), 2));
      }
    }
  }),
  PopPanel('Candle flicker', 'fire · bold flat · 2D · blow', G.rub, (c, s, p) {
    final base = Offset(s.width / 2, s.height - 40);
    rr(c, Rect.fromLTWH(base.dx - 13, base.dy, 26, 40), 3, pf(Pop.pink));
    c.drawLine(base, base + const Offset(0, -6), ps(Pop.ink, 2));
    final lean = _w(p, 2) * 14, h = 40 + _w(p, 5) * 14;
    final tip = base + Offset(lean, -h);
    Path flame(double k) => Path()
      ..moveTo(base.dx, base.dy - 4)
      ..cubicTo(base.dx - 14 * k, base.dy - 12, tip.dx - 6 * k, tip.dy + h * .4, tip.dx, tip.dy + (1 - k) * h * .35)
      ..cubicTo(tip.dx + 6 * k, tip.dy + h * .4, base.dx + 14 * k, base.dy - 12, base.dx, base.dy - 4);
    c.drawPath(flame(1), pf(Pop.orange));
    c.drawPath(flame(.55), pf(Pop.yellow));
  }, light: true),
  PopPanel('Buzzing fly', 'creature · crisp · 2D · paint', G.paint, (c, s, p) {
    final ctr = Offset(s.width / 2, s.height / 2 + 14);
    c.drawCircle(ctr, 18, pf(Pop.red));
    c.drawPath(poly([ctr + const Offset(0, -18), ctr + const Offset(8, -28), ctr + const Offset(2, -16)]), pf(Pop.lime));
    final pts = [for (var k = 0; k < 40; k++) ctr + const Offset(0, -30) + _w2(p, 3, k * .03) * 90];
    c.drawPath(poly(pts, close: false), ps(pa(Pop.cream, .35), 1.2));
    final f = pts.first;
    c.drawOval(Rect.fromCenter(center: f + const Offset(-4, -4), width: 9, height: 6), pf(pa(Pop.cyan, .7)));
    c.drawOval(Rect.fromCenter(center: f + const Offset(4, -4), width: 9, height: 6), pf(pa(Pop.cyan, .7)));
    c.drawCircle(f, 4.5, pf(Pop.cream));
  }),
  PopPanel('Shivering blob', 'character · bold flat · 2D · rub', G.rub, (c, s, p) {
    final ctr = Offset(s.width / 2, s.height / 2 + 8);
    final pts = <Offset>[];
    for (var i = 0; i < 28; i++) {
      final a = i / 28 * math.pi * 2;
      pts.add(polar(ctr, 36 + _w(p, i * .9) * 8 + (math.sin(a) > 0 ? 4 : 0), a));
    }
    c.drawPath(poly(pts), pf(Pop.cyan));
    final j = _w2(p, 40) * 3;
    for (final dx in [-12.0, 12.0]) {
      c.drawCircle(ctr + Offset(dx, -8) + j, 7, pf(Pop.white));
      c.drawCircle(ctr + Offset(dx, -7) + j * 1.6, 3, pf(Pop.ink));
    }
    final mouth = [for (var i = 0; i <= 8; i++) ctr + Offset(-12 + i * 3.0, 12 + (i.isEven ? -2 : 2) * (.5 + p[1] * 2))];
    c.drawPath(poly(mouth, close: false), ps(Pop.ink, 2));
  }),
  PopPanel('Tree in wind', 'weather · bold flat · landscape · blow', G.rub, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFFBDE6FF)));
    c.drawRect(Rect.fromLTWH(0, s.height - 14, s.width, 14), pf(Pop.lime));
    final base = Offset(s.width / 2, s.height - 14), sway = _w(p, 4) * 14;
    final top = base + Offset(sway, -54);
    final trunk = Path()
      ..moveTo(base.dx - 5, base.dy)
      ..quadraticBezierTo(base.dx - 3 + sway * .3, base.dy - 30, top.dx - 2, top.dy)
      ..lineTo(top.dx + 2, top.dy)
      ..quadraticBezierTo(base.dx + 3 + sway * .3, base.dy - 30, base.dx + 5, base.dy)
      ..close();
    c.drawPath(trunk, pf(const Color(0xFF7A4A24)));
    for (var k = 0; k < 7; k++) {
      final o = top + Offset(math.cos(k * 1.9) * 24, math.sin(k * 1.9) * 14 - 6) + _w2(p, k * 2.1) * 7;
      c.drawCircle(o, 15, pf(k.isEven ? const Color(0xFF3FA34D) : const Color(0xFF6CC24A)));
    }
    for (var k = 0; k < 3; k++) {
      final y = 20.0 + k * 22, x = ((p.t * 80 * (.4 + p[0])) + k * 50) % (s.width + 40) - 20;
      c.drawLine(Offset(x, y), Offset(x + 18, y), ps(Pop.white, 2));
    }
  }),
  PopPanel('Line boil', 'sketch · crisp · 2D · drag', G.drag, (c, s, p) {
    final ctr = s.center(Offset.zero);
    final fr = (p.t * (2 + p[0] * 22)).floor();
    final pts = <Offset>[];
    for (var i = 0; i < 10; i++) {
      final a = -math.pi / 2 + i * math.pi / 5, r = i.isEven ? 44.0 : 20.0;
      pts.add(polar(ctr, r, a) + Offset(wig(fr * 1.0, .5, p[2], i * 1.0), wig(fr * 1.0, .5, p[2], i + 20.0)) * p[1] * 9);
    }
    c.drawPath(poly(pts), pf(Pop.yellow));
    c.drawPath(poly(pts), ps(Pop.ink, 3));
    final eye = Offset(wig(fr * 1.0, .5, p[2], 77) * p[1] * 3, 0);
    c.drawCircle(ctr + const Offset(-7, -2) + eye, 2.5, pf(Pop.ink));
    c.drawCircle(ctr + const Offset(7, -2) + eye, 2.5, pf(Pop.ink));
  }, light: true),
  PopPanel('Choppy sea', 'landscape · bold flat · 2D · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFFFFB86B)));
    c.drawCircle(Offset(s.width - 36, 36), 16, pf(Pop.yellow));
    double sea(double x) => s.height * .62 + _ws(p, x / s.width, 3) * 26;
    final path = Path()..moveTo(0, s.height);
    for (var x = 0.0; x <= s.width; x += 3) {
      path.lineTo(x, sea(x));
    }
    path
      ..lineTo(s.width, s.height)
      ..close();
    final bx = s.width * .42, by = sea(bx), ang = math.atan2(sea(bx + 6) - sea(bx - 6), 12);
    c.save();
    c.translate(bx, by);
    c.rotate(ang);
    c.drawPath(poly([const Offset(-20, -6), const Offset(20, -6), const Offset(13, 4), const Offset(-13, 4)]), pf(Pop.red));
    c.drawLine(const Offset(0, -6), const Offset(0, -34), ps(Pop.ink, 2));
    c.drawPath(poly([const Offset(2, -32), const Offset(18, -10), const Offset(2, -10)]), pf(Pop.white));
    c.restore();
    c.drawPath(path, pf(Pop.blue));
  }),
  PopPanel('Neon buzz', 'sign · neon · 2D · rub', G.rub, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF16101F)));
    final j = _w2(p, 6) * 5, on = _w(p, 13) > -p[1] * .8;
    final heart = Path()
      ..moveTo(s.width / 2, 92)
      ..cubicTo(s.width / 2 - 70, 46, s.width / 2 - 22, 4, s.width / 2, 34)
      ..cubicTo(s.width / 2 + 22, 4, s.width / 2 + 70, 46, s.width / 2, 92);
    c.save();
    c.translate(j.dx, j.dy);
    if (on) c.drawPath(heart, ps(pa(Pop.pink, .3), 12));
    c.drawPath(heart, ps(on ? Pop.pink : pa(Pop.pink, .25), 4));
    if (on) c.drawPath(heart, ps(pa(Pop.white, .8), 1.2));
    c.restore();
  }),
  PopPanel('Glitch slices', 'glitch · neon · 2D · rub', G.rub, (c, s, p) {
    const n = 9;
    final h = s.height / n;
    for (var k = 0; k < n; k++) {
      final off = _w(p, k * 2.7) * 40;
      c.save();
      c.clipRect(Rect.fromLTWH(0, k * h, s.width, h));
      c.translate(off, 0);
      c.drawCircle(Offset(s.width / 2, s.height / 2), 38, pf(Pop.orange));
      c.drawRect(Rect.fromLTWH(s.width / 2 - 8, 14, 16, s.height - 28), pf(Pop.blue));
      c.restore();
      if (off.abs() > 12) c.drawRect(Rect.fromLTWH(off > 0 ? 0 : s.width - 10, k * h, 10, h), pf(k.isEven ? Pop.cyan : Pop.pink));
    }
  }),
  PopPanel('Twinkle sky', 'cosmic · crisp · 2D · pinch', G.pinch, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF0E1440)));
    for (var k = 0; k < 14; k++) {
      final o = Offset((k * 47.0 + 10) % (s.width - 10) + 5, (k * 29.0 + 8) % (s.height - 10) + 5);
      final b = cl(.55 + _w(p, k * 1.3) * .9, .05, 1);
      final r = 2 + (k % 3) * 1.5 + b * 3;
      final col = pa(k % 4 == 0 ? Pop.cyan : Pop.yellow, b);
      c.drawPath(poly([for (var i = 0; i < 8; i++) polar(o, i.isEven ? r : r * .35, i * math.pi / 4)]), pf(col));
    }
  }),
  PopPanel('Bumblebee', 'creature · bold flat · top-down · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF9BE564)));
    for (final (o, col) in [(const Offset(26, 30), Pop.pink), (const Offset(124, 88), Pop.white), (const Offset(40, 94), Pop.violet)]) {
      for (var i = 0; i < 5; i++) {
        c.drawCircle(polar(o, 8, i * math.pi * 2 / 5), 6, pf(col));
      }
      c.drawCircle(o, 5, pf(Pop.yellow));
    }
    final path = Offset((p.t * 30) % (s.width + 40) - 20, s.height / 2);
    final pos = path + _w2(p, 2) * 40;
    final prev = path - const Offset(2, 0) + _w2(p, 2, .03) * 40;
    final a = (pos - prev).direction;
    for (var k = 1; k < 14; k++) {
      final q = Offset((p.t - k * .05) * 30 % (s.width + 40) - 20, s.height / 2) + _w2(p, 2, k * .05) * 40;
      c.drawCircle(q, 1.4, pf(pa(Pop.ink, .35)));
    }
    c.save();
    c.translate(pos.dx, pos.dy);
    c.rotate(a);
    c.drawOval(const Rect.fromLTWH(-4, -12, 8, 10), pf(pa(Pop.white, .8)));
    c.drawOval(const Rect.fromLTWH(-4, 2, 8, 10), pf(pa(Pop.white, .8)));
    c.drawOval(Rect.fromCenter(center: Offset.zero, width: 20, height: 12), pf(Pop.yellow));
    c.drawLine(const Offset(-3, -6), const Offset(-3, 6), ps(Pop.ink, 3));
    c.drawLine(const Offset(3, -6), const Offset(3, 6), ps(Pop.ink, 3));
    c.restore();
  }),
  PopPanel('Shaken soda', 'food · bold flat · 2D · rub', G.rub, (c, s, p) {
    final j = _w2(p, 8) * 9, rot = _w(p, 11) * .25;
    final ctr = Offset(s.width / 2, s.height / 2 + 6) + j;
    for (var k = 0; k < 8; k++) {
      final age = (p.t * (.5 + p[0]) + k / 8) % 1;
      final o = Offset(s.width / 2 + math.sin(k * 2.3) * 22, s.height / 2 - 20 - age * 40) + j * (1 - age);
      c.drawCircle(o, 2 + p[1] * 4 * (1 - age), ps(pa(Pop.cyan, 1 - age), 1.5));
    }
    c.save();
    c.translate(ctr.dx, ctr.dy);
    c.rotate(rot);
    rr(c, const Rect.fromLTWH(-22, -36, 44, 72), 10, pf(Pop.red));
    rr(c, const Rect.fromLTWH(-18, -42, 36, 8), 3, pf(const Color(0xFFC0C0C8)));
    c.drawPath(poly([const Offset(-22, -6), const Offset(22, -14), const Offset(22, 2), const Offset(-22, 10)]), pf(Pop.white));
    c.drawLine(const Offset(-14, -26), const Offset(-14, 24), ps(pa(Pop.white, .4), 4));
    c.restore();
  }, light: true),
  PopPanel('Mountain ridge', 'landscape · bold flat · 2D · pinch', G.pinch, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFFFFD9C2)));
    c.drawCircle(const Offset(40, 32), 14, pf(Pop.orange));
    for (var layer = 0; layer < 3; layer++) {
      final base = 62.0 + layer * 20, col = [Pop.violet, Pop.blue, Pop.ink][layer];
      final path = Path()..moveTo(0, s.height);
      for (var x = 0.0; x <= s.width; x += 3) {
        path.lineTo(x, base + wig(x / s.width * 2.2 + layer * 5 + p.t * .05 * (layer + 1), p[0], p[2], layer * 9.0) * p[1] * 40);
      }
      path
        ..lineTo(s.width, s.height)
        ..close();
      c.drawPath(path, pf(col));
    }
  }, light: true),
  PopPanel('Snake slither', 'creature · bold flat · top-down · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFFE8C77A)));
    final pts = <Offset>[];
    for (var i = 0; i <= 18; i++) {
      pts.add(Offset(s.width - 24 - i * 6.5, s.height / 2 + _w(p, 1, i * .045) * 34));
    }
    c.drawPath(poly(pts, close: false), ps(Pop.lime, 13));
    for (var i = 1; i < pts.length; i += 2) {
      c.drawCircle(pts[i], 3, pf(const Color(0xFF3FA34D)));
    }
    final h = pts.first;
    c.drawOval(Rect.fromCenter(center: h + const Offset(4, 0), width: 22, height: 17), pf(Pop.lime));
    c.drawCircle(h + const Offset(8, -4), 2.2, pf(Pop.ink));
    c.drawCircle(h + const Offset(8, 4), 2.2, pf(Pop.ink));
    c.drawPath(poly([h + const Offset(14, 0), h + const Offset(22, -3), h + const Offset(19, 0), h + const Offset(22, 3)]), pf(Pop.red));
  }, light: true),
  PopPanel('Pencil scribble', 'sketch · crisp · 2D · paint', G.paint, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(Pop.white));
    for (var y = 18.0; y < s.height; y += 14) {
      c.drawLine(Offset(0, y), Offset(s.width, y), ps(pa(Pop.cyan, .35), 1));
    }
    final ctr = s.center(Offset.zero);
    final pts = [for (var k = 0; k < 60; k++) ctr + _w2(p, 5, k * .025) * 70];
    c.drawPath(poly(pts, close: false), ps(Pop.ink, 2));
    final tip = pts.first;
    c.save();
    c.translate(tip.dx, tip.dy);
    c.rotate(-.7);
    c.drawPath(poly([Offset.zero, const Offset(6, -12), const Offset(-6, -12)]), pf(const Color(0xFFF2D19A)));
    c.drawRect(const Rect.fromLTWH(-6, -42, 12, 30), pf(Pop.yellow));
    c.drawRect(const Rect.fromLTWH(-6, -50, 12, 8), pf(Pop.pink));
    c.restore();
  }, light: true),
  PopPanel('Wobble cube', 'jelly · neon · pseudo 3D · spin', G.spin, (c, s, p) {
    final ctr = s.center(Offset.zero) + const Offset(0, 4);
    final a = p.t * .4;
    final verts = <Offset>[];
    for (var i = 0; i < 8; i++) {
      final x = (i & 1) == 0 ? -1.0 : 1.0, y = (i & 2) == 0 ? -1.0 : 1.0, z = (i & 4) == 0 ? -1.0 : 1.0;
      final rx = x * math.cos(a) - z * math.sin(a), rz = x * math.sin(a) + z * math.cos(a);
      final ry = y * math.cos(.5) - rz * math.sin(.5);
      verts.add(ctr + Offset(rx, ry) * 30 + _w2(p, i * 2.0) * 12);
    }
    const edges = [[0, 1], [2, 3], [4, 5], [6, 7], [0, 2], [1, 3], [4, 6], [5, 7], [0, 4], [1, 5], [2, 6], [3, 7]];
    for (final e in edges) {
      c.drawLine(verts[e[0]], verts[e[1]], ps(Pop.violet, 3));
    }
    for (final v in verts) {
      c.drawCircle(v, 4, pf(Pop.lime));
    }
  }),
  PopPanel('Rain on glass', 'weather · crisp · 2D · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF2E3F5C)));
    for (var k = 0; k < 5; k++) {
      c.drawCircle(Offset(20.0 + k * 30, 98 - (k % 2) * 10), 9, pf(pa(k.isEven ? Pop.orange : Pop.yellow, .5)));
    }
    for (var k = 0; k < 5; k++) {
      final x0 = 16.0 + k * 31, speed = 18 + k * 5.0;
      final y = (p.t * speed + k * 37) % (s.height + 20) - 10;
      final pts = [for (var i = 0; i < 12; i++) Offset(x0 + _w(p, k * 4.1, i * .12) * 14, y - i * 5.0)];
      c.drawPath(poly(pts, close: false), ps(pa(Pop.cyan, .45), 3));
      c.drawCircle(pts.first, 4.5, pf(Pop.cyan));
      c.drawCircle(pts.first + const Offset(-1.5, -1.5), 1.5, pf(Pop.white));
    }
    c.drawLine(Offset(s.width / 2, 0), Offset(s.width / 2, s.height), ps(Pop.panel, 5));
  }),
];
