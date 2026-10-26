import 'package:flutter/widgets.dart';

class TourLetters extends StatefulWidget {
  const TourLetters({super.key});

  @override
  State<TourLetters> createState() => _TourLettersState();
}

class _TourLettersState extends State<TourLetters> {
  int _seconds = 12;

  void _step() => setState(() => _seconds += 1);

  String get _readout {
    final m = _seconds ~/ 60;
    final s = _seconds % 60;
    final ss = s < 10 ? '0$s' : '$s';
    return '$m:$ss  24';
  }

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: SizedBox.expand(
        child: ClipRect(
          child: ColoredBox(
            color: const Color(0xFFE3E3E3),
            child: LayoutBuilder(
              builder: (context, constraints) {
                final h = constraints.maxHeight;
                final fs = h * 0.026;
                return Stack(
                  clipBehavior: Clip.hardEdge,
                  fit: StackFit.expand,
                  children: [
                    Positioned(
                      left: 0,
                      right: 0,
                      top: 0,
                      height: h * 0.61,
                      child: const ClipRect(
                        child: CustomPaint(painter: _WordPainter()),
                      ),
                    ),
                    Positioned(
                      left: 0,
                      right: 0,
                      top: h * 0.69,
                      bottom: h * 0.028,
                      child: const CustomPaint(painter: _BandPainter()),
                    ),
                    Positioned(
                      left: 0,
                      right: 0,
                      top: 0,
                      height: h * 0.61,
                      child: Center(
                        child: GestureDetector(
                          onTap: _step,
                          behavior: HitTestBehavior.opaque,
                          child: DecoratedBox(
                            decoration: const BoxDecoration(
                              color: Color(0xFF5AF5D0),
                              borderRadius: BorderRadius.all(
                                Radius.circular(1000),
                              ),
                            ),
                            child: Padding(
                              padding: EdgeInsets.symmetric(
                                horizontal: fs * 1.2,
                                vertical: fs * 0.48,
                              ),
                              child: Text(
                                _readout,
                                textScaler: TextScaler.noScaling,
                                maxLines: 1,
                                style: TextStyle(
                                  color: const Color(0xFF000000),
                                  decoration: TextDecoration.none,
                                  fontFamily: 'Arial',
                                  fontSize: fs,
                                  fontWeight: FontWeight.w700,
                                  height: 1,
                                ),
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      ),
    );
  }
}

class _WordPainter extends CustomPainter {
  const _WordPainter();

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;
    final painter = TextPainter(
      text: TextSpan(
        text: 'FIELD',
        style: TextStyle(
          color: const Color(0xFF000000),
          decoration: TextDecoration.none,
          fontFamily: 'Arial Black',
          fontSize: size.height * 2.2,
          fontWeight: FontWeight.w900,
          height: 1,
          letterSpacing: -size.height * 0.03,
        ),
      ),
      textDirection: TextDirection.ltr,
      textScaler: TextScaler.noScaling,
    )..layout();
    final sx = size.width * 0.996 / painter.width;
    final sy = size.height * 1.85 / painter.height;
    final dx = (size.width - painter.width * sx) / 2;
    canvas.save();
    canvas.translate(dx, -size.height * 0.30);
    canvas.scale(sx, sy);
    painter.paint(canvas, Offset.zero);
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _WordPainter oldDelegate) => false;
}

class _BandPainter extends CustomPainter {
  const _BandPainter();

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;
    final sw0 = size.height * 0.09;
    final r0 = size.height * 0.5 - sw0 * 0.5;
    if (r0 <= 1) return;
    const pad = 1.2;
    final minGap = r0 * 0.78;
    final need = (r0 + sw0 * 0.5 + pad) * 2 + minGap * 3;
    final scale = need > size.width ? size.width / need : 1.0;
    final sw = sw0 * scale;
    final r = r0 * scale;
    final inset = r + sw * 0.5 + pad;
    final gapMax = r * 1.32;
    var gap = (size.width - inset * 2) / 3;
    if (gap > gapMax) gap = gapMax;
    if (gap < minGap * scale) gap = minGap * scale;
    final x0 = (size.width - gap * 3) / 2;
    final y = size.height * 0.5;
    final edge = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = sw
      ..strokeJoin = StrokeJoin.round
      ..strokeCap = StrokeCap.round
      ..color = const Color(0xFFFF4D00);
    final centers = [
      Offset(x0, y),
      Offset(x0 + gap, y),
      Offset(x0 + gap * 2, y),
      Offset(x0 + gap * 3, y),
    ];
    _fill(canvas, _flower(centers[0], r), const Color(0xFFFFD000), edge);
    _fill(canvas, _blob(centers[1], r), const Color(0xFF3A74FF), edge);
    _fill(canvas, _clover(centers[2], r), const Color(0xFF8BE04C), edge);
    _checker(canvas, centers[3], r, edge);
  }

  @override
  bool shouldRepaint(covariant _BandPainter oldDelegate) => false;
}

void _fill(Canvas canvas, Path path, Color color, Paint edge) {
  canvas.drawPath(path, Paint()..color = color);
  canvas.drawPath(path, edge);
}

Path _disk(Offset c, double radius) {
  return Path()..addOval(Rect.fromCircle(center: c, radius: radius));
}

Path _flower(Offset c, double r) {
  final petal = r * 0.58;
  final o = r * 0.40;
  return Path.combine(
    PathOperation.union,
    Path.combine(
      PathOperation.union,
      _disk(Offset(c.dx, c.dy - o), petal),
      _disk(c + Offset(o, 0), petal),
    ),
    Path.combine(
      PathOperation.union,
      _disk(c + Offset(0, o), petal),
      _disk(c + Offset(-o, 0), petal),
    ),
  );
}

Path _blob(Offset c, double r) {
  final rt = r * 0.90;
  final rr = r * 0.98;
  final rb = r * 0.78;
  final rl = r * 0.94;
  final k = 0.5522847498;
  return Path()
    ..moveTo(c.dx + r * 0.06, c.dy - rt)
    ..cubicTo(
      c.dx + rr * k,
      c.dy - rt,
      c.dx + rr,
      c.dy - rr * k * 0.82,
      c.dx + rr,
      c.dy + r * 0.06,
    )
    ..cubicTo(
      c.dx + rr,
      c.dy + rb * k,
      c.dx + rb * k * 0.65,
      c.dy + rb,
      c.dx - r * 0.04,
      c.dy + rb,
    )
    ..cubicTo(
      c.dx - rl * k,
      c.dy + rb * 0.94,
      c.dx - rl,
      c.dy + rl * k * 0.35,
      c.dx - rl,
      c.dy - r * 0.02,
    )
    ..cubicTo(
      c.dx - rl,
      c.dy - rt * k,
      c.dx - rt * k * 0.25,
      c.dy - rt,
      c.dx + r * 0.06,
      c.dy - rt,
    )
    ..close();
}

Path _clover(Offset c, double r) {
  final petal = r * 0.56;
  final o = r * 0.40;
  return Path.combine(
    PathOperation.union,
    _disk(c + Offset(0, -o), petal),
    Path.combine(
      PathOperation.union,
      _disk(c + Offset(-o * 0.92, o * 0.70), petal),
      _disk(c + Offset(o * 0.92, o * 0.70), petal),
    ),
  );
}

void _checker(Canvas canvas, Offset c, double r, Paint edge) {
  final side = r * 1.72;
  final rect = Rect.fromCenter(center: c, width: side, height: side);
  final rr = RRect.fromRectAndRadius(rect, Radius.circular(side * 0.40));
  canvas.save();
  canvas.clipRRect(rr);
  const n = 4;
  final cell = side / n;
  const dark = Color(0xFF141414);
  const light = Color(0xFFF7F4EE);
  for (var row = 0; row < n; row++) {
    for (var col = 0; col < n; col++) {
      canvas.drawRect(
        Rect.fromLTWH(
          rect.left + col * cell,
          rect.top + row * cell,
          cell + 0.6,
          cell + 0.6,
        ),
        Paint()..color = (row + col).isEven ? dark : light,
      );
    }
  }
  canvas.restore();
  canvas.drawRRect(rr, edge);
}
