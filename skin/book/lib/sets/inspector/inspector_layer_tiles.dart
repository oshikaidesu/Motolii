// The layer tiles: a flowing block grid for the Layer card's choices, switches and pads (the one piece of the Layer study the window shows).

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_tile.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// The same flowing block grid as [ParamGrid], for blocks that are not shader parameters (choices, switches, pads).
class LdGrid extends StatelessWidget {
  const LdGrid(this.cells, {super.key});
  final List<LdCell> cells;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      const g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <List<LdCell>>[[]];
      var used = 0;
      for (final c in cells) {
        final s = math.min(c.span, cols);
        if (used + s > cols) {
          rows.add([]);
          used = 0;
        }
        rows.last.add(c);
        used += s;
      }
      Widget row(List<LdCell> cs, bool last) {
        final filled = cs.fold<int>(0, (a, c) => a + math.min(c.span, cols));
        return Row(
          children: [
            for (var i = 0; i < cs.length; i++) ...[if (i > 0) const SizedBox(width: g), Expanded(flex: math.min(cs[i].span, cols), child: cs[i].child)],
            if (last && filled < cols) ...[const SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: const EdgeInsets.only(top: 4),
        child: Column(
          children: [
            for (var i = 0; i < rows.length; i++) ...[if (i > 0) const SizedBox(height: g), row(rows[i], i == rows.length - 1)],
          ],
        ),
      );
    },
  );
}

/// One block in a [LdGrid]: what it shows and how many columns it takes.
class LdCell {
  const LdCell(this.child, {this.span = 1});
  final Widget child;
  final int span;
}

/// A block's skeleton: name top-left, the control along the bottom, an optional picture top-right (as [ParamTile]).
class LdTile extends StatelessWidget {
  const LdTile({super.key, required this.label, required this.child, this.glyph, this.onTap, this.glyphW = Pop.glyph});
  final String label;
  final Widget child;
  final Widget? glyph;
  final VoidCallback? onTap;
  final double glyphW;
  @override
  Widget build(BuildContext context) {
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return Hov(
      onTap: on ? onTap : null,
      cursor: onTap != null && on ? SystemMouseCursors.click : SystemMouseCursors.basic,
      builder: (_, h) => Container(
        height: Pop.tile,
        decoration: BoxDecoration(color: h && onTap != null && on ? Grey.g20 : Pop.well, borderRadius: BorderRadius.all(Shape.r(6))),
        child: Stack(
          children: [
            if (glyph != null)
              Positioned(
                right: Pop.tilePad - 3,
                top: 4,
                bottom: glyphW > Pop.glyph ? 4 : null,
                height: glyphW > Pop.glyph ? null : Pop.glyph,
                width: glyphW,
                child: glyph!,
              ),
            Padding(
              padding: EdgeInsets.fromLTRB(Pop.tilePad, 6, glyph != null ? glyphW + Pop.tilePad : Pop.tilePad, 4),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                  child,
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
