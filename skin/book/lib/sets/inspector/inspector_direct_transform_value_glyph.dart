// The row's current value as a picture: where it sits, how big, which way, how solid.
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// The row's current value as a picture: where it sits, how big, which way, how solid.
class ValueGlyph extends StatelessWidget {
  const ValueGlyph(this.shape, {super.key});
  final ValueShape shape;
  static const size = 20.0;

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    final ids = switch (shape) {
      ValueShape.position => ['v_x', 'v_y'],
      ValueShape.scale => ['v_scale_x', 'v_scale_y'],
      ValueShape.rotation => ['v_rotate'],
      ValueShape.opacity => ['v_opacity'],
      ValueShape.depth => ['depth'],
      ValueShape.parent => ['v_parent'],
    };
    final values = shape == ValueShape.parent ? [(doc.s2['v_parent'] ?? 'None') == 'None' ? 0.0 : 1.0] : [for (final id in ids) doc.get(id)];
    final hot = shape.zone != null && doc.directHover == shape.zone!.prop;
    return CustomPaint(
      size: const Size(24, size),
      painter: _GlyphPaint(shape, values.contains(null) ? null : values.cast<double>(), hot ? shape.zone!.color : Grey.g76),
    );
  }
}

class _GlyphPaint extends CustomPainter {
  const _GlyphPaint(this.shape, this.values, this.ink);
  final ValueShape shape;
  final List<double>? values;
  final Color ink;

  @override
  void paint(Canvas c, Size s) {
    final mid = s.center(Offset.zero);
    final rim = Paint()
      ..color = Grey.g44
      ..style = PaintingStyle.stroke;
    final solid = Paint()..color = ink;
    final v = values;
    if (v == null) {
      c.drawLine(mid - const Offset(4, 0), mid + const Offset(4, 0), rim);
      return;
    }
    switch (shape) {
      case ValueShape.position:
        final f = Rect.fromCenter(center: mid, width: 20, height: 11.25);
        c.drawRect(f, rim);
        final p = Offset(f.left + (v[0] / 1920).clamp(0, 1) * f.width, f.top + (v[1] / 1080).clamp(0, 1) * f.height);
        c.drawCircle(p, 2.5, solid);
      case ValueShape.scale:
        // only the proportion shows: a uniform scale keeps the square
        const side = 14.0;
        final sx = v[0].abs(), sy = v[1].abs(), big = math.max(sx, sy);
        c.drawRect(
          Rect.fromCenter(center: mid, width: big == 0 ? 1 : math.max(1.5, side * sx / big), height: big == 0 ? 1 : math.max(1.5, side * sy / big)),
          solid..color = ink.withValues(alpha: .85),
        );
      case ValueShape.rotation:
        c.drawCircle(mid, 8, rim);
        final a = v[0] * math.pi / 180 - math.pi / 2;
        c.drawLine(
          mid,
          mid + Offset(math.cos(a), math.sin(a)) * 8,
          Paint()
            ..color = ink
            ..strokeWidth = 2
            ..strokeCap = StrokeCap.round,
        );
        c.drawCircle(mid, 1.5, solid);
        final turns = (v[0].abs() / 360).floor().clamp(0, 3);
        for (var i = 0; i < turns; i++) {
          c.drawCircle(Offset(s.width - 1.5, 2 + i * 4.0), 1.2, solid);
        }
      case ValueShape.opacity:
        final r = Rect.fromCenter(center: mid, width: 14, height: 14);
        const cell = 3.5;
        for (var y = 0; y < 4; y++) {
          for (var x = 0; x < 4; x++) {
            c.drawRect(Rect.fromLTWH(r.left + x * cell, r.top + y * cell, cell, cell), Paint()..color = (x + y).isEven ? Grey.g26 : Grey.g15);
          }
        }
        c.drawRect(r, Paint()..color = ink.withValues(alpha: (v[0] / 100).clamp(0, 1)));
        c.drawRect(r, rim);
      case ValueShape.depth:
        final off = (v[0] / 25).clamp(0.0, 5.0);
        final front = Rect.fromCenter(center: mid + Offset(-off / 2, off / 2), width: 11, height: 11);
        if (off > 0) c.drawRect(front.shift(Offset(off, -off)), rim);
        c.drawRect(front, solid..color = ink.withValues(alpha: .85));
      case ValueShape.parent:
        final child = Rect.fromCenter(center: mid + const Offset(3, 3), width: 9, height: 9);
        if (v[0] > 0) {
          final top = mid + const Offset(-5, -5);
          c.drawLine(top, Offset(top.dx, child.center.dy), rim..color = Grey.g56);
          c.drawLine(Offset(top.dx, child.center.dy), child.centerLeft, rim);
          c.drawCircle(top, 2.5, solid);
        }
        c.drawRect(
          child,
          v[0] > 0
              ? (Paint()..color = ink.withValues(alpha: .85))
              : (Paint()
                  ..color = Grey.g44
                  ..style = PaintingStyle.stroke),
        );
    }
  }

  @override
  bool shouldRepaint(_GlyphPaint old) => old.shape != shape || old.ink != ink || !listEquals(old.values, values);
}
