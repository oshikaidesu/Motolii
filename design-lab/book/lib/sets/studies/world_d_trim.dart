// World D 23: Trim paths / Stroke. A knotted line is being drawn along its own rail; the ink is only where the two nibs say.
// A Start (blue): the rear nib. B End (green): the front nib. C Width (white): the ink itself (pull it away from the line to swell it).
// D Offset (orange): the small arc beside the ink shows how far the whole stretch has slid round the rail; drag the empty paper to slide it.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_d_kit.dart';

const trimSpecs = [
  WdSpec('start', 'Start', 0, 100, 0, unit: '%', digits: 0, amp: .12),
  WdSpec('end', 'End', 0, 100, 100, unit: '%', digits: 0, amp: .2),
  WdSpec('width', 'Width', 0, 60, 2, unit: 'px', digits: 0, amp: .12),
  WdSpec('offset', 'Offset', 0, 100, 0, unit: '%', digits: 0, amp: .1),
];

class _Rail {
  _Rail(Rect a) {
    const n = 260;
    final r = a.deflate(16);
    final raw = [
      for (var i = 0; i <= n; i++)
        Offset(r.center.dx + math.sin(2 * (i / n) * 2 * math.pi + .7) * r.width / 2, r.center.dy + math.sin(3 * (i / n) * 2 * math.pi) * r.height / 2),
    ];
    pts = raw;
    var acc = 0.0;
    final c = <double>[0];
    for (var i = 1; i < raw.length; i++) {
      acc += (raw[i] - raw[i - 1]).distance;
      c.add(acc);
    }
    len = acc;
    cum = [for (final x in c) x / acc];
  }
  late final List<Offset> pts;
  late final List<double> cum;
  late final double len;

  Offset at(double s) {
    s = s - s.floorToDouble();
    var lo = 0, hi = cum.length - 1;
    while (hi - lo > 1) {
      final m = (lo + hi) >> 1;
      if (cum[m] <= s) {
        lo = m;
      } else {
        hi = m;
      }
    }
    final t = (s - cum[lo]) / math.max(1e-9, cum[hi] - cum[lo]);
    return Offset.lerp(pts[lo], pts[hi], t)!;
  }

  Offset tan(double s) {
    final d = at(s + .003) - at(s - .003);
    return d / math.max(1e-6, d.distance);
  }

  (double, double) nearest(Offset p) {
    var best = 1e9, bs = 0.0;
    for (var i = 0; i < pts.length; i++) {
      final d = (pts[i] - p).distance;
      if (d < best) {
        best = d;
        bs = cum[i];
      }
    }
    return (best, bs);
  }
}

class TrimMini extends WdMini {
  TrimMini(super.doc);
  double _dp = 0;

  @override
  List<List<String>> get zoneIds => const [['start'], ['end'], ['width'], ['offset']];

  @override
  List<int> get readSlots => const [0, 1, 2, 3];

  @override
  List<WdR> readout() => [
        (doc.show('start'), doc.changed('start')),
        (doc.show('end'), doc.changed('end')),
        (doc.show('width'), doc.changed('width')),
        (doc.show('offset'), doc.changed('offset')),
      ];

  /// The two tips as drawn (idle breath included) and the lit stretch in rail units.
  (double, double, double) _ends() {
    final off = doc.eff('offset') / 100;
    var s = doc.eff('start') / 100, e = doc.eff('end') / 100;
    s = wdClamp(s + wdNoise(doc.time * .9, 1) * .008, 0, 1);
    e = wdClamp(e + wdNoise(doc.time * .8, 2) * .012, 0, 1);
    return (s, e, off);
  }

  double _w() => 1 + doc.eff('width') / 60 * 3.5;

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s), rail = _Rail(a);
    final (st, en, off) = _ends();
    final ds = (rail.at(st + off) - p).distance, de = (rail.at(en + off) - p).distance;
    if (ds <= 18 && ds <= de) return 0;
    if (de <= 18) return 1;
    final (d, sp) = rail.nearest(p);
    final lo = math.min(st, en), hi = math.max(st, en);
    var rel = (sp - off) % 1.0;
    if (rel < 0) rel += 1;
    if (d <= 12 && rel >= lo && rel <= hi) return 2;
    if (a.inflate(10).contains(p)) return 3;
    return null;
  }

  @override
  void down(int z, Offset p, Size s) {
    if (z == 2) {
      _dp = _Rail(art(s)).nearest(p).$1;
    }
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, rail = _Rail(art(s));
    final (st, en, off) = _ends();
    switch (z) {
      case 0:
        final t = rail.tan(st + off);
        doc.set('start', doc.base('start') + (d.dx * t.dx + d.dy * t.dy) / rail.len * 100 * k);
      case 1:
        final t = rail.tan(en + off);
        doc.set('end', doc.base('end') + (d.dx * t.dx + d.dy * t.dy) / rail.len * 100 * k);
      case 2:
        final dist = rail.nearest(p).$1;
        // soft-limited: 1 unit per px of pull, the gain eases to a fifth towards both ends, so a short drag never reaches the extreme
        final u = wdClamp(doc.base('width') / 60, 0, 1), gain = .2 + .8 * math.sin(math.pi * u);
        doc.set('width', doc.base('width') + (dist - _dp) * gain * k);
        _dp = dist;
      case 3:
        doc.set('offset', doc.base('offset') + d.dx * k * .3);
    }
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), rail = _Rail(a);
    final (st, en, off) = _ends();
    final lo = math.min(st, en), hi = math.max(st, en), w = _w();

    // the rail
    final p = Path()..moveTo(rail.pts.first.dx, rail.pts.first.dy);
    for (final q in rail.pts.skip(1)) {
      p.lineTo(q.dx, q.dy);
    }
    c.drawPath(p, wdStroke(N.g26));

    // the ink
    final f = hi - lo;
    if (f > .002) {
      final m = (f * 220).ceil() + 1;
      final ink = Path();
      for (var i = 0; i < m; i++) {
        final q = rail.at(lo + off + f * i / (m - 1));
        i == 0 ? ink.moveTo(q.dx, q.dy) : ink.lineTo(q.dx, q.dy);
      }
      c.drawPath(ink, wdStroke(N.g76.withValues(alpha: .55 * wdClamp(view.a[2] + .2, 0, 1)), w));
      c.drawPath(ink, wdStroke(zc(2, .9), 1));
    }

    // D: the slide of the whole stretch (a hairline arc beside the ink) and the rail's own origin
    final o = off % 1.0;
    final sideOff = w / 2 + 4;
    Offset side(double sp) {
      final t = rail.tan(sp);
      return rail.at(sp) + Offset(-t.dy, t.dx) * sideOff;
    }

    if (o > .004) {
      final arc = Path();
      final m = (o * 160).ceil() + 1;
      for (var i = 0; i < m; i++) {
        final q = side(o * i / (m - 1));
        i == 0 ? arc.moveTo(q.dx, q.dy) : arc.lineTo(q.dx, q.dy);
      }
      c.drawPath(arc, wdStroke(zc(3)));
    }
    final t0 = rail.tan(0), p0 = rail.at(0);
    c.drawLine(p0 + Offset(-t0.dy, t0.dx) * (sideOff - 3), p0 + Offset(-t0.dy, t0.dx) * (sideOff + 3), wdStroke(zc(3)));

    // A / B: the two nibs, pointing away from the ink
    void nib(double sp, double dir, int z) {
      final q = rail.at(sp), t = rail.tan(sp) * dir, n = Offset(-t.dy, t.dx);
      final hw = math.max(3.5, w / 2 + 1.5);
      final tri = Path()
        ..moveTo(q.dx + t.dx * 8, q.dy + t.dy * 8)
        ..lineTo(q.dx + n.dx * hw, q.dy + n.dy * hw)
        ..lineTo(q.dx - n.dx * hw, q.dy - n.dy * hw)
        ..close();
      c.drawPath(tri, wdFill(zc(z, 1.3)));
      c.drawCircle(q, hw + 5, wdStroke(zc(z, .9)));
    }

    nib(st + off, -1, 0);
    nib(en + off, 1, 1);

    final (ps, pe) = (rail.at(st + off), rail.at(en + off));
    if (view.hot == 0 || view.grab == 0) c.drawCircle(ps, 16, wdStroke(wdSlot(0).withValues(alpha: .28)));
    if (view.hot == 1 || view.grab == 1) c.drawCircle(pe, 16, wdStroke(wdSlot(1).withValues(alpha: .28)));
    hint(c, a.inflate(4), 3);
  }
}
