// What stands beside the graph, as AbsentMotion's EasingPanel.kt lays it, Apache-2.0: LeftColumn (reverse and ⋯), Presets
// and Families, the two narrow rows of shapes, none named. Its layout and its moves, in this window's own colours.
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects.dart' show CardMore;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

/// Alight Motion's three families and the shapes of each (AbsentMotion's EasingCatalog); Hold, After Effects' and
/// Ableton's, is not its own and stands last.
const easeFamilies = [
  ['Linear', 'In', 'Out', 'In-out'],
  ['Bounce', 'Elastic', 'Cyclic', 'Random'],
  ['Steps', 'ElasticSteps', 'Hold'],
];

int easeFamilyOf(Ease e) {
  final word = said(e).split(' ').first;
  if (word == 'Bezier') return 0;
  final i = easeFamilies.indexWhere((f) => f.contains(word));
  return i < 0 ? easeFamilies.length : i;   // (past the three: one of the window's or the document's own)
}

/// The shapes of a family; past the three, every way that has no seat among them (a maker's own, one laid from cards).
List<Ease> easesOfFamily(int i, List<Ease> ways) {
  Ease? named(String n) => easePresets.where((p) => p.name == n).firstOrNull ?? ways.where((w) => w.name == n).firstOrNull;
  if (i < easeFamilies.length) return [for (final n in easeFamilies[i]) ?named(n)];
  final seated = {for (final f in easeFamilies) ...f};
  return [for (final e in [...easePresets, ...ways]) if (!seated.contains(e.name)) e];
}

bool _lit(Ease shape, Ease now) => easeOther(shape) ? said(shape).split(' ').first == said(now).split(' ').first : shape == now && !easeOther(now);

/// The shapes of the family the way is of: a press puts one on the stretch.
class EasePresets extends StatelessWidget {
  const EasePresets({super.key, required this.ease, required this.shapes, required this.onPick});
  final Ease ease;
  final List<Ease> shapes;
  final ValueChanged<Ease> onPick;

  Widget _tile(Ease e) => Padding(
    padding: EdgeInsets.zero,
    child: PresetTile(e, key: ValueKey('ease.${e.name}'), on: _lit(e, ease), label: '', onTap: () => _lit(e, ease) ? null : onPick(e)),   // (the one it is already: its own numbers kept)
  );

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: WsT.card,
    child: shapes.length <= 4
        ? Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [for (final e in shapes) Expanded(child: _tile(e))])
        : LayoutBuilder(builder: (_, box) => ListView(itemExtent: box.maxHeight / 4, children: [for (final e in shapes) _tile(e)])),   // (more than a row holds: scrolled)
  );
}

/// The families: a press on another puts its first shape on the stretch; a notch points at the shapes of the one chosen.
class EaseFamilies extends StatelessWidget {
  const EaseFamilies({super.key, required this.family, required this.ways, required this.onPick});
  final int family;
  final List<Ease> ways;
  final ValueChanged<Ease> onPick;

  @override
  Widget build(BuildContext context) {
    final rows = [for (var i = 0; i <= easeFamilies.length; i++) if (easesOfFamily(i, ways).isNotEmpty) i];
    return CustomPaint(
      painter: _Notch(rows.indexOf(family), rows.length),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final i in rows)
            Expanded(
              child: Padding(
                padding: EdgeInsets.only(left: Shape.s(4)),   // (past the notch)
                child: PresetTile(
                  easesOfFamily(i, ways)[i == 0 ? 3 : 0],
                  key: ValueKey('ease.family.$i'),
                  on: i == family,
                  label: '',
                  onTap: () => i == family ? null : onPick(easesOfFamily(i, ways).first),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _Notch extends CustomPainter {
  const _Notch(this.at, this.of);
  final int at, of;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = WsT.body);
    final y = (at + .5) * s.height / of, k = s.width / 168;
    c.drawPath(Path()..moveTo(0, y - 17 * k)..lineTo(0, y + 17 * k)..lineTo(18 * k, y)..close(), Paint()..color = WsT.card);
  }

  @override
  bool shouldRepaint(_Notch o) => o.at != at || o.of != of;
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
        Expanded(child: Center(child: CardMore(name: 'Ease', word: '⋯', above: true, items: menu))),
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
