// The desk's Align tool: the picked thing put against the picture's edges or its middle, by what of it is seen (the
// upright box of its outline, as the inside sees it). Moved as a hand moves it on the stage (held, carried by so many points of
// the picture, let go: one step), so the inside reckons it into its own units, through its parent's turn.
import 'package:flutter/widgets.dart';
import '../tokens.dart';

import 'desk_kit.dart';
import 'stage_art.dart' show WsArt;
import 'ws.dart';

class DeskAlign extends StatelessWidget {
  const DeskAlign({super.key});

  /// What of the picked thing is seen: the upright box of its outline, as the inside sees it (null: nothing picked or seen).
  static Rect? _seen(Ws ws) {
    final t = int.tryParse(ws.selected ?? '');
    return t == null ? null : ws.inside?.seen(t);
  }

  /// Put against `side` (0 left, 1 across the middle, 2 right; 3 top, 4 down the middle, 5 bottom): moved as a hand
  /// moves it, twice at most (a thing in depth or turned towards the eye is seen otherwise where it has gone).
  static void align(Ws ws, int side, [int pass = 0]) {
    final m = ws.inside, t = int.tryParse(ws.selected ?? ''), r = t == null ? null : m?.seen(t);
    if (m == null || t == null || r == null) return;
    final pic = WsArt.comp;
    final dx = switch (side) { 0 => -r.left, 1 => pic.width / 2 - r.center.dx, 2 => pic.width - r.right, _ => 0.0 };
    final dy = switch (side) { 3 => -r.top, 4 => pic.height / 2 - r.center.dy, 5 => pic.height - r.bottom, _ => 0.0 };
    if (dx.abs() < .25 && dy.abs() < .25) return;
    m.later('hold', thing: t);   // (moved as a hand moves it, without waiting; once more if a thing in depth is seen elsewhere where it went)
    m.carry('place', dx, dy);
    ws.after(m.later('let_go', number: 1), (done, _) => done && pass == 0 ? align(ws, side, 1) : null);
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
      padding: EdgeInsets.all(WsT.gap),
      children: [
        if (ws.layer case final l?) DeskStrip(left: [DeskLayerChip(l.name, wsTone(l.kind))]),
        SizedBox(height: Shape.s(8)),
        for (final row in const [[0, 1, 2], [3, 4, 5]])
          Padding(
            padding: EdgeInsets.only(bottom: Shape.s(4)),
            child: Opacity(
              opacity: r == null ? .35 : 1,   // (nothing picked that is seen: nothing to put anywhere)
              child: Row(
                children: [
                  for (final side in row) ...[
                    DeskIconButton(key: ValueKey('align.$side'), size: 36, on: at(side), glyph: (ink) => _Side(side, ink), onTap: () => r == null ? null : align(ws, side)),
                    SizedBox(width: Shape.s(4)),
                  ],
                ],
              ),
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
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(left, s.height * .32, w, s.height * .36), Shape.r(1.5)), box);
    } else {
      c.drawLine(Offset(s.width * .15, edge), Offset(s.width * .85, edge), line);
      final h = s.height * .32, top = k == 0 ? edge : k == 1 ? edge - h / 2 : edge - h;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .32, top, s.width * .36, h), Shape.r(1.5)), box);
    }
  }

  @override
  bool shouldRepaint(_Side o) => o.side != side || o.ink != ink;
}
