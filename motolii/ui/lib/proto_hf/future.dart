import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../hf/bp/common.dart';
import '../hf/bp/shell.dart';
import '../hf/metrics.dart' show Surface;

/// Identity tiles for panels that do not exist yet. They only test that the same shell can hold them.
const futureNames = ['Media', 'Audio', 'Music', 'Physics', 'Materials', 'Environments', 'AI', 'Templates', 'Plugins'];

class FutureTile extends StatelessWidget {
  const FutureTile(this.name, {super.key});
  final String name;
  @override
  Widget build(BuildContext context) => Container(
        width: 88,
        height: 70,
        decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(4), border: Border.all(color: Surface.dividerFine)),
        child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
          SizedBox(width: 24, height: 24, child: CustomPaint(painter: _Icon(name))),
          const SizedBox(height: 8),
          Text(name, softWrap: false, style: sans(10.5, c: const Color(0xFFC4C5C8))),
        ]),
      );
}

class _Icon extends CustomPainter {
  _Icon(this.n);
  final String n;
  @override
  void paint(Canvas c, Size s) {
    final k = const Color(0xFFD0D1D5);
    final f = Paint()..color = k;
    final st = Paint()..color = k..style = PaintingStyle.stroke..strokeWidth = 1.7..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final w = s.width, h = s.height;
    switch (n) {
      case 'Media':
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(2, 4, w - 4, h - 8), const Radius.circular(2)), st);
        c.drawPath(Path()..moveTo(4, h - 6)..lineTo(9, 10)..lineTo(13, 15)..lineTo(16, 12)..lineTo(w - 4, h - 6)..close(), f);
      case 'Audio':
        for (var i = 0; i < 7; i++) {
          final a = [.35, .7, .5, 1.0, .6, .8, .4][i];
          c.drawLine(Offset(3 + i * 3.1, h / 2 - a * 9), Offset(3 + i * 3.1, h / 2 + a * 9), st..strokeWidth = 1.8);
        }
      case 'Music':
        c.drawCircle(Offset(8, h - 6), 3.4, f);
        c.drawCircle(Offset(18, h - 8), 3.4, f);
        c.drawLine(Offset(11, h - 7), Offset(11, 5), st);
        c.drawLine(Offset(21, h - 9), Offset(21, 3), st);
        c.drawLine(Offset(11, 5), Offset(21, 3), st..strokeWidth = 2.6);
      case 'Physics':
        final p = Path()..moveTo(2, h / 2);
        for (var x = 2.0; x <= w - 2; x += 1) {
          p.lineTo(x, h / 2 + math.sin((x - 2) / (w - 4) * math.pi * 2) * 7);
        }
        c.drawPath(p, st);
      case 'Materials':
        final p = Path();
        for (var i = 0; i < 6; i++) {
          final a = math.pi / 6 + i * math.pi / 3;
          final q = Offset(w / 2 + math.cos(a) * 10, h / 2 + math.sin(a) * 10);
          i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
        }
        c.drawPath(p..close(), st);
        c.drawCircle(Offset(w / 2, h / 2), 4, f);
      case 'Environments':
        c.drawPath(Path()..moveTo(1, h - 4)..lineTo(9, 7)..lineTo(14, 14)..lineTo(17, 10)..lineTo(w - 1, h - 4)..close(), f);
      case 'AI':
        for (final o in [const Offset(7, 8), const Offset(17, 7), const Offset(12, 15), const Offset(6, 18), const Offset(19, 17)]) {
          c.drawCircle(o, 2.6, f);
        }
        c.drawLine(const Offset(7, 8), const Offset(12, 15), st..strokeWidth = 1);
        c.drawLine(const Offset(17, 7), const Offset(12, 15), st);
      case 'Templates':
        for (final r in [Rect.fromLTWH(3, 3, 8, 8), Rect.fromLTWH(13, 3, 8, 5), Rect.fromLTWH(3, 13, 8, 8), Rect.fromLTWH(13, 10, 8, 11)]) {
          c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(1.5)), st..strokeWidth = 1.4);
        }
      default:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(5, 8, 14, 12), const Radius.circular(2)), st..strokeWidth = 1.5);
        c.drawLine(const Offset(9, 8), const Offset(9, 3), st);
        c.drawLine(const Offset(15, 8), const Offset(15, 3), st);
    }
  }
  @override
  bool shouldRepaint(_Icon o) => o.n != n;
}
