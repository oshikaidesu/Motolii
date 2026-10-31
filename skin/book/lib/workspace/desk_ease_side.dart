// What stands beside the graph, as AbsentMotion's EasingPanel.kt lays it, Apache-2.0: LeftColumn (reverse and ⋯) and a
// narrow row of shapes, none named. Its layout and its moves, in this window's own colours.
import 'dart:convert';
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects.dart' show CardMore;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart' show EaseOff, easeIsTurned, easeTurn;
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

/// The shapes, one row, one press each (the maker, 10/31: with ways multiplied the second row of AbsentMotion's panel is
/// not needed). Alight Motion's, in the order of AbsentMotion's EasingCatalog.families, its four beziers first.
const easeShapes = ['Linear', 'In', 'Out', 'In-out', 'Bounce', 'Elastic', 'Cyclic', 'Random', 'Steps', 'ElasticSteps'];

/// How many a row of that length shows: all that stay a good size, seven at the least, and part of the next, cut by the
/// edge: it says there is more.
double easeFits(double length) => (length / Shape.z(34)).floorToDouble().clamp(7, easeShapes.length) + .45;

/// What a hand kept, each as it is said (a curve, a card, all the cards): carried to the shapes it stays there, as a device
/// carried to Live's browser by its title bar is saved (Live's manual, 23.2.5 Saving Presets). In ~/.motolii, as the
/// colours are in colors.json.
File _keptFile = File('${Platform.environment['HOME']}/.motolii/ease_kept.json');
List<String>? _kept;

/// (a test points it elsewhere, and forgets what was read: as coloursAt)
void easeKeptAt(File f) => (_keptFile = f, _kept = null);

List<String> easeKept() => _kept ??= () {
  try {
    return [for (final w in jsonDecode(_keptFile.readAsStringSync()) as List) '$w'];
  } catch (_) {
    return <String>[];
  }
}();

void easeKeep(String word, {bool off = false}) {
  final kept = easeKept()..remove(word);
  if (!off) kept.add(word);
  try {
    _keptFile.parent.createSync(recursive: true);
    _keptFile.writeAsStringSync(jsonEncode(kept));
  } catch (_) {}   // (not kept: known while the window is open)
}

bool easeIsBezier(Ease e) => const {'Linear', 'Bezier'}.contains(said(e).split(' ').first);

/// The row: the shapes, then, scrolled to, what has no seat among them: Hold, and the ways of the window and the document.
List<Ease> easeRow(List<Ease> ways) {
  Ease? named(String n) => easePresets.where((p) => p.name == n).firstOrNull ?? ways.where((w) => w.name == n).firstOrNull;
  return [for (final n in easeShapes) ?named(n), easePreset('Hold'), for (final w in ways) if (!easeShapes.contains(w.name)) w];
}

/// Whether the way now is this shape: AbsentMotion's EasingCatalog.matches, a bezier by its numbers, any other by its kind.
bool _isOf(Ease shape, Ease now) {
  final word = said(now), base = easeIsTurned(word) ? easeTurn(word) : word;
  return easeIsBezier(shape) ? shape == now : said(shape).split(' ').first == base.split(' ').first;
}

/// The row beside the graph: a press puts a shape on the stretch, or a curved line back straight; what was carried to it
/// stays at its foot, and comes off by its ×, as a card of the flow does.
class EaseShapes extends StatefulWidget {
  const EaseShapes({super.key, required this.ease, required this.shapes, required this.kept, required this.side, required this.onPick, required this.onKeep, required this.onForget, this.across = false});
  final Ease ease;
  final double side;
  final bool across;   // (laid across, under the graph, in a tall pane)
  final List<Ease> shapes, kept;
  final ValueChanged<Ease> onPick, onForget;
  final ValueChanged<int> onKeep;   // (a card of the flow carried here: its place; all of them: below 0)

  @override
  State<EaseShapes> createState() => _EaseShapesState();
}

class _EaseShapesState extends State<EaseShapes> {
  final _scroll = ScrollController();

  @override
  void didUpdateWidget(EaseShapes old) {
    super.didUpdateWidget(old);
    if (widget.kept.length > old.kept.length) {   // (the one just kept in sight, wherever the row was)
      WidgetsBinding.instance.addPostFrameCallback((_) => _scroll.hasClients ? _scroll.jumpTo(_scroll.position.maxScrollExtent) : null);
    }
  }

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final w = widget, side = w.side, ease = w.ease;
    return DragTarget<int>(
      onAcceptWithDetails: (d) => w.onKeep(d.data),
      builder: (_, hot, _) => DecoratedBox(
        position: DecorationPosition.foreground,
        decoration: BoxDecoration(border: hot.isEmpty ? null : Border.all(color: WsT.accent, width: 2)),
        child: ColoredBox(
          color: WsT.card,
          child: LayoutBuilder(
            builder: (_, box) => ListView(
              controller: _scroll,
              scrollDirection: w.across ? Axis.horizontal : Axis.vertical,
              itemExtent: (w.across ? box.maxWidth : box.maxHeight) / easeFits(w.across ? box.maxWidth : box.maxHeight),
              children: [
                for (final e in w.shapes)
                  PresetTile(e, key: ValueKey('ease.${e.name}'), on: _isOf(e, ease), side: side, onTap: () => _isOf(e, ease) && !(easeIsBezier(e) && said(e) != said(ease)) ? null : w.onPick(e)),   // (the one it is already: its own numbers kept)
                for (final e in w.kept)
                  Hov(
                    cursor: MouseCursor.defer,
                    builder: (_, h) => Stack(
                      children: [
                        Positioned.fill(child: PresetTile(e, key: ValueKey('ease.kept.${said(e)}'), on: said(e) == said(ease), side: side, onTap: () => said(e) == said(ease) ? null : w.onPick(e))),
                        if (h) Positioned(right: 0, top: 0, child: EaseOff(key: ValueKey('ease.forget.${said(e)}'), onTap: () => w.onForget(e))),
                      ],
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
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
