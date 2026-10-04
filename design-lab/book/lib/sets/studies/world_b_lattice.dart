// World B 9: Repeat, "the dividing figure". One seed hexagon (top-left, bright) divides into a lattice; copies pop out of the seed when the count grows.
// A is the faint blue column that would come next, B the faint green row, C the white dashes of the gap, D the orange leash of one wandering copy.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_b_kit.dart';

class _Lay {
  _Lay(this.k, this.fs, this.pitchX, this.pitchY, this.gapPx, this.o);
  final double k, fs, pitchX, pitchY, gapPx;
  final Offset o; // centre of figure (0,0)
  Offset at(num c, num r) => o + Offset(c * pitchX, r * pitchY);
}

class WbLattice extends WbWorld {
  final Map<int, double> _born = {};
  Set<int> _present = {};
  bool _first = true;

  @override
  String get name => 'Repeat lattice';
  @override
  String get silhouette => 'a figure dividing into a lattice';
  @override
  List<WbSpec> get specs => const [
        WbSpec('cols', 'Columns', 2, 8, 5, digits: 0, integer: true, amp: .22),
        WbSpec('rows', 'Rows', 2, 6, 3, digits: 0, integer: true, amp: .26),
        WbSpec('gap', 'Gap', 0, 100, 25, unit: ' %', digits: 0, amp: .25),
        WbSpec('jitter', 'Jitter', 0, 1, .25, amp: .25),
      ];
  @override
  List<List<String>> get zoneIds => const [['cols'], ['rows'], ['gap'], ['jitter']];

  _Lay _lay(Size s, Map<String, double> v) {
    final a = wbArea(s), cols = v['cols']!.round(), rows = v['rows']!.round();
    final fs = wbClamp(a.height / 4.6, 10, 30), gp = fs * v['gap']! / 100 * 1.2;
    final w = (cols + 1) * fs + cols * gp, h = (rows + 1) * fs + rows * gp;
    final k = math.min(1.0, math.min(a.width / w, a.height / h));
    final px = (fs + gp) * k, py = (fs + gp) * k;
    // centre the block that includes the two ghosts
    final o = Offset(a.center.dx - (cols * px) / 2 + 0, a.center.dy - (rows * py) / 2 + 0);
    return _Lay(k, fs * k, px, py, gp * k, o);
  }

  Offset _off(int c, int r, _Lay l, double j) {
    if (c == 0 && r == 0) return Offset.zero;
    final ang = wbHash(c * 7 + r, 3) * 2 * math.pi, mag = c == 1 && r == 0 ? 1.0 : .35 + .65 * wbHash(c, r + 11);
    return Offset(math.cos(ang), math.sin(ang)) * j * (l.pitchX * .32) * mag;
  }

  Offset _leashDir() => Offset(math.cos(wbHash(7, 3) * 2 * math.pi), math.sin(wbHash(7, 3) * 2 * math.pi));

  @override
  void tick(double t, double dt, Map<String, double> v, Size s) {
    final cols = v['cols']!.round(), rows = v['rows']!.round();
    final now = {for (var r = 0; r < rows; r++) for (var c = 0; c < cols; c++) r * 16 + c};
    for (final k in now) {
      if (!_present.contains(k)) _born[k] = _first ? -10 : t;
    }
    _present = now;
    _first = false;
  }

  @override
  List<List<Offset>> anchors(Size s, Map<String, double> v) {
    final l = _lay(s, v), cols = v['cols']!.round(), rows = v['rows']!.round(), j = v['jitter']!;
    final colX = cols < 8 ? cols : cols - 1, rowY = rows < 6 ? rows : rows - 1;
    final mr = rows ~/ 2;
    return [
      [for (var r = 0; r < rows; r++) l.at(colX, r)],
      [for (var c = 0; c < cols; c++) l.at(c, rowY)],
      [for (var c = 0; c < cols - 1; c++) l.at(c + .5, mr)],
      [l.at(1, 0) + _off(1, 0, l, j)],
    ];
  }

  @override
  Map<String, double> drag(int z, Offset p, Offset p0, Map<String, double> v0, Size s) {
    final l = _lay(s, v0), d = p - p0;
    switch (z) {
      case 0:
        return {...v0, 'cols': v0['cols']! + d.dx / l.pitchX};
      case 1:
        return {...v0, 'rows': v0['rows']! + d.dy / l.pitchY};
      case 2:
        return {...v0, 'gap': v0['gap']! + d.dx / (l.fs * 1.2) * 100};
      default:
        final u = _leashDir();
        return {...v0, 'jitter': v0['jitter']! + (d.dx * u.dx + d.dy * u.dy) / (l.pitchX * .32)};
    }
  }

  void _hex(Canvas c, Offset m, double r, Paint p, [double rot = 0]) {
    final path = Path();
    for (var i = 0; i < 6; i++) {
      final a = rot + i * math.pi / 3 + math.pi / 6, q = m + Offset(math.cos(a), math.sin(a)) * r;
      i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
    }
    c.drawPath(path..close(), p);
  }

  void _dashHex(Canvas c, Offset m, double r, Color col) {
    for (var i = 0; i < 6; i++) {
      final a0 = i * math.pi / 3 + math.pi / 6, a1 = a0 + math.pi / 3;
      final p = m + Offset(math.cos(a0), math.sin(a0)) * r, q = m + Offset(math.cos(a1), math.sin(a1)) * r;
      c.drawLine(Offset.lerp(p, q, .15)!, Offset.lerp(p, q, .6)!, wbStroke(col));
    }
  }

  @override
  void paint(Canvas c, Size s, WbFrame f) {
    final v = f.v, l = _lay(s, v), cols = v['cols']!.round(), rows = v['rows']!.round(), j = v['jitter']!, rr = l.fs * .46;
    final head = (f.t * 2.4) % (cols + rows + 5);
    // ghosts: where the next column / row would divide out
    if (cols < 8) {
      for (var r = 0; r < rows; r++) {
        _dashHex(c, l.at(cols, r), rr, f.mark(0, .55));
      }
    }
    if (rows < 6) {
      for (var cc = 0; cc < cols; cc++) {
        _dashHex(c, l.at(cc, rows), rr, f.mark(1, .55));
      }
    }
    // the gap: white dashes between neighbours in the middle row
    final mr = rows ~/ 2;
    for (var cc = 0; cc < cols - 1; cc++) {
      final m = l.at(cc + .5, mr), hl = math.max(1.5, l.gapPx / 2);
      c.drawLine(m - Offset(hl, 0), m + Offset(hl, 0), wbStroke(f.mark(2, .8), f.wid(2)));
    }
    final src = l.at(0, 0);
    for (var r = 0; r < rows; r++) {
      for (var cc = 0; cc < cols; cc++) {
        final k = r * 16 + cc, u = wbClamp((f.t - (_born[k] ?? -10)) / .24, 0, 1), e = wbEaseOut(u);
        final base = l.at(cc, r) + _off(cc, r, l, j);
        final pos = Offset.lerp(src, base, e)!;
        final glow = math.exp(-math.pow(head - (cc + r), 2) / 1.6) * .5;
        final isSrc = cc == 0 && r == 0;
        final col = f.grab == null ? Color.lerp(N.g56, N.g95, isSrc ? 1 : glow * 1.2)! : N.g44;
        final sc = u < 1 ? (.2 + .8 * e + .18 * math.sin(u * math.pi)) : 1.0;
        _hex(c, pos, rr * sc, wbStroke(isSrc ? N.g95 : col), j * .6 * wbNoise(cc * 1.7 + r, 2));
        if (isSrc) c.drawCircle(pos, 1.6, wbFill(N.g95));
      }
    }
    // the leash of one wandering copy (D)
    final ideal = l.at(1, 0), off = _off(1, 0, l, j);
    c.drawLine(ideal, ideal + off, wbStroke(f.mark(3), f.wid(3)));
    c.drawCircle(ideal, 1.8, wbFill(f.mark(3)));
    c.drawCircle(ideal + off, rr + 2.5, wbStroke(f.mark(3, .6), f.wid(3)));
  }

  @override
  String readout(Map<String, double> v) => '${v['cols']!.round()}×${v['rows']!.round()} · ${v['gap']!.round()} % · ${WbSpec.n(v['jitter']!, 2)}';
}
