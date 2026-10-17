// The desk's Sequence tool: the layers put one after another in time, as After Effects' Sequence Layers: each begins
// where the one before it ends, overlapping it by so many frames (less than none: a gap), in the stack's order from the
// front or from the back, the first staying where it is; each moved as its bar is, its keys with it. Shown first as
// bars of how they would lie, in the timeline's rows; one step when it is done.
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
  bool _fromBack = false;   // (After Effects': from the top of the stack)

  /// Which layers are put in time, in the order they will follow one another: those that have a length and stand on
  /// their own (not hidden, locked, going with another, a camera or a light; one that runs for good stays as it is).
  List<WsLayer> _order(Ws ws) {
    final m = ws.inside;
    if (m == null) return const [];
    final all = [
      for (final l in ws.layers)
        if (int.parse(l.id) case final t when !l.hidden && !l.locked && l.kind != WsKind.camera && m.word(t, 'kind') != 'light' && (m.value(t, 'parent', 0) ?? 0) == 0 && m.lasts(t) > 0) l,
    ];
    return _fromBack ? all.reversed.toList() : all;
  }

  /// Where each would begin, in seconds on a frame, and how long it is: the first where it is.
  List<(WsLayer, double, double)> _laid(Ws ws) {
    final m = ws.inside, order = _order(ws);
    if (m == null || order.isEmpty) return const [];
    double frame(double s) => (s * Ws.fps).round() / Ws.fps;
    var at = frame(m.value(int.parse(order.first.id), 'start', 0) ?? 0);
    return [
      for (final l in order)
        () {
          final lasts = m.lasts(int.parse(l.id)), one = (l, at, lasts);
          at = frame(at + lasts - _overlap).clamp(at, double.infinity);   // (never before the one before it)
          return one;
        }(),
    ];
  }

  /// Each moved as a hand moves its bar (its keys go with it), all one step.
  void _apply(Ws ws) {
    final m = ws.inside;
    if (m == null) return;
    final laid = _laid(ws);
    final ok = m.together(() => laid.every((e) {
      final t = int.parse(e.$1.id), by = e.$2 - (m.value(t, 'start', 0) ?? 0);
      if (by.abs() < 1e-9) return true;
      if (!m.hold(t)) return false;
      m.carry('bar', by);
      return m.letGo(true);
    }));
    if (ok) ws.reread();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), laid = _laid(ws);
    final rows = [for (final l in ws.layers) ?laid.where((e) => e.$1 == l).firstOrNull];   // (as the timeline's rows lie)
    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        DeskStrip(
          left: [
            DeskIconButton(key: const ValueKey('sequence.way'), size: 28, on: _fromBack, glyph: (ink) => _Way(_fromBack, ink), onTap: () => setState(() => _fromBack = !_fromBack)),
            const SizedBox(width: 8),
            SizedBox(
              width: 150,
              child: DeskScrub(key: const ValueKey('sequence.overlap'), label: 'Overlap', value: _overlap, min: -4, max: 4, onChanged: (v) => setState(() => _overlap = (v * Ws.fps).roundToDouble() / Ws.fps), format: (v) => '${(v * Ws.fps).round()} f'),
            ),
          ],
          right: [DeskIconButton(key: const ValueKey('sequence.apply'), size: 28, glyph: (ink) => _Stairs(ink), onTap: () => _apply(ws))],
        ),
        const SizedBox(height: 8),
        SizedBox(height: 14.0 * rows.length + 8, child: CustomPaint(painter: _Bars(rows, ws.inside?.seconds ?? 1))),
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
