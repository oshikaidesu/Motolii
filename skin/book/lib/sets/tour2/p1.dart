import 'package:flutter/widgets.dart';

class TourP1 extends StatefulWidget {
  const TourP1({super.key});

  @override
  State<TourP1> createState() => _TourP1State();
}

class _TourP1State extends State<TourP1> {
  int _front = 3;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return ClipRect(
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () => setState(() => _front = (_front + 1) % 4),
            child: CustomPaint(
              painter: _FieldPoster(_front),
              child: const SizedBox.expand(),
            ),
          ),
        );
      },
    );
  }
}

class _FieldPoster extends CustomPainter {
  const _FieldPoster(this.front);

  final int front;

  static const _bg = Color(0xFFE3E3E3);
  static const _ink = Color(0xFF000000);
  static const _orange = Color(0xFFFF4D00);
  static const _yellow = Color(0xFFFFD000);
  static const _blue = Color(0xFF3A74FF);
  static const _green = Color(0xFF8BE04C);
  static const _mint = Color(0xFF5AF5D0);
  static const _paper = Color(0xFFF7F4EE);

  static const _blueEdge = <Offset>[
    Offset(548, 743),
    Offset(556, 718),
    Offset(564, 708),
    Offset(572, 699),
    Offset(580, 692),
    Offset(588, 686),
    Offset(596, 680),
    Offset(604, 675),
    Offset(612, 671),
    Offset(620, 666),
    Offset(628, 663),
    Offset(636, 660),
    Offset(644, 657),
    Offset(656, 656),
    Offset(668, 658),
    Offset(676, 660),
    Offset(684, 663),
    Offset(692, 667),
    Offset(700, 672),
    Offset(708, 678),
    Offset(716, 700),
    Offset(728, 760),
    Offset(710, 820),
    Offset(684, 813),
    Offset(668, 819),
    Offset(652, 824),
    Offset(636, 825),
    Offset(620, 821),
    Offset(604, 813),
    Offset(588, 802),
    Offset(572, 787),
    Offset(556, 766),
  ];

  static const _greenEdge = <Offset>[
    Offset(708, 780),
    Offset(716, 742),
    Offset(724, 736),
    Offset(732, 732),
    Offset(740, 728),
    Offset(748, 678),
    Offset(756, 666),
    Offset(764, 659),
    Offset(772, 654),
    Offset(780, 651),
    Offset(788, 649),
    Offset(804, 649),
    Offset(820, 654),
    Offset(828, 660),
    Offset(836, 667),
    Offset(844, 679),
    Offset(850, 740),
    Offset(856, 800),
    Offset(860, 828),
    Offset(852, 831),
    Offset(844, 832),
    Offset(828, 831),
    Offset(812, 825),
    Offset(796, 816),
    Offset(780, 825),
    Offset(756, 832),
    Offset(740, 831),
    Offset(732, 828),
    Offset(724, 824),
    Offset(716, 817),
    Offset(708, 807),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) {
      return;
    }
    canvas.clipRect(Offset.zero & size);
    canvas.save();
    canvas.scale(size.width / 1440, size.height / 900);
    canvas.drawRect(const Rect.fromLTWH(0, 0, 1440, 900), Paint()..color = _bg);
    _letters(canvas);
    _pill(canvas);
    final order = <int>[0, 1, 2, 3]..remove(front);
    order.add(front);
    for (final i in order) {
        switch (i) {
          case 0:
            _clover(canvas);
            break;
          case 1:
            _seal(canvas, _poly(_blueEdge), _blue);
            break;
          case 2:
            _seal(canvas, _poly(_greenEdge), _green);
            break;
          default:
            _checker(canvas);
        }
    }
    canvas.restore();
  }

  void _letters(Canvas canvas) {
    final ink = Paint()..color = _ink;
    void bar(double l, double t, double r, double b) {
      canvas.drawRect(Rect.fromLTRB(l, t, r, b), ink);
    }

    bar(34, 0, 281, 56);
    bar(34, 0, 133, 548);
    bar(34, 185, 259, 331);
    bar(334, 0, 434, 548);
    bar(501, 0, 769, 56);
    bar(501, 0, 600, 548);
    bar(501, 172, 757, 320);
    bar(501, 464, 774, 548);
    bar(823, 0, 922, 548);
    bar(823, 450, 1079, 548);

    final outer = Path()
      ..moveTo(1122, 0)
      ..lineTo(1368, 0)
      ..cubicTo(1404, 0, 1419, 90, 1419, 180)
      ..lineTo(1419, 370)
      ..cubicTo(1419, 480, 1396, 548, 1336, 548)
      ..lineTo(1122, 548)
      ..close();
    final inner = Path()
      ..moveTo(1222, 68)
      ..lineTo(1272, 68)
      ..cubicTo(1318, 68, 1318, 130, 1318, 250)
      ..cubicTo(1318, 390, 1318, 462, 1272, 462)
      ..lineTo(1222, 462)
      ..close();
    canvas.drawPath(Path.combine(PathOperation.difference, outer, inner), ink);
  }

  void _pill(Canvas canvas) {
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        const Rect.fromLTRB(650, 252, 789, 296),
        const Radius.circular(22),
      ),
      Paint()..color = _mint,
    );
    final text = TextPainter(
      text: const TextSpan(
        text: '0:12 24',
        style: TextStyle(
          fontFamily: 'Helvetica Neue',
          fontSize: 17,
          height: 1,
          fontWeight: FontWeight.w700,
          color: _ink,
          decoration: TextDecoration.none,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    text.paint(canvas, Offset(719.5 - text.width / 2, 274 - text.height / 2));
  }

  void _clover(Canvas canvas) {
    const center = Offset(491, 747);
    const lobes = <Offset>[
      Offset(0, -46),
      Offset(0, 46),
      Offset(-46, 0),
      Offset(46, 0),
    ];
    final paint = Paint()..color = _orange;
    for (final lobe in lobes) {
      canvas.drawCircle(center + lobe, 76, paint);
    }
    paint.color = _yellow;
    for (final lobe in lobes) {
      canvas.drawCircle(center + lobe, 54, paint);
    }
  }

  void _seal(Canvas canvas, Path path, Color fill) {
    canvas.drawPath(
      path,
      Paint()
        ..color = _orange
        ..style = PaintingStyle.stroke
        ..strokeWidth = 44
        ..strokeJoin = StrokeJoin.round,
    );
    canvas.drawPath(path, Paint()..color = fill);
  }

  void _checker(Canvas canvas) {
    final inner = RRect.fromRectAndRadius(
      Rect.fromCircle(center: const Offset(948, 748), radius: 88),
      const Radius.circular(88),
    );
    canvas.drawRRect(
      inner.inflate(22),
      Paint()..color = _orange,
    );
    canvas.save();
    canvas.clipRRect(inner);
    const cell = 48.0;
    final bounds = inner.outerRect;
    var row = 0;
    for (var y = bounds.top; y < bounds.bottom; y += cell) {
      var col = 0;
      for (var x = bounds.left; x < bounds.right; x += cell) {
        canvas.drawRect(
          Rect.fromLTWH(x, y, cell, cell),
          Paint()..color = (row + col).isEven ? _paper : _ink,
        );
        col++;
      }
      row++;
    }
    canvas.restore();
  }

  Path _poly(List<Offset> pts) {
    final path = Path()..moveTo(pts.first.dx, pts.first.dy);
    for (final p in pts.skip(1)) {
      path.lineTo(p.dx, p.dy);
    }
    path.close();
    return path;
  }

  @override
  bool shouldRepaint(_FieldPoster oldDelegate) => oldDelegate.front != front;
}
