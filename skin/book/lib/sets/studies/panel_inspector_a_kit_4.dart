// Part of panel_inspector_a_kit.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

/// I2-c: the ladder that is visible only while the value is held.
class _Ladder extends StatelessWidget {
  const _Ladder({required this.band});
  final int band;
  @override
  Widget build(BuildContext context) => Container(
        width: 60,
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g26), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 14, offset: const Offset(0, 5))]),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < 4; i++)
            Container(
              height: 20,
              alignment: Alignment.centerLeft,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: i == band ? N.g20 : null),
              foregroundDecoration: i == band ? const BoxDecoration(border: Border(left: BorderSide(color: N.g95, width: 2))) : null,
              child: Text(_bandWords[i], style: T.value(i == band ? N.g95 : N.g63)),
            ),
        ]),
      );
}

// ---- rows ----

/// One row: the label column, the cells, an optional trailing column (key mark, reset). Every row has the same columns so the fields end at one edge.
Widget _shell({required Widget label, required Widget cells, Widget? trail, double labW = _labW, double trailW = _keyW, Widget? under, Color? band, bool selected = false}) => Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _Bleed(
          fill: selected ? N.g20 : band,
          tick: selected,
          child: Container(
            constraints: const BoxConstraints(minHeight: _rowH),
            child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              SizedBox(width: labW, child: label),
              Expanded(child: cells),
              SizedBox(width: trailW, child: trail),
            ]),
          ),
        ),
        ?under,
      ],
    );

Widget _cellsOf(_R r, _M m, {_Gr g = _Gr.face, bool dock = false, bool enabled = true, Map<String, String>? ghost}) {
  if (r.cells.length == 1) {
    return _Cell(m, r.cells.first, g: g, narrow: dock, enabled: enabled, ghost: ghost?[r.cells.first.id]);
  }
  return Row(children: [
    for (final (i, p) in r.cells.indexed) ...[
      if (i > 0) const SizedBox(width: 4),
      Expanded(child: _Cell(m, p, g: g, tag: p.label.isEmpty ? null : p.label, narrow: dock, enabled: enabled, ghost: ghost?[p.id])),
    ],
  ]);
}

/// The standard row. [tone] picks the label's grey, [trail] fills the key column, [labelHandle] makes the label a scrub handle.
Widget _prow(_R r, _M m, {_Gr g = _Gr.face, bool dock = false, Widget? trail, Color? tone, VoidCallback? labelDbl, Widget? under, bool enabled = true, double? labW, Widget? lead, Color? band, bool selected = false, double? trailW, Map<String, String>? ghost, Widget Function(Widget)? wrapLabel, bool labelHot = false}) {
  final changed = r.cells.any((c) => _chg(m, c.id));
  final single = r.cells.length == 1;
  final handle = g == _Gr.label && single && r.isNum && enabled;
  Widget labelRow([bool hot = false]) => Row(children: [
        if (lead != null) ...[lead, const SizedBox(width: 4)],
        Expanded(child: Text(r.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: _labStyle(changed: changed, enabled: enabled, hot: hot, tone: tone))),
        if (handle) ...[const SizedBox(width: 4), _grip(hot ? N.g63 : N.g38)],
      ]);
  Widget text = labelRow();
  if (g == _Gr.label && single && r.isNum) {
    text = Hov(cursor: SystemMouseCursors.resizeLeftRight, builder: (_, h) => _LabelHandle(m: m, p: r.cells.first, child: Padding(padding: const EdgeInsets.only(right: 8), child: labelRow(h || labelHot))));
  } else if (labelDbl != null) {
    text = GestureDetector(behavior: HitTestBehavior.opaque, onDoubleTap: labelDbl, child: Padding(padding: const EdgeInsets.only(right: 8), child: text));
  } else {
    text = Padding(padding: const EdgeInsets.only(right: 8), child: text);
  }
  if (wrapLabel != null) {
    text = wrapLabel(text);
  }
  return _shell(
    label: text,
    cells: _cellsOf(r, m, g: g, dock: dock, enabled: enabled, ghost: ghost),
    trail: trail,
    labW: labW ?? (r.cells.length == 1 && r.cells.first.kind == _K.toggle ? (dock ? 104 : 136) : (dock ? _labWDock : _labW)),
    under: under,
    band: band,
    selected: selected,
    trailW: trailW ?? _keyW,
  );
}

// ---- key mark (own painter: the shapes of the lab's KeyMark, plus the "will key" state of I4-b) ----

class _Key extends StatelessWidget {
  const _Key(this.s, {this.will = false, this.onTap, this.onLong, this.onSecondary, this.enabled = true, this.hover, this.focus, this.pressed});
  final KeyS s;
  final bool will, enabled;
  final bool? hover, focus, pressed; // pinned states (Parts)
  final VoidCallback? onTap, onLong, onSecondary;

  /// rest: the mark alone. hover g15 plate; focus 1 px g76 edge; pressed g26 plate. Disabled: no plate, no state.
  @override
  Widget build(BuildContext context) => _Ctl(
        enabled: enabled,
        onTap: onTap,
        onLong: onLong,
        onSecondary: onSecondary,
        hover: hover,
        focus: focus,
        pressed: pressed,
        builder: (_, h, f, p) => Container(
          width: _keyW,
          height: _cellH,
          decoration: BoxDecoration(color: p ? N.g26 : (h ? N.g15 : null), borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g76 : const Color(0x00000000))),
          child: CustomPaint(painter: _KeyPaint(s, h || p, will, enabled)),
        ),
      );
}

/// A dashed outline (2 on, 2 off): the shape of "present but not available".
void _dashed(Canvas c, Path p, Paint paint) {
  for (final m in p.computeMetrics()) {
    for (var d = 0.0; d < m.length; d += 4) {
      c.drawPath(m.extractPath(d, math.min(d + 2, m.length)), paint);
    }
  }
}

class _KeyPaint extends CustomPainter {
  const _KeyPaint(this.s, this.hover, this.will, this.enabled);
  final KeyS s;
  final bool hover, will, enabled;
  @override
  void paint(Canvas c, Size z) {
    final o = Offset(z.width / 2, z.height / 2), r = s == KeyS.at ? (hover ? 6 : 5.2) : 4.6;
    final dia = Path()..moveTo(o.dx, o.dy - r)..lineTo(o.dx + r, o.dy)..lineTo(o.dx, o.dy + r)..lineTo(o.dx - r, o.dy)..close();
    Paint line(Color col, double w) => Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = w;
    switch (s) {
      case KeyS.off:
        if (!enabled) {
          _dashed(c, dia, line(N.g38, 1));
        } else {
          c.drawPath(dia, line(will ? Role.of(C.mode, Role.still) : (hover ? Role.of(N.g76, Role.still) : Role.of(N.g44, Role.still)), will ? 1.5 : 1));
        }
      case KeyS.anim:
        final col = !enabled ? Role.disabled : (hover ? Role.of(N.g95, Role.key) : Role.of(N.g91, Role.key));
        c.drawPath(dia, line(col, 1));
        c.drawLine(Offset(o.dx - r - 5, o.dy), Offset(o.dx - r, o.dy), line(col, 1));
        c.drawLine(Offset(o.dx + r, o.dy), Offset(o.dx + r + 5, o.dy), line(col, 1));
      case KeyS.at:
        c.drawPath(dia, Paint()..color = enabled ? C.playhead : N.g44);
        c.drawPath(dia, line(enabled ? N.g100 : N.g56, 1.5));
    }
  }

  @override
  bool shouldRepaint(_KeyPaint o) => o.s != s || o.hover != hover || o.will != will || o.enabled != enabled;
}

// ---- small shared bits ----

/// A small word-button in the lab's choice-control style (g20 pill, g95 text, accent underline when on).
Widget _gap(double h) => SizedBox(height: h);

/// Count word, never colour alone: "変更 7".
Widget _count(String s, {Color tone = N.g63}) => Text(s, style: T.label(tone));

/// The header mark: two stacked diamonds (layers). Not a box, so it never reads as a checkbox.
class _LayersPaint extends CustomPainter {
  const _LayersPaint();
  @override
  void paint(Canvas c, Size s) {
    final line = Paint()..color = N.g63..style = PaintingStyle.stroke..strokeWidth = 1.2..strokeJoin = StrokeJoin.round;
    Path d(double cy) => Path()..moveTo(8, cy - 4)..lineTo(15, cy)..lineTo(8, cy + 4)..lineTo(1, cy)..close();
    c.drawPath(d(11), line);
    c.drawPath(d(7), line..color = N.g76);
  }

  @override
  bool shouldRepaint(_LayersPaint o) => false;
}
