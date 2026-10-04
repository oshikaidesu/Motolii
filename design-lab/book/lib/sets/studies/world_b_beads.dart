// World B 8: Colour gradient, "beads on a wire". Eight beads on a wavy wire take their colour from the two ends. A is the first bead, B the last,
// C the white tick where the colour is half way (it slides along the wire), D the orange halo that says how far the beads scatter off the wire.
// The bead colours are the CONTENT (a gradient is colour); every grab mark stays a slot colour.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_b_kit.dart';

class WbBeads extends WbWorld {
  static const _n = 8, _dust = 4;
  double _t = 0;

  @override
  String get name => 'Colour beads';
  @override
  String get silhouette => 'beads on a wire';
  @override
  List<WbSpec> get specs => const [
        WbSpec('from', 'From', 0, 360, 232, unit: '°', digits: 0, amp: .08),
        WbSpec('to', 'To', 0, 360, 16, unit: '°', digits: 0, amp: .08),
        WbSpec('ease', 'Ease', .4, 2.5, 1, amp: .2),
        WbSpec('spread', 'Spread', 0, 100, 20, unit: ' %', digits: 0, amp: .25),
      ];
  @override
  List<List<String>> get zoneIds => const [['from'], ['to'], ['ease'], ['spread']];

  @override
  void tick(double t, double dt, Map<String, double> v, Size s) => _t = t;

  double _amp(Rect a) => math.min(a.height * .26, 24);
  Offset _wp(double u, Rect a) => Offset(a.left + 8 + u * (a.width - 16), a.center.dy + _amp(a) * math.sin(u * 2 * math.pi));
  Offset _nrm(double u, Rect a) {
    final dy = _amp(a) * 2 * math.pi / (a.width - 16) * math.cos(u * 2 * math.pi), l = math.sqrt(1 + dy * dy);
    return Offset(-dy / l, 1 / l);
  }

  double _u(int i) => (i + .3 * math.sin(_t * .9 + i * .7)) / (_n - 1) * .92 + .04;
  double _ring(Rect a, Map<String, double> v) => 8 + v['spread']! / 100 * math.min(14, a.height * .22);

  Offset _bead(int i, Rect a, Map<String, double> v) {
    final u = _u(i), off = (wbHash(i, 5) * 2 - 1) * (_ring(a, v) - 8) * (i == 0 || i == _n - 1 ? .0 : 1);
    return _wp(u, a) + _nrm(u, a) * (off + wbNoise(_t * .8 + i, i.toDouble()) * 1.2);
  }

  Color _col(int i, Map<String, double> v) {
    final tt = math.pow(i / (_n - 1), v['ease']!).toDouble();
    final d = wbWrapDeg(v['to']! - v['from']!);
    return HSLColor.fromAHSL(1, ((v['from']! + d * tt) % 360 + 360) % 360, .6, .62).toColor();
  }

  double _uMid(Map<String, double> v) => math.pow(.5, 1 / v['ease']!).toDouble();

  @override
  List<List<Offset>> anchors(Size s, Map<String, double> v) {
    final a = wbArea(s), r = _ring(a, v), mid = _wp(_u(_dust), a);
    return [
      [_bead(0, a, v)],
      [_bead(_n - 1, a, v)],
      [_wp(_uMid(v), a)],
      [for (var k = 0; k < 8; k++) mid + Offset(math.cos(k * math.pi / 4), math.sin(k * math.pi / 4)) * r],
    ];
  }

  @override
  Map<String, double> drag(int z, Offset p, Offset p0, Map<String, double> v0, Size s) {
    final a = wbArea(s), dx = p.dx - p0.dx;
    switch (z) {
      case 0:
        return {...v0, 'from': ((v0['from']! + dx * 1.2) % 360 + 360) % 360};
      case 1:
        return {...v0, 'to': ((v0['to']! + dx * 1.2) % 360 + 360) % 360};
      case 2:
        final u = wbClamp(_uMid(v0) + dx / (a.width - 16), .04, .96);
        return {...v0, 'ease': math.log(.5) / math.log(u)};
      default:
        final c = _wp(_u(_dust), a), k = math.min(14, a.height * .22);
        return {...v0, 'spread': v0['spread']! + ((p - c).distance - (p0 - c).distance) / k * 100};
    }
  }

  @override
  void paint(Canvas c, Size s, WbFrame f) {
    final a = wbArea(s), v = f.v, br = math.min(5.0, a.height * .12);
    // the wire
    final wire = Path()..moveTo(_wp(0, a).dx, _wp(0, a).dy);
    for (var k = 1; k <= 60; k++) {
      final q = _wp(k / 60, a);
      wire.lineTo(q.dx, q.dy);
    }
    c.drawPath(wire, wbStroke(N.g26));
    // the half-way tick (C)
    final um = _uMid(v), pm = _wp(um, a), nm = _nrm(um, a);
    c.drawLine(pm - nm * 7, pm + nm * 7, wbStroke(f.mark(2), f.wid(2)));
    // the scatter halo (D)
    final mid = _wp(_u(_dust), a);
    c.drawCircle(mid, _ring(a, v), wbStroke(f.mark(3, .7), f.wid(3)));
    // beads: the content
    for (var i = 0; i < _n; i++) {
      final p = _bead(i, a, v);
      c.drawCircle(p, br, wbFill(f.grab == null || i == 0 && f.grab == 0 || i == _n - 1 && f.grab == 1 ? _col(i, v) : _col(i, v).withValues(alpha: .55)));
      if (i == 0) c.drawCircle(p, br + 3.5, wbStroke(f.mark(0), f.wid(0)));
      if (i == _n - 1) c.drawCircle(p, br + 3.5, wbStroke(f.mark(1), f.wid(1)));
    }
  }

  static String _hex(Color c) => '#${(c.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}';
  @override
  String readout(Map<String, double> v) => '${_hex(_col(0, v))} → ${_hex(_col(_n - 1, v))}';
}
