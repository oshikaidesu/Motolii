// Faces: how a declared face type is drawn. A new thing picks an existing type and gives parameters;
// a new type is added here once, never per thing.
import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import '../neutral.dart';
import 'create.dart';
import 'effects.dart';
import 'seat.dart';
import 'shell.dart' show GlyphBox;
import 'things.dart';

Color hexColor(String h) => Color(0xFF000000 | int.parse(h.substring(1), radix: 16));

class ThingFace extends StatelessWidget {
  const ThingFace(this.thing, {super.key, this.scene});
  final Thing thing;
  final EffectScene? scene;
  @override
  Widget build(BuildContext context) {
    final own = BrowserSeatScope.of(context)?.face(context, thing);
    if (own != null) return own;
    final f = thing.face;
    switch (f['type']) {
      case 'mark':
        return CustomPaint(painter: MarkPainter(Mk.values.byName('${f['mark']}'), hexColor('${f['color']}')));
      case 'fx':
        final s = scene;
        if (s == null) return const QuietFace();
        Widget p = CustomPaint(painter: FxPainter('${f['base']}', s));
        final hue = (f['hue'] as num).toDouble();
        if (hue != 0) p = ColorFiltered(colorFilter: hueFilter(hue), child: p);
        return ClipRRect(borderRadius: BorderRadius.circular(3), child: p);
      case 'curve':
        return CustomPaint(painter: CurvePainter('${f['fn']}', (f['hue'] as num).toDouble()));
      default:
        return const QuietFace();
    }
  }
}

/// A thing with no picture to show: a quiet tile, so a grid of faces has no holes (its name is the caption).
class QuietFace extends StatelessWidget {
  const QuietFace({super.key});
  @override
  Widget build(BuildContext context) => DecoratedBox(
        decoration: BoxDecoration(color: N.g13, border: Border.all(color: N.g20), borderRadius: BorderRadius.circular(3)),
        child: const Center(child: GlyphBox(HG.pie, size: 16, color: N.g33)),
      );
}

/// A phenomenon drawn as its own curve: spring, decay, orbit... the face for things that act over time.
class CurvePainter extends CustomPainter {
  CurvePainter(this.fn, this.hue);
  final String fn;
  final double hue;
  @override
  void paint(Canvas cv, Size s) {
    final col = HSLColor.fromAHSL(1, hue % 360, .78, .68).toColor();
    final p = Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = math.max(1.6, s.shortestSide * .045)..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final w = s.width, h = s.height, mid = h / 2;
    cv.drawLine(Offset(w * .06, mid), Offset(w * .94, mid), Paint()..color = col.withValues(alpha: .25)..strokeWidth = 1);
    final path = Path();
    const n = 64;
    final rnd = math.Random(hue.round());
    var walk = 0.0;
    for (var i = 0; i <= n; i++) {
      final t = i / n;
      final x = w * (.06 + .88 * t);
      double y;
      switch (fn) {
        case 'spring':
          y = math.cos(t * math.pi * 7) * math.exp(-t * 3.2);
        case 'decay':
          y = math.exp(-t * 4) * 2 - 1;
        case 'noise':
          walk += (rnd.nextDouble() - .5) * .55;
          y = walk.clamp(-1.0, 1.0);
        case 'bounce':
          y = 1 - 2 * (math.sin(t * math.pi * 3.5).abs() * math.exp(-t * 2.2));
        case 'orbit':
          y = math.sin(t * math.pi * 5) * (.25 + t * .7);
        default:
          y = math.sin(t * math.pi * 3);
      }
      final py = mid - y * h * .36;
      i == 0 ? path.moveTo(x, py) : path.lineTo(x, py);
    }
    cv.drawPath(path, p);
  }
  @override
  bool shouldRepaint(CurvePainter o) => o.fn != fn || o.hue != hue;
}
