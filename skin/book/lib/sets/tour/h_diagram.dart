import 'package:flutter/widgets.dart';

class TourDiagram extends StatefulWidget {
  const TourDiagram({super.key});

  @override
  State<TourDiagram> createState() => _TourDiagramState();
}

class _TourDiagramState extends State<TourDiagram> {
  int _blue = 0;

  @override
  Widget build(BuildContext context) {
    return ClipRect(
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: (details) {
          final box = context.findRenderObject() as RenderBox?;
          if (box == null || !box.hasSize) return;
          if (_blockOf(box.size).contains(details.localPosition)) {
            setState(() => _blue = (_blue + 1) % 6);
          }
        },
        child: CustomPaint(
          painter: _DiagramPainter(_blue),
          child: const SizedBox.expand(),
        ),
      ),
    );
  }
}

class _Laid {
  _Laid(this.painter, this.at);
  final TextPainter painter;
  final Offset at;
  double get right => at.dx + painter.width;
  double get bottom => at.dy + painter.height;
}

class _DiagramPainter extends CustomPainter {
  _DiagramPainter(this.blue);

  final int blue;

  static const _ink = Color(0xFF121212);
  static const _sheet = Color(0xFFF3EFE4);
  static const _blue = Color(0xFF1A3CE0);
  static const _yellow = Color(0xFFFFD000);

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width < 8 || size.height < 8) return;
    canvas.clipRect(Offset.zero & size);
    final block = _blockOf(size);
    final letter = _layoutM(size, block);
    _paper(canvas, size);
    _rules(canvas, size, block, letter);
    _barcode(canvas, size, block);
    _rings(canvas, size, block, letter);
    _geometry(canvas, block);
    letter.painter.paint(canvas, letter.at);
    _words(canvas, size, block, letter);
    _zero(canvas, size, block);
  }

  @override
  bool shouldRepaint(covariant _DiagramPainter oldDelegate) =>
      oldDelegate.blue != blue;

  void _paper(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = _sheet);
    final rng = _Rng(11);
    final paint = Paint();
    for (var i = 0; i < 640; i++) {
      paint.color = Color.fromRGBO(70, 56, 38, 0.035 + rng.next() * 0.06);
      canvas.drawCircle(
        Offset(rng.next() * size.width, rng.next() * size.height),
        0.25 + rng.next() * 0.75,
        paint,
      );
    }
    for (var i = 0; i < 80; i++) {
      final x = rng.next() * size.width;
      final y = rng.next() * size.height;
      paint
        ..color = Color.fromRGBO(60, 48, 30, 0.035)
        ..strokeWidth = 0.4;
      canvas.drawLine(
        Offset(x, y),
        Offset(x + 2 + rng.next() * 8, y + rng.next()),
        paint,
      );
    }
  }

  void _rules(Canvas canvas, Size size, Rect block, _Laid letter) {
    final gapLeft = letter.right + size.width * 0.04;
    final y = letter.at.dy + letter.painter.height * 0.5;
    final end = Offset(block.left - size.width * 0.014, y);
    if (end.dx - gapLeft < 28 || y < 4 || y > size.height - 4) return;
    final start = Offset(gapLeft, y);
    _dash(canvas, start, end);
    _dot(canvas, start);
    _dot(canvas, end);
    _dot(canvas, Offset((start.dx + end.dx) * 0.5, y + size.height * 0.06));
    final foot = Offset(end.dx, block.bottom);
    _dash(canvas, end, foot);
    _dot(canvas, foot);
    final back = Offset(end.dx - size.width * 0.055, block.bottom);
    if (back.dx > letter.right) {
      _dash(canvas, foot, back);
      _dot(canvas, back);
    }
  }

  void _rings(Canvas canvas, Size size, Rect block, _Laid letter) {
    final gap = block.left - letter.right;
    if (gap < 26) return;
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.15
      ..color = _ink;
    final x = letter.right + _min(12, gap * 0.12);
    final y = letter.at.dy;
    final h = letter.painter.height;
    final marks = <(double, double, double)>[
      (x + gap * 0.045, y + h * 0.16, _min(7.5, gap * 0.08)),
      (x + gap * 0.12, y + h * 0.27, _min(4.2, gap * 0.05)),
      (x + gap * 0.03, y + h * 0.36, _min(2.4, gap * 0.035)),
    ];
    for (final mark in marks) {
      if (mark.$1 + mark.$3 < block.left - 6) {
        canvas.drawCircle(Offset(mark.$1, mark.$2), mark.$3, paint);
      }
    }
  }

  void _barcode(Canvas canvas, Size size, Rect block) {
    final rng = _Rng(19);
    final right = size.width - _max(3, size.width * 0.008);
    var x = block.right + _max(7, size.width * 0.012);
    final paint = Paint()..color = _ink;
    while (x < right) {
      final thick = 0.7 + rng.next() * 2.5;
      if (x + thick > right) break;
      final span = block.height;
      final crop = rng.next();
      final y0 =
          block.top + (crop < 0.22 ? span * (0.08 + rng.next() * 0.18) : 0);
      final y1 =
          block.bottom - (crop > 0.78 ? span * (0.06 + rng.next() * 0.16) : 0);
      paint.strokeWidth = thick;
      canvas.drawLine(
        Offset(x + thick / 2, y0),
        Offset(x + thick / 2, y1),
        paint,
      );
      x += thick + 0.85 + rng.next() * 1.45;
    }
  }

  void _geometry(Canvas canvas, Rect block) {
    final yellow = blue == 4 ? 2 : 4;
    final tiles = _tilesOf(block);
    for (var i = 0; i < tiles.length; i++) {
      final tone = _tone(i, yellow);
      switch (i) {
        case 0:
          _circle(canvas, tiles[i], tone.$1, tone.$2);
        case 1:
          _arcs(canvas, tiles[i], tone.$1, tone.$2);
        case 2:
          _quarter(canvas, tiles[i], tone.$1, tone.$2);
        case 3:
          _triangles(canvas, tiles[i], tone.$1, tone.$2);
        case 4:
          _sun(canvas, tiles[i], tone.$1, tone.$2);
        default:
          _lobes(canvas, tiles[i], tone.$1, tone.$2);
      }
      canvas.drawRect(
        tiles[i],
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = _ink,
      );
    }
    canvas.drawRect(
      block,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.25
        ..color = _ink,
    );
  }

  (Color, Color) _tone(int kind, int yellow) {
    const white = Color(0xFFFFFFFF);
    final dark = kind == 1;
    if (kind == blue) {
      return dark ? (_blue, white) : (white, _blue);
    }
    if (kind == yellow) return (_yellow, _ink);
    if (dark) return (_ink, white);
    return (white, _ink);
  }

  void _circle(Canvas canvas, Rect r, Color ground, Color mark) {
    canvas.drawRect(r, Paint()..color = ground);
    final rad = _min(r.width, r.height) * 0.36;
    canvas.drawCircle(r.center, rad, Paint()..color = mark);
  }

  void _arcs(Canvas canvas, Rect r, Color ground, Color mark) {
    canvas.drawRect(r, Paint()..color = ground);
    canvas.save();
    canvas.clipRect(r);
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = _max(1.1, _min(r.width, r.height) * 0.042)
      ..color = mark;
    const n = 8;
    for (var i = 1; i <= n; i++) {
      canvas.drawArc(
        Rect.fromCircle(
          center: Offset(r.left - r.width * 0.04, r.bottom + r.height * 0.06),
          radius: r.width * 0.155 * i,
        ),
        -_pi / 2,
        _pi / 2,
        false,
        paint,
      );
    }
    canvas.restore();
  }

  void _quarter(Canvas canvas, Rect r, Color ground, Color mark) {
    canvas.drawRect(r, Paint()..color = ground);
    final side = _min(r.width, r.height) * 0.86;
    final oval = Rect.fromCenter(center: r.center, width: side, height: side);
    final fill = Paint()..color = mark;
    canvas.drawArc(oval, _pi, _pi / 2, true, fill);
    canvas.drawArc(oval, 0, _pi / 2, true, fill);
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = _max(1, side * 0.016)
      ..color = mark;
    canvas.drawOval(oval, stroke);
    canvas.drawLine(
      Offset(oval.left, r.center.dy),
      Offset(oval.right, r.center.dy),
      stroke,
    );
    canvas.drawLine(
      Offset(r.center.dx, oval.top),
      Offset(r.center.dx, oval.bottom),
      stroke,
    );
  }

  void _triangles(Canvas canvas, Rect r, Color ground, Color mark) {
    canvas.drawRect(r, Paint()..color = ground);
    const cols = 4;
    const rows = 3;
    final cw = r.width / cols;
    final ch = r.height / rows;
    final fill = Paint()..color = mark;
    final edge = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = _max(0.7, _min(cw, ch) * 0.035)
      ..color = mark;
    for (var y = 0; y < rows; y++) {
      for (var x = 0; x < cols; x++) {
        final ax = r.left + x * cw;
        final ay = r.top + y * ch;
        final path = Path();
        if ((x + y).isEven) {
          path
            ..moveTo(ax, ay + ch)
            ..lineTo(ax + cw / 2, ay)
            ..lineTo(ax + cw, ay + ch);
        } else {
          path
            ..moveTo(ax, ay)
            ..lineTo(ax + cw / 2, ay + ch)
            ..lineTo(ax + cw, ay);
        }
        path.close();
        canvas.drawPath(path, fill);
        canvas.drawPath(path, edge);
      }
    }
  }

  void _sun(Canvas canvas, Rect r, Color ground, Color mark) {
    canvas.drawRect(r, Paint()..color = ground);
    final paint = Paint()..color = mark;
    const n = 18;
    final sweep = _tau / n;
    final oval = Rect.fromCircle(
      center: r.center,
      radius: _len(r.width, r.height),
    );
    canvas.save();
    canvas.clipRect(r);
    for (var i = 0; i < n; i += 2) {
      canvas.drawArc(oval, -_pi / 2 + i * sweep, sweep, true, paint);
    }
    canvas.restore();
  }

  void _lobes(Canvas canvas, Rect r, Color ground, Color mark) {
    canvas.drawRect(r, Paint()..color = ground);
    final paint = Paint()..color = mark;
    final c = r.center;
    final rad = _min(r.width, r.height) * 0.23;
    canvas.drawCircle(Offset(c.dx - rad, c.dy), rad, paint);
    canvas.drawCircle(Offset(c.dx + rad, c.dy), rad, paint);
    canvas.drawCircle(Offset(c.dx, c.dy - rad), rad, paint);
    canvas.drawCircle(Offset(c.dx, c.dy + rad), rad, paint);
  }

  void _words(Canvas canvas, Size size, Rect block, _Laid letter) {
    final word = _min(_max(12, size.height * 0.046), 40);
    final style = TextStyle(
      fontFamily: 'Arial Black',
      fontSize: word,
      fontWeight: FontWeight.w400,
      letterSpacing: -word * 0.045,
      height: 1,
      color: _ink,
      decoration: TextDecoration.none,
    );
    final maxWidth = _max(20, block.left - letter.at.dx - 8);
    final y = letter.bottom + size.height * 0.012;
    _text(canvas, 'CUT', Offset(letter.at.dx, y), style, maxWidth);
    _text(
      canvas,
      'HOLD',
      Offset(letter.at.dx, y + word * 0.88),
      style,
      maxWidth,
    );
  }

  void _zero(Canvas canvas, Size size, Rect block) {
    final z = _min(_max(10, size.shortestSide * 0.02), 18);
    final style = TextStyle(
      fontFamily: 'Helvetica Neue',
      fontSize: z,
      fontWeight: FontWeight.w700,
      letterSpacing: 0.6,
      height: 1,
      color: _ink,
      decoration: TextDecoration.none,
    );
    var y = block.bottom + z * 0.7;
    if (y + z > size.height - 3) y = _max(3, block.top - z * 1.35);
    _text(canvas, '00', Offset(size.width - z * 2.5, y), style, z * 3);
  }
}

_Laid _layoutM(Size size, Rect block) {
  final left = size.width * 0.045;
  final top = size.height * 0.05;
  final word = _min(_max(12.0, size.height * 0.046), 40.0);
  final maxH = _max(12.0, size.height - top - word * 2.05 - size.height * 0.04);
  final maxW = _max(
    12.0,
    _min(block.left - left - size.width * 0.1, size.width * 0.3),
  );
  final probe = TextPainter(
    text: TextSpan(text: 'M', style: _mStyle(100)),
    textDirection: TextDirection.ltr,
  )..layout();
  final fs = 100 * _min(maxW / probe.width, maxH / probe.height);
  final painter = TextPainter(
    text: TextSpan(text: 'M', style: _mStyle(fs)),
    textDirection: TextDirection.ltr,
  )..layout();
  return _Laid(painter, Offset(left, top));
}

TextStyle _mStyle(double size) {
  return TextStyle(
    fontFamily: 'Arial Black',
    fontSize: size,
    fontWeight: FontWeight.w400,
    height: 1,
    color: _DiagramPainter._ink,
    decoration: TextDecoration.none,
  );
}

void _text(
  Canvas canvas,
  String text,
  Offset at,
  TextStyle style,
  double maxWidth,
) {
  final painter = TextPainter(
    text: TextSpan(text: text, style: style),
    textDirection: TextDirection.ltr,
    maxLines: 1,
  )..layout(maxWidth: maxWidth);
  painter.paint(canvas, at);
}

Rect _blockOf(Size size) {
  final gutter = _max(18, size.width * 0.055);
  final bw = _min(
    size.width * 0.33,
    _min(size.height * 0.7, size.width - gutter - size.width * 0.46),
  );
  final bh = _min(size.height * 0.64, _max(bw, 1) * 1.16);
  return Rect.fromLTWH(
    size.width - gutter - bw,
    size.height * 0.13,
    _max(bw, 1),
    _max(bh, 1),
  );
}

List<Rect> _tilesOf(Rect b) {
  final l = b.left;
  final t = b.top;
  final circleSide = _min(b.width * 0.5, b.height * 0.5);
  final rightW = b.width - circleSide;
  final sunSide = _min(rightW * 0.46, circleSide * 0.4);
  final arcH = circleSide - sunSide;
  final bot = b.height - circleSide;
  return [
    Rect.fromLTWH(l, t, circleSide, circleSide),
    Rect.fromLTWH(l + circleSide, t, rightW, arcH),
    Rect.fromLTWH(
      l + circleSide + sunSide,
      t + arcH,
      rightW - sunSide,
      sunSide,
    ),
    Rect.fromLTWH(l + circleSide, t + circleSide, rightW, bot),
    Rect.fromLTWH(l + circleSide, t + arcH, sunSide, sunSide),
    Rect.fromLTWH(l, t + circleSide, circleSide, bot),
  ];
}

void _dash(Canvas canvas, Offset a, Offset b) {
  final dx = b.dx - a.dx;
  final dy = b.dy - a.dy;
  final len = _len(dx, dy);
  if (len < 2) return;
  final ux = dx / len;
  final uy = dy / len;
  final paint = Paint()
    ..color = _DiagramPainter._ink
    ..strokeWidth = 0.8;
  const dash = 4.5;
  const gap = 3.5;
  var t = 0.0;
  while (t < len) {
    final t2 = _min(t + dash, len);
    canvas.drawLine(
      Offset(a.dx + ux * t, a.dy + uy * t),
      Offset(a.dx + ux * t2, a.dy + uy * t2),
      paint,
    );
    t = t2 + gap;
  }
}

void _dot(Canvas canvas, Offset p) {
  canvas.drawCircle(p, 2, Paint()..color = _DiagramPainter._ink);
}

const _pi = 3.141592653589793;
const _tau = 6.283185307179586;

double _min(double a, double b) => a < b ? a : b;
double _max(double a, double b) => a > b ? a : b;

double _sqrt(double v) {
  if (v <= 0) return 0;
  var g = v < 1 ? 1.0 : v;
  for (var i = 0; i < 8; i++) {
    g = 0.5 * (g + v / g);
  }
  return g;
}

double _len(double x, double y) => _sqrt(x * x + y * y);

class _Rng {
  _Rng(this._s);
  int _s;
  double next() {
    _s = (1664525 * _s + 1013904223) & 0x7fffffff;
    return _s / 0x7fffffff;
  }
}
