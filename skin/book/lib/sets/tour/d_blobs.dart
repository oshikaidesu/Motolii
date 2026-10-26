import 'package:flutter/widgets.dart';

class TourBlobs extends StatefulWidget {
  const TourBlobs({super.key});

  @override
  State<TourBlobs> createState() => _TourBlobsState();
}

class _TourBlobsState extends State<TourBlobs> {
  static const _colors = <Color>[
    Color(0xFFFFD84A),
    Color(0xFFCDB6FF),
    Color(0xFF4C7CFF),
    Color(0xFF3DCF84),
    Color(0xFFFF6F5E),
    Color(0xFF7348F5),
    Color(0xFFFF7EB6),
  ];

  static const _seeds = <_Seed>[
    _Seed(0, 0, x: -0.04, y: -0.10, w: 0.50, h: 0.18, yShort: true, wide: true),
    _Seed(1, 2, x: 0.34, y: -0.13, w: 0.24, h: 0.24, yShort: true),
    _Seed(
      0,
      1,
      x: -0.05,
      y: -0.08,
      w: 0.44,
      h: 0.16,
      fromRight: true,
      yShort: true,
      wide: true,
    ),
    _Seed(
      2,
      5,
      x: -0.11,
      y: -0.06,
      w: 0.32,
      h: 0.32,
      fromRight: true,
      yShort: true,
    ),
    _Seed(
      0,
      3,
      x: -0.16,
      y: 0.28,
      w: 0.26,
      h: 0.32,
      fromRight: true,
      tall: true,
    ),
    _Seed(1, 4, x: -0.14, y: 0.70, w: 0.24, h: 0.24, fromRight: true),
    _Seed(
      1,
      0,
      x: -0.12,
      y: -0.10,
      w: 0.28,
      h: 0.28,
      xShort: true,
      fromBottom: true,
    ),
    _Seed(4, 3, x: 0.12, y: -0.03, w: 0.42, h: 0.21, fromBottom: true),
    _Seed(
      0,
      6,
      x: 0.50,
      y: -0.08,
      w: 0.58,
      h: 0.16,
      fromBottom: true,
      wide: true,
    ),
    _Seed(0, 2, x: -0.13, y: 0.12, w: 0.26, h: 0.30, xShort: true, tall: true),
    _Seed(3, 4, x: -0.08, y: 0.44, w: 0.26, h: 0.20, xShort: true),
    _Seed(1, 6, x: -0.11, y: 0.64, w: 0.24, h: 0.24, xShort: true),
    _Seed(
      1,
      3,
      x: -0.08,
      y: -0.08,
      w: 0.18,
      h: 0.18,
      xShort: true,
      yShort: true,
    ),
    _Seed(
      1,
      1,
      x: -0.09,
      y: -0.07,
      w: 0.24,
      h: 0.24,
      fromRight: true,
      fromBottom: true,
    ),
    _Seed(1, 6, x: 0.26, y: -0.025, w: 0.055, h: 0.055, yShort: true),
    _Seed(1, 2, x: -0.02, y: 0.20, w: 0.05, h: 0.05, fromRight: true),
    _Seed(1, 1, x: -0.025, y: 0.56, w: 0.06, h: 0.06, fromRight: true),
    _Seed(1, 0, x: 0.78, y: -0.02, w: 0.07, h: 0.07, fromBottom: true),
    _Seed(
      1,
      5,
      x: 0.03,
      y: -0.03,
      w: 0.05,
      h: 0.05,
      xShort: true,
      fromBottom: true,
    ),
    _Seed(1, 4, x: -0.02, y: 0.34, w: 0.048, h: 0.048, xShort: true),
    _Seed(1, 5, x: 0.58, y: -0.03, w: 0.042, h: 0.042, yShort: true),
    _Seed(1, 2, x: -0.04, y: 0.78, w: 0.055, h: 0.055, fromRight: true),
  ];

  final List<int> _at = [for (final seed in _seeds) seed.color];

  void _cycle(int index) {
    setState(() {
      _at[index] = (_at[index] + 1) % _colors.length;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: SizedBox.expand(
        child: ColoredBox(
          color: const Color(0xFFFFFFFF),
          child: ClipRect(
            child: LayoutBuilder(
              builder: (context, constraints) {
                final size = Size(constraints.maxWidth, constraints.maxHeight);
                return Stack(
                  clipBehavior: Clip.hardEdge,
                  fit: StackFit.expand,
                  children: [
                    for (var i = 0; i < _seeds.length; i++)
                      Positioned.fromRect(
                        rect: _seeds[i].rect(size),
                        child: _Tap(
                          kind: _seeds[i].kind,
                          color: _colors[_at[i]],
                          onTap: () => _cycle(i),
                        ),
                      ),
                    Positioned(
                      left: size.width * 0.22,
                      right: size.width * 0.22,
                      top: size.height * 0.36,
                      bottom: size.height * 0.36,
                      child: const FittedBox(
                        fit: BoxFit.contain,
                        child: Text(
                          'CUT',
                          softWrap: false,
                          style: TextStyle(
                            fontFamily: 'Helvetica Neue',
                            fontFamilyFallback: <String>[
                              'Arial Black',
                              'Arial',
                            ],
                            fontWeight: FontWeight.w900,
                            fontSize: 200,
                            height: 1,
                            letterSpacing: -6,
                            color: Color(0xFF000000),
                            decoration: TextDecoration.none,
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

class _Tap extends StatelessWidget {
  const _Tap({required this.kind, required this.color, required this.onTap});

  final int kind;
  final Color color;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final hit = GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: ColoredBox(color: color),
    );
    if (kind == 0 || kind == 1) {
      return LayoutBuilder(
        builder: (context, constraints) {
          final short = constraints.maxWidth < constraints.maxHeight
              ? constraints.maxWidth
              : constraints.maxHeight;
          return ClipRRect(
            borderRadius: BorderRadius.circular(short / 2),
            child: hit,
          );
        },
      );
    }
    return ClipPath(clipper: _Outline(kind), child: hit);
  }
}

class _Seed {
  const _Seed(
    this.kind,
    this.color, {
    required this.x,
    required this.y,
    required this.w,
    required this.h,
    this.fromRight = false,
    this.fromBottom = false,
    this.xShort = false,
    this.yShort = false,
    this.wide = false,
    this.tall = false,
  });

  final int kind;
  final int color;
  final double x;
  final double y;
  final double w;
  final double h;
  final bool fromRight;
  final bool fromBottom;
  final bool xShort;
  final bool yShort;
  final bool wide;
  final bool tall;

  Rect rect(Size size) {
    final s = size.shortestSide;
    final width = wide ? w * size.width : w * s;
    final height = tall ? h * size.height : h * s;
    final left = fromRight
        ? size.width - width - x * s
        : xShort
        ? x * s
        : x * size.width;
    final top = fromBottom
        ? size.height - height - y * s
        : yShort
        ? y * s
        : y * size.height;
    return Rect.fromLTWH(left, top, width, height);
  }
}

class _Outline extends CustomClipper<Path> {
  const _Outline(this.kind);

  final int kind;

  @override
  Path getClip(Size size) => _outline(kind, size);

  @override
  bool shouldReclip(_Outline oldClipper) => oldClipper.kind != kind;
}

Path _outline(int kind, Size size) {
  if (size.isEmpty) return Path();
  if (kind == 2) {
    return _rounded(<Offset>[
      Offset(size.width / 2, 0),
      Offset(size.width, size.height / 2),
      Offset(size.width / 2, size.height),
      Offset(0, size.height / 2),
    ], size.shortestSide * 0.24);
  }
  if (kind == 3) {
    return _rounded(
      <Offset>[
        Offset(0, size.height * 0.08),
        Offset(size.width, size.height / 2),
        Offset(0, size.height * 0.92),
      ],
      size.shortestSide * 0.42,
      pull: 0.22,
    );
  }
  final circle = Path()
    ..addOval(Rect.fromLTWH(0, 0, size.width, size.height * 2));
  final mask = Path()
    ..addRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(0, 0, size.width, size.height),
        Radius.circular(size.shortestSide * 0.16),
      ),
    );
  return Path.combine(PathOperation.intersect, circle, mask);
}

Path _rounded(List<Offset> points, double radius, {double pull = 1}) {
  final path = Path();
  final count = points.length;
  for (var i = 0; i < count; i++) {
    final prev = points[(i - 1 + count) % count];
    final curr = points[i];
    final next = points[(i + 1) % count];
    final toPrev = prev - curr;
    final toNext = next - curr;
    final prevLen = toPrev.distance;
    final nextLen = toNext.distance;
    if (prevLen == 0 || nextLen == 0) {
      if (i == 0) {
        path.moveTo(curr.dx, curr.dy);
      } else {
        path.lineTo(curr.dx, curr.dy);
      }
      continue;
    }
    var r = radius;
    final limit = prevLen < nextLen ? prevLen : nextLen;
    if (r > limit * 0.45) r = limit * 0.45;
    final start = curr + (toPrev / prevLen) * r;
    final end = curr + (toNext / nextLen) * r;
    if (i == 0) {
      path.moveTo(start.dx, start.dy);
    } else {
      path.lineTo(start.dx, start.dy);
    }
    final mid = Offset((start.dx + end.dx) / 2, (start.dy + end.dy) / 2);
    final control = Offset(
      mid.dx + (curr.dx - mid.dx) * pull,
      mid.dy + (curr.dy - mid.dy) * pull,
    );
    path.quadraticBezierTo(control.dx, control.dy, end.dx, end.dy);
  }
  path.close();
  return path;
}
