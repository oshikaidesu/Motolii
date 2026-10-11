// OP 05 sheet 3, Camera. v[0] = distance (2.2..9 m), v[1] = orbit (wraps, 0..360), v[2] = focal length (14..200 mm, log).
// Encoder colours: distance blue, orbit green, focal white, red = the camera or the subject's key point.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'op_05_kit.dart';

double dist(OpP p) => lerp(2.2, 9, p.a);
double yaw(OpP p) => p.b * math.pi * 2;
double mm(OpP p) => 14 * math.pow(200 / 14, p.c).toDouble();
Cam cam(OpP p, Offset o, {double pitch = .35}) => Cam(yaw: yaw(p), pitch: pitch, dist: dist(p), f: mm(p) * 2.4, o: o);

void distNum(Canvas c, OpP p, Offset o, {double h = 10, double ax = 0}) => num(c, f1(dist(p)), o, h, kB, ax: ax);
void orbNum(Canvas c, OpP p, Offset o, {double h = 10, double ax = 0}) => num(c, '${(p.b * 360).round()}', o, h, kG, ax: ax);
void mmNum(Canvas c, OpP p, Offset o, {double h = 10, double ax = 0}) => num(c, '${mm(p).round()}', o, h, kW, ax: ax);

List<OpSpec> cameraPanels() => [
      OpSpec('Viewfinder', 'the view · effect · drag · drawing', _finder, x: 1, y: 0, wrap: 2, init: const [.3, .12, .28]),
      OpSpec('Plan view', 'diagram · mechanism · spin · diagram', _plan, g: G.spin, x: 1, y: 0, wrap: 2, init: const [.4, .1, .45]),
      OpSpec('Vertigo hall', 'landscape · effect · drag v · pushed', _vertigo, x: -1, y: 0, init: const [.4, 0, .5]),
      OpSpec('Lens barrel', 'machine · mechanism · drag · drawing', _barrel, x: 2, init: const [.5, 0, .5]),
      OpSpec('Skyline orbit', 'landscape · effect · spin · drawing', _skyline, g: G.spin, x: 1, wrap: 2, init: const [.55, .1, .35]),
      OpSpec('Satellite', 'cosmic · mechanism · spin · drawing', _satellite, g: G.spin, x: 1, y: 0, wrap: 2, init: const [.5, .1, .5]),
      OpSpec('Portrait', 'character · effect · drag v · drawing', _portrait, x: -1, y: 0, init: const [.2, .08, .5]),
      OpSpec('Dolly rail', 'isometric · mechanism · drag · drawing', _rail, init: const [.5, 0, .5]),
      OpSpec('Focal numeral', 'typographic · readout · drag v · numeral', _fovNum, x: -1, y: 2, init: const [.5, 0, .4]),
      OpSpec('Barrel grid', 'diagram · effect · pinch · pushed', _bulge, g: G.pinch, x: 2, init: const [.5, 0, .2]),
      OpSpec('Bird circling', 'animal · effect · spin · drawing', _bird, g: G.spin, x: 1, wrap: 2, init: const [.5, .15, .5]),
      OpSpec('Telescope', 'instrument · effect · drag · drawing', _scope, x: 2, init: const [.5, 0, .5]),
      OpSpec('Parallax planes', 'landscape · effect · drag · drawing', _parallax, x: 1, y: 0, wrap: 2, init: const [.4, .5, .5]),
      OpSpec('Draw the path', 'diagram · mechanism · draw · diagram', _drawn, g: G.draw, init: const [.4, 0, .4]),
      OpSpec('Turntable helix', 'machine · effect · flick · drawing', _turntable, g: G.flick, x: 1, wrap: 2, init: const [.4, .1, .45]),
      OpSpec('Distance ruler', 'typographic · effect · drag v · numeral', _ruler, x: -1, y: 0, init: const [.3, 0, .5]),
      OpSpec('Inside the sphere', 'cosmic · effect · pinch · pushed', _sphere, g: G.pinch, init: const [.62, .1, .4]),
      OpSpec('Crop frames', 'diagram · mechanism · drag v · diagram', _crops, x: -1, y: 2, init: const [.5, .1, .45]),
      OpSpec('Eye gaze', 'character · effect · drag · drawing', _eye, x: 1, y: 2, wrap: 2, init: const [.5, .5, .5]),
      OpSpec('Crash zoom', 'character · effect · rub · pushed', _crash, g: G.rub, x: 2, init: const [.5, 0, .3]),
    ];

void _finder(Canvas c, Size s, OpP p) {
  final cm = cam(p, const Offset(78, 62));
  cm.floor(c, 2.4, 6, kDim);
  cm.cube(c, 1, kB);
  cm.seg(c, const V3(0, 1, 0), const V3(0, 1.8, 0), kR);
  const k = 10.0;
  for (final q in [const Offset(8, 8), const Offset(148, 8), const Offset(8, 112), const Offset(148, 112)]) {
    final sx = q.dx < 78 ? 1.0 : -1.0, sy = q.dy < 60 ? 1.0 : -1.0;
    pl(c, [q + Offset(0, sy * k), q, q + Offset(sx * k, 0)], kW);
  }
  ln(c, const Offset(74, 60), const Offset(82, 60), kW, 1);
  ln(c, const Offset(78, 56), const Offset(78, 64), kW, 1);
  distNum(c, p, const Offset(14, 98));
  orbNum(c, p, const Offset(78, 98), ax: .5);
  mmNum(c, p, const Offset(142, 98), ax: 1);
}

void _plan(Canvas c, Size s, OpP p) {
  const o = Offset(60, 60);
  final r = dist(p) * 5.5;
  c.drawRect(Rect.fromCenter(center: o, width: 12, height: 12), st(kW));
  ring(c, o, r, kDim, 1);
  final cp = pol(o, r, yaw(p) + math.pi / 2), look = (o - cp).direction, half = math.atan(18 / mm(p));
  for (final sg in [-1.0, 1.0]) {
    ln(c, cp, pol(cp, r * 1.6, look + sg * half), kW, 1);
  }
  arc(c, cp, 14, look - half, half * 2, kW, 1);
  c.save();
  c.translate(cp.dx, cp.dy);
  c.rotate(look);
  c.drawRect(const Rect.fromLTWH(-9, -4, 8, 8), st(kR));
  pl(c, const [Offset(-1, -2), Offset(3, -4), Offset(3, 4), Offset(-1, 2)], kR, close: true);
  c.restore();
  dots(c, o, cp, kB, 3);
  orbNum(c, p, const Offset(148, 8), h: 16, ax: 1);
  lab(c, 'DEG', const Offset(148, 28), kMid, size: 7, ax: 1);
  distNum(c, p, const Offset(148, 92), h: 12, ax: 1);
  mmNum(c, p, const Offset(148, 108), h: 8, ax: 1);
}

void _vertigo(Canvas c, Size s, OpP p) {
  final d = dist(p), f = d * 30;
  final cm = Cam(yaw: 0, pitch: 0, dist: d, f: f, o: const Offset(78, 62));
  for (var z = 1.0; z <= 12; z += 1.6) {
    final q = [const V3(-2.4, -1.6, 0), const V3(2.4, -1.6, 0), const V3(2.4, 2.2, 0), const V3(-2.4, 2.2, 0)];
    final pts = [for (final v in q) cm.p(V3(v.x, v.y, z))];
    if (pts.any((e) => e == null)) continue;
    pl(c, pts.cast<Offset>(), al(kPu, 1 - z / 14), close: true);
  }
  for (final x in [-2.4, 2.4]) {
    for (final y in [-1.6, 2.2]) {
      cm.seg(c, V3(x, y, 1), V3(x, y, 12), kDim, 1);
    }
  }
  cm.seg(c, const V3(0, -1, 0), const V3(0, .1, 0), kW);
  final head = cm.p(const V3(0, .35, 0));
  if (head != null) ring(c, head, .25 * f / d, kW);
  cm.seg(c, const V3(-.35, -.2, 0), const V3(.35, -.2, 0), kW);
  distNum(c, p, const Offset(8, 8), h: 12);
  num(c, '${(f / 2.4).round()}', const Offset(148, 8), 12, kW, ax: 1);
  lab(c, 'DOLLY + ZOOM', const Offset(8, 108), kMid, size: 7);
}

void _barrel(Canvas c, Size s, OpP p) {
  final f = mm(p), ext = lerp(10, 70, p.c);
  const sensor = 20.0, cy = 58.0;
  ln(c, const Offset(sensor, 30), const Offset(sensor, 86), kR, 1.6);
  final segs = 3;
  for (var i = 0; i < segs; i++) {
    final x0 = sensor + 4 + i * ext / segs, hgt = 34 - i * 4.0;
    c.drawRect(Rect.fromLTWH(x0, cy - hgt / 2, ext / segs + 2, hgt), st(i == segs - 1 ? kW : kDim));
  }
  final lx = sensor + 6 + ext;
  c.drawOval(Rect.fromCenter(center: Offset(lx, cy), width: 6, height: 30), st(kW));
  const objX = 150.0, objH = 18.0;
  ln(c, const Offset(objX, cy), const Offset(objX, cy - objH), kG);
  pl(c, const [Offset(objX - 3, cy - objH + 4), Offset(objX, cy - objH), Offset(objX + 3, cy - objH + 4)], kG);
  final img = objH * (lx - sensor) / (objX - lx);
  ln(c, Offset(objX, cy - objH), Offset(lx, cy), al(kB, .8), 1);
  ln(c, Offset(lx, cy), Offset(sensor, cy + img), al(kB, .8), 1);
  ln(c, Offset(sensor, cy), Offset(sensor, cy + img), kG, 2);
  num(c, '${f.round()}', const Offset(148, 92), 18, kW, ax: 1);
  lab(c, 'MM', const Offset(148, 82), kMid, size: 7, ax: 1);
}

void _skyline(Canvas c, Size s, OpP p) {
  final cm = cam(p, const Offset(78, 74), pitch: .28);
  const blds = [[-1.6, -1.2, .5, 1.6], [-.4, -1.4, .45, 2.6], [.9, -1.0, .5, 1.2], [-1.2, .6, .4, 2.0], [.5, .8, .55, 3.2], [1.6, .2, .35, .9]];
  for (var i = 0; i < blds.length; i++) {
    final b = blds[i];
    final x = b[0], z = b[1], w = b[2], h = b[3];
    final v = [for (var k = 0; k < 8; k++) V3(x + ((k & 1) == 0 ? -w : w), (k & 2) == 0 ? -1.0 : -1 + h, z + ((k & 4) == 0 ? -w : w))];
    for (final e in const [[0, 1], [2, 3], [4, 5], [6, 7], [0, 2], [1, 3], [4, 6], [5, 7], [0, 4], [1, 5], [2, 6], [3, 7]]) {
      cm.seg(c, v[e[0]], v[e[1]], i == 4 ? kG : kB, 1);
    }
  }
  cm.floor(c, 2.4, 4, kDim);
  orbNum(c, p, const Offset(8, 8), h: 14);
  lab(c, 'ORBIT', const Offset(8, 26), kMid, size: 7);
}

void _satellite(Canvas c, Size s, OpP p) {
  const o = Offset(60, 64);
  ring(c, o, 22, kW);
  for (var k = 0; k < 4; k++) {
    final ph = frac(k / 4 + p.t * .03) * math.pi;
    final w = 44 * math.cos(ph).abs();
    c.drawOval(Rect.fromCenter(center: o, width: w, height: 44), st(kDim, 1));
  }
  c.drawOval(Rect.fromCenter(center: o, width: 44, height: 10), st(kDim, 1));
  final r = 26 + dist(p) * 5;
  c.drawOval(Rect.fromCenter(center: o, width: r * 2, height: r * .7), st(kDim, 1));
  final sp = pol(o, r, yaw(p), .35);
  dots(c, sp, o, kG, 3);
  c.drawRect(Rect.fromCenter(center: sp, width: 6, height: 6), st(kR));
  ln(c, sp + const Offset(-13, 0), sp + const Offset(-4, 0), kB);
  ln(c, sp + const Offset(4, 0), sp + const Offset(13, 0), kB);
  c.drawRect(Rect.fromCenter(center: sp + const Offset(-10, 0), width: 6, height: 8), st(kB, 1));
  c.drawRect(Rect.fromCenter(center: sp + const Offset(10, 0), width: 6, height: 8), st(kB, 1));
  orbNum(c, p, const Offset(148, 8), h: 14, ax: 1);
  distNum(c, p, const Offset(148, 104), ax: 1);
}

void _portrait(Canvas c, Size s, OpP p) {
  final d = lerp(1.3, 9, p.a), f = d * 52;
  final cm = Cam(yaw: (p.b - .08) * 2, pitch: .05, dist: d, f: f, o: const Offset(70, 60));
  List<V3> ringPts(double y, double rx, double rz, int n) => [for (var i = 0; i <= n; i++) V3(math.cos(i / n * math.pi * 2) * rx, y, math.sin(i / n * math.pi * 2) * rz)];
  final outline = <Offset>[];
  for (var i = 0; i <= 36; i++) {
    final a = i / 36 * math.pi * 2;
    final q = cm.p(V3(math.cos(a) * .52, math.sin(a) * .7, 0));
    if (q != null) outline.add(q);
  }
  pl(c, outline, kW, close: true);
  for (final y in [-.2, .2]) {
    final pts = [for (final v in ringPts(y, .5, .5, 24)) cm.p(v)].whereType<Offset>().toList();
    pl(c, pts, kDim, w: 1);
  }
  final nose = [cm.p(const V3(0, .15, -.5)), cm.p(const V3(0, -.1, -.78)), cm.p(const V3(.06, -.15, -.52))];
  if (!nose.contains(null)) pl(c, nose.cast<Offset>(), kR);
  for (final x in [-.2, .2]) {
    final e = cm.p(V3(x, .18, -.45));
    if (e != null) ring(c, e, .05 * f / d, kB);
  }
  final m = [cm.p(const V3(-.16, -.35, -.42)), cm.p(const V3(0, -.38, -.5)), cm.p(const V3(.16, -.35, -.42))];
  if (!m.contains(null)) pl(c, m.cast<Offset>(), kW);
  for (final sx in [-1.0, 1.0]) {
    final ear = cm.p(V3(sx * .53, .05, 0));
    if (ear != null) arc(c, ear, .1 * f / d, sx < 0 ? math.pi / 2 : -math.pi / 2, math.pi, kG);
  }
  distNum(c, p, const Offset(148, 8), h: 14, ax: 1);
  num(c, '${(f / 2.4 / 2).round()}', const Offset(148, 104), 10, kW, ax: 1);
}

void _rail(Canvas c, Size s, OpP p) {
  const o = Offset(70, 34);
  const k = 9.0;
  pl(c, [iso(o, 0, 0, 0, k), iso(o, 10, 0, 0, k), iso(o, 10, 5, 0, k), iso(o, 0, 5, 0, k)], kDim, close: true);
  ln(c, iso(o, 0, 0, 0, k), iso(o, 0, 0, 5, k), kDim);
  ln(c, iso(o, 0, 5, 0, k), iso(o, 0, 5, 5, k), kDim);
  ln(c, iso(o, 0, 0, 5, k), iso(o, 0, 5, 5, k), kDim);
  ln(c, iso(o, 10, 0, 0, k), iso(o, 10, 0, 5, k), kDim);
  ln(c, iso(o, 0, 0, 5, k), iso(o, 10, 0, 5, k), kDim);
  isoBox(c, o, .5, 2, 0, 1.4, 1.4, 1.4, kG, k);
  for (final y in [2.3, 3.1]) {
    ln(c, iso(o, 2.5, y, 0, k), iso(o, 9.8, y, 0, k), kB);
  }
  for (var x = 3.0; x < 9.8; x += .8) {
    ln(c, iso(o, x, 2.1, 0, k), iso(o, x, 3.3, 0, k), kDim, 1);
  }
  final cx = lerp(2.6, 8.6, p.a);
  isoBox(c, o, cx, 2.2, .4, 1.2, 1, .9, kR, k);
  final lens = iso(o, cx, 2.7, .85, k);
  c.drawOval(Rect.fromCenter(center: lens, width: 5, height: 8), st(kR));
  dots(c, lens, iso(o, 1.2, 2.7, .7, k), al(kW, .6), 3);
  distNum(c, p, const Offset(8, 104), h: 12);
}

void _fovNum(Canvas c, Size s, OpP p) {
  final f = mm(p), half = math.atan(18 / f);
  num(c, '${f.round()}', const Offset(10, 8), 50, kW, w: 1.1);
  lab(c, 'MM', const Offset(148, 10), kMid, size: 7, ax: 1);
  const eye = Offset(78, 116);
  for (var i = -4; i <= 4; i++) {
    final a = -math.pi / 2 + i / 4 * half;
    ln(c, eye, pol(eye, 50, a), i.abs() == 4 ? kW : kDim, i.abs() == 4 ? 1.2 : .8);
  }
  arc(c, eye, 40, -math.pi / 2 - half, half * 2, kB, 1);
  dot(c, eye, kR, 2);
}

void _bulge(Canvas c, Size s, OpP p) {
  final k = (1 - p.c) * 1.4;
  const o = Offset(78, 60);
  Offset warp(double x, double y) {
    final r2 = x * x + y * y, m = 1 + k * r2 * .35;
    return o + Offset(x * 70 / m * (1 + k * .35), y * 70 / m * (1 + k * .35));
  }

  for (var i = -6; i <= 6; i++) {
    final u = i / 6;
    pl(c, [for (var j = -12; j <= 12; j++) warp(u, j / 12 * .85)], i == 0 ? kB : al(kB, .55), w: 1);
    if (i.abs() * .85 <= 6 * .85) pl(c, [for (var j = -12; j <= 12; j++) warp(j / 12, u * .85)], i == 0 ? kG : al(kG, .45), w: 1);
  }
  mmNum(c, p, const Offset(148, 104), h: 10, ax: 1);
}

void _bird(Canvas c, Size s, OpP p) {
  const o = Offset(70, 70);
  ln(c, const Offset(70, 108), const Offset(70, 70), kW);
  for (final r in [[0.0, 18.0, 16.0], [-12.0, 6.0, 12.0], [12.0, 6.0, 12.0], [0.0, -8.0, 12.0]]) {
    ring(c, o + Offset(r[0], -r[1] - 10), r[2], kG);
  }
  ln(c, const Offset(20, 108), const Offset(120, 108), kDim);
  final a = yaw(p) + p.t * .2, ry = 52.0, front = math.sin(a) > 0;
  final bp = pol(o + const Offset(0, -24), ry, a, .3), sc = front ? 1.9 : 1.1, flap = math.sin(p.t * 9) * 4 * sc;
  final path = Path()
    ..moveTo(bp.dx - 8 * sc, bp.dy - flap)
    ..quadraticBezierTo(bp.dx - 4 * sc, bp.dy - 4 * sc, bp.dx, bp.dy)
    ..quadraticBezierTo(bp.dx + 4 * sc, bp.dy - 4 * sc, bp.dx + 8 * sc, bp.dy - flap);
  c.drawPath(path, st(front ? kR : al(kR, .6)));
  c.drawOval(Rect.fromCenter(center: o + const Offset(0, -24), width: ry * 2, height: ry * .6), st(kDim, .8));
  orbNum(c, p, const Offset(148, 8), h: 14, ax: 1);
}

void _scope(Canvas c, Size s, OpP p) {
  final ext = lerp(8, 40, p.c);
  c.save();
  c.translate(14, 96);
  c.rotate(-.5);
  c.drawRect(Rect.fromLTWH(0, -6, 26, 12), st(kW));
  c.drawRect(Rect.fromLTWH(26, -5, ext, 10), st(kB));
  c.drawRect(Rect.fromLTWH(26 + ext, -7, 8, 14), st(kW));
  c.restore();
  pl(c, const [Offset(22, 92), Offset(14, 116), Offset(30, 116)], kDim);
  const o = Offset(108, 54);
  ring(c, o, 40, kW);
  final r = lerp(6, 60, p.c);
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: o, radius: 39)));
  ring(c, o + Offset(r * .2, -r * .1), r, kG);
  for (final cr in [[.3, .2, .18], [-.35, -.3, .12], [-.1, .45, .1], [.5, -.4, .08]]) {
    ring(c, o + Offset(r * .2 + cr[0] * r, -r * .1 + cr[1] * r), cr[2] * r, kG, 1);
  }
  c.restore();
  mmNum(c, p, const Offset(8, 8), h: 14);
}

void _parallax(Canvas c, Size s, OpP p) {
  final sh = (p.b - .5) * 2, z = lerp(1.6, .7, p.a);
  pl(c, [for (var i = 0; i <= 12; i++) Offset(-10 + i * 15 + sh * 4, 50 - [6, 18, 10, 24, 14, 8, 20, 12, 26, 10, 16, 6, 12][i].toDouble())], kPu);
  for (var i = 0; i < 6; i++) {
    final x = 4 + i * 30 + sh * 18;
    pl(c, [Offset(x - 8, 82), Offset(x, 60), Offset(x + 8, 82)], kB, close: true);
    ln(c, Offset(x, 82), Offset(x, 88), kB);
  }
  ln(c, const Offset(0, 88), const Offset(156, 88), kDim);
  final fx = 78 + sh * 50;
  c.save();
  c.translate(fx, 104);
  c.scale(z);
  ring(c, const Offset(0, -38), 6, kW);
  ln(c, const Offset(0, -32), const Offset(0, -12), kW);
  pl(c, const [Offset(-8, 0), Offset(0, -12), Offset(8, 0)], kW);
  pl(c, const [Offset(-9, -20), Offset(0, -28), Offset(9, -20)], kW);
  c.restore();
  orbNum(c, p, const Offset(8, 8), h: 12);
  distNum(c, p, const Offset(148, 8), h: 12, ax: 1);
}

void _drawn(Canvas c, Size s, OpP p) {
  const o = Offset(56, 60);
  final path = p.pts.length >= 3
      ? [for (final q in p.pts) Offset(q.dx * s.width, q.dy * s.height)]
      : [for (var i = 0; i <= 40; i++) o + Offset(math.cos(i / 40 * math.pi * 2) * 40, math.sin(i / 40 * math.pi * 2) * 30 + math.sin(i / 40 * math.pi * 4) * 8)];
  for (var i = 0; i < path.length; i += 2) {
    c.drawRect(Rect.fromCenter(center: path[i], width: 1.2, height: 1.2), fl(kB));
  }
  c.drawRect(Rect.fromCenter(center: o, width: 10, height: 10), st(kW));
  final idx = (frac(p.t * .15) * path.length).floor().clamp(0, path.length - 1), cp = path[idx];
  ln(c, cp, o, al(kG, .7), 1);
  dot(c, cp, kR, 3);
  final rel = cp - o;
  final cm = Cam(yaw: -rel.direction - math.pi / 2, pitch: .3, dist: 2 + rel.distance / 14, f: 70, o: const Offset(128, 26));
  c.drawRect(const Rect.fromLTWH(104, 6, 48, 40), st(kDim, 1));
  c.save();
  c.clipRect(const Rect.fromLTWH(104, 6, 48, 40));
  cm.cube(c, .8, kW);
  c.restore();
  lab(c, 'DRAW PATH', const Offset(8, 108), kMid, size: 7);
}

void _turntable(Canvas c, Size s, OpP p) {
  final cm = cam(p, const Offset(78, 66), pitch: .45);
  for (var i = 0; i <= 2; i++) {
    final r = 1.6 - i * .02;
    final pts = [for (var k = 0; k <= 36; k++) cm.p(V3(math.cos(k / 36 * math.pi * 2) * r, -1 - i * .12, math.sin(k / 36 * math.pi * 2) * r))].whereType<Offset>().toList();
    pl(c, pts, i == 0 ? kW : kDim, w: 1);
  }
  final helix = <Offset>[], spine = <Offset>[];
  for (var k = 0; k <= 90; k++) {
    final t = k / 90, a = t * math.pi * 2 * 3, r = .9 * (1 - t * .6);
    final q = cm.p(V3(math.cos(a) * r, -1 + t * 2.4, math.sin(a) * r));
    if (q != null) helix.add(q);
  }
  for (final y in [-1.0, 1.4]) {
    final q = cm.p(V3(0, y, 0));
    if (q != null) spine.add(q);
  }
  pl(c, spine, kDim, w: 1);
  pl(c, helix, kB, w: 1.2);
  final mark = cm.p(const V3(1.6, -1, 0));
  if (mark != null) dot(c, mark, kR, 2.2);
  orbNum(c, p, const Offset(8, 8), h: 14);
  lab(c, 'FLICK', const Offset(148, 108), kMid, size: 7, ax: 1);
}

void _ruler(Canvas c, Size s, OpP p) {
  final d = dist(p);
  num(c, f1(d), const Offset(8, 10), 34, kB);
  lab(c, 'M', const Offset(8, 50), kMid, size: 7);
  const vp = Offset(150, 62);
  dots(c, const Offset(60, 116), vp, kDim, 3);
  ln(c, const Offset(40, 116), const Offset(150, 116), kDim, 1);
  for (var i = 1; i <= 9; i++) {
    final t = 1 - 1 / (1 + i * .45), x = lerp(60, 150, t), y = lerp(116, 62, t);
    ln(c, Offset(x, y), Offset(x, y + 4), i.isEven ? kW : kDim, 1);
  }
  final t = 1 - 1 / (1 + (d - 1) * .45), x = lerp(60, 150, t), y = lerp(116, 62, t), sc = 1 - t * .85;
  c.save();
  c.translate(x, y);
  c.scale(sc);
  ring(c, const Offset(0, -50), 7, kW, 1.2 / sc);
  ln(c, const Offset(0, -43), const Offset(0, -16), kW, 1.2 / sc);
  pl(c, const [Offset(-10, 0), Offset(0, -16), Offset(10, 0)], kW, w: 1.2 / sc);
  pl(c, const [Offset(-11, -26), Offset(0, -36), Offset(11, -26)], kW, w: 1.2 / sc);
  c.restore();
  dot(c, Offset(x, y), kR, 1.6);
}

void _sphere(Canvas c, Size s, OpP p) {
  final d = lerp(9, 1.08, p.a);
  final cm = Cam(yaw: yaw(p) + p.t * .15, pitch: .3, dist: d, f: 90, o: const Offset(78, 60));
  for (var i = 1; i < 8; i++) {
    final lat = -math.pi / 2 + i / 8 * math.pi;
    final pts = <Offset>[];
    for (var k = 0; k <= 40; k++) {
      final q = cm.p(V3(math.cos(lat) * math.cos(k / 40 * math.pi * 2), math.sin(lat), math.cos(lat) * math.sin(k / 40 * math.pi * 2)));
      if (q != null) pts.add(q);
    }
    pl(c, pts, i == 4 ? kG : kB, w: 1);
  }
  for (var m = 0; m < 10; m++) {
    final lon = m / 10 * math.pi * 2;
    final pts = <Offset>[];
    for (var k = 0; k <= 30; k++) {
      final lat = -math.pi / 2 + k / 30 * math.pi;
      final q = cm.p(V3(math.cos(lat) * math.cos(lon), math.sin(lat), math.cos(lat) * math.sin(lon)));
      if (q != null) pts.add(q);
    }
    pl(c, pts, al(kPu, .8), w: 1);
  }
  distNum(c, p, const Offset(8, 8), h: 12);
}

void _crops(Canvas c, Size s, OpP p) {
  const o = Offset(78, 58);
  final cm = Cam(yaw: yaw(p), pitch: .35, dist: dist(p), f: 14 * 2.4 * 1.4, o: o);
  cm.floor(c, 2.4, 6, kDim);
  cm.cube(c, 1, kDim);
  const lens = [18, 24, 35, 50, 85, 135];
  final f = mm(p);
  var best = 0;
  for (var i = 0; i < lens.length; i++) {
    if ((lens[i] - f).abs() < (lens[best] - f).abs()) best = i;
  }
  for (var i = 0; i < lens.length; i++) {
    final w = 150 * 14 / lens[i], h = w * .72;
    final r = Rect.fromCenter(center: o, width: w, height: h);
    c.drawRect(r, st(i == best ? kW : al(kB, .6), i == best ? 1.4 : 1));
    if (r.top > 2) lab(c, '${lens[i]}', Offset(r.left + 2, r.top + 2), i == best ? kW : kB, size: 6.5);
  }
  final w = 150 * 14 / f;
  c.drawRect(Rect.fromCenter(center: o, width: w, height: w * .72), st(kR, 1));
  mmNum(c, p, const Offset(148, 106), h: 9, ax: 1);
}

void _eye(Canvas c, Size s, OpP p) {
  const o = Offset(78, 58);
  final lid = Path()
    ..moveTo(18, 58)
    ..quadraticBezierTo(78, 4, 138, 58)
    ..quadraticBezierTo(78, 112, 18, 58);
  c.drawPath(lid, st(kW, 1.4));
  for (var i = 0; i < 7; i++) {
    final t = .15 + i * .7 / 6, q = Offset(lerp(18, 138, t), 58 - 54 * 4 * t * (1 - t) * .5 - 2);
    ln(c, q, q + Offset((t - .5) * 10, -8), kDim, 1);
  }
  final ang = yaw(p);
  final io = o + Offset(math.cos(ang) * 22, math.sin(ang) * 8);
  c.save();
  c.clipPath(lid);
  ring(c, io, 20, kG);
  for (var k = 0; k < 16; k++) {
    ln(c, pol(io, 6 + 6 * (1 - p.c), k * math.pi / 8), pol(io, 18, k * math.pi / 8), al(kG, .5), .8);
  }
  dot(c, io, kW, 3 + 6 * (1 - p.c));
  c.restore();
  orbNum(c, p, const Offset(8, 104), h: 10);
  mmNum(c, p, const Offset(148, 104), h: 10, ax: 1);
}

void _crash(Canvas c, Size s, OpP p) {
  final z = mm(p) / 30;
  const o = Offset(78, 62);
  for (var i = 0; i < 24; i++) {
    final a = i / 24 * math.pi * 2 + .1;
    final r0 = 30 + 40 / z, len = (z - .5).clamp(0.0, 6.0) * 10;
    if (len > 1) ln(c, pol(o, r0, a), pol(o, r0 + len, a), al(kPu, .7), 1);
  }
  c.save();
  c.translate(o.dx, o.dy + 10 * z);
  c.scale(z);
  final w = 1.3 / z;
  ring(c, const Offset(0, -32), 8, kW, w);
  dot(c, const Offset(-3, -33), kW, 1);
  dot(c, const Offset(3, -33), kW, 1);
  arc(c, const Offset(0, -31), 4, .3, math.pi - .6, kR, w);
  ln(c, const Offset(0, -24), const Offset(0, 0), kW, w);
  pl(c, const [Offset(-12, -10), Offset(0, -18), Offset(12, -10)], kW, w: w);
  pl(c, const [Offset(-8, 16), Offset(0, 0), Offset(8, 16)], kW, w: w);
  c.restore();
  mmNum(c, p, const Offset(8, 8), h: 16);
  lab(c, 'RUB', const Offset(148, 108), kMid, size: 7, ax: 1);
}
