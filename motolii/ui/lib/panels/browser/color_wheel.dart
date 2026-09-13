import 'dart:math' as math;
import 'dart:ui' as ui show Vertices, VertexMode;

import 'package:flutter/material.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';

/// Where the ring and the inner area are, for one wheel size and shape.
/// Sampling and painting both read this, so they cannot disagree.
class ColorWheel {
  ColorWheel(this.side, this.shape);
  final double side;
  final String shape;
  static const double ring = 14;
  Offset get center => Offset(side / 2, side / 2);
  double get hueRadius => side / 2 - ring / 2;
  double get inner => side / 2 - ring - EditorMetrics.s3;
  Rect get square => Rect.fromCenter(
    center: center,
    width: inner * math.sqrt2,
    height: inner * math.sqrt2,
  );
  Offset _rim(double degrees) {
    final a = (degrees - 90) * math.pi / 180;
    return center + Offset(math.cos(a), math.sin(a)) * inner;
  }

  /// Hue, white, black corners; the hue corner points at the hue handle.
  List<Offset> triangle(double hue) => [
    _rim(hue),
    _rim(hue + 120),
    _rim(hue + 240),
  ];
  double hueAt(Offset p) =>
      (math.atan2(p.dy - center.dy, p.dx - center.dx) * 180 / math.pi + 90) %
      360;

  /// Barycentric weights of p against the triangle (hue, white, black).
  List<double> _weights(Offset p, List<Offset> t) {
    final d =
        (t[1].dy - t[2].dy) * (t[0].dx - t[2].dx) +
        (t[2].dx - t[1].dx) * (t[0].dy - t[2].dy);
    final a =
        ((t[1].dy - t[2].dy) * (p.dx - t[2].dx) +
            (t[2].dx - t[1].dx) * (p.dy - t[2].dy)) /
        d;
    final b =
        ((t[2].dy - t[0].dy) * (p.dx - t[2].dx) +
            (t[0].dx - t[2].dx) * (p.dy - t[2].dy)) /
        d;
    return [a, b, 1 - a - b];
  }

  Offset _closestOn(Offset p, Offset a, Offset b) {
    final edge = b - a;
    final length2 = edge.dx * edge.dx + edge.dy * edge.dy;
    if (length2 == 0) return a;
    final u = (((p - a).dx * edge.dx + (p - a).dy * edge.dy) / length2).clamp(
      0.0,
      1.0,
    );
    return a + edge * u;
  }

  Offset _insideTriangle(Offset p, List<Offset> t) {
    if (_weights(p, t).every((weight) => weight >= 0)) return p;
    final candidates = [
      _closestOn(p, t[0], t[1]),
      _closestOn(p, t[1], t[2]),
      _closestOn(p, t[2], t[0]),
    ];
    candidates.sort(
      (a, b) => (a - p).distanceSquared.compareTo((b - p).distanceSquared),
    );
    return candidates.first;
  }

  bool hitsInner(Offset p, HSVColor hsv) => shape == 'triangle'
      ? (_insideTriangle(p, triangle(hsv.hue)) - p).distance <= EditorMetrics.s4
      : square.inflate(EditorMetrics.s4).contains(p);
  HSVColor pickInner(Offset p, HSVColor hsv) {
    if (shape == 'triangle') {
      final triangle = this.triangle(hsv.hue);
      final w = _weights(_insideTriangle(p, triangle), triangle);
      final a = w[0].clamp(0.0, 1.0), b = w[1].clamp(0.0, 1.0);
      final value = a + b;
      return hsv
          .withValue(value.clamp(0.0, 1.0))
          .withSaturation(value == 0 ? 0 : (a / value).clamp(0.0, 1.0));
    }
    final r = square;
    return hsv
        .withSaturation(((p.dx - r.left) / r.width).clamp(0.0, 1.0))
        .withValue((1 - (p.dy - r.top) / r.height).clamp(0.0, 1.0));
  }

  Offset innerHandle(HSVColor hsv) {
    if (shape == 'triangle') {
      final t = triangle(hsv.hue);
      final a = hsv.saturation * hsv.value, b = hsv.value - a;
      return t[0] * a + t[1] * b + t[2] * (1 - a - b);
    }
    final r = square;
    return Offset(
      r.left + hsv.saturation * r.width,
      r.top + (1 - hsv.value) * r.height,
    );
  }

  Offset hueHandle(HSVColor hsv) {
    final a = (hsv.hue - 90) * math.pi / 180;
    return center + Offset(math.cos(a), math.sin(a)) * hueRadius;
  }
}

class ColorWheelPainter extends CustomPainter {
  ColorWheelPainter(this.color, this.wheel, {this.hue});
  final Color color;
  final ColorWheel wheel;
  final double? hue;

  @override
  void paint(Canvas canvas, Size size) {
    var hsv = HSVColor.fromColor(color);
    hsv = hsv.withHue(hue ?? hsv.hue);
    final pure = HSVColor.fromAHSV(1, hsv.hue, 1, 1).toColor();
    final bounds = Offset.zero & size;
    canvas.drawCircle(
      wheel.center,
      wheel.hueRadius,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = ColorWheel.ring
        ..shader = SweepGradient(
          colors: [
            for (var i = 0; i <= 6; i++)
              HSVColor.fromAHSV(1, (i * 60) % 360, 1, 1).toColor(),
          ],
          transform: const GradientRotation(-math.pi / 2),
        ).createShader(bounds),
    );
    // A dark hairline on both rims lifts the ring off the panel.
    final rim = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = EditorTheme.line;
    canvas.drawCircle(wheel.center, wheel.side / 2 - ColorWheel.ring, rim);
    canvas.drawCircle(wheel.center, wheel.side / 2 - .5, rim);

    final shadow = Paint()
      ..color = Colors.black45
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2);
    if (wheel.shape == 'triangle') {
      final t = wheel.triangle(hsv.hue);
      final path = Path()..addPolygon(t, true);
      canvas.drawPath(path.shift(const Offset(0, 1)), shadow);
      canvas.drawVertices(
        ui.Vertices(
          ui.VertexMode.triangles,
          t,
          colors: [pure, Colors.white, Colors.black],
        ),
        BlendMode.srcOver,
        Paint(),
      );
      canvas.drawPath(path, rim);
    } else {
      final r = RRect.fromRectAndRadius(
        wheel.square,
        const Radius.circular(EditorMetrics.s4),
      );
      canvas.drawRRect(r.shift(const Offset(0, 1)), shadow);
      canvas.drawRRect(
        r,
        Paint()
          ..shader = LinearGradient(colors: [Colors.white, pure])
              .createShader(wheel.square),
      );
      canvas.drawRRect(
        r,
        Paint()
          ..shader = const LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [Colors.transparent, Colors.black],
          ).createShader(wheel.square),
      );
      canvas.drawRRect(r, rim);
    }

    _handle(canvas, wheel.innerHandle(hsv), color.withValues(alpha: 1));
    _handle(canvas, wheel.hueHandle(hsv), pure);
  }

  void _handle(Canvas canvas, Offset at, Color fill) {
    canvas.drawCircle(
      at.translate(0, 1),
      5,
      Paint()
        ..color = Colors.black54
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.5),
    );
    canvas.drawCircle(at, 5, Paint()..color = Colors.white);
    canvas.drawCircle(at, 3.5, Paint()..color = fill);
  }

  @override
  bool shouldRepaint(covariant ColorWheelPainter oldDelegate) =>
      color != oldDelegate.color ||
      hue != oldDelegate.hue ||
      wheel.side != oldDelegate.wheel.side ||
      wheel.shape != oldDelegate.wheel.shape;
}
