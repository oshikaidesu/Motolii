import 'package:flutter/widgets.dart';

class TourStickers extends StatefulWidget {
  const TourStickers({super.key});

  @override
  State<TourStickers> createState() => _TourStickersState();
}

class _TourStickersState extends State<TourStickers> {
  final List<int> _step = List<int>.filled(9, 0);

  static const List<Offset> _hops = <Offset>[
    Offset.zero,
    Offset(10, 2),
    Offset(-2, 10),
    Offset(-10, -2),
    Offset(2, -10),
  ];

  void _nudge(int index) {
    setState(() => _step[index] = (_step[index] + 1) % _hops.length);
  }

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: LayoutBuilder(
        builder: (context, box) {
          final w = box.hasBoundedWidth ? box.maxWidth : 0.0;
          final h = box.hasBoundedHeight ? box.maxHeight : 0.0;
          if (w <= 0 || h <= 0) {
            return const ColoredBox(color: Color(0xFF8F8F8F));
          }
          final u = w < h ? w : h;
          final label = (u * 0.02).clamp(8.0, 12.0);
          const gap = 4.0;
          final numStyle = TextStyle(
            decoration: TextDecoration.none,
            fontFamily: 'Helvetica Neue',
            fontFamilyFallback: const <String>['Arial', 'Helvetica'],
            fontSize: label,
            height: 1,
            color: const Color(0xFF161616),
          );

          Widget sticker({
            required int index,
            required double fx,
            required double fy,
            required double fw,
            required double fh,
            required Widget mark,
            String? number,
          }) {
            final markW = fw * u;
            final markH = fh * u;
            final extra = number == null ? 0.0 : gap + label * 1.3;
            final boxW = markW;
            final boxH = markH + extra;
            final left = _clamp(fx * w + _hops[_step[index]].dx, boxW, w);
            final top = _clamp(fy * h + _hops[_step[index]].dy, boxH, h);
            return Positioned(
              left: left,
              top: top,
              width: boxW,
              height: boxH,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => _nudge(index),
                child: Stack(
                  children: <Widget>[
                    Positioned(
                      left: 0,
                      top: 0,
                      width: markW,
                      height: markH,
                      child: mark,
                    ),
                    if (number != null)
                      Positioned(
                        left: 0,
                        top: markH + gap,
                        child: Text(number, style: numStyle),
                      ),
                  ],
                ),
              ),
            );
          }

          return ColoredBox(
            color: const Color(0xFF8F8F8F),
            child: SizedBox(
              width: w,
              height: h,
              child: Stack(
                clipBehavior: Clip.hardEdge,
                children: <Widget>[
                  sticker(
                    index: 0,
                    fx: 0.05,
                    fy: 0.08,
                    fw: 0.15,
                    fh: 0.18,
                    number: '01',
                    mark: const CustomPaint(painter: _RingsPainter()),
                  ),
                  sticker(
                    index: 1,
                    fx: 0.68,
                    fy: 0.05,
                    fw: 0.13,
                    fh: 0.145,
                    number: '02',
                    mark: const _AsteriskTriangle(),
                  ),
                  sticker(
                    index: 2,
                    fx: 0.32,
                    fy: 0.44,
                    fw: 0.10,
                    fh: 0.10,
                    number: '03',
                    mark: const CustomPaint(painter: _PixelCrossPainter()),
                  ),
                  sticker(
                    index: 3,
                    fx: 0.62,
                    fy: 0.28,
                    fw: 0.17,
                    fh: 0.052,
                    number: '04',
                    mark: const ColoredBox(
                      color: Color(0xFFE00086),
                      child: FittedBox(
                        fit: BoxFit.contain,
                        child: Padding(
                          padding: EdgeInsets.symmetric(horizontal: 8),
                          child: Text(
                            '24',
                            style: TextStyle(
                              decoration: TextDecoration.none,
                              fontFamily: 'Arial Black',
                              fontFamilyFallback: <String>[
                                'Arial',
                                'Helvetica Neue',
                              ],
                              fontSize: 28,
                              height: 1,
                              color: Color(0xFFFFFFFF),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  sticker(
                    index: 4,
                    fx: 0.16,
                    fy: 0.73,
                    fw: 0.16,
                    fh: 0.092,
                    number: '05',
                    mark: const _GreenLines(),
                  ),
                  sticker(
                    index: 5,
                    fx: 0.44,
                    fy: 0.58,
                    fw: 0.11,
                    fh: 0.11,
                    number: '06',
                    mark: const ColoredBox(
                      color: Color(0xFFC8F04A),
                      child: CustomPaint(painter: _DotRingPainter()),
                    ),
                  ),
                  sticker(
                    index: 6,
                    fx: 0.76,
                    fy: 0.66,
                    fw: 0.115,
                    fh: 0.115,
                    number: '07',
                    mark: const CustomPaint(painter: _GlobePainter()),
                  ),
                  sticker(
                    index: 7,
                    fx: 0.58,
                    fy: 0.80,
                    fw: 0.048,
                    fh: 0.048,
                    mark: const CustomPaint(painter: _CheckerPainter()),
                  ),
                  sticker(
                    index: 8,
                    fx: 0.60,
                    fy: 0.905,
                    fw: 0.12,
                    fh: 0.02,
                    mark: const DecoratedBox(
                      decoration: BoxDecoration(
                        gradient: LinearGradient(
                          colors: <Color>[
                            Color(0xFF1A1A1A),
                            Color(0xFFE23A2E),
                            Color(0xFFFFD15A),
                            Color(0xFFF4F4F4),
                          ],
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  double _clamp(double origin, double extent, double limit) {
    final max = limit - extent;
    if (max <= 0) return 0;
    if (origin < 0) return 0;
    if (origin > max) return max;
    return origin;
  }
}

class _RingsPainter extends CustomPainter {
  const _RingsPainter();

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = (size.shortestSide * 0.07).clamp(2.0, 6.0)
      ..color = const Color(0xFFFF5C1C);
    final ringW = size.width * 0.84;
    final ringH = size.height * 0.30;
    final cx = size.width * 0.5;
    for (var i = 0; i < 3; i++) {
      canvas.drawOval(
        Rect.fromCenter(
          center: Offset(cx, size.height * (0.22 + i * 0.27)),
          width: ringW,
          height: ringH,
        ),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _RingsPainter oldDelegate) => false;
}

class _AsteriskTriangle extends StatelessWidget {
  const _AsteriskTriangle();

  @override
  Widget build(BuildContext context) {
    return const Stack(
      children: <Widget>[
        Positioned.fill(child: CustomPaint(painter: _TrianglePainter())),
        Align(
          alignment: Alignment(0, 0.28),
          child: FractionallySizedBox(
            widthFactor: 0.34,
            heightFactor: 0.34,
            child: FittedBox(
              child: Text(
                '*',
                style: TextStyle(
                  decoration: TextDecoration.none,
                  fontFamily: 'Arial Black',
                  fontFamilyFallback: <String>['Arial', 'Helvetica Neue'],
                  fontSize: 40,
                  height: 1,
                  color: Color(0xFFFFFFFF),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _TrianglePainter extends CustomPainter {
  const _TrianglePainter();

  @override
  void paint(Canvas canvas, Size size) {
    final path = Path()
      ..moveTo(size.width * 0.50, size.height * 0.02)
      ..lineTo(size.width * 0.98, size.height * 0.96)
      ..lineTo(size.width * 0.02, size.height * 0.96)
      ..close();
    canvas.drawPath(path, Paint()..color = const Color(0xFF2A4BFF));
  }

  @override
  bool shouldRepaint(covariant _TrianglePainter oldDelegate) => false;
}

class _PixelCrossPainter extends CustomPainter {
  const _PixelCrossPainter();

  static const List<String> _rows = <String>[
    '001100',
    '001100',
    '111111',
    '111111',
    '001100',
    '001100',
  ];

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = const Color(0xFFF3F3F3),
    );
    final cw = size.width / 6;
    final ch = size.height / 6;
    final ink = Paint()..color = const Color(0xFF111111);
    for (var y = 0; y < 6; y++) {
      for (var x = 0; x < 6; x++) {
        if (_rows[y][x] != '1') continue;
        canvas.drawRect(
          Rect.fromLTWH(x * cw, y * ch, cw + 0.4, ch + 0.4),
          ink,
        );
      }
    }
  }

  @override
  bool shouldRepaint(covariant _PixelCrossPainter oldDelegate) => false;
}

class _GreenLines extends StatelessWidget {
  const _GreenLines();

  static const TextStyle _mono = TextStyle(
    decoration: TextDecoration.none,
    fontFamily: 'Menlo',
    fontFamilyFallback: <String>['Courier New', 'Courier', 'monospace'],
    fontSize: 12,
    height: 1.15,
    color: Color(0xFFFFFFFF),
  );

  @override
  Widget build(BuildContext context) {
    return const ColoredBox(
      color: Color(0xFF147A34),
      child: FittedBox(
        alignment: Alignment.centerLeft,
        fit: BoxFit.scaleDown,
        child: Padding(
          padding: EdgeInsets.fromLTRB(8, 5, 12, 5),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              Text('FINAL', style: _mono),
              Text('HOLD', style: _mono),
              Text('CUT', style: _mono),
            ],
          ),
        ),
      ),
    );
  }
}

class _DotRingPainter extends CustomPainter {
  const _DotRingPainter();

  static const List<Offset> _ring = <Offset>[
    Offset(1, 0),
    Offset(0.809, 0.588),
    Offset(0.309, 0.951),
    Offset(-0.309, 0.951),
    Offset(-0.809, 0.588),
    Offset(-1, 0),
    Offset(-0.809, -0.588),
    Offset(-0.309, -0.951),
    Offset(0.309, -0.951),
    Offset(0.809, -0.588),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final c = Offset(size.width / 2, size.height / 2);
    final radius = size.shortestSide * 0.30;
    final dot = size.shortestSide * 0.055;
    final paint = Paint()..color = const Color(0xFF14200C);
    for (final p in _ring) {
      canvas.drawCircle(
        c + Offset(p.dx * radius, p.dy * radius),
        dot,
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _DotRingPainter oldDelegate) => false;
}

class _GlobePainter extends CustomPainter {
  const _GlobePainter();

  @override
  void paint(Canvas canvas, Size size) {
    final c = Offset(size.width / 2, size.height / 2);
    final r = size.shortestSide * 0.46;
    canvas.drawCircle(c, r, Paint()..color = const Color(0xFFF7F7F7));
    final ink = Paint()
      ..color = const Color(0xFF1A1A1A)
      ..style = PaintingStyle.stroke
      ..strokeWidth = (size.shortestSide * 0.035).clamp(1.0, 2.0);
    canvas.drawCircle(c, r, ink);
    canvas.drawLine(Offset(c.dx - r, c.dy), Offset(c.dx + r, c.dy), ink);
    canvas.drawLine(Offset(c.dx, c.dy - r), Offset(c.dx, c.dy + r), ink);
    canvas.drawOval(
      Rect.fromCenter(center: c, width: r * 0.72, height: r * 2),
      ink,
    );
  }

  @override
  bool shouldRepaint(covariant _GlobePainter oldDelegate) => false;
}

class _CheckerPainter extends CustomPainter {
  const _CheckerPainter();

  @override
  void paint(Canvas canvas, Size size) {
    const n = 4;
    final cw = size.width / n;
    final ch = size.height / n;
    final red = Paint()..color = const Color(0xFFE10600);
    final white = Paint()..color = const Color(0xFFFFFFFF);
    for (var y = 0; y < n; y++) {
      for (var x = 0; x < n; x++) {
        canvas.drawRect(
          Rect.fromLTWH(x * cw, y * ch, cw + 0.4, ch + 0.4),
          (x + y).isEven ? red : white,
        );
      }
    }
  }

  @override
  bool shouldRepaint(covariant _CheckerPainter oldDelegate) => false;
}
