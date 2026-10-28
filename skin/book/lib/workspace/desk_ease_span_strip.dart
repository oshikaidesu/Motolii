// Which keys.

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

/// Which keys: the layer (in its hue), from and to frames, the span's length, and Reverse; the curve copied, pasted on
/// the picked keys, or on every key of the layer (Alight Motion's Copy, Paste and Paste to All Keyframes).
class SpanStrip extends StatelessWidget {
  const SpanStrip({super.key, required this.span, required this.onReverse, this.onCopy, this.onPaste, this.onPasteAll});
  final EaseSpan? span;
  final VoidCallback onReverse;
  final VoidCallback? onCopy, onPaste, onPasteAll;
  @override
  Widget build(BuildContext context) {
    final s = span, ws = WsScope.of(context, {WsAspect.document});
    final frames = s == null ? 0 : s.to - s.from;
    // (the curve before this one, or after it: its first key picked, as Alight Motion's arrows go between the curves)
    final keys = s == null ? const <int>[] : ([...s.layer.keys]..sort()), before = keys.where((f) => f < s!.from).lastOrNull;
    final after = keys.any((f) => s != null && f > s.to) ? s!.to : null;
    Widget step(int? to, bool back) => to == null   // (its place kept: the numbers do not move as the arrows come and go)
        ? SizedBox(width: Shape.z(18))
        : DeskIconButton(key: ValueKey(back ? 'ease-previous' : 'ease-next'), size: Shape.z(18), glyph: (ink) => _Step(ink, back), onTap: () => ws.pickKey((layer: s!.layer.id, frame: to)));
    return DeskStrip(
      left: [
        if (s == null)
          Text('No keys picked · pick a key on the timeline', style: T.label(Grey.g63)) // (a way is of the stretch after a key; with none picked it is of nothing)
        else ...[
          step(before, true),   // (first, where the strip never hides them)
          step(after, false),
          SizedBox(width: Shape.s(6)),
          DeskLayerChip(s.layer.name, wsTone(s.layer.kind)),
          SizedBox(width: Shape.s(8)),
          _KeyMark(),
          SizedBox(width: Shape.s(4)),
          Text('${s.from}', style: T.value(Grey.g91)),
          SizedBox(width: Shape.s(6)),
          CustomPaint(size: const Size(14, 8), painter: _Arrow(Grey.g56)),
          SizedBox(width: Shape.s(6)),
          _KeyMark(),
          SizedBox(width: Shape.s(4)),
          Text('${s.to}', style: T.value(Grey.g91)),
          SizedBox(width: Shape.s(8)),
          Text('$frames f · ${(frames / Ws.fps).toStringAsFixed(2)} s${s.picked > 1 ? ' · ${s.picked} keys' : ''}', style: T.label(Grey.g56)),
        ],
      ],
      right: [
        for (final (label, tap) in [('Copy', onCopy), ('Paste', onPaste), ('Paste All', onPasteAll)])
          if (s != null && tap != null) ...[DeskChip(key: ValueKey('ease-${label.toLowerCase().replaceAll(' ', '-')}'), label: label, onTap: tap), SizedBox(width: WsT.gap)],
        DeskIconButton(size: Shape.z(18), glyph: (ink) => _ReverseGlyph(ink), onTap: onReverse),
      ],
    );
  }
}

class _ReverseGlyph extends CustomPainter {
  const _ReverseGlyph(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..strokeWidth = 1.3
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    final u = s.width / 18;
    c.drawLine(Offset(5 * u, 7 * u), Offset(13 * u, 7 * u), p);
    c.drawPath(
      Path()
        ..moveTo(11 * u, 5 * u)
        ..lineTo(13 * u, 7 * u)
        ..lineTo(11 * u, 9 * u),
      p,
    );
    c.drawLine(Offset(5 * u, 11.5 * u), Offset(13 * u, 11.5 * u), p);
    c.drawPath(
      Path()
        ..moveTo(7 * u, 9.5 * u)
        ..lineTo(5 * u, 11.5 * u)
        ..lineTo(7 * u, 13.5 * u),
      p,
    );
  }

  @override
  bool shouldRepaint(_ReverseGlyph o) => o.ink != ink;
}

class _KeyMark extends StatelessWidget {
  @override
  Widget build(BuildContext context) => Transform.rotate(
    angle: math.pi / 4,
    child: Container(width: Shape.z(6), height: Shape.z(6), color: WsT.keyDot),
  );
}

class _Arrow extends CustomPainter {
  const _Arrow(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..strokeWidth = 1.2
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    final y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), p);
    c.drawPath(
      Path()
        ..moveTo(s.width - 3.5, y - 3)
        ..lineTo(s.width, y)
        ..lineTo(s.width - 3.5, y + 3),
      p,
    );
  }

  @override
  bool shouldRepaint(_Arrow o) => o.ink != ink;
}

/// A chevron, back or on: to the curve before or after.
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
