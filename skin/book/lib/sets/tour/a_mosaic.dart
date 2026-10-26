import 'package:flutter/widgets.dart';

const _field = Color(0xFF99CBFE);
const _cream = Color(0xFFE9E3CD);
const _black = Color(0xFF0C0C0C);
const _pink = Color(0xFFFEADCE);
const _orange = Color(0xFFF83A04);
const _yellow = Color(0xFFF8A018);
const _green = Color(0xFF124B38);
const _blue = Color(0xFF327AFF);
const _light = Color(0xFF99CBFE);

class TourMosaic extends StatelessWidget {
  const TourMosaic({super.key});

  @override
  Widget build(BuildContext context) {
    return const ColoredBox(
      color: _field,
      child: _Sheet(),
    );
  }
}

class _Sheet extends StatefulWidget {
  const _Sheet();

  @override
  State<_Sheet> createState() => _SheetState();
}

class _SheetState extends State<_Sheet> {
  final Set<int> _flipped = <int>{};
  int _epoch = 0;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final w = constraints.maxWidth;
        final h = constraints.maxHeight;
        if (w <= 0 || h <= 0 || !w.isFinite || !h.isFinite) {
          return const SizedBox.shrink();
        }
        final mx = w * 0.084;
        final my = h * 0.056;
        final sheetW = w - mx * 2;
        final sheetH = h - my * 2;
        if (sheetW <= 2 || sheetH <= 2) return const SizedBox.shrink();
        final tile = sheetW / 6;
        final rows = (sheetH / tile).ceil();
        return Padding(
          padding: EdgeInsets.fromLTRB(mx, my, mx, my),
          child: ClipRect(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTapUp: (details) {
                final col = (details.localPosition.dx / tile).floor();
                final row = (details.localPosition.dy / tile).floor();
                if (col < 0 || col > 5 || row < 0 || row >= rows) return;
                final id = row * 6 + col;
                setState(() {
                  if (!_flipped.remove(id)) _flipped.add(id);
                  _epoch++;
                });
              },
              child: CustomPaint(
                size: Size(sheetW, sheetH),
                painter: _SheetPainter(flipped: _flipped, epoch: _epoch),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _SheetPainter extends CustomPainter {
  _SheetPainter({required this.flipped, required this.epoch});

  final Set<int> flipped;
  final int epoch;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.clipRect(Offset.zero & size);
    canvas.drawRect(Offset.zero & size, Paint()..color = _cream);
    final tile = size.width / 6;
    final rows = (size.height / tile).ceil();
    for (var row = 0; row < rows; row++) {
      for (var col = 0; col < 6; col++) {
        final id = row * 6 + col;
        canvas.save();
        canvas.clipRect(Rect.fromLTWH(col * tile, row * tile, tile, tile));
        canvas.translate(col * tile, row * tile);
        _paintTile(canvas, id % 48, tile, flipped.contains(id));
        _grain(canvas, tile, id);
        canvas.restore();
      }
    }
  }

  @override
  bool shouldRepaint(_SheetPainter oldDelegate) => oldDelegate.epoch != epoch;
}

void _paintTile(Canvas canvas, int kind, double s, bool swap) {
  switch (kind) {
    case 0:
      _diag(canvas, s, _swap(_cream, _orange, swap), backslash: true);
    case 1:
      final top = _swap(_blue, _black, swap);
      _edges(canvas, s, top.$1, top.$2, _orange, _pink);
    case 2:
      _diag(canvas, s, _swap(_pink, _orange, swap), backslash: false);
    case 3:
      _quarter(canvas, s, _swap(_orange, _blue, swap), 2, s * 1.12);
    case 4:
      _fill(canvas, s, _swap(_pink, _cream, swap).$1);
      _star(canvas, s, _swap(_pink, _cream, swap).$2, s * 0.46, 0.34);
    case 5:
      final pair = _swap(_blue, _black, swap);
      _fill(canvas, s, pair.$1);
      _disc(canvas, Offset(0, s), s * 0.62, pair.$2);
      _disc(canvas, Offset(s, 0), s * 0.46, pair.$2);
    case 6:
      _scallop(canvas, s, _swap(_blue, _green, swap), 2, false);
    case 7:
      _diag(canvas, s, _swap(_cream, _black, swap), backslash: true);
    case 8:
      _diag(canvas, s, _swap(_green, _cream, swap), backslash: true);
    case 9:
      _quarter(canvas, s, _swap(_cream, _orange, swap), 2, s * 0.58);
    case 10:
      _quarter(canvas, s, _swap(_green, _black, swap), 0, s * 0.92);
    case 11:
      _quarter(canvas, s, _swap(_cream, _yellow, swap), 1, s * 0.92);
    case 12:
      _fill(canvas, s, _swap(_pink, _orange, swap).$1);
      _leaf(
        canvas,
        Offset(s * 0.06, s * 0.78),
        Offset(s * 0.94, s * 0.16),
        s * 0.30,
        _swap(_pink, _orange, swap).$2,
      );
    case 13:
      _quarter(canvas, s, _swap(_cream, _yellow, swap), 3, s * 0.88);
    case 14:
      _pinwheel(canvas, s, _swap(_blue, _black, swap));
    case 15:
      _quarter(canvas, s, _swap(_cream, _green, swap), 3, s * 1.08);
    case 16:
      _triGrid(canvas, s, _swap(_cream, _orange, swap));
    case 17:
      _fill(canvas, s, _swap(_orange, _black, swap).$1);
      _asterisk(canvas, s, _swap(_orange, _black, swap).$2);
    case 18:
      _fill(canvas, s, _swap(_green, _cream, swap).$1);
      final face = _swap(_green, _cream, swap).$2;
      _disc(canvas, Offset(0, s / 2), s * 0.48, face);
      _disc(canvas, Offset(s, s / 2), s * 0.48, face);
    case 19:
      _diag(canvas, s, _swap(_blue, _orange, swap), backslash: false);
    case 20:
      _fill(canvas, s, _swap(_pink, _cream, swap).$1);
      _disc(canvas, Offset(s / 2, s), s * 1.02, _swap(_pink, _cream, swap).$2);
    case 21:
      _window(canvas, s, _swap(_cream, _black, swap));
    case 22:
      _scallop(canvas, s, _swap(_blue, _green, swap), 3, true);
    case 23:
      _scallop(canvas, s, _swap(_pink, _black, swap), 2, true);
    case 24:
      final ring = _swap(_blue, _orange, swap);
      _fill(canvas, s, ring.$1);
      _disc(canvas, Offset(s / 2, s / 2), s * 0.36, ring.$2);
      _star(canvas, s, _black, s * 0.22, 0.28);
    case 25:
      final ground = _swap(_orange, _cream, swap);
      _dots(canvas, s, ground.$1, ground.$2);
    case 26:
      _fill(canvas, s, _swap(_green, _pink, swap).$1);
      _petals(canvas, s / 2, s / 2, s * 0.40, s * 0.22, _swap(_green, _pink, swap).$2);
    case 27:
      _chevrons(canvas, s, _swap(_orange, _yellow, swap));
    case 28:
      _fill(canvas, s, _swap(_pink, _blue, swap).$1);
      final leafColor = _swap(_pink, _blue, swap).$2;
      _sprout(canvas, Offset(s / 2, s * 0.40), s * 0.22, leafColor);
      _sprout(canvas, Offset(s / 2, s * 0.78), s * 0.24, leafColor);
    case 29:
      _checker(canvas, s, _swap(_cream, _green, swap));
    case 30:
      _quarter(canvas, s, _swap(_orange, _cream, swap), 0, s * 1.02);
    case 31:
      _splitDisc(canvas, s, _swap(_black, _green, swap));
    case 32:
      _fill(canvas, s, _cream);
      final caps = _swap(_yellow, _pink, swap);
      _disc(canvas, Offset(s / 2, 0), s * 0.54, caps.$1);
      _disc(canvas, Offset(s / 2, s), s * 0.54, caps.$2);
    case 33:
      _lobes(canvas, s, _swap(_light, _cream, swap));
    case 34:
      final block = _swap(_green, _cream, swap);
      _fill(canvas, s, block.$1);
      canvas.drawRect(
        Rect.fromCenter(center: Offset(s / 2, s / 2), width: s * 0.42, height: s * 0.42),
        Paint()..color = block.$2,
      );
    case 35:
      _bullseye(canvas, s, _swap(_orange, _blue, swap));
    case 36:
      _quadCircles(canvas, s, _swap(_pink, _cream, swap));
    case 37:
      _stripes(canvas, s, _swap(_cream, _blue, swap), vertical: true, count: 5);
    case 38:
      _fill(canvas, s, _swap(_blue, _light, swap).$1);
      _leaf(
        canvas,
        Offset(s * 0.02, s * 0.82),
        Offset(s * 0.98, s * 0.18),
        s * 0.34,
        _swap(_blue, _light, swap).$2,
      );
    case 39:
      _twoDiscs(canvas, s, _swap(_orange, _blue, swap));
    case 40:
      _fill(canvas, s, _cream);
      final halves = _swap(_pink, _light, swap);
      _half(canvas, s, halves.$1, true);
      _half(canvas, s, halves.$2, false);
    case 41:
      final halves = _swap(_orange, _cream, swap);
      canvas.drawRect(Rect.fromLTWH(0, 0, s / 2, s), Paint()..color = halves.$1);
      canvas.drawRect(Rect.fromLTWH(s / 2, 0, s / 2, s), Paint()..color = halves.$2);
      _disc(canvas, Offset(s * 0.25, s * 0.5), s * 0.18, _green);
    case 42:
      _edges(canvas, s, _swap(_yellow, _black, swap).$1, _cream, _swap(_yellow, _black, swap).$2, _cream);
    case 43:
      _quarter(canvas, s, _swap(_cream, _light, swap), 0, s * 0.62);
    case 44:
      final ground = _swap(_pink, _orange, swap);
      _fill(canvas, s, ground.$1);
      _disc(canvas, Offset(s * 0.42, s * 0.56), s * 0.50, ground.$2);
      _disc(canvas, Offset.zero, s * 0.46, _black);
    case 45:
      _stripes(canvas, s, _swap(_cream, _blue, swap), vertical: false, count: 6);
    case 46:
      _diag(canvas, s, _swap(_orange, _blue, swap), backslash: false);
      _petals(canvas, s / 2, s / 2, s * 0.26, s * 0.15, _cream);
    case 47:
      _fill(canvas, s, _swap(_black, _orange, swap).$1);
      _sprout(canvas, Offset(s / 2, s * 0.46), s * 0.26, _swap(_black, _orange, swap).$2);
      _disc(canvas, Offset(s, s), s * 0.52, _cream);
    default:
      _fill(canvas, s, _cream);
  }
}

(Color, Color) _swap(Color a, Color b, bool swap) => swap ? (b, a) : (a, b);

void _fill(Canvas canvas, double s, Color color) {
  canvas.drawRect(Rect.fromLTWH(0, 0, s, s), Paint()..color = color);
}

void _disc(Canvas canvas, Offset center, double radius, Color color) {
  canvas.drawCircle(center, radius, Paint()..color = color);
}

void _diag(Canvas canvas, double s, (Color, Color) colors, {required bool backslash}) {
  final a = colors.$1;
  final b = colors.$2;
  if (backslash) {
    _fill(canvas, s, a);
    canvas.drawPath(
      Path()
        ..moveTo(s, 0)
        ..lineTo(s, s)
        ..lineTo(0, s)
        ..close(),
      Paint()..color = b,
    );
  } else {
    _fill(canvas, s, a);
    canvas.drawPath(
      Path()
        ..moveTo(0, 0)
        ..lineTo(s, 0)
        ..lineTo(s, s)
        ..close(),
      Paint()..color = b,
    );
  }
}

void _edges(Canvas canvas, double s, Color top, Color right, Color bottom, Color left) {
  final c = Offset(s / 2, s / 2);
  void tri(Offset a, Offset b, Color color) {
    canvas.drawPath(
      Path()
        ..moveTo(c.dx, c.dy)
        ..lineTo(a.dx, a.dy)
        ..lineTo(b.dx, b.dy)
        ..close(),
      Paint()..color = color,
    );
  }

  tri(Offset.zero, Offset(s, 0), top);
  tri(Offset(s, 0), Offset(s, s), right);
  tri(Offset(s, s), Offset(0, s), bottom);
  tri(Offset(0, s), Offset.zero, left);
}

void _quarter(Canvas canvas, double s, (Color, Color) colors, int corner, double radius) {
  _fill(canvas, s, colors.$1);
  const anchors = <Offset>[
    Offset(0, 0),
    Offset(1, 0),
    Offset(0, 1),
    Offset(1, 1),
  ];
  final anchor = anchors[corner];
  _disc(canvas, Offset(anchor.dx * s, anchor.dy * s), radius, colors.$2);
}

void _star(Canvas canvas, double s, Color color, double outer, double waist) {
  final c = s / 2;
  final inner = outer * waist;
  final points = <Offset>[
    Offset(c, c - outer),
    Offset(c + inner, c - inner),
    Offset(c + outer, c),
    Offset(c + inner, c + inner),
    Offset(c, c + outer),
    Offset(c - inner, c + inner),
    Offset(c - outer, c),
    Offset(c - inner, c - inner),
  ];
  final path = Path()..moveTo(points.first.dx, points.first.dy);
  for (final point in points.skip(1)) {
    path.lineTo(point.dx, point.dy);
  }
  path.close();
  canvas.drawPath(path, Paint()..color = color);
}

void _scallop(Canvas canvas, double s, (Color, Color) colors, int lobes, bool invert) {
  _fill(canvas, s, colors.$2);
  final path = Path()..moveTo(0, 0);
  for (var i = 0; i < lobes; i++) {
    final y0 = s * i / lobes;
    final y1 = s * (i + 1) / lobes;
    final mid = (y0 + y1) / 2;
    final bulge = (i.isEven != invert) ? 0.78 : 0.28;
    path.quadraticBezierTo(s * bulge, mid, s * 0.50, y1);
  }
  path
    ..lineTo(0, s)
    ..close();
  canvas.drawPath(path, Paint()..color = colors.$1);
}

void _leaf(Canvas canvas, Offset a, Offset b, double bulge, Color color) {
  final dx = b.dx - a.dx;
  final dy = b.dy - a.dy;
  final len = _sqrt(dx * dx + dy * dy);
  final nx = len == 0 ? 0.0 : -dy / len * bulge;
  final ny = len == 0 ? 0.0 : dx / len * bulge;
  final mx = (a.dx + b.dx) / 2;
  final my = (a.dy + b.dy) / 2;
  canvas.drawPath(
    Path()
      ..moveTo(a.dx, a.dy)
      ..quadraticBezierTo(mx + nx, my + ny, b.dx, b.dy)
      ..quadraticBezierTo(mx - nx, my - ny, a.dx, a.dy)
      ..close(),
    Paint()..color = color,
  );
}

void _pinwheel(Canvas canvas, double s, (Color, Color) colors) {
  final h = s / 2;
  final a = colors.$1;
  final b = colors.$2;
  canvas.drawRect(Rect.fromLTWH(0, 0, h, h), Paint()..color = b);
  canvas.drawPath(
    Path()
      ..moveTo(0, 0)
      ..lineTo(h, 0)
      ..lineTo(0, h)
      ..close(),
    Paint()..color = a,
  );
  canvas.drawRect(Rect.fromLTWH(h, 0, h, h), Paint()..color = b);
  canvas.drawPath(
    Path()
      ..moveTo(s, 0)
      ..lineTo(h, 0)
      ..lineTo(s, h)
      ..close(),
    Paint()..color = a,
  );
  canvas.drawRect(Rect.fromLTWH(h, h, h, h), Paint()..color = b);
  canvas.drawPath(
    Path()
      ..moveTo(s, s)
      ..lineTo(h, s)
      ..lineTo(s, h)
      ..close(),
    Paint()..color = a,
  );
  canvas.drawRect(Rect.fromLTWH(0, h, h, h), Paint()..color = b);
  canvas.drawPath(
    Path()
      ..moveTo(0, s)
      ..lineTo(h, s)
      ..lineTo(0, h)
      ..close(),
    Paint()..color = a,
  );
}

void _triGrid(Canvas canvas, double s, (Color, Color) colors) {
  const n = 3;
  final cell = s / n;
  for (var row = 0; row < n; row++) {
    for (var col = 0; col < n; col++) {
      final x = col * cell;
      final y = row * cell;
      final flip = (row + col).isEven;
      canvas.drawRect(
        Rect.fromLTWH(x, y, cell, cell),
        Paint()..color = flip ? colors.$1 : colors.$2,
      );
      canvas.drawPath(
        Path()
          ..moveTo(x, y)
          ..lineTo(x + cell, y)
          ..lineTo(x + cell, y + cell)
          ..close(),
        Paint()..color = flip ? colors.$2 : colors.$1,
      );
    }
  }
}

void _asterisk(Canvas canvas, double s, Color color) {
  final paint = Paint()
    ..color = color
    ..strokeWidth = s * 0.04
    ..strokeCap = StrokeCap.square;
  final c = Offset(s / 2, s / 2);
  const arms = <Offset>[
    Offset(1, 0),
    Offset(0.7071, 0.7071),
    Offset(0, 1),
    Offset(-0.7071, 0.7071),
    Offset(-1, 0),
    Offset(-0.7071, -0.7071),
    Offset(0, -1),
    Offset(0.7071, -0.7071),
  ];
  final reach = s * 0.30;
  for (final arm in arms) {
    canvas.drawLine(c, Offset(c.dx + arm.dx * reach, c.dy + arm.dy * reach), paint);
  }
}

void _window(Canvas canvas, double s, (Color, Color) colors) {
  _fill(canvas, s, colors.$2);
  const n = 3;
  final bar = s * 0.07;
  final cell = (s - bar * (n + 1)) / n;
  for (var row = 0; row < n; row++) {
    for (var col = 0; col < n; col++) {
      canvas.drawRect(
        Rect.fromLTWH(bar + col * (cell + bar), bar + row * (cell + bar), cell, cell),
        Paint()..color = colors.$1,
      );
    }
  }
}

void _dots(Canvas canvas, double s, Color ground, Color dot) {
  _fill(canvas, s, ground);
  const n = 3;
  for (var row = 0; row < n; row++) {
    for (var col = 0; col < n; col++) {
      final marked = (row == 0 && col == 1) || (row == 2 && col == 2);
      _disc(
        canvas,
        Offset((col + 0.5) * s / n, (row + 0.5) * s / n),
        s * 0.105,
        marked ? _black : dot,
      );
    }
  }
}

void _petals(Canvas canvas, double cx, double cy, double reach, double width, Color color) {
  for (var i = 0; i < 4; i++) {
    canvas.save();
    canvas.translate(cx, cy);
    canvas.rotate(i * 1.5707963267948966);
    canvas.drawPath(
      Path()
        ..moveTo(0, 0)
        ..quadraticBezierTo(width, -reach * 0.55, 0, -reach)
        ..quadraticBezierTo(-width, -reach * 0.55, 0, 0)
        ..close(),
      Paint()..color = color,
    );
    canvas.restore();
  }
}

void _chevrons(Canvas canvas, double s, (Color, Color) colors) {
  const cols = 2;
  const rows = 3;
  final cw = s / cols;
  final rh = s / rows;
  for (var row = 0; row < rows; row++) {
    for (var col = 0; col < cols; col++) {
      final x = col * cw;
      final y = row * rh;
      final fore = (row + col).isEven ? colors.$1 : colors.$2;
      final back = (row + col).isEven ? colors.$2 : colors.$1;
      canvas.drawRect(Rect.fromLTWH(x, y, cw, rh), Paint()..color = back);
      canvas.drawPath(
        Path()
          ..moveTo(x, y)
          ..lineTo(x, y + rh)
          ..lineTo(x + cw, y + rh / 2)
          ..close(),
        Paint()..color = fore,
      );
    }
  }
}

void _sprout(Canvas canvas, Offset origin, double size, Color color) {
  for (final turn in <double>[-0.55, 0.55]) {
    canvas.save();
    canvas.translate(origin.dx, origin.dy);
    canvas.rotate(turn);
    canvas.drawPath(
      Path()
        ..moveTo(0, 0)
        ..quadraticBezierTo(size * 0.70, -size * 0.55, 0, -size)
        ..quadraticBezierTo(-size * 0.70, -size * 0.55, 0, 0)
        ..close(),
      Paint()..color = color,
    );
    canvas.restore();
  }
}

void _checker(Canvas canvas, double s, (Color, Color) colors) {
  final h = s / 2;
  canvas.drawRect(Rect.fromLTWH(0, 0, h, h), Paint()..color = colors.$1);
  canvas.drawRect(Rect.fromLTWH(h, 0, h, h), Paint()..color = colors.$2);
  canvas.drawRect(Rect.fromLTWH(0, h, h, h), Paint()..color = colors.$2);
  canvas.drawRect(Rect.fromLTWH(h, h, h, h), Paint()..color = colors.$1);
}

void _splitDisc(Canvas canvas, double s, (Color, Color) field) {
  _diag(canvas, s, field, backslash: false);
  final center = Offset(s * 0.56, s * 0.46);
  final radius = s * 0.28;
  canvas.save();
  canvas.clipPath(Path()..addOval(Rect.fromCircle(center: center, radius: radius)));
  _diag(canvas, s, (_pink, _blue), backslash: false);
  canvas.restore();
}

void _lobes(Canvas canvas, double s, (Color, Color) colors) {
  _fill(canvas, s, colors.$1);
  final radius = s * 0.40;
  final paint = Paint()..color = colors.$2;
  canvas.drawCircle(Offset(s / 2, 0), radius, paint);
  canvas.drawCircle(Offset(s / 2, s), radius, paint);
  canvas.drawCircle(Offset(0, s / 2), radius, paint);
  canvas.drawCircle(Offset(s, s / 2), radius, paint);
}

void _bullseye(Canvas canvas, double s, (Color, Color) colors) {
  _fill(canvas, s, colors.$1);
  _disc(canvas, Offset(s / 2, s / 2), s * 0.40, colors.$2);
  _disc(canvas, Offset(s / 2, s / 2), s * 0.26, _cream);
  _disc(canvas, Offset(s / 2, s / 2), s * 0.12, _black);
}

void _quadCircles(Canvas canvas, double s, (Color, Color) colors) {
  _fill(canvas, s, colors.$1);
  final radius = s * 0.30;
  final spots = <Offset>[
    Offset(s * 0.26, s * 0.26),
    Offset(s * 0.74, s * 0.26),
    Offset(s * 0.26, s * 0.74),
    Offset(s * 0.74, s * 0.74),
  ];
  final paints = <Color>[colors.$2, _green, _blue, _yellow];
  for (var i = 0; i < 4; i++) {
    _disc(canvas, spots[i], radius, paints[i]);
  }
}

void _stripes(
  Canvas canvas,
  double s,
  (Color, Color) colors, {
  required bool vertical,
  required int count,
}) {
  final span = s / count;
  for (var i = 0; i < count; i++) {
    final color = i.isEven ? colors.$1 : colors.$2;
    final rect = vertical
        ? Rect.fromLTWH(i * span, 0, span, s)
        : Rect.fromLTWH(0, i * span, s, span);
    canvas.drawRect(rect, Paint()..color = color);
  }
}

void _twoDiscs(Canvas canvas, double s, (Color, Color) colors) {
  _fill(canvas, s, _black);
  _disc(canvas, Offset(s * 0.30, s * 0.50), s * 0.42, colors.$1);
  _disc(canvas, Offset(s * 0.70, s * 0.50), s * 0.42, colors.$2);
}

void _half(Canvas canvas, double s, Color color, bool upper) {
  canvas.drawArc(
    Rect.fromLTWH(0, 0, s, s),
    upper ? 3.141592653589793 : 0,
    upper ? -3.141592653589793 : 3.141592653589793,
    true,
    Paint()..color = color,
  );
}

void _grain(Canvas canvas, double s, int seed) {
  final dark = Paint()..color = const Color(0x28140E08);
  final light = Paint()..color = const Color(0x22FFF8EA);
  final count = (s * 0.45).clamp(22, 72).round();
  for (var i = 0; i < count; i++) {
    final h = _mix(seed, i);
    final x = (h % 997) / 997 * s;
    final y = ((h >> 9) % 991) / 991 * s;
    final radius = 0.35 + (h % 3) * 0.35;
    canvas.drawCircle(Offset(x, y), radius, i.isEven ? dark : light);
  }
}

int _mix(int seed, int i) {
  var n = seed * 374761393 + i * 668265263;
  n = (n ^ (n >> 13)) * 1274126177;
  return n & 0x7fffffff;
}

double _sqrt(double value) {
  if (value <= 0) return 0;
  var guess = value;
  for (var i = 0; i < 8; i++) {
    guess = 0.5 * (guess + value / guess);
  }
  return guess;
}
