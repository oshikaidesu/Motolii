import 'package:flutter/widgets.dart';

class TourP4 extends StatefulWidget {
  const TourP4({super.key});

  @override
  State<TourP4> createState() => _TourP4State();
}

class _TourP4State extends State<TourP4> {
  bool _blueFront = false;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.hasBoundedWidth ? box.maxWidth : 1440.0;
        final h = box.hasBoundedHeight ? box.maxHeight : 900.0;
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: () => setState(() => _blueFront = !_blueFront),
          child: ClipRect(
            child: CustomPaint(
              size: Size(w, h),
              painter: _FieldSheet(blueFront: _blueFront),
            ),
          ),
        );
      },
    );
  }
}

class _FieldSheet extends CustomPainter {
  const _FieldSheet({required this.blueFront});

  final bool blueFront;

  static const _paper = Color(0xFFE3E3E3);
  static const _ink = Color(0xFF000000);
  static const _orange = Color(0xFFFF4D00);
  static const _yellow = Color(0xFFFFD000);
  static const _blue = Color(0xFF3A74FF);
  static const _green = Color(0xFF8BE04C);
  static const _mint = Color(0xFF5AF5D0);
  static const _cream = Color(0xFFF7F4EE);
  static const _check = Color(0xFF141414);

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = _paper);
    canvas.save();
    canvas.scale(size.width / 1440, size.height / 900);
    _letters(canvas);
    _pill(canvas);
    _clover(canvas);
    if (!blueFront) _drop(canvas);
    _heart(canvas);
    _disc(canvas);
    if (blueFront) _drop(canvas);
    canvas.restore();
  }

  void _letters(Canvas canvas) {
    final ink = Paint()..color = _ink;
    canvas.drawRect(const Rect.fromLTRB(34, 0, 134, 549), ink);
    canvas.drawRect(const Rect.fromLTRB(34, 0, 282, 57), ink);
    canvas.drawRect(const Rect.fromLTRB(34, 185, 260, 331), ink);
    canvas.drawRect(const Rect.fromLTRB(334, 0, 435, 549), ink);
    canvas.drawRect(const Rect.fromLTRB(501, 0, 601, 549), ink);
    canvas.drawRect(const Rect.fromLTRB(501, 0, 770, 56), ink);
    canvas.drawRect(const Rect.fromLTRB(501, 172, 758, 320), ink);
    canvas.drawRect(const Rect.fromLTRB(501, 464, 775, 549), ink);
    canvas.drawRect(const Rect.fromLTRB(823, 0, 923, 549), ink);
    canvas.drawRect(const Rect.fromLTRB(823, 450, 1080, 549), ink);
    canvas.drawPath(_dee(), ink);
  }

  Path _dee() {
    final p = Path()..fillType = PathFillType.evenOdd;
    p.moveTo(1122, 0);
    p.lineTo(1384, 0);
    for (final o in _deeEdge) {
      p.lineTo(o.dx, o.dy);
    }
    p.lineTo(1122, 548);
    p.close();
    p.moveTo(1222, 66);
    p.lineTo(1222, 462);
    for (final o in _deeHole) {
      p.lineTo(o.dx, o.dy);
    }
    p.close();
    return p;
  }

  void _pill(Canvas canvas) {
    const pill = Rect.fromLTWH(650, 252, 139, 44);
    canvas.drawRRect(
      RRect.fromRectAndRadius(pill, const Radius.circular(22)),
      Paint()..color = _mint,
    );
    final type = TextPainter(
      text: const TextSpan(
        text: '0:12 24',
        style: TextStyle(
          fontFamily: 'Helvetica Neue',
          fontFamilyFallback: ['Arial', 'Futura'],
          fontSize: 17,
          height: 1,
          color: _ink,
          fontWeight: FontWeight.w500,
          decoration: TextDecoration.none,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    type.paint(
      canvas,
      Offset(pill.center.dx - type.width / 2, pill.center.dy - type.height / 2),
    );
    type.dispose();
  }

  void _clover(Canvas canvas) {
    const c = Offset(490, 746);
    const d = 40.0;
    const r = 60.0;
    var path = _disk(c.dx, c.dy - d, r);
    path = Path.combine(PathOperation.union, path, _disk(c.dx + d, c.dy, r));
    path = Path.combine(PathOperation.union, path, _disk(c.dx, c.dy + d, r));
    path = Path.combine(PathOperation.union, path, _disk(c.dx - d, c.dy, r));
    _rim(canvas, path, _yellow);
  }

  void _drop(Canvas canvas) {
    final path = Path()..addPolygon(_dropEdge, true);
    _rim(canvas, path, _blue);
  }

  void _heart(Canvas canvas) {
    final path = Path()..addPolygon(_heartEdge, true);
    _rim(canvas, path, _green);
  }

  void _disc(Canvas canvas) {
    final ring = _disk(948, 747, 87);
    canvas.drawPath(
      ring,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 46
        ..color = _orange,
    );
    canvas.save();
    canvas.clipPath(ring);
    for (var iy = -3; iy <= 3; iy++) {
      for (var ix = -3; ix <= 3; ix++) {
        canvas.drawRect(
          Rect.fromLTWH(948 + ix * 49.0, 748 + iy * 49.0, 49, 49),
          Paint()..color = (ix + iy).isEven ? _check : _cream,
        );
      }
    }
    canvas.restore();
  }

  Path _disk(double cx, double cy, double r) {
    return Path()..addOval(Rect.fromCircle(center: Offset(cx, cy), radius: r));
  }

  void _rim(Canvas canvas, Path path, Color fill) {
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 46
        ..strokeJoin = StrokeJoin.round
        ..color = _orange,
    );
    canvas.drawPath(path, Paint()..color = fill);
  }

  @override
  bool shouldRepaint(_FieldSheet oldDelegate) => oldDelegate.blueFront != blueFront;
}

const _deeEdge = <Offset>[
  Offset(1387, 8),
  Offset(1394, 32),
  Offset(1400, 56),
  Offset(1405, 80),
  Offset(1409, 104),
  Offset(1412, 128),
  Offset(1415, 152),
  Offset(1417, 188),
  Offset(1418, 212),
  Offset(1419, 248),
  Offset(1419, 296),
  Offset(1418, 332),
  Offset(1416, 368),
  Offset(1413, 404),
  Offset(1410, 428),
  Offset(1406, 452),
  Offset(1400, 476),
  Offset(1394, 500),
  Offset(1386, 524),
  Offset(1375, 548),
];

const _deeHole = <Offset>[
  Offset(1261, 462),
  Offset(1269, 460),
  Offset(1275, 458),
  Offset(1279, 456),
  Offset(1283, 454),
  Offset(1288, 450),
  Offset(1293, 444),
  Offset(1297, 438),
  Offset(1300, 432),
  Offset(1304, 422),
  Offset(1308, 406),
  Offset(1312, 390),
  Offset(1315, 366),
  Offset(1317, 342),
  Offset(1319, 300),
  Offset(1319, 250),
  Offset(1318, 214),
  Offset(1316, 190),
  Offset(1314, 166),
  Offset(1311, 146),
  Offset(1306, 126),
  Offset(1300, 110),
  Offset(1295, 98),
  Offset(1290, 86),
  Offset(1284, 80),
  Offset(1280, 76),
  Offset(1274, 72),
  Offset(1270, 70),
  Offset(1265, 68),
  Offset(1257, 66),
];

const _heartEdge = <Offset>[
  Offset(794, 649),
  Offset(820, 655),
  Offset(840, 672),
  Offset(846, 690),
  Offset(840, 710),
  Offset(837, 736),
  Offset(838, 770),
  Offset(841, 800),
  Offset(828, 820),
  Offset(800, 830),
  Offset(768, 832),
  Offset(740, 828),
  Offset(718, 816),
  Offset(705, 798),
  Offset(701, 776),
  Offset(706, 754),
  Offset(718, 738),
  Offset(736, 728),
  Offset(744, 712),
  Offset(743, 692),
  Offset(752, 674),
  Offset(770, 658),
];

const _dropEdge = <Offset>[
  Offset(653, 654),
  Offset(629, 662),
  Offset(616, 668),
  Offset(606, 674),
  Offset(596, 680),
  Offset(588, 686),
  Offset(580, 692),
  Offset(573, 698),
  Offset(567, 704),
  Offset(562, 710),
  Offset(557, 716),
  Offset(553, 722),
  Offset(551, 728),
  Offset(549, 734),
  Offset(547, 740),
  Offset(547, 746),
  Offset(548, 752),
  Offset(551, 758),
  Offset(555, 764),
  Offset(558, 770),
  Offset(563, 776),
  Offset(568, 782),
  Offset(573, 788),
  Offset(579, 794),
  Offset(586, 800),
  Offset(593, 806),
  Offset(602, 812),
  Offset(613, 818),
  Offset(642, 826),
  Offset(691, 818),
  Offset(702, 812),
  Offset(711, 806),
  Offset(718, 800),
  Offset(725, 794),
  Offset(731, 788),
  Offset(736, 782),
  Offset(741, 776),
  Offset(746, 770),
  Offset(749, 764),
  Offset(753, 758),
  Offset(756, 752),
  Offset(757, 746),
  Offset(757, 740),
  Offset(755, 734),
  Offset(753, 728),
  Offset(751, 722),
  Offset(747, 716),
  Offset(742, 710),
  Offset(737, 704),
  Offset(731, 698),
  Offset(724, 692),
  Offset(716, 686),
  Offset(708, 680),
  Offset(698, 674),
  Offset(688, 668),
  Offset(675, 662),
];
