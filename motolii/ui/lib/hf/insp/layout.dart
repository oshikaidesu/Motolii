// The Layout Instrument: one container diagram everything acts on, with the exact values beside it.
// Classic gating is kept: Grid enables the rest, Fixed enables its size number, Hug / Fill / Fixed stay distinct,
// the child's own lines and the Advanced rows stay reachable.
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../desk/common.dart' show kInk;
import 'layout_diagram.dart';
import 'layout_model.dart';
import 'panel.dart' show ParamCell;
import 'toys.dart';

class LayoutInstrument extends StatefulWidget {
  const LayoutInstrument(this.store, {super.key, this.title = 'Group', this.advancedOpen = false});
  final LayoutStore store;
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
          final narrow = box.maxWidth < 230;
          _narrow = narrow;
          final pad = narrow ? 8.0 : 12.0;
          final w = box.maxWidth - pad * 2;
          final body = Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _header(),
            const SizedBox(height: 8),
            if (s.child) ..._childBody(narrow) else ..._groupBody(w, narrow),
          ]);
          return narrow
              ? SingleChildScrollView(key: const ValueKey('layout-scroll'), padding: EdgeInsets.fromLTRB(pad, 10, pad, 16), child: body)
              : SingleChildScrollView(key: const ValueKey('layout-scroll'), padding: EdgeInsets.fromLTRB(pad, 10, pad, 16), child: body);
        }),
      );

  Widget _header() => SizedBox(
        height: 24,
        child: Row(children: [
          Container(width: 3, height: 14, margin: const EdgeInsets.only(right: 7), decoration: BoxDecoration(color: arrangeColor, borderRadius: BorderRadius.circular(1.5))),
          Expanded(child: Text(s.child ? 'Child in layout' : widget.title, key: const ValueKey('layout-title'), softWrap: false, overflow: TextOverflow.clip, style: sans(13, c: kInk, w: FontWeight.w600))),
          if (s.frozen) Padding(padding: const EdgeInsets.only(right: 8), child: Text('Locked', style: sans(9.5, c: kMuted, w: FontWeight.w600))),
          if (!s.child) _switch('grid-switch', 'Grid', s.gridOn, () => s.setGrid(!s.gridOn), arrangeColor)
          else _switch('ignore-switch', 'Ignore layout', s.gi('layout.position_type') == 1, () => s.set('layout.position_type', s.gi('layout.position_type') == 1 ? 0 : 1), sizeColor),
        ]),
      );

  Widget _switch(String key, String label, bool on, VoidCallback f, Color tone) => GestureDetector(
        key: ValueKey(key),
        behavior: HitTestBehavior.opaque,
        onTap: s.frozen ? null : f,
        child: Row(children: [
          Text(label, style: sans(10.5, c: on ? tone : kMuted, w: FontWeight.w600)),
          const SizedBox(width: 7),
          Container(width: 30, height: 17, padding: const EdgeInsets.all(2), alignment: on ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: on ? (s.frozen ? dim(tone) : tone) : const Color(0xFF3A3B40), borderRadius: BorderRadius.circular(9)), child: Container(width: 13, height: 13, decoration: const BoxDecoration(color: kInk, shape: BoxShape.circle))),
        ]),
      );

  Color dim(Color c) => Color.lerp(c, const Color(0xFF3A3B40), .7)!;

  Widget _val(String id, Color tone, {String? tag, int? axis, bool gated = true, bool units = false, bool whole = true}) => Expanded(
        child: Opacity(
          opacity: gated ? 1 : .45,
          child: IgnorePointer(ignoring: !gated, child: ValueToy(Slot(s, id, axis), tag: tag, tone: tone, showUnit: units && !_narrow, decimals: whole ? 0 : null)),
        ),
      );

  bool _narrow = false;

  List<Widget> _groupBody(double w, bool narrow) {
    final on = s.gridOn;
    final adv = [for (final r in s.rows) if (r['advanced'] == true) r];
    return [
      LayoutDiagram(s, size: Size(w, narrow ? 156 : 196)),
      const SizedBox(height: 8),
      // Columns and Rows stay with Grid off, as Classic's wells do; the rest is gated by Grid
      if (narrow) ...[
        Row(children: [_val('layout.grid_columns', arrangeColor, tag: 'C'), const SizedBox(width: 3), _val('layout.grid_rows', arrangeColor, tag: 'R')]),
        if (on) ...[const SizedBox(height: 3), Row(children: [_val('layout.gap', spaceColor, tag: 'Gap')]), const SizedBox(height: 3), Row(children: [_val('layout.padding', spaceColor, tag: 'X', axis: 0), const SizedBox(width: 3), _val('layout.padding', spaceColor, tag: 'Y', axis: 1)])],
      ] else ...[
        Row(children: [_val('layout.grid_columns', arrangeColor, tag: 'Col'), const SizedBox(width: 3), _val('layout.grid_rows', arrangeColor, tag: 'Row'), if (on) ...[const SizedBox(width: 3), _val('layout.gap', spaceColor, tag: 'Gap')]]),
        if (on) ...[const SizedBox(height: 4), Row(children: [_val('layout.padding', spaceColor, tag: 'Pad X', axis: 0), const SizedBox(width: 3), _val('layout.padding', spaceColor, tag: 'Pad Y', axis: 1)])],
      ],
      if (on) ...[
        const SizedBox(height: 8),
        _sizeLine('w'),
        const SizedBox(height: 4),
        _sizeLine('h'),
        const SizedBox(height: 8),
        if (narrow) ...[Row(children: [_val('layout.transition_duration', spaceColor, tag: 'Dur', whole: false)]), const SizedBox(height: 3), ChoiceToy(s, 'layout.transition_easing', tone: alignColor)]
        else Row(children: [_val('layout.transition_duration', spaceColor, tag: 'Dur', units: true, whole: false), const SizedBox(width: 6), Expanded(flex: 2, child: ChoiceToy(s, 'layout.transition_easing', tone: alignColor))]),
        if (adv.isNotEmpty) ...[
          GestureDetector(
            key: const ValueKey('layout-advanced'),
            behavior: HitTestBehavior.opaque,
            onTap: () => setState(() => advancedOpen = !advancedOpen),
            child: Padding(padding: const EdgeInsets.only(top: 14, bottom: 6), child: Row(children: [Text(advancedOpen ? '▾' : '▸', style: sans(10, c: kMuted)), const SizedBox(width: 6), Text('ADVANCED', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.3)), const SizedBox(width: 6), Text('${adv.length}', style: mono(9.5, c: const Color(0xFF7E7F86)))])),
          ),
          if (advancedOpen) for (final r in adv) Padding(padding: const EdgeInsets.only(bottom: 6), child: ParamCell(s, r, tone: kMutedTone)),
        ],
      ],
    ];
  }

  static const kMutedTone = Color(0xFF8E8F92);

  // Hug / Fill / Fixed are three different relationships to the space offered; the number counts under Fixed alone
  Widget _sizeLine(String axis) {
    final id = 'layout.${axis == 'w' ? 'horizontal' : 'vertical'}_sizing';
    final cur = s.gi(id);
    final fixed = cur == 2;
    final chips = [
        SizedBox(width: 18, child: Text(axis.toUpperCase(), style: sans(10.5, c: sizeColor, w: FontWeight.w700))),
        for (var i = 0; i < 3; i++)
          GestureDetector(
            key: ValueKey('size-$axis-$i'),
            behavior: HitTestBehavior.opaque,
            onTap: s.frozen ? null : () => s.setSizing(axis, i),
            child: Container(width: 26, height: 26, margin: const EdgeInsets.only(right: 3), decoration: BoxDecoration(color: cur == i ? (s.frozen ? dim(sizeColor) : sizeColor) : kTile, borderRadius: BorderRadius.circular(5)), child: Center(child: SizedBox(width: 14, height: 12, child: CustomPaint(painter: SizingGlyph(i, cur == i ? const Color(0xFF1B1B1D) : kMuted))))),
          ),
    ];
    final value = _val('layout.${axis == 'w' ? 'width' : 'height'}', sizeColor, gated: fixed, units: true);
    // narrow: the three relationships first, the number under them
    return _narrow
        ? Column(children: [SizedBox(height: 30, child: Row(children: chips)), const SizedBox(height: 3), SizedBox(height: 30, child: Row(children: [value])), const SizedBox(height: 4)])
        : SizedBox(height: 30, child: Row(children: [...chips, const SizedBox(width: 3), value]));
  }

  List<Widget> _childBody(bool narrow) => [
        _sizeLine('w'),
        const SizedBox(height: 4),
        _sizeLine('h'),
        if (s.rows.any((r) => r['id'] == 'layout.column_start')) ...[
          Padding(padding: const EdgeInsets.only(top: 14, bottom: 6), child: Text('GRID AREA', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.3))),
          Row(children: [_val('layout.column_start', arrangeColor, tag: 'Col'), const SizedBox(width: 3), _val('layout.row_start', arrangeColor, tag: 'Row')]),
          const SizedBox(height: 4),
          Row(children: [_val('layout.column_span', arrangeColor, tag: 'Cols'), const SizedBox(width: 3), _val('layout.row_span', arrangeColor, tag: 'Rows')]),
        ],
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
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .22, cy - 1, w * .56, cy + 1.5), const Radius.circular(2)), f);
        c.drawArc(Rect.fromLTWH(w * .32, 0, w * .36, cy + 1), 3.14, 3.14, false, p);
    }
  }
  @override
  bool shouldRepaint(SizingGlyph o) => o.i != i || o.col != col;
}
