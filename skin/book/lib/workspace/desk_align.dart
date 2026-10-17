// The desk's Align tool: the picked thing put against the picture's edges or its middle, by what of it is seen (its
// frame, all its corners as it is turned). Moved as a hand moves it on the stage (held, carried by so many points of
// the picture, let go: one step), so the inside reckons it into its own units, through its parent's turn.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'desk_kit.dart';
import 'stage_art.dart' show WsArt;
import 'ws.dart';

class DeskAlign extends StatelessWidget {
  const DeskAlign({super.key});

  /// What of the picked thing is seen: left, top, right, bottom in points of the picture (null: nothing picked or seen).
  static Rect? _seen(Ws ws) {
    final t = int.tryParse(ws.selected ?? ''), f = t == null ? null : ws.inside?.frame(t);
    if (f == null) return null;
    final turn = f[4] * math.pi / 180, c = math.cos(turn), s = math.sin(turn);
    final xs = <double>[], ys = <double>[];
    for (final (u, v) in const [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]) {
      final x = u * f[2] / 2, y = v * f[3] / 2;
      xs.add(f[0] + x * c - y * s);
      ys.add(f[1] + x * s + y * c);
    }
    return Rect.fromLTRB(xs.reduce(math.min), ys.reduce(math.min), xs.reduce(math.max), ys.reduce(math.max));
  }

  /// Put against `side` (0 left, 1 across the middle, 2 right; 3 top, 4 down the middle, 5 bottom).
  static void align(Ws ws, int side) {
    final m = ws.inside, t = int.tryParse(ws.selected ?? ''), r = _seen(ws);
    if (m == null || t == null || r == null) return;
    final pic = WsArt.comp;
    final dx = switch (side) { 0 => -r.left, 1 => pic.width / 2 - r.center.dx, 2 => pic.width - r.right, _ => 0.0 };
    final dy = switch (side) { 3 => -r.top, 4 => pic.height / 2 - r.center.dy, 5 => pic.height - r.bottom, _ => 0.0 };
    if (dx.abs() < 1e-3 && dy.abs() < 1e-3) return;
    if (!m.hold(t)) return;
    m.carry('place', dx, dy);
    if (m.letGo(true)) ws.reread();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), r = _seen(ws), pic = WsArt.comp;
    bool at(int side) => r != null && switch (side) {
      0 => r.left.abs() < .5,
      1 => (r.center.dx - pic.width / 2).abs() < .5,
      2 => (r.right - pic.width).abs() < .5,
      3 => r.top.abs() < .5,
      4 => (r.center.dy - pic.height / 2).abs() < .5,
      _ => (r.bottom - pic.height).abs() < .5,
    };
    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        if (ws.layer case final l?) DeskStrip(left: [DeskLayerChip(l.name, wsTone(l.kind))]),
        const SizedBox(height: 8),
        for (final row in const [[0, 1, 2], [3, 4, 5]])
          Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Row(
              children: [
                for (final side in row) ...[
                  DeskIconButton(key: ValueKey('align.$side'), size: 36, on: at(side), glyph: (ink) => _Side(side, ink), onTap: r == null ? () {} : () => align(ws, side)),
                  const SizedBox(width: 4),
                ],
              ],
            ),
          ),
      ],
    );
  }
}

/// A side drawn as what it does: the picture's edge or middle as a line, a box put against it.
class _Side extends CustomPainter {
  _Side(this.side, this.ink);
  final int side;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final line = Paint()
      ..color = ink
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round;
    final box = Paint()..color = ink.withValues(alpha: .55);
    final across = side < 3, k = side % 3, lo = s.width * .2, hi = s.width * .8, mid = s.width / 2;
    final edge = k == 0 ? lo : k == 1 ? mid : hi;
    if (across) {
      c.drawLine(Offset(edge, s.height * .15), Offset(edge, s.height * .85), line);
      final w = s.width * .32, left = k == 0 ? edge : k == 1 ? edge - w / 2 : edge - w;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(left, s.height * .32, w, s.height * .36), const Radius.circular(1.5)), box);
    } else {
      c.drawLine(Offset(s.width * .15, edge), Offset(s.width * .85, edge), line);
      final h = s.height * .32, top = k == 0 ? edge : k == 1 ? edge - h / 2 : edge - h;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .32, top, s.width * .36, h), const Radius.circular(1.5)), box);
    }
  }

  @override
  bool shouldRepaint(_Side o) => o.side != side || o.ink != ink;
}
