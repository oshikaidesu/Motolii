// Ways laid over one another: each a card. A press on a shape puts it in place of the card in hand; a shape laid on the
// graph or the flow joins them; a card carried out comes off. Top to bottom each acts on what is above it as its intent
// asks (the maker, 10/31: "そのeaseの意図から考えるべき"): one that moves a value along, or cuts it into steps, takes what
// is above through it, as GSAP's timingEase and Blender's stack; a wave is as large as what is above, as GSAP's
// amplitudeEase; a shake is added to it, as Blender's Noise. To the document they are one way, defined so (way(),
// doc_match.h), so it is kept, undone and listed as any way is. Linear changes nothing, wherever it lies.
import 'package:flutter/widgets.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/ws.dart';

const _over = '×', _space = '~', _turned = 'Reverse:';   // (a way defined in a document is one word)

/// The ways a way is laid from, each as it is said; one not laid: itself.
List<String> easeCards(String said) => [for (final w in said.split(_over)) w.startsWith(_turned) ? w : w.replaceAll(_space, ' ')];

bool easeIsLaid(String said) => said.contains(_over);
bool easeIsTurned(String said) => said.startsWith(_turned);

/// A way turned end for end, as it is said, or turned back (AbsentMotion's "reverse": this document has no such word, so
/// it is a way defined so, as ways laid are).
String easeTurn(String said) => easeIsTurned(said) ? said.substring(_turned.length).replaceAll(_space, ' ') : '$_turned${said.replaceAll(' ', _space)}';

enum _By { through, times, plus }

// (how a card acts on what is above it)
_By _by(String word) => switch ((easeIsTurned(word) ? easeTurn(word) : word).split(' ').first) {
  'Cyclic' => _By.times,
  'Random' => _By.plus,
  _ => _By.through,
};

// (how far cards laid have gone at u, as the document works it out)
String _goes(List<String> cards) {
  var went = "way('${_quoted(cards.first)}', u)";
  for (final c in cards.skip(1)) {
    final way = _quoted(c);
    went = switch (_by(c)) {
      _By.through => "way('$way', $went)",
      _By.times => "($went) * way('$way', u)",
      _By.plus => "($went) + way('$way', u) - u",
    };
  }
  return went;
}

String _quoted(String word) => word.replaceAll("'", "''");

/// The ways a word needs defined before a stretch can go by it: name and how it goes.
List<(String, String)> easeDefines(String word) => [
  for (final c in easeCards(word)) if (easeIsTurned(c)) (c, "a + (b - a) * (1 - way('${easeTurn(c).replaceAll("'", "''")}', 1 - u))"),
  if (easeIsLaid(word)) (word, 'a + (b - a) * (${_goes(easeCards(word))})'),
];

const _steps = 192;   // (a curve drawn large, its peaks whole)
final _worked = <String, List<double>>{};   // (a card's curve, worked out once: all but the one a hand moves stay as they were)

/// A card, as the way it says.
Ease easeCard(String word, Motolii? m) {
  if (easeIsTurned(word)) {
    final base = easeCard(easeTurn(word), m);
    return Ease(EaseKind.bezier, Offset.zero, const Offset(1, 1), word, [for (var i = 0; i <= _steps; i++) 1 - base.at(1 - i / _steps)]);
  }
  final e = heard(word);
  if (!easeOther(e) || m == null) return e;
  if (_worked.length > 256) _worked.clear();
  return Ease(e.kind, e.p1, e.p2, e.name, _worked[word] ??= [for (var i = 0; i <= _steps; i++) m.wayExact(word, i / _steps)]);
}

/// The cards laid as one way; a card left alone: itself.
Ease easeLaid(List<Ease> cards) => cards.length == 1
    ? cards.single
    : Ease(EaseKind.bezier, Offset.zero, const Offset(1, 1), [for (final c in cards) said(c).replaceAll(' ', _space)].join(_over), [
        for (var i = 0; i <= _steps; i++) _went(cards, i / _steps),
      ]);

// (as _goes, for the window to draw)
double _went(List<Ease> cards, double u) {
  var went = cards.first.at(u);
  for (final c in cards.skip(1)) {
    went = switch (_by(said(c))) {
      _By.through => c.at(went),
      _By.times => went * c.at(u),
      _By.plus => went + c.at(u) - u,
    };
  }
  return went;
}

/// What a way is called under the graph.
String easeWord(Ease e) {
  final s = said(e);
  return (easeIsTurned(s) ? easeTurn(s) : easeName(e) ?? 'Bezier').split(' ').first;
}

/// The flow: the ways laid, top to bottom, each acting on what is above it (the maker's arrows, 10/31; Blender's stack); an
/// empty frame under them for the next (Monster Train's empty slots: it says it takes one, and nothing need be learnt);
/// what they make small at its foot. The card pressed is the one the graph shows, its handles in hand (the maker: seen
/// always, the whole has no handles). A card carried up or down it takes that place, a shape let go on a card goes in
/// before it, and one carried off the flow comes off.
class EaseCards extends StatelessWidget {
  const EaseCards({super.key, required this.cards, required this.now, required this.whole, required this.onPress, required this.onOff, required this.onMove, required this.onLay, required this.onHold});
  final List<Ease> cards;
  final int now;
  final Ease? whole;
  final ValueChanged<int> onPress, onOff;
  final void Function(int from, int to) onMove;
  final void Function(Ease shape, int at) onLay;
  final void Function(Ease? shape, int at) onHold;   // (a shape held over a card: seen before it is let go)

  @override
  Widget build(BuildContext context) {
    final side = Size.square(Shape.z(30));
    Widget card(int i, Ease c) => Hov(   // (under the hand, its × : as Cavalry's, Bitwig's and Serum's rows take a modulation off)
      builder: (_, h) => Stack(
        clipBehavior: Clip.none,
        children: [
          SizedBox.fromSize(size: side, child: PresetTile(c, on: i == now, onTap: () => onPress(i), carried: false)),
          if (h && (cards.length > 1 || c != easePreset('Linear')))
            Positioned(
              right: -Shape.s(4),
              top: -Shape.s(3),
              child: Hov(
                key: ValueKey('ease.off.$i'),
                onTap: () => onOff(i),
                builder: (_, over) => Container(
                  width: Shape.z(15),
                  height: Shape.z(15),
                  alignment: Alignment.center,
                  decoration: BoxDecoration(color: over ? Grey.g95 : Grey.g38, shape: BoxShape.circle),
                  child: Text('×', style: T.micro(over ? Grey.g13 : Grey.g95).copyWith(fontWeight: FontWeight.w700, height: 1)),
                ),
              ),
            ),
        ],
      ),
    );
    Widget place(int i, Widget child) => DragTarget<Object>(
      onWillAcceptWithDetails: (d) {
        if (d.data case final Ease e) onHold(e, i);
        return d.data != i;
      },
      onLeave: (d) => d is Ease ? onHold(null, i) : null,
      onAcceptWithDetails: (d) {
        if (d.data case final Ease e) onLay(e, i);
        if (d.data case final int from) onMove(from, i);
      },
      builder: (_, hot, _) => DecoratedBox(
        position: DecorationPosition.foreground,
        decoration: BoxDecoration(borderRadius: BorderRadius.all(Shape.r(5)), border: hot.isEmpty ? null : Border.all(color: WsT.accent, width: 2)),
        child: child,
      ),
    );
    Widget arrow() => CustomPaint(size: Size(side.width, Shape.z(12)), painter: _Arrow(Grey.g56));
    return ColoredBox(
      color: WsT.card,
      child: Column(
        children: [
          Expanded(
            child: SingleChildScrollView(
              padding: EdgeInsets.symmetric(vertical: Shape.s(4)),
              child: Column(
                children: [
            for (final (i, c) in cards.indexed) ...[
              if (i > 0) arrow(),
              cards.length < 2
                  ? card(i, c)
                  : place(i, Draggable<int>(
                      key: ValueKey('ease.card.$i'),
                      data: i,
                      dragAnchorStrategy: pointerDragAnchorStrategy,
                      feedback: easeHeld(card(i, c)),
                      childWhenDragging: Opacity(opacity: .3, child: card(i, c)),
                      onDraggableCanceled: (_, _) => onOff(i),   // (let go on the flow or the graph it is accepted there, and stays)
                      child: card(i, c),
                    )),
            ],
            arrow(),
            CustomPaint(key: const ValueKey('ease.slot'), size: side, painter: _Slot(Grey.g38)),
                ],
              ),
            ),
          ),
          if (whole case final w?) Padding(
            padding: EdgeInsets.only(top: Shape.s(4), bottom: Shape.s(6)),
            child: CustomPaint(key: const ValueKey('ease.whole'), size: side, painter: EaseIcon(w, Grey.g95)),
          ),
        ],
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
