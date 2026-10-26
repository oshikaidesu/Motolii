import 'package:flutter/widgets.dart';

class TourP8 extends StatefulWidget {
  const TourP8({super.key});

  @override
  State<TourP8> createState() => _TourP8State();
}

class _TourP8State extends State<TourP8> {
  bool _swap = false;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final w = constraints.maxWidth;
        final h = constraints.maxHeight;
        if (!w.isFinite || !h.isFinite || w <= 0 || h <= 0) {
          return const SizedBox.shrink();
        }
        return ClipRect(
          child: SizedBox(
            width: w,
            height: h,
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => setState(() => _swap = !_swap),
              child: CustomPaint(
                painter: _Plate(swap: _swap),
                child: const SizedBox.expand(),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _Plate extends CustomPainter {
  const _Plate({required this.swap});

  final bool swap;

  static const _cream = Color(0xFFF3EFE4);
  static const _ink = Color(0xFF121212);
  static const _blue = Color(0xFF1A3CE0);
  static const _yolk = Color(0xFFC4A005);
  static const _paper = Color(0xFFFFFFFF);

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = _cream);
    canvas.save();
    canvas.scale(size.width / 1440, size.height / 900);
    _mark(canvas);
    _plate(canvas);
    _bars(canvas);
    canvas.restore();
  }

  @override
  bool shouldRepaint(_Plate oldDelegate) => oldDelegate.swap != swap;

  void _mark(Canvas canvas) {
    _type(canvas, 'M', 'Arial Black', 459, 65, 47);
    _type(canvas, 'CUT', 'Arial Black', 38.6, 64, 514);
    _type(canvas, 'HOLD', 'Arial Black', 38.6, 64, 549);
    final ring = Paint()
      ..color = _ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.7;
    canvas.drawCircle(const Offset(526, 118), 8, ring);
    ring.strokeWidth = 1.45;
    canvas.drawCircle(const Offset(555, 168.5), 5, ring);
    ring.strokeWidth = 1.2;
    canvas.drawCircle(const Offset(520, 209.5), 3.2, ring);
    _dotted(canvas, const Offset(554, 273), const Offset(865, 273));
    _dotted(canvas, const Offset(865, 273), const Offset(865, 668));
    _dotted(canvas, const Offset(786, 668), const Offset(865, 668));
    final dot = Paint()..color = _ink;
    canvas.drawCircle(const Offset(554, 273), 3.4, dot);
    canvas.drawCircle(const Offset(786, 668), 3.1, dot);
    canvas.drawCircle(const Offset(709, 328), 2.3, dot);
  }

  void _type(Canvas canvas, String text, String family, double fontSize, double x, double y) {
    final painter = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          fontFamily: family,
          fontFamilyFallback: const ['Helvetica Neue', 'Arial', 'Futura', 'Menlo'],
          fontSize: fontSize,
          height: 1,
          color: _ink,
          decoration: TextDecoration.none,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, Offset(x, y));
  }

  void _dotted(Canvas canvas, Offset a, Offset b) {
    final delta = b - a;
    final len = delta.distance;
    if (len == 0) {
      return;
    }
    final step = 5.6;
    final dir = delta / len;
    final paint = Paint()..color = _ink;
    for (var d = 0.0; d <= len; d += step) {
      canvas.drawCircle(a + dir * d, 1.25, paint);
    }
  }

  void _plate(Canvas canvas) {
    final panel = const Rect.fromLTRB(885, 117, 1360, 668);
    final splitX = 1123.0;
    final splitY = 354.0;
    final arcY = 259.0;
    canvas.drawRect(panel, Paint()..color = _paper);
    _arcs(canvas, Rect.fromLTRB(splitX, panel.top, panel.right, arcY));
    _burst(canvas, const Rect.fromLTWH(1124, 260, 93, 93));
    _quarters(
      canvas,
      Rect.fromLTRB(1218, 260, panel.right, splitY),
    );
    _clover(canvas, Rect.fromLTRB(panel.left, splitY, splitX, panel.bottom));
    _field(canvas, Rect.fromLTRB(splitX, splitY, panel.right, panel.bottom));
    final disc = Rect.fromLTRB(panel.left, panel.top, splitX, splitY);
    canvas.drawCircle(
      disc.center,
      disc.width * 0.357,
      Paint()..color = swap ? _yolk : _blue,
    );
    final rule = Paint()
      ..color = _ink
      ..strokeWidth = 2
      ..style = PaintingStyle.stroke;
    canvas.drawRect(panel, rule);
    final line = Paint()
      ..color = _ink
      ..strokeWidth = 2;
    canvas.drawLine(Offset(splitX, panel.top), Offset(splitX, panel.bottom), line);
    canvas.drawLine(Offset(panel.left, splitY), Offset(panel.right, splitY), line);
    canvas.drawLine(Offset(splitX, arcY), Offset(panel.right, arcY), line);
  }

  void _arcs(Canvas canvas, Rect cell) {
    canvas.save();
    canvas.clipRect(cell);
    canvas.drawRect(cell, Paint()..color = _ink);
    final stroke = Paint()
      ..color = _paper
      ..style = PaintingStyle.stroke
      ..strokeWidth = 8;
    for (var i = 0; i < 7; i++) {
      final radius = 22.0 + i * 36;
      canvas.drawArc(
        Rect.fromCircle(center: cell.topLeft, radius: radius),
        0,
        1.5707963,
        false,
        stroke,
      );
    }
    canvas.restore();
  }

  void _burst(Canvas canvas, Rect sun) {
    canvas.save();
    canvas.clipRect(sun);
    const wedge = 0.3926990817;
    final disc = Rect.fromCircle(center: sun.center, radius: sun.width);
    for (var i = 0; i < 16; i++) {
      canvas.drawArc(
        disc,
        i * wedge,
        wedge,
        true,
        Paint()..color = i.isOdd ? _yolk : _ink,
      );
    }
    canvas.restore();
  }

  void _quarters(Canvas canvas, Rect cell) {
    final radius = cell.height * 0.44;
    final oval = Rect.fromCircle(center: cell.center, radius: radius);
    final ink = Paint()..color = _ink;
    canvas.drawArc(oval, 0, 1.5707963, true, ink);
    canvas.drawArc(oval, 3.14159265, 1.5707963, true, ink);
    canvas.drawCircle(
      cell.center,
      radius,
      Paint()
        ..color = _ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.6,
    );
  }

  void _clover(Canvas canvas, Rect cell) {
    final radius = cell.width * 0.231;
    final paint = Paint()..color = _ink;
    final c = cell.center;
    canvas.drawCircle(c + Offset(radius, 0), radius, paint);
    canvas.drawCircle(c + Offset(-radius, 0), radius, paint);
    canvas.drawCircle(c + Offset(0, radius), radius, paint);
    canvas.drawCircle(c + Offset(0, -radius), radius, paint);
  }

  void _field(Canvas canvas, Rect cell) {
    canvas.save();
    canvas.clipRect(cell);
    canvas.drawRect(cell, Paint()..color = _ink);
    const cols = 4;
    final colW = cell.width / cols;
    final paint = Paint()
      ..color = _paper
      ..style = PaintingStyle.stroke
      ..strokeWidth = colW * 0.52
      ..strokeJoin = StrokeJoin.miter
      ..strokeMiterLimit = 3
      ..strokeCap = StrokeCap.butt;
    const bends = 4;
    final amp = colW * 0.34;
    for (var c = 0; c < cols; c++) {
      final mid = cell.left + colW * (c + 0.5);
      final flip = c.isOdd ? -1.0 : 1.0;
      final path = Path();
      for (var i = 0; i <= bends; i++) {
        final x = mid + flip * (i.isEven ? -amp : amp);
        final y = i == 0
            ? cell.top - 12
            : i == bends
                ? cell.bottom + 12
                : cell.top + cell.height * i / bends;
        if (i == 0) {
          path.moveTo(x, y);
        } else {
          path.lineTo(x, y);
        }
      }
      canvas.drawPath(path, paint);
    }
    canvas.restore();
  }

  void _bars(Canvas canvas) {
    final paint = Paint()..color = _ink;
    const bars = <List<double>>[
      [1378, 2, 227, 667],
      [1386, 3, 117, 667],
      [1391, 2, 117, 667],
      [1399, 1.6, 117, 667],
      [1404, 2, 117, 667],
      [1409, 2, 255, 667],
      [1413, 2, 193, 667],
      [1417, 2, 231, 667],
      [1421, 2, 117, 667],
    ];
    for (final bar in bars) {
      canvas.drawRect(
        Rect.fromLTWH(bar[0], bar[2], bar[1], bar[3] - bar[2]),
        paint,
      );
    }
    _type(canvas, '00', 'Arial', 17, 1395, 680);
  }
}
