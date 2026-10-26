import 'package:flutter/widgets.dart';

class TourP3 extends StatefulWidget {
  const TourP3({super.key});

  @override
  State<TourP3> createState() => _TourP3State();
}

class _TourP3State extends State<TourP3> {
  var _turned = false;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => setState(() => _turned = !_turned),
      child: LayoutBuilder(
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
              child: CustomPaint(painter: _Poster(turn: _turned)),
            ),
          );
        },
      ),
    );
  }
}

class _Poster extends CustomPainter {
  const _Poster({required this.turn});

  final bool turn;

  static const _ground = Color(0xFFE3E3E3);
  static const _ink = Color(0xFF000000);
  static const _orange = Color(0xFFFF4D00);
  static const _yellow = Color(0xFFFFD000);
  static const _blue = Color(0xFF3A74FF);
  static const _green = Color(0xFF8BE04C);
  static const _mint = Color(0xFF5AF5D0);
  static const _cream = Color(0xFFF7F4EE);
  static const _check = Color(0xFF141414);

  static const _pill = TextStyle(
    fontFamily: 'Helvetica Neue',
    fontFamilyFallback: ['Arial', 'Futura', 'Arial Black', 'Menlo'],
    fontSize: 24,
    height: 1,
    fontWeight: FontWeight.w700,
    color: _ink,
    letterSpacing: 0,
    decoration: TextDecoration.none,
  );

  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.scale(size.width / 1440, size.height / 900);
    canvas.clipRect(const Rect.fromLTWH(0, -8, 1440, 916));
    canvas.drawRect(
      const Rect.fromLTWH(0, 0, 1440, 900),
      Paint()..color = _ground,
    );
    _letters(canvas);
    _pillMark(canvas);
    _chain(canvas);
    canvas.restore();
  }

  void _letters(Canvas canvas) {
    final ink = Paint()..color = _ink;
    void bar(double l, double t, double r, double b) {
      canvas.drawRect(Rect.fromLTRB(l, t, r, b), ink);
    }

    bar(34, -6, 133, 548);
    bar(34, -6, 281, 57);
    bar(34, 185, 259, 331);
    bar(334, -6, 434, 548);
    bar(501, -6, 600, 548);
    bar(501, -6, 769, 56);
    bar(501, 172, 756, 320);
    bar(501, 464, 774, 548);
    bar(823, -6, 922, 548);
    bar(823, 450, 1079, 548);

    final d = Path()..fillType = PathFillType.evenOdd;
    d.moveTo(1122, -6);
    d.lineTo(1384, -6);
    for (final p in _dOuter) {
      d.lineTo(p.dx, p.dy);
    }
    d.lineTo(1122, 548);
    d.close();
    d.moveTo(1222, 66);
    d.lineTo(1260, 68);
    for (final p in _dInner) {
      d.lineTo(p.dx, p.dy);
    }
    d.lineTo(1222, 462);
    d.close();
    canvas.drawPath(d, ink);
  }

  void _pillMark(Canvas canvas) {
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        const Rect.fromLTRB(650, 252, 789, 296),
        const Radius.circular(22),
      ),
      Paint()..color = _mint,
    );
    final text = TextPainter(
      text: const TextSpan(text: '0:12 24', style: _pill),
      textDirection: TextDirection.ltr,
    )..layout();
    final fit = text.width > 116 ? 116 / text.width : 1.0;
    canvas.save();
    canvas.translate(719.5, 274);
    canvas.scale(fit);
    text.paint(canvas, Offset(-text.width / 2, -text.height / 2));
    canvas.restore();
  }

  void _chain(Canvas canvas) {
    _clover(canvas);
    _blob(canvas, _loop(_blueEdge), _blue);
    _blob(canvas, _loop(_greenEdge), _green);
    _disc(canvas);
  }

  void _clover(Canvas canvas) {
    canvas.save();
    canvas.translate(491, 747);
    if (turn) canvas.rotate(0.78539816339);
    canvas.translate(-491, -747);
    const lobes = <Offset>[
      Offset(491, 696.5),
      Offset(541.5, 747),
      Offset(491, 797.5),
      Offset(440.5, 747),
    ];
    final orange = Paint()..color = _orange;
    final yellow = Paint()..color = _yellow;
    for (final lobe in lobes) {
      canvas.drawCircle(lobe, 73.5, orange);
    }
    for (final lobe in lobes) {
      canvas.drawCircle(lobe, 50.5, yellow);
    }
    canvas.restore();
  }

  void _blob(Canvas canvas, Path path, Color fill) {
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

  void _disc(Canvas canvas) {
    const center = Offset(948, 747);
    canvas.drawCircle(center, 110, Paint()..color = _orange);
    canvas.save();
    canvas.clipPath(
      Path()..addOval(Rect.fromCircle(center: center, radius: 87)),
    );
    const cell = 49.0;
    for (var j = -3; j <= 3; j++) {
      for (var i = -3; i <= 3; i++) {
        canvas.drawRect(
          Rect.fromLTWH(948 + i * cell, 747 + j * cell, cell + 0.5, cell + 0.5),
          Paint()..color = (i + j).isEven ? _check : _cream,
        );
      }
    }
    canvas.restore();
  }

  Path _loop(List<Offset> points) {
    final path = Path()..moveTo(points.first.dx, points.first.dy);
    for (final p in points.skip(1)) {
      path.lineTo(p.dx, p.dy);
    }
    path.close();
    return path;
  }

  @override
  bool shouldRepaint(_Poster old) => old.turn != turn;
}

const _dOuter = <Offset>[
  Offset(1384, 0),
  Offset(1390, 16),
  Offset(1394, 32),
  Offset(1398, 48),
  Offset(1402, 64),
  Offset(1405, 80),
  Offset(1408, 96),
  Offset(1410, 112),
  Offset(1412, 128),
  Offset(1414, 144),
  Offset(1415, 160),
  Offset(1416, 176),
  Offset(1417, 192),
  Offset(1418, 208),
  Offset(1419, 224),
  Offset(1419, 272),
  Offset(1418, 320),
  Offset(1417, 352),
  Offset(1415, 384),
  Offset(1412, 416),
  Offset(1406, 448),
  Offset(1399, 480),
  Offset(1390, 512),
  Offset(1377, 544),
];

const _dInner = <Offset>[
  Offset(1271, 70),
  Offset(1291, 86),
  Offset(1300, 102),
  Offset(1307, 118),
  Offset(1311, 134),
  Offset(1315, 166),
  Offset(1318, 198),
  Offset(1320, 246),
  Offset(1320, 294),
  Offset(1318, 342),
  Offset(1315, 374),
  Offset(1309, 406),
  Offset(1298, 438),
  Offset(1284, 454),
];

const _blueEdge = <Offset>[
  Offset(646, 656),
  Offset(660, 656),
  Offset(674, 660),
  Offset(688, 665),
  Offset(701, 673),
  Offset(713, 683),
  Offset(719, 695),
  Offset(719, 709),
  Offset(708, 718),
  Offset(697, 727),
  Offset(688, 739),
  Offset(681, 753),
  Offset(677, 767),
  Offset(676, 781),
  Offset(678, 795),
  Offset(682, 809),
  Offset(671, 818),
  Offset(657, 822),
  Offset(643, 825),
  Offset(629, 823),
  Offset(615, 818),
  Offset(601, 811),
  Offset(589, 803),
  Offset(578, 793),
  Offset(568, 782),
  Offset(559, 770),
  Offset(551, 758),
  Offset(547, 744),
  Offset(548, 737),
  Offset(553, 723),
  Offset(561, 711),
  Offset(571, 700),
  Offset(583, 690),
  Offset(595, 681),
  Offset(608, 673),
  Offset(622, 666),
  Offset(636, 660),
];

const _greenEdge = <Offset>[
  Offset(787, 649),
  Offset(808, 650),
  Offset(822, 655),
  Offset(834, 665),
  Offset(843, 677),
  Offset(845, 684),
  Offset(840, 705),
  Offset(837, 733),
  Offset(837, 761),
  Offset(838, 782),
  Offset(844, 803),
  Offset(855, 822),
  Offset(860, 828),
  Offset(846, 832),
  Offset(825, 830),
  Offset(805, 820),
  Offset(791, 817),
  Offset(779, 825),
  Offset(765, 831),
  Offset(751, 832),
  Offset(737, 830),
  Offset(723, 823),
  Offset(712, 813),
  Offset(705, 800),
  Offset(701, 786),
  Offset(701, 772),
  Offset(706, 758),
  Offset(714, 745),
  Offset(725, 735),
  Offset(739, 729),
  Offset(745, 717),
  Offset(743, 703),
  Offset(745, 689),
  Offset(750, 675),
  Offset(759, 663),
  Offset(771, 655),
  Offset(785, 650),
];
