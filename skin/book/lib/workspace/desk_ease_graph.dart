// The graph.

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

/// The graph: value up, time across. Handles are dragged; only a drag or hover repaints it.
class EaseGraph extends StatefulWidget {
  const EaseGraph({super.key, required this.ease, required this.span, required this.frame, required this.onChanged});
  final Ease ease;
  final EaseSpan? span;
  final int frame;
  final ValueChanged<Ease> onChanged;
  @override
  State<EaseGraph> createState() => _EaseGraphState();
}

class _EaseGraphState extends State<EaseGraph> {
  int? _drag, _hot;
  static const _ymin = -.42, _ymax = 1.62;

  static Rect _box(Size s) {
    final r = EdgeInsets.fromLTRB(Shape.s(30), Shape.s(10), Shape.s(14), Shape.s(22)).deflateRect(Offset.zero & s);
    return Rect.fromLTRB(r.left, r.top, math.max(r.left + 1, r.right), math.max(r.top + 1, r.bottom));
  }

  static Offset _px(Rect g, Offset v) => Offset(g.left + v.dx * g.width, g.bottom - (v.dy - _ymin) / (_ymax - _ymin) * g.height);
  static Offset _val(Rect g, Offset p) =>
      Offset(((p.dx - g.left) / g.width).clamp(0.0, 1.0), (_ymin + (g.bottom - p.dy) / g.height * (_ymax - _ymin)).clamp(_ymin + .02, _ymax - .02));

  int? _near(Rect g, Offset p) {
    final d1 = (_px(g, widget.ease.p1) - p).distance, d2 = (_px(g, widget.ease.p2) - p).distance;
    return math.min(d1, d2) > 14 ? null : (d1 <= d2 ? 0 : 1);
  }

  void _move(Rect g, Offset p) {
    final v = _val(g, p), e = widget.ease;
    widget.onChanged(_drag == 0 ? e.withHandles(v, e.p2) : e.withHandles(e.p1, v));
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final g = _box(box.biggest);
      return MouseRegion(
        cursor: _hot != null || _drag != null ? SystemMouseCursors.grab : MouseCursor.defer,
        onHover: (e) {
          final n = _near(g, e.localPosition);
          if (n != _hot) setState(() => _hot = n);
        },
        onExit: (_) {
          if (_hot != null) setState(() => _hot = null);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanDown: (d) {
            _drag = _near(g, d.localPosition);
            if (_drag != null) setState(() {});
          },
          onPanUpdate: (d) {
            if (_drag != null) _move(g, d.localPosition);
          },
          onPanEnd: (_) => setState(() => _drag = null),
          onPanCancel: () => setState(() => _drag = null),
          child: CustomPaint(
            size: box.biggest,
            painter: _GraphPainter(
              ease: widget.ease,
              from: widget.span?.from,
              to: widget.span?.to,
              frame: widget.frame,
              active: _drag ?? _hot,
              dragging: _drag != null,
            ),
          ),
        ),
      );
    },
  );
}

class _GraphPainter extends CustomPainter {
  const _GraphPainter({required this.ease, required this.from, required this.to, required this.frame, required this.active, required this.dragging});
  final Ease ease;
  final int? from, to, active;
  final int frame;
  final bool dragging;

  @override
  void paint(Canvas c, Size s) {
    final g = _EaseGraphState._box(s);
    Offset px(Offset v) => _EaseGraphState._px(g, v);
    final a = px(Offset.zero), b = px(const Offset(1, 1));
    final fill = Paint();
    // The unit band (0..100 %) is a well; overshoot above and below stays on the card.
    c.drawRect(Rect.fromLTRB(g.left, b.dy, g.right, a.dy), fill..color = Grey.g10);
    final grid = Paint()..strokeWidth = 1;
    for (var i = 0; i <= 8; i++) {
      final x = (g.left + g.width * i / 8).floorToDouble() + .5;
      c.drawLine(Offset(x, g.top), Offset(x, g.bottom), grid..color = i % 4 == 0 ? Grey.g26 : Grey.g15);
    }
    for (var i = -2; i <= 6; i++) {
      final y = px(Offset(0, i / 4)).dy.floorToDouble() + .5;
      if (y < g.top || y > g.bottom) continue;
      c.drawLine(Offset(g.left, y), Offset(g.right, y), grid..color = i == 0 || i == 4 ? Grey.g38 : Grey.g15);
    }
    final lab = T.label(Grey.g56).copyWith(fontSize: T.px(10));
    deskText(c, '100', Offset(g.left - 5, b.dy), lab, ax: 1, ay: .5);
    deskText(c, '50', Offset(g.left - 5, px(const Offset(0, .5)).dy), lab, ax: 1, ay: .5);
    deskText(c, '0', Offset(g.left - 5, a.dy), lab, ax: 1, ay: .5);
    // Frame ticks under the box.
    if (from != null && to != null) {
      final span = to! - from!;
      final step = span <= 12 ? 2 : (span <= 30 ? 6 : (span <= 60 ? 12 : 24));
      for (var f = from!; f <= to!; f++) {
        if ((f - from!) % step != 0 && f != to) continue;
        final x = g.left + g.width * (f - from!) / span;
        c.drawLine(Offset(x, g.bottom), Offset(x, g.bottom + 3), grid..color = Grey.g38);
        if (f == from || f == to || (to! - f) >= step * .6) {
          deskText(c, '$f', Offset(x, g.bottom + 6), T.value(Grey.g63).copyWith(fontSize: T.px(9.5)), ax: f == from ? 0 : (f == to ? 1 : .5));
        }
      }
    } else {
      deskText(c, 'start', Offset(g.left, g.bottom + 6), lab);
      deskText(c, 'end', Offset(g.right, g.bottom + 6), lab, ax: 1);
    }
    final bez = ease.kind == EaseKind.bezier;
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    // Handle stems.
    c.drawLine(a, px(ease.p1), stroke..color = bez ? Grey.g63 : Grey.g26);
    c.drawLine(b, px(ease.p2), stroke);
    // The curve.
    final path = Path()..moveTo(a.dx, a.dy);
    const n = 96;
    for (var i = 1; i <= n; i++) {
      final x = i / n;
      final o = px(Offset(x, ease.at(x)));
      if (ease.kind == EaseKind.hold && i == n) path.lineTo(o.dx, a.dy);
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(
      path,
      stroke
        ..strokeWidth = 2.2
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round
        ..color = WsT.accentInk,
    );
    // The playhead, when it is inside the span.
    if (from != null && to != null && frame >= from! && frame <= to! && to! > from!) {
      final t = (frame - from!) / (to! - from!), p = px(Offset(t, ease.at(t)));
      c.drawLine(Offset(p.dx, g.top), Offset(p.dx, g.bottom), Paint()..color = Grey.g44);
      c.drawCircle(p, 3.5, fill..color = Grey.g95);
      c.drawCircle(
        p,
        3.5,
        Paint()
          ..color = Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2,
      );
    }
    // Keys at both ends.
    for (final k in [a, b]) {
      c.save();
      c.translate(k.dx, k.dy);
      c.rotate(math.pi / 4);
      c.drawRect(const Rect.fromLTWH(-4, -4, 8, 8), fill..color = WsT.keyDot);
      c.drawRect(
        const Rect.fromLTWH(-4, -4, 8, 8),
        Paint()
          ..color = Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2,
      );
      c.restore();
    }
    // Handles.
    for (var i = 0; i < 2; i++) {
      final o = px(i == 0 ? ease.p1 : ease.p2), on = active == i;
      final r = on ? 5.0 : 4.0;
      c.drawCircle(o, r, fill..color = !bez ? Grey.g38 : (on ? WsT.accent : Grey.g95));
      c.drawCircle(
        o,
        r,
        Paint()
          ..color = Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.5,
      );
      if (dragging && on) {
        final v = i == 0 ? ease.p1 : ease.p2;
        deskText(
          c,
          '${v.dx.toStringAsFixed(2)}, ${v.dy.toStringAsFixed(2)}',
          o + Offset(i == 0 ? 9 : -9, -9),
          T.value(Grey.g95).copyWith(fontSize: T.px(10)),
          ax: i == 0 ? 0 : 1,
          ay: 1,
        );
      }
    }
    // Name of the shape, top-left inside the box.
    final name = (easeName(ease) ?? 'Custom').split(' ').first;   // (a kind with its numbers: its name; the numbers stand beside)
    deskText(c, name.toUpperCase(), Offset(g.left + 6, g.top + 5), T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700, letterSpacing: 1));
  }

  @override
  bool shouldRepaint(_GraphPainter o) => o.ease != ease || o.from != from || o.to != to || o.frame != frame || o.active != active || o.dragging != dragging;
}

