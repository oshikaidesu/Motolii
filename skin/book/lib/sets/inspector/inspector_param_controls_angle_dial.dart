// A dial for an angle: turn it by moving around it (the farther out, the finer), or scrub the number beside it.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// A dial for an angle: turn it by moving around it (the farther out, the finer), or scrub the number beside it.
/// contract: one turn of the pointer is one turn of the value, so the dial never lies about direction; one undo per drag.
class AngleDial extends StatefulWidget {
  const AngleDial({super.key, required this.id, required this.p});
  final String id;
  final Param p;
  @override
  State<AngleDial> createState() => _AngleDialState();
}

class _AngleDialState extends State<AngleDial> {
  double? _last;
  bool _hot = false;
  double _angle(Offset local) => math.atan2(local.dy - 11, local.dx - 11);
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, v = doc.get(widget.id) ?? 0;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    return MouseRegion(
      cursor: on ? SystemMouseCursors.grab : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => _hot = true),
      onExit: (_) => setState(() => _hot = false),
      child: Listener(
        onPointerDown: on
            ? (e) {
                _last = _angle(e.localPosition);
                doc.beginGesture();
              }
            : null,
        onPointerMove: (e) {
          final last = _last;
          if (last == null) return;
          final a = _angle(e.localPosition);
          var d = a - last;
          if (d > math.pi) d -= 2 * math.pi;
          if (d < -math.pi) d += 2 * math.pi;
          _last = a;
          final lo = widget.p.min ?? double.negativeInfinity, hi = widget.p.max ?? double.infinity;
          doc.set(widget.id, ((doc.get(widget.id) ?? 0) + d * 180 / math.pi).clamp(lo, hi).roundToDouble());
        },
        onPointerUp: (_) {
          if (_last != null) doc.endGesture();
          _last = null;
        },
        onPointerCancel: (_) {
          if (_last != null) doc.endGesture();
          _last = null;
        },
        child: CustomPaint(size: const Size(22, 22), painter: _DialPaint(v, _hot || _last != null)),
      ),
    );
  }
}

class _DialPaint extends CustomPainter {
  const _DialPaint(this.deg, this.hot);
  final double deg;
  final bool hot;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), r = 9.0;
    c.drawCircle(o, r, Paint()..color = hot ? Grey.g38 : Grey.g26);
    final a = deg * math.pi / 180;
    c.drawArc(
      Rect.fromCircle(center: o, radius: r - 1.5),
      -math.pi / 2,
      a.clamp(-2 * math.pi, 2 * math.pi),
      false,
      Paint()
        ..color = Grey.g63
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5,
    );
    final tip = o + Offset(math.sin(a), -math.cos(a)) * (r - 1);
    c.drawLine(
      o,
      tip,
      Paint()
        ..color = Grey.g95
        ..strokeWidth = 2
        ..strokeCap = StrokeCap.round,
    );
    c.drawCircle(o, 1.8, Paint()..color = Grey.g95);
  }

  @override
  bool shouldRepaint(_DialPaint o) => o.deg != deg || o.hot != hot;
}
