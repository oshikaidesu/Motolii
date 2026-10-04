// World B 7: Rotation distribution, "the needle fan". Seven needles pinned on a line; the first and last needle are held by A and B,
// the white arc is how far one needle may shiver (C), the orange ring is the pin the needles turn on (D).
// Needles are foreshortened as they lean sideways so that neighbours can never cross: the fan stays readable at every value.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_b_kit.dart';

class WbFan extends WbWorld {
  static const _n = 7, _iC = 2, _iD = 4;
  static const _jMax = 14.0; // degrees of shiver at jitter 1
  double _t = 0;

  @override
  String get name => 'Rotation fan';
  @override
  String get silhouette => 'a fan of pinned needles';
  @override
  List<WbSpec> get specs => const [
        WbSpec('start', 'Start', -180, 180, 0, unit: '°', digits: 0, amp: .1),
        WbSpec('step', 'Step', -30, 30, 12, unit: '°', digits: 0, amp: .16),
        WbSpec('jitter', 'Jitter', 0, 1, .3, amp: .2),
        WbSpec('pivot', 'Pivot', 15, 85, 50, unit: ' %', digits: 0, amp: .3),
      ];
  @override
  List<List<String>> get zoneIds => const [['start'], ['step'], ['jitter'], ['pivot']];

  @override
  void tick(double t, double dt, Map<String, double> v, Size s) => _t = t;

  double _d(Size s) => (wbArea(s).width - 4) / 7; // pin spacing
  Offset _pin(int i, Size s) {
    final a = wbArea(s), d = _d(s);
    return Offset(a.left + d * .5 + 2 + d * i, a.center.dy);
  }

  double _len(Size s) {
    final a = wbArea(s);
    return math.min((a.height / 2 - 2) / .85, _d(s) * 2.6);
  }

  /// A needle half that leans sideways is foreshortened (smooth p-norm) so its horizontal reach stays under half a pin spacing.
  double _reach(double l, double ang, Size s) {
    final c = _d(s) * .48, sn = math.sin(wbRad(ang)).abs() + 1e-6;
    return 1 / math.pow(math.pow(1 / l, 4) + math.pow(sn / c, 4), .25);
  }

  double _base(int i, Map<String, double> v) => v['start']! + v['step']! * i;
  double _ang(int i, Map<String, double> v) => _base(i, v) + 1.2 * math.sin(_t * 1.5 + i * .55) + v['jitter']! * _jMax * wbNoise(_t * .9 + i * 2.3, i.toDouble());

  (Offset, Offset) _ends(int i, Size s, Map<String, double> v, double ang) {
    final d = wbDir(ang), p = v['pivot']! / 100, l = _len(s), pin = _pin(i, s);
    return (pin - d * _reach(l * p, ang, s), pin + d * _reach(l * (1 - p), ang, s));
  }

  Offset _ghost(Size s, Map<String, double> v) {
    final ang = _base(_iC, v) + v['jitter']! * _jMax;
    return _pin(_iC, s) + wbDir(ang) * (_reach(_len(s) * (1 - v['pivot']! / 100), ang, s) + 4);
  }

  @override
  List<List<Offset>> anchors(Size s, Map<String, double> v) => [
        [_ends(0, s, v, _ang(0, v)).$2],
        [_ends(_n - 1, s, v, _ang(_n - 1, v)).$2],
        [_ghost(s, v)],
        [_pin(_iD, s)],
      ];

  @override
  Map<String, double> drag(int z, Offset p, Offset p0, Map<String, double> v0, Size s) {
    switch (z) {
      case 0:
        final c = _pin(0, s);
        final d = wbAngDiff((p - c).direction, (p0 - c).direction) * 180 / math.pi;
        return {...v0, 'start': wbWrapDeg(v0['start']! + d)};
      case 1:
        final c = _pin(_n - 1, s);
        final d = wbAngDiff((p - c).direction, (p0 - c).direction) * 180 / math.pi;
        return {...v0, 'step': v0['step']! + d / (_n - 1)};
      case 2:
        final c = _pin(_iC, s), base = wbRad(_base(_iC, v0) - 90);
        final a1 = wbAngDiff((p - c).direction, base), a0 = wbAngDiff((p0 - c).direction, base);
        return {...v0, 'jitter': v0['jitter']! + (a1.abs() - a0.abs()) * 180 / math.pi / _jMax};
      default:
        // the pin slides along the needle: 1:1 with the picture, and a sideways pull counts a little too
        final u = wbDir(_base(_iD, v0)), q = p - p0, ax = q.dx * u.dx + q.dy * u.dy;
        return {...v0, 'pivot': v0['pivot']! + (ax + q.dx * .25) / (_len(s) * 2.4) * 100};
    }
  }

  @override
  void paint(Canvas c, Size s, WbFrame f) {
    final len = _len(s), jr = f['jitter'] * _jMax;
    for (var i = 0; i < _n; i++) {
      final pin = _pin(i, s), ang = _ang(i, f.v), e = _ends(i, s, f.v, ang);
      final z = i == 0 ? 0 : (i == _n - 1 ? 1 : -1);
      final line = z >= 0 ? f.mark(z) : (f.grab == null ? N.g63 : N.g38);
      c.drawLine(e.$1, e.$2, wbStroke(line, z >= 0 ? f.wid(z) : 1));
      if (z >= 0) {
        final d = Path()..addPolygon([e.$2 + const Offset(0, -3.4), e.$2 + const Offset(3.4, 0), e.$2 + const Offset(0, 3.4), e.$2 + const Offset(-3.4, 0)], true);
        c.drawPath(d, wbFill(line));
      } else {
        c.drawCircle(e.$2, 1.3, wbFill(f.grab == null ? N.g91 : N.g38));
      }
      if (i == _iD) {
        c.drawCircle(pin, 3.6, wbStroke(f.mark(3), f.wid(3)));
      } else {
        c.drawCircle(pin, 1.2, wbFill(N.g44));
      }
    }
    // the shiver range of the C needle: one fine arc around its tip, and the handle at its far end
    final pc = _pin(_iC, s), bc = _base(_iC, f.v), rr = _reach(len * (1 - f['pivot'] / 100), bc, s);
    if (jr > .3 && rr > 3) {
      c.drawArc(Rect.fromCircle(center: pc, radius: rr + 4), wbRad(bc - jr - 90), wbRad(jr * 2), false, wbStroke(f.mark(2, .8), f.wid(2)));
    }
    c.drawCircle(_ghost(s, f.v), 2.0, wbFill(f.mark(2)));
  }

  @override
  String readout(Map<String, double> v) =>
      '${v['start']!.round()}° · ${v['step']! >= 0 ? '+' : '−'}${v['step']!.abs().round()}° · ${WbSpec.n(v['jitter']!, 2)} · ${v['pivot']!.round()} %';
}
