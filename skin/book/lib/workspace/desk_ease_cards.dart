// Ways laid over one another: each a card. A press on a tile puts it in place of them all; a tile laid on the graph
// multiplies them (Blender's F-Curve Noise modifier's Multiply; GSAP's CustomWiggle's amplitudeEase); a card carried out
// comes off. To the document they are one way, defined as they multiply (way(), doc_match.h), so it is kept, undone and
// listed as any way is.
import 'package:flutter/widgets.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/ws.dart';

const _over = '×', _space = '~';   // (a way defined in a document is one word)

/// The ways a way is laid from, each as it is said; one not laid: itself.
List<String> easeCards(String said) => [for (final w in said.split(_over)) w.replaceAll(_space, ' ')];

bool easeIsLaid(String said) => said.contains(_over);

/// How a way laid from [cards] goes, as a way defined in a document says it.
String easeGoes(List<String> cards) => 'a + (b - a) * ${[for (final c in cards) "way('${c.replaceAll("'", "''")}', u)"].join(' * ')}';

/// A card, as the way it says.
Ease easeCard(String word, Motolii? m) {
  final e = heard(word);
  return easeOther(e) && m != null ? Ease(e.kind, e.p1, e.p2, e.name, [for (var i = 0; i <= 96; i++) m.wayAt(word, i / 96)]) : e;
}

/// The cards laid as one way; a card left alone: itself.
Ease easeLaid(List<Ease> cards) => cards.length == 1
    ? cards.single
    : Ease(EaseKind.bezier, Offset.zero, const Offset(1, 1), [for (final c in cards) said(c).replaceAll(' ', _space)].join(_over), [
        for (var i = 0; i <= 96; i++) cards.fold<double>(1, (v, c) => v * c.at(i / 96)),
      ]);

/// The flow: the ways laid, top to bottom, each multiplied by the next (the maker's arrows, 10/31; Blender's stack), what
/// they make the graph beside them; an empty frame under them for the next (Monster Train's empty slots: it says it takes
/// one, and nothing need be learnt). A card carried off the flow comes off.
class EaseCards extends StatelessWidget {
  const EaseCards({super.key, required this.cards, required this.onOff});
  final List<Ease> cards;
  final ValueChanged<int> onOff;

  @override
  Widget build(BuildContext context) {
    final side = Size.square(Shape.z(30));
    Widget card(Ease c) => SizedBox.fromSize(size: side, child: PresetTile(c, on: true, onTap: () {}, carried: false));
    Widget arrow() => CustomPaint(size: Size(side.width, Shape.z(12)), painter: _Arrow(Grey.g56));
    return ColoredBox(
      color: WsT.card,
      child: SingleChildScrollView(
        padding: EdgeInsets.symmetric(vertical: Shape.s(4)),
        child: Column(
          children: [
            for (final (i, c) in cards.indexed) ...[
              if (i > 0) arrow(),
              cards.length < 2
                  ? card(c)
                  : Draggable<int>(
                      key: ValueKey('ease.card.$i'),
                      data: i,
                      dragAnchorStrategy: pointerDragAnchorStrategy,
                      feedback: easeHeld(card(c)),
                      childWhenDragging: Opacity(opacity: .3, child: card(c)),
                      onDraggableCanceled: (_, _) => onOff(i),   // (let go on the flow or the graph it is accepted there, and stays)
                      child: card(c),
                    ),
            ],
            arrow(),
            CustomPaint(key: const ValueKey('ease.slot'), size: side, painter: _Slot(Grey.g38)),
          ],
        ),
      ),
    );
  }
}

class _Arrow extends CustomPainter {
  const _Arrow(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final x = s.width / 2, p = Paint()..color = ink..strokeWidth = 1.4..style = PaintingStyle.stroke..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    c.drawLine(Offset(x, 1), Offset(x, s.height - 2), p);
    c.drawPath(Path()..moveTo(x - 3, s.height - 5)..lineTo(x, s.height - 2)..lineTo(x + 3, s.height - 5), p);
  }

  @override
  bool shouldRepaint(_Arrow o) => o.ink != ink;
}

class _Slot extends CustomPainter {
  const _Slot(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final k = s.width / 68, box = RRect.fromRectAndRadius(Rect.fromLTWH(3 * k, 4 * k, 61 * k, 60 * k), Radius.circular(8 * k));
    final line = Paint()..color = ink..strokeWidth = 1.2..style = PaintingStyle.stroke;
    for (final m in (Path()..addRRect(box)).computeMetrics()) {
      for (var d = 0.0; d < m.length; d += 7) {
        c.drawPath(m.extractPath(d, d + 4), line);
      }
    }
  }

  @override
  bool shouldRepaint(_Slot o) => o.ink != ink;
}
