// The relation system: 15 interactive gadgets, then everything that carries a relation around (chip, badge, menu, card, link line, empty state).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../parts/relations.dart';
import '../../tokens.dart';
import 'relation_parts.dart';
part 'relation_set_2.dart';

// [open] the menu's width and row height (the art shows ~210 x 30).
const _menuW = 220.0, _menuRow = 30.0;
// [open] how big a layer on the connection canvas is.
const _layerW = 112.0, _layerH = 40.0;
// [open] chip heights: compact (lists) and regular.
const _chipC = 20.0, _chipR = 24.0;

void _dash(Canvas cv, Path p, Paint pt, {double on = 4, double off = 4}) {
  for (final m in p.computeMetrics()) {
    for (var d = 0.0; d < m.length; d += on + off) {
      cv.drawPath(m.extractPath(d, math.min(d + on, m.length)), pt);
    }
  }
}

enum _G { plus, chevron, dots, close }

class _Glyph extends StatelessWidget {
  const _Glyph(this.kind, {this.color = N.g76, this.size = 12});
  final _G kind;
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => SizedBox.square(dimension: size, child: CustomPaint(painter: _GlyphPainter(kind, color)));
}

class _GlyphPainter extends CustomPainter {
  const _GlyphPainter(this.kind, this.c);
  final _G kind;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, p = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round;
    switch (kind) {
      case _G.plus:
        cv.drawLine(Offset(w * .5, w * .2), Offset(w * .5, w * .8), p);
        cv.drawLine(Offset(w * .2, w * .5), Offset(w * .8, w * .5), p);
      case _G.chevron:
        cv.drawPath(Path()..moveTo(w * .22, w * .38)..lineTo(w * .5, w * .64)..lineTo(w * .78, w * .38), p);
      case _G.dots:
        for (var i = 0; i < 3; i++) {
          cv.drawCircle(Offset(w * (.2 + i * .3), w * .5), 1.1, Paint()..color = c);
        }
      case _G.close:
        cv.drawLine(Offset(w * .28, w * .28), Offset(w * .72, w * .72), p);
        cv.drawLine(Offset(w * .72, w * .28), Offset(w * .28, w * .72), p);
    }
  }

  @override
  bool shouldRepaint(_GlyphPainter o) => o.kind != kind || o.c != c;
}

/// A hover-aware wrapper: builds with `hover`.
class _Hover extends StatefulWidget {
  const _Hover({required this.builder, this.onTap});
  final Widget Function(bool hover) builder;
  final VoidCallback? onTap;
  @override
  State<_Hover> createState() => _HoverState();
}

class _HoverState extends State<_Hover> {
  bool h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        onEnter: (_) => setState(() => h = true),
        onExit: (_) => setState(() => h = false),
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(h)),
      );
}

// ---------------------------------------------------------------- chip
/// A relation in a list: dot, name, optional value. The dot is the only colour until it is selected.
class RelationChip extends StatelessWidget {
  const RelationChip(this.fam, {super.key, this.look = Look.concept, this.value, this.on = true, this.selected = false, this.compact = false, this.onTap, this.onRemove});
  final Fam fam;
  final Look look;
  final String? value;
  final bool on, selected, compact;
  final VoidCallback? onTap, onRemove;
  @override
  Widget build(BuildContext context) => _Hover(
        onTap: onTap,
        builder: (hover) {
          final tint = look != Look.quiet && selected;
          return AnimatedContainer(
            duration: const Duration(milliseconds: 120),
            curve: Curves.easeOut,
            height: compact ? _chipC : _chipR,
            padding: EdgeInsets.only(left: compact ? 7 : 9, right: onRemove != null ? 4 : (compact ? 7 : 9)),
            decoration: BoxDecoration(
              color: tint ? fam.c.withValues(alpha: .16) : (selected ? N.g20 : (hover ? N.g15 : N.g13)),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: tint ? fam.c.withValues(alpha: .6) : (selected ? N.g44 : N.g20)),
              boxShadow: look == Look.glow && selected ? [BoxShadow(color: fam.c.withValues(alpha: .28), blurRadius: 10)] : null,
            ),
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              Container(width: compact ? 6 : 8, height: compact ? 6 : 8, decoration: BoxDecoration(color: on ? fam.c : N.g38, shape: BoxShape.circle)),
              SizedBox(width: compact ? 5 : 7),
              Text(fam.name, style: T.name(on ? N.g91 : N.g56).copyWith(fontSize: compact ? 10 : 11)),
              if (value != null) ...[const SizedBox(width: 6), Text(value!, style: T.value(N.g56).copyWith(fontSize: 10))],
              if (onRemove != null) ...[
                const SizedBox(width: 3),
                GestureDetector(onTap: onRemove, child: SizedBox(width: 16, height: 16, child: Center(child: _Glyph(_G.close, color: hover ? N.g91 : N.g56, size: 10)))),
              ],
            ]),
          );
        },
      );
}

class _ChipDemo extends StatefulWidget {
  const _ChipDemo(this.look, this.compact, this.values);
  final Look look;
  final bool compact, values;
  @override
  State<_ChipDemo> createState() => _ChipDemoState();
}

class _ChipDemoState extends State<_ChipDemo> {
  int sel = 0;
  final off = <int>{3};
  static const vals = ['0.72', '8 / s', '0.35', '12°', '0.4 s', 'node 2'];
  @override
  Widget build(BuildContext context) => Wrap(spacing: 8, runSpacing: 8, children: [
        for (var i = 0; i < 6; i++)
          RelationChip(Fam.all[i], look: widget.look, compact: widget.compact, value: widget.values ? vals[i] : null, selected: sel == i, on: !off.contains(i), onTap: () => setState(() => sel = i), onRemove: i == 4 ? () {} : null),
      ]);
}

// ---------------------------------------------------------------- badge on a layer row
class RelBadge extends StatelessWidget {
  const RelBadge(this.fam, {super.key, this.style = 1, this.on = true, this.look = Look.concept, this.onTap});
  final Fam fam;
  final int style;
  final bool on;
  final Look look;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => _Hover(
        onTap: onTap,
        builder: (hover) {
          final col = on ? fam.c : N.g38;
          switch (style) {
            case 0:
              return SizedBox(width: 14, height: 14, child: Center(child: AnimatedContainer(duration: const Duration(milliseconds: 120), curve: Curves.easeOut, width: 8, height: 8, decoration: BoxDecoration(color: col, shape: BoxShape.circle, boxShadow: look == Look.glow && on ? [BoxShadow(color: col.withValues(alpha: .6), blurRadius: 6)] : null), foregroundDecoration: hover ? BoxDecoration(shape: BoxShape.circle, border: Border.all(color: N.g95, width: 1)) : null)));
            case 2:
              return SizedBox(width: 8, height: 15, child: Center(child: AnimatedContainer(duration: const Duration(milliseconds: 120), curve: Curves.easeOut, width: 4, height: 13, decoration: BoxDecoration(color: hover ? col.withValues(alpha: 1) : col.withValues(alpha: .8), borderRadius: BorderRadius.circular(2)))));
            default:
              return AnimatedContainer(
                duration: const Duration(milliseconds: 120),
                curve: Curves.easeOut,
                width: 17,
                height: 15,
                alignment: Alignment.center,
                decoration: BoxDecoration(color: hover ? N.g10 : N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g26), boxShadow: look == Look.glow && on ? [BoxShadow(color: fam.c.withValues(alpha: .35), blurRadius: 8)] : null),
                child: FamIcon(fam, size: 11, color: look == Look.quiet && on ? N.g76 : col),
              );
          }
        },
      );
}

class _LayerRows extends StatefulWidget {
  const _LayerRows(this.look, this.style, this.zoom);
  final Look look;
  final int style;
  final double zoom;
  @override
  State<_LayerRows> createState() => _LayerRowsState();
}

class _LayerRowsState extends State<_LayerRows> {
  int sel = 1;
  final off = <String>{'1Stagger'};
  static const rows = [
    ('Gem 01', [0]),
    ('Jewel Field', [0, 2, 1, 3]),
    ('Orbit Ring', [1, 4]),
    ('Camera Null', <int>[]),
    ('Light Rig', [5, 4, 0, 2, 3]),
  ];
  @override
  Widget build(BuildContext context) => Container(
        width: 230 * widget.zoom,
        decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (var r = 0; r < rows.length; r++)
            _Hover(
              onTap: () => setState(() => sel = r),
              builder: (hv) => Container(
                height: TL.height * widget.zoom,
                padding: EdgeInsets.only(left: sel == r ? 8 : 10, right: 8),
                decoration: BoxDecoration(color: sel == r ? N.g20 : (hv ? N.g15 : (r.isEven ? N.bandA : N.bandB)), border: Border(left: sel == r ? const BorderSide(color: N.g95, width: 2) : BorderSide.none, bottom: const BorderSide(color: N.rowLine))),
                child: Row(children: [
                  Expanded(child: Text(rows[r].$1, style: T.name(sel == r ? N.g100 : N.g91))),
                  for (final f in rows[r].$2.take(3))
                    Padding(
                      padding: const EdgeInsets.only(left: 3),
                      child: RelBadge(Fam.all[f], style: widget.style, look: widget.look, on: !off.contains('$r${Fam.all[f].name}'), onTap: () => setState(() {
                            final k = '$r${Fam.all[f].name}';
                            off.contains(k) ? off.remove(k) : off.add(k);
                          })),
                    ),
                  if (rows[r].$2.length > 3) Padding(padding: const EdgeInsets.only(left: 5), child: Text('+${rows[r].$2.length - 3}', style: T.micro(N.g56))),
                ]),
              ),
            ),
        ]),
      );
}

// ---------------------------------------------------------------- add relation menu
class _MenuItem extends StatelessWidget {
  const _MenuItem(this.fam, this.look, this.onTap);
  final Fam? fam;
  final Look look;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Hover(
        onTap: onTap,
        builder: (hover) => AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          height: _menuRow,
          padding: const EdgeInsets.symmetric(horizontal: 9),
          decoration: BoxDecoration(
            color: hover ? (look == Look.quiet ? N.g15 : (look == Look.glow && fam != null ? fam!.c.withValues(alpha: .12) : N.g20)) : const Color(0x00000000),
            borderRadius: BorderRadius.circular(6),
          ),
          child: Row(children: [
            if (fam != null) FamIcon(fam!, size: 16, color: look == Look.quiet && !hover ? N.g63 : fam!.c) else const SizedBox(width: 16, child: Center(child: _Glyph(_G.dots, color: N.g56, size: 16))),
            const SizedBox(width: 10),
            Text(fam?.name ?? 'More…', style: T.name(fam == null ? N.g63 : N.g91)),
            const Spacer(),
            if (fam != null) Text(fam!.jp, style: T.label(N.g44)),
          ]),
        ),
      );
}

class RelationMenuList extends StatelessWidget {
  const RelationMenuList({super.key, this.look = Look.concept, this.onPick});
  final Look look;
  final ValueChanged<Fam>? onPick;
  @override
  Widget build(BuildContext context) => Container(
        width: _menuW,
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20), boxShadow: const [BoxShadow(color: Color(0x80000000), blurRadius: 18, offset: Offset(0, 6))]),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (final f in Fam.all) _MenuItem(f, look, () => onPick?.call(f)),
          _MenuItem(null, look, () {}),
        ]),
      );
}

class _AddButton extends StatelessWidget {
  const _AddButton(this.open, this.look, this.onTap);
  final bool open;
  final Look look;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Hover(
        onTap: onTap,
        builder: (hover) => AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          width: _menuW,
          height: 30,
          padding: const EdgeInsets.symmetric(horizontal: 6),
          decoration: BoxDecoration(color: open || hover ? N.g20 : N.g13, borderRadius: BorderRadius.circular(7), border: Border.all(color: open ? N.g44 : N.g20)),
          child: Row(children: [
            Container(width: 18, height: 18, decoration: BoxDecoration(color: look == Look.quiet ? N.g26 : C.mode, borderRadius: BorderRadius.circular(5)), child: const Center(child: _Glyph(_G.plus, color: N.g100, size: 12))),
            const SizedBox(width: 9),
            Text('Add Relation', style: T.name()),
            const Spacer(),
            AnimatedRotation(turns: open ? .5 : 0, duration: const Duration(milliseconds: 120), curve: Curves.easeOut, child: const _Glyph(_G.chevron, color: N.g63)),
          ]),
        ),
      );
}

class _AddMenuDemo extends StatefulWidget {
  const _AddMenuDemo(this.look, this.startOpen);
  final Look look;
  final bool startOpen;
  @override
  State<_AddMenuDemo> createState() => _AddMenuDemoState();
}

class _AddMenuDemoState extends State<_AddMenuDemo> {
  late bool open = widget.startOpen;
  final added = <Fam>[];
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        _AddButton(open, widget.look, () => setState(() => open = !open)),
        const SizedBox(height: 6),
        if (open) RelationMenuList(look: widget.look, onPick: (f) => setState(() {
              added.add(f);
              open = false;
            })),
        const SizedBox(height: 14),
        Text(added.isEmpty ? 'Pick one to add it to the layer' : 'On this layer', style: T.label(N.g56)),
        const SizedBox(height: 8),
        SizedBox(width: _menuW, child: Wrap(spacing: 6, runSpacing: 6, children: [
          for (var i = 0; i < added.length; i++) RelationChip(added[i], look: widget.look, compact: true, onRemove: () => setState(() => added.removeAt(i))),
        ])),
      ]);
}

// ---------------------------------------------------------------- card
enum CardMode { collapsed, expanded, disabled, error }

class RelCard extends StatefulWidget {
  const RelCard(this.fam, {super.key, this.look = Look.concept, this.mode = CardMode.expanded, this.message = 'Target layer "Gem 04" was deleted'});
  final Fam fam;
  final Look look;
  final CardMode mode;
  final String message;
  @override
  State<RelCard> createState() => _RelCardState();
}

class _RelCardState extends State<RelCard> {
  static const names = {
    'Scatter': ['Density', 'Spread', 'Falloff'],
    'Along Path': ['Start', 'End', 'Spacing'],
    'Stagger': ['Offset', 'Direction', 'Range'],
    'Face': ['Target', 'Smooth', 'Lag'],
    'Follow': ['Target', 'Lag', 'Smooth'],
    'Attach': ['Target', 'Offset', 'Follow'],
  };
  late bool open = widget.mode != CardMode.collapsed, on = widget.mode != CardMode.disabled;
  double density = .72, spread = .48, falloff = .36;
  @override
  void didUpdateWidget(RelCard old) {
    super.didUpdateWidget(old);
    if (old.mode != widget.mode) {
      open = widget.mode != CardMode.collapsed;
      on = widget.mode != CardMode.disabled;
    }
  }

  @override
  Widget build(BuildContext context) {
    final f = widget.fam, look = widget.look, err = widget.mode == CardMode.error;
    final edge = err ? Role.error : f.c;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 120),
      curve: Curves.easeOut,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: look == Look.quiet ? N.g13 : N.g10,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: err ? Role.error.withValues(alpha: .55) : (look == Look.glow && on ? f.c.withValues(alpha: .4) : N.g20)),
        boxShadow: look == Look.glow && on ? [BoxShadow(color: edge.withValues(alpha: .16), blurRadius: 24)] : null,
      ),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
        GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: () => setState(() => open = !open),
          child: Row(children: [
            Container(width: 10, height: 10, decoration: BoxDecoration(color: on ? f.c : N.g38, shape: BoxShape.circle)),
            const SizedBox(width: 9),
            Text(f.name, style: T.title(on ? N.g95 : N.g56)),
            const SizedBox(width: 8),
            Text(f.jp, style: T.label(N.g44)),
            if (err) ...[const SizedBox(width: 8), const ErrMark(gap: 4), Text('ERROR', style: T.micro(Role.error).copyWith(letterSpacing: .6))],
            const Spacer(),
            PillSwitch(on: on, fam: f, onChanged: (v) => setState(() => on = v)),
            const SizedBox(width: 8),
            AnimatedRotation(turns: open ? .5 : 0, duration: const Duration(milliseconds: 120), curve: Curves.easeOut, child: const _Glyph(_G.chevron, color: N.g56, size: 14)),
          ]),
        ),
        AnimatedSize(
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          alignment: Alignment.topCenter,
          child: !open
              ? const SizedBox(width: double.infinity)
              : Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                    if (err)
                      Container(
                        margin: const EdgeInsets.only(bottom: 10),
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                        decoration: BoxDecoration(color: Role.error.withValues(alpha: .1), borderRadius: BorderRadius.circular(6), border: Border.all(color: Role.error.withValues(alpha: .4))),
                        child: Row(children: [
                          const ErrMark(gap: 6),
                          Expanded(child: Text(widget.message, style: T.label(N.g91))),
                          const SizedBox(width: 8),
                          _Hover(builder: (h) => Container(height: 22, padding: const EdgeInsets.symmetric(horizontal: 10), alignment: Alignment.center, decoration: BoxDecoration(color: h ? Role.error.withValues(alpha: .16) : const Color(0x00000000), borderRadius: BorderRadius.circular(6), border: Border.all(color: Role.error)), child: Text('Relink', style: T.name(N.g91)))),
                        ]),
                      ),
                    Opacity(
                      opacity: on ? 1 : .42,
                      child: IgnorePointer(
                        ignoring: !on || err,
                        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                          gadgetFor(f, look: look, size: 132),
                          const SizedBox(width: 12),
                          Expanded(child: Column(mainAxisSize: MainAxisSize.min, children: [
                            ValueWell(label: names[f.name]![0], value: density, fam: f, onChanged: (v) => setState(() => density = v)),
                            const SizedBox(height: 6),
                            ValueWell(label: names[f.name]![1], value: spread, fam: f, onChanged: (v) => setState(() => spread = v)),
                            const SizedBox(height: 6),
                            ValueWell(label: names[f.name]![2], value: falloff, fam: f, onChanged: (v) => setState(() => falloff = v)),
                          ])),
                        ]),
                      ),
                    ),
                  ]),
                ),
        ),
      ]),
    );
  }
}

// ---------------------------------------------------------------- connection line
class _Link {
  _Link(this.path, this.a, this.b);
  final Path path;
  final Offset a, b;
}

_Link _link(Offset pa, Offset pb, int style) {
  final ra = Rect.fromLTWH(pa.dx, pa.dy, _layerW, _layerH), rb = Rect.fromLTWH(pb.dx, pb.dy, _layerW, _layerH);
  final d = rb.center - ra.center, horiz = d.dx.abs() * .55 >= d.dy.abs();
  final Offset a, b;
  if (horiz) {
    a = d.dx >= 0 ? ra.centerRight : ra.centerLeft;
    b = d.dx >= 0 ? rb.centerLeft : rb.centerRight;
  } else {
    a = d.dy >= 0 ? ra.bottomCenter : ra.topCenter;
    b = d.dy >= 0 ? rb.topCenter : rb.bottomCenter;
  }
  final p = Path()..moveTo(a.dx, a.dy);
  switch (style) {
    case 0:
      p.lineTo(b.dx, b.dy);
    case 1:
      if (horiz) {
        p.cubicTo(a.dx + (b.dx - a.dx) / 2, a.dy, b.dx - (b.dx - a.dx) / 2, b.dy, b.dx, b.dy);
      } else {
        p.cubicTo(a.dx, a.dy + (b.dy - a.dy) / 2, b.dx, b.dy - (b.dy - a.dy) / 2, b.dx, b.dy);
      }
    default:
      if (horiz) {
        final mx = (a.dx + b.dx) / 2;
        p..lineTo(mx, a.dy)..lineTo(mx, b.dy)..lineTo(b.dx, b.dy);
      } else {
        final my = (a.dy + b.dy) / 2;
        p..lineTo(a.dx, my)..lineTo(b.dx, my)..lineTo(b.dx, b.dy);
      }
  }
  return _Link(p, a, b);
}
