// What stands beside the graph, as AbsentMotion's EasingPanel.kt lays it, Apache-2.0: LeftColumn (reverse and ⋯) and a
// narrow row of shapes, none named. Its layout and its moves, in this window's own colours.
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects.dart' show CardMore;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart' show easeIsTurned, easeTurn;
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

/// The shapes, one row, one press each (the maker, 10/31: with ways multiplied the second row of AbsentMotion's panel is
/// not needed). Alight Motion's, its beziers one straight line: its two handles make any of them.
const easeShapes = ['Linear', 'Bounce', 'Elastic', 'Cyclic', 'Random', 'Steps', 'ElasticSteps'];

bool easeIsBezier(Ease e) => const {'Linear', 'Bezier'}.contains(said(e).split(' ').first);

/// The row: the shapes, then, scrolled to, what has no seat among them: Hold, and the ways of the window and the document.
List<Ease> easeRow(List<Ease> ways) {
  Ease? named(String n) => easePresets.where((p) => p.name == n).firstOrNull ?? ways.where((w) => w.name == n).firstOrNull;
  return [for (final n in easeShapes) ?named(n), easePreset('Hold'), for (final w in ways) if (!easeShapes.contains(w.name)) w];
}

/// Whether the way now is this shape: a curve pulled from the straight line is still of it; its numbers its own.
bool _isOf(Ease shape, Ease now) {
  final word = said(now), base = easeIsTurned(word) ? easeTurn(word) : word;
  return easeIsBezier(shape) ? easeIsBezier(now) : said(shape).split(' ').first == base.split(' ').first;
}

/// The row beside the graph: a press puts a shape on the stretch, or a curved line back straight.
class EaseShapes extends StatelessWidget {
  const EaseShapes({super.key, required this.ease, required this.shapes, required this.side, required this.onPick, this.across = false});
  final Ease ease;
  final double side;
  final bool across;   // (laid across, under the graph, in a tall pane)
  final List<Ease> shapes;
  final ValueChanged<Ease> onPick;

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: WsT.card,
    child: LayoutBuilder(
      builder: (_, box) => ListView(
        scrollDirection: across ? Axis.horizontal : Axis.vertical,
        itemExtent: (across ? box.maxWidth : box.maxHeight) / (easeShapes.length + .45),   // (the next cut by the edge: it says there is more)
        children: [
          for (final e in shapes)
            PresetTile(e, key: ValueKey('ease.${e.name}'), on: _isOf(e, ease), side: side, onTap: () => _isOf(e, ease) && !(easeIsBezier(e) && e != ease) ? null : onPick(e)),   // (the one it is already: its own numbers kept)
        ],
      ),
    ),
  );
}

/// The left column: reverse, and under ⋯ the curve copied and pasted (Alight Motion's Copy Curve, Paste Curve, Paste
/// Curve to All Keyframes, Enable Overshoot), and the maker's own Jump (desk_ease_state.dart easeJump).
class EaseLeft extends StatelessWidget {
  const EaseLeft({super.key, this.onReverse, required this.menu});
  final VoidCallback? onReverse;   // (none: the way cannot be turned)
  final List<(String, String, VoidCallback)> menu;

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: WsT.card,
    child: Column(
      children: [
        const Spacer(flex: 2),
        Expanded(
          child: Center(
            child: Hov(
              key: const ValueKey('ease-reverse'),
              onTap: onReverse,
              builder: (_, h) => CustomPaint(size: Size.square(Shape.z(18)), painter: _Reverse(onReverse == null ? Grey.g38 : (h ? Grey.g95 : Grey.g76))),
            ),
          ),
        ),
        Expanded(child: Center(child: CardMore(name: 'Ease', word: '⋯', above: true, big: true, items: menu))),
      ],
    ),
  );
}

class _Reverse extends CustomPainter {   // (AbsentMotion's EasingIcons.kt REVERSE_ARROWS)
  const _Reverse(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final k = s.width / 24;
    c.scale(k);
    c.drawPath(
      Path()
        ..moveTo(16, 3)..lineTo(20, 7)..lineTo(16, 11)
        ..moveTo(20, 7)..lineTo(4, 7)
        ..moveTo(8, 21)..lineTo(4, 17)..lineTo(8, 13)
        ..moveTo(4, 17)..lineTo(20, 17),
      Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 1.9..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(_Reverse o) => o.ink != ink;
}

/// The arrow to the curve before or after, in a bottom corner of the graph (AbsentMotion's StretchButton).
class EaseStep extends StatelessWidget {
  const EaseStep({super.key, required this.span, required this.back});
  final EaseSpan? span;
  final bool back;

  @override
  Widget build(BuildContext context) {
    final s = span, ws = WsScope.of(context, {WsAspect.document});
    final keys = s == null ? const <int>[] : ([...s.layer.keys]..sort());
    final to = s == null ? null : back ? keys.where((f) => f < s.from).lastOrNull : (keys.any((f) => f > s.to) ? s.to : null);
    return to == null
        ? const SizedBox.shrink()
        : DeskIconButton(key: ValueKey(back ? 'ease-previous' : 'ease-next'), size: Shape.z(18), glyph: (ink) => _Step(ink, back), onTap: () => ws.pickKey((layer: s!.layer.id, frame: to)));
  }
}

class _Step extends CustomPainter {
  const _Step(this.ink, this.back);
  final Color ink;
  final bool back;
  @override
  void paint(Canvas c, Size s) {
    final x = back ? s.width * .6 : s.width * .4, d = back ? -s.width * .2 : s.width * .2;
    c.drawPath(Path()..moveTo(x, s.height * .25)..lineTo(x + d, s.height / 2)..lineTo(x, s.height * .75), Paint()..color = ink..strokeWidth = 1.4..style = PaintingStyle.stroke..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  }

  @override
  bool shouldRepaint(_Step o) => o.ink != ink || o.back != back;
}
