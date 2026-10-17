// The desk's Color tool: the colours the document already uses (each thing's fill, its edge, its copies' colours: every
// six digits in them), the most used first; a tap gives one to the picked thing's fill, or to its edge (the ring).
// A palette is what is there, not a list kept in the window.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

class DeskColor extends StatefulWidget {
  const DeskColor({super.key});
  @override
  State<DeskColor> createState() => _DeskColorState();
}

class _DeskColorState extends State<DeskColor> {
  bool _edge = false;
  final _kept = <String>[];   // (a colour once seen stays while the desk shows this: given away from its last thing, it is still to be had)

  /// Every colour the document's things say, how many times, the most first.
  static List<(String, int)> _used(Ws ws) {
    final m = ws.inside, seen = <String, int>{};
    if (m == null) return const [];
    for (final l in ws.layers) {
      final t = int.tryParse(l.id);
      if (t == null) continue;
      for (final name in const ['fill', 'stroke', 'copy_fill']) {
        for (final w in m.word(t, name).split(' ')) {
          if (RegExp(r'^[0-9a-fA-F]{6}$').hasMatch(w)) seen[w.toLowerCase()] = (seen[w.toLowerCase()] ?? 0) + 1;
        }
      }
    }
    return seen.entries.map((e) => (e.key, e.value)).toList()..sort((a, b) => b.$2.compareTo(a.$2));
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), m = ws.inside, t = int.tryParse(ws.selected ?? '');
    final inUse = _used(ws).map((u) => u.$1).toList();
    for (final h in inUse) {
      if (!_kept.contains(h)) _kept.add(h);
    }
    final used = [for (final h in inUse) (h, 0), for (final h in _kept) if (!inUse.contains(h)) (h, 0)];
    final now = t == null || m == null ? '' : m.word(t, _edge ? 'stroke' : 'fill').toLowerCase();
    void give(String hex) {
      if (m == null || t == null) return;
      if (m.say(t, _edge ? 'stroke' : 'fill', hex)) ws.reread();
    }

    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        DeskStrip(
          left: [
            if (ws.layer case final l?) DeskLayerChip(l.name, wsTone(l.kind)),
          ],
          right: [
            DeskIconButton(key: const ValueKey('color.fill'), size: 26, on: !_edge, glyph: (ink) => _Target(false, ink), onTap: () => setState(() => _edge = false)),
            DeskIconButton(key: const ValueKey('color.edge'), size: 26, on: _edge, glyph: (ink) => _Target(true, ink), onTap: () => setState(() => _edge = true)),
          ],
        ),
        const SizedBox(height: 8),
        Wrap(
          spacing: 6,
          runSpacing: 6,
          children: [
            for (final (hex, _) in used)
              GestureDetector(
                key: ValueKey('swatch.$hex'),
                onTap: () => give(hex),
                child: Container(
                  width: 34,
                  height: 34,
                  decoration: BoxDecoration(
                    color: Color(0xFF000000 | int.parse(hex, radix: 16)),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: hex == now ? WsT.accent : Grey.g26, width: hex == now ? 2 : 1),   // (the one the picked thing has now)
                  ),
                ),
              ),
          ],
        ),
      ],
    );
  }
}

/// Where a colour goes: the fill (a disc) or the edge (a ring).
class _Target extends CustomPainter {
  _Target(this.edge, this.ink);
  final bool edge;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = ink;
    if (edge) {
      p
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.2;
    }
    c.drawCircle(s.center(Offset.zero), s.shortestSide * .26, p);
  }

  @override
  bool shouldRepaint(_Target o) => o.edge != edge || o.ink != ink;
}
