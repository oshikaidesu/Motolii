import 'package:flutter/widgets.dart';

class TourP5 extends StatefulWidget {
  const TourP5({super.key});

  @override
  State<TourP5> createState() => _TourP5State();
}

class _TourP5State extends State<TourP5> {
  bool _swap = false;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        final h = box.maxHeight;
        if (!w.isFinite || !h.isFinite || w <= 0 || h <= 0) {
          return const ColoredBox(color: _paper);
        }
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: () => setState(() => _swap = !_swap),
          child: ClipRect(
            child: CustomPaint(
              painter: _Sheet(swap: _swap),
              child: const SizedBox.expand(),
            ),
          ),
        );
      },
    );
  }
}

const _paper = Color(0xFFF3EFE4);
const _ink = Color(0xFF121212);
const _white = Color(0xFFFFFFFF);
const _blue = Color(0xFF1A3CE0);
const _yellow = Color(0xFFFFD000);

class _Sheet extends CustomPainter {
  const _Sheet({required this.swap});

  final bool swap;

  @override
  void paint(Canvas canvas, Size size) {
    final m = _Map(size);
    canvas.drawRect(Offset.zero & size, Paint()..color = _paper);
    _guides(canvas, m);
    _plate(canvas, m);
    _type(canvas, m);
  }

  void _guides(Canvas canvas, _Map m) {
    final ink = Paint()..color = _ink;
    final ring = Paint()
      ..color = _ink
      ..style = PaintingStyle.stroke;
    ring.strokeWidth = 1.6 * m.scale;
    canvas.drawCircle(m.at(526, 116.5), 7.6 * m.scale, ring);
    ring.strokeWidth = 1.3 * m.scale;
    canvas.drawCircle(m.at(555, 168.5), 4.5 * m.scale, ring);
    ring.strokeWidth = 1.15 * m.scale;
    canvas.drawCircle(m.at(520, 209.5), 3.2 * m.scale, ring);
    _beads(canvas, m, 568, 274, 848, 274);
    _beads(canvas, m, 865, 286, 865, 654);
    _beads(canvas, m, 800, 668, 850, 668);
    canvas.drawCircle(m.at(557, 274), 3.3 * m.scale, ink);
    canvas.drawCircle(m.at(862, 275), 3.6 * m.scale, ink);
    canvas.drawCircle(m.at(863, 666), 3.5 * m.scale, ink);
    canvas.drawCircle(m.at(788, 668), 3.2 * m.scale, ink);
    canvas.drawCircle(m.at(709, 328), 2.1 * m.scale, ink);
  }

  void _beads(Canvas canvas, _Map m, double x0, double y0, double x1, double y1) {
    final paint = Paint()..color = _ink;
    final dx = x1 - x0;
    final dy = y1 - y0;
    final span = dx.abs() > dy.abs() ? dx.abs() : dy.abs();
    if (span == 0) {
      return;
    }
    for (var t = 0.0; t <= span; t += 7) {
      final f = t / span;
      canvas.drawCircle(
        m.at(x0 + dx * f, y0 + dy * f),
        1.15 * m.scale,
        paint,
      );
    }
  }

  void _plate(Canvas canvas, _Map m) {
    final white = Paint()..color = _white;
    final ink = Paint()..color = _ink;
    final yellow = Paint()..color = _yellow;
    canvas.drawRect(m.box(885, 117, 1123, 354), white);
    canvas.drawRect(m.box(885, 354, 1123, 668), white);
    canvas.drawRect(m.box(1218, 259, 1360, 354), white);
    canvas.drawRect(m.box(1123, 117, 1360, 259), ink);
    canvas.drawRect(m.box(1123, 354, 1360, 668), ink);

    canvas.save();
    canvas.clipRect(m.box(1123, 117, 1360, 259));
    final arc = Paint()
      ..color = _white
      ..style = PaintingStyle.stroke
      ..strokeWidth = 6 * m.scale;
    final origin = m.at(1123, 259);
    for (var r = 24.0; r < 250; r += 37) {
      canvas.drawCircle(origin, r * m.scale, arc);
    }
    canvas.restore();

    canvas.save();
    canvas.clipRect(m.box(1123, 259, 1218, 354));
    final hub = m.at(1170.5, 306.5);
    final burst = Rect.fromCircle(center: hub, radius: 74 * m.scale);
    for (var i = 0; i < 18; i++) {
      final wedge = Path()
        ..moveTo(hub.dx, hub.dy)
        ..arcTo(burst, 0.192 + i * 0.349066, 0.349066, false)
        ..close();
      canvas.drawPath(wedge, i.isEven ? yellow : ink);
    }
    canvas.restore();

    canvas.drawCircle(
      m.at(1004, 236),
      84 * m.scale,
      Paint()..color = swap ? _yellow : _blue,
    );

    final clover = m.at(1004, 510);
    final reach = 53.5 * m.scale;
    final lobe = 54.5 * m.scale;
    canvas.drawCircle(clover + Offset(0, -reach), lobe, ink);
    canvas.drawCircle(clover + Offset(0, reach), lobe, ink);
    canvas.drawCircle(clover + Offset(-reach, 0), lobe, ink);
    canvas.drawCircle(clover + Offset(reach, 0), lobe, ink);

    final eye = m.at(1289, 306.5);
    final eyeR = 41 * m.scale;
    final eyeBox = Rect.fromCircle(center: eye, radius: eyeR);
    canvas.drawArc(eyeBox, 3.141592653589793, 1.5707963267948966, true, ink);
    canvas.drawArc(eyeBox, 0, 1.5707963267948966, true, ink);
    canvas.drawCircle(
      eye,
      eyeR,
      Paint()
        ..color = _ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.4 * m.scale,
    );
    final cross = Paint()
      ..color = _ink
      ..strokeWidth = 2.6 * m.scale;
    canvas.drawLine(Offset(eye.dx - eyeR, eye.dy), Offset(eye.dx + eyeR, eye.dy), cross);
    canvas.drawLine(Offset(eye.dx, eye.dy - eyeR), Offset(eye.dx, eye.dy + eyeR), cross);

    canvas.save();
    canvas.clipRect(m.box(1123, 354, 1360, 668));
    const mids = <double>[1122.5, 1182, 1241.5, 1301, 1360.5];
    for (var i = 0; i < mids.length; i++) {
      final dir = i.isOdd ? 1.0 : -1.0;
      const turns = <double>[354, 360, 465, 570, 668];
      final edge = <Offset>[for (final y in turns) _band(m, mids[i], dir, y)];
      final path = Path()..moveTo(edge.first.dx - 13 * m.scale, edge.first.dy);
      for (final p in edge) {
        path.lineTo(p.dx - 13 * m.scale, p.dy);
      }
      for (var k = edge.length - 1; k >= 0; k--) {
        path.lineTo(edge[k].dx + 13 * m.scale, edge[k].dy);
      }
      path.close();
      canvas.drawPath(path, white);
    }
    canvas.restore();

    final rule = Paint()
      ..color = _ink
      ..strokeWidth = 2 * m.scale
      ..style = PaintingStyle.stroke;
    canvas.drawRect(m.box(885, 117, 1360, 668), rule);
    final hair = Paint()
      ..color = _ink
      ..strokeWidth = 1.6 * m.scale;
    canvas.drawLine(m.at(1123, 117), m.at(1123, 668), hair);
    canvas.drawLine(m.at(885, 354), m.at(1360, 354), hair);
    canvas.drawLine(m.at(1123, 259), m.at(1360, 259), hair);
    canvas.drawLine(m.at(1218, 259), m.at(1218, 354), hair);

    const bars = <List<double>>[
      [1378, 2, 227],
      [1386, 2, 117],
      [1391, 2, 117],
      [1399, 1, 117],
      [1404, 2, 117],
      [1409, 2, 255],
      [1413, 2, 193],
      [1417, 2, 231],
      [1421, 2, 117],
    ];
    for (final bar in bars) {
      canvas.drawRect(m.box(bar[0], bar[2], bar[0] + bar[1], 667), ink);
    }
  }

  Offset _band(_Map m, double mid, double dir, double y) {
    final wave = _tri((y - 412.5) * (6.283185307179586 / 210));
    return m.at(mid + dir * 14.5 * wave, y);
  }

  void _type(Canvas canvas, _Map m) {
    _word(canvas, m, 'M', 'Arial Black', 455, 66, 401);
    _word(canvas, m, 'CUT', 'Arial Black', 38, 66, 544);
    _word(canvas, m, 'HOLD', 'Arial Black', 38, 66, 579);
    _word(canvas, m, '00', 'Arial', 15, 1395, 694);
  }

  void _word(
    Canvas canvas,
    _Map m,
    String text,
    String family,
    double fontSize,
    double originX,
    double baselineY,
  ) {
    final painter = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          fontFamily: family,
          fontSize: fontSize * m.scale,
          color: _ink,
          decoration: TextDecoration.none,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    final baseline = painter.computeDistanceToActualBaseline(TextBaseline.alphabetic);
    painter.paint(
      canvas,
      Offset(m.ox + originX * m.scale, m.oy + baselineY * m.scale - baseline),
    );
  }

  @override
  bool shouldRepaint(_Sheet oldDelegate) => oldDelegate.swap != swap;
}

class _Map {
  _Map(this.size)
      : scale = size.width / 1440 < size.height / 900
            ? size.width / 1440
            : size.height / 900,
        ox = 0,
        oy = 0 {
    final fittedW = 1440 * scale;
    final fittedH = 900 * scale;
    ox = (size.width - fittedW) / 2;
    oy = (size.height - fittedH) / 2;
  }

  final Size size;
  final double scale;
  double ox;
  double oy;

  Offset at(double x, double y) => Offset(ox + x * scale, oy + y * scale);

  Rect box(double x0, double y0, double x1, double y1) => Rect.fromPoints(at(x0, y0), at(x1, y1));
}

double _tri(double x) {
  const pi = 3.141592653589793;
  const tau = 6.283185307179586;
  var a = x % tau;
  if (a < 0) {
    a += tau;
  }
  if (a > pi) {
    a -= tau;
  }
  final q = a / (pi / 2);
  if (q < -1) {
    return -2 - q;
  }
  if (q > 1) {
    return 2 - q;
  }
  return q;
}
