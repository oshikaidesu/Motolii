// Pop 04 sheet 1: Easing (in / out / overshoot) x20. The reason you touch easing: "how does it set off, how does it arrive, does it fly
// past and come back". Every panel shows a thing setting off and arriving; values: [0] in (slow start), [1] out (soft arrival), [2] overshoot.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'pop_04_kit.dart';

double _e(PopV p, [double t = -1]) => ease(t < 0 ? tripT(p.t) : t, p[0], p[1], p[2]);
double _sp(PopV p) {
  final t = tripT(p.t);
  if (t <= 0 || t >= 1) return 0;
  return (ease(math.min(1, t + .02), p[0], p[1], p[2]) - ease(t, p[0], p[1], p[2])) / .02;
}

void _stars(Canvas c, Size s, int n, [Color col = Pop.cream]) {
  for (var i = 0; i < n; i++) {
    final x = (i * 73.3) % s.width, y = (i * 41.7 + i * i * 3.1) % s.height;
    c.drawCircle(Offset(x, y), i % 3 == 0 ? 1.2 : .7, pf(pa(col, .5)));
  }
}

void _isoBox(Canvas c, Offset o, double w, double d, double h, Color top, Color left, Color right) {
  final dx = Offset(w * .87, w * .5), dy = Offset(-d * .87, d * .5), up = Offset(0, -h);
  c.drawPath(poly([o + up, o + up + dx, o + up + dx + dy, o + up + dy]), pf(top));
  c.drawPath(poly([o + dy, o + dy + up, o + dy + dx + up, o + dy + dx]), pf(left));
  c.drawPath(poly([o + dx, o + dx + up, o + dx + dy + up, o + dx + dy]), pf(right));
}

final List<PopPanel> easePanels = [
  PopPanel('Rocket hop', 'machine · bold flat · 2D · drag', G.drag, (c, s, p) {
    _stars(c, s, 26);
    final e = _e(p), v = _sp(p);
    final ring = Offset(s.width / 2, 22), pad = Offset(s.width / 2, s.height - 12);
    c.drawRect(Rect.fromLTWH(pad.dx - 22, pad.dy, 44, 4), pf(Pop.panel2));
    c.drawOval(Rect.fromCenter(center: ring, width: 52, height: 12), ps(Pop.cyan, 3));
    final y = lr(pad.dy - 14, ring.dy + 4, e);
    final f = cl(v / 2.2) * 26 + 3;
    c.drawPath(poly([Offset(s.width / 2 - 6, y + 12), Offset(s.width / 2 + 6, y + 12), Offset(s.width / 2, y + 12 + f)]), pf(Pop.orange));
    c.drawPath(poly([Offset(s.width / 2 - 3, y + 12), Offset(s.width / 2 + 3, y + 12), Offset(s.width / 2, y + 12 + f * .55)]), pf(Pop.yellow));
    final body = Path()
      ..moveTo(s.width / 2, y - 16)
      ..quadraticBezierTo(s.width / 2 + 9, y - 6, s.width / 2 + 7, y + 12)
      ..lineTo(s.width / 2 - 7, y + 12)
      ..quadraticBezierTo(s.width / 2 - 9, y - 6, s.width / 2, y - 16);
    c.drawPath(poly([Offset(s.width / 2 - 7, y + 4), Offset(s.width / 2 - 13, y + 14), Offset(s.width / 2 - 6, y + 12)]), pf(Pop.red));
    c.drawPath(poly([Offset(s.width / 2 + 7, y + 4), Offset(s.width / 2 + 13, y + 14), Offset(s.width / 2 + 6, y + 12)]), pf(Pop.red));
    c.drawPath(body, pf(Pop.cream));
    c.drawCircle(Offset(s.width / 2, y - 3), 3, pf(Pop.cyan));
    c.drawArc(Rect.fromCenter(center: ring, width: 52, height: 12), 0, math.pi, false, ps(Pop.cyan, 3));
  }),
  PopPanel('Red light stop', 'map · flat · top-down · drag', G.drag, (c, s, p) {
    final e = _e(p), road = Rect.fromLTWH(0, s.height / 2 - 24, s.width, 48);
    c.drawRect(road, pf(Pop.ink));
    for (var x = 4.0; x < s.width; x += 16) {
      c.drawLine(Offset(x, s.height / 2), Offset(x + 8, s.height / 2), ps(Pop.yellow, 2));
    }
    final a = 18.0, b = s.width - 34;
    c.drawLine(Offset(b + 14, road.top), Offset(b + 14, road.bottom), ps(Pop.white, 4));
    c.drawCircle(Offset(a - 8, road.top - 9), 5, pf(Pop.lime));
    c.drawCircle(Offset(b + 14, road.top - 9), 5, pf(Pop.red));
    final x = lr(a, b, e), y = s.height / 2 + 12;
    final decel = tripT(p.t) > .5 ? cl(_sp(p) * .8) : 0.0;
    if (decel > .05) {
      for (final dy in [-5.0, 5.0]) {
        c.drawLine(Offset(x - 28 * decel, y + dy), Offset(x - 8, y + dy), ps(pa(Pop.cream, .35), 2.5));
      }
    }
    rr(c, Rect.fromCenter(center: Offset(x + 4, y), width: 28, height: 15), 5, pf(Pop.orange));
    rr(c, Rect.fromCenter(center: Offset(x + 8, y), width: 8, height: 11), 2, pf(Pop.cyan));
    if (x + 18 > b + 14) c.drawCircle(Offset(b + 14, road.top - 9), 9, ps(Pop.red, 2));
  }, light: true),
  PopPanel('Zipper run', 'material · crisp · 2D · drag', G.drag, (c, s, p) {
    final y = s.height / 2, a = 10.0, b = s.width - 12, x = lr(a, b, cl(_e(p), 0, 1.15));
    double gap(double px) => px < x ? 0 : cl((px - x) / 70) * 30;
    final top = Path()..moveTo(0, 0), bot = Path()..moveTo(0, s.height);
    for (var px = 0.0; px <= s.width; px += 4) {
      top.lineTo(px, y - 4 - gap(px));
      bot.lineTo(px, y + 4 + gap(px));
    }
    top
      ..lineTo(s.width, 0)
      ..close();
    bot
      ..lineTo(s.width, s.height)
      ..close();
    c.drawPath(top, pf(Pop.violet));
    c.drawPath(bot, pf(Pop.pink));
    for (var px = 2.0; px < s.width; px += 6) {
      final g = gap(px);
      c.drawRect(Rect.fromLTWH(px, y - 6 - g, 4, 5), pf(Pop.yellow));
      c.drawRect(Rect.fromLTWH(px + 3, y + 1 + g, 4, 5), pf(Pop.yellow));
    }
    rr(c, Rect.fromCenter(center: Offset(x + 2, y), width: 16, height: 14), 4, pf(Pop.cream));
    rr(c, Rect.fromLTWH(x - 2, y + 4, 8, 22), 4, pf(Pop.cream));
    c.drawCircle(Offset(x + 2, y + 20), 2.5, pf(Pop.ink));
  }),
  PopPanel('Frog leap', 'creature · bold flat · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(Pop.blue));
    final a = Offset(26, s.height - 26), b = Offset(s.width - 30, s.height - 26);
    for (final o in [a, b]) {
      c.drawOval(Rect.fromCenter(center: o, width: 44, height: 16), pf(Pop.lime));
      c.drawPath(poly([o, o + const Offset(16, -3), o + const Offset(16, 4)]), pf(Pop.blue));
    }
    final t = tripT(p.t), e = _e(p);
    final pos = ol(a, b, e) + Offset(0, -math.sin(math.pi * cl(t)) * 58 - 9);
    if (e > 1.03 && t > .8) {
      c.drawOval(Rect.fromCenter(center: b + Offset(22 * (e - 1) * 4, 2), width: 30, height: 8), ps(Pop.cyan, 2));
    }
    c.drawOval(Rect.fromCenter(center: pos, width: 26, height: 18), pf(Pop.lime));
    c.drawOval(Rect.fromCenter(center: pos + const Offset(0, 3), width: 16, height: 9), pf(Pop.yellow));
    for (final dx in [-6.0, 6.0]) {
      c.drawCircle(pos + Offset(dx, -9), 5, pf(Pop.white));
      c.drawCircle(pos + Offset(dx + 1, -9), 2.2, pf(Pop.ink));
    }
  }),
  PopPanel('Curling glide', 'sport · crisp · top-down · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFFE9F6FB)));
    final h = Offset(s.width - 36, s.height / 2);
    for (final (r, col) in [(30.0, Pop.blue), (21.0, Pop.white), (12.0, Pop.red), (4.0, Pop.white)]) {
      c.drawCircle(h, r, pf(col));
    }
    final x = lr(16, h.dx, _e(p));
    c.drawLine(Offset(10, h.dy), Offset(x, h.dy), ps(pa(Pop.cyan, .5), 10));
    c.drawCircle(Offset(x, h.dy) + const Offset(1.5, 2), 11, pf(pa(Pop.ink, .2)));
    c.drawCircle(Offset(x, h.dy), 11, pf(const Color(0xFF8E8E96)));
    c.drawCircle(Offset(x, h.dy), 7, pf(Pop.yellow));
    rr(c, Rect.fromCenter(center: Offset(x, h.dy), width: 12, height: 4), 2, pf(Pop.ink));
  }, light: true),
  PopPanel('Lift shaft', 'building · bold · isometric · drag', G.drag, (c, s, p) {
    final base = Offset(s.width / 2 - 18, s.height - 10);
    for (var f = 0; f < 4; f++) {
      final y = base.dy - f * 24.0;
      c.drawLine(Offset(base.dx - 26, y + 6), Offset(base.dx + 58, y - 6), ps(pa(Pop.violet, .45), 2));
    }
    c.drawLine(base + const Offset(15, -96), base + const Offset(15, 4), ps(Pop.panel2, 3));
    final y = lr(0, 72, _e(p));
    _isoBox(c, base + Offset(0, -y), 20, 18, 22, Pop.yellow, Pop.orange, Pop.red);
    for (var f = 0; f < 4; f++) {
      final on = (y / 24 - f).abs() < .5;
      c.drawCircle(Offset(s.width - 20, base.dy - 10 - f * 24.0), 4, pf(on ? Pop.lime : Pop.panel2));
    }
  }),
  PopPanel('Curtain call', 'theatre · bold flat · 2D · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF241A2E)));
    c.drawPath(poly([Offset(s.width / 2, 0), Offset(s.width / 2 - 34, s.height), Offset(s.width / 2 + 34, s.height)]), pf(pa(Pop.yellow, .22)));
    c.drawCircle(Offset(s.width / 2, s.height - 40), 9, pf(Pop.cream));
    rr(c, Rect.fromLTWH(s.width / 2 - 9, s.height - 30, 18, 26), 6, pf(Pop.pink));
    final open = cl(_e(p), 0, 1.3) * (s.width / 2 - 8);
    for (final side in [-1, 1]) {
      final edge = s.width / 2 + side * open, outer = side < 0 ? 0.0 : s.width;
      final path = Path()..moveTo(outer, 0);
      path.lineTo(edge, 0);
      for (var y = 0.0; y <= s.height; y += 12) {
        path.lineTo(edge + side * (y / s.height) * 6 + math.sin(y / 6) * 2, y);
      }
      path
        ..lineTo(outer, s.height)
        ..close();
      c.drawPath(path, pf(Pop.red));
      for (var k = 1; k < 4; k++) {
        final xx = lr(edge, outer, k / 4);
        c.drawLine(Offset(xx, 0), Offset(xx, s.height), ps(pa(Pop.ink, .25), 2));
      }
    }
    c.drawRect(Rect.fromLTWH(0, 0, s.width, 10), pf(Pop.yellow));
  }),
  PopPanel('Paint dabs', 'brush · neon · 2D · paint', G.paint, (c, s, p) {
    for (var k = 0; k <= 22; k++) {
      final t = k / 22, e = _e(p, t), e2 = _e(p, math.min(1, t + .03));
      final sp = ((e2 - e) / .03).abs();
      final pos = Offset(lr(14, s.width - 14, e), s.height / 2 + math.sin(e * math.pi * 2) * 22);
      final r = cl(7 / (sp + .5), 2, 11);
      c.drawCircle(pos, r, pf(Color.lerp(Pop.pink, Pop.yellow, t)!));
    }
    final e = _e(p);
    c.drawCircle(Offset(lr(14, s.width - 14, e), s.height / 2 + math.sin(e * math.pi * 2) * 22), 4, pf(Pop.white));
  }),
  PopPanel('Comet transfer', 'cosmic · neon · 2D · spin', G.spin, (c, s, p) {
    _stars(c, s, 34, Pop.violet);
    final a = Offset(22, s.height - 26), b = Offset(s.width - 26, 30);
    c.drawCircle(a, 15, pf(Pop.orange));
    c.drawCircle(b, 13, pf(Pop.cyan));
    c.drawOval(Rect.fromCenter(center: b, width: 40, height: 9), ps(Pop.cream, 2));
    Offset at(double e) => ol(a, b, e) + Offset(0, -math.sin(math.pi * cl(e)) * 26);
    final e = _e(p), v = _sp(p);
    final tail = cl(v / 2.2) * .22 + .02;
    for (var k = 0; k < 10; k++) {
      final q = at(e - tail * k / 10);
      c.drawCircle(q, 5.5 * (1 - k / 10), pf(pa(Pop.pink, 1 - k / 10)));
    }
    c.drawCircle(at(e), 5, pf(Pop.white));
  }),
  PopPanel('Jelly slide', 'food · bold flat · 2D · rub', G.rub, (c, s, p) {
    c.drawOval(Rect.fromLTWH(8, s.height - 32, s.width - 16, 22), pf(Pop.white));
    c.drawOval(Rect.fromLTWH(14, s.height - 30, s.width - 28, 16), pf(const Color(0xFFE6DCCB)));
    final t = tripT(p.t);
    final e = _e(p);
    final acc = t <= 0 || t >= 1 ? 0.0 : (_e(p, math.min(1, t + .03)) - 2 * e + _e(p, math.max(0, t - .03))) / (.03 * .03);
    final ring = t >= 1 ? math.sin((p.t % 2.4 - 1.8) * 22) * math.exp(-(p.t % 2.4 - 1.8) * 5) * p[2] * 10 : 0.0;
    final lean = cl(-acc / 30, -1, 1) * 9 + ring, x = lr(34, s.width - 34, e), y = s.height - 22;
    c.drawPath(poly([Offset(x - 18, y), Offset(x + 18, y), Offset(x + 13 + lean, y - 34), Offset(x - 13 + lean, y - 34)]), pf(Pop.pink));
    c.drawPath(poly([Offset(x - 13 + lean, y - 34), Offset(x + 13 + lean, y - 34), Offset(x + 10 + lean, y - 38), Offset(x - 10 + lean, y - 38)]), pf(const Color(0xFFFF8CC0)));
    c.drawCircle(Offset(x + lean, y - 43), 5, pf(Pop.red));
    c.drawLine(Offset(x - 10 + lean * .6, y - 22), Offset(x - 8 + lean * .4, y - 8), ps(pa(Pop.white, .6), 3));
  }, light: true),
  PopPanel('Footprints', 'nature · bold flat · top-down · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFFF2D9A6)));
    final t = tripT(p.t);
    for (var k = 0; k <= 14; k++) {
      if (k / 14 > t + .001) break;
      final x = lr(12, s.width - 16, _e(p, k / 14)), y = s.height / 2 + (k.isEven ? -12 : 12);
      final col = k / 14 > t - .08 ? Pop.orange : pa(Pop.ink, .55);
      c.drawOval(Rect.fromCenter(center: Offset(x, y), width: 15, height: 9), pf(col));
      for (var j = 0; j < 3; j++) {
        c.drawCircle(Offset(x + 10, y - 4 + j * 4.0), 1.9, pf(col));
      }
    }
    c.drawCircle(Offset(s.width - 12, s.height / 2), 6, ps(Pop.red, 2.5));
  }, light: true),
  PopPanel('Hare vs ghost', 'toy · bold flat · 2D · flick', G.flick, (c, s, p) {
    final fin = s.width - 14;
    for (var k = 0; k < 8; k++) {
      c.drawRect(Rect.fromLTWH(fin, k * s.height / 8, 7, s.height / 8), pf(k.isEven ? Pop.ink : Pop.white));
    }
    final lin = tripT(p.t), e = _e(p);
    final gx = lr(16, fin - 12, lin), hx = lr(16, fin - 12, e);
    c.drawLine(Offset(0, s.height / 2), Offset(fin, s.height / 2), ps(pa(Pop.ink, .15), 2));
    c.drawCircle(Offset(gx, 32), 10, ps(pa(Pop.ink, .4), 2));
    c.drawCircle(Offset(gx - 3, 30), 1.6, pf(pa(Pop.ink, .4)));
    c.drawCircle(Offset(gx + 3, 30), 1.6, pf(pa(Pop.ink, .4)));
    final hop = (math.sin(e * 30)).abs() * 5, y = s.height - 30 - hop;
    c.drawOval(Rect.fromCenter(center: Offset(hx, y), width: 24, height: 15), pf(Pop.orange));
    c.drawCircle(Offset(hx + 11, y - 5), 7, pf(Pop.orange));
    rr(c, Rect.fromLTWH(hx + 8, y - 22, 4, 12), 2, pf(Pop.orange));
    rr(c, Rect.fromLTWH(hx + 13, y - 21, 4, 12), 2, pf(Pop.orange));
    c.drawCircle(Offset(hx + 13, y - 6), 1.6, pf(Pop.ink));
    c.drawCircle(Offset(hx - 12, y - 2), 4, pf(Pop.white));
  }, light: true),
  PopPanel('Cloud shade', 'weather · bold flat · landscape · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(Pop.cyan));
    final sun = Offset(s.width / 2 + 8, 30);
    c.drawCircle(sun, 14, pf(Pop.yellow));
    final x = lr(-30, s.width / 2 + 8, _e(p));
    final hills = Path()
      ..moveTo(0, s.height)
      ..lineTo(0, s.height - 30)
      ..quadraticBezierTo(40, s.height - 56, 80, s.height - 34)
      ..quadraticBezierTo(120, s.height - 14, s.width, s.height - 44)
      ..lineTo(s.width, s.height)
      ..close();
    c.drawPath(hills, pf(Pop.lime));
    c.save();
    c.clipPath(hills);
    c.drawOval(Rect.fromCenter(center: Offset(x, s.height - 24), width: 70, height: 26), pf(pa(Pop.ink, .25)));
    c.restore();
    for (final (dx, dy, r) in [(-14.0, 4.0, 10.0), (0.0, -3.0, 13.0), (15.0, 4.0, 10.0)]) {
      c.drawCircle(Offset(x + dx, 34 + dy), r, pf(Pop.white));
    }
    rr(c, Rect.fromLTWH(x - 24, 34, 48, 12), 6, pf(Pop.white));
  }),
  PopPanel('Depth rush', 'tunnel · neon · pseudo 3D · pinch', G.pinch, (c, s, p) {
    final ctr = s.center(Offset.zero);
    for (var k = 1; k <= 5; k++) {
      final z = math.pow(k / 5, 2).toDouble();
      c.drawRect(Rect.fromCenter(center: ctr, width: s.width * z, height: s.height * z), ps(pa(Pop.violet, .3 + .14 * k), 1.5));
    }
    for (final q in [Offset.zero, Offset(s.width, 0), Offset(0, s.height), Offset(s.width, s.height)]) {
      c.drawLine(ctr, q, ps(pa(Pop.violet, .35), 1));
    }
    final e = _e(p), z = lr(.04, .62, e);
    final r = Rect.fromCenter(center: ctr, width: s.width * z, height: s.height * z);
    c.drawRect(r, pf(Pop.pink));
    c.drawRect(r.deflate(r.width * .18), pf(Pop.yellow));
  }),
  PopPanel('Card deal', 'deck · crisp · 2D · stack', G.drag, (c, s, p) {
    final deck = Offset(28, s.height - 32), slot = Offset(s.width - 34, 36);
    rr(c, Rect.fromCenter(center: slot, width: 32, height: 44), 5, ps(pa(Pop.cream, .4), 2));
    for (var k = 4; k >= 0; k--) {
      rr(c, Rect.fromCenter(center: deck + Offset(-k * 1.5, k * 2.5), width: 32, height: 44), 5, pf(k == 0 ? Pop.blue : const Color(0xFF2448CC)));
    }
    final e = _e(p);
    c.save();
    final pos = ol(deck, slot, e);
    c.translate(pos.dx, pos.dy);
    c.rotate((1 - e) * -1.2);
    rr(c, Rect.fromCenter(center: Offset.zero, width: 32, height: 44), 5, pf(Pop.cream));
    c.drawPath(poly([const Offset(0, -9), const Offset(7, 0), const Offset(0, 9), const Offset(-7, 0)]), pf(Pop.red));
    c.restore();
  }),
  PopPanel('Balloon summit', 'landscape · bold flat · 2D · drag', G.drag, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF243B8F)));
    c.drawPath(poly([Offset(s.width - 70, s.height), Offset(s.width - 26, 40), Offset(s.width + 20, s.height)]), pf(Pop.violet));
    c.drawPath(poly([Offset(s.width - 34, 55), Offset(s.width - 26, 40), Offset(s.width - 18, 55)]), pf(Pop.white));
    c.drawRect(Rect.fromLTWH(0, s.height - 12, s.width, 12), pf(Pop.lime));
    final e = _e(p), pos = ol(Offset(26, s.height - 34), Offset(s.width - 26, 16), e);
    c.drawLine(pos + const Offset(-5, 10), pos + const Offset(-3, 17), ps(Pop.cream, 1));
    c.drawLine(pos + const Offset(5, 10), pos + const Offset(3, 17), ps(Pop.cream, 1));
    c.drawCircle(pos, 13, pf(Pop.orange));
    c.drawOval(Rect.fromCenter(center: pos, width: 9, height: 26), pf(Pop.yellow));
    rr(c, Rect.fromCenter(center: pos + const Offset(0, 19), width: 9, height: 6), 1.5, pf(Pop.red));
  }),
  PopPanel('Soft-close drawer', 'furniture · bold · isometric · drag', G.drag, (c, s, p) {
    final o = Offset(s.width / 2 + 4, 34);
    _isoBox(c, o, 46, 30, 22, const Color(0xFF45454D), const Color(0xFF34343A), const Color(0xFF2A2A30));
    final e = _e(p), out = (1 - e) * 30 + 2;
    final face = o + const Offset(-30 * .87, 30 * .5) + const Offset(6 * .87, 6 * .5);
    _isoBox(c, face + const Offset(0, 4), 34, out, 13, Pop.orange, Pop.yellow, Pop.red);
    final f2 = face + Offset(-out * .87, out * .5) + const Offset(0, 4);
    c.drawLine(f2 + const Offset(11, -2), f2 + const Offset(20, 3), ps(Pop.ink, 3));
    final v = _sp(p);
    if (tripT(p.t) > .8 && v > .6) {
      for (var k = 0; k < 3; k++) {
        c.drawLine(f2 + Offset(-6, -14.0 + k * 8), f2 + Offset(-18, -16.0 + k * 8), ps(Pop.white, 2));
      }
    }
  }),
  PopPanel('Fish dart', 'nature · bold flat · 2D · flick', G.flick, (c, s, p) {
    c.drawRect(Offset.zero & s, pf(const Color(0xFF0F3D63)));
    for (final (x, w) in [(0.0, 34.0), (s.width - 30, 30.0)]) {
      c.drawOval(Rect.fromLTWH(x, s.height - 24, w, 30), pf(Pop.violet));
    }
    final t = tripT(p.t), e = _e(p), v = _sp(p);
    final pos = Offset(lr(30, s.width - 32, e), s.height / 2 + math.sin(t * math.pi) * -10);
    for (var k = 1; k <= 6; k++) {
      final bt = cl(t - k * .05);
      final be = _e(p, bt), bp = Offset(lr(30, s.width - 32, be), s.height / 2 - 10 - k * 3.0);
      if (v > .4) c.drawCircle(bp, 2 + k * .4, ps(pa(Pop.cyan, 1 - k / 7), 1.2));
    }
    final wag = math.sin(p.t * (4 + v * 18)) * 5;
    c.drawPath(poly([pos + const Offset(-10, 0), pos + Offset(-22, -8 + wag), pos + Offset(-22, 8 + wag)]), pf(Pop.yellow));
    c.drawOval(Rect.fromCenter(center: pos, width: 30, height: 18), pf(Pop.orange));
    c.drawCircle(pos + const Offset(8, -2), 2.5, pf(Pop.white));
    c.drawLine(pos + const Offset(-2, -8), pos + const Offset(-2, 8), ps(Pop.white, 2.5));
  }),
  PopPanel('Marquee chase', 'lights · neon · 2D · spin', G.spin, (c, s, p) {
    final ctr = Offset(s.width / 2, s.height - 14);
    const n = 15;
    final e = _e(p);
    for (var k = 0; k < n; k++) {
      final a = math.pi + k / (n - 1) * math.pi, q = polar(ctr, 62, a);
      final d = (e * (n - 1) - k).abs();
      final glow = cl(1 - d / 2.5);
      if (glow > 0) c.drawCircle(q, 6 + glow * 5, pf(pa(Pop.yellow, glow * .35)));
      c.drawCircle(q, 5, pf(Color.lerp(Pop.panel2, Pop.yellow, glow)!));
    }
    c.drawCircle(ctr, 30, pf(Pop.red));
    c.drawCircle(ctr, 22, pf(Pop.panel));
  }),
  PopPanel('Toaster pop', 'kitchen · bold flat · 2D · drag', G.drag, (c, s, p) {
    final e = _e(p), base = s.height - 16;
    final ty = lr(base - 30, base - 84, e);
    rr(c, Rect.fromLTWH(s.width / 2 - 22, ty, 44, 46), 10, pf(const Color(0xFFD9A15A)));
    rr(c, Rect.fromLTWH(s.width / 2 - 18, ty + 4, 36, 40), 8, pf(const Color(0xFFF2D19A)));
    rr(c, Rect.fromLTWH(s.width / 2 - 42, base - 46, 84, 46), 14, pf(Pop.red));
    rr(c, Rect.fromLTWH(s.width / 2 - 26, base - 48, 52, 6), 3, pf(Pop.ink));
    rr(c, Rect.fromLTWH(s.width / 2 + 30, base - 30 + e * 14, 14, 5), 2, pf(Pop.ink));
    c.drawLine(Offset(s.width / 2 - 30, base - 30), Offset(s.width / 2 - 30, base - 12), ps(pa(Pop.white, .6), 4));
    if (e > 1.02) {
      for (final dx in [-30.0, 30.0]) {
        c.drawLine(Offset(s.width / 2 + dx, ty + 6), Offset(s.width / 2 + dx * 1.4, ty - 2), ps(Pop.ink, 2.5));
      }
    }
  }, light: true),
];
