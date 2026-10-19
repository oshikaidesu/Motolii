// The Copies card's drawing, in the Transform card's manner (a control is a drawing of its own result, pulled by hand):
// the thing and its copies as they lie, so many as there are, as far apart as they are, turned and grown each one more;
// the + at the end of the row adds or takes away copies, the knob on the second one pulls them apart.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

class CopiesDrawing extends StatefulWidget {
  const CopiesDrawing({super.key});
  static const height = 84.0;
  @override
  State<CopiesDrawing> createState() => _CopiesDrawingState();
}

enum _Knob { count, gap }

class _CopiesDrawingState extends State<CopiesDrawing> {
  _Knob? _held;

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    final card = doc.cards.where((c) => c.name == 'Copies').firstOrNull;
    Param? param(String id) => card?.params.where((p) => p.id == id).firstOrNull;


    return Padding(
      padding: EdgeInsets.only(bottom: Shape.s(6)),
      child: SizedBox(
        height: CopiesDrawing.height,
        child: LayoutBuilder(
          builder: (context, box) {
            final g = copiesRowOf(doc, box.biggest), count = g.count;
            _Knob? pick(Offset p) => (p - g.countKnob).distance <= 12 ? _Knob.count : (count > 1 && (p - g.gapKnob).distance <= 12 ? _Knob.gap : null);
            return GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanStart: (d) {
                _held = pick(d.localPosition);
                if (_held != null) doc.beginGesture();
              },
              onPanUpdate: (d) {
                switch (_held) {
                  case _Knob.count:
                    doc.set('v_copies', g.countAt(d.localPosition).toDouble());
                  case _Knob.gap:
                    if (param('copy_x')?.max case final top?) doc.set('v_copy_x', g.spreadAt(d.localPosition) * top);
                  case null:
                }
              },
              onPanEnd: (_) {
                if (_held != null) doc.endGesture();
                setState(() => _held = null);
              },
              child: CustomPaint(size: box.biggest, painter: _CopiesPaint(g, _held)),
            );
          },
        ),
      ),
    );
  }
}

/// The copies as the drawing lays them out for this thing (and where its two knobs are: the + adds, the other spreads).
CopiesRow copiesRowOf(Doc doc, Size size) {
  final p = doc.cards.where((c) => c.name == 'Copies').firstOrNull?.params.where((p) => p.id == 'copy_x').firstOrNull, v = doc.get('v_copy_x') ?? 0;
  final spread = p?.max == null || p!.max == 0 ? 0.0 : (v / p.max!).clamp(-1.0, 1.0); // (how far apart, -1 to 1 of the knob's span)
  return CopiesRow(size, count: (doc.get('v_copies') ?? 1).round().clamp(1, 200), spread: spread, turn: doc.get('v_copy_rotate') ?? 0, grow: doc.get('v_copy_size') ?? 1);
}

/// Where the copies lie in the drawing: one cell a copy, so many as fit; past that, the row says how many more.
class CopiesRow {
  CopiesRow(this.size, {required this.count, required this.spread, required this.turn, required this.grow});
  final Size size;
  final int count;
  final double spread, turn, grow;
  static const cell = 16.0, most = 24.0; // (a copy's square; the most a copy stands from the one before)
  double get way => spread < 0 ? -1 : 1; // (copies to the right, or to the left when the gap is less than nought)
  Offset get first => Offset(spread < 0 ? size.width - 60 : 20, size.height / 2);
  double get step => 3 + (most - 3) * spread.abs(); // (all in one place: just showing they are many)
  int get shown => math.min(count, ((size.width - 70) / math.max(step, 3)).floor().clamp(1, 60));
  Offset at(int i) => first + Offset(way * i * step, 0);
  Offset get countKnob => at(shown - 1) + Offset(way * (cell / 2 + 16), 0);
  Offset get gapKnob => at(1) + const Offset(0, -cell / 2 - 7);
  int countAt(Offset p) => (1 + (way * (p.dx - first.dx) / math.max(step, 6)).round()).clamp(1, 200);
  double spreadAt(Offset p) => (way * (way * (p.dx - first.dx) - 3) / (most - 3)).clamp(-1.0, 1.0);
}

class _CopiesPaint extends CustomPainter {
  const _CopiesPaint(this.g, this.held);
  final CopiesRow g;
  final _Knob? held;

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, Shape.r(8)), Paint()..color = Grey.g07);
    for (var i = g.shown - 1; i >= 0; i--) { // (the thing first, its copies after, each turned and grown one more)
      final at = g.at(i), k = math.pow(g.grow, i).toDouble().clamp(.3, 2.0);
      c.save();
      c.translate(at.dx, at.dy);
      c.rotate(g.turn * i * math.pi / 180);
      final r = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: CopiesRow.cell * k, height: CopiesRow.cell * k), Shape.r(3));
      c.drawRRect(r, Paint()..color = (i == 0 ? Pop.tonePlace : Pop.tonePlace.withValues(alpha: .55)));
      c.drawRRect(r, Paint()
        ..color = Grey.g07
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1);
      c.restore();
    }
    if (g.shown < g.count) _dots(c, g.at(g.shown - 1) + Offset(g.way * (CopiesRow.cell / 2 + 5), 0)); // (more than fit)
    // (the + adds a copy: pulled right, more; left, fewer)
    final plus = g.countKnob, ink = Paint()
      ..color = Grey.g95
      ..strokeWidth = 1.6
      ..strokeCap = StrokeCap.round;
    c.drawCircle(plus, held == _Knob.count ? 9 : 8, Paint()..color = held == _Knob.count ? Pop.accent : Grey.g26);
    c.drawLine(plus - const Offset(4, 0), plus + const Offset(4, 0), ink);
    c.drawLine(plus - const Offset(0, 4), plus + const Offset(0, 4), ink);
    if (g.count > 1) { // (the second copy's knob: pulled, the copies stand further apart, or closer)
      final k = g.gapKnob;
      c.drawLine(k + const Offset(-8, 0), k + const Offset(8, 0), ink..color = Grey.g56);
      c.drawCircle(k, held == _Knob.gap ? 6.5 : 5, Paint()..color = held == _Knob.gap ? Pop.accent : const Color(0xFF8FB8FF));
    }
  }

  void _dots(Canvas c, Offset at) {
    for (var i = 0; i < 3; i++) {
      c.drawCircle(at + Offset(i * 4.0, 0), 1.2, Paint()..color = Grey.g56);
    }
  }

  @override
  bool shouldRepaint(_CopiesPaint o) => true;
}
