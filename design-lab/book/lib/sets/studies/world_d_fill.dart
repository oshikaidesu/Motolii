// World D 22: Fill. Sheets of pigment fanned out on dotted paper like a pile of cards; the paper and the lower sheets show through as it thins.
// A Colour (blue): the top sheet (its dot). B Opacity (green): the paper and its crop marks (more paint, fewer dots). C Gradient angle (white): the
// top sheet's edge; drag it round the pile and the fan swings. D Stops (orange): the sheets below, one dot each; a new sheet slides out and fades in.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_d_kit.dart';

const fillSpecs = [
  WdSpec('hue', 'Colour', 0, 360, 335, unit: '°', digits: 0),
  WdSpec('op', 'Opacity', 0, 100, 80, unit: '%', digits: 0),
  WdSpec('ang', 'Gradient angle', 0, 360, 90, unit: '°', digits: 0, amp: .08),
  WdSpec('stops', 'Stops', 2, 5, 2, digits: 0, amp: .22),
];

class FillMini extends WdMini {
  FillMini(super.doc);

  @override
  List<List<String>> get zoneIds => const [['hue'], ['op'], ['ang'], ['stops']];

  /// The pigment's true colour (readout, the one small accent); the sheets themselves are only tinted towards it, in the calm grey family.
  Color _pure(double hue) => HSLColor.fromAHSL(1, hue % 360, .50, .58).toColor();
  Color _c0(double hue) => Color.lerp(N.g44, _pure(hue), .07)!;
  Color _c1(double hue) => Color.lerp(N.g15, _pure(hue), .05)!;

  @override
  List<WdR> readout() {
    final c = _pure(doc.eff('hue'));
    String hx(double v) => (v * 255).round().toRadixString(16).padLeft(2, '0');
    return [
      ('#${hx(c.r)}${hx(c.g)}${hx(c.b)}'.toUpperCase(), doc.changed('hue')),
      (doc.show('op'), doc.changed('op')),
      (doc.show('ang'), doc.changed('ang')),
      (doc.eff('stops').round().toString(), doc.changed('stops')),
    ];
  }

  double _theta() => (doc.eff('ang') + math.sin(doc.time * 2 * math.pi / 5) * 2.5) * math.pi / 180;

  /// The pile: [n] sheets, sheet i centred at centre + u * (i - (n-1)/2) * step; the last one is on top.
  ({Rect sw, Offset u, double step, int n, double rx, double ry}) _pile(Size s) {
    final a = art(s);
    final sw = a.deflate(math.min(14.0, a.shortestSide * .14));
    final th = _theta(), u = Offset(math.cos(th), math.sin(th));
    final n = doc.eff('stops').ceil().clamp(2, 5);
    final step = math.min(sw.width, sw.height) * .13;
    final rx = sw.width / 2 - 1.0 * step * u.dx.abs() - 2, ry = sw.height / 2 - 1.0 * step * u.dy.abs() - 2;
    return (sw: sw, u: u, step: step, n: n, rx: rx, ry: ry);
  }

  Offset _centre(({Rect sw, Offset u, double step, int n, double rx, double ry}) g, int i) => g.sw.center + g.u * ((i - (g.n - 1) / 2) * g.step);

  Path _blob(Offset c, double rx, double ry, double ph) {
    final p = Path();
    for (var i = 0; i <= 56; i++) {
      final t = i / 56 * 2 * math.pi;
      final r = 1 + .06 * math.sin(3 * t + ph) + .035 * math.sin(5 * t + 2 * ph + doc.time * .4);
      final q = Offset(c.dx + math.cos(t) * rx * r * .96, c.dy + math.sin(t) * ry * r * .96);
      i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    return p..close();
  }

  Path _sheet(({Rect sw, Offset u, double step, int n, double rx, double ry}) g, int i) => _blob(_centre(g, i), g.rx, g.ry, 1.0 + i * .5);

  @override
  int? zoneAt(Offset p, Size s) {
    final g = _pile(s);
    final top = _sheet(g, g.n - 1);
    final cTop = _centre(g, g.n - 1);
    final inTop = top.contains(p);
    final nearEdge = (p - cTop).distance;
    // the top sheet: its rim is the angle, its core is the colour
    final rim = inTop && !_blobDeflated(g, cTop, 8).contains(p);
    if (inTop && !rim) return 0;
    if (rim) return 2;
    for (var i = 0; i < g.n - 1; i++) {
      if (_sheet(g, i).contains(p) || _blobInflated(g, i, 7).contains(p)) return 3;
    }
    if (nearEdge >= 0 && _blobInflated(g, g.n - 1, 7).contains(p)) return 2;
    return 1;
  }

  Path _blobDeflated(({Rect sw, Offset u, double step, int n, double rx, double ry}) g, Offset c, double d) =>
      _blob(c, math.max(4, g.rx - d), math.max(4, g.ry - d), 1.0 + (g.n - 1) * .5);
  Path _blobInflated(({Rect sw, Offset u, double step, int n, double rx, double ry}) g, int i, double d) => _blob(_centre(g, i), g.rx + d, g.ry + d, 1.0 + i * .5);

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s), g = _pile(s);
    switch (z) {
      case 0:
        doc.set('hue', doc.base('hue') + d.dx * k * 1.1);
      case 1:
        doc.set('op', doc.base('op') - d.dy * k * 100 / (a.height * .7));
      case 2:
        final r = p - g.sw.center, l2 = math.max(100.0, r.dx * r.dx + r.dy * r.dy);
        doc.set('ang', doc.base('ang') + (r.dx * d.dy - r.dy * d.dx) / l2 * 180 / math.pi * k);
      case 3:
        doc.set('stops', doc.base('stops') + (d.dx * g.u.dx + d.dy * g.u.dy) * k / 18);
    }
  }

  @override
  void up(int z) {
    if (z == 3) doc.set('stops', doc.base('stops').roundToDouble());
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), g = _pile(s);
    final hue = doc.eff('hue'), op = doc.eff('op') / 100, st = doc.eff('stops');
    final c0 = _c0(hue), c1 = _c1(hue);
    final v = Offset(-g.u.dy, g.u.dx);

    // the paper: a fine dot lattice, and green crop marks at its corners
    final dot = wdFill(N.g26);
    for (var x = a.left + 4; x < a.right; x += 6) {
      for (var y = a.top + 4; y < a.bottom; y += 6) {
        c.drawCircle(Offset(x, y), .7, dot);
      }
    }
    final crop = wdStroke(zc(1));
    for (final sx in const [-1, 1]) {
      for (final sy in const [-1, 1]) {
        final o = Offset(sx < 0 ? a.left + 2 : a.right - 2, sy < 0 ? a.top + 2 : a.bottom - 2);
        c.drawLine(o, o + Offset(-sx * 7.0, 0), crop);
        c.drawLine(o, o + Offset(0, -sy * 7.0), crop);
      }
    }

    // the sheets, back to front; the sheet that is still arriving slides out of the one on top of it and fades in
    for (var i = 0; i < g.n; i++) {
      final f = i == g.n - 1 && i >= 2 ? wdClamp(st - i, 0, 1) : 1.0; // sheets beyond the first two arrive one by one
      final arriving = i == g.n - 1 && i >= 2 ? wdSmooth(f) : 1.0;
      // draw order: the highest index is the top; centres run along u
      final ci = _centre(g, i) - g.u * ((1 - arriving) * g.step);
      final col = Color.lerp(c0, c1, g.n == 1 ? 0 : 1 - i / (g.n - 1))!; // the top sheet carries the colour of A
      final path = _blob(ci, g.rx, g.ry, 1.0 + i * .5);
      c.drawPath(path, wdFill(col.withValues(alpha: op * .7 * (.35 + .65 * f))));
      final top = i == g.n - 1;
      c.drawPath(path, wdStroke(top ? zc(2, .7) : N.g95.withValues(alpha: .16 * f)));
      if (!top) {
        final ext = 1 / math.sqrt(math.pow(g.u.dx / g.rx, 2) + math.pow(g.u.dy / g.ry, 2)) * .8;
        final o = ci - g.u * ext + v * ((i.isEven ? -1 : 1) * math.min(g.rx, g.ry) * .3);
        c.drawCircle(o, 2, wdFill(zc(3, f)));
      }
    }
    c.drawCircle(_centre(g, g.n - 1), 3.2, wdFill(_pure(hue)));
    c.drawCircle(_centre(g, g.n - 1), 5.5, wdStroke(zc(0)));

    if (view.hot == 0 || view.grab == 0) c.drawPath(_blobDeflated(g, _centre(g, g.n - 1), 10), wdStroke(wdSlot(0).withValues(alpha: .28)));
    hint(c, a, 1);
  }
}
