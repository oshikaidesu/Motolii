import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_tile.dart';

import '../../tokens.dart';
import 'inspector_parts.dart';

/// An effect's parameters as blocks that fill the panel's width: three across in a normal panel, more when it is widened,
/// so thirty parameters stay a short list. Order is the shader's; a block that needs room (a point, a size, a choice) takes two.
class ParamGrid extends StatelessWidget {
  const ParamGrid(this.fx, this.params, {super.key, this.enabled = true});
  final String fx;
  final List<Param> params;
  final bool enabled;

  static int span(Param p) => switch (p.look) {
    ParamLook.point || ParamLook.size || ParamLook.choice => 2,
    _ => 1,
  };

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <List<Param>>[[]];
      var used = 0;
      for (final p in params) {
        final s = math.min(span(p), cols);
        if (used + s > cols) {
          rows.add([]);
          used = 0;
        }
        rows.last.add(p);
        used += s;
      }
      // contract: a row cut short by a wide block stretches to the edge, so the grid never shows holes; only the last row may end early
      Widget row(List<Param> ps, bool last) {
        final filled = ps.fold<int>(0, (a, p) => a + math.min(span(p), cols));
        return Row(
          children: [
            for (var i = 0; i < ps.length; i++) ...[
              if (i > 0) SizedBox(width: g),
              Expanded(
                flex: math.min(span(ps[i]), cols),
                child: RowOff(off: !enabled, narrow: false, child: ParamTile(fx, ps[i])),
              ),
            ],
            if (last && filled < cols) ...[SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: EdgeInsets.only(top: Shape.s(4)),
        child: Column(
          children: [
            for (var i = 0; i < rows.length; i++) ...[if (i > 0) SizedBox(height: g), row(rows[i], i == rows.length - 1)],
          ],
        ),
      );
    },
  );
}


Color? hexControls(String? s) => s != null && RegExp(r'^#[0-9A-Fa-f]{6}$').hasMatch(s) ? Color(int.parse(s.substring(1), radix: 16) | 0xFF000000) : null;

/// The default value of every id a parameter owns, for a fresh layer and for Reset.
Map<String, double> paramDefaults(String fx, Param p) => switch (p.look) {
  ParamLook.point || ParamLook.size => {'${fx}_${p.id}.x': p.def, '${fx}_${p.id}.y': p.defY},
  ParamLook.colour => const {},
  _ => {'${fx}_${p.id}': p.def},
};

/// A colour parameter's default as #RRGGBB, from its 0–1 components.
String paramColour(Decl d) {
  final c = [for (final s in d.attrs['default'] ?? const <String>[]) double.tryParse(s) ?? 1];
  int ch(int i) => ((c.elementAtOrNull(i) ?? 1) * 255).round().clamp(0, 255);
  return '#${[ch(0), ch(1), ch(2)].map((e) => e.toRadixString(16).padLeft(2, '0')).join().toUpperCase()}';
}

class Swatch extends StatelessWidget {
  const Swatch(this.hex, {super.key});
  final String? hex;
  @override
  Widget build(BuildContext context) => Center(
    child: Container(
      width: Shape.z(16),
      height: Shape.z(16),
      decoration: BoxDecoration(
        color: hexControls(hex) ?? Grey.g26,
        borderRadius: BorderRadius.all(Shape.r(4)),
        border: Border.all(color: Grey.g38),
      ),
    ),
  );
}

enum GaugeKind { pie, up, down, across, centre, ramp, free, count, point, box }

/// A small whole number's plus and minus, stacked to stay one block tall; the count itself is the field beside them.
class Steps extends StatelessWidget {
  const Steps({super.key, required this.id, required this.p});
  final String id;
  final Param p;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, v = doc.get(id) ?? p.def;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    Widget btn(String s, double by) {
      final ok = on && (by < 0 ? v > (p.min ?? -1e9) : v < (p.max ?? 1e9));
      return Hov(
        onTap: ok ? () => doc.set(id, (v + by).clamp(p.min ?? -1e9, p.max ?? 1e9)) : null,
        builder: (_, h) => Container(
          width: Pop.glyph,
          height: (Pop.tile - 2 - 8 - 2) / 2,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: h && ok ? Grey.g38 : Grey.g20, borderRadius: BorderRadius.all(Shape.r(4))),
          child: Text(s, style: T.label(ok ? Grey.g91 : Grey.g44).copyWith(fontWeight: FontWeight.w700, height: 1)),
        ),
      );
    }

    return Column(mainAxisSize: MainAxisSize.min, children: [btn('+', 1), SizedBox(height: Shape.s(2)), btn('−', -1)]);
  }
}

/// A seed's re-roll: click the die for another random look; the number stays editable beside it.
class Dice extends StatelessWidget {
  const Dice({super.key, required this.id, required this.p});
  final String id;
  final Param p;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    return Hov(
      onTap: on
          ? () {
              final v = doc.get(id) ?? 0, hi = p.max ?? 9999, lo = p.min ?? 0;
              doc.set(id, (lo + (v * 7919 + 104729) % (hi - lo + 1)).roundToDouble());
            }
          : null,
      builder: (_, h) => CustomPaint(size: const Size(22, 22), painter: _DicePaint(h && on, (doc.get(id) ?? 0).round() % 6 + 1)),
    );
  }
}

class _DicePaint extends CustomPainter {
  const _DicePaint(this.hot, this.face);
  final bool hot;
  final int face;
  @override
  void paint(Canvas c, Size s) {
    final r = Rect.fromCenter(center: s.center(Offset.zero), width: Shape.z(16), height: Shape.z(16));
    c.drawRRect(RRect.fromRectAndRadius(r, Shape.r(4)), Paint()..color = hot ? Grey.g95 : Grey.g76);
    const pips = {
      1: [(0, 0)],
      2: [(-1, -1), (1, 1)],
      3: [(-1, -1), (0, 0), (1, 1)],
      4: [(-1, -1), (1, -1), (-1, 1), (1, 1)],
      5: [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)],
      6: [(-1, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (1, 1)],
    };
    for (final (dx, dy) in pips[face]!) {
      c.drawCircle(r.center + Offset(dx * 4.0, dy * 4.0), 1.5, Paint()..color = Grey.g10);
    }
  }

  @override
  bool shouldRepaint(_DicePaint o) => o.hot != hot || o.face != face;
}


