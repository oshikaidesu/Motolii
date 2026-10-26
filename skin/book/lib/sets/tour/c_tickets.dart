import 'package:flutter/widgets.dart';

class TourTickets extends StatefulWidget {
  const TourTickets({super.key});

  @override
  State<TourTickets> createState() => _TourTicketsState();
}

class _TourTicketsState extends State<TourTickets> {
  static const _bands = <_Spec>[
    _Spec(
      fill: Color(0xFF1E4BFF),
      ink: Color(0xFFF7F7F2),
      word: 'CUT',
      when: '10:30',
      degrees: 18,
      tipX: 0.02,
      tipY: 0.30,
      len: 1.28,
      thick: 0.150,
    ),
    _Spec(
      fill: Color(0xFF1BBE3C),
      ink: Color(0xFFF7F7F2),
      word: 'HOLD',
      when: '18:30',
      degrees: 24,
      tipX: 0.08,
      tipY: 0.06,
      len: 1.22,
      thick: 0.158,
    ),
    _Spec(
      fill: Color(0xFF121212),
      ink: Color(0xFFB6FF3C),
      word: 'MASK',
      when: '24',
      degrees: 31,
      tipX: -0.04,
      tipY: 0.40,
      len: 1.16,
      thick: 0.144,
    ),
    _Spec(
      fill: Color(0xFF1E4BFF),
      ink: Color(0xFFF7F7F2),
      word: 'ARC',
      when: '10:30',
      degrees: 36,
      tipX: 0.16,
      tipY: 0.02,
      len: 1.18,
      thick: 0.136,
    ),
    _Spec(
      fill: Color(0xFF1BBE3C),
      ink: Color(0xFFF7F7F2),
      word: 'CUT',
      when: '24',
      degrees: 40,
      tipX: -0.06,
      tipY: 0.14,
      len: 1.34,
      thick: 0.152,
    ),
  ];

  late List<int> _order = List<int>.generate(_bands.length, (i) => i);

  void _lift(int index) {
    setState(() {
      _order = [
        for (final i in _order)
          if (i != index) i,
        index,
      ];
    });
  }

  @override
  Widget build(BuildContext context) {
    return ClipRect(
      child: ColoredBox(
        color: const Color(0xFFF4F1EA),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final w = constraints.maxWidth;
            final h = constraints.maxHeight;
            if (w <= 0 || h <= 0 || !w.isFinite || !h.isFinite) {
              return const SizedBox.shrink();
            }
            final span = w > h ? w : h;
            final short = w < h ? w : h;
            return SizedBox(
              width: w,
              height: h,
              child: Stack(
                alignment: Alignment.topLeft,
                clipBehavior: Clip.hardEdge,
                children: [
                  for (final i in _order)
                    _Band(
                      key: ValueKey(i),
                      spec: _bands[i],
                      canvas: Size(w, h),
                      span: span,
                      short: short,
                      onTap: () => _lift(i),
                    ),
                ],
              ),
            );
          },
        ),
      ),
    );
  }
}

class _Band extends StatelessWidget {
  const _Band({
    required this.spec,
    required this.canvas,
    required this.span,
    required this.short,
    required this.onTap,
    super.key,
  });

  final _Spec spec;
  final Size canvas;
  final double span;
  final double short;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final len = spec.len * span;
    final thick = spec.thick * short;
    final rad = spec.degrees * _deg;
    final cosT = _cos(rad);
    final sinT = _sin(rad);
    final cx = spec.tipX * canvas.width + len * 0.5 * cosT;
    final cy = spec.tipY * canvas.height + len * 0.5 * sinT;
    return Positioned(
      left: cx - len * 0.5,
      top: cy - thick * 0.5,
      width: len,
      height: thick,
      child: Transform.rotate(
        angle: rad,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: onTap,
          child: CustomPaint(painter: _BandPainter(spec)),
        ),
      ),
    );
  }
}

class _BandPainter extends CustomPainter {
  const _BandPainter(this.spec);

  final _Spec spec;

  @override
  void paint(Canvas canvas, Size size) {
    final path = _ribbon(size);
    canvas.drawPath(path, Paint()..color = spec.fill);
    canvas.save();
    canvas.clipPath(path);
    _arrows(canvas, size, spec.ink);
    _pair(canvas, size, spec);
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _BandPainter oldDelegate) => false;
}

class _Spec {
  const _Spec({
    required this.fill,
    required this.ink,
    required this.word,
    required this.when,
    required this.degrees,
    required this.tipX,
    required this.tipY,
    required this.len,
    required this.thick,
  });

  final Color fill;
  final Color ink;
  final String word;
  final String when;
  final double degrees;
  final double tipX;
  final double tipY;
  final double len;
  final double thick;
}

const _deg = 0.017453292519943295;

double _cos(double r) {
  final x2 = r * r;
  return 1 - x2 * (1 / 2 - x2 * (1 / 24 - x2 / 720));
}

double _sin(double r) {
  final x2 = r * r;
  return r * (1 - x2 * (1 / 6 - x2 * (1 / 120 - x2 / 5040)));
}

Path _ribbon(Size s) {
  final w = s.width;
  final h = s.height;
  final cap = w * 0.16;
  final bite = h * 0.96 < cap ? h * 0.96 : cap;
  return Path()
    ..moveTo(bite, 0)
    ..lineTo(w - bite, 0)
    ..lineTo(w, h * 0.5)
    ..lineTo(w - bite, h)
    ..lineTo(bite, h)
    ..lineTo(0, h * 0.5)
    ..close();
}

void _arrows(Canvas canvas, Size size, Color color) {
  final h = size.height;
  final paint = Paint()
    ..color = color
    ..style = PaintingStyle.stroke
    ..strokeWidth = h * 0.062
    ..strokeJoin = StrokeJoin.miter
    ..strokeMiterLimit = 2;
  final mid = h * 0.5;
  for (var n = 0; n < 2; n++) {
    final spine = h * (1.85 + n * 0.95);
    for (var i = 0; i < 3; i++) {
      final spanY = h * (0.34 - i * 0.09);
      final spanX = h * (0.58 - i * 0.13);
      canvas.drawPath(
        Path()
          ..moveTo(spine, mid - spanY)
          ..lineTo(spine + spanX, mid)
          ..lineTo(spine, mid + spanY),
        paint,
      );
    }
  }
}

TextPainter _lay(
  String text,
  Color color,
  String family,
  double fontSize,
  double tracking,
  FontWeight weight,
) {
  return TextPainter(
    text: TextSpan(
      text: text,
      style: TextStyle(
        decoration: TextDecoration.none,
        color: color,
        fontSize: fontSize,
        height: 1,
        fontFamily: family,
        fontFamilyFallback: const ['Helvetica Neue', 'Arial Black'],
        fontWeight: weight,
        letterSpacing: tracking,
      ),
    ),
    textDirection: TextDirection.ltr,
    maxLines: 1,
  )..layout();
}

void _pair(Canvas canvas, Size size, _Spec spec) {
  final h = size.height;
  final word = _lay(
    spec.word,
    spec.ink,
    'Arial Black',
    h * 0.40,
    0,
    FontWeight.w900,
  );
  final date = _lay(
    spec.when,
    spec.ink,
    'Helvetica Neue',
    h * 0.19,
    h * 0.012,
    FontWeight.w700,
  );
  final gap = h * 0.42;
  final pair = word.width + gap + date.width;
  var left = size.width * 0.44 - word.width * 0.5;
  final minLeft = h * 4.05;
  if (left < minLeft) left = minLeft;
  final maxLeft = size.width - h - pair;
  if (left > maxLeft && maxLeft > minLeft) left = maxLeft;
  word.paint(canvas, Offset(left, (h - word.height) * 0.5));
  date.paint(
    canvas,
    Offset(left + word.width + gap, (h - date.height) * 0.5),
  );
  word.dispose();
  date.dispose();
}
