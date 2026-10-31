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
  return easeOther(e) && m != null ? Ease(e.kind, e.p1, e.p2, e.name, [for (var i = 0; i <= 48; i++) m.wayAt(word, i / 48)]) : e;
}

/// The cards laid as one way; a card left alone: itself.
Ease easeLaid(List<Ease> cards) => cards.length == 1
    ? cards.single
    : Ease(EaseKind.bezier, Offset.zero, const Offset(1, 1), [for (final c in cards) said(c).replaceAll(' ', _space)].join(_over), [
        for (var i = 0; i <= 48; i++) cards.fold<double>(1, (v, c) => v * c.at(i / 48)),
      ]);

String _word(Ease e) => easeName(e)?.split(' ').first ?? 'Bezier';

/// What a way is called on its tile: those it is laid from, each by its name.
String easeTitle(Ease e) {
  final s = said(e);
  return easeIsLaid(s) ? [for (final c in easeCards(s)) _word(heard(c))].join(' $_over ') : e.name ?? '';
}

/// The cards of a way laid from more than one, the top edge of each seen behind the next and the graph the card in
/// front, as Apple Wallet stacks its cards. One carried off the stack comes off.
class EaseCards extends StatelessWidget {
  const EaseCards({super.key, required this.cards, required this.onOff});
  final List<Ease> cards;
  final ValueChanged<int> onOff;

  @override
  Widget build(BuildContext context) {
    final high = (Shape.z(44) / cards.length).clamp(Shape.z(14), Shape.z(24));   // (more cards, less of each: the graph keeps its room)
    Widget edge(int i, Ease c) => Container(
      height: high,
      margin: EdgeInsets.symmetric(horizontal: Shape.s(4.0 * (cards.length - i))),
      padding: EdgeInsets.symmetric(horizontal: Shape.s(6)),
      decoration: BoxDecoration(color: WsT.raised, borderRadius: BorderRadius.vertical(top: Shape.r(6)), border: Border(top: BorderSide(color: Grey.g38))),
      child: Row(
        children: [
          if (high >= Shape.z(18)) ...[
            SizedBox(width: Shape.z(30), height: Shape.z(16), child: CustomPaint(painter: EaseThumb(c, Grey.g91, Grey.g38))),
            SizedBox(width: Shape.s(6)),
          ],
          Text(_word(c), maxLines: 1, style: T.micro(Grey.g91).copyWith(fontSize: T.px(10), fontWeight: FontWeight.w600)),
        ],
      ),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (i, c) in cards.indexed)
          Draggable<int>(
            key: ValueKey('ease.card.$i'),
            data: i,
            dragAnchorStrategy: pointerDragAnchorStrategy,
            feedback: easeHeld(PresetTile(c, on: false, onTap: () {}, label: _word(c), carried: false)),
            childWhenDragging: Opacity(opacity: .3, child: edge(i, c)),
            onDraggableCanceled: (_, _) => onOff(i),   // (let go on the stack it is accepted there, and stays)
            child: edge(i, c),
          ),
      ],
    );
  }
}
