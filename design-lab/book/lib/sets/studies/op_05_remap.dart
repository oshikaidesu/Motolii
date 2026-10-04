// OP 05 sheet 1, Time remap. v[0] -> speed -2..+4: 1x at .5, hold window round 0, below 0 = reverse.
// Encoder colours: speed blue, reverse red, hold white with a red [HOLD] tag.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'op_05_kit.dart';

double spd(OpP p) {
  final v = p.a * 6 - 2;
  return v.abs() < .2 ? 0 : v;
}

Color spc(double v) => v == 0 ? kW : (v < 0 ? kR : kB);

/// Quiet readout: speed in blue (red when reverse), [HOLD] when frozen.
void readout(Canvas c, Offset o, double v, {double h = 14, double ax = 0}) {
  if (v == 0) {
    tag(c, 'HOLD', o, kR, ax: ax);
    return;
  }
  num(c, f1(v.abs()), o, h, spc(v), ax: ax);
  if (v < 0) tag(c, 'REV', o + Offset(ax == 0 ? 0 : -40, h + 4), kR);
}

List<OpSpec> remapPanels() => [
      OpSpec('Stick walker', 'character · effect · drag · drawing', _walker),
      OpSpec('Tape reels', 'machine · mechanism · spin · drawing', _reels, g: G.spin, init: const [.58, .5, .5]),
      OpSpec('Time curve', 'diagram · mechanism · drag v · diagram', _curve, x: -1, y: 0, init: const [.62, .5, .5]),
      OpSpec('Night drive', 'vehicle · effect · drag v · drawing', _drive, x: -1, y: 0, init: const [.7, .5, .5]),
      OpSpec('Hourglass', 'object · effect · flick · drawing', _hourglass, g: G.flick, init: const [.45, .5, .5]),
      OpSpec('Metronome', 'instrument · mechanism · drag · numeral', _metronome),
      OpSpec('Onion bounce', 'sample · effect · drag · drawing', _onion, init: const [.42, .5, .5]),
      OpSpec('Clock hands', 'object · mechanism · spin · numeral', _clock, g: G.spin, init: const [.6, .5, .5]),
      OpSpec('Frame numbers', 'typographic · effect · drag · type', _frames, init: const [.42, .5, .5]),
      OpSpec('Scratch platter', 'instrument · effect · rub · pushed', _platter, g: G.rub),
      OpSpec('Draw the speed', 'diagram · mechanism · draw · diagram', _drawn, g: G.draw),
      OpSpec('Planet trail', 'cosmic · effect · spin · drawing', _planet, g: G.spin, init: const [.75, .5, .5]),
      OpSpec('Heart monitor', 'diagram · effect · drag v · numeral', _ecg, x: -1, y: 0),
      OpSpec('Monkey drummer', 'character · effect · flick · drawing', _drummer, g: G.flick),
      OpSpec('Iso conveyor', 'isometric · effect · drag · drawing', _conveyor, init: const [.6, .5, .5]),
      OpSpec('Frame wiring', 'diagram · mechanism · drag · pushed', _wiring, init: const [.42, .5, .5]),
      OpSpec('Giant numeral', 'typographic · readout · drag v · pushed', _giant, x: -1, y: 0, init: const [.7, .5, .5]),
      OpSpec('Faucet drip', 'object · effect · drag v · drawing', _faucet, x: -1, y: 0, init: const [.45, .5, .5]),
      OpSpec('Time tunnel', 'cosmic · effect · pinch · pushed', _tunnel, g: G.pinch, init: const [.8, .5, .5]),
      OpSpec('Frog hops', 'animal · effect · drag · drawing', _frog, init: const [.55, .5, .5]),
    ];

void _walker(Canvas c, Size s, OpP p) {
  final v = spd(p), gy = 98.0;
  ln(c, Offset(8, gy), Offset(s.width - 8, gy), kG);
  for (var i = 0; i < 9; i++) {
    final x = 8 + frac(i / 9 - p.ph * .25) * (s.width - 16);
    ln(c, Offset(x, gy + 3), Offset(x, gy + 7), kDim);
  }
  final cx = 84.0, w = math.sin(p.ph * math.pi * 2.2) * .55, bob = math.cos(p.ph * math.pi * 4.4).abs() * 2;
  final hip = Offset(cx, 68 - bob), neck = Offset(cx + 2, 44 - bob);
  ring(c, Offset(cx + 4, 35 - bob), 7, kW);
  ln(c, hip, neck, kW);
  for (final sg in [1.0, -1.0]) {
    final knee = pol(hip, 15, math.pi / 2 + w * sg);
    pl(c, [hip, knee, pol(knee, 15, math.pi / 2 + w * sg - .25 * (1 + sg * math.sin(p.ph * 13.8)))], sg > 0 ? kB : kPu);
    final el = pol(neck + const Offset(0, 4), 12, math.pi / 2 - w * sg * .8);
    pl(c, [neck + const Offset(0, 4), el, pol(el, 10, math.pi / 2 - w * sg * .8 - .6)], sg > 0 ? kB : kPu);
  }
  readout(c, const Offset(10, 10), v);
  lab(c, 'SPEED', const Offset(10, 28), kMid, size: 7);
}

void _reels(Canvas c, Size s, OpP p) {
  final v = spd(p), a = p.ph * math.pi * 2 * .5;
  void reel(Offset o, double full, double rot) {
    ring(c, o, 22, kDim);
    ring(c, o, 9 + full * 12, kB);
    ring(c, o, 4, kW);
    for (var k = 0; k < 3; k++) {
      ln(c, pol(o, 4, rot + k * 2.094), pol(o, 9, rot + k * 2.094), kW);
    }
  }

  final fill = .5 + .4 * math.sin(p.ph * .1);
  reel(const Offset(44, 50), fill, a);
  reel(const Offset(112, 50), 1 - fill, a * 1.3);
  pl(c, [Offset(44 - 9 - fill * 12, 50), const Offset(30, 94), const Offset(126, 94), Offset(112 + 9 + (1 - fill) * 12, 50)], kG);
  c.drawRect(const Rect.fromLTWH(70, 88, 16, 10), st(kW));
  readout(c, const Offset(148, 100), v, h: 11, ax: 1);
  lab(c, 'TAPE', const Offset(8, 104), kMid, size: 7);
}

void _curve(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const o = Offset(16, 70);
  ln(c, const Offset(16, 108), const Offset(16, 10), kDim);
  ln(c, o, const Offset(150, 70), kDim);
  lab(c, 'IN', const Offset(140, 74), kMid, size: 7);
  lab(c, 'OUT', const Offset(20, 8), kMid, size: 7);
  final pts = [for (var i = 0; i <= 20; i++) Offset(16 + i * 6.5, (70 - i * 6.5 * v * .35).clamp(4.0, 116.0))];
  pl(c, pts, spc(v), w: 1.4);
  final u = frac(p.t * .3), x = 16 + u * 130, y = (70 - u * 130 * v * .35).clamp(4.0, 116.0);
  dots(c, Offset(x, 70), Offset(x, y), kW);
  dots(c, Offset(16, y), Offset(x, y), kDim);
  dot(c, Offset(x, y), kW, 2.5);
  readout(c, const Offset(148, 86), v, h: 22, ax: 1);
}

void _drive(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const vp = Offset(78, 46);
  final sky = <Offset>[const Offset(40, 46)];
  for (var i = 0; i < 10; i++) {
    final x = 40 + i * 8.0, h = [6, 12, 9, 18, 7, 14, 22, 10, 8, 5][i].toDouble();
    sky.addAll([Offset(x, 46 - h), Offset(x + 8, 46 - h)]);
  }
  sky.add(const Offset(120, 46));
  pl(c, sky, kPu);
  ln(c, const Offset(6, 46), const Offset(150, 46), kB);
  ln(c, vp, const Offset(8, 118), kW);
  ln(c, vp, const Offset(148, 118), kW);
  for (var i = 0; i < 7; i++) {
    final z = frac(i / 7 + p.ph * .35), z2 = frac(i / 7 + p.ph * .35 + .05);
    ln(c, Offset(78, 46 + 72 * z * z), Offset(78, 46 + 72 * z2 * z2), kG, 1 + z * 1.4);
    for (final sd in [-1.0, 1.0]) {
      final y = 46 + 72 * z * z;
      ln(c, Offset(78 + sd * (y - 46) * 1.0, y), Offset(78 + sd * ((y - 46) * 1.0 + 6), y), kDim);
    }
  }
  final gear = v == 0 ? 'P' : (v < 0 ? 'R' : 'D');
  tag(c, gear, const Offset(8, 104), v < 0 ? kR : (v == 0 ? kW : kG));
  if (v != 0) num(c, f1(v.abs()), const Offset(148, 8), 14, spc(v), ax: 1);
  if (v == 0) tag(c, 'HOLD', const Offset(148, 8), kR, ax: 1);
}

void _hourglass(Canvas c, Size s, OpP p) {
  final v = spd(p);
  c.save();
  c.translate(60, 60);
  c.rotate(v < 0 ? math.pi : 0);
  pl(c, const [Offset(-24, -44), Offset(24, -44), Offset(3, -2), Offset(3, 2), Offset(24, 44), Offset(-24, 44), Offset(-3, 2), Offset(-3, -2)], kW, close: true);
  final u = frac(p.ph * .12), top = 40 * (1 - u), bot = 40 * u;
  final yt = -2 - top, hw = 24 * top / 42;
  pl(c, [Offset(-hw, yt), Offset(hw, yt)], kB);
  final yb = 44 - bot * .55;
  pl(c, [Offset(-22, 44), Offset(0, yb), Offset(22, 44)], kB);
  for (var k = 0; k < 6; k++) {
    final y = 2 + frac(k / 6 + p.ph * 1.5) * (yb - 4);
    dot(c, Offset(0, y), v == 0 ? kDim : kB, 1);
  }
  c.restore();
  readout(c, const Offset(148, 10), v, h: 22, ax: 1);
  lab(c, v < 0 ? 'FLIPPED' : 'FLOW', const Offset(148, 106), kMid, size: 7, ax: 1);
}

void _metronome(Canvas c, Size s, OpP p) {
  final v = spd(p), bpm = (120 * v.abs()).round();
  num(c, '$bpm', const Offset(10, 14), 34, spc(v));
  lab(c, v == 0 ? 'HOLD' : (v < 0 ? 'BPM  REV' : 'BPM'), const Offset(10, 54), v == 0 ? kR : kMid, size: 7);
  pl(c, const [Offset(104, 14), Offset(124, 14), Offset(140, 110), Offset(88, 110)], kW, close: true);
  const pivot = Offset(114, 98);
  final ang = -math.pi / 2 + .45 * math.sin(p.ph * math.pi);
  final tip = pol(pivot, 78, ang);
  ln(c, pivot, tip, kB);
  final wpos = Offset.lerp(pivot, tip, .85 - ((v.abs()).clamp(0, 4) / 4) * .55)!;
  c.save();
  c.translate(wpos.dx, wpos.dy);
  c.rotate(ang + math.pi / 2);
  c.drawRect(const Rect.fromLTWH(-5, -3, 10, 6), st(kG));
  c.restore();
  dot(c, pivot, kW, 2);
}

Offset _bounce(double t) {
  final u = frac(t * .35), x = 14 + u * 128, hop = frac(u * 3);
  return Offset(x, 96 - 64 * (1 - (2 * hop - 1) * (2 * hop - 1)) * (1 - u * .5));
}

void _onion(Canvas c, Size s, OpP p) {
  final v = spd(p);
  ln(c, const Offset(8, 104), const Offset(148, 104), kG);
  for (var k = 8; k >= 1; k--) {
    final q = _bounce(p.ph - k * .09 * v);
    ring(c, q, 6, al(kB, .85 - k * .09), 1);
  }
  ring(c, _bounce(p.ph), 7, kW, 1.4);
  readout(c, const Offset(148, 8), v, ax: 1);
  lab(c, 'GHOSTS = SPEED', const Offset(8, 8), kMid, size: 7);
}

void _clock(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const o = Offset(56, 60);
  ring(c, o, 42, kW);
  for (var i = 0; i < 12; i++) {
    final a = i * math.pi / 6;
    ln(c, pol(o, i % 3 == 0 ? 34 : 37, a), pol(o, 40, a), i % 3 == 0 ? kW : kDim);
  }
  final m = p.ph * .5;
  ln(c, o, pol(o, 32, -math.pi / 2 + m * math.pi * 2), kB, 1.4);
  ln(c, o, pol(o, 20, -math.pi / 2 + m * math.pi * 2 / 12), kG, 1.6);
  dot(c, o, kW, 2);
  final mins = (m * 60).floor();
  num(c, '${d2((mins ~/ 60) % 24)}:${d2(mins % 60)}', const Offset(148, 14), 13, kW, ax: 1);
  readout(c, const Offset(148, 92), v, h: 12, ax: 1);
}

void _frames(Canvas c, Size s, OpP p) {
  final v = spd(p);
  for (var k = 0; k < 4; k++) {
    final r = Rect.fromLTWH(6 + k * 37.0, 30, 33, 46);
    c.drawRect(r, st(kDim));
    for (var j = 0; j < 4; j++) {
      c.drawRect(Rect.fromLTWH(r.left + 3 + j * 8, 24, 3, 3), st(kDim, 1));
      c.drawRect(Rect.fromLTWH(r.left + 3 + j * 8, 79, 3, 3), st(kDim, 1));
    }
    final srcF = ((p.ph * 2) - (3 - k) * v).floor();
    final n = ((srcF % 100) + 100) % 100;
    num(c, d2(n), Offset(r.center.dx, 44), 16, k == 3 ? kW : (v < 0 ? kR : kB), ax: .5);
  }
  lab(c, 'OUT 1  2  3  4', const Offset(8, 96), kMid, size: 7);
  lab(c, 'SRC FRAME', const Offset(8, 8), kMid, size: 7);
  if (v == 0) tag(c, 'HOLD', const Offset(148, 8), kR, ax: 1);
  if (v < 0) tag(c, 'REV', const Offset(148, 8), kR, ax: 1);
  if (v > 0) num(c, f1(v), const Offset(148, 96), 11, kB, ax: 1);
}

void _platter(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const o = Offset(66, 58);
  for (var r = 14.0; r <= 50; r += 6) {
    c.drawOval(Rect.fromCenter(center: o, width: r * 2, height: r * 1.1), st(r > 46 ? kW : kDim, 1));
  }
  final a = p.ph * math.pi * 2 * .6;
  dot(c, pol(o, 9, a, .55), kR, 2);
  dot(c, o, kW, 1.5);
  pl(c, [const Offset(140, 16), const Offset(138, 46), const Offset(100, 62)], kB);
  ring(c, const Offset(140, 16), 4, kB);
  final pts = [for (var i = 0; i <= 40; i++) Offset(10 + i * 3.4, 104 + math.sin(i * .35 * (1 + v.abs()) - p.ph * 6) * (2 + v.abs() * 1.5))];
  pl(c, pts, kG);
  readout(c, const Offset(8, 8), v, h: 12);
  lab(c, 'RUB', const Offset(148, 106), kMid, size: 7, ax: 1);
}

double _curveAt(List<Offset> pts, double u) {
  if (pts.length < 2) return .35 - .25 * math.cos(u * math.pi * 2);
  var best = pts.first;
  for (final q in pts) {
    if ((q.dx - u).abs() < (best.dx - u).abs()) best = q;
  }
  return best.dy;
}

void _drawn(Canvas c, Size s, OpP p) {
  for (var x = 8.0; x < s.width; x += 10) {
    for (var y = 8.0; y < s.height; y += 10) {
      c.drawRect(Rect.fromLTWH(x, y, .8, .8), fl(kDim));
    }
  }
  dots(c, const Offset(4, 80), Offset(s.width - 4, 80), kW, 4);
  lab(c, 'HOLD', const Offset(6, 83), kMid, size: 7);
  lab(c, 'REV', const Offset(6, 108), kR, size: 7);
  final line = p.pts.length >= 2 ? [for (final q in p.pts) Offset(q.dx * s.width, q.dy * s.height)] : [for (var i = 0; i <= 30; i++) Offset(i / 30 * s.width, _curveAt(const [], i / 30) * s.height)];
  pl(c, line, kB, w: 1.4);
  final u = frac(p.t * .25), y = _curveAt(p.pts, u);
  dots(c, Offset(u * s.width, 0), Offset(u * s.width, s.height), kDim);
  dot(c, Offset(u * s.width, y * s.height), kW, 2.6);
  final v = (80 / s.height - y) * 8;
  readout(c, const Offset(148, 8), v.abs() < .2 ? 0 : v, h: 12, ax: 1);
}

void _planet(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const o = Offset(70, 62);
  ring(c, o, 9, kW);
  for (var k = 0; k < 8; k++) {
    ln(c, pol(o, 12, k * .785), pol(o, 15, k * .785), kW, 1);
  }
  c.drawOval(Rect.fromCenter(center: o, width: 124, height: 58), st(kDim, 1));
  final a = p.ph * 1.2;
  final trail = [for (var i = 0; i <= 16; i++) pol(o, 62, a - i / 16 * v * .8, 29 / 62)];
  pl(c, trail, spc(v), w: 1.4);
  final pp = pol(o, 62, a, 29 / 62);
  ring(c, pp, 5, kB);
  dot(c, pol(pp, 11, p.ph * 4), kG, 1.5);
  readout(c, const Offset(8, 8), v, h: 12);
}

double _beat(double x) {
  final u = frac(x);
  if (u < .1) return math.sin(u / .1 * math.pi) * .15;
  if (u < .14) return -.2;
  if (u < .18) return 1;
  if (u < .22) return -.4;
  if (u > .35 && u < .5) return math.sin((u - .35) / .15 * math.pi) * .25;
  return 0;
}

void _ecg(Canvas c, Size s, OpP p) {
  final v = spd(p), bpm = (72 * v.abs()).round();
  num(c, '$bpm', const Offset(10, 10), 28, v == 0 ? kW : (v < 0 ? kR : kB));
  lab(c, v == 0 ? 'HOLD' : 'BPM', const Offset(10, 44), v == 0 ? kR : kMid, size: 7);
  final pts = [for (var i = 0; i <= 78; i++) Offset(4 + i * 1.9, 88 - _beat(i / 78 * 2.2 - p.ph * .6) * 28)];
  pl(c, pts, kG);
  for (var x = 4.0; x < s.width; x += 30) {
    dots(c, Offset(x, 56), Offset(x, 116), kDim, 4);
  }
}

void _drummer(Canvas c, Size s, OpP p) {
  final v = spd(p), hitL = frac(p.ph * 1.5), hitR = frac(p.ph * 1.5 + .5);
  const head = Offset(60, 36);
  ring(c, head, 10, kW);
  ring(c, head + const Offset(-11, -2), 4, kW);
  ring(c, head + const Offset(11, -2), 4, kW);
  c.drawOval(Rect.fromCenter(center: head + const Offset(0, 4), width: 11, height: 7), st(kW, 1));
  dot(c, head + const Offset(-3.5, -3), kW, 1.2);
  dot(c, head + const Offset(3.5, -3), kW, 1.2);
  ln(c, head + const Offset(0, 10), const Offset(60, 74), kW);
  void arm(double sg, double h) {
    final sh = Offset(60 + sg * 5, 52), lift = math.pow(1 - h, 3).toDouble();
    final hand = sh + Offset(sg * 14, 6 - lift * 14);
    pl(c, [sh, hand, hand + Offset(sg * 14, 8 - lift * 22)], sg < 0 ? kB : kG);
    if (h < .12 && v != 0) {
      for (var k = 0; k < 3; k++) {
        arc(c, hand + Offset(sg * 18, 12), 5.0 + k * 4, -math.pi / 2 - .5, 1, al(kR, 1 - h * 6));
      }
    }
  }

  arm(-1, hitL);
  arm(1, hitR);
  c.drawOval(Rect.fromCenter(center: const Offset(34, 76), width: 26, height: 8), st(kB));
  c.drawOval(Rect.fromCenter(center: const Offset(88, 72), width: 30, height: 6), st(kG));
  ln(c, const Offset(88, 75), const Offset(88, 110), kDim);
  c.drawOval(Rect.fromCenter(center: const Offset(60, 98), width: 38, height: 14), st(kW));
  readout(c, const Offset(148, 10), v, h: 18, ax: 1);
}

void _conveyor(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const o = Offset(26, 52);
  const k = 1.0;
  isoBox(c, o, 0, 0, 0, 100, 22, 6, kDim, k);
  for (var i = 0; i < 6; i++) {
    final x = i * 20.0;
    final q = iso(o, x, 22, 0, k);
    c.drawOval(Rect.fromCenter(center: q + const Offset(0, 3), width: 6, height: 8), st(kDim, 1));
  }
  for (var i = 0; i < 4; i++) {
    final x = frac(i / 4 + p.ph * .18) * 88;
    isoBox(c, o, x, 4, 6, 12, 12, 12, i == 0 ? kW : kB, k);
  }
  for (var i = 0; i < 5; i++) {
    final x = frac(i / 5 + p.ph * .18) * 100;
    final q = iso(o, x, 11, 6.1, k), dir = v < 0 ? -1.0 : 1.0;
    pl(c, [q + Offset(-3 * dir, -2), q, q + Offset(-3 * dir, 2)], v == 0 ? kDim : (v < 0 ? kR : kG), w: 1);
  }
  readout(c, const Offset(8, 8), v, h: 14);
}

void _wiring(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const n = 16;
  double xs(int i) => 8 + i * 140 / (n - 1);
  final base = (p.ph * 2).floor();
  for (var i = 0; i < n; i++) {
    ln(c, Offset(xs(i), 18), Offset(xs(i), 24), kDim);
    ln(c, Offset(xs(i), 96), Offset(xs(i), 102), kW);
  }
  for (var i = 0; i < n; i++) {
    final src = (((base + (i * v).floor()) % n) + n) % n;
    ln(c, Offset(xs(i), 96), Offset(xs(src), 24), al(spc(v), .9), 1);
  }
  lab(c, 'SRC', const Offset(8, 6), kMid, size: 7);
  lab(c, 'OUT', const Offset(8, 106), kMid, size: 7);
  readout(c, const Offset(148, 106), v, h: 10, ax: 1);
}

void _giant(Canvas c, Size s, OpP p) {
  final v = spd(p);
  final str = v == 0 ? '0' : f1(v.abs());
  final dir = v < 0 ? 1.0 : -1.0;
  for (var k = 5; k >= 1; k--) {
    num(c, str, Offset(78 + dir * k * v.abs() * 2.2, 22), 70, al(kPu, .5 - k * .08), w: 1, ax: .5);
  }
  num(c, str, const Offset(78, 22), 70, spc(v), w: 1.2, ax: .5);
  if (v == 0) tag(c, 'HOLD', const Offset(148, 6), kR, ax: 1);
  if (v < 0) tag(c, 'REV', const Offset(148, 6), kR, ax: 1);
}

void _faucet(Canvas c, Size s, OpP p) {
  final v = spd(p);
  pl(c, const [Offset(10, 14), Offset(70, 14), Offset(80, 22), Offset(80, 30)], kW);
  pl(c, const [Offset(10, 22), Offset(66, 22), Offset(72, 28), Offset(72, 30)], kW);
  ln(c, const Offset(70, 30), const Offset(82, 30), kW);
  ln(c, const Offset(36, 14), const Offset(36, 6), kG);
  ln(c, const Offset(28, 6), const Offset(44, 6), kG);
  for (var k = 0; k < 3; k++) {
    final u = frac(k / 3 + p.ph * .45), y = 34 + u * 66;
    final path = Path()
      ..moveTo(76, y - 6)
      ..quadraticBezierTo(80, y, 76, y + 2)
      ..quadraticBezierTo(72, y, 76, y - 6);
    c.drawPath(path, st(kB));
  }
  final r = frac(p.ph * .45);
  for (var k = 0; k < 2; k++) {
    final rr = (r + k * .5) % 1;
    c.drawOval(Rect.fromCenter(center: const Offset(76, 106), width: 8 + rr * 50, height: 2 + rr * 8), st(al(kG, 1 - rr), 1));
  }
  ln(c, const Offset(20, 106), const Offset(132, 106), kDim);
  readout(c, const Offset(148, 40), v, h: 20, ax: 1);
}

void _tunnel(Canvas c, Size s, OpP p) {
  final v = spd(p);
  const o = Offset(78, 60);
  for (var i = 0; i < 14; i++) {
    final z = frac(i / 14 + p.ph * .22), k = z * z * z;
    c.save();
    c.translate(o.dx, o.dy);
    c.rotate(z * .9);
    final r = Rect.fromCenter(center: Offset.zero, width: 4 + k * 230, height: 3 + k * 180);
    c.drawRect(r, st(al(i.isEven ? kB : kPu, .25 + z), .6 + z * 1.2));
    c.restore();
  }
  dot(c, o, kW, 1.5);
  readout(c, const Offset(8, 8), v, h: 12);
}

void _frog(Canvas c, Size s, OpP p) {
  final v = spd(p), u = frac(p.ph * .4), hop = frac(u * 3), seg = (u * 3).floor();
  final pads = [20.0, 64.0, 108.0, 148.0];
  for (final x in pads) {
    c.drawOval(Rect.fromCenter(center: Offset(x, 100), width: 30, height: 7), st(kG));
  }
  final x0 = pads[seg], x1 = pads[seg + 1];
  final arcPts = [for (var i = 0; i <= 12; i++) Offset(lerp(x0, x1, i / 12), 92 - 50 * math.sin(i / 12 * math.pi))];
  for (final q in arcPts) {
    c.drawRect(Rect.fromCenter(center: q, width: 1, height: 1), fl(kDim));
  }
  final fx = lerp(x0, x1, hop), fy = 92 - 50 * math.sin(hop * math.pi), ext = math.sin(hop * math.pi);
  final f = Offset(fx, fy);
  arc(c, f, 9, math.pi, math.pi, kW);
  ln(c, f + const Offset(-9, 0), f + const Offset(9, 0), kW);
  ring(c, f + const Offset(-5, -9), 3, kW);
  ring(c, f + const Offset(5, -9), 3, kW);
  pl(c, [f + const Offset(-6, 0), f + Offset(-10 - ext * 6, 4 + ext * 6), f + Offset(-4 - ext * 10, 6 + ext * 10)], kB);
  pl(c, [f + const Offset(6, 0), f + Offset(10 + ext * 6, 4 + ext * 6), f + Offset(4 + ext * 10, 6 + ext * 10)], kB);
  readout(c, const Offset(8, 8), v, h: 12);
}
