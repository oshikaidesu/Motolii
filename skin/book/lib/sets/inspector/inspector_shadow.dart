// The Drop Shadow card's drawing, in the Transform card's manner (a control is a drawing of its own result, pulled by
// hand, as Photoshop's layer style lets you drag the shadow on the canvas): the thing as a square, its shadow lying as
// far and as soft as it does, as dark as it is; take the shadow and put it where it should fall (which way and how far),
// pull the knob on its edge out for softer. Its numbers stay below as the card's tiles.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform.dart' show TransformZone;
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/sets/inspector/inspector_pulled.dart';
import 'package:lab_book/tokens.dart';

class ShadowDrawing extends StatefulWidget {
  const ShadowDrawing({super.key});
  static double get height => Shape.z(92.0);
  @override
  State<ShadowDrawing> createState() => _ShadowDrawingState();
}

enum _Knob { fall, soft }

class _ShadowDrawingState extends State<ShadowDrawing> {
  _Knob? _held;

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return ListenableBuilder( // (drawn again when one of its numbers is told anew, by its tiles or the inside)
      listenable: Listenable.merge([for (final id in const ['v_shadow_distance', 'v_shadow_angle', 'v_shadow_blur', 'v_shadow_opacity']) doc.watch(id)]),
      builder: (context, _) => _drawing(doc),
    );
  }

  Widget _drawing(Doc doc) {
    return Padding(
      padding: EdgeInsets.only(bottom: Shape.s(6)),
      child: SizedBox(
        height: ShadowDrawing.height,
        child: LayoutBuilder(
          builder: (context, box) {
            final g = _Shadow(box.biggest, distance: doc.get('v_shadow_distance') ?? 0, angle: doc.get('v_shadow_angle') ?? 135, blur: doc.get('v_shadow_blur') ?? 0,
                opacity: doc.get('v_shadow_opacity') ?? 1, colour: _colour(doc.s2['shadow']));
            _Knob? pick(Offset p) => (p - g.softKnob).distance <= 10 ? _Knob.soft : (g.shadow.inflate(4).contains(p) ? _Knob.fall : null);
            return Pulled(
              takes: (at) => pick(at) != null,
              onDown: (d) {
                _held = pick(d);
                if (_held != null) doc.beginGesture();
              },
              onMove: (d) {
                switch (_held) {
                  case _Knob.fall:
                    final (distance, angle) = g.fallAt(d);
                    doc.setMany({'v_shadow_distance': distance, 'v_shadow_angle': angle});
                  case _Knob.soft:
                    doc.set('v_shadow_blur', g.softAt(d));
                  case null:
                }
              },
              onUp: () {
                if (_held != null) doc.endGesture();
                setState(() => _held = null);
              },
              child: CustomPaint(size: box.biggest, painter: _ShadowPaint(g, _held)),
            );
          },
        ),
      ),
    );
  }
}

Color _colour(String? word) => switch (int.tryParse((word ?? '').split(' ').first, radix: 16)) {
  final v? when (word ?? '').split(' ').first.length == 6 => Color(0xFF000000 | v),
  _ => const Color(0xFF000000),
};

/// Where the thing, its shadow and the knob lie in the drawing. A distance is drawn so any of them fits (far ones closer
/// together); the angle is ThorVG's (0 to the right, 90 down).
class _Shadow {
  _Shadow(this.size, {required this.distance, required this.angle, required this.blur, required this.opacity, required this.colour});
  final Size size;
  final double distance, angle, blur, opacity;
  final Color colour;
  static const side = 30.0, far = 60.0; // (the thing's square; how many points of the picture draw half way out)
  Offset get middle => size.center(Offset.zero);
  double get reach => size.height / 2 - side / 2 - 4;
  double drawn(double d) => reach * (1 - math.exp(-d / far));
  Offset get fall => Offset(math.cos(angle * math.pi / 180), math.sin(angle * math.pi / 180)) * drawn(distance);
  Rect get thing => Rect.fromCenter(center: middle, width: side, height: side);
  Rect get shadow => thing.shift(fall);
  double get softDrawn => math.min(14, blur * .25);
  Offset get softKnob => shadow.centerRight + Offset(6 + softDrawn, 0);

  (double, double) fallAt(Offset p) {
    final v = p - middle, r = math.min(v.distance, reach * .98);
    return (-far * math.log(1 - r / reach), v.distance < .5 ? angle : v.direction * 180 / math.pi);
  }

  double softAt(Offset p) => (math.max(0, p.dx - shadow.right - 6) / .25).clamp(0, 56).toDouble();
}

class _ShadowPaint extends CustomPainter {
  const _ShadowPaint(this.g, this.held);
  final _Shadow g;
  final _Knob? held;

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, Shape.r(8)), Paint()..color = Grey.g07);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: g.middle, width: g.reach * 2 + _Shadow.side + 8, height: s.height - 8), Shape.r(6)), Paint()..color = Grey.g63); // (a ground a dark shadow and a light one both show on)
    final shade = Paint()..color = g.colour.withValues(alpha: g.opacity.clamp(0.0, 1.0) * .9);
    if (g.softDrawn > 0) shade.maskFilter = MaskFilter.blur(BlurStyle.normal, g.softDrawn / 2);
    c.drawRRect(RRect.fromRectAndRadius(g.shadow, Shape.r(3)), shade);
    if (held == _Knob.fall) c.drawLine(g.middle, g.shadow.center, Paint()..color = Grey.g56..strokeWidth = 1);
    c.drawRRect(RRect.fromRectAndRadius(g.thing, Shape.r(3)), Paint()..color = Grey.g95);
    // (the knob: out from the shadow's edge as far as it is soft)
    final k = g.softKnob;
    c.drawLine(g.shadow.centerRight + const Offset(2, 0), k, Paint()..color = Grey.g44..strokeWidth = 1);
    c.drawCircle(k, held == _Knob.soft ? 6.5 : 5, Paint()..color = held == _Knob.soft ? Pop.accent : TransformZone.lift.color);
  }

  @override
  bool shouldRepaint(_ShadowPaint o) => true;
}
