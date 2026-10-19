// The Light card's drawing, in the Transform card's manner (a control is a drawing of its own result, pulled by hand, as
// Ableton's filter curve and the OP-1's envelope): the light's glow as far as it reaches, its rays as long as it is
// strong, a spot's cone as wide as it opens; a knob on each to pull. Its numbers stay below as the card's tiles.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform.dart' show TransformZone;
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// The light as it shines: a sun's parallel rays, a point's glow all round, a spot's cone; pull the right rail for its
/// strength, the ring's knob for how far it reaches, the cone's edge for how wide it opens.
class LightDrawing extends StatefulWidget {
  const LightDrawing({super.key});
  static const height = 112.0;
  @override
  State<LightDrawing> createState() => _LightDrawingState();
}

enum _Knob { strength, reach, cone }

class _LightDrawingState extends State<LightDrawing> {
  _Knob? _held;

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    final card = doc.cards.where((c) => c.name == 'Light').firstOrNull;
    double norm(String id) { // (where a value lies on its range, 0 to 1: the drawing is the range, not the number)
      final p = card?.params.where((p) => p.id == id).firstOrNull, v = doc.get('v_$id');
      if (p == null || v == null || p.min == null || p.max == null || p.max == p.min) return .5;
      return ((v - p.min!) / (p.max! - p.min!)).clamp(0.0, 1.0);
    }

    void pull(String id, double at) { // (a value set where the drawing says: its range, from 0 to 1)
      final p = card?.params.where((p) => p.id == id).firstOrNull;
      if (p == null || p.min == null || p.max == null) return;
      doc.set('v_$id', p.min! + (p.max! - p.min!) * at.clamp(0.0, 1.0));
    }

    final spot = doc.s2['light'] == 'spot', point = doc.s2['light'] == 'point'; // (else a sun)
    return Padding(
      padding: EdgeInsets.only(bottom: Shape.s(6)),
      child: SizedBox(
        height: LightDrawing.height,
        child: LayoutBuilder(
          builder: (context, box) {
            final g = _Light(box.biggest, spot: spot, point: point, strength: norm('intensity'), reach: norm('falloff'), cone: norm('cone'));
            _Knob? pick(Offset p) => [
              for (final (k, at) in [(_Knob.strength, g.strengthKnob), if (spot || point) (_Knob.reach, g.reachKnob), if (spot) (_Knob.cone, g.coneKnob)])
                if ((p - at).distance <= 12) k,
            ].firstOrNull;
            void drag(Offset p) => switch (_held) {
              _Knob.strength => pull('intensity', g.strengthAt(p)),
              _Knob.reach => pull('falloff', g.reachAt(p)),
              _Knob.cone => pull('cone', g.coneAt(p)),
              null => null,
            };
            return GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanStart: (d) {
                _held = pick(d.localPosition);
                if (_held != null) doc.beginGesture();
              },
              onPanUpdate: (d) => drag(d.localPosition),
              onPanEnd: (_) {
                if (_held != null) doc.endGesture();
                setState(() => _held = null);
              },
              child: CustomPaint(size: box.biggest, painter: _LightPaint(g, _held)),
            );
          },
        ),
      ),
    );
  }
}

/// Where the light and its knobs lie in the drawing.
class _Light {
  _Light(this.size, {required this.spot, required this.point, required this.strength, required this.reach, required this.cone});
  final Size size;
  final bool spot, point;
  final double strength, reach, cone; // (each on its range, 0 to 1)
  static const rail = 14.0;
  double get railX => size.width - rail;
  double get railTop => 18;
  double get railBottom => size.height - 16;
  Offset get source => spot ? Offset(28, size.height / 2) : point ? Offset((size.width - rail * 2) / 2, size.height / 2) : const Offset(20, 14);
  double get far => (spot ? size.width - rail * 2 - 40 : size.height / 2 - 6) * (.2 + .8 * reach); // (how far its glow reaches)
  double get half => (8 + 52 * cone) * math.pi / 180; // (half the cone's opening)
  Offset get strengthKnob => Offset(railX, railBottom - (railBottom - railTop) * strength);
  Offset get reachKnob => source + Offset(far, 0);
  Offset get coneKnob => source + Offset(math.cos(-half), math.sin(-half)) * far * .8;
  double strengthAt(Offset p) => (railBottom - p.dy) / (railBottom - railTop);
  double reachAt(Offset p) => ((p.dx - source.dx) / (spot ? size.width - rail * 2 - 40 : size.height / 2 - 6) - .2) / .8;
  double coneAt(Offset p) => (((p - source).direction.abs() * 180 / math.pi) - 8) / 52;
}

class _LightPaint extends CustomPainter {
  const _LightPaint(this.g, this.held);
  final _Light g;
  final _Knob? held;
  static const warm = Color(0xFFFFE6A8);

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, Shape.r(8)), Paint()..color = Grey.g07);
    final glow = warm.withValues(alpha: .12 + .55 * g.strength), line = Paint()
      ..color = warm.withValues(alpha: .35 + .6 * g.strength)
      ..strokeWidth = 1.6
      ..strokeCap = StrokeCap.round;
    c.save();
    c.clipRRect(RRect.fromRectAndRadius(Rect.fromLTWH(0, 0, s.width - _Light.rail * 2, s.height), Shape.r(8)));
    if (g.spot) {
      // (a cone as wide as it opens, as bright as it is strong, as long as it reaches)
      final path = Path()
        ..moveTo(g.source.dx, g.source.dy)
        ..arcTo(Rect.fromCircle(center: g.source, radius: g.far), -g.half, g.half * 2, false)
        ..close();
      c.drawPath(path, Paint()..shader = RadialGradient(colors: [glow, glow.withValues(alpha: 0)]).createShader(Rect.fromCircle(center: g.source, radius: g.far)));
      c.drawLine(g.source, g.source + Offset(math.cos(-g.half), math.sin(-g.half)) * g.far, line);
      c.drawLine(g.source, g.source + Offset(math.cos(g.half), math.sin(g.half)) * g.far, line);
    } else if (g.point) {
      // (a glow all round, as far as it reaches; rays as long as it is strong)
      c.drawCircle(g.source, g.far, Paint()..shader = RadialGradient(colors: [glow, glow.withValues(alpha: 0)]).createShader(Rect.fromCircle(center: g.source, radius: g.far)));
      for (var i = 0; i < 8; i++) {
        final a = i * math.pi / 4, d = Offset(math.cos(a), math.sin(a));
        c.drawLine(g.source + d * 9, g.source + d * (11 + 22 * g.strength), line);
      }
    } else {
      // (a sun: parallel rays from far away, as long as it is strong, and the ground they light)
      final ground = s.height - 14;
      c.drawRect(Rect.fromLTRB(0, ground, s.width, s.height), Paint()..color = warm.withValues(alpha: .08 + .5 * g.strength));
      for (var i = 0; i < 7; i++) {
        final from = g.source + Offset(14.0 + i * 26, 6);
        c.drawLine(from, from + const Offset(.5, .87) * (24 + (ground - from.dy - 24) * g.strength), line);
      }
    }
    c.drawCircle(g.source, g.spot || g.point ? 6 : 9, Paint()..color = warm);
    c.restore();
    // (the strength rail at the right, as the Transform card's depth rail: up is stronger)
    final rail = Paint()
      ..color = Grey.g26
      ..strokeWidth = 2
      ..strokeCap = StrokeCap.round;
    c.drawLine(Offset(g.railX, g.railTop), Offset(g.railX, g.railBottom), rail);
    // (the brightness marks every hand has seen: a sun with rays at the top of the rail, a dim dot at its foot)
    final top = Offset(g.railX, g.railTop - 4), mark = Paint()
      ..color = Grey.g63
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round;
    c.drawCircle(top, 2.5, mark);
    for (var i = 0; i < 8; i++) {
      final d = Offset(math.cos(i * math.pi / 4), math.sin(i * math.pi / 4));
      c.drawLine(top + d * 4, top + d * 5.5, mark);
    }
    c.drawCircle(Offset(g.railX, g.railBottom + 6), 1.8, mark);
    void knob(Offset at, _Knob k, Color col) {
      c.drawCircle(at, held == k ? 7 : 5.5, Paint()..color = col);
      c.drawCircle(at, held == k ? 7 : 5.5, Paint()
        ..color = Grey.g07
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5);
    }

    knob(g.strengthKnob, _Knob.strength, warm);
    if (g.spot || g.point) knob(g.reachKnob, _Knob.reach, TransformZone.lift.color);
    if (g.spot) knob(g.coneKnob, _Knob.cone, TransformZone.pivot.color);
  }

  @override
  bool shouldRepaint(_LightPaint o) => true;
}
