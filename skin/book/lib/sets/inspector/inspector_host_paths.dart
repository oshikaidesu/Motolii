import 'dart:math' as math;
import 'inspector_studies.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';
part 'inspector_host_paths_2.dart';

// ---- host: path operations ------------------------------------------------------------------------------------------
/// A path operation as the host ships it (motolii-render extensions/pathop.rs): its fields in display units (%, px, °),
/// what a folded card says about them, and which fields only exist in some mode.
class _PathOp {
  const _PathOp(this.id, this.name, this.params, this.sum, {this.when = const {}});
  final String id, name;
  final List<Param> params;
  final List<String> Function(double Function(String) g) sum;
  final Map<String, bool Function(double Function(String) g)> when;
}

Param _pn(String id, String label, ParamLook look, double def, {double? min, double? max, String unit = '', int dec = 0, bool adv = false}) => Param(
  id: id,
  label: label,
  look: look,
  type: look == ParamLook.stepper ? WType.i32 : WType.f32,
  min: min,
  max: max,
  def: def,
  unit: unit,
  dec: dec,
  advanced: adv,
);
Param _pc(String id, String label, List<String> o, {bool adv = false}) =>
    Param(id: id, label: label, look: ParamLook.choice, type: WType.u32, min: 0, max: o.length - 1.0, options: o, advanced: adv);
Param _pp(String id, String label) => Param(id: id, label: label, look: ParamLook.point, type: WType.vec2, unit: 'px');

const _multiples = ['Simultaneously', 'Individually'], points = ['Corner', 'Smooth'], joins = ['Miter', 'Round', 'Bevel'];
String sgPaths(double v, [int dec = 0]) => '${v > 0 ? '+' : ''}${brief(v, dec)}';
String px(double v) => '${brief(v, 1)} px';

final _kPathOps = <_PathOp>[
  _PathOp('trim', 'Trim Paths', [
    _pn('start', 'Start', ParamLook.percent, 0, min: 0, max: 100, unit: '%'),
    _pn('end', 'End', ParamLook.percent, 100, min: 0, max: 100, unit: '%'),
    _pn('offset', 'Offset', ParamLook.angle, 0, unit: '°'),
    _pc('multiple', 'Trim', _multiples),
  ], (g) => ['${brief(g('start'), 0)}–${brief(g('end'), 0)}%', if (g('offset') != 0) '${sgPaths(g('offset'))}°', if (g('multiple') == 1) 'Individually']),
  _PathOp('round', 'Rounded Corners', [_pn('radius', 'Radius', ParamLook.scrub, 10, min: 0, unit: 'px')], (g) => [px(g('radius'))]),
  _PathOp('pucker', 'Pucker & Bloat', [_pn('amount', 'Amount', ParamLook.bipolar, 0, min: -100, max: 100, unit: '%')], (g) {
    final a = g('amount');
    return [a == 0 ? '0%' : '${a < 0 ? 'Pucker' : 'Bloat'} ${brief(a.abs(), 0)}%'];
  }),
  _PathOp('zigzag', 'Zig Zag', [
    _pn('amplitude', 'Size', ParamLook.scrub, 10, unit: 'px'),
    _pn('frequency', 'Ridges', ParamLook.stepper, 10, min: 0),
    _pc('point_type', 'Points', points),
  ], (g) => [px(g('amplitude')), '${brief(g('frequency'), 0)} ridges', if (g('point_type') == 1) 'Smooth']),
  _PathOp(
    'offset',
    'Offset Paths',
    [
      _pn('amount', 'Amount', ParamLook.scrub, 10, unit: 'px'),
      _pc('join', 'Join', joins),
      _pn('miter_limit', 'Miter Limit', ParamLook.scrub, 4, min: 1, dec: 1),
    ],
    (g) => ['${sgPaths(g('amount'), 1)} px', joins[g('join').round().clamp(0, 2)]],
    when: {'miter_limit': (g) => g('join') == 0},
  ),
  _PathOp('twist', 'Twist', [_pn('angle', 'Angle', ParamLook.angle, 0, unit: '°'), _pp('center', 'Center')], (g) => ['${sgPaths(g('angle'))}°']),
  _PathOp('wiggle', 'Wiggle Paths', [
    _pn('size', 'Size', ParamLook.scrub, 10, min: 0, unit: 'px'),
    _pn('detail', 'Detail', ParamLook.bar, 5, min: 1, max: 100),
    _pn('phase', 'Phase', ParamLook.angle, 0, unit: '°'),
    _pn('seed', 'Seed', ParamLook.seed, 1, min: 0, max: 9999, adv: true),
    _pc('point_type', 'Points', points, adv: true),
  ], (g) => [px(g('size')), 'Detail ${brief(g('detail'), 0)}', 'Seed ${brief(g('seed'), 0)}']),
  _PathOp('smooth', 'Smooth', [
    _pn('strength', 'Strength', ParamLook.percent, 50, min: 0, max: 100, unit: '%'),
    _pn('iterations', 'Iterations', ParamLook.stepper, 1, min: 1, max: 50),
  ], (g) => ['${brief(g('strength'), 0)}%', '×${brief(g('iterations'), 0)}']),
  _PathOp('subdivide', 'Subdivide', [_pn('divisions', 'Divisions', ParamLook.stepper, 1, min: 1, max: 64)], (g) => ['×${brief(g('divisions'), 0)}']),
  _PathOp('reverse', 'Reverse Path', const [], (g) => const ['Flips direction']),
  _PathOp('extend', 'Extend Paths', [
    _pn('start', 'Start', ParamLook.scrub, 0, unit: 'px'),
    _pn('end', 'End', ParamLook.scrub, 0, unit: 'px'),
  ], (g) => ['${sgPaths(g('start'), 1)} / ${sgPaths(g('end'), 1)} px']),
  _PathOp('chop', 'Chop Path', [
    _pn('length', 'Length', ParamLook.scrub, 50, min: 0, unit: 'px'),
    _pn('gap', 'Gap', ParamLook.scrub, 10, min: 0, unit: 'px'),
  ], (g) => ['${brief(g('length'), 1)} on', '${brief(g('gap'), 1)} off']),
  _PathOp('resample', 'Resample', [
    _pn('spacing', 'Spacing', ParamLook.scrub, 20, min: 0, unit: 'px'),
    _pc('point_type', 'Points', points),
  ], (g) => ['Every ${px(g('spacing'))}', if (g('point_type') == 1) 'Smooth']),
  _PathOp('bend', 'Bend', [_pn('angle', 'Angle', ParamLook.angle, 0, min: -360, max: 360, unit: '°'), _pp('center', 'Center')], (g) => ['${sgPaths(g('angle'))}°']),
  // contract: Frequency's 0–1000 is clamped below only; a bounded block scrubs range/200 per px, too coarse for a value around 2
  _PathOp('osc', 'Oscillator', [
    _pn('amplitude', 'Amplitude', ParamLook.scrub, 20, unit: 'px'),
    _pn('frequency', 'Frequency', ParamLook.scrub, 2, min: 0, dec: 1),
    _pn('offset', 'Offset', ParamLook.scrub, 0, dec: 1),
    _pn('detail', 'Detail', ParamLook.bar, 16, min: 2, max: 256, adv: true),
  ], (g) => [px(g('amplitude')), '${brief(g('frequency'), 1)} waves']),
];

/// contract: the stack is `pop_order`, comma-separated instances (`trim1`); an instance is its kind plus a never-reused number,
/// so a removed card's values can never leak into a new one. `pop_next` is that number.
List<String> _popOrder(Doc d) => [
  for (final s in (d.s2['pop_order'] ?? '').split(','))
    if (s.isNotEmpty) s,
];
void _popSet(Doc d, List<String> o) => d.str('pop_order', o.join(','));
_PathOp? _popOp(String inst) {
  final t = inst.replaceFirst(RegExp(r'\d+$'), '');
  return _kPathOps.where((o) => o.id == t).firstOrNull;
}

Map<String, double> _popDefaults(String inst, _PathOp op) => {for (final p in op.params) ...paramDefaults('pop_$inst', p)};

String _popAdd(Doc d, _PathOp op, {int? at, Map<String, double> copy = const {}}) {
  final n = int.tryParse(d.s2['pop_next'] ?? '') ?? 1, inst = '${op.id}$n';
  d.s2['pop_next'] = '${n + 1}';
  d.seed(_popDefaults(inst, op));
  copy.forEach((id, v) => d.v[id.replaceFirst(RegExp(r'^pop_[a-z]+\d+'), 'pop_$inst')] = v);
  final o = _popOrder(d)..insert(at ?? _popOrder(d).length, inst);
  _popSet(d, o);
  return inst;
}

/// The name a card wears: a second Trim Paths in the stack reads "Trim Paths 2".
String _popTitle(List<String> order, String inst, _PathOp op) {
  final same = order.takeWhile((s) => s != inst).where((s) => _popOp(s)?.id == op.id).length;
  return same == 0 ? op.name : '${op.name} ${same + 1}';
}

class _OpDrag {
  const _OpDrag(this.inst, this.name);
  final String inst, name;
}

/// The stack of path operations on a shape layer, top to bottom in the order they apply.
/// contract: order matters, so every card is held by its head and dragged to a new place; More keeps Move earlier / later for the keyboard.
class PathOpsStack extends StatelessWidget {
  const PathOpsStack({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed({}, strs: {'pop_order': 'trim1,round2,wiggle3', 'pop_next': '4'});
    final order = _popOrder(doc);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (order.isEmpty)
          Container(
            color: Pop.card,
            margin: EdgeInsets.only(bottom: Pop.gap),
            padding: EdgeInsets.symmetric(horizontal: Pop.inset, vertical: Pop.insetY),
            child: Text('No path operations. The path is drawn as built.', style: T.label(Grey.g56)),
          ),
        for (final (i, s) in order.indexed)
          if (_popOp(s) != null) _OpCard(s, key: ValueKey(s), first: i == 0),
        const _AddOp(),
      ],
    );
  }
}

class _OpCard extends StatelessWidget {
  const _OpCard(this.inst, {super.key, this.first = false});
  final String inst;
  final bool first;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), op = _popOp(inst)!, k = 'pop_$inst';
    doc.seed(_popDefaults(inst, op));
    final on = doc.b['$k.on'] ?? true, open = doc.b['$k.open'] ?? true, adv = doc.b['$k.adv'] ?? false;
    double g(String id) => doc.get('${k}_$id') ?? 0;
    final shown = [
      for (final p in op.params)
        if (op.when[p.id]?.call(g) ?? true) p,
    ];
    final heroes = [
          for (final p in shown)
            if (!p.advanced) p,
        ],
        rest = [
          for (final p in shown)
            if (p.advanced) p,
        ];
    final title = _popTitle(_popOrder(doc), inst, op), locked = x.cfg.locked, bare = op.params.isEmpty;
    final trailing = Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (open || bare) _OpMore(inst, op),
        OnOff(on: on, onChanged: (v) => doc.flag('$k.on', v)),
      ],
    );
    Widget? wrap(Widget head) => locked
        ? null
        : LongPressDraggable<_OpDrag>(
            data: _OpDrag(inst, title),
            delay: const Duration(milliseconds: 180),
            axis: Axis.vertical,
            feedback: _OpGhost(title, op.id),
            childWhenDragging: Opacity(opacity: .35, child: head),
            child: head,
          );
    final Widget card = bare
        ? _OpLine(title: title, op: op, dim: !on, trailing: trailing, wrap: wrap)
        : Sect(
            title: title,
            sub: true,
            first: first,
            open: open,
            tone: Pop.tonePath,
            mark: CardMark.path,
            headWrap: locked ? null : (h) => wrap(h)!,
            dim: !on,
            keyIds: [for (final p in op.params) '${k}_${p.id}'],
            brief: op.sum(g),
            onToggle: () => doc.flag('$k.open', !open),
            trailing: trailing,
            children: [
              if (heroes.isNotEmpty) ParamGrid(k, heroes, enabled: on),
              if (rest.isNotEmpty) ...[
                Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Hov(
                    onTap: () => doc.flag('$k.adv', !adv),
                    builder: (_, h) => Container(
                      height: 24,
                      decoration: BoxDecoration(
                        border: Border(top: BorderSide(color: ed.rule)),
                      ),
                      child: Row(
                        children: [
                          Text('Advanced', style: ed.labStyle(h || adv ? Grey.g95 : Grey.g63)),
                          const Spacer(),
                          Text(adv ? 'Fold' : '${rest.length} more', style: T.label(Grey.g56)),
                        ],
                      ),
                    ),
                  ),
                ),
                if (adv) ParamGrid(k, rest, enabled: on),
              ],
            ],
          );
    return locked ? card : _OpDrop(inst: inst, child: card);
  }
}

/// A card with nothing to set (Reverse Path): one line, laid on the same columns as a folded card, with no fold mark since there is nothing to unfold.
class _OpLine extends StatelessWidget {
  const _OpLine({required this.title, required this.op, required this.dim, required this.trailing, required this.wrap});
  final String title;
  final _PathOp op;
  final bool dim;
  final Widget trailing;
  final Widget? Function(Widget) wrap;
  @override
  Widget build(BuildContext context) {
    final head = SizedBox(
      height: Pop.titleH,
      child: Row(
        children: [
          SizedBox(
            width: briefX,
            child: Row(
              children: [
                SizedBox(width: 8 + Pop.inset),
                ToneBadge(CardMark.path, Pop.tonePath, dim: dim),
                const SizedBox(width: 6),
                Flexible(
                  child: Text(
                    title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: T
                        .title(dim ? Grey.g56 : Grey.g95)
                        .copyWith(fontSize: 12, fontWeight: FontWeight.w700, decoration: dim ? TextDecoration.lineThrough : null),
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: Row(
              children: [for (final t in op.sum((_) => 0)) Flexible(child: BriefChip(t, dim: dim))],
            ),
          ),
          trailing,
        ],
      ),
    );
    return Padding(
      padding: EdgeInsets.only(bottom: Pop.gap),
      child: ColoredBox(
        color: Pop.card,
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: Pop.inset, vertical: Pop.insetY),
          child: wrap(head) ?? head,
        ),
      ),
    );
  }
}

/// Where a dragged card lands: a line in the family's colour shows the gap it will fill.
class _OpDrop extends StatelessWidget {
  const _OpDrop({required this.inst, required this.child});
  final String inst;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return DragTarget<_OpDrag>(
      onWillAcceptWithDetails: (d) => d.data.inst != inst,
      onAcceptWithDetails: (d) {
        final o = _popOrder(doc)..remove(d.data.inst);
        o.insert(_popOrder(doc).indexOf(inst), d.data.inst);
        _popSet(doc, o);
      },
      builder: (context, cand, _) {
        final o = _popOrder(doc), from = cand.isEmpty ? -1 : o.indexOf(cand.first!.inst);
        final below = from >= 0 && from < o.indexOf(inst);
        return Stack(
          clipBehavior: Clip.none,
          children: [
            child,
            if (from >= 0)
              Positioned(
                left: 0,
                right: 0,
                top: below ? null : -Pop.gap,
                bottom: below ? 0 : null,
                height: 2,
                child: ColoredBox(color: Pop.tonePath),
              ),
          ],
        );
      },
    );
  }
}

class _OpGhost extends StatelessWidget {
  const _OpGhost(this.name, this.type);
  final String name, type;
  @override
  Widget build(BuildContext context) => Container(
    width: 220,
    height: Pop.titleH + Pop.insetY * 2,
    padding: EdgeInsets.symmetric(horizontal: Pop.inset),
    decoration: BoxDecoration(
      color: Grey.g20,
      boxShadow: [BoxShadow(color: Color(0x99000000), blurRadius: 14, offset: Offset(0, 6))],
    ),
    child: Row(
      children: [
        OpGlyph(type),
        const SizedBox(width: 6),
        Text(name, style: T.title(Grey.g95).copyWith(fontSize: 12, fontWeight: FontWeight.w700)),
      ],
    ),
  );
}

/// A menu that floats over the panel from its anchor and never pushes the cards below; it opens upward when there is no room under it.
class Float extends StatefulWidget {
  const Float({super.key, required this.anchor, required this.menu, required this.need, this.width, this.right = false});
  final Widget Function(bool open, VoidCallback toggle) anchor;
  final Widget Function(VoidCallback close) menu;
  final double need;
  final double? width;
  final bool right;
  @override
  State<Float> createState() => _FloatState();
}

class _FloatState extends State<Float> {
  final _link = LayerLink(), _portal = OverlayPortalController();
  bool _up = false;

  void _set(bool open) {
    if (open) {
      final box = context.findRenderObject() as RenderBox?;
      final overlay = Overlay.maybeOf(context)?.context.findRenderObject() as RenderBox?;
      if (box != null && overlay != null) {
        final bottom = box.localToGlobal(Offset(0, box.size.height), ancestor: overlay).dy;
        _up = overlay.size.height - bottom < widget.need && bottom - box.size.height > widget.need;
      }
    }
    setState(() => open ? _portal.show() : _portal.hide());
  }

  @override
  Widget build(BuildContext context) {
    final r = widget.right;
    final at = _up ? (r ? Alignment.topRight : Alignment.topLeft) : (r ? Alignment.bottomRight : Alignment.bottomLeft);
    final from = _up ? (r ? Alignment.bottomRight : Alignment.bottomLeft) : (r ? Alignment.topRight : Alignment.topLeft);
    return CompositedTransformTarget(
      link: _link,
      child: OverlayPortal(
        controller: _portal,
        overlayChildBuilder: (_) => Stack(
          children: [
            CompositedTransformFollower(
              link: _link,
              showWhenUnlinked: false,
              targetAnchor: at,
              followerAnchor: from,
              offset: Offset(0, _up ? -4 : 4),
              child: Align(
                alignment: from,
                child: TapRegion(groupId: _link, onTapOutside: (_) => _set(false), child: SizedBox(
                  width: widget.width ?? (context.findRenderObject() as RenderBox?)?.size.width ?? 200,
                  child: Container(
                    padding: const EdgeInsets.all(3),
                    decoration: BoxDecoration(
                      color: Grey.g15,
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: Grey.g26),
                      boxShadow: const [BoxShadow(color: Color(0x99000000), blurRadius: 16, offset: Offset(0, 6))],
                    ),
                    child: widget.menu(() => _set(false)),
                  ),
                )),
              ),
            ),
          ],
        ),
        child: TapRegion(groupId: _link, child: widget.anchor(_portal.isShowing, () => _set(!_portal.isShowing))),
      ),
    );
  }
}
