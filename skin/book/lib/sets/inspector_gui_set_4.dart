// Part of inspector_gui_set.dart (cut there by size, as it was).
part of 'inspector_gui_set.dart';

Widget _layoutHand(GDoc d, Size size) {
  final g = _LayGeo(size, d);
  return _Hand(
    d: d,
    size: size,
    pick: (p) {
      if ((p - g.colAt).distance < 9) {
        return 'cols';
      }
      if (!g.on) {
        return null; // Grid gates the rest
      }
      if ((p - g.gapAt).distance < 9) {
        return 'gap';
      }
      if ((p - g.padAt).distance < 9 && g.pad > 0) {
        return 'pad';
      }
      if ((p - g.rightAt).distance < 9 && g.hs != 0 || (p - g.rightAt).distance < 9) {
        return 'right';
      }
      if ((p - g.bottomAt).distance < 9) {
        return 'bottom';
      }
      return null;
    },
    start: (grab, p, fine) {
      // pulling an edge turns Hug / Fill into Fixed at the size it already has (nothing jumps)
      if (grab == 'right' && d['lay.hs'] != 2) {
        d.set('lay.w', g.parent.width / Gui.layK);
        d.set('lay.hs', 2);
      }
      if (grab == 'bottom' && d['lay.vs'] != 2) {
        d.set('lay.h', g.parent.height / Gui.layK);
        d.set('lay.vs', 2);
      }
    },
    drag: (grab, p, last, fine) {
      final k = fine ? Gui.fine : 1.0, dd = (p - last) * k;
      switch (grab) {
        case 'cols':
          d.set('lay.cols', (d['lay.cols'] + dd.dx / (Gui.layCw + g.gap)).clamp(1.0, Gui.layN + .49));
        case 'gap':
          d.set('lay.gap', (d['lay.gap'] + (g.cols > 1 ? dd.dx : dd.dy) / Gui.layK).clamp(0.0, 80.0));
        case 'pad':
          d.set('lay.pad', (d['lay.pad'] + (dd.dx + dd.dy) / 2 / Gui.layK).clamp(0.0, 80.0));
        case 'right':
          d.set('lay.w', (d['lay.w'] + dd.dx / Gui.layK).clamp(60.0, 800.0));
        case 'bottom':
          d.set('lay.h', (d['lay.h'] + dd.dy / Gui.layK).clamp(40.0, 600.0));
      }
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final role = [Fam.along.c, Fam.stagger.c, Fam.scatter.c], a = g.on ? 1.0 : .5;
      cv.drawRect(g.avail, _ln(N.g15)); // the space on offer
      final pr = g.parent;
      cv.drawRect(pr, _ln((d.locked ? Role.disabled : N.g76).withValues(alpha: a)));
      if (g.pad > 0) {
        cv.drawRect(g.inner, _ln(role[1].withValues(alpha: .35)));
      }
      for (final b in g.boxes) {
        cv.drawRect(b, _fl(N.g20.withValues(alpha: a)));
        cv.drawRect(b, _ln(N.g44.withValues(alpha: a)));
      }
      void dot(Offset o, Color c, String key, {bool square = false}) {
        final on = hot == key, col = d.locked ? Role.disabled : c;
        if (square) {
          final r = Rect.fromCenter(center: o, width: Gui.handle + 2, height: Gui.handle + 2);
          cv.drawRect(r, _fl(N.g07));
          cv.drawRect(r, _ln(on ? N.g95 : col));
        } else {
          cv.drawCircle(o, on ? 4 : Gui.dot, _fl(on ? N.g95 : col));
        }
      }

      cv.drawLine(g.colAt + const Offset(0, -7), g.colAt + const Offset(0, 7), _ln(d.locked ? Role.disabled : role[0]));
      cv.drawLine(g.colAt + const Offset(-2, 0), g.colAt + const Offset(2, 0), _ln(d.locked ? Role.disabled : role[0]));
      if (g.on) {
        dot(g.gapAt, role[1], 'gap');
        if (g.pad > 0) {
          dot(g.padAt, role[1], 'pad', square: true);
        }
        cv.drawLine(g.rightAt + const Offset(0, -7), g.rightAt + const Offset(0, 7), _ln(hot == 'right' ? N.g95 : (d.locked ? Role.disabled : role[2])));
        cv.drawLine(g.bottomAt + const Offset(-7, 0), g.bottomAt + const Offset(7, 0), _ln(hot == 'bottom' ? N.g95 : (d.locked ? Role.disabled : role[2])));
      }
    },
  );
}

class _SizingGlyph extends StatelessWidget {
  const _SizingGlyph(this.i, this.col);
  final int i;
  final Color col;
  @override
  Widget build(BuildContext context) => CustomPaint(
        size: const Size(16, 10),
        painter: _Fn((c, s) {
          final w = s.width, cy = s.height / 2, p = _ln(col), f = _fl(col);
          switch (i) {
            case 0: // Hug: the walls close in
              c.drawLine(const Offset(.5, 0), Offset(.5, s.height), p);
              c.drawLine(Offset(w - .5, 0), Offset(w - .5, s.height), p);
              c.drawPath(Path()..moveTo(2.5, cy - 2.4)..lineTo(5.5, cy)..lineTo(2.5, cy + 2.4)..close(), f);
              c.drawPath(Path()..moveTo(w - 2.5, cy - 2.4)..lineTo(w - 5.5, cy)..lineTo(w - 2.5, cy + 2.4)..close(), f);
            case 1: // Fill: the walls reach out
              c.drawLine(Offset(w * .3, cy), Offset(w * .7, cy), p);
              c.drawPath(Path()..moveTo(w * .3, cy - 2.6)..lineTo(0, cy)..lineTo(w * .3, cy + 2.6)..close(), f);
              c.drawPath(Path()..moveTo(w * .7, cy - 2.6)..lineTo(w, cy)..lineTo(w * .7, cy + 2.6)..close(), f);
            default: // Fixed: a set size
              c.drawRect(Rect.fromLTWH(w * .2, 1.5, w * .6, s.height - 3), p);
          }
        }),
      );
}

const _sizingNames = ['Hug', 'Fill', 'Fixed'];

Widget _sizeLine(GDoc d, String axis) {
  final id = axis == 'W' ? 'lay.hs' : 'lay.vs', on = d['lay.on'] > .5 && !d.locked, cur = d[id].round(), fixed = cur == 2;
  return Row(children: [
    SizedBox(width: 14, child: Text(axis, style: T.label(N.g63))),
    for (var i = 0; i < 3; i++)
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: on
            ? () {
                d.begin();
                d.set(id, i.toDouble());
                d.end();
              }
            : null,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          width: 28,
          height: Gui.field,
          margin: const EdgeInsets.only(right: 2),
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: cur == i ? N.g20 : N.g07,
            borderRadius: BorderRadius.circular(5),
            border: Border(bottom: BorderSide(color: cur == i && on ? Role.selected : const Color(0x00000000))),
          ),
          child: _SizingGlyph(i, !on ? N.g44 : (cur == i ? N.g95 : N.g63)),
        ),
      ),
    const SizedBox(width: 4),
    Expanded(child: XyzStack(d, [Spec(axis == 'W' ? 'lay.w' : 'lay.h', '', unit: 'px', dec: 0, per: 2, on: on && fixed, word: fixed || !d.locked && !on ? null : _sizingNames[cur])])),
  ]);
}

Widget _layNumbers(GDoc d) {
  final on = d['lay.on'] > .5;
  Widget f(String t, Spec s) => Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [Text(t, style: T.label(N.g63)), _gap(4), XyzStack(d, [s])]));
  return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
    Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      f('Columns', Spec('lay.cols', 'C', color: Fam.along.c, dec: 0, per: .05)),
      const SizedBox(width: 6),
      f('Gap', Spec('lay.gap', 'G', color: Fam.stagger.c, unit: 'px', dec: 0, on: on)),
      const SizedBox(width: 6),
      f('Padding', Spec('lay.pad', 'P', color: Fam.stagger.c, unit: 'px', dec: 0, on: on)),
    ]),
    _gap(8),
    _sizeLine(d, 'W'),
    _gap(2),
    _sizeLine(d, 'H'),
  ]);
}

class LayoutGui extends StatelessWidget {
  const LayoutGui(this.d, {super.key, this.mode = Mode.gui});
  final GDoc d;
  final Mode mode;
  @override
  Widget build(BuildContext context) {
    final diagram = _layoutHand(d, const Size(Gui.content, Gui.layoutH));
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Layout', trailing: Row(mainAxisSize: MainAxisSize.min, children: [
        Text('Grid', style: T.label(d['lay.on'] > .5 ? N.g95 : N.g56)),
        const SizedBox(width: 5),
        PillSwitch(on: d['lay.on'] > .5, onChanged: d.locked ? null : (v) {
          d.begin();
          d.set('lay.on', v ? 1 : 0); // Grid gates the rest; nothing it gates is lost
          d.end();
        }),
      ])),
      _gap(6),
      if (mode == Mode.fields) _layNumbers(d) else if (mode == Mode.gui) ...[diagram, _gap(6), _layNumbers(d)] else ...[_layNumbers(d), _gap(8), diagram],
    ]);
  }
}

// ---- hierarchy options: the SAME Transform block in three arrangements (research/precedents-gui.md) ---------------------------------

Widget _axisChips(GDoc d) => Row(children: [
      for (var i = 0; i < 3; i++) Chip(const ['Z', 'X', 'Y'][i], on: d['rot.axis'].round() == i, enabled: !d.locked, onTap: () => d.set('rot.axis', i.toDouble())),
    ]);

Spec _rotSpec(GDoc d) {
  final i = d['rot.axis'].round().clamp(0, 2);
  return Spec(const ['rot.z', 'rot.x', 'rot.y'][i], const ['Z', 'X', 'Y'][i], axis: const [2, 0, 1][i], unit: '°', per: .5);
}

/// A graphic with its numbers as a quiet readout (one line per axis). Hover, focus or a drag on the cell turns the readout into the real fields.
class _Cell extends StatefulWidget {
  const _Cell(this.d, this.hand, this.specs, {this.even = false, this.horizontal = false});
  final GDoc d;
  final Widget hand;
  final List<Spec> specs;
  final bool even; // the readout takes the balanced step whatever the knob says (option 3: equal weight)
  final bool horizontal; // the readouts sit side by side (the wide pad)
  @override
  State<_Cell> createState() => _CellState();
}

class _CellState extends State<_Cell> {
  bool hover = false;
  @override
  Widget build(BuildContext context) {
    final d = widget.d, w = widget.even ? 1 : d.weight, shown = (hover || d.live) && !d.locked;
    Widget line(Spec s) => SizedBox(
          height: Gui.field,
          child: Row(children: [
            SizedBox(width: 14, child: Text(s.tag, maxLines: 1, style: T.label(N.g56))),
            Expanded(
              child: FittedBox(
                fit: BoxFit.scaleDown,
                alignment: Alignment.centerLeft,
                child: Text(d.mixed.contains(s.id) ? '—' : '${_fmt(d[s.id] * s.scale, s.dec)}${s.unit.isEmpty ? '' : ' ${s.unit}'}', maxLines: 1, style: T.value(Gui.readoutInk[w]).copyWith(fontSize: Gui.readoutSize[w])),
              ),
            ),
          ]),
        );
    final fields = widget.horizontal
        ? Row(children: [for (final (i, s) in widget.specs.indexed) ...[if (i > 0) const SizedBox(width: Gui.rowGap), Expanded(child: shown ? XyzStack(d, [s], gap: Gui.rowGap) : line(s))]])
        : (shown
            ? XyzStack(d, widget.specs, gap: Gui.rowGap)
            : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [for (final (i, s) in widget.specs.indexed) ...[if (i > 0) const SizedBox(height: Gui.rowGap), line(s)]]));
    return MouseRegion(
      onEnter: (_) => setState(() => hover = true),
      onExit: (_) => setState(() => hover = false),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [widget.hand, const SizedBox(height: Gui.rowGap), fields]),
    );
  }
}

Widget _lab(String t, {Widget? trailing}) => SizedBox(height: 20, child: Row(children: [Text(t, style: T.label(N.g63)), const Spacer(), ?trailing]));
Widget _linkChip(GDoc d) => Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () => d.set('link', d['link'] > .5 ? 0 : 1));

/// Option 1. Instruments primary: the pad is the largest thing, dial / scale / anchor share one cell size, numbers are a readout.
/// From precedents-gui.md: Pattern 1C lines 26-30 (Ableton: the value appears with the knob), candidates P1-c line 75 and P2-c line 81 (one larger pad over a bank row of equal small instruments).
class TransformInstrumentsFirst extends StatelessWidget {
  const TransformInstrumentsFirst(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    final three = d['space'] > 0;
    const w = Gui.content, colW = (w - 16) / 3;
    Widget cell(String t, Widget hand, List<Spec> specs, {Widget? trailing, Widget? under}) => SizedBox(
          width: colW,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_lab(t, trailing: trailing), _Cell(d, hand, specs), if (under != null) ...[const SizedBox(height: Gui.rowGap), under]]),
        );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      _lab('Position'),
      _Cell(
        d,
        Row(children: [_padHand(d, Size(three ? w - Gui.rail - 6 : w, Gui.padH + 16)), if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.padH + 16))]]),
        _posSpecs(d),
        horizontal: true,
      ),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        cell('Rotation', _dialHand(d, Size(colW, colW)), [_rotSpec(d)], under: three ? _axisChips(d) : null),
        const SizedBox(width: 8),
        cell('Scale', _scaleHand(d, Size(colW, colW)), _scaleSpecs, trailing: _linkChip(d)),
        const SizedBox(width: 8),
        cell('Anchor', _anchorHand(d, Size(colW, colW)), _anchorSpecs),
      ]),
    ]);
  }
}

/// Option 2. Figma-like: the fields are the primary at full weight, one small helper of one size sits beside each.
/// From precedents-gui.md: Pattern 1B lines 20-24 (Figma: numbers always visible, the graphic is a smaller key beside them), candidate P2-d line 82 (instruments as keys of identical size in the row grid).
class TransformFieldsFirst extends StatelessWidget {
  const TransformFieldsFirst(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    const key = 72.0;
    Widget row(String t, List<Spec> specs, Widget helper, {Widget? trailing, Widget? under}) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _title(t, trailing: trailing),
          _gap(6),
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [XyzStack(d, specs, gap: Gui.rowGap), if (under != null) ...[const SizedBox(height: Gui.rowGap), under]])),
            const SizedBox(width: 8),
            SizedBox(width: key, height: key, child: helper),
          ]),
          _gap(12),
        ]);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      row('Position', _posSpecs(d), _padHand(d, const Size(key, key))),
      row('Rotation', [_rotSpec(d)], _dialHand(d, const Size(key, key)), under: d['space'] > 0 ? _axisChips(d) : null),
      row('Scale', _scaleSpecs, _scaleHand(d, const Size(key, key)), trailing: _linkChip(d)),
      row('Anchor', _anchorSpecs, _anchorHand(d, const Size(key, key))),
    ]);
  }
}

/// Option 3. One shared cell grid: four instruments of the same size in 2 x 2, an equal-weight readout under each.
/// From precedents-gui.md: Pattern 2C lines 47-48 (Ableton: a uniform bank, one diameter, one label slot), P2 finding line 52, candidate P2-a line 79 (every instrument a square cell of one side, a shared top edge).
class TransformGrid extends StatelessWidget {
  const TransformGrid(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    const side = (Gui.content - 8) / 2;
    Widget cell(String t, Widget hand, List<Spec> specs, {Widget? trailing, Widget? under}) => SizedBox(
          width: side,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_lab(t, trailing: trailing), _Cell(d, hand, specs, even: true), if (under != null) ...[const SizedBox(height: Gui.rowGap), under]]),
        );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        cell('Position', _padHand(d, const Size(side, side)), _posSpecs(d)),
        const SizedBox(width: 8),
        cell('Rotation', _dialHand(d, const Size(side, side)), [_rotSpec(d)], under: d['space'] > 0 ? _axisChips(d) : null),
      ]),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        cell('Scale', _scaleHand(d, const Size(side, side)), _scaleSpecs, trailing: _linkChip(d)),
        const SizedBox(width: 8),
        cell('Anchor', _anchorHand(d, const Size(side, side)), _anchorSpecs),
      ]),
    ]);
  }
}

// ---- UX options: four interaction models for the SAME Transform block --------------------------------------------------------------

enum GMode { all, move, scale, rotate, anchor }

String _zoneName(Object? hot) {
  if (hot == null) {
    return '';
  }
  if (hot == 'move') {
    return 'Move: drag the body';
  }
  if (hot == 'rot') {
    return 'Rotate: drag in the ring';
  }
  if (hot == 'pivot' || (hot is String && hot.startsWith('a'))) {
    return 'Anchor: the pivot';
  }
  if (hot is int) {
    return hot < 4 ? 'Scale: corner' : 'Scale: edge';
  }
  return '';
}
