// World D 21: Opacity / Blend. A small cat sits behind a frosted pane; the pane is the layer on top, the cat is the layer below.
// A Opacity (blue): the pane's frost (drag up for more grain; the cat fades behind it). B Blend (green): the overlap, where the pane meets the cat:
// the mode decides how the cat's lines mix with the frost. C Clip to below (white): the cat's own outline; set, the frost is cut to the cat's shape.
// D Ghost (orange): the pane leaves faint copies of its edge behind its drift.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_d_kit.dart';

const blendNames = ['Normal', 'Add', 'Multiply', 'Screen'];

const blendSpecs = [
  WdSpec('op', 'Opacity', 0, 100, 60, unit: '%', digits: 0, amp: .16),
  WdSpec('mode', 'Blend', 0, 3, 0, digits: 0, range: 'Normal/Add/Multiply/Screen', defText: 'Normal', driven: false),
  WdSpec('clip', 'Clip to below', 0, 1, 0, digits: 0, range: 'off/on', defText: 'off', driven: false),
  WdSpec('ghost', 'Ghost', 0, 100, 0, unit: '%', digits: 0, amp: .2),
];

class BlendMini extends WdMini {
  BlendMini(super.doc);
  double _acc = 0, _modeAt = 0, _moved = 0;
  bool _clipAt = false;

  @override
  List<List<String>> get zoneIds => const [['op'], ['mode'], ['clip'], ['ghost']];

  @override
  List<WdR> readout() => [
        (doc.show('op'), doc.changed('op')),
        (blendNames[doc.eff('mode').round().clamp(0, 3)], doc.changed('mode')),
        (doc.base('clip') > .5 ? 'clip' : 'free', doc.changed('clip')),
        (doc.show('ghost'), doc.changed('ghost')),
      ];

  /// The cat's box fills the left of the art; the pane covers its right half and runs on to the right edge.
  ({Rect cat, Rect pane, Rect over, Rect a}) _g(Size s) {
    final a = art(s);
    final h = a.height * .94, w = math.min(a.width * .56, h * 1.25);
    final cat = Rect.fromLTWH(a.left + a.width * .03, a.center.dy - h / 2, w, h);
    final pane = Rect.fromLTRB(cat.left + w * .46, a.top + 1, a.right - a.width * .12, a.bottom - 1);
    return (cat: cat, pane: pane, over: cat.intersect(pane), a: a);
  }

  Offset _drift() => Offset(math.sin(doc.time * 1.5) * 3, math.sin(doc.time * 1.05 + 1) * 2);
  Offset _vel() => Offset(math.cos(doc.time * 1.5) * 3 * 1.5, math.cos(doc.time * 1.05 + 1) * 2 * 1.05) / 5;

  /// The cat as a closed silhouette and as its line work, drawn inside [r].
  Path _body(Rect r) {
    Offset p(double x, double y) => Offset(r.left + r.width * x, r.top + r.height * y);
    final body = Path()..addOval(Rect.fromCenter(center: p(.5, .70), width: r.width * .78, height: r.height * .52));
    final head = Path()..addOval(Rect.fromCircle(center: p(.5, .33), radius: r.height * .24));
    final ears = Path()
      ..moveTo(p(.30, .24).dx, p(.30, .24).dy)
      ..lineTo(p(.31, .02).dx, p(.31, .02).dy)
      ..lineTo(p(.45, .13).dx, p(.45, .13).dy)
      ..close()
      ..moveTo(p(.70, .24).dx, p(.70, .24).dy)
      ..lineTo(p(.69, .02).dx, p(.69, .02).dy)
      ..lineTo(p(.55, .13).dx, p(.55, .13).dy)
      ..close();
    return Path.combine(PathOperation.union, Path.combine(PathOperation.union, body, head), ears);
  }

  Path _lines(Rect r) {
    Offset p(double x, double y) => Offset(r.left + r.width * x, r.top + r.height * y);
    final l = Path();
    // whiskers and tail: the fine detail
    for (final sgn in const [-1.0, 1.0]) {
      for (final dy in const [-.02, .04]) {
        l.moveTo(p(.5 + sgn * .10, .40 + dy).dx, p(.5 + sgn * .10, .40 + dy).dy);
        l.lineTo(p(.5 + sgn * .27, .38 + dy * 2).dx, p(.5 + sgn * .27, .38 + dy * 2).dy);
      }
    }
    final t = p(.90, .78);
    l.moveTo(t.dx, t.dy);
    l.cubicTo(p(1.0, .72).dx, p(1.0, .72).dy, p(1.0, .5).dx, p(1.0, .5).dy, p(.94, .42).dx, p(.94, .42).dy);
    l.moveTo(p(.34, .86).dx, p(.34, .86).dy);
    l.lineTo(p(.34, .96).dx, p(.34, .96).dy);
    l.moveTo(p(.66, .86).dx, p(.66, .86).dy);
    l.lineTo(p(.66, .96).dx, p(.66, .96).dy);
    return l;
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final g = _g(s);
    final dz = Rect.fromLTRB(g.pane.right, g.a.top, g.a.right + 10, g.a.bottom);
    if (dz.contains(p)) return 3;
    if (wdFat(g.over).contains(p)) return 1;
    if (g.pane.contains(p)) return 0;
    if (wdFat(g.cat).contains(p)) return 2;
    if (wdFat(g.pane, 24).contains(p)) return 0;
    return null;
  }

  @override
  void down(int z, Offset p, Size s) {
    _acc = 0;
    _moved = 0;
    _modeAt = doc.base('mode');
    _clipAt = doc.base('clip') > .5;
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0;
    final a = art(s);
    switch (z) {
      case 0:
        doc.set('op', doc.base('op') - d.dy * k * 100 / (a.height * .9));
      case 1:
        _acc += d.dx * (fine ? .4 : 1);
        doc.set('mode', (_modeAt + _acc / 28).round().toDouble());
      case 2:
        _moved += d.distance;
        _acc += d.dx;
        if (_moved > 6 && _acc > 10) doc.set('clip', 1);
        if (_moved > 6 && _acc < -10) doc.set('clip', 0);
      case 3:
        doc.set('ghost', doc.base('ghost') + d.dx * k * .5);
    }
  }

  @override
  void up(int z) {
    if (z == 2 && _moved <= 6) doc.set('clip', _clipAt ? 0 : 1);
  }

  @override
  void paint(Canvas c, Size s) {
    final g = _g(s);
    final op = doc.eff('op') / 100, ghost = doc.eff('ghost') / 100;
    final mode = doc.eff('mode').round().clamp(0, 3);
    final clip = doc.base('clip') > .5;
    final pane = g.pane.shift(_drift());
    final rr = RRect.fromRectAndRadius(pane, const Radius.circular(5));
    final body = _body(g.cat), lines = _lines(g.cat);

    // the cat, whole and fine, as the layer below
    final fig = wdStroke(N.g76.withValues(alpha: .8));
    c.drawPath(body, fig);
    c.drawPath(lines, wdStroke(N.g56.withValues(alpha: .6)));
    for (final sgn in const [-1.0, 1.0]) {
      c.drawCircle(Offset(g.cat.left + g.cat.width * (.5 + sgn * .09), g.cat.top + g.cat.height * .32), 1.4, wdFill(zc(2)));
    }
    if (clip) c.drawPath(body, wdStroke(zc(2, .8)));

    // ghosts: faint copies of the pane's edge behind its drift
    final vel = _vel();
    for (var k = 2; k >= 1; k--) {
      c.drawRRect(rr.shift(-vel * (ghost * 18.0 * k)), wdStroke(zc(3, ghost * (k == 1 ? .8 : .45))));
    }

    // the pane: a thin glaze and frost grain; the cat behind it is mixed by the blend mode
    c.save();
    c.clipRRect(rr);
    c.drawRect(pane, wdFill(N.glaze9.withValues(alpha: .09 * op)));
    final (lineCol, lineW, grainCol) = switch (mode) {
      1 => (N.g100.withValues(alpha: .95), 1.5, N.g95),
      2 => (N.g38.withValues(alpha: .9), 1.0, N.g26),
      3 => (N.g91.withValues(alpha: .65), 1.0, N.g76),
      _ => (N.g76.withValues(alpha: (.8 * (1 - .85 * op)).clamp(.05, 1.0)), 1.0, N.g63),
    };
    // behind the frost the cat is redrawn in the mixed tone (covering its plain line)
    if (mode != 0 || op > .02) {
      c.save();
      c.clipRect(g.cat);
      if (mode == 2) c.drawPath(body, wdStroke(N.g10, 2.2)); // multiply: the line is darkened over
      if (mode != 2) c.drawPath(body, wdStroke(N.g10.withValues(alpha: .85 * op), 2.2));
      c.drawPath(body, wdStroke(lineCol, lineW));
      c.drawPath(lines, wdStroke(lineCol.withValues(alpha: lineCol.a * .7)));
      c.restore();
    }
    // the grain: the cell is lit when its hash is under the density
    c.save();
    if (clip) c.clipPath(body);
    final dot = wdFill(grainCol.withValues(alpha: .5));
    const step = 4.0;
    final nx = (pane.width / step).floor(), ny = (pane.height / step).floor();
    for (var ix = 0; ix < nx; ix++) {
      for (var iy = 0; iy < ny; iy++) {
        final h = ((ix * 73856093) ^ (iy * 19349663)) & 0xffff;
        if (h / 65535 < op * .85) c.drawCircle(Offset(pane.left + 2 + ix * step, pane.top + 2 + iy * step), .6, dot);
      }
    }
    c.restore();
    c.restore();

    // the pane edge (A), the overlap (B), the orange tick (D)
    if (clip) {
      wdDashPath(c, Path()..addRRect(rr), wdStroke(zc(0)), on: 2, off: 3);
    } else {
      c.drawRRect(rr, wdStroke(zc(0)));
    }
    final ov = g.cat.intersect(pane);
    if (ov.width > 0 && ov.height > 0) c.drawRect(ov.deflate(.5), wdStroke(zc(1, .7)));
    c.drawLine(pane.topRight + const Offset(-9, 6), pane.topRight + const Offset(-3, 6), wdStroke(zc(3)));

    hint(c, g.pane, 0);
    hint(c, wdFat(g.over), 1);
    hint(c, wdFat(g.cat), 2);
    hint(c, Rect.fromLTRB(g.pane.right, g.a.top, g.a.right + 10, g.a.bottom), 3);
  }
}
