import 'package:flutter/widgets.dart';

const _pi = 3.141592653589793;
const _ink = Color(0xFF121212);
const _paper = Color(0xFFF3EFE4);
const _blue = Color(0xFF1A3CE0);
const _yellow = Color(0xFFFFD000);
const _white = Color(0xFFFFFFFF);

class TourP2 extends StatefulWidget {
  const TourP2({super.key});

  @override
  State<TourP2> createState() => _TourP2State();
}

class _TourP2State extends State<TourP2> {
  bool _alt = false;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth.isFinite ? box.maxWidth : 1440.0;
        final h = box.maxHeight.isFinite ? box.maxHeight : 900.0;
        return ClipRect(
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () => setState(() => _alt = !_alt),
            child: SizedBox(
              width: w,
              height: h,
              child: FittedBox(
                fit: BoxFit.cover,
                clipBehavior: Clip.hardEdge,
                child: SizedBox(
                  width: 1440,
                  height: 900,
                  child: Stack(
                    clipBehavior: Clip.hardEdge,
                    children: [
                      Positioned.fill(
                        child: CustomPaint(
                          painter: _Plate(alt: _alt),
                          child: const SizedBox.expand(),
                        ),
                      ),
                      Positioned(
                        left: 66,
                        top: 512,
                        child: Text('CUT', style: _type('Arial Black', 38)),
                      ),
                      Positioned(
                        left: 66,
                        top: 548,
                        child: Text('HOLD', style: _type('Arial Black', 38)),
                      ),
                      const Positioned(
                        left: 1392,
                        top: 678,
                        child: Text(
                          '00',
                          style: TextStyle(
                            fontFamily: 'Menlo',
                            fontSize: 12,
                            height: 1,
                            color: _ink,
                            decoration: TextDecoration.none,
                          ),
                        ),
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

TextStyle _type(String family, double size) {
  return TextStyle(
    fontFamily: family,
    fontSize: size,
    height: 0.9,
    color: _ink,
    decoration: TextDecoration.none,
  );
}

class _Plate extends CustomPainter {
  const _Plate({required this.alt});

  final bool alt;

  @override
  void paint(Canvas canvas, Size size) {
    final ink = Paint()..color = _ink;
    final disc = alt ? _yellow : _blue;
    final burst = alt ? _blue : _yellow;
    canvas.drawRect(Offset.zero & size, Paint()..color = _paper);
    _marks(canvas, ink);
    canvas.drawRect(const Rect.fromLTRB(885, 116, 1361, 669), Paint()..color = _white);
    _disc(canvas, disc);
    _arcs(canvas, ink);
    _diamonds(canvas, ink);
    _clover(canvas, ink);
    _burst(canvas, burst, ink);
    _split(canvas, ink, alt ? _pi / 2 : 0);
    _frame(canvas, ink);
    _bars(canvas, ink);
    _em(canvas, ink);
  }

  void _marks(Canvas canvas, Paint ink) {
    _ring(canvas, const Offset(528, 118), 8);
    _ring(canvas, const Offset(556, 168), 5.5);
    _ring(canvas, const Offset(523, 209), 4);
    _dashes(canvas, const Offset(556, 274), const Offset(864, 274), ink);
    _dashes(canvas, const Offset(865, 276), const Offset(865, 666), ink);
    _dashes(canvas, const Offset(786, 668), const Offset(864, 668), ink);
  }

  void _ring(Canvas canvas, Offset c, double r) {
    canvas.drawCircle(
      c,
      r,
      Paint()
        ..color = _ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.6,
    );
  }

  void _dashes(Canvas canvas, Offset a, Offset b, Paint ink) {
    final dx = b.dx - a.dx;
    final dy = b.dy - a.dy;
    final span = dx.abs() + dy.abs();
    final steps = span / 8;
    for (var i = 0; i < steps; i++) {
      final t = i / steps;
      canvas.drawRect(
        Rect.fromCenter(
          center: Offset(a.dx + dx * t, a.dy + dy * t),
          width: dx.abs() >= dy.abs() ? 3.4 : 2.2,
          height: dx.abs() >= dy.abs() ? 2.2 : 3.4,
        ),
        ink,
      );
    }
  }

  void _disc(Canvas canvas, Color color) {
    canvas.save();
    canvas.clipRect(const Rect.fromLTRB(887, 118, 1122, 353));
    canvas.drawCircle(const Offset(1004, 236), 85, Paint()..color = color);
    canvas.restore();
  }

  void _arcs(Canvas canvas, Paint ink) {
    const cell = Rect.fromLTRB(1124, 118, 1359, 353);
    canvas.save();
    canvas.clipRect(cell);
    canvas.drawRect(cell, ink);
    const origin = Offset(1123, 117);
    final white = Paint()
      ..color = _white
      ..style = PaintingStyle.stroke
      ..strokeWidth = 8;
    for (final radius in const [84.0, 125.0, 165.0, 205.0, 244.0]) {
      canvas.drawArc(
        Rect.fromCircle(center: origin, radius: radius),
        0,
        _pi / 2,
        false,
        white,
      );
    }
    canvas.restore();
  }

  void _diamonds(Canvas canvas, Paint ink) {
    canvas.save();
    canvas.clipRect(const Rect.fromLTRB(1124, 356, 1359, 667));
    const pitch = 59.5;
    const hw = pitch / 2;
    const vh = 104.0;
    const centers = [1152.5, 1212.0, 1271.5, 1331.0];
    for (var i = 0; i < centers.length; i++) {
      final cx = centers[i];
      final anchor = i.isEven ? 456.0 : 560.0;
      for (var k = -1; k <= 2; k++) {
        final cy = anchor + k * vh * 2;
        canvas.drawPath(
          Path()
            ..moveTo(cx, cy - vh)
            ..lineTo(cx + hw, cy)
            ..lineTo(cx, cy + vh)
            ..lineTo(cx - hw, cy)
            ..close(),
          ink,
        );
      }
    }
    canvas.restore();
  }

  void _clover(Canvas canvas, Paint ink) {
    canvas.save();
    canvas.clipRect(const Rect.fromLTRB(887, 356, 1122, 667));
    const r = 54.0;
    canvas.drawCircle(const Offset(1004, 456), r, ink);
    canvas.drawCircle(const Offset(1004, 565), r, ink);
    canvas.drawCircle(const Offset(949, 511), r, ink);
    canvas.drawCircle(const Offset(1058, 511), r, ink);
    canvas.restore();
  }

  void _burst(Canvas canvas, Color ground, Paint ink) {
    const rect = Rect.fromLTWH(1124, 260, 92, 94);
    final oval = Rect.fromCircle(center: rect.center, radius: 70);
    canvas.save();
    canvas.clipRect(rect);
    canvas.drawRect(rect, Paint()..color = ground);
    for (var i = 0; i < 9; i++) {
      canvas.drawArc(oval, -_pi / 18 + i * 2 * _pi / 9, _pi / 9, true, ink);
    }
    canvas.restore();
    canvas.drawRect(
      rect,
      Paint()
        ..color = _ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
  }

  void _split(Canvas canvas, Paint ink, double turn) {
    const c = Offset(1289, 306);
    const r = 41.0;
    canvas.drawRect(const Rect.fromLTRB(1219, 260, 1359, 353), Paint()..color = _white);
    final oval = Rect.fromCircle(center: c, radius: r);
    canvas.drawCircle(c, r, Paint()..color = _white);
    canvas.drawArc(oval, turn, _pi / 2, true, ink);
    canvas.drawArc(oval, turn + _pi, _pi / 2, true, ink);
    canvas.drawCircle(
      c,
      r,
      Paint()
        ..color = _ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
  }

  void _frame(Canvas canvas, Paint ink) {
    canvas.drawRect(
      const Rect.fromLTRB(885, 116, 1361, 669),
      Paint()
        ..color = _ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
    canvas.drawLine(const Offset(1123, 116), const Offset(1123, 669), ink);
    canvas.drawLine(const Offset(885, 354), const Offset(1361, 354), ink);
  }

  void _bars(Canvas canvas, Paint ink) {
    const bars = <(double, double, double)>[
      (1378, 2, 227),
      (1382, 1, 117),
      (1384, 1, 117),
      (1386, 3, 117),
      (1390, 3, 117),
      (1395, 1, 173),
      (1399, 1, 117),
      (1402, 1, 117),
      (1404, 2, 117),
      (1409, 2, 254),
      (1412, 3, 192),
      (1417, 2, 230),
      (1421, 2, 117),
    ];
    for (final bar in bars) {
      canvas.drawRect(Rect.fromLTWH(bar.$1, bar.$3, bar.$2, 668 - bar.$3), ink);
    }
  }

  void _em(Canvas canvas, Paint ink) {
    canvas.drawRect(const Rect.fromLTRB(98, 75, 179, 401), ink);
    canvas.drawRect(const Rect.fromLTRB(383, 75, 464, 401), ink);
    canvas.drawPath(
      Path()
        ..moveTo(159, 75)
        ..lineTo(229, 75)
        ..lineTo(314, 401)
        ..lineTo(244, 401)
        ..close(),
      ink,
    );
    canvas.drawPath(
      Path()
        ..moveTo(333, 75)
        ..lineTo(403, 75)
        ..lineTo(318, 401)
        ..lineTo(248, 401)
        ..close(),
      ink,
    );
  }

  @override
  bool shouldRepaint(_Plate oldDelegate) => oldDelegate.alt != alt;
}
