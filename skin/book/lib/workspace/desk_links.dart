// The desk's Links tool: every way one thing points at another (the values the inside shows as a thing: Parent, Join,
// With...), drawn as arcs between the layers' rows, the stack's order as the timeline's. The picked thing's ways to
// point are chips; a tap on another row's dot points it there (After Effects' pick whip), a tap on the one it points at
// already lets go. Which values point, and what they are called, is the inside's.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

class DeskLinks extends StatefulWidget {
  const DeskLinks({super.key});
  @override
  State<DeskLinks> createState() => _DeskLinksState();
}

class _DeskLinksState extends State<DeskLinks> {
  String _way = 'parent';
  static const _row = 26.0;

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), m = ws.inside, picked = int.tryParse(ws.selected ?? '');
    if (m == null) return const SizedBox.shrink();
    // (each thing's values that point at a thing, by the inside's looks; and where each points now)
    List<String> pointers(int t) => [for (final (_, names) in m.cards(t)) for (final n in names) if (m.shown(n)?.look == 'thing') n];
    final rows = ws.layers, at = {for (final (i, l) in rows.indexed) int.parse(l.id): i};
    final links = <(int, int, String)>[
      for (final l in rows)
        for (final n in pointers(int.parse(l.id)))
          if ((m.value(int.parse(l.id), n, 0) ?? 0).round() case final to when to > 0 && at.containsKey(to)) (at[int.parse(l.id)]!, at[to]!, n),
    ];
    final ways = picked == null ? const <String>[] : pointers(picked);
    final way = ways.contains(_way) ? _way : ways.firstOrNull;
    final now = picked == null || way == null ? 0 : (m.value(picked, way, 0) ?? 0).round();
    final names = {for (final w in {...ways, for (final k in links) k.$3}) w: m.shown(w)?.label ?? w};
    void point(int to) {
      if (picked == null || way == null || to == picked) return;
      if (m.set(picked, way, now == to ? 0 : to.toDouble())) ws.reread();
    }

    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        DeskStrip(
          left: [if (ws.layer case final l?) DeskLayerChip(l.name, wsTone(l.kind))],
          right: [
            for (final w in ways)
              Padding(
                padding: const EdgeInsets.only(left: 4),
                child: GestureDetector(
                  key: ValueKey('links.way.$w'),
                  onTap: () => setState(() => _way = w),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
                    decoration: BoxDecoration(color: w == way ? _tone(w, ways) : WsT.well, borderRadius: BorderRadius.circular(4)),
                    child: Text(names[w]!, style: T.micro(w == way ? WsT.onAccent : Grey.g76)),
                  ),
                ),
              ),
          ],
        ),
        const SizedBox(height: 8),
        SizedBox(
          height: _row * rows.length,
          child: Stack(
            children: [
              Positioned(left: 0, top: 0, bottom: 0, width: 40, child: CustomPaint(painter: _Arcs(links, [for (final k in links) _tone(k.$3, names.keys.toList())], _row))),
              Column(
                children: [
                  for (final l in rows)
                    SizedBox(
                      height: _row,
                      child: Row(
                        children: [
                          const SizedBox(width: 44),
                          Container(width: 4, height: 14, decoration: BoxDecoration(color: wsTone(l.kind), borderRadius: BorderRadius.circular(2))),
                          const SizedBox(width: 8),
                          Expanded(
                            child: GestureDetector(
                              behavior: HitTestBehavior.opaque,
                              onTap: () => ws.select(l.id),
                              child: Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(l.id == ws.selected ? Grey.g95 : Grey.g76)),
                            ),
                          ),
                          if (picked != null && way != null && int.parse(l.id) != picked)
                            GestureDetector(
                              key: ValueKey('links.to.${l.id}'),
                              onTap: () => point(int.parse(l.id)),
                              child: SizedBox(
                                width: 26,
                                height: _row,
                                child: CustomPaint(painter: _Whip(now == int.parse(l.id), _tone(way, ways))),
                              ),
                            )
                          else
                            const SizedBox(width: 26),
                        ],
                      ),
                    ),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  /// A way of pointing's own colour, by its place among the ways shown.
  static Color _tone(String w, List<String> ways) => const [WsT.accent, Color(0xFF7FB8B0), Color(0xFFF08A5D), Color(0xFFE974AB)][ways.indexOf(w).clamp(0, 3)];
}

/// Each link an arc on the left, from the row that points to the row it points at, an arrowhead there.
class _Arcs extends CustomPainter {
  _Arcs(this.links, this.tones, this.row);
  final List<(int, int, String)> links;
  final List<Color> tones;
  final double row;
  @override
  void paint(Canvas c, Size s) {
    for (final (k, (from, to, _)) in links.indexed) {
      final a = Offset(s.width - 2, row * (from + .5)), b = Offset(s.width - 2, row * (to + .5));
      final reach = 8.0 + 9.0 * (from - to).abs().clamp(1, 3);
      final line = Paint()
        ..color = tones[k]
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.4;
      c.drawPath(Path()
        ..moveTo(a.dx, a.dy)
        ..cubicTo(a.dx - reach, a.dy, b.dx - reach, b.dy, b.dx, b.dy), line);
      c.drawCircle(a, 2, Paint()..color = tones[k]);
      c.drawPath(Path()
        ..moveTo(b.dx, b.dy)
        ..lineTo(b.dx - 5, b.dy - 3)
        ..lineTo(b.dx - 5, b.dy + 3)
        ..close(), Paint()..color = tones[k]);
    }
  }

  @override
  bool shouldRepaint(_Arcs o) => true;
}

/// Where the picked thing may point: a ring, filled where it points now.
class _Whip extends CustomPainter {
  _Whip(this.on, this.tone);
  final bool on;
  final Color tone;
  @override
  void paint(Canvas c, Size s) {
    c.drawCircle(s.center(Offset.zero), 5, Paint()
      ..color = on ? tone : Grey.g56
      ..style = on ? PaintingStyle.fill : PaintingStyle.stroke
      ..strokeWidth = 1.3);
  }

  @override
  bool shouldRepaint(_Whip o) => o.on != on || o.tone != tone;
}
