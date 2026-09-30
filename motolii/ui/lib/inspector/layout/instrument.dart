// The Layout Instrument: one container diagram everything acts on, with the exact values beside it.
// Classic gating is kept: Grid enables the rest, Fixed enables its size number, Hug / Fill / Fixed stay distinct,
// the child's own lines and the Advanced rows stay reachable.
import 'package:flutter/widgets.dart';
import '../../browser/parts.dart';

import 'diagram.dart';
import 'model.dart';
import '../panel.dart' show ParamCell;
import '../value_controls.dart';
import '../../theme/neutral.dart';
import '../../theme/surface.dart' show Dn, Surface;

class LayoutInstrument extends StatefulWidget {
  const LayoutInstrument(this.store, {super.key, this.title = 'Group', this.advancedOpen = false, this.embedded = false});
  final LayoutStore store;

  /// Inside a card of a longer scrolling panel: take the height the content needs instead of scrolling by itself.
  final bool embedded;
  final String title;
  final bool advancedOpen;
  @override
  State<LayoutInstrument> createState() => _LayoutInstrumentState();
}

class _LayoutInstrumentState extends State<LayoutInstrument> {
  late bool advancedOpen = widget.advancedOpen;
  LayoutStore get s => widget.store;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: s,
        builder: (context, _) => LayoutBuilder(builder: (context, box) {
          final narrow = box.maxWidth < 172;
          _narrow = narrow;
          final pad = narrow ? 6.0 : 9.0;
          final w = box.maxWidth - pad * 2;
          final body = Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _header(),
            const SizedBox(height: 6),
            if (s.child) ..._childBody(narrow) else ..._groupBody(w, narrow),
          ]);
          if (widget.embedded) return Padding(padding: EdgeInsets.fromLTRB(pad, 7.5, pad, 12), child: body);
          return narrow
              ? SingleChildScrollView(key: const ValueKey('layout-scroll'), padding: EdgeInsets.fromLTRB(pad, 7.5, pad, 12), child: body)
              : SingleChildScrollView(key: const ValueKey('layout-scroll'), padding: EdgeInsets.fromLTRB(pad, 7.5, pad, 12), child: body);
        }),
      );

  Widget _header() => SizedBox(
        height: 18,
        child: Row(children: [
          Container(width: 2, height: 10.5, margin: const EdgeInsets.only(right: 5), decoration: BoxDecoration(color: arrangeColor, borderRadius: BorderRadius.circular(1.5))),
          Expanded(child: Text(s.child ? 'Child in layout' : widget.title, key: const ValueKey('layout-title'), softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600))),
          if (s.frozen) Padding(padding: const EdgeInsets.only(right: 6), child: Text('Locked', style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w600))),
          if (!s.child) _switch('grid-switch', 'Grid', s.gridOn, () => s.setGrid(!s.gridOn), arrangeColor)
          else _switch('ignore-switch', 'Ignore layout', s.gi('layout.position_type') == 1, () => s.set('layout.position_type', s.gi('layout.position_type') == 1 ? 0 : 1), sizeColor),
        ]),
      );

  Widget _switch(String key, String label, bool on, VoidCallback f, Color tone) => GestureDetector(
        key: ValueKey(key),
        behavior: HitTestBehavior.opaque,
        onTap: s.frozen ? null : f,
        child: Row(children: [
          Text(label, style: sans(Dn.labelSize, c: on ? tone : Surface.muted, w: FontWeight.w600)),
          const SizedBox(width: 5),
          Container(width: 22.5, height: 13, padding: const EdgeInsets.all(1.5), alignment: on ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: on ? (s.frozen ? dim(tone) : tone) : N.g26, borderRadius: BorderRadius.circular(7)), child: Container(width: 10, height: 10, decoration: const BoxDecoration(color: Surface.ink, shape: BoxShape.circle))),
        ]),
      );

  Color dim(Color c) => Color.lerp(c, N.g26, .7)!;

  Widget _val(String id, Color tone, {String? tag, int? axis, bool gated = true, bool units = false, bool whole = true}) => Expanded(
        child: Opacity(
          opacity: gated ? 1 : .45,
          child: IgnorePointer(ignoring: !gated, child: ValueToy(Slot(s, id, axis), tag: tag, tone: tone, showUnit: units && !_narrow, decimals: whole ? 0 : null)),
        ),
      );

  bool _narrow = false;

  List<Widget> _groupBody(double w, bool narrow) {
    final on = s.arranged;
    final adv = [for (final r in s.rows) if (r['advanced'] == true) r];
    return [
      LayoutDiagram(s, size: Size(w, narrow ? 117 : 147)),
      const SizedBox(height: 6),
      // Columns and Rows stay with Grid off, as Classic's wells do; the rest is gated by Grid
      if (narrow) ...[
        Row(children: [_val('layout.grid_columns', arrangeColor, tag: 'C'), const SizedBox(width: 2), _val('layout.grid_rows', arrangeColor, tag: 'R')]),
        if (on) ...[const SizedBox(height: 2), Row(children: [_val('layout.gap', spaceColor, tag: 'Gap')]), const SizedBox(height: 2), Row(children: [_val('layout.padding', spaceColor, tag: 'X', axis: 0), const SizedBox(width: 2), _val('layout.padding', spaceColor, tag: 'Y', axis: 1)])],
      ] else ...[
        Row(children: [_val('layout.grid_columns', arrangeColor, tag: 'Col'), const SizedBox(width: 2), _val('layout.grid_rows', arrangeColor, tag: 'Row'), if (on) ...[const SizedBox(width: 2), _val('layout.gap', spaceColor, tag: 'Gap')]]),
        if (on) ...[const SizedBox(height: 3), Row(children: [_val('layout.padding', spaceColor, tag: 'Pad X', axis: 0), const SizedBox(width: 2), _val('layout.padding', spaceColor, tag: 'Pad Y', axis: 1)])],
      ],
      if (on) ...[
        const SizedBox(height: 6),
        _sizeLine('w'),
        const SizedBox(height: 3),
        _sizeLine('h'),
        const SizedBox(height: 6),
        if (narrow) ...[Row(children: [_val('layout.transition_duration', spaceColor, tag: 'Dur', whole: false)]), const SizedBox(height: 2), ChoiceToy(s, 'layout.transition_easing', tone: alignColor)]
        else Row(children: [_val('layout.transition_duration', spaceColor, tag: 'Dur', units: true, whole: false), const SizedBox(width: 4.5), Expanded(flex: 2, child: ChoiceToy(s, 'layout.transition_easing', tone: alignColor))]),
        ..._advanced(adv),
      ],
    ];
  }

  /// Every other layout row the document declares, behind one fold (Classic IN-091): for a group while Grid is on
  /// (Grid gates it, as in Classic), and for a laid-out child.
  List<Widget> _advanced(List<Map<String, dynamic>> adv) => [
        if (adv.isNotEmpty) ...[
          GestureDetector(
            key: const ValueKey('layout-advanced'),
            behavior: HitTestBehavior.opaque,
            onTap: () => setState(() => advancedOpen = !advancedOpen),
            child: Padding(padding: const EdgeInsets.only(top: 10.5, bottom: 4.5), child: Row(children: [Text(advancedOpen ? '▾' : '▸', style: sans(Dn.labelSize, c: Surface.muted)), const SizedBox(width: 4.5), Text('ADVANCED', style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.3)), const SizedBox(width: 4.5), Text('${adv.length}', style: mono(Dn.microSize, c: N.g51))])),
          ),
          if (advancedOpen) for (final r in adv) Padding(padding: const EdgeInsets.only(bottom: 4.5), child: ParamCell(s, r, tone: Surface.muted)),
        ],
      ];

  // Hug / Fill / Fixed are three different relationships to the space offered; the number counts under Fixed alone
  Widget _sizeLine(String axis) {
    final id = 'layout.${axis == 'w' ? 'horizontal' : 'vertical'}_sizing';
    final cur = s.gi(id);
    final fixed = cur == 2;
    final chips = [
        SizedBox(width: 13.5, child: Text(axis.toUpperCase(), style: sans(Dn.labelSize, c: sizeColor, w: FontWeight.w700))),
        for (var i = 0; i < 3; i++)
          GestureDetector(
            key: ValueKey('size-$axis-$i'),
            behavior: HitTestBehavior.opaque,
            onTap: s.frozen ? null : () => s.setSizing(axis, i),
            child: Container(width: 19.5, height: 19.5, margin: const EdgeInsets.only(right: 2), decoration: BoxDecoration(color: cur == i ? (s.frozen ? dim(sizeColor) : sizeColor) : Surface.raised, borderRadius: BorderRadius.circular(4)), child: Center(child: SizedBox(width: 10.5, height: 9, child: CustomPaint(painter: SizingGlyph(i, cur == i ? N.g10 : Surface.muted))))),
          ),
    ];
    final value = _val('layout.${axis == 'w' ? 'width' : 'height'}', sizeColor, gated: fixed, units: true);
    // narrow: the three relationships first, the number under them
    return _narrow
        ? Column(children: [SizedBox(height: 22.5, child: Row(children: chips)), const SizedBox(height: 2), SizedBox(height: 22.5, child: Row(children: [value])), const SizedBox(height: 3)])
        : SizedBox(height: 22.5, child: Row(children: [...chips, const SizedBox(width: 2), value]));
  }

  List<Widget> _childBody(bool narrow) => [
        _sizeLine('w'),
        const SizedBox(height: 3),
        _sizeLine('h'),
        if (s.rows.any((r) => r['id'] == 'layout.column_start')) ...[
          Padding(padding: const EdgeInsets.only(top: 10.5, bottom: 4.5), child: Text('GRID AREA', style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.3))),
          Row(children: [_val('layout.column_start', arrangeColor, tag: 'Col'), const SizedBox(width: 2), _val('layout.row_start', arrangeColor, tag: 'Row')]),
          const SizedBox(height: 3),
          Row(children: [_val('layout.column_span', arrangeColor, tag: 'Cols'), const SizedBox(width: 2), _val('layout.row_span', arrangeColor, tag: 'Rows')]),
        ],
        ..._advanced([for (final r in s.rows) if (r['advanced'] == true) r]),
      ];
}

/// Hug (walls close in), Fill (walls reach out), Fixed (a set size).
class SizingGlyph extends CustomPainter {
  SizingGlyph(this.i, this.col);
  final int i;
  final Color col;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final f = Paint()..color = col;
    final w = s.width, h = s.height, cy = h / 2;
    switch (i) {
      case 0:
        c.drawLine(const Offset(0, 1), Offset(0, h - 1), p);
        c.drawLine(Offset(w, 1), Offset(w, h - 1), p);
        c.drawPath(Path()..moveTo(2.5, cy - 2.4)..lineTo(5.5, cy)..lineTo(2.5, cy + 2.4)..close(), f);
        c.drawPath(Path()..moveTo(w - 2.5, cy - 2.4)..lineTo(w - 5.5, cy)..lineTo(w - 2.5, cy + 2.4)..close(), f);
      case 1:
        c.drawLine(Offset(w * .3, cy), Offset(w * .7, cy), p);
        c.drawPath(Path()..moveTo(w * .3, cy - 2.6)..lineTo(0, cy)..lineTo(w * .3, cy + 2.6)..close(), f);
        c.drawPath(Path()..moveTo(w * .7, cy - 2.6)..lineTo(w, cy)..lineTo(w * .7, cy + 2.6)..close(), f);
      default:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .22, cy - 1, w * .56, cy + 1.5), const Radius.circular(1.5)), f);
        c.drawArc(Rect.fromLTWH(w * .32, 0, w * .36, cy + 1), 3.14, 3.14, false, p);
    }
  }
  @override
  bool shouldRepaint(SizingGlyph o) => o.i != i || o.col != col;
}
