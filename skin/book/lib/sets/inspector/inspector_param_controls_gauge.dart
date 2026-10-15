// The value as a small live picture beside its label: a pie for a share, a level for a height, a dot for a point.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// The value as a small live picture beside its label: a pie for a share, a level for a height, a dot for a point.
class Gauge extends StatelessWidget {
  const Gauge(this.kind, this.v, this.p, {super.key, this.x, this.y});
  final GaugeKind kind;
  final double? v, x, y;
  final Param? p;
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(22, 22), painter: _GaugePaint(kind, v, p, x, y));
}

class _GaugePaint extends CustomPainter {
  const _GaugePaint(this.kind, this.v, this.p, this.x, this.y);
  final GaugeKind kind;
  final double? v, x, y;
  final Param? p;

  double _t(double? val, [double? lo, double? hi]) {
    final a = lo ?? p?.min, b = hi ?? p?.max;
    if (val == null || a == null || b == null || b == a) return 0;
    return ((val - a) / (b - a)).clamp(0.0, 1.0);
  }

  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero);
    final dim = Paint()..color = Grey.g26;
    final lit = Paint()..color = Grey.g91;
    final line = Paint()
      ..color = Grey.g56
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    switch (kind) {
      case GaugeKind.pie:
        final r = Rect.fromCircle(center: o, radius: 8);
        c.drawCircle(o, 8, dim);
        final t = (v ?? 0) / ((p?.max ?? 100) == 0 ? 100 : (p!.max!.abs() > 100 ? p!.max!.abs() : 100));
        c.drawArc(r, -math.pi / 2, 2 * math.pi * t.clamp(-1.0, 1.0), true, lit);
        if (t.abs() > 1) c.drawCircle(o, 8, line..color = Grey.g91);
      case GaugeKind.up || GaugeKind.down:
        final box = Rect.fromCenter(center: o, width: 8, height: 18);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(2)), dim);
        final h = box.height * _t(v);
        final fill = kind == GaugeKind.up
            ? Rect.fromLTRB(box.left, box.bottom - h, box.right, box.bottom)
            : Rect.fromLTRB(box.left, box.top, box.right, box.top + h);
        c.drawRRect(RRect.fromRectAndRadius(fill, const Radius.circular(2)), lit);
        final tip = kind == GaugeKind.up ? box.top - 1 : box.bottom + 1;
        c.drawLine(Offset(o.dx - 2.5, tip + (kind == GaugeKind.up ? 2.5 : -2.5)), Offset(o.dx, tip), line);
        c.drawLine(Offset(o.dx + 2.5, tip + (kind == GaugeKind.up ? 2.5 : -2.5)), Offset(o.dx, tip), line);
      case GaugeKind.across:
        final box = Rect.fromCenter(center: o, width: 18, height: 8);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(2)), dim);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(box.left, box.top, box.width * _t(v), box.height), const Radius.circular(2)), lit);
      case GaugeKind.centre:
        final box = Rect.fromCenter(center: o, width: 18, height: 8);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(2)), dim);
        final z = box.left + box.width * _t(0), at = box.left + box.width * _t(v);
        c.drawRect(Rect.fromLTRB(math.min(z, at), box.top, math.max(z, at), box.bottom), lit);
        c.drawLine(Offset(z, box.top - 2), Offset(z, box.bottom + 2), line);
      case GaugeKind.ramp:
        final t = _t(v);
        for (var i = 0; i < 5; i++) {
          final h = 4.0 + i * 3;
          final r = Rect.fromLTWH(o.dx - 9 + i * 3.8, o.dy + 8 - h, 2.6, h);
          c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(1)), (i + .5) / 5 <= t ? lit : dim);
        }
      case GaugeKind.free:
        c.drawLine(Offset(o.dx - 7, o.dy), Offset(o.dx + 7, o.dy), line);
        for (final d in const [-1.0, 1.0]) {
          final tip = Offset(o.dx + d * 8, o.dy);
          c.drawLine(tip, tip + Offset(-d * 3, -3), line);
          c.drawLine(tip, tip + Offset(-d * 3, 3), line);
        }
      case GaugeKind.count:
        final n = ((p?.max ?? 1) - (p?.min ?? 0)).round() + 1, k = ((v ?? 0) - (p?.min ?? 0)).round();
        final cols = math.min(n, 4), rows = (n / cols).ceil().clamp(1, 3);
        for (var i = 0; i < math.min(n, cols * rows); i++) {
          final cx = o.dx + (i % cols - (cols - 1) / 2) * 4.6, cy = o.dy + (i ~/ cols - (rows - 1) / 2) * 4.6;
          c.drawCircle(Offset(cx, cy), 1.6, i <= k ? lit : dim);
        }
      case GaugeKind.point:
        final box = Rect.fromCenter(center: o, width: 18, height: 18);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(3)), dim);
        final px = box.left + box.width * _t(x), py = box.top + box.height * _t(y);
        c.drawLine(Offset(px, box.top), Offset(px, box.bottom), line);
        c.drawLine(Offset(box.left, py), Offset(box.right, py), line);
        c.drawCircle(Offset(px, py), 2.6, lit);
      case GaugeKind.box:
        final mx = math.max(x ?? 1, y ?? 1).abs();
        final w = mx == 0 ? 0.0 : 18 * (x ?? 0).abs() / mx, h = mx == 0 ? 0.0 : 18 * (y ?? 0).abs() / mx;
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: o, width: math.max(w, 1.5), height: math.max(h, 1.5)), const Radius.circular(2)), lit);
    }
  }

  @override
  bool shouldRepaint(_GaugePaint o) => o.kind != kind || o.v != v || o.x != x || o.y != y || o.p != p;
}
