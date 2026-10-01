// The relation system: 15 interactive gadgets, then everything that carries a relation around (chip, badge, menu, card, link line, empty state).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../kit.dart';
import '../parts/controls.dart';
import '../parts/relations.dart';
import '../tokens.dart';
import 'relation_parts.dart';

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
    final edge = err ? C.danger : f.c;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 120),
      curve: Curves.easeOut,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: look == Look.quiet ? N.g13 : N.g10,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: err ? C.danger.withValues(alpha: .55) : (look == Look.glow && on ? f.c.withValues(alpha: .4) : N.g20)),
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
            if (err) ...[const SizedBox(width: 8), Text('ERROR', style: T.micro(C.danger).copyWith(letterSpacing: .6))],
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
                        decoration: BoxDecoration(color: C.danger.withValues(alpha: .1), borderRadius: BorderRadius.circular(6), border: Border.all(color: C.danger.withValues(alpha: .4))),
                        child: Row(children: [
                          Expanded(child: Text(widget.message, style: T.label(N.g91))),
                          const SizedBox(width: 8),
                          _Hover(builder: (h) => Container(height: 22, padding: const EdgeInsets.symmetric(horizontal: 10), alignment: Alignment.center, decoration: BoxDecoration(color: h ? C.danger.withValues(alpha: .16) : const Color(0x00000000), borderRadius: BorderRadius.circular(6), border: Border.all(color: C.danger)), child: Text('Relink', style: T.name(N.g91)))),
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

class _LinkPainter extends CustomPainter {
  _LinkPainter(this.link, this.k, this.dashed, this.flow, this.t) : super(repaint: null);
  final _Link link;
  final Skin k;
  final bool dashed, flow;
  final double t;
  @override
  void paint(Canvas cv, Size s) {
    for (var x = 12.0; x < s.width; x += 24) {
      for (var y = 12.0; y < s.height; y += 24) {
        cv.drawCircle(Offset(x, y), .8, Paint()..color = N.g20);
      }
    }
    if (dashed) {
      if (k.glow) {
        for (final m in link.path.computeMetrics()) {
          for (var d = 0.0; d < m.length; d += 9) {
            cv.drawPath(m.extractPath(d, math.min(d + 5, m.length)), k.stroke(k.c.withValues(alpha: .3), 5)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
          }
        }
      }
      _dash(cv, link.path, k.stroke(k.c, 1.6), on: 5, off: 4);
    } else {
      k.line(cv, link.path, w: 1.6);
    }
    final m = link.path.computeMetrics().first, end = m.getTangentForOffset(m.length)!, u = Offset.fromDirection(-end.angle);
    k.dot(cv, link.a, 4);
    cv.drawCircle(link.a, 4, k.stroke(N.g100, 1.2));
    final head = Path()
      ..moveTo(link.b.dx, link.b.dy)
      ..lineTo(link.b.dx - u.dx * 9 + u.dy * 4.5, link.b.dy - u.dy * 9 - u.dx * 4.5)
      ..lineTo(link.b.dx - u.dx * 9 - u.dy * 4.5, link.b.dy - u.dy * 9 + u.dx * 4.5)
      ..close();
    cv.drawPath(head, k.fill(k.c));
    if (flow) {
      final f = (t % 1.0), pos = m.getTangentForOffset(m.length * f)!.position;
      k.dot(cv, pos, 2.6, col: N.g95);
    }
  }

  @override
  bool shouldRepaint(_LinkPainter o) => true;
}

class _ConnectionDemo extends StatefulWidget {
  const _ConnectionDemo({required this.look, required this.fam, required this.style, required this.dashed, required this.label, required this.flow});
  final Look look;
  final Fam fam;
  final int style;
  final bool dashed, label, flow;
  @override
  State<_ConnectionDemo> createState() => _ConnectionDemoState();
}

class _ConnectionDemoState extends State<_ConnectionDemo> with SingleTickerProviderStateMixin {
  Offset a = const Offset(40, 50), b = const Offset(330, 190);
  late final AnimationController clock = AnimationController(vsync: this, duration: const Duration(seconds: 2))..repeat();
  static const w = 520.0, h = 300.0;
  @override
  void dispose() {
    clock.dispose();
    super.dispose();
  }

  Widget layer(String name, Offset p, ValueChanged<Offset> set, bool badge) => Positioned(
        left: p.dx,
        top: p.dy,
        child: GestureDetector(
          onPanUpdate: (d) => set(Offset((p.dx + d.delta.dx).clamp(0.0, w - _layerW), (p.dy + d.delta.dy).clamp(0.0, h - _layerH))),
          child: MouseRegion(
            cursor: SystemMouseCursors.grab,
            child: Container(
              width: _layerW,
              height: _layerH,
              padding: const EdgeInsets.symmetric(horizontal: 10),
              decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26)),
              child: Row(children: [
                Expanded(child: Text(name, style: T.name())),
                if (badge) FamIcon(widget.fam, size: 13),
              ]),
            ),
          ),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final link = _link(a, b, widget.style), m = link.path.computeMetrics().first, mid = m.getTangentForOffset(m.length / 2)!.position;
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: Container(
        width: w,
        height: h,
        color: N.g07,
        child: Stack(children: [
          Positioned.fill(
            child: AnimatedBuilder(
              animation: clock,
              builder: (_, _) => CustomPaint(painter: _LinkPainter(link, Skin(widget.look, widget.fam.c), widget.dashed, widget.flow, clock.value)),
            ),
          ),
          layer('Leader', a, (p) => setState(() => a = p), false),
          layer('Gem 04', b, (p) => setState(() => b = p), true),
          if (widget.label)
            Positioned(left: mid.dx, top: mid.dy, child: FractionalTranslation(translation: const Offset(-.5, -.5), child: RelationChip(widget.fam, look: widget.look, compact: true, selected: true))),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- empty state
class _EmptyDemo extends StatefulWidget {
  const _EmptyDemo(this.look);
  final Look look;
  @override
  State<_EmptyDemo> createState() => _EmptyDemoState();
}

class _EmptyDemoState extends State<_EmptyDemo> {
  final picked = <Fam>[];
  @override
  Widget build(BuildContext context) {
    final look = widget.look;
    return SizedBox(
      width: 320,
      child: Container(
          padding: const EdgeInsets.fromLTRB(20, 22, 20, 20),
          decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            if (picked.isEmpty) ...[
              SizedBox(
                height: 34,
                child: Row(mainAxisAlignment: MainAxisAlignment.center, children: [
                  for (var i = 0; i < 3; i++)
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 6),
                      child: FamIcon(Fam.all[i * 2], size: 24, color: N.g56),
                    ),
                ]),
              ),
              const SizedBox(height: 14),
              Text('No relations yet', style: T.title(N.g76)),
              const SizedBox(height: 6),
              Text('まだ関係がありません', style: T.label(N.g44)),
              const SizedBox(height: 4),
              Text('Connect this layer to another to start.', style: T.label(N.g56)),
              const SizedBox(height: 16),
              _Hover(
                onTap: () => setState(() => picked.add(Fam.scatter)),
                builder: (h) => AnimatedContainer(
                  duration: const Duration(milliseconds: 120),
                  curve: Curves.easeOut,
                  height: 28,
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  decoration: BoxDecoration(color: h ? N.g20 : N.g13, borderRadius: BorderRadius.circular(7), border: Border.all(color: h ? N.g44 : N.g20)),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    const _Glyph(_G.plus, color: N.g91),
                    const SizedBox(width: 7),
                    Text('Add Relation', style: T.name()),
                  ]),
                ),
              ),
              const SizedBox(height: 14),
              Text('START WITH', style: T.micro(N.g44).copyWith(letterSpacing: .8)),
              const SizedBox(height: 8),
              Wrap(spacing: 6, runSpacing: 6, alignment: WrapAlignment.center, children: [
                for (final f in Fam.all) RelationChip(f, look: look, compact: true, onTap: () => setState(() => picked.add(f))),
              ]),
            ] else ...[
              Wrap(spacing: 6, runSpacing: 6, children: [
                for (var i = 0; i < picked.length; i++) RelationChip(picked[i], look: look, onRemove: () => setState(() => picked.removeAt(i))),
              ]),
              const SizedBox(height: 10),
              Text('Remove them all to see the empty state again.', style: T.label(N.g56)),
            ],
          ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- usage flow (add -> tune -> see)
class _ResultPainter extends CustomPainter {
  _ResultPainter(this.k, this.centre, this.radius, this.curve) : super(repaint: centre);
  final Skin k;
  final ValueNotifier<(Offset, double)> centre;
  final double radius, curve;
  @override
  void paint(Canvas cv, Size s) {
    final (c, r) = centre.value;
    final rnd = math.Random(11);
    for (var i = 0; i < 110; i++) {
      final p = Offset(rnd.nextDouble(), rnd.nextDouble()), d = (p - c).distance / math.max(.05, r), f = math.pow(1 - d.clamp(0.0, 1.0), curve).toDouble();
      k.dot(cv, p * s.width, 1.2 + 3.6 * f, a: .25 + .75 * f);
    }
  }

  @override
  bool shouldRepaint(_ResultPainter o) => o.k.look != k.look;
}

class _Flow extends StatefulWidget {
  const _Flow(this.look, this.curve);
  final Look look;
  final double curve;
  @override
  State<_Flow> createState() => _FlowState();
}

class _FlowState extends State<_Flow> {
  final centre = ValueNotifier<(Offset, double)>((const Offset(.5, .5), .3));
  @override
  void dispose() {
    centre.dispose();
    super.dispose();
  }

  Widget step(String no, String en, String jp, Widget child) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Text('$no  $en', style: T.name(N.g76)),
        const SizedBox(height: 4),
        Text(jp, style: T.label(N.g56)),
        const SizedBox(height: 10),
        child,
      ]);

  Widget arrow() => const Padding(padding: EdgeInsets.fromLTRB(14, 120, 14, 0), child: RotatedBox(quarterTurns: 3, child: _Glyph(_G.chevron, color: N.g44, size: 22)));

  @override
  Widget build(BuildContext context) => Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        step('1', 'Add a relation', 'Relationを追加', RelationMenuList(look: widget.look)),
        arrow(),
        step('2', 'Tune with the gadget', 'ガジェットで調整', FalloffGadget(look: widget.look, curve: widget.curve, onChanged: (c, r) => centre.value = (c, r))),
        arrow(),
        step('3', 'See it at once', 'すぐに反映', Container(
          width: 200,
          height: 200,
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
          child: ClipRRect(borderRadius: BorderRadius.circular(8), child: CustomPaint(painter: _ResultPainter(Skin(widget.look, Fam.scatter.c), centre, 0, widget.curve))),
        )),
      ]);
}

// ---------------------------------------------------------------- use cases
Widget _scroll(Widget child) => Stack(children: [
      SingleChildScrollView(padding: const EdgeInsets.only(bottom: 24), child: child),
      Positioned(left: 0, right: 0, bottom: 0, height: 24, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [N.g07.withValues(alpha: 0), N.g07]))))),
    ]);

Widget _page(List<Widget> kids) => _scroll(Wrap(spacing: 14, runSpacing: 14, children: kids));

WidgetbookComponent relationSetSet() => WidgetbookComponent(name: 'relation_set', useCases: [
      uc('Gadgets 1-4 · Space (Falloff, Direction, Scatter, Region)', (c) {
        final look = lookKnob(c);
        return _page([
          FalloffGadget(look: look, size: 150, curve: c.knobs.double.slider(label: 'Falloff curve', initialValue: 1.6, min: .4, max: 4), rings: c.knobs.int.slider(label: 'Rings', initialValue: 3, min: 1, max: 6)),
          DirectionGadget(look: look, size: 150, pull: c.knobs.double.slider(label: 'Attract (0 = direction only)', initialValue: .55, min: 0, max: 1), cells: c.knobs.int.slider(label: 'Arrow grid', initialValue: 7, min: 4, max: 10)),
          ScatterGadget(look: look, size: 150, density: c.knobs.int.slider(label: 'Density', initialValue: 60, min: 8, max: 160), seed: c.knobs.int.slider(label: 'Seed', initialValue: 3, min: 1, max: 30)),
          RegionGadget(look: look, size: 150, feather: c.knobs.double.slider(label: 'Feather', initialValue: .4, min: 0, max: 1), invert: c.knobs.boolean(label: 'Invert', initialValue: false)),
        ]);
      }, width: 792, height: 552),
      uc('Gadgets 5 · Path and time (Along Path, Stagger, Follow, Curve)', (c) {
        final look = lookKnob(c);
        return _page([
          AlongPathGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Path items', initialValue: 7, min: 2, max: 16), bias: c.knobs.double.slider(label: 'Path bias', initialValue: 1, min: .4, max: 2.5)),
          StaggerGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Stagger items', initialValue: 8, min: 3, max: 14), ease: c.knobs.double.slider(label: 'Stagger ease', initialValue: 1, min: .5, max: 2.5), preview: c.knobs.boolean(label: 'Stagger preview', initialValue: true)),
          FollowGadget(look: look, size: 150, lag: c.knobs.double.slider(label: 'Follow lag (s)', initialValue: .35, min: .05, max: 1.2), trail: c.knobs.int.slider(label: 'Follow trail', initialValue: 36, min: 8, max: 80)),
          CurveGadget(look: look, size: 150, preset: c.knobs.int.slider(label: 'Curve preset (0 linear .. 4 back)', initialValue: 1, min: 0, max: 4), preview: c.knobs.boolean(label: 'Curve preview', initialValue: true)),
        ]);
      }, width: 792, height: 552),
      uc('Gadgets 6 · Distribution (Scale, Rotation, Color, Repeat)', (c) {
        final look = lookKnob(c);
        return _page([
          ScaleGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Scale items', initialValue: 7, min: 3, max: 10)),
          RotationGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Rotation items', initialValue: 9, min: 3, max: 12), jitter: c.knobs.double.slider(label: 'Rotation random', initialValue: .15, min: 0, max: 1)),
          ColorGadget(look: look, size: 150, ease: c.knobs.double.slider(label: 'Gradient ease', initialValue: 1, min: .4, max: 2.5)),
          RepeatGadget(look: look, size: 150, cols: c.knobs.int.slider(label: 'Columns', initialValue: 5, min: 2, max: 8), rows: c.knobs.int.slider(label: 'Rows', initialValue: 3, min: 2, max: 6), jitter: c.knobs.double.slider(label: 'Grid jitter', initialValue: .25, min: 0, max: 1)),
        ]);
      }, width: 792, height: 552),
      uc('Gadgets 7 · Signal (Noise, Graph / Link, Audio)', (c) {
        final look = lookKnob(c);
        return _page([
          NoiseGadget(look: look, size: 150, roughness: c.knobs.double.slider(label: 'Roughness', initialValue: .45, min: 0, max: 1), drift: c.knobs.double.slider(label: 'Drift (0 = still)', initialValue: .5, min: 0, max: 2)),
          GraphGadget(look: look, size: 150, strength: c.knobs.double.slider(label: 'Link strength', initialValue: .7, min: .1, max: 1), type: c.knobs.int.slider(label: 'Link type (0 line, 1 curve, 2 step)', initialValue: 1, min: 0, max: 2), pulse: c.knobs.boolean(label: 'Pulse', initialValue: true)),
          AudioGadget(look: look, size: 150, smooth: c.knobs.double.slider(label: 'Audio smooth', initialValue: .5, min: 0, max: 1)),
        ]);
      }, width: 792, height: 340),
      uc('Gadget · one, large', (c) {
        final look = lookKnob(c);
        final no = c.knobs.int.slider(label: 'Gadget (1-15)', initialValue: 1, min: 1, max: 15);
        return _scroll( Column(mainAxisSize: MainAxisSize.min, children: [
          Text(gadgetNames[no - 1], style: T.label(N.g56)),
          const SizedBox(height: 10),
          gadgetByNo(no, look: look, size: c.knobs.double.slider(label: 'Size', initialValue: 340, min: 200, max: 440), framed: c.knobs.boolean(label: 'Framed', initialValue: true)),
        ]));
      }, width: 520, height: 552),
      uc('Gadget wall · all 15', (c) {
        final look = lookKnob(c);
        final size = c.knobs.double.slider(label: 'Size', initialValue: 130, min: 110, max: 170);
        return _page([for (var i = 1; i <= 15; i++) gadgetByNo(i, look: look, size: size)]);
      }, width: 792, height: 552),
      uc('Chip · compact, for lists', (c) => _ChipDemo(lookKnob(c), c.knobs.boolean(label: 'Compact', initialValue: false), c.knobs.boolean(label: 'Show value', initialValue: true)), width: 480, height: 160),
      uc('Badge · on a layer row', (c) => Align(child: _LayerRows(lookKnob(c), c.knobs.int.slider(label: 'Badge style (0 dot, 1 mark, 2 bar)', initialValue: 1, min: 0, max: 2), c.knobs.double.slider(label: 'Row scale', initialValue: 1.4, min: 1, max: 2))), width: 340, height: 200),
      uc('Add relation menu', (c) => _AddMenuDemo(lookKnob(c), c.knobs.boolean(label: 'Start open', initialValue: true)), width: 260, height: 420),
      uc('Card · collapsed / expanded', (c) {
        final look = lookKnob(c), f = Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 0, min: 0, max: 5)];
        return _scroll( Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          RelCard(f, look: look, mode: CardMode.expanded),
          const SizedBox(height: 10),
          RelCard(Fam.all[(Fam.all.indexOf(f) + 1) % 6], look: look, mode: CardMode.collapsed),
        ]));
      }, width: 400, height: 520),
      uc('Card · disabled / in error', (c) {
        final look = lookKnob(c), f = Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 5, min: 0, max: 5)];
        final msg = c.knobs.string(label: 'Error message', initialValue: 'Target layer "Gem 04" was deleted');
        return _scroll( Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          RelCard(f, look: look, mode: CardMode.disabled),
          const SizedBox(height: 10),
          RelCard(f, look: look, mode: CardMode.error, message: msg),
        ]));
      }, width: 400, height: 552),
      uc('Connection line · drag the layers', (c) => _ConnectionDemo(
            look: lookKnob(c),
            fam: Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 5, min: 0, max: 5)],
            style: c.knobs.int.slider(label: 'Line (0 straight, 1 curve, 2 elbow)', initialValue: 1, min: 0, max: 2),
            dashed: c.knobs.boolean(label: 'Dashed', initialValue: false),
            label: c.knobs.boolean(label: 'Label', initialValue: true),
            flow: c.knobs.boolean(label: 'Flow', initialValue: true),
          ), width: 520, height: 300),
      uc('Empty state · no relations yet', (c) => Align(child: _EmptyDemo(lookKnob(c))), width: 340, height: 330),
      uc('Flow · add, tune, see', (c) => _Flow(lookKnob(c), c.knobs.double.slider(label: 'Result falloff', initialValue: 1.6, min: .4, max: 4)), width: 760, height: 400),
      uc('Card list · inspector column', (c) {
        final look = lookKnob(c);
        return _scroll( Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          RelCard(Fam.scatter, look: look, mode: CardMode.expanded),
          const SizedBox(height: 6),
          RelCard(Fam.stagger, look: look, mode: CardMode.collapsed),
          const SizedBox(height: 6),
          RelCard(Fam.along, look: look, mode: CardMode.collapsed),
          const SizedBox(height: 6),
          RelCard(Fam.face, look: look, mode: CardMode.disabled),
        ]));
      }, width: 372, height: 560),
    ]);
