// World B 12: Audio react, "the ear and the gate". The ear (left) sends rings across; a white gate-arc lets only the loud rings through; the bead on
// the right is shaken by what gets through. A is the blue wake the bead leaves (smooth = a glassy wake), B the green halo of how far it may be shaken,
// C the white gate (closer = stricter), D the orange ear (its arcs 1-3 = low, mid, high band).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_b_kit.dart';

class _Ring {
  _Ring(this.t0, this.db);
  final double t0, db;
}

class WbEar extends WbWorld {
  double _t = 0, _drive = 0, _band = 1;
  final List<_Ring> _rings = [];
  double _emit = 0;
  final List<double> _hist = []; // recent shake offsets of the bead (the wake)

  @override
  String get name => 'Audio ear';
  @override
  String get silhouette => 'an ear, a gate and a shaken bead';
  @override
  List<WbSpec> get specs => const [
        WbSpec('smooth', 'Smooth', 0, 1, .5, amp: .25),
        WbSpec('gain', 'Gain', 0, 400, 100, unit: ' %', digits: 0, amp: .2),
        WbSpec('thr', 'Threshold', -60, 0, -30, unit: ' dB', digits: 0, amp: .2),
        WbSpec('band', 'Band', 0, 2, 1, digits: 0, integer: true, driven: false),
      ];
  @override
  List<List<String>> get zoneIds => const [['smooth'], ['gain'], ['thr'], ['band']];
  @override
  Set<int> get tapZones => const {3};

  static const _period = [1.1, .75, .34], _decay = [20.0, 34.0, 80.0], _every = [.15, .095, .06];

  double _db(double t, int band) {
    final k = (t / _period[band]).floor(), ph = t - k * _period[band];
    final peak = -3 - 30 * wbHash(k, band + 2);
    return math.max(-80, peak - ph * _decay[band]);
  }

  Rect _a(Size s) => wbArea(s);
  Offset _ear(Rect a) => Offset(a.left + 12, a.center.dy);
  Offset _bead(Rect a) => Offset(a.right - _cap(a) - 8, a.center.dy);
  double _cap(Rect a) => math.min(a.height * .34, 38);
  double _rmax(Rect a) => _bead(a).dx - _ear(a).dx - 6;
  double _speed(Rect a) => _rmax(a) / 1.4;
  double _rg(Rect a, double thr) => _rmax(a) * (.14 + .71 * (-thr / 60));
  double _shakeR(Rect a, double gain) => wbClamp(gain / 100 * _cap(a) / 4, 4, _cap(a));

  @override
  void tick(double t, double dt, Map<String, double> v, Size s) {
    final a = _a(s), band = v['band']!.round();
    _t = t;
    _band += (band - _band) * (1 - math.exp(-dt / .1));
    _emit += dt;
    if (_emit >= _every[band]) {
      _emit = 0;
      _rings.add(_Ring(t, _db(t, band)));
    }
    final life = (_rmax(a) + 12) / _speed(a);
    _rings.removeWhere((r) => t - r.t0 > life);
    final delay = (_bead(a).dx - _ear(a).dx) / _speed(a);
    final tgt = wbClamp((_db(math.max(0, t - delay), band) - v['thr']!) / 25, 0, 1);
    _drive += (tgt - _drive) * (1 - math.exp(-dt / (.012 + .4 * math.pow(v['smooth']!, 1.5))));
    _hist.add(_shake(a, v).dy);
    if (_hist.length > 60) _hist.removeAt(0);
  }

  Offset _shake(Rect a, Map<String, double> v) {
    final r = _shakeR(a, v['gain']!) * _drive;
    return Offset(wbNoise(_t * 19, 1) * r, wbNoise(_t * 23, 2) * r);
  }

  @override
  List<List<Offset>> anchors(Size s, Map<String, double> v) {
    final a = _a(s), b = _bead(a), r = _shakeR(a, v['gain']!), rg = _rg(a, v['thr']!), e = _ear(a);
    return [
      [for (var i = 0; i < 6; i++) b + Offset(-10 - i * 6.0, 0)],
      [for (var k = 0; k < 8; k++) b + Offset(math.cos(k * math.pi / 4), math.sin(k * math.pi / 4)) * (r + 4)],
      [for (var d = -50; d <= 50; d += 20) e + Offset(math.cos(wbRad(d.toDouble())), math.sin(wbRad(d.toDouble()))) * rg],
      [e],
    ];
  }

  @override
  Map<String, double> drag(int z, Offset p, Offset p0, Map<String, double> v0, Size s) {
    final a = _a(s), d = p - p0, e = _ear(a);
    switch (z) {
      case 0:
        return {...v0, 'smooth': v0['smooth']! - d.dx / 50};
      case 1:
        final b = _bead(a), k = _cap(a) / 4;
        return {...v0, 'gain': v0['gain']! + ((p - b).distance - (p0 - b).distance) / k * 100};
      case 2:
        return {...v0, 'thr': v0['thr']! - ((p - e).distance - (p0 - e).distance) * 60 / (.71 * _rmax(a))};
      default:
        return {...v0, 'band': v0['band']! + d.dy / 18};
    }
  }

  @override
  Map<String, double>? tap(int z, Map<String, double> v) => z == 3 ? {'band': (v['band']! + 1) % 3} : null;

  @override
  void paint(Canvas c, Size s, WbFrame f) {
    final a = _a(s), v = f.v, e = _ear(a), b = _bead(a), rg = _rg(a, v['thr']!), sp = _speed(a), thr = v['thr']!;
    c.save();
    c.clipRect(a.inflate(5));
    // the rings
    for (final r in _rings) {
      final rad = 12 + (_t - r.t0) * sp;
      final lv = wbClamp((r.db + 60) / 60, 0, 1), pass = r.db >= thr;
      var al = .12 + .7 * lv;
      if (!pass) al *= 1 - wbClamp((rad - rg) / 10, 0, 1);
      if (al < .02) continue;
      final col = (pass ? N.g91 : N.g44).withValues(alpha: f.grab == null ? al : al * .55);
      c.drawArc(Rect.fromCircle(center: e, radius: rad), wbRad(-38), wbRad(76), false, wbStroke(col));
    }
    c.restore();
    // the gate (C)
    for (var d = -52; d < 52; d += 9) {
      c.drawArc(Rect.fromCircle(center: e, radius: rg), wbRad(d.toDouble()), wbRad(5), false, wbStroke(f.mark(2, .85), f.wid(2)));
    }
    // the ear (D): its arcs say the band
    c.drawCircle(e, 2.4, wbFill(f.mark(3)));
    final nArcs = 1 + _band.round();
    for (var i = 0; i < nArcs; i++) {
      c.drawArc(Rect.fromCircle(center: e, radius: 5.5 + i * 3.2), wbRad(-45), wbRad(90), false, wbStroke(f.mark(3, .9), f.wid(3)));
    }
    // the shake halo (B) and the bead
    final r = _shakeR(a, v['gain']!);
    c.drawCircle(b, r + 3, wbStroke(f.mark(1, .75), f.wid(1)));
    c.drawCircle(b + _shake(a, v), 3.4, wbFill(f.grab == null ? N.g91 : N.g63));
    // the wake (A): where the bead has just been, plotted going back in time
    if (_hist.length > 1) {
      final w2 = Path()..moveTo(b.dx - 12, b.dy + _hist.last);
      for (var i = _hist.length - 1; i >= 0; i--) {
        w2.lineTo(b.dx - 12 - (_hist.length - 1 - i) * 1.6, b.dy + _hist[i]);
      }
      c.drawPath(w2, wbStroke(f.mark(0)));
    }
  }

  @override
  String readout(Map<String, double> v) => '${v['thr']!.round().toString().replaceFirst('-', '−')} dB · ×${WbSpec.n(v['gain']! / 100, 2)} · ${WbSpec.n(v['smooth']!, 2)}';
}
