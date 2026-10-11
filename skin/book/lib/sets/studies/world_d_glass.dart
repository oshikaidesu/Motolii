// World D 19: Glass. A beam of light crosses a wedge of glass and bends; you grab the bend, the grain, the depth and the rainbow.
// A IOR (blue): the angle arc and the beam inside the glass. B Roughness (green): the scuffed entry face and its scattered beams.
// C Thickness (white): the far face of the wedge. D Dispersion (orange): the exit beam splits.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_d_kit.dart';

const glassSpecs = [
  WdSpec('ior', 'IOR', 1, 2.5, 1.5),
  WdSpec('rough', 'Roughness', 0, 1, .1),
  WdSpec('thick', 'Thickness', 0, 100, 20, unit: 'px', digits: 0),
  WdSpec('disp', 'Dispersion', 0, 1, 0),
];

class _Ray {
  _Ray(this.start, this.p, this.q, this.end);
  final Offset start, p, q, end;
}

class GlassMini extends WdMini {
  GlassMini(super.doc);

  @override
  List<List<String>> get zoneIds => const [['ior'], ['rough'], ['thick'], ['disp']];

  @override
  List<WdR> readout() => [for (final s in glassSpecs) (doc.show(s.id), doc.changed(s.id))];

  // ---- geometry ---------------------------------------------------------------------------------------------------------------------

  ({double x0, double x1t, double x1b, double top, double bot, Rect a}) _g(Size s, double thick) {
    final a = art(s);
    final x0 = a.left + a.width * .30, w = a.width * wdLerp(.15, .34, thick / 100);
    final top = a.top + 3, bot = a.bottom - 3;
    return (x0: x0, x1t: x0 + w, x1b: x0 + w + (bot - top) * .25, top: top, bot: bot, a: a);
  }

  static Offset _refract(Offset d, Offset n, double eta) {
    final cosi = -(n.dx * d.dx + n.dy * d.dy);
    final k = math.max(0.0, 1 - eta * eta * (1 - cosi * cosi));
    final r = d * eta + n * (eta * cosi - math.sqrt(k));
    return r / r.distance;
  }

  _Ray _trace(({double x0, double x1t, double x1b, double top, double bot, Rect a}) g, double theta, double n) {
    final h = g.bot - g.top, tilt = g.x1b - g.x1t;
    final py = g.a.top + g.a.height * .42;
    final d0 = Offset(math.cos(theta), math.sin(theta));
    final p = Offset(g.x0, py);
    final start = p - d0 * ((g.x0 - g.a.left) / d0.dx);
    final d1 = _refract(d0, const Offset(-1, 0), 1 / n);
    final e = Offset(tilt, h), a0 = Offset(g.x1t, g.top);
    final ap = a0 - p;
    final den = d1.dx * e.dy - d1.dy * e.dx;
    var t = den.abs() < 1e-6 ? 40.0 : (ap.dx * e.dy - ap.dy * e.dx) / den;
    t = wdClamp(t, 4, 600);
    var q = p + d1 * t;
    q = Offset(q.dx, wdClamp(q.dy, g.top, g.bot));
    final no = Offset(h, -tilt) / math.sqrt(h * h + tilt * tilt);
    final d2 = _refract(d1, -no, n);
    return _Ray(start, p, q, q + d2 * 600);
  }

  _Zones _z(Size s) {
    final g = _g(s, doc.eff('thick'));
    final xm = (g.x1t + g.x1b) / 2;
    return _Zones(
      a: wdFat(Rect.fromLTRB(g.x0 + 10, g.top, xm - 12, g.bot)),
      b: wdFat(Rect.fromLTRB(g.x0 - 14, g.top, g.x0 + 10, g.bot)),
      c: wdFat(Rect.fromLTRB(xm - 12, g.top, xm + 12, g.bot)),
      d: Rect.fromLTRB(g.x1b + 14, g.top, g.a.right + 10, g.bot),
    );
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final z = _z(s);
    if (z.c.contains(p)) return 2;
    if (z.b.contains(p)) return 1;
    if (z.a.contains(p)) return 0;
    if (z.d.contains(p)) return 3;
    return null;
  }

  // ---- gestures: zone-specific ------------------------------------------------------------------------------------------------------

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0;
    switch (z) {
      case 0:
        doc.set('ior', doc.base('ior') - d.dy * k * .009);
      case 1:
        doc.set('rough', doc.base('rough') - d.dy * k * .011);
      case 2:
        // soft-limited: the gain eases to a third at both ends, and a full sweep needs about 3/4 of the box width
        final u = doc.base('thick') / 100, gain = .3 + .7 * math.sin(math.pi * wdClamp(u, 0, 1));
        doc.set('thick', doc.base('thick') + d.dx * k * gain * 100 / (art(s).width * .75));
      case 3:
        doc.set('disp', doc.base('disp') + d.dx * k * .012);
    }
  }

  // ---- drawing ----------------------------------------------------------------------------------------------------------------------

  @override
  void paint(Canvas c, Size s) {
    final g = _g(s, doc.eff('thick'));
    final ior = doc.eff('ior'), rough = doc.eff('rough'), disp = doc.eff('disp');
    final theta = (22 + math.sin(doc.time * 2 * math.pi / 5.2) * 4) * math.pi / 180;
    final z = _z(s);
    c.save();
    c.clipRect(g.a.inflate(2));

    final poly = Path()
      ..moveTo(g.x0, g.top)
      ..lineTo(g.x1t, g.top)
      ..lineTo(g.x1b, g.bot)
      ..lineTo(g.x0, g.bot)
      ..close();
    c.drawPath(poly, wdFill(N.glaze9));
    wdDashLine(c, Offset(g.x0 - 18, g.a.top + g.a.height * .42), Offset(g.x0 + 18, g.a.top + g.a.height * .42), wdStroke(N.g38), on: 2, off: 2);

    // the beam, outside the glass
    final main = _trace(g, theta, ior);
    c.drawLine(main.start, main.p, wdStroke(N.g76.withValues(alpha: .9)));

    // B: scattered beams (grain)
    for (final k in const [-2, -1, 1, 2]) {
      final r = _trace(g, theta + k * rough * .10, ior);
      final pa = Paint()
        ..color = zc(1, .18 + .62 * rough)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1;
      c.drawLine(r.p, r.q, pa);
      c.drawLine(r.q, r.end, pa);
    }
    // B: the scuffed entry face
    final tick = wdStroke(zc(1));
    for (var i = 0; i < 9; i++) {
      final y = g.top + (i + .5) / 9 * (g.bot - g.top);
      final len = 1.5 + rough * 7 * (.5 + .5 * wdNoise(i * 1.7 + doc.time * .3, 2));
      c.drawLine(Offset(g.x0, y), Offset(g.x0 - len, y), tick);
    }

    // A: the beam inside and the angle it makes
    c.drawLine(main.p, main.q, wdStroke(zc(0)));
    final ang = math.atan2((main.q - main.p).dy, (main.q - main.p).dx);
    c.drawArc(Rect.fromCircle(center: main.p, radius: 13), 0, ang, false, wdStroke(zc(0)));

    // D: the exit splits (a tick always marks the place)
    final spread = disp * .30;
    for (final sgn in const [-1, 1]) {
      final r = _trace(g, theta, math.max(1.0, ior + sgn * spread));
      final pa = wdStroke(zc(3, math.min(1.0, disp * 3.5)));
      c.drawLine(r.p, r.q, wdStroke(zc(3, math.min(1.0, disp * 3.5) * .35)));
      c.drawLine(r.q, r.end, pa);
    }
    c.drawLine(main.q, main.end, wdStroke(N.g76.withValues(alpha: .9)));
    c.drawCircle(main.q, 1.8, wdFill(zc(3)));

    // C: the far face and the rest of the outline
    c.drawLine(Offset(g.x0, g.top), Offset(g.x1t, g.top), wdStroke(N.g38));
    c.drawLine(Offset(g.x0, g.bot), Offset(g.x1b, g.bot), wdStroke(N.g38));
    c.drawLine(Offset(g.x0, g.top), Offset(g.x0, g.bot), wdStroke(N.g56.withValues(alpha: .7)));
    c.drawLine(Offset(g.x1t, g.top), Offset(g.x1b, g.bot), wdStroke(zc(2)));

    c.restore();
    hint(c, z.a, 0);
    hint(c, z.b, 1);
    hint(c, z.c, 2);
    hint(c, z.d, 3);
  }
}

class _Zones {
  _Zones({required this.a, required this.b, required this.c, required this.d});
  final Rect a, b, c, d;
}
