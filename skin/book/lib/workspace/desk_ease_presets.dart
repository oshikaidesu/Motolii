// The presets, each a tiny curve.

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_ease_library.dart';
import 'package:lab_book/workspace/ws.dart';

/// The presets, each a tiny curve; the one in use is lit in the accent.
class Presets extends StatelessWidget {
  const Presets({super.key, required this.ease, required this.onPick, this.more = const []});
  final Ease ease;
  final ValueChanged<Ease> onPick;

  /// Tiles under the sample's eight (joined: the document's ways that have no seat among them).
  final List<Ease> more;

  Widget _row(List<Ease> tiles, bool Function(Ease) on) => Container(
    height: Shape.z(50),
    color: WsT.card,
    padding: EdgeInsets.all(Shape.s(5)),
    child: Row(
      children: [
        for (final (i, p) in tiles.indexed) ...[
          if (i > 0) SizedBox(width: Shape.s(3)),
          Expanded(
            child: PresetTile(p, key: ValueKey('ease.${p.name}'), on: on(p), onTap: () => on(p) ? null : onPick(p)),   // (the one it is already: its own numbers kept)
          ),
        ],
      ],
    ),
  );

  @override
  Widget build(BuildContext context) => Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      _row(easePresets, (p) => p == ease && !easeOther(ease)),
      if (more.isNotEmpty) EaseLibrary(ease: ease, more: more, onPick: onPick),
    ],
  );
}

class PresetTile extends StatelessWidget {
  const PresetTile(this.ease, {super.key, required this.on, required this.onTap, this.label, this.carried = true});
  final Ease ease;

  /// What it is called on its tile, where that is less than its name (anime.js's Elastic, its way picked beside).
  final String? label;
  final bool on;
  final VoidCallback onTap;

  /// Whether a hand may carry it to the graph, to lay it over the way there (desk_ease_cards.dart).
  final bool carried;
  @override
  Widget build(BuildContext context) {
    final tile = _tile();
    return !carried
        ? tile
        : Draggable<Ease>(
            data: ease,
            dragAnchorStrategy: pointerDragAnchorStrategy,
            feedback: easeHeld(tile),
            childWhenDragging: Opacity(opacity: .3, child: tile),
            child: tile,
          );
  }

  Widget _tile() => Hov(
    onTap: onTap,
    builder: (_, h) {
      final ink = on ? WsT.onAccent : (h ? Grey.g95 : Grey.g76);
      return Transform.translate(   // (a card: it lifts under the hand)
        offset: Offset(0, h && carried ? -Shape.s(2) : 0),
        child: Container(
        decoration: BoxDecoration(color: on ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.all(Shape.r(WsT.radius)), boxShadow: h && carried ? easeShadow : null),
        padding: EdgeInsets.fromLTRB(Shape.s(2), Shape.s(5), Shape.s(2), Shape.s(4)),
        child: Column(
          children: [
            Expanded(
              child: CustomPaint(size: Size.infinite, painter: EaseThumb(ease, ink, on ? WsT.onAccent : Grey.g38)),
            ),
            SizedBox(height: Shape.s(3)),
            FittedBox( // (a name wider than its tile is drawn smaller, never cut)
              fit: BoxFit.scaleDown,
              child: Text(
                label ?? easeTitle(ease),
                maxLines: 1,
                style: T.micro(ink).copyWith(fontSize: T.px(9.5), fontWeight: on ? FontWeight.w700 : FontWeight.w500),
              ),
            ),
          ],
        ),
        ),
      );
    },
  );
}

const easeShadow = [BoxShadow(color: Color(0x80000000), blurRadius: 10, offset: Offset(0, 4))];

/// A card in the hand: held by its middle, tilted a little.
Widget easeHeld(Widget card) {
  final w = Shape.z(58), h = Shape.z(50);
  return IgnorePointer(
    child: Transform.translate(
      offset: Offset(-w / 2, -h / 2),
      child: Transform.rotate(
        angle: -.06,
        child: Container(width: w, height: h, decoration: BoxDecoration(borderRadius: BorderRadius.all(Shape.r(WsT.radius)), boxShadow: easeShadow), child: card),
      ),
    ),
  );
}

class EaseThumb extends CustomPainter {
  const EaseThumb(this.ease, this.ink, this.base);
  final Ease ease;
  final Color ink, base;
  @override
  void paint(Canvas c, Size s) {
    const lo = -.25, hi = 1.6;
    final w = math.min(s.width, s.height * 1.6), r = Rect.fromCenter(center: s.center(Offset.zero), width: w, height: s.height);
    Offset px(double x, double y) => Offset(r.left + x * r.width, r.bottom - (y - lo) / (hi - lo) * r.height);
    c.drawLine(
      px(0, 0),
      px(1, 0),
      Paint()
        ..color = base
        ..strokeWidth = 1,
    );
    final path = Path()..moveTo(px(0, 0).dx, px(0, 0).dy);
    for (var i = 1; i <= 32; i++) {
      final x = i / 32, o = px(x, ease.at(x));
      if (ease.kind == EaseKind.hold && i == 32) path.lineTo(o.dx, px(0, 0).dy);
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(
      path,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(EaseThumb o) => o.ease != ease || o.ink != ink || o.base != base;
}
