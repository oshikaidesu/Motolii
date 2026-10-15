import 'dart:math' as math;

import 'inspector_studies.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_tile.dart';

import '../../tokens.dart';
import 'inspector_parts.dart';

const _taGroups = ['Chars', 'Word', 'Line', 'All'];
const _taSample = 'Motolii in motion';

const paraGroupP = Param(id: 'grouping', label: 'Anchor grouping', look: ParamLook.choice, type: WType.u32, min: 0, max: 3, options: _taGroups);



String taWord(Doc d) {
  final t = (d.s2['text.content'] ?? '').split('\n').first.trim();
  return t.isEmpty ? _taSample : (t.length > 22 ? t.substring(0, 22) : t);
}

Map<String, double> taDefaults(Iterable<Param> ps, String fx) => {for (final p in ps) ...paramDefaults(fx, p)};

/// Blocks laid like [ParamGrid] (2–5 across by width, a wide block takes two), but any widget may be a block.
class TaGrid extends StatelessWidget {
  const TaGrid(this.items, {super.key, this.on = true});
  final List<(int, Widget)> items;
  final bool on;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      const g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <List<(int, Widget)>>[[]];
      var used = 0;
      for (final (span, w) in items) {
        final s = math.min(span, cols);
        if (used + s > cols && rows.last.isNotEmpty) {
          rows.add([]);
          used = 0;
        }
        rows.last.add((s, w));
        used += s;
      }
      Widget row(List<(int, Widget)> r, bool last) {
        final filled = r.fold<int>(0, (a, it) => a + it.$1);
        return Row(
          children: [
            for (final (i, (s, w)) in r.indexed) ...[if (i > 0) const SizedBox(width: g), Expanded(flex: s, child: w)],
            if (last && filled < cols) ...[const SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: const EdgeInsets.only(top: 4),
        child: RowOff(
          off: !on,
          narrow: false,
          child: Column(
            children: [
              for (var i = 0; i < rows.length; i++) ...[if (i > 0) const SizedBox(height: g), row(rows[i], i == rows.length - 1)],
            ],
          ),
        ),
      );
    },
  );
}

(int, Widget) taTile(String fx, Param p) => (ParamGrid.span(p), ParamTile(fx, p));

/// A block that is not a number: the name top-left, the control along the bottom, an optional picture top-right.
class TaBlock extends StatelessWidget {
  const TaBlock({super.key, required this.label, required this.child, this.glyph});
  final String label;
  final Widget child;
  final Widget? glyph;
  @override
  Widget build(BuildContext context) {
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return Container(
      height: Pop.tile,
      decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
      child: Stack(
        children: [
          // the picture stays in the name's line so the control below keeps the block's full width
          if (glyph != null)
            Positioned(
              right: Pop.tilePad - 3,
              top: 2,
              width: Pop.glyph,
              height: 16,
              child: Center(child: glyph),
            ),
          Padding(
            padding: const EdgeInsets.fromLTRB(Pop.tilePad, 6, Pop.tilePad, 4),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Padding(
                  padding: EdgeInsets.only(right: glyph != null ? Pop.glyph : 0),
                  child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                ),
                SizedBox(height: 22, child: child),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// The fold under a card's main blocks (as in an effect card): rarely touched values wait behind it.
class TaFold extends StatelessWidget {
  const TaFold(this.flag, this.word, this.count, {super.key});
  final String flag, word;
  final int count;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed), open = x.doc.b[flag] ?? false;
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: Hov(
        onTap: () => x.doc.flag(flag, !open),
        builder: (_, h) => Container(
          height: 24,
          decoration: BoxDecoration(
            border: Border(top: BorderSide(color: ed.rule)),
          ),
          child: Row(
            children: [
              Text(word, style: ed.labStyle(h || open ? Grey.g95 : Grey.g63)),
              const Spacer(),
              Text(open ? 'Fold' : '$count more', style: T.label(Grey.g56)),
            ],
          ),
        ),
      ),
    );
  }
}




class ParaJustPaint extends CustomPainter {
  const ParaJustPaint(this.align, this.on);
  final int align;
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = on ? Grey.g95 : Grey.g63
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round;
    for (final (i, w) in const [(0, 1.0), (1, .6), (2, .85), (3, .5)].map((e) => (e.$1, e.$2 * s.width))) {
      final y = 1 + i * (s.height - 2) / 3,
          x0 = switch (align) {
            0 => 0.0,
            1 => (s.width - w) / 2,
            _ => s.width - w,
          };
      c.drawLine(Offset(x0, y), Offset(x0 + w, y), p);
    }
  }

  @override
  bool shouldRepaint(ParaJustPaint o) => o.align != align || o.on != on;
}

