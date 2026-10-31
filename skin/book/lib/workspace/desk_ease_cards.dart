// Ways laid over one another: each a card. A press on a shape puts it in place of the card in hand; a shape laid on the
// graph or the flow joins them; a card carried out comes off. Top to bottom each acts on what is above it as its intent
// asks (the maker, 10/31: "そのeaseの意図から考えるべき"): one that moves a value along, or cuts it into steps, takes what
// is above through it, as GSAP's timingEase and Blender's stack; a wave is as large as what is above, as GSAP's
// amplitudeEase; a shake is added to it, as Blender's Noise. Between two cards the flow says which: an arrow, a ×, a + (the
// maker, 10/31, of a reader who could not tell). To the document they are one way, defined so (way(), doc_match.h), so it is
// kept, undone and listed as any way is. Linear changes nothing that goes through it; a wave under it is as large as it.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/ws.dart';

const _over = '×', _space = '~', _turned = 'Reverse:';   // (a way defined in a document is one word)
const _all = -1;   // (what the tile at the foot of the flow carries: all the cards, EaseShapes.onKeep)

/// The ways a way is laid from, each as it is said; one not laid: itself.
List<String> easeCards(String said) => [for (final w in said.split(_over)) w.startsWith(_turned) || easeIsPoints(w) ? w : w.replaceAll(_space, ' ')];

bool easeIsLaid(String said) => said.contains(_over);

/// A way said by its points, as CSS's linear() (one word: a space in it is said as ~; anime.js works it out: doc_match.h).
bool easeIsPoints(String said) => said.startsWith('linear(');
bool easeIsTurned(String said) => said.startsWith(_turned);

/// A way turned end for end, as it is said, or turned back (AbsentMotion's "reverse": this document has no such word, so
/// it is a way defined so, as ways laid are).
String easeTurn(String said) => easeIsTurned(said) ? said.substring(_turned.length).replaceAll(_space, ' ') : '$_turned${said.replaceAll(' ', _space)}';

enum _By { through, times, plus }

/// Whether a card takes what is above it through it, so that it is walked by that and not by the clock.
bool easeThrough(Ease card) => _by(said(card)) == _By.through;

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
  for (final c in easeCards(word)) ...[
    if (easeIsTurned(c)) (c, "a + (b - a) * (1 - way('${easeTurn(c).replaceAll("'", "''")}', 1 - u))"),
    if (easeIsPoints(c)) (c, "a + (b - a) * anime('${c.replaceAllMapped(RegExp('[^(,)]*$_space[^(,)]*'), (stop) => '"${stop[0]!.replaceAll(_space, ' ')}"')}', u)"),   // (a stop with its place is a string to anime.js's linear())
  ],
  if (easeIsLaid(word)) (word, 'a + (b - a) * (${_goes(easeCards(word))})'),
];

const _steps = 192;   // (a curve drawn large, its peaks whole)
final _worked = <String, List<double>>{};   // (a card's curve, worked out once: all but the one a hand moves stay as they were)

/// A card, as the way it says.
Ease easeCard(String word, Motolii? m) {
  if (easeIsLaid(word)) return easeLaid([for (final c in easeCards(word)) easeCard(c, m)]);
  if (easeIsTurned(word)) {
    final base = easeCard(easeTurn(word), m);
    return Ease(EaseKind.bezier, Offset.zero, const Offset(1, 1), word, [for (var i = 0; i <= _steps; i++) 1 - base.at(1 - i / _steps)]);
  }
  final e = heard(word);
  if (!easeOther(e) || m == null) return e;
  if (easeIsPoints(word) && !_worked.containsKey(word) && m.wayExact(word, 1) < .5) return e;   // (not defined yet, so it ends nowhere: drawn when it is)
  if (_worked.length > 256) _worked.clear();
  return Ease(e.kind, e.p1, e.p2, e.name, _worked[word] ??= [for (var i = 0; i <= _steps; i++) m.wayExact(word, i / _steps)]);
}

/// A way's curve at so many steps, for a bar: one laid or turned is worked out here, not asked of the inside, whose answer
/// waits on its hand, for each step.
List<double> easeShape(String way, Motolii m, int steps) {
  if (!easeIsLaid(way) && !easeIsTurned(way)) return [for (var i = 0; i <= steps; i++) m.wayAt(way, i / steps)];
  final cards = [for (final c in easeCards(way)) easeCard(c, m)];
  return [for (var i = 0; i <= steps; i++) _went(cards, i / steps)];
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
      _By.through => went <= 0 || went >= 1 ? went : c.at(went),   // (past its ends as it was: an overshoot above it stays one)
      _By.times => went * c.at(u),
      _By.plus => went + c.at(u) - u,
    };
  }
  return went;
}

/// What a way is called under the graph.
String easeWord(Ease e) {
  final s = said(e);
  return easeIsPoints(s) ? 'linear()' : (easeIsTurned(s) ? easeTurn(s) : easeName(e) ?? 'Bezier').split(' ').first;
}

/// A way as words anyone can read: a bezier as CSS says it, so it can be pasted into a page too; any other as it is said.
String easeText(Ease e) => easeIsBezierAlone(e) ? 'cubic-bezier(${e.p1.dx}, ${e.p1.dy}, ${e.p2.dx}, ${e.p2.dy})' : said(e);

bool easeIsBezierAlone(Ease e) => said(e).startsWith('Bezier ');

/// A way out of copied words: CSS's cubic-bezier() and linear() (CSS Easing Functions 2), wherever they stand in what was
/// copied (a line of a page's style, cubic-bezier.com), or as [easeText] said it. None: no way in them.
Ease? easeFromText(String? text, Motolii? m) {
  final t = (text ?? '').trim();
  if (RegExp(r'cubic-bezier\(([^)]*)\)').firstMatch(t) case final b?) {
    final n = [for (final x in b[1]!.split(',')) double.tryParse(x.trim())];
    if (n.length != 4 || n.contains(null) || n[0]! < 0 || n[0]! > 1 || n[2]! < 0 || n[2]! > 1) return null;   // (CSS's own rule: time goes only forward)
    return Ease(EaseKind.bezier, Offset(n[0]!, n[1]!), Offset(n[2]!, n[3]!));
  }
  if (RegExp(r'linear\(([^)]*)\)').firstMatch(t) case final l?) {
    final stops = [for (final s in l[1]!.split(',')) s.trim().replaceAll(RegExp(r'\s+'), _space)];
    return stops.length < 2 ? null : heard('linear(${stops.join(',')})');
  }
  return const {'Linear', 'Hold', 'Bezier', 'Bounce', 'Elastic', 'Cyclic', 'Random', 'Steps', 'ElasticSteps'}.contains(t.split(RegExp('[ ~×]')).first) || easeIsTurned(t) ? easeCard(t, m) : null;
}

/// The flow: the ways laid, top to bottom, each acting on what is above it (the maker's arrows, 10/31; Blender's stack); an
/// empty frame under them for the next (Monster Train's empty slots: it says it takes one, and nothing need be learnt);
/// what they make small at its foot. The card pressed is the one the graph shows, its handles in hand (the maker: seen
/// always, the whole has no handles); that at the foot pressed, the graph shows the whole. A card carried up or down it takes that place, a shape let go on a card goes in
/// before it, and one carried off the flow comes off.
class EaseCards extends StatelessWidget {
  const EaseCards({super.key, this.adding = false, this.onSlot, required this.cards, required this.now, required this.whole, required this.onWhole, required this.onPress, required this.onOff, required this.onMove, required this.onLay, required this.onHold, required this.side, this.across = false});
  final double side;
  final bool across;   // (laid across, under the graph, in a tall pane)
  final bool adding;   // (the empty frame pressed: it waits for what is picked next)
  final VoidCallback? onSlot;
  final List<Ease> cards;
  final int now;
  final Ease? whole;
  final VoidCallback onWhole;
  final ValueChanged<int> onPress, onOff;
  final void Function(int from, int to) onMove;
  final void Function(Ease shape, int at) onLay;
  final void Function(Ease? shape, int at) onHold;   // (a shape held over a card: seen before it is let go)

  @override
  Widget build(BuildContext context) {
    final side = Size.square(this.side);
    Widget card(int i, Ease c) => Hov(   // (under the hand, its × : as Cavalry's, Bitwig's and Serum's rows take a modulation off)
      builder: (_, h) => Stack(
        clipBehavior: Clip.none,
        children: [
          SizedBox.fromSize(size: side, child: PresetTile(c, on: i == now, side: this.side, onTap: () => onPress(i), carried: false)),
          if (h && (cards.length > 1 || c != easePreset('Linear')))
            Positioned(right: -Shape.s(4), top: -Shape.s(3), child: EaseOff(key: ValueKey('ease.off.$i'), onTap: () => onOff(i))),
        ],
      ),
    );
    Widget place(int i, Widget child) => DragTarget<Object>(
      onWillAcceptWithDetails: (d) {
        if (d.data case final Ease e) onHold(e, i);
        return d.data != i && d.data != _all;
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
    Widget carried(int i, Ease c) => Draggable<int>(
      key: ValueKey('ease.card.$i'),
      data: i,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: easeHeld(card(i, c), this.side),
      childWhenDragging: Opacity(opacity: .3, child: card(i, c)),
      onDraggableCanceled: (_, _) => onOff(i),   // (let go on the flow or the graph it is accepted there, and stays; on the shapes it is kept, and stays)
      child: card(i, c),
    );
    final axis = across ? Axis.horizontal : Axis.vertical;
    final gap = across ? Size(this.side * .4, side.height) : Size(side.width, this.side * .4);
    Widget arrow() => CustomPaint(size: gap, painter: _Arrow(Grey.g56, across));
    Widget by(int i) => switch (_by(said(cards[i]))) {   // (how it takes what is above it)
      _By.through => arrow(),
      final how => SizedBox.fromSize(key: ValueKey('ease.by.$i'), size: gap, child: Center(child: Text(how == _By.times ? '×' : '+', style: T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700, height: 1)))),
    };
    return ColoredBox(
      color: WsT.card,
      child: Flex(
        direction: axis,
        children: [
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: axis,
              padding: across ? EdgeInsets.symmetric(horizontal: Shape.s(4)) : EdgeInsets.symmetric(vertical: Shape.s(4)),
              child: Flex(
                direction: axis,
                mainAxisSize: MainAxisSize.min,
                children: [
            for (final (i, c) in cards.indexed) ...[
              if (i > 0) by(i),
              if (cards.length > 1) place(i, carried(i, c)) else if (c == easePreset('Linear')) card(i, c) else carried(i, c),   // (a straight line alone: nothing to carry)
            ],
            arrow(),
            Hov(key: const ValueKey('ease.slot'), onTap: onSlot, builder: (_, h) => EaseSlot(side: this.side, ink: adding ? WsT.accent : h ? Grey.g76 : Grey.g38)),
                ],
              ),
            ),
          ),
          if (whole case final w?) Padding(
            padding: across ? EdgeInsets.only(left: Shape.s(4), right: Shape.s(6)) : EdgeInsets.only(top: Shape.s(4), bottom: Shape.s(6)),
            child: Draggable<int>(
              data: _all,
              dragAnchorStrategy: pointerDragAnchorStrategy,
              feedback: easeHeld(PresetTile(w, on: false, side: this.side, onTap: () {}, carried: false), this.side),
              child: SizedBox.fromSize(size: side, child: PresetTile(w, key: const ValueKey('ease.whole'), on: now < 0, side: this.side, onTap: onWhole, carried: false)),
            ),
          ),
        ],
      ),
    );
  }
}

/// A round × that takes a thing off (Cavalry's, Bitwig's and Serum's rows).
class EaseOff extends StatelessWidget {
  const EaseOff({super.key, required this.onTap});
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    builder: (_, over) => Container(
      width: Shape.z(15),
      height: Shape.z(15),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: over ? Grey.g95 : Grey.g38, shape: BoxShape.circle),
      child: Text('×', style: T.micro(over ? Grey.g13 : Grey.g95).copyWith(fontWeight: FontWeight.w700, height: 1)),
    ),
  );
}

class _Arrow extends CustomPainter {
  const _Arrow(this.ink, this.across);
  final Color ink;
  final bool across;
  @override
  void paint(Canvas c, Size s) {
    if (across) {   // (the same arrow, turned to point along the row)
      c.translate(0, s.height);
      c.rotate(-math.pi / 2);
      s = Size(s.height, s.width);
    }
    final x = s.width / 2, p = Paint()..color = ink..strokeWidth = 1.4..style = PaintingStyle.stroke..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    c.drawLine(Offset(x, 1), Offset(x, s.height - 2), p);
    c.drawPath(Path()..moveTo(x - 3, s.height - 5)..lineTo(x, s.height - 2)..lineTo(x + 3, s.height - 5), p);
  }

  @override
  bool shouldRepaint(_Arrow o) => o.ink != ink || o.across != across;
}

/// An empty frame, dashed: it says it takes one (Monster Train's empty slots).
class EaseSlot extends StatelessWidget {
  const EaseSlot({super.key, required this.side, required this.ink});
  final double side;
  final Color ink;
  @override
  Widget build(BuildContext context) => CustomPaint(size: Size.square(side), painter: _Slot(ink));
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
