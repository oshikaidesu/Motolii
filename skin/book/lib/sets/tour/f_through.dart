import 'package:flutter/widgets.dart';

class TourThrough extends StatefulWidget {
  const TourThrough({super.key});

  @override
  State<TourThrough> createState() => _TourThroughState();
}

enum _Kind { clover, blob, cross, ring, flower, hex }

class _Piece {
  const _Piece(
    this.cx,
    this.cy,
    this.s,
    this.turn,
    this.fill,
    this.ink,
    this.kind,
  );

  final double cx;
  final double cy;
  final double s;
  final double turn;
  final Color fill;
  final Color ink;
  final _Kind kind;
}

const _lavender = Color(0xFFCDB8EC);
const _blue = Color(0xFF2A4ECA);
const _darkRed = Color(0xFF7A2030);
const _green = Color(0xFF1C7A3C);
const _black = Color(0xFF141414);
const _lime = Color(0xFFC6EE3C);
const _red = Color(0xFFE1332C);
const _ink = Color(0xFF101010);

const _pieces = <_Piece>[
  _Piece(0.64, 0.18, 0.28, -0.04, _blue, _ink, _Kind.clover),
  _Piece(0.78, 0.13, 0.22, 0.055, _darkRed, _lime, _Kind.blob),
  _Piece(0.82, 0.30, 0.24, 0, _green, _ink, _Kind.cross),
  _Piece(0.68, 0.32, 0.16, -0.03, _black, _lime, _Kind.ring),
  _Piece(0.89, 0.16, 0.12, 0.07, _lime, _ink, _Kind.flower),
  _Piece(0.74, 0.42, 0.15, 0.025, _red, _ink, _Kind.hex),
];

class _TourThroughState extends State<TourThrough> {
  final List<int> _order = List<int>.generate(_pieces.length, (i) => i);

  void _front(int id) {
    setState(() {
      _order.remove(id);
      _order.add(id);
    });
  }

  static const _tight = TextHeightBehavior(
    applyHeightToFirstAscent: false,
    applyHeightToLastDescent: false,
  );

  TextStyle _sentence(double size) {
    return TextStyle(
      inherit: false,
      decoration: TextDecoration.none,
      fontFamily: 'Helvetica Neue',
      fontFamilyFallback: const ['Arial Black', 'Arial'],
      fontWeight: FontWeight.w900,
      fontSize: size,
      height: 0.78,
      letterSpacing: size * -0.045,
      color: const Color(0xFF000000),
    );
  }

  double _sentenceSize(double maxWidth, double maxHeight) {
    final probe = TextPainter(
      text: TextSpan(text: 'Hold\nstill.', style: _sentence(100)),
      textDirection: TextDirection.ltr,
      textHeightBehavior: _tight,
    )..layout();
    final sx = probe.width > 0 ? maxWidth / probe.width : 1.0;
    final sy = probe.height > 0 ? maxHeight / probe.height : 1.0;
    probe.dispose();
    return 100 * (sx < sy ? sx : sy);
  }

  Widget _tile(int id, double w, double h) {
    final piece = _pieces[id];
    final unit = w < h ? w : h;
    var side = unit * piece.s;
    final fitW = w * 0.42;
    final fitH = h * 0.40;
    if (side > fitW) side = fitW;
    if (side > fitH) side = fitH;
    final spill = side * piece.turn.abs() * 0.52;
    var left = w * piece.cx - side / 2;
    var top = h * piece.cy - side / 2;
    final minLeft = spill;
    final maxLeft = w - side - spill;
    final minTop = spill;
    final maxTop = h * 0.56 - side - spill;
    if (maxLeft <= minLeft) {
      left = minLeft;
    } else if (left < minLeft) {
      left = minLeft;
    } else if (left > maxLeft) {
      left = maxLeft;
    }
    if (maxTop <= minTop) {
      top = minTop;
    } else if (top < minTop) {
      top = minTop;
    } else if (top > maxTop) {
      top = maxTop;
    }
    return Positioned(
      key: ValueKey<int>(id),
      left: left,
      top: top,
      width: side,
      height: side,
      child: Transform.rotate(
        angle: piece.turn,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: () => _front(id),
          child: ColoredBox(
            color: piece.fill,
            child: CustomPaint(
              painter: _Mark(piece.kind, piece.ink),
              child: const SizedBox.expand(),
            ),
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox.expand(
      child: ClipRect(
        child: ColoredBox(
          color: _lavender,
          child: LayoutBuilder(
            builder: (context, constraints) {
              final w = constraints.maxWidth;
              final h = constraints.maxHeight;
              final size = _sentenceSize(w * 0.56, h * 0.32);
              return Stack(
                fit: StackFit.expand,
                clipBehavior: Clip.hardEdge,
                children: [
                  for (final id in _order) _tile(id, w, h),
                  Positioned(
                    left: w * 0.04,
                    bottom: h * 0.045,
                    child: Text(
                      'Hold\nstill.',
                      textHeightBehavior: _tight,
                      style: _sentence(size),
                    ),
                  ),
                ],
              );
            },
          ),
        ),
      ),
    );
  }
}

class _Mark extends CustomPainter {
  const _Mark(this.kind, this.ink);

  final _Kind kind;
  final Color ink;

  @override
  void paint(Canvas canvas, Size size) {
    final fill = Paint()..color = ink;
    switch (kind) {
      case _Kind.clover:
        _clover(canvas, size, fill);
      case _Kind.blob:
        _blob(canvas, size, fill);
      case _Kind.cross:
        _cross(canvas, size, ink);
      case _Kind.ring:
        canvas.drawCircle(
          Offset(size.width / 2, size.height / 2),
          size.shortestSide * 0.15,
          fill,
        );
      case _Kind.flower:
        _flower(canvas, size, fill);
      case _Kind.hex:
        _hex(canvas, size, fill);
    }
  }

  void _clover(Canvas canvas, Size size, Paint paint) {
    final c = Offset(size.width / 2, size.height / 2);
    final r = size.shortestSide * 0.19;
    final d = r * 0.76;
    canvas.drawCircle(c + const Offset(0, -1) * d, r, paint);
    canvas.drawCircle(c + const Offset(0, 1) * d, r, paint);
    canvas.drawCircle(c + const Offset(-1, 0) * d, r, paint);
    canvas.drawCircle(c + const Offset(1, 0) * d, r, paint);
  }

  void _blob(Canvas canvas, Size size, Paint paint) {
    final w = size.width;
    final h = size.height;
    final path = Path()
      ..moveTo(w * 0.34, h * 0.30)
      ..cubicTo(w * 0.50, h * 0.16, w * 0.70, h * 0.18, w * 0.74, h * 0.36)
      ..cubicTo(w * 0.80, h * 0.52, w * 0.74, h * 0.70, w * 0.58, h * 0.76)
      ..cubicTo(w * 0.40, h * 0.84, w * 0.22, h * 0.70, w * 0.22, h * 0.52)
      ..cubicTo(w * 0.20, h * 0.36, w * 0.24, h * 0.34, w * 0.34, h * 0.30)
      ..close();
    canvas.drawPath(path, paint);
  }

  void _cross(Canvas canvas, Size size, Color color) {
    final s = size.shortestSide;
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = s * 0.085
      ..strokeCap = StrokeCap.square;
    final m = s * 0.28;
    canvas.drawLine(
      Offset(m, m),
      Offset(size.width - m, size.height - m),
      paint,
    );
    canvas.drawLine(
      Offset(size.width - m, m),
      Offset(m, size.height - m),
      paint,
    );
  }

  void _flower(Canvas canvas, Size size, Paint paint) {
    final c = Offset(size.width / 2, size.height / 2);
    final petal = size.shortestSide * 0.048;
    final reach = size.shortestSide * 0.09;
    const arms = <Offset>[
      Offset(0, -1),
      Offset(0.866, -0.5),
      Offset(0.866, 0.5),
      Offset(0, 1),
      Offset(-0.866, 0.5),
      Offset(-0.866, -0.5),
    ];
    for (final arm in arms) {
      canvas.drawCircle(c + arm * reach, petal, paint);
    }
    canvas.drawCircle(c, petal * 0.7, paint);
  }

  void _hex(Canvas canvas, Size size, Paint paint) {
    final c = Offset(size.width / 2, size.height / 2);
    final r = size.shortestSide * 0.17;
    const pts = <Offset>[
      Offset(1, 0),
      Offset(0.5, 0.866),
      Offset(-0.5, 0.866),
      Offset(-1, 0),
      Offset(-0.5, -0.866),
      Offset(0.5, -0.866),
    ];
    final path = Path();
    for (var i = 0; i < pts.length; i++) {
      final o = c + pts[i] * r;
      if (i == 0) {
        path.moveTo(o.dx, o.dy);
      } else {
        path.lineTo(o.dx, o.dy);
      }
    }
    path.close();
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(_Mark old) => old.kind != kind || old.ink != ink;
}
