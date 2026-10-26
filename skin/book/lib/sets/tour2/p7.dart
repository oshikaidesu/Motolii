import 'package:flutter/widgets.dart';

class TourP7 extends StatefulWidget {
  const TourP7({super.key});

  @override
  State<TourP7> createState() => _TourP7State();
}

class _TourP7State extends State<TourP7> {
  bool _swapped = false;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final w = constraints.maxWidth.isFinite ? constraints.maxWidth : 1440.0;
        final h = constraints.maxHeight.isFinite ? constraints.maxHeight : 900.0;
        return ClipRect(
          child: SizedBox(
            width: w,
            height: h,
            child: FittedBox(
              fit: BoxFit.cover,
              clipBehavior: Clip.hardEdge,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => setState(() => _swapped = !_swapped),
                child: SizedBox(
                  width: 1440,
                  height: 900,
                  child: Stack(
                    clipBehavior: Clip.hardEdge,
                    children: [
                      Positioned.fill(
                        child: CustomPaint(painter: _Sheet(swapped: _swapped)),
                      ),
                      const Positioned(
                        left: 66,
                        top: 508,
                        child: Text('CUT', style: _word),
                      ),
                      const Positioned(
                        left: 66,
                        top: 546,
                        child: Text('HOLD', style: _word),
                      ),
                      const Positioned(
                        left: 1394,
                        top: 676,
                        child: Text('00', style: _index),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

const _word = TextStyle(
  fontFamily: 'Arial Black',
  fontSize: 34,
  height: 1,
  color: Color(0xFF121212),
  decoration: TextDecoration.none,
);

const _index = TextStyle(
  fontFamily: 'Arial',
  fontSize: 13,
  height: 1,
  color: Color(0xFF121212),
  decoration: TextDecoration.none,
);

class _Sheet extends CustomPainter {
  const _Sheet({required this.swapped});

  final bool swapped;

  static const _paper = Color(0xFFF3EFE4);
  static const _ink = Color(0xFF121212);
  static const _blue = Color(0xFF1A3CE0);
  static const _yellow = Color(0xFFFFD000);
  static const _white = Color(0xFFFFFFFF);

  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.scale(size.width / 1440, size.height / 900);
    final ink = Paint()..color = _ink;
    final paper = Paint()..color = _paper;
    canvas.drawRect(const Rect.fromLTWH(0, 0, 1440, 900), paper);

    canvas.drawRect(const Rect.fromLTRB(98, 75, 180, 402), ink);
    canvas.drawRect(const Rect.fromLTRB(383, 75, 465, 402), ink);
    canvas.drawPath(
      Path()
        ..moveTo(180, 75)
        ..lineTo(230, 75)
        ..lineTo(281, 273)
        ..lineTo(333, 75)
        ..lineTo(383, 75)
        ..lineTo(383, 151)
        ..lineTo(319, 402)
        ..lineTo(244, 402)
        ..lineTo(180, 151)
        ..close(),
      ink,
    );

    final ring = Paint()
      ..color = _ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4;
    canvas.drawCircle(const Offset(526, 118), 6.6, ring);
    ring.strokeWidth = 1.2;
    canvas.drawCircle(const Offset(555, 166), 4, ring);
    ring.strokeWidth = 1.1;
    canvas.drawCircle(const Offset(520, 210), 2.8, ring);

    _dots(canvas, ink, 554, 274, 865, 274);
    _dots(canvas, ink, 865, 274, 865, 668);
    _dots(canvas, ink, 785, 668, 865, 668);
    canvas.drawRect(const Rect.fromLTWH(551, 271, 5, 5), ink);
    canvas.drawRect(const Rect.fromLTWH(863, 271, 5, 5), ink);
    canvas.drawRect(const Rect.fromLTWH(782, 665, 5, 5), ink);
    canvas.drawRect(const Rect.fromLTWH(863, 665, 5, 5), ink);
    canvas.drawCircle(const Offset(709.5, 328), 2.2, ink);

    canvas.drawRect(const Rect.fromLTRB(887, 118, 1122, 353), Paint()..color = _white);
    canvas.drawRect(const Rect.fromLTRB(1123, 118, 1360, 353), ink);
    canvas.drawRect(const Rect.fromLTRB(887, 355, 1122, 666), Paint()..color = _white);
    canvas.drawRect(const Rect.fromLTRB(1124, 356, 1359, 666), ink);

    canvas.drawCircle(
      const Offset(1004, 235.5),
      85,
      Paint()..color = swapped ? _yellow : _blue,
    );

    canvas.save();
    canvas.clipRect(const Rect.fromLTRB(1123, 118, 1360, 354));
    final arc = Paint()
      ..color = _white
      ..style = PaintingStyle.stroke
      ..strokeWidth = 5;
    const corner = Offset(1360, 118);
    for (final radius in [22.0, 50.0, 78.0, 106.0, 134.0, 162.0, 190.0, 218.0]) {
      canvas.drawArc(
        Rect.fromCircle(center: corner, radius: radius),
        1.5707963,
        1.5707963,
        false,
        arc,
      );
    }
    canvas.restore();

    final burst = Rect.fromLTRB(1124, 260, 1216, 353);
    canvas.drawRect(burst, Paint()..color = swapped ? _blue : _yellow);
    canvas.save();
    canvas.clipRect(burst);
    const hub = Offset(1170, 306);
    const tips = <Offset>[
      Offset(1170, 216),
      Offset(1204.4, 222.9),
      Offset(1233.6, 242.4),
      Offset(1253.1, 271.6),
      Offset(1260, 306),
      Offset(1253.1, 340.4),
      Offset(1233.6, 369.6),
      Offset(1204.4, 389.1),
      Offset(1170, 396),
      Offset(1135.6, 389.1),
      Offset(1106.4, 369.6),
      Offset(1086.9, 340.4),
      Offset(1080, 306),
      Offset(1086.9, 271.6),
      Offset(1106.4, 242.4),
      Offset(1135.6, 222.9),
    ];
    for (var i = 0; i < tips.length; i += 2) {
      canvas.drawPath(
        Path()
          ..moveTo(hub.dx, hub.dy)
          ..lineTo(tips[i].dx, tips[i].dy)
          ..lineTo(tips[i + 1].dx, tips[i + 1].dy)
          ..close(),
        ink,
      );
    }
    canvas.restore();

    canvas.drawRect(const Rect.fromLTRB(1219, 260, 1359, 353), Paint()..color = _white);
    final pie = Paint()..color = _ink;
    final dial = Rect.fromCircle(center: const Offset(1289, 307), radius: 41);
    canvas.drawArc(dial, 0, 1.5707963, true, pie);
    canvas.drawArc(dial, 3.14159265, 1.5707963, true, pie);

    final lobe = Paint()..color = _ink;
    for (final c in const [
      Offset(1004, 457),
      Offset(1004, 565),
      Offset(950, 511),
      Offset(1058, 511),
    ]) {
      canvas.drawCircle(c, 54, lobe);
    }

    canvas.save();
    canvas.clipRect(const Rect.fromLTRB(1125, 357, 1358, 665));
    final lozenge = Paint()..color = _white;
    for (var i = 0; i < 5; i++) {
      canvas.drawPath(_ribbon(1122 + i * 60, i.isEven), lozenge);
    }
    canvas.restore();

    final rule = Paint()
      ..color = _ink
      ..strokeWidth = 2;
    canvas.drawLine(const Offset(1123, 118), const Offset(1123, 668), rule);
    canvas.drawLine(const Offset(887, 354), const Offset(1360, 354), rule);
    canvas.drawRect(
      const Rect.fromLTRB(885, 116, 1362, 669),
      Paint()
        ..color = _ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3,
    );

    _bar(canvas, ink, 1378, 2, 227);
    _bar(canvas, ink, 1382, 1, 117);
    _bar(canvas, ink, 1386, 3, 117);
    _bar(canvas, ink, 1391, 2, 117);
    _bar(canvas, ink, 1399, 1, 117);
    _bar(canvas, ink, 1404, 2, 117);
    _bar(canvas, ink, 1409, 2, 255);
    _bar(canvas, ink, 1413, 2, 193);
    _bar(canvas, ink, 1417, 2, 231);
    _bar(canvas, ink, 1421, 2, 117);

    canvas.restore();
  }

  Path _ribbon(double mean, bool invert) {
    const amp = 14.0;
    const half = 13.0;
    final left = <Offset>[];
    final right = <Offset>[];
    for (var k = 0; k <= 5; k++) {
      final y = 260.0 + k * 100;
      final sign = (k.isEven != invert) ? 1.0 : -1.0;
      final cx = mean + sign * amp;
      left.add(Offset(cx - half, y));
      right.add(Offset(cx + half, y));
    }
    final path = Path()..moveTo(left.first.dx, left.first.dy);
    for (var i = 1; i < left.length; i++) {
      path.lineTo(left[i].dx, left[i].dy);
    }
    for (var i = right.length - 1; i >= 0; i--) {
      path.lineTo(right[i].dx, right[i].dy);
    }
    return path..close();
  }

  void _bar(Canvas canvas, Paint paint, double x, double w, double y) {
    canvas.drawRect(Rect.fromLTWH(x, y, w, 667 - y), paint);
  }

  void _dots(Canvas canvas, Paint paint, double x0, double y0, double x1, double y1) {
    const step = 5.2;
    final dx = x1 - x0;
    final dy = y1 - y0;
    final span = dx.abs() > dy.abs() ? dx.abs() : dy.abs();
    final n = (span / step).round();
    for (var i = 0; i <= n; i++) {
      final t = n == 0 ? 0.0 : i / n;
      canvas.drawRect(Rect.fromLTWH(x0 + dx * t, y0 + dy * t, 2, 2), paint);
    }
  }

  @override
  bool shouldRepaint(_Sheet oldDelegate) => oldDelegate.swapped != swapped;
}
