// World B 10: Noise, "the ridge line". A mountain range seen from a lookout: three ridges one behind the other, the near one is the noise.
// A is the loupe in the sky (it magnifies the finest grain of the ridge: roughness), B the green footprints on the ground in front (pull them to walk
// the land past faster or stop it), C the white survey staff (how high the range may rise, its cap is the ceiling), D the orange flag planted in the
// ground (pull it sideways to roll a new seed). A place, not a chart: the land is filled and hatched, the far ridges slide slower.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_b_kit.dart';

class WbTrace extends WbWorld {
  double _phase = 0, _t = 0;

  @override
  String get name => 'Noise ridge';
  @override
  String get silhouette => 'a ridge line seen from a lookout';
  @override
  List<WbSpec> get specs => const [
        WbSpec('rough', 'Roughness', 0, 1, .45, amp: .25),
        WbSpec('drift', 'Drift', 0, 2, .5, amp: .3),
        WbSpec('amount', 'Amount', 0, 100, 50, unit: ' %', digits: 0, amp: .3),
        WbSpec('seed', 'Seed', 1, 999, 3, digits: 0, integer: true, driven: false),
      ];
  @override
  List<List<String>> get zoneIds => const [['rough'], ['drift'], ['amount'], ['seed']];

  @override
  void tick(double t, double dt, Map<String, double> v, Size s) {
    _t = t;
    _phase += (v['drift']! * 1.6 + .03) * dt; // never quite still
  }

  static double _f(double u, double seed, double g, {int from = 0}) {
    var sum = 0.0, norm = 0.0, w = 1.0;
    for (var k = 0; k < 7; k++) {
      final fk = math.pow(1.9, k).toDouble(), ph = wbHash(seed * 7 + k, 1) * 6.283, ph2 = wbHash(seed * 5 + k, 9) * 6.283;
      if (k >= from) sum += w * (math.sin(u * fk + ph) * .7 + math.sin(u * fk * 1.37 + ph2) * .3);
      norm += w;
      w *= g;
    }
    return sum / norm;
  }

  static double _g(double rough) => .2 + .8 * math.pow(rough, 1.1).toDouble();
  static const _ku = .05;

  double _ry(Rect a) => a.bottom - 10; // foot of the near ridge
  double _maxAmp(Rect a) => a.height * .72;
  double _gy(Rect a) => a.bottom - 2; // the ground in front
  double _staffX(Rect a) => a.right - 12;
  double _flagX(Rect a) => a.left + a.width * .56;
  double _loupeR(Rect a) => math.min(17, a.height * .17);
  Offset _loupe(Rect a) => Offset(a.left + _loupeR(a) + 3, a.top + _loupeR(a) + 2);

  double _ridge(double x, int k, Rect a, Map<String, double> v) {
    final par = .5 + .25 * k, ry = _ry(a), mx = _maxAmp(a);
    final f = _f(_phase * par + (x - a.left) * _ku * (.8 + .1 * k), v['seed']! + k * 17, _g(v['rough']!));
    final h = wbClamp(f * .85 + .5, 0, 1), base = ry - (2 - k) * mx * .2, amp = v['amount']! / 100 * mx * (.55 + .225 * k);
    return base - h * amp + (k == 2 ? .7 * math.sin(_t * 1.1 + x * .03) : 0);
  }

  @override
  List<List<Offset>> anchors(Size s, Map<String, double> v) {
    final a = wbArea(s), l = _loupe(a), r = _loupeR(a), top = _ry(a) - v['amount']! / 100 * _maxAmp(a), fx = _flagX(a);
    return [
      [l, for (var k = 0; k < 8; k++) l + Offset(math.cos(k * math.pi / 4), math.sin(k * math.pi / 4)) * r],
      wbSamples(Offset(a.left, _gy(a)), Offset(a.right, _gy(a)), 10),
      wbSamples(Offset(_staffX(a), _ry(a)), Offset(_staffX(a), top), 10),
      [Offset(fx, _gy(a) - 20), Offset(fx + 4, _gy(a) - 17), Offset(fx, _gy(a) - 11)],
    ];
  }

  @override
  Map<String, double> drag(int z, Offset p, Offset p0, Map<String, double> v0, Size s) {
    final a = wbArea(s), d = p - p0;
    switch (z) {
      case 0:
        return {...v0, 'rough': v0['rough']! + (d.dx - d.dy) / 120};
      case 1:
        return {...v0, 'drift': v0['drift']! - d.dx / 80};
      case 2:
        return {...v0, 'amount': v0['amount']! - d.dy / _maxAmp(a) * 100};
      default:
        return {...v0, 'seed': v0['seed']! + ((d.dx - d.dy) / 8).roundToDouble()};
    }
  }

  @override
  void paint(Canvas c, Size s, WbFrame f) {
    final a = wbArea(s), v = f.v, ry = _ry(a), gy = _gy(a);
    const cols = [N.g26, N.g38, N.g63];
    for (var k = 0; k < 3; k++) {
      final path = Path()..moveTo(a.left, a.bottom);
      final line = Path();
      for (var x = a.left; x <= a.right + 1; x += 3) {
        final y = _ridge(x, k, a, v);
        path.lineTo(x, y);
        x == a.left ? line.moveTo(x, y) : line.lineTo(x, y);
      }
      path.lineTo(a.right, a.bottom);
      c.drawPath(path, wbFill(Color.lerp(N.g13, N.g20, k * .3)!));
      c.drawPath(line, wbStroke(k == 2 ? (f.grab == null ? N.g76 : N.g56) : cols[k]));
      if (k == 2) {
        // hatching under the near ridge: the land has weight, drawn in hairlines
        final hp = Paint()
          ..color = N.g26.withValues(alpha: .7)
          ..strokeWidth = 1;
        for (var x = a.left + 2; x <= a.right; x += 5) {
          final y = _ridge(x, 2, a, v);
          c.drawLine(Offset(x, y + 3), Offset(x, math.min(y + 11, ry + 4)), hp);
        }
      }
    }
    // the ground in front and the footprints (B)
    c.drawLine(Offset(a.left, gy), Offset(a.right, gy), wbStroke(N.g26));
    final shift = (_phase / _ku) % 14;
    for (var x = a.left - shift + 14; x < a.right; x += 14) {
      if (x < a.left) continue;
      c.drawLine(Offset(x, gy - 1), Offset(x, gy - 5), wbStroke(f.mark(1, .8), f.wid(1)));
    }
    // the survey staff and the ceiling (C)
    final sx = _staffX(a), top = ry - v['amount']! / 100 * _maxAmp(a);
    final ceil = Paint()
      ..color = N.g26.withValues(alpha: .8)
      ..strokeWidth = 1;
    for (var x = a.left; x < a.right; x += 8) {
      c.drawLine(Offset(x, top), Offset(math.min(x + 3, a.right), top), ceil);
    }
    c.drawLine(Offset(sx, ry), Offset(sx, top), wbStroke(f.mark(2), f.wid(2)));
    c.drawLine(Offset(sx - 4, top), Offset(sx + 4, top), wbStroke(f.mark(2), f.wid(2)));
    c.drawCircle(Offset(sx, ry), 1.6, wbFill(f.mark(2)));
    // the flag (D)
    final fx = _flagX(a), flut = math.sin(_t * 3) * 1.1;
    c.drawLine(Offset(fx, gy - 3), Offset(fx, gy - 22), wbStroke(f.ink(3, N.g56)));
    final pen = Path()..moveTo(fx, gy - 22)..lineTo(fx + 9, gy - 18.5 + flut)..lineTo(fx, gy - 14);
    c.drawPath(pen, wbFill(f.mark(3, .25)));
    c.drawPath(pen, wbStroke(f.mark(3), f.wid(3)));
    // the loupe on the grain of the ridge (A): only the finest octaves, magnified
    final l = _loupe(a), r = _loupeR(a);
    c.save();
    c.clipPath(Path()..addOval(Rect.fromCircle(center: l, radius: r - 1)));
    c.drawRect(Rect.fromCircle(center: l, radius: r), wbFill(N.g10));
    final fine = Path();
    for (var i = 0; i <= 28; i++) {
      final x = l.dx - r + i * (2 * r / 28), y = l.dy - _f(_phase * 3 + (x - l.dx) * .12, v['seed']!, _g(v['rough']!), from: 2) * (r * .75) * 4.2 * (.3 + v['rough']! * .7);
      i == 0 ? fine.moveTo(x, y) : fine.lineTo(x, y);
    }
    c.drawPath(fine, wbStroke(f.grab == null ? N.g63 : N.g38));
    c.restore();
    c.drawCircle(l, r, wbStroke(f.mark(0), f.wid(0)));
    c.drawLine(l + Offset(r * .72, r * .72), l + Offset(r * 1.2, r * 1.2), wbStroke(f.mark(0, .8), f.wid(0)));
  }

  @override
  String readout(Map<String, double> v) => '${WbSpec.n(v['rough']!, 2)} · ${WbSpec.n(v['drift']!, 2)} · ${v['amount']!.round()} % · #${v['seed']!.round()}';
}
