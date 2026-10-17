// The desk's Sequence tool: the layers put one after another in time, as After Effects' Sequence Layers: each begins
// where the one before it ends, overlapping it by so many seconds (less than none: a gap), in the stack's order from the
// back or from the front. One that runs to the end is given an even share of the document. Shown first as bars of
// how they would lie; one step when it is done. Cameras and lights are not put in time.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

class DeskSequence extends StatefulWidget {
  const DeskSequence({super.key});
  @override
  State<DeskSequence> createState() => _DeskSequenceState();
}

class _DeskSequenceState extends State<DeskSequence> {
  double _overlap = 0;
  bool _fromBack = true;

  /// Which layers are put in time, in the order they will follow one another.
  List<WsLayer> _order(Ws ws) {
    final all = [for (final l in ws.layers) if (l.kind != WsKind.camera && ws.inside?.word(int.parse(l.id), 'kind') != 'light') l];   // (the inside says what a thing is)
    return _fromBack ? all.reversed.toList() : all;
  }

  /// Where each would begin and how long it is, in seconds.
  List<(WsLayer, double, double)> _laid(Ws ws) {
    final m = ws.inside, order = _order(ws);
    if (m == null || order.isEmpty) return const [];
    final share = m.seconds / order.length;
    var at = 0.0;
    return [
      for (final l in order)
        () {
          final lasts = m.lasts(int.parse(l.id)) > 0 ? m.lasts(int.parse(l.id)) : share;
          final one = (l, at, lasts);
          at = (at + lasts - _overlap).clamp(0.0, double.infinity);
          return one;
        }(),
    ];
  }

  void _apply(Ws ws) {
    final m = ws.inside;
    if (m == null) return;
    final laid = _laid(ws);
    final ok = m.together(() => laid.every((e) {
      final t = int.parse(e.$1.id);
      return m.set(t, 'start', e.$2) && (m.lasts(t) > 0 || m.set(t, 'duration', e.$3));
    }));
    if (ok) ws.reread();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), laid = _laid(ws);
    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        DeskStrip(
          left: [
            DeskIconButton(key: const ValueKey('sequence.way'), size: 28, on: !_fromBack, glyph: (ink) => _Way(_fromBack, ink), onTap: () => setState(() => _fromBack = !_fromBack)),
            const SizedBox(width: 8),
            SizedBox(
              width: 150,
              child: DeskScrub(key: const ValueKey('sequence.overlap'), label: 'Overlap', value: _overlap, min: -4, max: 4, onChanged: (v) => setState(() => _overlap = (v * 10).roundToDouble() / 10), format: (v) => '${v.toStringAsFixed(1)} s'),
            ),
          ],
          right: [DeskIconButton(key: const ValueKey('sequence.apply'), size: 28, glyph: (ink) => _Stairs(ink), onTap: () => _apply(ws))],
        ),
        const SizedBox(height: 8),
        SizedBox(height: 14.0 * laid.length + 8, child: CustomPaint(painter: _Bars(laid, ws.inside?.seconds ?? 1))),
      ],
    );
  }
}

/// How they would lie: a bar each, in its layer's hue, along the document's time.
class _Bars extends CustomPainter {
  _Bars(this.laid, this.seconds);
  final List<(WsLayer, double, double)> laid;
  final double seconds;
  @override
  void paint(Canvas c, Size s) {
    final end = laid.fold(seconds, (e, l) => e > l.$2 + l.$3 ? e : l.$2 + l.$3);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(0, 0, s.width, s.height), const Radius.circular(4)), Paint()..color = WsT.well);
    c.drawLine(Offset(s.width * seconds / end, 2), Offset(s.width * seconds / end, s.height - 2), Paint()
      ..color = Grey.g38
      ..strokeWidth = 1);
    for (final (k, (l, at, lasts)) in laid.indexed) {
      final r = Rect.fromLTWH(s.width * at / end, 4.0 + 14 * k, s.width * lasts / end, 10);
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), Paint()..color = wsTone(l.kind).withValues(alpha: .85));
    }
  }

  @override
  bool shouldRepaint(_Bars o) => true;
}

/// One after another: three steps going down to the right.
class _Stairs extends CustomPainter {
  _Stairs(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = ink;
    for (var k = 0; k < 3; k++) {
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * (.18 + .2 * k), s.height * (.24 + .18 * k), s.width * .3, s.height * .12), const Radius.circular(1)), p);
    }
  }

  @override
  bool shouldRepaint(_Stairs o) => o.ink != ink;
}

/// From the back of the stack, or from its front: a stack with the first one lit.
class _Way extends CustomPainter {
  _Way(this.fromBack, this.ink);
  final bool fromBack;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    for (var k = 0; k < 3; k++) {
      final first = fromBack ? k == 2 : k == 0;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .25, s.height * (.24 + .2 * k), s.width * .5, s.height * .13), const Radius.circular(1)),
          Paint()..color = first ? ink : ink.withValues(alpha: .35));
    }
  }

  @override
  bool shouldRepaint(_Way o) => o.fromBack != fromBack || o.ink != ink;
}
