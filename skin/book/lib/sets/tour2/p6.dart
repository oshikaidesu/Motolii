import 'package:flutter/widgets.dart';

class TourP6 extends StatefulWidget {
  const TourP6({super.key});

  @override
  State<TourP6> createState() => _TourP6State();
}

class _TourP6State extends State<TourP6> {
  bool turned = false;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final w = constraints.maxWidth;
        final h = constraints.maxHeight;
        if (!w.isFinite || !h.isFinite || w <= 0 || h <= 0) {
          return const ColoredBox(color: Color(0xFFCDB8EC));
        }
        final s = w / 1440 < h / 900 ? w / 1440 : h / 900;

        Positioned plate(double x, double y, double side, Widget child) {
          return Positioned(
            left: w - (1440 - x) * s,
            top: y * s,
            width: side * s,
            height: side * s,
            child: child,
          );
        }

        const ink = Color(0xFF101010);
        const lime = Color(0xFFC6EE3C);
        final type = TextStyle(
          fontFamily: 'Arial Black',
          fontSize: 151 * s,
          height: 0.82,
          color: const Color(0xFF000000),
          decoration: TextDecoration.none,
        );

        return Directionality(
          textDirection: TextDirection.ltr,
          child: ClipRect(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => setState(() => turned = !turned),
              child: ColoredBox(
                color: const Color(0xFFCDB8EC),
                child: Stack(
                  clipBehavior: Clip.hardEdge,
                  children: [
                    plate(
                      791,
                      33,
                      258,
                      const Stack(
                        fit: StackFit.expand,
                        children: [
                          ColoredBox(color: Color(0xFF2A4ECA)),
                          CustomPaint(painter: _Clover()),
                        ],
                      ),
                    ),
                    plate(
                      1020,
                      14,
                      206,
                      Stack(
                        fit: StackFit.expand,
                        children: [
                          ColoredBox(color: Color(0xFF7A2030)),
                          Align(
                            alignment: Alignment(0, -0.15),
                            child: FractionallySizedBox(
                              widthFactor: 0.64,
                              heightFactor: 0.64,
                              child: DecoratedBox(
                                decoration: BoxDecoration(
                                  color: Color(0xFFC6EE3C),
                                  shape: BoxShape.circle,
                                ),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                    plate(
                      900,
                      210,
                      150,
                      const Stack(
                        fit: StackFit.expand,
                        children: [
                          ColoredBox(color: ink),
                          Center(
                            child: FractionallySizedBox(
                              widthFactor: 0.36,
                              heightFactor: 0.36,
                              child: DecoratedBox(
                                decoration: BoxDecoration(
                                  color: lime,
                                  shape: BoxShape.circle,
                                ),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                    plate(
                      1073,
                      162,
                      215,
                      Transform.rotate(
                        angle: turned ? 0.78539816339 : 0,
                        child: const Stack(
                          fit: StackFit.expand,
                          children: [
                            ColoredBox(color: Color(0xFF1C7A3C)),
                            _Mark(),
                          ],
                        ),
                      ),
                    ),
                    plate(
                      1225,
                      87,
                      114,
                      const Stack(
                        fit: StackFit.expand,
                        children: [
                          ColoredBox(color: lime),
                          Center(
                            child: FractionallySizedBox(
                              widthFactor: 0.30,
                              heightFactor: 0.30,
                              child: CustomPaint(painter: _Flower()),
                            ),
                          ),
                        ],
                      ),
                    ),
                    plate(
                      996,
                      309,
                      137,
                      const Stack(
                        fit: StackFit.expand,
                        children: [
                          ColoredBox(color: Color(0xFFE1332C)),
                          Center(
                            child: FractionallySizedBox(
                              widthFactor: 0.40,
                              heightFactor: 0.40,
                              child: CustomPaint(painter: _Hex()),
                            ),
                          ),
                        ],
                      ),
                    ),
                    Positioned(
                      left: 58 * s,
                      bottom: 62 * s,
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text('HOLD', style: type),
                          Text('CUT.', style: type),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _Mark extends StatelessWidget {
  const _Mark();

  @override
  Widget build(BuildContext context) {
    return const Stack(
      alignment: Alignment.center,
      children: [_Bar(angle: 0.78539816339), _Bar(angle: -0.78539816339)],
    );
  }
}

class _Bar extends StatelessWidget {
  const _Bar({required this.angle});

  final double angle;

  @override
  Widget build(BuildContext context) {
    return Transform.rotate(
      angle: angle,
      child: const FractionallySizedBox(
        widthFactor: 0.15,
        heightFactor: 0.66,
        child: ColoredBox(color: Color(0xFF101010)),
      ),
    );
  }
}

class _Clover extends CustomPainter {
  const _Clover();

  @override
  void paint(Canvas canvas, Size size) {
    final c = Offset(size.width * 0.50, size.height * 0.52);
    final r = size.width * 0.210;
    final sep = size.width * 0.148;
    final paint = Paint()..color = const Color(0xFF101010);
    for (final o in <Offset>[
      Offset(0, -sep),
      Offset(-sep, 0),
      Offset(sep, 0),
      Offset(0, sep),
    ]) {
      canvas.drawCircle(c + o, r, paint);
    }
  }

  @override
  bool shouldRepaint(covariant _Clover oldDelegate) => false;
}

class _Flower extends CustomPainter {
  const _Flower();

  @override
  void paint(Canvas canvas, Size size) {
    final c = Offset(size.width / 2, size.height / 2);
    final paint = Paint()..color = const Color(0xFF101010);
    final orbit = size.width * 0.30;
    final petal = size.width * 0.175;
    const dirs = <Offset>[
      Offset(1, 0),
      Offset(0.5, 0.8660254),
      Offset(-0.5, 0.8660254),
      Offset(-1, 0),
      Offset(-0.5, -0.8660254),
      Offset(0.5, -0.8660254),
    ];
    for (final d in dirs) {
      canvas.drawCircle(c + d * orbit, petal, paint);
    }
    canvas.drawCircle(c, size.width * 0.07, paint);
  }

  @override
  bool shouldRepaint(covariant _Flower oldDelegate) => false;
}

class _Hex extends CustomPainter {
  const _Hex();

  @override
  void paint(Canvas canvas, Size size) {
    final c = Offset(size.width / 2, size.height / 2);
    final r = size.shortestSide * 0.48;
    const k = 0.8660254;
    final pts = <Offset>[
      const Offset(0, -1),
      const Offset(k, -0.5),
      const Offset(k, 0.5),
      const Offset(0, 1),
      const Offset(-k, 0.5),
      const Offset(-k, -0.5),
    ];
    final path = Path()..moveTo(c.dx + pts[0].dx * r, c.dy + pts[0].dy * r);
    for (var i = 1; i < pts.length; i++) {
      path.lineTo(c.dx + pts[i].dx * r, c.dy + pts[i].dy * r);
    }
    path.close();
    canvas.drawPath(path, Paint()..color = const Color(0xFF101010));
  }

  @override
  bool shouldRepaint(covariant _Hex oldDelegate) => false;
}
