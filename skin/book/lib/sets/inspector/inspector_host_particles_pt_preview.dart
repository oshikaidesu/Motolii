// A still of the stream as it would stand once full: dots at evenly spread ages, each with the faint path it came along.
import 'inspector_studies.dart';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// A still of the stream as it would stand once full: dots at evenly spread ages, each with the faint path it came along.
/// contract: drawn from the handles only when they change; no ticker, no per-frame work.
class PtPreview extends StatelessWidget {
  const PtPreview({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, q = _ptQ(doc);
    final alive = math.min(q[0] * q[1], 100000);
    return Container(
      height: 76,
      margin: const EdgeInsets.only(bottom: 2),
      decoration: BoxDecoration(color: Grey.g10, borderRadius: BorderRadius.circular(6)),
      child: Stack(
        children: [
          Positioned.fill(
            child: ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: CustomPaint(painter: _SprayPaint(q, hexControls(doc.s2['pt_color']) ?? Grey.g95, hexControls(doc.s2['pt_color_end']) ?? Grey.g95)),
            ),
          ),
          Positioned(left: 7, top: 6, child: Text('${alive.round()} alive', style: T.label(Grey.g56))),
        ],
      ),
    );
  }
}

/// The handles the pictures read, packed in one list so a repaint is one compare.
List<double> _ptQ(Doc d) => [
  for (final id in const [
    'rate', 'life', 'life_random', 'emitter.x', 'emitter.y', 'direction', 'spread', 'speed', 'speed_random', 'size', 'size_end', //
    'opacity_end', 'gravity', 'wind.x', 'wind.y', 'bounce', 'floor', 'turbulence', 'turbulence_size', 'seed', 'connect', 'line_width', 'line_opacity',
  ])
    d.get('pt_$id') ?? 0,
];

class _SprayPaint extends CustomPainter {
  const _SprayPaint(this.q, this.c0, this.c1);
  final List<double> q;
  final Color c0, c1;
  static const n = 36, steps = 8;

  @override
  void paint(Canvas c, Size s) {
    final [_, life0, lifeR, ex, ey, dir, spread, speed, speedR, size0, size1, op1, grav, wx, wy, bounce, floor, turb, turbS, seedV, connect, lw, lop] = q;
    final life = math.max(life0, .01), seed = seedV.round(), acc = Offset(wx, wy + grav);
    final trails = <List<Offset>>[], ages = <double>[];
    final f = 120 / math.max(turbS, 1);
    for (var i = 0; i < n; i++) {
      double u(int ch) => _ptU(seed, i, ch);
      final li = life * (1 - lifeR / 100 * (u(0) * .5 + .5)), age = life * (i + .5) / n;
      if (age >= li) continue;
      final a = (dir + spread * .5 * u(1)) * math.pi / 180, sp = speed * (1 + speedR / 100 * u(2));
      final p0 = Offset(ex * .5 * u(3), ey * .5 * u(4)), v0 = Offset(math.cos(a), math.sin(a)) * sp;
      Offset at(double t) =>
          _ptFly(p0, v0, acc, t, bounce / 100, floor) + Offset(math.sin(i * 1.7 + t * 3 * f), math.cos(i * 2.3 + t * 2.1 * f)) * (turb * t / life);
      trails.add([for (var k = 0; k <= steps; k++) at(age * k / steps)]);
      ages.add(age / li);
    }
    var r = Rect.fromCenter(center: Offset.zero, width: math.max(ex, 1), height: math.max(ey, 1));
    for (final t in trails) {
      for (final p in t) {
        r = r.expandToInclude(Rect.fromCenter(center: p, width: 0, height: 0));
      }
    }
    if (bounce > 0 && floor >= r.top) r = r.expandToInclude(Rect.fromLTWH(r.left, floor, 0, 0));
    r = Rect.fromCenter(center: r.center, width: math.max(r.width, 80), height: math.max(r.height, 40));
    final k = math.min((s.width - 20) / r.width, (s.height - 14) / r.height);
    final o = s.center(Offset.zero) - r.center * k;
    Offset m(Offset p) => o + p * k;
    if (bounce > 0) {
      final y = o.dy + floor * k;
      if (y > 0 && y < s.height) c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = Grey.g26);
    }
    final o0 = m(Offset.zero), d0 = dir * math.pi / 180, sw = spread.clamp(0.0, 360.0) * math.pi / 180;
    c.drawArc(Rect.fromCircle(center: o0, radius: 16), d0 - sw / 2, math.max(sw, .02), true, Paint()..color = Pop.toneParticles.withValues(alpha: .16));
    if (ex > 0 || ey > 0) {
      final er = Rect.fromCenter(center: o0, width: math.max(ex * k, 1), height: math.max(ey * k, 1));
      c.drawRect(
        er,
        Paint()
          ..color = Grey.g44
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
    final dots = <Offset>[];
    final mx = math.max(math.max(size0, size1), 1e-6);
    for (var i = 0; i < trails.length; i++) {
      final t = ages[i], pts = trails[i].map(m).toList(), col = Color.lerp(c0, c1, t)!, alpha = 1 + (op1 / 100 - 1) * t;
      c.drawPath(
        Path()..addPolygon(pts, false),
        Paint()
          ..color = col.withValues(alpha: .14 * alpha)
          ..style = PaintingStyle.stroke
          ..strokeWidth = .8,
      );
      final rad = .7 + 2.3 * (size0 + (size1 - size0) * t) / mx;
      dots.add(pts.last);
      c.drawCircle(pts.last, rad, Paint()..color = col.withValues(alpha: (.25 + .75 * alpha).clamp(0.0, 1.0)));
    }
    if (connect > 0) {
      final line = Paint()..strokeWidth = (lw * .5).clamp(.5, 2.0);
      final dk = connect * k;
      for (var i = 0; i < dots.length; i++) {
        for (var j = i + 1; j < dots.length; j++) {
          final d = (dots[i] - dots[j]).distance;
          if (d < dk) c.drawLine(dots[i], dots[j], line..color = c0.withValues(alpha: lop / 100 * (1 - d / dk) * .8));
        }
      }
    }
    c.drawCircle(o0, 2, Paint()..color = Pop.toneParticles);
  }

  @override
  bool shouldRepaint(_SprayPaint o) => !_ptSame(o.q, q) || o.c0 != c0 || o.c1 != c1;
}

/// A stand-in for the layer's noise(seed, index, channel): -1..1, the same for the same inputs.
double _ptU(int seed, int i, int ch) {
  var h = (seed * 73856093) ^ (i * 19349663) ^ (ch * 83492791) ^ 0x5bd1e995;
  h = ((h ^ (h >> 15)) * 0x2c1b3c6d) & 0xFFFFFFFF;
  h = ((h ^ (h >> 12)) * 0x297a2d39) & 0xFFFFFFFF;
  h ^= h >> 15;
  return (h & 0xFFFF) / 0xFFFF * 2 - 1;
}

bool _ptSame(List<double> a, List<double> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

/// The layer's ballistic(): constant acceleration, bouncing off y = floor (down is +) only while bounce > 0.
Offset _ptFly(Offset p0, Offset v0, Offset a, double age, double bounce, double floor) {
  Offset at(Offset p, Offset v, double t) => p + v * t + a * (.5 * t * t);
  if (bounce <= 0 || p0.dy > floor) return at(p0, v0, age);
  var p = p0, v = v0, left = age;
  for (var n = 0; n < 32; n++) {
    final qa = .5 * a.dy, qb = v.dy, qc = p.dy - floor;
    var hit = double.infinity;
    if (qa.abs() < 1e-9) {
      if (qb > 1e-9) hit = -qc / qb;
    } else {
      final disc = qb * qb - 4 * qa * qc;
      if (disc >= 0) {
        final r = math.sqrt(disc);
        for (final t in [(-qb - r) / (2 * qa), (-qb + r) / (2 * qa)]) {
          if (t > 1e-6 && t < hit) hit = t;
        }
      }
    }
    if (hit >= left) return at(p, v, left);
    p = Offset(at(p, v, hit).dx, floor);
    v = Offset(v.dx + a.dx * hit, -(v.dy + a.dy * hit) * bounce);
    left -= hit;
    if (v.dy.abs() < 1) return Offset(p.dx + v.dx * left + .5 * a.dx * left * left, floor);
  }
  return p;
}
