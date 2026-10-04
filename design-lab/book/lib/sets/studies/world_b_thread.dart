// World B 11: Graph / Link, "two beads on a slack thread". The left bead wanders by itself; the right bead answers through the thread and a faint
// trail of afterimages shows how far it travels. A is the blue ring on the answering bead (pull it away from its rest: it answers louder), B the
// green knot of the thread (pull the thread down to let it hang slack: the sag IS the bend of the answer, taut = straight), C the white notch where the
// answering bead rests, D the orange ring on the left bead (pull it sideways to make the answer arrive late).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_b_kit.dart';

class WbThread extends WbWorld {
  double _t = 0;

  @override
  String get name => 'Link thread';
  @override
  String get silhouette => 'two beads on a slack thread';
  @override
  List<WbSpec> get specs => const [
        WbSpec('strength', 'Strength', 0, 1.5, 1, amp: .2),
        WbSpec('slack', 'Slack', 0, 1, .5, amp: .25),
        WbSpec('offset', 'Offset', -100, 100, 0, unit: ' %', digits: 0, amp: .25),
        WbSpec('lag', 'Lag', 0, 1, .15, unit: ' s', amp: .25),
      ];
  @override
  List<List<String>> get zoneIds => const [['strength'], ['slack'], ['offset'], ['lag']];

  @override
  void tick(double t, double dt, Map<String, double> v, Size s) => _t = t;

  double _amp(Rect a) => a.height * .15;
  double _rest(Rect a, Map<String, double> v) => a.center.dy + a.height * .15 - v['offset']! / 100 * a.height * .12;
  double _xs(Rect a) => a.left + a.width * .13;
  double _xt(Rect a) => a.right - a.width * .17;
  double _sig(double t) => math.sin(t * 1.1 + .6 * math.sin(t * .37)) * .8 + math.sin(t * 2.3) * .2; // -1..1, up positive

  /// The answer curve: straight when taut, an eased S when slack.
  double _map(double s, double slack) => s * (1 - slack) + slack * s * (1.5 - .5 * s * s);

  Offset _src(Rect a, [double back = 0]) => Offset(_xs(a), a.center.dy - a.height * .04 - _sig(_t - back) * a.height * .26);
  Offset _dst(Rect a, Map<String, double> v, [double back = 0]) =>
      Offset(_xt(a), _rest(a, v) - v['strength']! * _map(_sig(_t - v['lag']! * 1.2 - back), v['slack']!) * _amp(a));

  Offset _thread(double u, Rect a, Offset s, Offset t, double slack) {
    final mid = Offset.lerp(s, t, .5)!, room = math.max(0.0, 2 * (a.bottom - 3 - mid.dy)), sag = slack * math.min((t.dx - s.dx) * .45, room) * (1 + .05 * math.sin(_t * 1.3));
    final ctrl = mid + Offset(0, sag), m = 1 - u;
    return s * (m * m) + ctrl * (2 * m * u) + t * (u * u);
  }

  @override
  List<List<Offset>> anchors(Size s, Map<String, double> v) {
    final a = wbArea(s), sp = _src(a), tp = _dst(a, v), rest = _rest(a, v), nx = tp.dx + 19;
    return [
      [tp],
      [for (var i = 3; i <= 9; i++) _thread(i / 12, a, sp, tp, v['slack']!)],
      [Offset(nx, rest), Offset(nx - 5, rest), Offset(nx + 5, rest)],
      [sp],
    ];
  }

  @override
  Map<String, double> drag(int z, Offset p, Offset p0, Map<String, double> v0, Size s) {
    final a = wbArea(s), d = p - p0;
    switch (z) {
      case 0:
        final yc = _rest(a, v0);
        return {...v0, 'strength': v0['strength']! + ((p.dy - yc).abs() - (p0.dy - yc).abs()) / (_amp(a) * 1.5)};
      case 1:
        return {...v0, 'slack': v0['slack']! + d.dy / 70};
      case 2:
        return {...v0, 'offset': v0['offset']! - d.dy * 1.1};
      default:
        return {...v0, 'lag': v0['lag']! + d.dx / 110};
    }
  }

  @override
  void paint(Canvas c, Size s, WbFrame f) {
    final a = wbArea(s), v = f.v, sp = _src(a), tp = _dst(a, v), slack = v['slack']!;
    final th = Path()..moveTo(sp.dx, sp.dy);
    for (var k = 1; k <= 40; k++) {
      final q = _thread(k / 40, a, sp, tp, slack);
      th.lineTo(q.dx, q.dy);
    }
    c.drawPath(th, wbStroke(f.grab == null ? N.g63 : N.g44));
    // a slow chain of small beads rides the thread from the left to the right
    for (var i = 0; i < 6; i++) {
      c.drawCircle(_thread((i + (_t * .3) % 1) / 6, a, sp, tp, slack), 1.2, wbFill(f.grab == null ? N.g56 : N.g38));
    }
    // afterimages: where each bead has just been (the strength is how far the right one reaches)
    for (var j = 1; j <= 8; j++) {
      final al = (1 - j / 9) * .55, q1 = _src(a, j * .13), q2 = _dst(a, v, j * .13);
      c.drawCircle(q1, 1.3, wbFill(N.g56.withValues(alpha: al)));
      c.drawCircle(q2, 1.3, wbFill(N.g56.withValues(alpha: al)));
    }
    // the notch where the answering bead rests (C)
    final rest = _rest(a, v), nx = tp.dx + 19;
    c.drawLine(Offset(nx - 5, rest), Offset(nx + 5, rest), wbStroke(f.mark(2), f.wid(2)));
    c.drawCircle(Offset(nx, rest), 1.5, wbFill(f.mark(2)));
    // the left bead and its ring (D)
    c.drawCircle(sp, 4, wbFill(N.g91));
    c.drawCircle(sp, 9, wbStroke(f.mark(3, .9), f.wid(3)));
    // the knot (B): where the thread hangs lowest
    final k = _thread(.5, a, sp, tp, slack);
    c.drawCircle(k, 3.4, wbFill(N.g10));
    c.drawCircle(k, 3.4, wbStroke(f.mark(1), f.wid(1)));
    // the answering bead and its ring (A)
    c.drawCircle(tp, 4, wbFill(N.g91));
    c.drawCircle(tp, 9, wbStroke(f.mark(0), f.wid(0)));
  }

  @override
  String readout(Map<String, double> v) => '×${WbSpec.n(v['strength']!, 2)} · slack ${WbSpec.n(v['slack']!, 2)} · ${v['offset']!.round()} % · ${WbSpec.n(v['lag']!, 2)} s';
}
