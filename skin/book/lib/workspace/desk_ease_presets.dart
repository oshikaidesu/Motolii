// A way as a tile, its curve small.

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/ws.dart';

class PresetTile extends StatelessWidget {
  const PresetTile(this.ease, {super.key, required this.on, required this.onTap, this.label, this.carried = true});
  final Ease ease;

  /// What it is called on its tile, where that is less than its name (anime.js's Elastic, its way picked beside).
  final String? label;
  final bool on;
  final VoidCallback onTap;

  /// Whether a hand may carry it to the graph, to lay it over the way there (desk_ease_cards.dart).
  final bool carried;
  @override
  Widget build(BuildContext context) {
    final tile = _tile();
    return !carried
        ? tile
        : Draggable<Ease>(
            data: ease,
            dragAnchorStrategy: pointerDragAnchorStrategy,
            feedback: easeHeld(tile),
            childWhenDragging: Opacity(opacity: .3, child: tile),
            child: tile,
          );
  }

  Widget _tile() => Hov(
    onTap: onTap,
    builder: (_, h) {
      if (label == '') {   // (a shape beside the graph: a square of one size, its ink all that says it is the one: AbsentMotion's CurveThumb)
        return Center(child: Transform.translate(
          offset: Offset(0, h && carried ? -Shape.s(2) : 0),
          child: CustomPaint(size: Size.square(Shape.z(36)), painter: EaseIcon(ease, on ? WsT.accentInk : (h ? Grey.g95 : Grey.g76))),
        ));
      }
      final ink = on ? WsT.onAccent : (h ? Grey.g95 : Grey.g76);
      return Transform.translate(   // (a card: it lifts under the hand)
        offset: Offset(0, h && carried ? -Shape.s(2) : 0),
        child: Container(
        decoration: BoxDecoration(color: on ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.all(Shape.r(WsT.radius)), boxShadow: h && carried ? easeShadow : null),
        padding: EdgeInsets.fromLTRB(Shape.s(2), Shape.s(5), Shape.s(2), Shape.s(4)),
        child: Column(
          children: [
            Expanded(
              child: CustomPaint(size: Size.infinite, painter: EaseThumb(ease, ink, on ? WsT.onAccent : Grey.g38)),
            ),
            if (label != '') SizedBox(height: Shape.s(3)),
            if (label != '') FittedBox( // (a name wider than its tile is drawn smaller, never cut)
              fit: BoxFit.scaleDown,
              child: Text(
                label ?? easeTitle(ease),
                maxLines: 1,
                style: T.micro(ink).copyWith(fontSize: T.px(9.5), fontWeight: on ? FontWeight.w700 : FontWeight.w500),
              ),
            ),
          ],
        ),
        ),
      );
    },
  );
}

const easeShadow = [BoxShadow(color: Color(0x80000000), blurRadius: 10, offset: Offset(0, 4))];

/// A card in the hand: held by its middle, tilted a little.
Widget easeHeld(Widget card) {
  final w = Shape.z(50), h = Shape.z(50);
  return IgnorePointer(
    child: Transform.translate(
      offset: Offset(-w / 2, -h / 2),
      child: Transform.rotate(
        angle: -.06,
        child: Container(width: w, height: h, decoration: BoxDecoration(borderRadius: BorderRadius.all(Shape.r(WsT.radius)), boxShadow: easeShadow), child: card),
      ),
    ),
  );
}

class EaseThumb extends CustomPainter {
  const EaseThumb(this.ease, this.ink, this.base);
  final Ease ease;
  final Color ink, base;
  @override
  void paint(Canvas c, Size s) {
    const lo = -.25, hi = 1.6;
    final w = math.min(s.width, s.height * 1.6), r = Rect.fromCenter(center: s.center(Offset.zero), width: w, height: s.height);
    Offset px(double x, double y) => Offset(r.left + x * r.width, r.bottom - (y - lo) / (hi - lo) * r.height);
    c.drawLine(
      px(0, 0),
      px(1, 0),
      Paint()
        ..color = base
        ..strokeWidth = 1,
    );
    final path = Path()..moveTo(px(0, 0).dx, px(0, 0).dy);
    for (var i = 1; i <= 32; i++) {
      final x = i / 32, o = px(x, ease.at(x));
      if (ease.kind == EaseKind.hold && i == 32) path.lineTo(o.dx, px(0, 0).dy);
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(
      path,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(EaseThumb o) => o.ease != ease || o.ink != ink || o.base != base;
}

/// A shape as AbsentMotion's EasingIcons.kt draws it, in a square of its own: what it does, not its true curve, too fine
/// to read so small. A way it has no drawing for: its own curve.
class EaseIcon extends CustomPainter {
  const EaseIcon(this.ease, this.ink);
  final Ease ease;
  final Color ink;

  static double _x(double u) => 3 + 61 * u;
  static double _y(double v) => 64 - 60 * v;
  static double _bounceOut(double t) {
    const n = 7.5625, d = 2.75;
    if (t < 1 / d) return n * t * t;
    if (t < 2 / d) return n * (t - 1.5 / d) * (t - 1.5 / d) + .75;
    if (t < 2.5 / d) return n * (t - 2.25 / d) * (t - 2.25 / d) + .9375;
    return n * (t - 2.625 / d) * (t - 2.625 / d) + .984375;
  }

  @override
  void paint(Canvas c, Size s) {
    final k = s.width / 68;
    c.scale(k);
    const box = Rect.fromLTWH(3, 4, 61, 60);
    final square = RRect.fromRectAndRadius(box, const Radius.circular(8));
    c.drawRRect(square, Paint()..color = ink.withValues(alpha: .16));
    c.drawRRect(square, Paint()..color = ink.withValues(alpha: .4)..style = PaintingStyle.stroke..strokeWidth = 3);
    c.clipRRect(square);
    final curve = Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 3.8..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final bar = Paint()..color = ink..strokeWidth = 4.4..strokeCap = StrokeCap.round;
    void dot(double u, double v) => c.drawCircle(Offset(_x(u), _y(v)), 5.5, Paint()..color = ink);
    void segment(double u0, double u1, double v0, [double? v1]) => c.drawLine(Offset(_x(u0), _y(v0)), Offset(_x(u1), _y(v1 ?? v0)), bar);
    void line(double Function(double) f, int count, [double scale = 1]) {
      final path = Path();
      for (var i = 0; i < count; i++) {
        final u = i / (count - 1), x = _x(u), y = _y((f(u) * scale).clamp(0.0, 1.0));
        i == 0 ? path.moveTo(x, y) : path.lineTo(x, y);
      }
      c.drawPath(path, curve);
    }
    void bezier(Offset a, Offset b) {
      c.drawPath(Path()..moveTo(_x(0), _y(0))..cubicTo(_x(a.dx), _y(a.dy), _x(b.dx), _y(b.dy), _x(1), _y(1)), curve);
      segment(0, a.dx, 0, a.dy);
      dot(a.dx, a.dy);
      segment(1, b.dx, 1, b.dy);
      dot(b.dx, b.dy);
    }

    final word = said(ease).split(' ').first;
    switch (word) {
      case 'Linear':
        bezier(const Offset(.25, .25), const Offset(.75, .75));
      case 'Bezier':
        bezier(ease.p1, ease.p2);
      case 'Hold':
        segment(.08, .78, .14);
        segment(.78, .78, .14, .86);
        dot(.78, .14);
      case 'Bounce':
        line(_bounceOut, 60);
        dot(1 / 2.75, 1);
      case 'Elastic':
        line((u) => 1 - math.exp(-5 * u) * math.cos(2 * math.pi * 1.7 * u), 60, .8);
        dot(.15, .8);
      case 'Cyclic':
        line((u) => .5 + .42 * math.cos(2 * math.pi * 1.25 * u), 60);
        dot(0, .92);
      case 'Random':
        const noise = [0.0, 1.0, -1.0, 1.0, -.6, 1.0, -1.0, .8, -1.0, 1.0, -.8, 1.0, -1.0, .7, -.5, 0.0];
        line((u) => .1 + .8 * u + .12 * noise[(u * 15 + .5).toInt()], 16);
        dot(.2, .27);
      case 'Steps':
        segment(0, .37, 0);
        dot(.37, 0);
        segment(.18, .49, .27);
        segment(.35, .66, .5);
        segment(.53, .83, .73);
        segment(.69, 1, 1);
      case 'ElasticSteps':
        segment(0, .45, 0);
        dot(.45, 0);
        for (final (u0, v) in const [(0.0, .3), (.3, .6), (.55, .9)]) {
          final path = Path()..moveTo(_x(u0), _y(v));
          for (var i = 1; i <= 6; i++) {
            path.lineTo(_x(u0 + .07 * i), _y(v + (i.isOdd ? .07 : -.04)));
          }
          c.drawPath(path, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 3.2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
        }
      default:
        line(ease.at, 48);
    }
  }

  @override
  bool shouldRepaint(EaseIcon o) => o.ease != ease || o.ink != ink;
}
