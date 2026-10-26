import 'package:flutter/widgets.dart';

class TourScatter extends StatefulWidget {
  const TourScatter({super.key});

  @override
  State<TourScatter> createState() => _TourScatterState();
}

class _TourScatterState extends State<TourScatter> {
  double _redCard = 0.2792526803;
  double _grayCard = -0.3665191429;

  @override
  Widget build(BuildContext context) {
    return ClipRect(
      child: ColoredBox(
        color: _paper,
        child: SizedBox.expand(
          child: _Field(
            redCard: _redCard,
            grayCard: _grayCard,
            onRed: () => setState(() => _redCard += 0.1047197551),
            onGray: () => setState(() => _grayCard += 0.1047197551),
          ),
        ),
      ),
    );
  }
}

class _Field extends StatelessWidget {
  const _Field({
    required this.redCard,
    required this.grayCard,
    required this.onRed,
    required this.onGray,
  });

  final double redCard;
  final double grayCard;
  final VoidCallback onRed;
  final VoidCallback onGray;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        final h = box.maxHeight;
        final letter = h * 0.135 < w * 0.16 ? h * 0.135 : w * 0.16;
        final dot = w * 0.064;
        return Stack(
          alignment: Alignment.topLeft,
          clipBehavior: Clip.hardEdge,
          children: [
            Positioned(
              left: -w * 0.02,
              top: h * 0.04,
              width: w * 0.15,
              height: h * 0.92,
              child: const FittedBox(
                fit: BoxFit.contain,
                alignment: Alignment.centerLeft,
                child: RotatedBox(
                  quarterTurns: 3,
                  child: Text(
                    'MASK',
                    textDirection: TextDirection.ltr,
                    style: TextStyle(
                      inherit: false,
                      decoration: TextDecoration.none,
                      fontFamily: 'Arial Black',
                      fontFamilyFallback: <String>['Futura', 'Helvetica Neue'],
                      fontSize: 120,
                      letterSpacing: 24,
                      height: 1,
                      color: _ink,
                    ),
                  ),
                ),
              ),
            ),
            Positioned(
              right: -letter * 0.28,
              top: h * 0.02,
              child: Text(
                'H\nO\nL\nD',
                textAlign: TextAlign.right,
                textDirection: TextDirection.ltr,
                style: TextStyle(
                  inherit: false,
                  decoration: TextDecoration.none,
                  fontFamily: 'Arial Black',
                  fontFamilyFallback: const <String>['Futura', 'Helvetica Neue'],
                  fontSize: letter,
                  height: 0.78,
                  color: _ink,
                ),
              ),
            ),
            _at(w * 0.33, -h * 0.055, w * 0.3, h * 0.42, 0.3141592654, const CustomPaint(painter: _Poly(_burst, _teal))),
            _at(w * 0.62, h * 0.32, w * 0.22, h * 0.36, -0.4886921906, const CustomPaint(painter: _Poly(_chunk, _teal, _hole))),
            _at(w * 0.785, h * 0.18, w * 0.175, h * 0.4, 0.2443460953, const CustomPaint(painter: _Poly(_spark, _gray))),
            _at(
              w * 0.385,
              h * 0.48,
              w * 0.2,
              h * 0.46,
              grayCard,
              _Card(
                color: _gray,
                onTap: onGray,
                child: const _GrayFace(),
              ),
            ),
            _at(w * 0.22, h * 0.25, dot * 1.28, dot * 2.5, -0.2094395102, _Dots(d: dot)),
            _at(
              w * 0.598,
              -h * 0.015,
              w * 0.185,
              h * 0.34,
              redCard,
              _Card(
                color: _red,
                onTap: onRed,
                child: const _RedFace(),
              ),
            ),
            _at(w * 0.545, h * 0.7, w * 0.24, h * 0.25, 0.4188790205, const _Stamp()),
            _at(w * 0.12, h * 0.58, w * 0.135, h * 0.1, 0.5410520681, const CustomPaint(painter: _Poly(_arrow, _ink))),
            _at(w * 0.825, -h * 0.02, w * 0.14, h * 0.11, -0.5759586532, const CustomPaint(painter: _Poly(_arrow, _ink))),
            Positioned(
              right: w * 0.03,
              top: h * 0.66,
              width: w * 0.26,
              height: h * 0.24,
              child: FittedBox(
                fit: BoxFit.scaleDown,
                alignment: Alignment.topRight,
                child: Transform.rotate(
                  angle: 0.1396263402,
                  alignment: Alignment.topRight,
                  child: Text(
                    '0:12\n24',
                    textAlign: TextAlign.right,
                    textDirection: TextDirection.ltr,
                    style: _face(64, _ink, 'Arial Black'),
                  ),
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}

class _Card extends StatelessWidget {
  const _Card({required this.color, required this.onTap, required this.child});

  final Color color;
  final VoidCallback onTap;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: ColoredBox(color: color, child: child),
    );
  }
}

class _RedFace extends StatelessWidget {
  const _RedFace();

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        final h = box.maxHeight;
        return Stack(
          alignment: Alignment.topLeft,
          fit: StackFit.expand,
          children: [
            Positioned(
              left: w * 0.13,
              top: h * 0.24,
              width: w * 0.74,
              height: h * 0.52,
              child: const CustomPaint(painter: _Disk(_paper)),
            ),
            _label(w, h, 'CUT', top: h * 0.045),
            _label(w, h, '0:12', bottom: h * 0.04),
          ],
        );
      },
    );
  }
}

class _GrayFace extends StatelessWidget {
  const _GrayFace();

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        final h = box.maxHeight;
        return Stack(
          alignment: Alignment.topLeft,
          fit: StackFit.expand,
          children: [
            Positioned(
              left: w * 0.16,
              top: h * 0.16,
              width: w * 0.68,
              height: h * 0.68,
              child: const CustomPaint(painter: _Poly(_cardStar, _paper)),
            ),
            _label(w, h, 'MASK', top: h * 0.03),
            _label(w, h, '24', bottom: h * 0.025),
            _side(w, h, 'HOLD', left: true),
            _side(w, h, 'CUT', left: false),
          ],
        );
      },
    );
  }
}

class _Stamp extends StatelessWidget {
  const _Stamp();

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        final h = box.maxHeight;
        return Stack(
          alignment: Alignment.topLeft,
          fit: StackFit.expand,
          children: [
            const CustomPaint(painter: _Disk(_red)),
            _label(w, h, 'CUT', top: h * 0.07),
            _label(w, h, 'MASK', bottom: h * 0.05),
            _side(w, h, '24', left: true),
            _side(w, h, 'HOLD', left: false),
          ],
        );
      },
    );
  }
}

class _Dots extends StatelessWidget {
  const _Dots({required this.d});

  final double d;

  @override
  Widget build(BuildContext context) {
    return Stack(
      alignment: Alignment.topLeft,
      children: [
        for (var i = 0; i < 3; i++)
          Positioned(
            left: i == 1 ? d * 0.2 : (i == 2 ? d * 0.05 : 0),
            top: i * d * 0.68,
            width: d,
            height: d,
            child: const DecoratedBox(
              decoration: BoxDecoration(color: _red, shape: BoxShape.circle),
            ),
          ),
      ],
    );
  }
}

Widget _label(double width, double height, String text, {double? top, double? bottom}) {
  final band = height * 0.14;
  return Positioned(
    left: width * 0.05,
    right: width * 0.05,
    top: top,
    bottom: bottom,
    height: band,
    child: FittedBox(
      fit: BoxFit.scaleDown,
      child: Text(text, textDirection: TextDirection.ltr, style: _face(band * 0.7, _paper, 'Helvetica Neue')),
    ),
  );
}

Widget _side(double width, double height, String text, {required bool left}) {
  return Positioned(
    left: left ? 0 : null,
    right: left ? null : 0,
    top: height * 0.2,
    bottom: height * 0.2,
    width: width * 0.16,
    child: FittedBox(
      fit: BoxFit.scaleDown,
      child: RotatedBox(
        quarterTurns: left ? 3 : 1,
        child: Text(text, textDirection: TextDirection.ltr, style: _face(height * 0.07, _paper, 'Helvetica Neue')),
      ),
    ),
  );
}

Widget _at(double left, double top, double width, double height, double angle, Widget child) {
  return Positioned(
    left: left,
    top: top,
    width: width,
    height: height,
    child: Transform.rotate(angle: angle, child: child),
  );
}

TextStyle _face(double size, Color color, String family) {
  return TextStyle(
    inherit: false,
    decoration: TextDecoration.none,
    fontFamily: family,
    fontFamilyFallback: const <String>['Futura', 'Arial Black', 'Helvetica Neue'],
    fontSize: size,
    height: 1,
    color: color,
  );
}

class _Poly extends CustomPainter {
  const _Poly(this.points, this.color, [this.hole = const <Offset>[]]);

  final List<Offset> points;
  final Color color;
  final List<Offset> hole;

  @override
  void paint(Canvas canvas, Size size) {
    final path = Path()..fillType = PathFillType.evenOdd;
    path.addPolygon(_scaled(points, size), true);
    if (hole.isNotEmpty) path.addPolygon(_scaled(hole, size), true);
    canvas.drawPath(path, Paint()..color = color);
  }

  List<Offset> _scaled(List<Offset> src, Size size) {
    return <Offset>[
      for (final p in src) Offset(p.dx * size.width, p.dy * size.height),
    ];
  }

  @override
  bool shouldRepaint(covariant _Poly old) => false;
}

class _Disk extends CustomPainter {
  const _Disk(this.color);

  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawOval(Rect.fromLTWH(0, 0, size.width, size.height), Paint()..color = color);
  }

  @override
  bool shouldRepaint(covariant _Disk old) => false;
}

const _paper = Color(0xFFF7F5F6);
const _ink = Color(0xFF111111);
const _red = Color(0xFFEB474E);
const _teal = Color(0xFF3B867F);
const _gray = Color(0xFFC0C0C0);

const _burst = <Offset>[
  Offset(0.50, 0.00),
  Offset(0.63, 0.18),
  Offset(0.88, 0.04),
  Offset(0.74, 0.26),
  Offset(1.00, 0.34),
  Offset(0.76, 0.46),
  Offset(0.96, 0.64),
  Offset(0.68, 0.58),
  Offset(0.80, 0.90),
  Offset(0.52, 0.66),
  Offset(0.48, 1.00),
  Offset(0.36, 0.70),
  Offset(0.14, 0.94),
  Offset(0.28, 0.58),
  Offset(0.00, 0.60),
  Offset(0.24, 0.44),
  Offset(0.02, 0.26),
  Offset(0.30, 0.30),
  Offset(0.14, 0.04),
  Offset(0.40, 0.20),
];

const _chunk = <Offset>[
  Offset(0.46, 0.00),
  Offset(0.68, 0.20),
  Offset(1.00, 0.12),
  Offset(0.78, 0.40),
  Offset(0.98, 0.68),
  Offset(0.66, 0.60),
  Offset(0.76, 1.00),
  Offset(0.42, 0.70),
  Offset(0.10, 0.96),
  Offset(0.30, 0.56),
  Offset(0.00, 0.44),
  Offset(0.28, 0.32),
  Offset(0.08, 0.08),
  Offset(0.40, 0.22),
];

const _hole = <Offset>[
  Offset(0.50, 0.24),
  Offset(0.58, 0.42),
  Offset(0.76, 0.40),
  Offset(0.62, 0.54),
  Offset(0.70, 0.74),
  Offset(0.50, 0.62),
  Offset(0.30, 0.76),
  Offset(0.38, 0.54),
  Offset(0.24, 0.42),
  Offset(0.42, 0.42),
];

const _cardStar = <Offset>[
  Offset(0.50, 0.00),
  Offset(0.62, 0.30),
  Offset(0.94, 0.12),
  Offset(0.72, 0.42),
  Offset(1.00, 0.58),
  Offset(0.70, 0.64),
  Offset(0.82, 1.00),
  Offset(0.50, 0.74),
  Offset(0.18, 0.98),
  Offset(0.32, 0.62),
  Offset(0.00, 0.54),
  Offset(0.30, 0.40),
  Offset(0.08, 0.10),
  Offset(0.40, 0.30),
];

const _spark = <Offset>[
  Offset(0.50, 0.00),
  Offset(0.58, 0.26),
  Offset(0.80, 0.06),
  Offset(0.68, 0.32),
  Offset(1.00, 0.28),
  Offset(0.72, 0.48),
  Offset(0.98, 0.64),
  Offset(0.66, 0.62),
  Offset(0.84, 0.92),
  Offset(0.52, 0.70),
  Offset(0.46, 1.00),
  Offset(0.38, 0.68),
  Offset(0.14, 0.94),
  Offset(0.30, 0.58),
  Offset(0.00, 0.56),
  Offset(0.28, 0.44),
  Offset(0.04, 0.24),
  Offset(0.36, 0.32),
  Offset(0.20, 0.02),
  Offset(0.44, 0.24),
];

const _arrow = <Offset>[
  Offset(0.00, 0.26),
  Offset(0.58, 0.26),
  Offset(0.58, 0.00),
  Offset(1.00, 0.50),
  Offset(0.58, 1.00),
  Offset(0.58, 0.74),
  Offset(0.00, 0.74),
];
