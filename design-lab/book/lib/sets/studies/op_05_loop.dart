// OP 05 sheet 2, Loop. v[0] = length (0.5..3 s), v[1] = offset (wraps), m = mode (tap): CYCLE, PING, ONCE.
// Encoder colours: length blue, offset green, mode white tag, red marks the seam where the loop restarts.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'op_05_kit.dart';

const _modes = ['CYCLE', 'PING', 'ONCE'];
double lenS(OpP p) => lerp(.5, 3, p.a);

/// Position inside the loop, 0..1.
double lu(OpP p) {
  final l = lenS(p), x = p.t / l + p.b;
  return switch (p.m) { 1 => tri(x * .5), 2 => (p.b + (p.t % (l + 1.2)) / l).clamp(0.0, 1.0), _ => frac(x) };
}

void modeTag(Canvas c, OpP p, Offset o, {double ax = 0}) => tag(c, _modes[p.m], o, kW, ax: ax);
void lenNum(Canvas c, OpP p, Offset o, {double h = 12, double ax = 0}) => num(c, f1(lenS(p)), o, h, kB, ax: ax);
void offNum(Canvas c, OpP p, Offset o, {double h = 12, double ax = 0}) => num(c, '${(p.b * 100).round()}', o, h, kG, ax: ax);

List<OpSpec> loopPanels() => [
      OpSpec('Spring turns', 'object · mechanism · drag · drawing', _spring, y: 1, wrap: 2, modes: 3),
      OpSpec('Racetrack', 'vehicle · effect · drag · drawing', _track, y: 1, wrap: 2, modes: 3, init: const [.45, .1, 0]),
      OpSpec('Hamster wheel', 'animal · effect · spin · drawing', _hamster, g: G.spin, modes: 3),
      OpSpec('Step lane', 'M4L · mechanism · drag · diagram', _steps, y: 1, wrap: 2, modes: 3, init: const [.5, .2, 0]),
      OpSpec('Tape splice', 'machine · mechanism · drag · drawing', _tape, y: 1, wrap: 2, modes: 3),
      OpSpec('Boomerang', 'object · effect · flick · drawing', _boomerang, g: G.flick, modes: 3, m: 1),
      OpSpec('Ferris start seat', 'landscape · effect · spin · drawing', _ferris, g: G.spin, x: 1, wrap: 2, modes: 3, init: const [.6, .2, 0]),
      OpSpec('Seam tiles', 'sample · effect · drag · diagram', _tiles, y: 1, wrap: 2, modes: 3, init: const [.3, 0, 0]),
      OpSpec('Infinity', 'typographic · effect · pinch · drawing', _infinity, g: G.pinch, modes: 3, m: 1),
      OpSpec('Moon phases', 'cosmic · effect · drag · numeral', _moons, y: 1, wrap: 2, modes: 3),
      OpSpec('Ouroboros', 'animal · mechanism · spin · drawing', _snake, g: G.spin, x: 1, y: 0, wrap: 2, modes: 3),
      OpSpec('Shelf reset', 'sample · effect · drag · drawing', _shelf, y: 1, wrap: 2, modes: 3, init: const [.4, 0, 0]),
      OpSpec('Draw a loop', 'diagram · mechanism · draw · diagram', _drawn, g: G.draw, modes: 3),
      OpSpec('Yo-yo', 'object · effect · drag v · drawing', _yoyo, x: -1, y: 0, modes: 3, m: 1),
      OpSpec('Penrose stairs', 'isometric · effect · drag · pushed', _penrose, y: 1, wrap: 2, modes: 3, init: const [.4, 0, 0]),
      OpSpec('Keyboard phrase', 'instrument · mechanism · drag · diagram', _keys, y: 1, wrap: 2, modes: 3, init: const [.35, .15, 0]),
      OpSpec('Counter', 'typographic · readout · drag v · numeral', _counter, x: -1, y: 0, modes: 3),
      OpSpec('Iso train', 'vehicle · isometric · flick · drawing', _train, g: G.flick, modes: 3),
      OpSpec('Echo rings', 'cosmic · diagram · pinch · pushed', _echo, g: G.pinch, modes: 3, init: const [.25, 0, 0]),
      OpSpec('Kaleido trail', 'diagram · effect · drag · pushed', _kaleido, y: 1, wrap: 2, modes: 3, init: const [.6, .3, 0]),
    ];

void _spring(Canvas c, Size s, OpP p) {
  final turns = 3 + p.a * 9, u = lu(p);
  const x0 = 18.0, x1 = 138.0, cy = 58.0, r = 14.0;
  Offset at(double q) {
    final th = q * turns * math.pi * 2;
    return Offset(lerp(x0, x1, q) - math.sin(th) * 5, cy - math.cos(th) * r);
  }

  pl(c, [for (var i = 0; i <= 260; i++) at(i / 260)], kB);
  final o = at(frac(p.b));
  ln(c, o + const Offset(0, -20), o + const Offset(0, 20), kR);
  dot(c, at(u), kW, 3);
  lab(c, 'TURNS', const Offset(8, 8), kMid, size: 7);
  num(c, '${turns.round()}', const Offset(8, 92), 18, kB);
  offNum(c, p, const Offset(148, 92), h: 18, ax: 1);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _track(Canvas c, Size s, OpP p) {
  final w = lerp(50, 132, p.a), u = lu(p);
  const cy = 60.0, h = 54.0;
  Offset at(double q, double inset) {
    final per = 2 * (w - h) + math.pi * h, d = frac(q) * per, rx = w / 2 - h / 2, rr = h / 2 - inset;
    if (d < w - h) return Offset(78 - rx + d, cy + rr);
    if (d < w - h + math.pi * h / 2) return Offset(78 + rx, cy) + Offset(math.sin((d - (w - h)) / (h / 2)), math.cos((d - (w - h)) / (h / 2))) * rr;
    if (d < 2 * (w - h) + math.pi * h / 2) return Offset(78 + rx - (d - (w - h) - math.pi * h / 2), cy - rr);
    return Offset(78 - rx, cy) + Offset(-math.sin((d - 2 * (w - h) - math.pi * h / 2) / (h / 2)), -math.cos((d - 2 * (w - h) - math.pi * h / 2) / (h / 2))) * rr;
  }

  for (final ins in [0.0, 10.0]) {
    pl(c, [for (var i = 0; i <= 80; i++) at(i / 80, ins)], ins == 0 ? kW : kDim, close: true);
  }
  final f0 = at(p.b, 0), f1p = at(p.b, 10);
  ln(c, f0, f1p, kG, 1.6);
  ln(c, f0, f0 + const Offset(0, -12), kG);
  pl(c, [f0 + const Offset(0, -12), f0 + const Offset(7, -9), f0 + const Offset(0, -6)], kG);
  final q = p.b + u, car = at(q, 5), ahead = at(q + .01, 5);
  c.save();
  c.translate(car.dx, car.dy);
  c.rotate((ahead - car).direction);
  c.drawRect(const Rect.fromLTWH(-5, -3, 10, 6), st(kB));
  c.restore();
  lenNum(c, p, const Offset(8, 104), h: 10);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _hamster(Canvas c, Size s, OpP p) {
  final r = lerp(26, 50, p.a), u = lu(p);
  final o = Offset(70, 108 - r);
  ring(c, o, r, kW);
  ring(c, o, r - 4, kDim, 1);
  final rot = u * math.pi * 2;
  for (var k = 0; k < 8; k++) {
    ln(c, pol(o, 3, rot + k * .785), pol(o, r - 4, rot + k * .785), kDim, 1);
  }
  ln(c, o, Offset(o.dx - 14, 112), kDim);
  ln(c, o, Offset(o.dx + 14, 112), kDim);
  final b = Offset(o.dx, o.dy + r - 19), step = math.sin(p.t * 18);
  c.save();
  c.translate(b.dx, b.dy);
  c.scale(1.6);
  c.translate(-b.dx, -b.dy);
  c.drawOval(Rect.fromCenter(center: b, width: 22, height: 13), st(kB));
  ring(c, b + const Offset(9, -6), 2.5, kB, 1);
  dot(c, b + const Offset(9, -1), kW, 1.2);
  ln(c, b + const Offset(-6, 6), b + Offset(-6 + step * 3, 10), kB, 1);
  ln(c, b + const Offset(5, 6), b + Offset(5 - step * 3, 10), kB, 1);
  c.restore();
  dot(c, pol(o, r - 2, -math.pi / 2 + p.b * math.pi * 2), kR, 2.5);
  lenNum(c, p, const Offset(148, 92), h: 16, ax: 1);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _steps(Canvas c, Size s, OpP p) {
  const n = 16, cw = 8.6, x0 = 9.0;
  final len = 2 + (p.a * 14).round(), st0 = (p.b * n).floor() % n;
  final cur = (st0 + (lu(p) * len).floor().clamp(0, len - 1)) % n;
  for (var i = 0; i < n; i++) {
    final h = 8 + 26 * (.5 + .5 * math.sin(i * 1.7)).abs();
    final inLoop = ((i - st0) % n + n) % n < len;
    final x = x0 + i * cw;
    ln(c, Offset(x + cw / 2, 78), Offset(x + cw / 2, 78 - h), i == cur ? kW : (inLoop ? kB : kDim), 1.4);
    c.drawRect(Rect.fromLTWH(x + 1, 84, cw - 2, cw - 2), st(i == cur ? kW : kDim, 1));
    if (i == cur) c.drawRect(Rect.fromLTWH(x + 3, 86, cw - 6, cw - 6), fl(kW));
  }
  final a = x0 + st0 * cw, b = x0 + ((st0 + len) % n) * cw;
  pl(c, [Offset(a + 3, 98), Offset(a, 98), Offset(a, 108), Offset(a + 3, 108)], kG);
  pl(c, [Offset(b - 3, 98), Offset(b, 98), Offset(b, 108), Offset(b - 3, 108)], kB);
  num(c, '$len', const Offset(8, 8), 18, kB);
  lab(c, 'STEPS', const Offset(8, 30), kMid, size: 7);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _tape(Canvas c, Size s, OpP p) {
  final d = lerp(30, 100, p.a);
  final ps = [const Offset(24, 60), Offset(24 + d, 32), Offset(24 + d, 88)];
  for (final q in ps) {
    ring(c, q, 9, kW);
    ring(c, q, 2, kW, 1);
  }
  final path = [ps[0] + const Offset(0, -9), ps[1] + const Offset(0, -9), ps[1] + const Offset(9, 0), ps[2] + const Offset(9, 0), ps[2] + const Offset(0, 9), ps[0] + const Offset(0, 9)];
  pl(c, path, kB, close: true);
  final lens = <double>[];
  var tot = 0.0;
  for (var i = 0; i < path.length; i++) {
    lens.add((path[(i + 1) % path.length] - path[i]).distance);
    tot += lens.last;
  }
  Offset at(double q) {
    var dd = frac(q) * tot;
    for (var i = 0; i < path.length; i++) {
      if (dd <= lens[i]) return Offset.lerp(path[i], path[(i + 1) % path.length], dd / lens[i])!;
      dd -= lens[i];
    }
    return path.first;
  }

  final sp = at(p.b + lu(p));
  ln(c, sp + const Offset(-3, -4), sp + const Offset(3, 4), kR, 1.6);
  dot(c, at(p.b), kG, 2);
  lenNum(c, p, const Offset(148, 104), h: 10, ax: 1);
  lab(c, 'SPLICE', const Offset(8, 108), kR, size: 7);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _boomerang(Canvas c, Size s, OpP p) {
  final w = lerp(40, 120, p.a), u = lu(p);
  Offset at(double q) => Offset(20 + math.sin(q * math.pi) * w, 70 - math.sin(q * math.pi * 2) * 26);
  for (var i = 0; i <= 40; i++) {
    c.drawRect(Rect.fromCenter(center: at(i / 40), width: 1, height: 1), fl(kDim));
  }
  final q = at(u);
  c.save();
  c.translate(q.dx, q.dy);
  c.rotate(p.t * 14);
  pl(c, const [Offset(-10, -6), Offset(0, 2), Offset(10, -6), Offset(8, -8), Offset(0, -2), Offset(-8, -8)], kB, close: true);
  c.restore();
  pl(c, const [Offset(14, 80), Offset(20, 72), Offset(26, 80)], kW);
  ln(c, const Offset(20, 80), const Offset(20, 108), kW);
  lenNum(c, p, const Offset(148, 92), h: 16, ax: 1);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _ferris(Canvas c, Size s, OpP p) {
  const o = Offset(70, 54);
  const r = 40.0;
  final rot = lu(p) * math.pi * 2 * (.5 + p.a);
  ring(c, o, r, kW);
  ring(c, o, 4, kW);
  pl(c, [const Offset(46, 112), o, const Offset(94, 112)], kDim);
  ln(c, const Offset(20, 112), const Offset(130, 112), kDim);
  for (var k = 0; k < 8; k++) {
    final a = rot + k * math.pi / 4, q = pol(o, r, a);
    ln(c, o, q, kDim, 1);
    final first = k == 0;
    c.drawRect(Rect.fromLTWH(q.dx - 4, q.dy + 2, 8, 7), st(first ? kR : kB, 1));
    ln(c, q, q + const Offset(0, 2), first ? kR : kB, 1);
  }
  offNum(c, p, const Offset(148, 92), h: 16, ax: 1);
  lab(c, 'START', const Offset(148, 82), kMid, size: 7, ax: 1);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

double _ease(double u) => u * u * (3 - 2 * u);

void _tiles(Canvas c, Size s, OpP p) {
  final per = lerp(18, 90, p.a);
  const y0 = 92.0, hh = 60.0;
  final pts = <Offset>[];
  for (var x = 0.0; x <= s.width; x += 1.5) {
    final k = ((x + p.b * per) / per), i = k.floor(), f = frac(k);
    double y;
    if (p.m == 2 && i > 0) {
      y = 1;
    } else if (p.m == 1 && i.isOdd) {
      y = _ease(1 - f);
    } else {
      y = _ease(f);
    }
    pts.add(Offset(x, y0 - y * hh));
  }
  ln(c, Offset(0, y0), Offset(s.width, y0), kDim, 1);
  for (var x = -p.b * per; x < s.width; x += per) {
    if (x > 0) dots(c, Offset(x, 24), Offset(x, y0 + 6), kR, 3);
  }
  pl(c, pts, kB, w: 1.4);
  final px = frac(p.t * .12) * s.width;
  ln(c, Offset(px, 20), Offset(px, 112), al(kW, .5), 1);
  lenNum(c, p, const Offset(8, 102), h: 10);
  offNum(c, p, const Offset(60, 102), h: 10);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _infinity(Canvas c, Size s, OpP p) {
  final a = lerp(26, 66, p.a);
  const o = Offset(78, 56);
  Offset at(double q) {
    final t = q * math.pi * 2, d = 1 + math.sin(t) * math.sin(t);
    return o + Offset(a * math.cos(t) / d, a * math.sin(t) * math.cos(t) / d);
  }

  pl(c, [for (var i = 0; i <= 120; i++) at(i / 120)], kW, close: true);
  final q = p.b + lu(p);
  for (var k = 1; k < 10; k++) {
    dot(c, at(q - k * .012), al(kB, 1 - k * .1), 1.6);
  }
  dot(c, at(q), kB, 3.2);
  dot(c, at(p.b), kR, 2);
  lenNum(c, p, const Offset(78, 98), h: 14, ax: .5);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _moon(Canvas c, Offset o, double r, double ph, Color col) {
  ring(c, o, r, col, 1);
  final k = math.cos(ph * math.pi * 2);
  final path = Path()
    ..moveTo(o.dx, o.dy - r)
    ..arcTo(Rect.fromCenter(center: o, width: 2 * r * k.abs(), height: 2 * r), -math.pi / 2, k > 0 ? math.pi : -math.pi, false);
  c.drawPath(path, st(col, 1));
}

void _moons(Canvas c, Size s, OpP p) {
  final n = 4 + (p.a * 8).round(), u = lu(p), cur = (u * n).floor().clamp(0, n - 1);
  final loops = (p.t / lenS(p)).floor() % 100;
  num(c, d2(loops), const Offset(8, 10), 34, kW);
  lab(c, 'LOOPS', const Offset(8, 50), kMid, size: 7);
  final r = math.min(8.0, 140 / n / 2 - 1.5);
  for (var i = 0; i < n; i++) {
    final x = 8 + r + i * (140 / n);
    _moon(c, Offset(x, 86), r, frac(p.b + i / n), i == cur ? kW : (i == 0 ? kG : kDim));
  }
  num(c, '$n', const Offset(148, 12), 14, kB, ax: 1);
  modeTag(c, p, const Offset(148, 106), ax: 1);
}

void _snake(Canvas c, Size s, OpP p) {
  const o = Offset(64, 60);
  const r = 38.0;
  final head = -math.pi / 2 + p.b * math.pi * 2 + lu(p) * math.pi * 2, body = lerp(math.pi * .8, math.pi * 1.9, p.a);
  final pts = [for (var i = 0; i <= 60; i++) pol(o, r + math.sin(i * .9 + p.t * 4) * 1.5, head - i / 60 * body)];
  pl(c, pts, kG, w: 1.4);
  for (var i = 2; i < 60; i += 4) {
    final a = head - i / 60 * body;
    ln(c, pol(o, r - 4, a), pol(o, r + 4, a), kDim, 1);
  }
  final hp = pol(o, r, head), dir = head + math.pi / 2;
  pl(c, [pol(hp, 6, dir - 1.2), pol(hp, 10, dir), pol(hp, 6, dir + 1.2)], kW, close: true);
  dot(c, pol(hp, 6, dir - .2), kR, 1.4);
  final tail = pol(o, r, head - body);
  dot(c, tail, kW, 1.5);
  lenNum(c, p, const Offset(148, 92), h: 16, ax: 1);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _shelf(Canvas c, Size s, OpP p) {
  final w = lerp(30, 120, p.a), u = lu(p);
  final x0 = 14 + p.b * (136 - w), x1 = x0 + w;
  ln(c, const Offset(8, 80), const Offset(148, 80), kW);
  ln(c, Offset(x0, 84), Offset(x0, 90), kG);
  ln(c, Offset(x1, 84), Offset(x1, 90), kB);
  dots(c, Offset(x0, 96), Offset(x1, 96), kDim, 3);
  final bx = lerp(x0, x1, u);
  ring(c, Offset(bx, 72), 8, kW);
  ln(c, Offset(bx, 72), pol(Offset(bx, 72), 8, u * w / 8), kW, 1);
  if (p.m == 0 && u > .9) {
    final path = Path()
      ..moveTo(x1, 60)
      ..quadraticBezierTo((x0 + x1) / 2, 22, x0, 60);
    c.drawPath(path, st(kR, 1));
  }
  if (p.m == 1) {
    pl(c, [Offset(x0 + 6, 50), Offset(x0, 54), Offset(x0 + 6, 58)], kW, w: 1);
    pl(c, [Offset(x1 - 6, 50), Offset(x1, 54), Offset(x1 - 6, 58)], kW, w: 1);
  }
  lenNum(c, p, const Offset(8, 104), h: 10);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _drawn(Canvas c, Size s, OpP p) {
  final pts = p.pts.length >= 3
      ? [for (final q in p.pts) Offset(q.dx * s.width, q.dy * s.height)]
      : [for (var i = 0; i <= 40; i++) Offset(78 + math.cos(i / 40 * math.pi * 2) * 50 + math.cos(i / 40 * math.pi * 6) * 8, 60 + math.sin(i / 40 * math.pi * 2) * 34)];
  pl(c, pts, kB, close: true);
  var len = 0.0;
  for (var i = 1; i < pts.length; i++) {
    len += (pts[i] - pts[i - 1]).distance;
  }
  final idx = (lu(p) * (pts.length - 1)).floor().clamp(0, pts.length - 1);
  dot(c, pts.first, kR, 2.4);
  ring(c, pts[idx], 4, kW);
  num(c, '${len.round()}', const Offset(148, 104), 10, kB, ax: 1);
  lab(c, 'DRAW', const Offset(8, 108), kMid, size: 7);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _yoyo(Canvas c, Size s, OpP p) {
  final len = lerp(24, 90, p.a), u = p.m == 2 ? math.min(1.0, (p.t % (lenS(p) + 1.2)) / lenS(p)) : lu(p);
  const hand = Offset(70, 14);
  pl(c, const [Offset(56, 4), Offset(62, 14), Offset(80, 14), Offset(86, 6)], kW);
  final y = hand.dy + 6 + u * len;
  ln(c, hand, Offset(70, y), kG, 1);
  ring(c, Offset(70, y + 8), 8, kB);
  ring(c, Offset(70, y + 8), 2, kB, 1);
  ln(c, pol(Offset(70, y + 8), 3, p.t * 12), pol(Offset(70, y + 8), 7, p.t * 12), kB, 1);
  dots(c, const Offset(100, 20), Offset(100, 20 + len + 8), kDim, 3);
  num(c, f1(lenS(p)), const Offset(108, 20), 14, kB);
  modeTag(c, p, const Offset(148, 106), ax: 1);
}

void _penrose(Canvas c, Size s, OpP p) {
  final n = 4 + (p.a * 12).round(), side = (n / 4).ceil();
  const o = Offset(70, 34);
  const k = 9.0;
  final cells = <List<int>>[];
  for (var i = 0; i < side; i++) {
    cells.add([i, 0]);
  }
  for (var i = 0; i < side; i++) {
    cells.add([side, i]);
  }
  for (var i = side; i > 0; i--) {
    cells.add([i, side]);
  }
  for (var i = side; i > 0; i--) {
    cells.add([0, i]);
  }
  final cnt = cells.length, cur = ((p.b + lu(p)) * cnt).floor() % cnt;
  for (var i = 0; i < cnt; i++) {
    final h = .6 + i / cnt * 2.6;
    isoBox(c, o, cells[i][0].toDouble(), cells[i][1].toDouble(), 0, 1, 1, h, i == cur ? kW : (i == 0 ? kR : kB), k);
  }
  final cc = cells[cur];
  dot(c, iso(o, cc[0] + .5, cc[1] + .5, 0.6 + cur / cnt * 2.6, k), kG, 2.6);
  num(c, '$cnt', const Offset(8, 96), 16, kB);
  lab(c, 'STEPS', const Offset(8, 86), kMid, size: 7);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _keys(Canvas c, Size s, OpP p) {
  const n = 14, kw = 10.0, x0 = 8.0, y0 = 48.0;
  final len = 2 + (p.a * 10).round(), st0 = (p.b * n).floor() % n;
  final cur = (st0 + (lu(p) * len).floor().clamp(0, len - 1)) % n;
  for (var i = 0; i < n; i++) {
    final inL = ((i - st0) % n + n) % n < len;
    c.drawRect(Rect.fromLTWH(x0 + i * kw, y0, kw, 50), st(i == cur ? kW : (inL ? kB : kDim), 1));
    if (i == cur) dot(c, Offset(x0 + i * kw + kw / 2, y0 + 42), kW, 2.2);
  }
  for (var i = 0; i < n - 1; i++) {
    if (i % 7 == 2 || i % 7 == 6) continue;
    c.drawRect(Rect.fromLTWH(x0 + i * kw + kw * .65, y0, kw * .7, 30), Paint()..color = kBk);
    c.drawRect(Rect.fromLTWH(x0 + i * kw + kw * .65, y0, kw * .7, 30), st(kDim, 1));
  }
  final a = x0 + st0 * kw, b = x0 + math.min(n, st0 + len) * kw;
  ln(c, Offset(a, 40), Offset(b, 40), kB);
  ln(c, Offset(a, 36), Offset(a, 44), kG, 1.6);
  for (var i = 0; i < len; i++) {
    final x = x0 + ((st0 + i) % n) * kw + kw / 2;
    dot(c, Offset(x, 24 - 8 * math.sin(i * 1.3).abs()), i == (cur - st0 + n) % n ? kW : kDim, 1.4);
  }
  num(c, '$len', const Offset(8, 104), 10, kB);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _counter(Canvas c, Size s, OpP p) {
  final n = 2 + (p.a * 14).round(), i = (lu(p) * n).floor().clamp(0, n - 1) + 1;
  num(c, d2(i), const Offset(10, 14), 74, i == 1 ? kR : kW, w: 1.1);
  lab(c, 'OF', const Offset(148, 76), kMid, size: 7, ax: 1);
  num(c, d2(n), const Offset(148, 86), 16, kB, ax: 1);
  modeTag(c, p, const Offset(148, 6), ax: 1);
  if (p.m == 2 && i == n) tag(c, 'END', const Offset(148, 22), kR, ax: 1);
}

void _train(Canvas c, Size s, OpP p) {
  final rx = lerp(36, 66, p.a);
  const o = Offset(78, 66);
  for (final d in [-5.0, 5.0]) {
    c.drawOval(Rect.fromCenter(center: o, width: (rx + d) * 2, height: (rx + d)), st(kDim, 1));
  }
  for (var i = 0; i < 24; i++) {
    final a = i / 24 * math.pi * 2;
    ln(c, pol(o, rx - 7, a, .5), pol(o, rx + 7, a, .5), kDim, 1);
  }
  final u = lu(p);
  for (var k = 0; k < 3; k++) {
    final a = (u - k * .07) * math.pi * 2, q = pol(o, rx, a, .5);
    c.save();
    c.translate(q.dx, q.dy);
    final tan = pol(Offset.zero, 1, a + math.pi / 2, .5);
    c.rotate(tan.direction);
    c.drawRect(const Rect.fromLTWH(-6, -9, 12, 9), st(k == 0 ? kW : kB));
    c.restore();
  }
  lenNum(c, p, const Offset(8, 8), h: 12);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _echo(Canvas c, Size s, OpP p) {
  const o = Offset(78, 60);
  final l = lenS(p);
  for (var k = 0; k < 18; k++) {
    final age = (p.t % l) + k * l;
    final r = age * 34;
    if (r > 120) break;
    ring(c, o, r, al(k.isEven ? kB : kPu, 1 - r / 120), 1);
  }
  final u = lu(p);
  dot(c, pol(o, 10, -math.pi / 2 + u * math.pi * 2), kW, 2.4);
  ring(c, o, 10, kDim, 1);
  lenNum(c, p, const Offset(8, 8), h: 12);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}

void _kaleido(Canvas c, Size s, OpP p) {
  const o = Offset(78, 60);
  Offset at(double q) => Offset(math.sin(q * math.pi * 2 * 3) * 22 + 20, math.sin(q * math.pi * 2 * 2) * 14);
  final u = lu(p), span = .15 + p.a * .85;
  for (var r = 0; r < 6; r++) {
    c.save();
    c.translate(o.dx, o.dy);
    c.rotate(r * math.pi / 3);
    pl(c, [for (var i = 0; i <= 40; i++) at(p.b + i / 40 * span)], al(r.isEven ? kB : kG, .9));
    dot(c, at(p.b + u * span), kW, 1.8);
    c.restore();
  }
  lenNum(c, p, const Offset(8, 104), h: 10);
  modeTag(c, p, const Offset(148, 6), ax: 1);
}
