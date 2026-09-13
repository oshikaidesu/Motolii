import 'dart:math' as math;
import 'dart:ui' as ui show Vertices, VertexMode;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'metrics.dart';
import 'theme.dart';
import 'panel_controls.dart';

/// A colour value and its local editing surface. The caller owns the edit.
class EditorColorField extends StatefulWidget {
  const EditorColorField({
    super.key,
    required this.value,
    required this.label,
    required this.onPreview,
    required this.onFinish,
    required this.onCancel,
    this.enabled = true,
    this.allowAlpha = true,
  });
  final Color value;
  final String label;
  final bool enabled, allowAlpha;
  final Future<void> Function(Color) onPreview;
  final Future<void> Function() onFinish, onCancel;
  @override
  State<EditorColorField> createState() => _EditorColorFieldState();
}

class _EditorColorFieldState extends State<EditorColorField> {
  bool _open = false, _changing = false, _ending = false;
  Color? _draft;
  double? _hue;
  String? _part;
  final _focus = FocusNode();
  late final _queue = EditorPreviewQueue<Color>(
    (value) => widget.onPreview(value),
  );
  void _change(Color value) {
    if (_ending || !widget.enabled) return;
    _focus.requestFocus();
    setState(() {
      _draft = value;
      _changing = true;
    });
    _queue.add(value);
  }

  Future<void> _finish(bool cancel) async {
    if (!_changing || _ending) return;
    _ending = true;
    try {
      await _queue.finish(cancel, cancel ? widget.onCancel : widget.onFinish);
    } finally {
      _ending = false;
      _changing = false;
      _part = null;
      if (mounted) setState(() => _draft = null);
    }
  }

  @override
  void dispose() {
    if (_changing && !_ending) _queue.finish(true, widget.onCancel);
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final value = _draft ?? widget.value;
    return Focus(
      focusNode: _focus,
      onFocusChange: (on) {
        if (!on) _finish(true);
      },
      onKeyEvent: (_, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
          _finish(true);
          setState(() => _open = false);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          EditorTooltip(
            message: 'Choose ${widget.label}',
            child: InkWell(
              onTap: !widget.enabled
                  ? null
                  : () {
                      _focus.requestFocus();
                      setState(() => _open = !_open);
                    },
              child: Container(
                height: EditorMetrics.row,
                decoration: BoxDecoration(
                  color: value,
                  border: Border.all(color: EditorTheme.border),
                ),
              ),
            ),
          ),
          if (_open) ...[
            const SizedBox(height: EditorMetrics.s6),
            LayoutBuilder(
              builder: (context, box) {
                final wheel = ColorWheel(
                  math.min(box.maxWidth, EditorMetrics.s200),
                  'square',
                );
                void sample(Offset p) {
                  var hsv = HSVColor.fromColor(_draft ?? widget.value);
                  hsv = hsv.withHue(_hue ?? hsv.hue);
                  _part ??= wheel.hitsInner(p, hsv) ? 'sv' : 'hue';
                  final next = (_part == 'sv'
                      ? wheel.pickInner(p, hsv)
                      : hsv.withHue(wheel.hueAt(p)));
                  _hue = next.hue;
                  _change(next.toColor());
                }

                return Align(
                  alignment: Alignment.centerLeft,
                  child: GestureDetector(
                    onPanStart: (e) => sample(e.localPosition),
                    onPanUpdate: (e) => sample(e.localPosition),
                    onPanEnd: (_) => _finish(false),
                    onPanCancel: () => _finish(true),
                    onTapUp: (e) {
                      sample(e.localPosition);
                      _finish(false);
                    },
                    child: CustomPaint(
                      size: Size.square(wheel.side),
                      painter: ColorWheelPainter(value, wheel, hue: _hue),
                    ),
                  ),
                );
              },
            ),
            if (widget.allowAlpha)
              Row(
                children: [
                  const Icon(
                    Icons.opacity,
                    size: EditorMetrics.s14,
                    color: EditorTheme.muted,
                  ),
                  Expanded(
                    child: Slider(
                      value: value.a,
                      onChanged: (a) => _change(value.withValues(alpha: a)),
                      onChangeEnd: (_) => _finish(false),
                    ),
                  ),
                ],
              ),
          ],
        ],
      ),
    );
  }
}

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

  bool hitsInner(Offset p, HSVColor hsv) => shape == 'triangle'
      ? (p - center).distance <= inner + EditorMetrics.s4
      : square.inflate(EditorMetrics.s4).contains(p);
  HSVColor pickInner(Offset p, HSVColor hsv) {
    if (shape == 'triangle') {
      final w = _weights(
        p,
        triangle(hsv.hue),
      ).map((x) => x.clamp(0.0, 1.0)).toList();
      final sum = w[0] + w[1] + w[2];
      final a = w[0] / sum, b = w[1] / sum;
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

/// Transparency shows as the usual light/dark checker.
class CheckerPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    const cell = EditorMetrics.s4;
    final light = Paint()..color = const Color(0xffbbbbbb);
    final dark = Paint()..color = const Color(0xff777777);
    for (var y = 0.0; y < size.height; y += cell)
      for (var x = 0.0; x < size.width; x += cell)
        canvas.drawRect(
          Rect.fromLTWH(x, y, cell, cell),
          ((x + y) / cell).round().isEven ? light : dark,
        );
  }

  @override
  bool shouldRepaint(covariant CheckerPainter oldDelegate) => false;
}

class Swatch extends StatelessWidget {
  const Swatch({required this.color, required this.size});
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    clipBehavior: Clip.antiAlias,
    decoration: BoxDecoration(
      borderRadius: BorderRadius.circular(EditorMetrics.s4),
      border: Border.all(color: Colors.black26),
    ),
    child: CustomPaint(
      painter: CheckerPainter(),
      child: ColoredBox(color: color),
    ),
  );
}
