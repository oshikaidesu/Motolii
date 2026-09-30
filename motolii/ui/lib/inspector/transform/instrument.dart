// The Transform Instrument: one gizmo and the exact values that belong to it, on one surface.
// Position, Scale, Rotation and Anchor are four ways of touching the same body, not four property cards.
// Space, Parent and Depth sit around it. Precision is the Phase 1 Value; the instrument adds gesture, not a second system.
import 'package:flutter/widgets.dart';
import '../../browser/parts.dart';
import '../../desks/parts.dart' show kYellow, kBlue, kPink, kViolet, kAccentDim;
import '../../theme/metrics.dart';
import '../rows.dart';
import '../value_controls.dart';
import 'gizmo.dart';
import 'model.dart';
import '../../theme/neutral.dart';
import '../../theme/identity.dart' show H;

class TransformInstrument extends StatefulWidget {
  const TransformInstrument(this.store, {super.key, this.initialMode = TMode.move, this.showHeader = true});
  final TransformStore store;

  /// The layer's name and Animate. A host that already shows them (the Inspector's own header) hides this one.
  final bool showHeader;
  final TMode initialMode;
  @override
  State<TransformInstrument> createState() => _TransformInstrumentState();
}

class _TransformInstrumentState extends State<TransformInstrument> {
  late TMode mode = widget.initialMode;
  int rotAxis = 0;
  final _nodes = <String, FocusNode>{};
  TransformStore get s => widget.store;

  FocusNode node(String k) => _nodes.putIfAbsent(k, () => FocusNode(debugLabel: 'tf:$k'));

  @override
  void initState() {
    super.initState();
    s.focusRequest.addListener(_reveal);
  }

  @override
  void dispose() {
    s.focusRequest.removeListener(_reveal);
    for (final n in _nodes.values) { n.dispose(); }
    super.dispose();
  }

  // Another part of the editor asks to see a property: show the mode that owns it and focus its number.
  void _reveal() {
    final id = s.focusRequest.value;
    if (id == null) return;
    setState(() => mode = id.startsWith('position') ? TMode.move : (id.startsWith('scale') ? TMode.scale : (id.startsWith('rotation') ? TMode.rotate : (id == 'anchor' ? TMode.anchor : mode))));
    final key = switch (id) { 'position' => 'position:0', 'scale' => 'scale:0', _ => id };
    WidgetsBinding.instance.addPostFrameCallback((_) => _nodes[key]?.requestFocus());
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: s,
        builder: (context, _) => LayoutBuilder(builder: (context, box) {
          final narrow = box.maxWidth < 172;
          final pad = narrow ? Surface.px(6.0) : Surface.px(9.0);
          final inner = box.maxWidth - pad * 2;
          final l = s.active;
          final gizmoW = narrow ? inner : inner - 96;
          final gizmo = Stack(children: [
            TransformGizmo(s, mode: mode, onMode: (m) => setState(() => mode = m), rotAxis: rotAxis, size: Size(gizmoW, narrow ? Surface.px(99) : Surface.px(129))),
            if (mode == TMode.rotate && l.projection != '2D')
              Positioned(right: Surface.px(4.5), bottom: Surface.px(4.5), child: Row(children: [
                for (final (i, t) in ['Z', 'X', 'Y'].indexed)
                  GestureDetector(key: ValueKey('rot-axis-$i'), onTap: () => setState(() => rotAxis = i), child: Container(margin: EdgeInsets.only(left: Surface.px(2)), width: Surface.px(15), height: Surface.px(13.5), alignment: Alignment.center, decoration: BoxDecoration(color: rotAxis == i ? kPink : N.g15, borderRadius: BorderRadius.circular(Surface.controlRadius)), child: Text(t, style: sans(Dn.microSize, c: rotAxis == i ? N.g10 : Surface.muted, w: FontWeight.w700)))),
              ])),
          ]);
          final strip = _modeStrip(narrow);
          final body = Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            if (widget.showHeader) ...[_header(), SizedBox(height: Surface.sectionGap)],
            if (narrow) ...[strip, SizedBox(height: Surface.px(4.5)), gizmo, SizedBox(height: Surface.px(4.5)), _spaceRow(), SizedBox(height: Surface.px(4.5)), _parent(narrow)]
            else Row(crossAxisAlignment: CrossAxisAlignment.start, children: [strip, SizedBox(width: Surface.px(4.5)), gizmo, SizedBox(width: Surface.px(4.5)), SizedBox(width: Surface.px(58.5), child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_spaceCol(), SizedBox(height: Surface.sectionGap), _parent(narrow)]))]),
            SizedBox(height: Surface.px(7.5)),
            _rows(narrow),
            SizedBox(height: Surface.px(7.5)),
            _world(),
          ]);
          // A seat short enough (a resized Dock pane, a smaller window) that even the wide layout doesn't fit its
          // own height scrolls instead of overflowing — the same rule the narrow layout already followed.
          return SingleChildScrollView(
            key: const ValueKey('tf-scroll'),
            padding: EdgeInsets.fromLTRB(pad, Surface.px(7.5), pad, narrow ? Surface.px(16) : Surface.px(10)),
            child: body,
          );
        }),
      );

  // ---- header: who is shown, and Animate ----------------------------------------------------------------------
  Widget _header() {
    final l = s.active;
    return SizedBox(
      height: Surface.px(16.5),
      child: Row(children: [
        Container(width: Surface.px(2), height: Surface.px(10.5), margin: EdgeInsets.only(right: Surface.px(5)), decoration: BoxDecoration(color: modeColor[mode], borderRadius: BorderRadius.circular(Surface.px(1.5)))),
        Expanded(child: Text(s.multiple ? '${s.selection.length} layers' : l.name, key: const ValueKey('tf-title'), softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600))),
        if (l.locked) Padding(padding: EdgeInsets.only(right: Surface.sectionGap), child: Text('Locked', style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w600))),
        GestureDetector(
          key: const ValueKey('tf-animate'),
          behavior: HitTestBehavior.opaque,
          onTap: s.toggleAnimate,
          child: Row(children: [
            SizedBox(width: Surface.px(9), height: Surface.px(9), child: CustomPaint(painter: _DiamondP(s.animating, kYellow))),
            SizedBox(width: Surface.px(4)),
            Text('Animate', style: sans(Dn.labelSize, c: s.animating ? kYellow : Surface.muted, w: FontWeight.w600)),
          ]),
        ),
      ]),
    );
  }

  // ---- modes: small glyphs at the gizmo's edge; the number rows switch them too ------------------------------
  Widget _modeStrip(bool narrow) {
    Widget b(TMode m) => GestureDetector(
          key: ValueKey('mode-${m.name}'),
          behavior: HitTestBehavior.opaque,
          onTap: () => setState(() => mode = m),
          child: Container(
            width: narrow ? null : Surface.px(28),
            height: narrow ? Surface.px(26) : Surface.topBar,
            margin: EdgeInsets.only(bottom: narrow ? 0 : Surface.px(4), right: narrow ? Surface.px(4) : 0),
            decoration: BoxDecoration(color: mode == m ? modeColor[m] : N.g15, borderRadius: BorderRadius.circular(Surface.px(4))),
            child: Center(child: SizedBox(width: Surface.px(11), height: Surface.px(11), child: CustomPaint(painter: RoleGlyph(m, mode == m ? N.g10 : modeColor[m]!)))),
          ),
        );
    return narrow ? Row(children: [for (final m in TMode.values) Expanded(child: b(m))]) : Column(children: [for (final m in TMode.values) b(m)]);
  }

  // ---- around the instrument: Space, Parent ---------------------------------------------------------------------
  Widget _chip(String key, String t, bool on, VoidCallback f, {double? h}) => GestureDetector(
        key: ValueKey(key),
        behavior: HitTestBehavior.opaque,
        onTap: s.canEdit ? f : null,
        child: Container(height: h ?? Surface.px(24), margin: EdgeInsets.only(bottom: Surface.labelGap), alignment: Alignment.center, decoration: BoxDecoration(color: on ? (s.canEdit ? kYellow : kAccentDim) : Surface.raised, borderRadius: BorderRadius.circular(Surface.px(4))), child: Text(t, style: sans(Dn.nameSize, c: on ? N.g10 : (s.canEdit ? N.g69 : N.g33), w: on ? FontWeight.w700 : FontWeight.w500))),
      );

  Widget _spaceCol() => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Padding(padding: EdgeInsets.only(bottom: Surface.inlineGap), child: Text('SPACE', style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.2))),
        for (final p in ['2D', '2.5D', '3D']) _chip('space-$p', p, s.active.projection == p, () => s.setProjection(p)),
      ]);

  Widget _spaceRow() => Row(children: [for (final p in ['2D', '2.5D', '3D']) Expanded(child: Padding(padding: EdgeInsets.only(right: Surface.px(2)), child: _chip('space-$p', p, s.active.projection == p, () => s.setProjection(p))))]);

  Widget _parent(bool narrow) {
    final others = [for (final l in s.layers) if (l.id != s.activeId) l];
    final cur = s.active.parent;
    final names = ['None', for (final l in others) l.name];
    var idx = cur == null ? 0 : (others.indexWhere((l) => l.id == cur) + 1).clamp(0, names.length - 1);
    void pick(int i) {
      final n = (i + names.length) % names.length;
      s.setParent(n == 0 ? null : others[n - 1].id);
    }
    Widget arrow(String k, String t, int d) => GestureDetector(key: ValueKey('parent-$k'), behavior: HitTestBehavior.opaque, onTap: s.canEdit ? () => pick(idx + d) : null, child: SizedBox(width: Surface.px(15), height: Surface.px(19.5), child: Center(child: Text(t, style: sans(Dn.nameSize, c: Surface.muted)))));
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: EdgeInsets.only(bottom: Surface.inlineGap), child: Text('PARENT', style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.2))),
      Container(height: Surface.px(19.5), decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(Surface.px(4))), child: Row(children: [arrow('prev', '‹', -1), Expanded(child: Center(child: Text(names[idx], key: const ValueKey('parent-name'), softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: s.canEdit ? N.g82 : N.g33, w: FontWeight.w500)))), arrow('next', '›', 1)])),
    ]);
  }

  // ---- what the layer meets the scene with: how it blends and (an Image only) its flags -------------------------
  /// A world switch: a short name on a chip, lit when on; what it does is its semantics label.
  Widget _flag(String key, String label, String meaning, bool on, VoidCallback? tap) => Semantics(
        label: meaning,
        toggled: on,
        child: GestureDetector(
          key: ValueKey('world-$key'),
          behavior: HitTestBehavior.opaque,
          onTap: tap,
          child: Container(
            height: Surface.control,
            padding: EdgeInsets.symmetric(horizontal: Surface.px(7)),
            decoration: BoxDecoration(color: on ? kYellow.withValues(alpha: .16) : Surface.raised, border: Border.all(color: on ? kYellow.withValues(alpha: .6) : Surface.raised), borderRadius: BorderRadius.circular(Surface.px(4))),
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              Container(width: Surface.px(4.5), height: Surface.px(4.5), decoration: BoxDecoration(color: on ? kYellow : N.g33, shape: BoxShape.circle)),
              SizedBox(width: Surface.px(4.5)),
              Text(label, style: sans(Dn.nameSize, c: tap == null ? N.g33 : (on ? kYellow : N.g76), w: FontWeight.w500)),
            ]),
          ),
        ),
      );

  Widget _world() {
    final l = s.active;
    if (l.kind == 'Camera') return const SizedBox.shrink();
    final edit = s.canEdit;
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: EdgeInsets.only(bottom: Surface.px(4.5)), child: Text('WORLD', style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.2))),
      Row(children: [
        SizedBox(width: Surface.px(33), child: Text('Blend', style: sans(Dn.labelSize, c: N.g56))),
        Expanded(
          child: GestureDetector(
            key: const ValueKey('world-blend'),
            behavior: HitTestBehavior.opaque,
            onTap: edit ? s.openBlend : null,
            child: Container(height: Surface.control, alignment: Alignment.centerLeft, padding: EdgeInsets.symmetric(horizontal: Surface.sectionGap), decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(Surface.px(4))), child: Row(children: [
              Expanded(child: Text(l.blendMode, key: const ValueKey('world-blend-name'), style: sans(Dn.nameSize, c: edit ? N.g82 : N.g33, w: FontWeight.w500))),
              Text('›', style: sans(Dn.nameSize, c: N.g44)),
            ])),
          ),
        ),
      ]),
      SizedBox(height: Surface.px(4.5)),
      Wrap(spacing: Surface.px(4), runSpacing: Surface.px(4), children: [
        if (l.ghostable) _flag('ghost', 'Ghost', 'Ghost: the same layer seen later by a delay', l.ghost != null, edit && s.worldCan('ghost') ? () => s.setWorldFlag('ghost', l.ghost == null) : null),
        _flag('clip', 'Clip to below', 'Clip to the layer below', l.clipToBelow, edit && s.worldCan('clip') ? () => s.setWorldFlag('clipToBelow', null) : null),
        if (l.kind == 'Image') _flag('environment', 'Environment', 'Environment: this image lights and surrounds the scene', l.environment, edit ? () => s.setWorldFlag('environment', !l.environment) : null),
      ]),
    ]);
  }

  // ---- the exact values: Phase 1 Values, grouped by what they are ------------------------------------------------
  Widget _rows(bool narrow) {
    final l = s.active;
    final out2D = l.projection != '2D';
    Slot slot(String id, [int? axis, String? link]) => Slot(s, id, axis, link);
    Widget val(Slot sl, TMode? m, {String? tag, String? nodeKey, bool units = true}) => Expanded(child: ValueToy(sl, tag: tag, tone: m == null ? N.g56 : modeColor[m]!, showUnit: units && !narrow, decimals: narrow ? 0 : null, focusNode: node(nodeKey ?? '${sl.id}${sl.axis == null ? '' : ':${sl.axis}'}')));
    Widget gap() => SizedBox(width: Surface.px(2));

    // Position: X, Y, and Z outside 2D
    final px = slot('position', 0), py = slot('position', 1);
    // three numbers across need room; a narrow panel stacks the third under the pair rather than shrinking the numbers
    Widget three(Widget a, Widget b, Widget c3) => narrow
        ? Column(children: [Row(children: [a, gap(), b]), SizedBox(height: Surface.px(2)), Row(children: [c3])])
        : Row(children: [a, gap(), b, gap(), c3]);
    Widget positionValues() => out2D
        ? three(val(px, TMode.move, tag: 'X', units: false), val(py, TMode.move, tag: 'Y', units: false), val(slot('position.z'), TMode.move, tag: 'Z', units: false))
        : Row(children: [val(px, TMode.move, tag: 'X'), gap(), val(py, TMode.move, tag: 'Y')]);

    // Scale: one number while it is kept in shape and even, the axes when not
    final sx = slot('scale', 0, 'scale'), sy = slot('scale', 1, 'scale');
    sx.peers = [sy];
    sy.peers = [sx];
    final one = s.linked.contains('scale') && s.scaleEven;
    Widget scaleValues() {
      if (one && !out2D) return Row(children: [val(sx, TMode.scale, tag: 'S')]);
      if (out2D) return three(val(sx, TMode.scale, tag: 'X', units: false), val(sy, TMode.scale, tag: 'Y', units: false), val(slot('scale.z'), TMode.scale, tag: 'Z', units: false));
      return Row(children: [val(sx, TMode.scale, tag: 'X'), gap(), val(sy, TMode.scale, tag: 'Y')]);
    }

    Widget rotationValues() => out2D
        ? three(val(slot('rotation.x'), TMode.rotate, tag: 'X', units: false), val(slot('rotation.y'), TMode.rotate, tag: 'Y', units: false), val(slot('rotation'), TMode.rotate, tag: 'Z', units: false))
        : Row(children: [val(slot('rotation'), TMode.rotate)]);

    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _line(TMode.move, 'position', ['position', 'position.z'], positionValues(), narrow),
      _line(TMode.scale, 'scale', ['scale', 'scale.z'], scaleValues(), narrow, extra: _linkChip()),
      _line(TMode.rotate, 'rotation', ['rotation', 'rotation.x', 'rotation.y'], rotationValues(), narrow),
      _anchorLine(narrow),
      _line(null, 'opacity', ['opacity'], Row(children: [val(slot('opacity'), null)]), narrow, glyph: _OpacityGlyph()),
      if (out2D) _line(null, 'depth', ['depth'], Row(children: [val(slot('depth'), null), SizedBox(width: Surface.px(4)), _routePill()]), narrow, glyph: _DepthGlyph()),
    ]);
  }

  Widget _routePill() => GestureDetector(
        key: const ValueKey('route-depth'),
        behavior: HitTestBehavior.opaque,
        onTap: () => s.route('Depth', 'depth'),
        child: Container(height: Surface.px(16.5), padding: EdgeInsets.symmetric(horizontal: Surface.inlineGap), alignment: Alignment.center, child: Text('Depth →', style: sans(Dn.labelSize, c: kViolet.withValues(alpha: .85), w: FontWeight.w600))),
      );

  Widget _linkChip() => GestureDetector(
        key: const ValueKey('link-scale'),
        behavior: HitTestBehavior.opaque,
        onTap: s.canEdit ? () => s.toggleLink('scale') : null,
        child: Container(width: Surface.px(19.5), height: Surface.control, decoration: BoxDecoration(color: s.linked.contains('scale') ? kBlue.withValues(alpha: .16) : null, border: Border.all(color: s.linked.contains('scale') ? kBlue.withValues(alpha: .7) : N.g20), borderRadius: BorderRadius.circular(Surface.px(4))), child: Center(child: SizedBox(width: Surface.px(10.5), height: Surface.px(10.5), child: CustomPaint(painter: _LinkP(s.linked.contains('scale') ? kBlue : Surface.muted, s.linked.contains('scale')))))),
      );

  bool _modified(List<String> ids) => ids.any((i) => s.rows.any((r) => r['id'] == i) && modified(s.row(i)));
  bool _keyed(String id) => s.active.keyedNow.contains(id);
  bool _animated(String id) => s.active.animated.contains(id);

  Widget _marks(TMode? m, String id, List<String> ids) {
    final tone = m == null ? kViolet : modeColor[m]!;
    return Row(mainAxisSize: MainAxisSize.min, children: [
      GestureDetector(key: ValueKey('key-$id'), behavior: HitTestBehavior.opaque, onTap: () => s.toggleKey(id), child: Padding(padding: EdgeInsets.symmetric(horizontal: Surface.inlineGap), child: SizedBox(width: Surface.px(7.5), height: Surface.px(7.5), child: CustomPaint(painter: _DiamondP(_keyed(id), _animated(id) || _keyed(id) ? tone : N.g26))))),
      if (_modified(ids)) GestureDetector(key: ValueKey('reset-$id'), behavior: HitTestBehavior.opaque, onTap: () => s.resetMany(ids), child: Padding(padding: EdgeInsets.only(left: Surface.px(1.5)), child: Text('↺', style: sans(Dn.nameSize, c: N.g38)))) else SizedBox(width: Surface.px(9)),
    ]);
  }

  Widget _line(TMode? m, String id, List<String> ids, Widget values, bool narrow, {Widget? extra, Widget? glyph}) {
    final active = m != null && mode == m;
    final tone = m == null ? kViolet : modeColor[m]!;
    return Container(
      key: ValueKey('row-$id'),
      margin: EdgeInsets.only(bottom: Surface.cellGap),
      child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
        GestureDetector(
          key: ValueKey('glyph-$id'),
          behavior: HitTestBehavior.opaque,
          onTap: m == null ? null : () => setState(() => mode = m),
          child: Container(width: Surface.px(16.5), height: Surface.control, margin: EdgeInsets.only(right: Surface.px(4)), alignment: Alignment.center, decoration: BoxDecoration(color: active ? tone.withValues(alpha: .18) : null, borderRadius: BorderRadius.circular(Surface.px(4))), child: SizedBox(width: Surface.px(10.5), height: Surface.px(10.5), child: glyph ?? CustomPaint(painter: RoleGlyph(m!, tone)))),
        ),
        Expanded(child: values),
        // every line keeps the same end column (the scale line's link chip, empty elsewhere): the X · Y · Z columns line up
        SizedBox(width: Surface.px(22.5), child: Align(alignment: Alignment.centerRight, child: extra)),
        ..._relationBadge(id, ids),
        if (!narrow) _marks(m, id, ids),
      ]),
    );
  }

  /// A relation on this line: driven by another value (◉ its source), or driving others (◉ how many). Tapping points at it.
  List<Widget> _relationBadge(String id, List<String> ids) {
    final rows = [for (final i in ids) if (s.rows.any((r) => r['id'] == i)) s.row(i)];
    final driven = rows.where((r) => r['link'] is Map).firstOrNull;
    final drives = rows.fold<int>(0, (n, r) => n + ((r['drives'] as int?) ?? 0));
    if (driven == null && drives == 0) return const [];
    final from = driven == null ? id : '${driven['id']}';
    return [
      GestureDetector(
        key: ValueKey('relation-$id'),
        behavior: HitTestBehavior.opaque,
        onTap: () => s.focusRelation(from),
        child: Container(
          margin: EdgeInsets.only(left: Surface.px(4.5)),
          padding: EdgeInsets.symmetric(horizontal: Surface.px(4.5), vertical: Surface.px(1.5)),
          decoration: BoxDecoration(color: H.relation, borderRadius: BorderRadius.circular(Surface.faceRadius)),
          child: Text(driven != null ? '◉ ${(driven['link'] as Map)['name'] ?? 'Relation'}' : '◉ $drives', style: sans(Dn.microSize, c: N.g10, w: FontWeight.w700)),
        ),
      ),
    ];
  }

  static const _anchorNames = [['Top left', 'Top', 'Top right'], ['Left', 'Center', 'Right'], ['Bottom left', 'Bottom', 'Bottom right']];

  Widget _anchorLine(bool narrow) {
    final l = s.active;
    final ax = [0.0, .5, 1.0].indexWhere((v) => (v - l.anchor[0]).abs() < .01), ay = [0.0, .5, 1.0].indexWhere((v) => (v - l.anchor[1]).abs() < .01);
    final name = ax < 0 || ay < 0 ? 'Custom' : _anchorNames[ay][ax];
    return Container(
      key: const ValueKey('row-anchor'),
      margin: EdgeInsets.only(bottom: Surface.cellGap),
      child: Row(children: [
        GestureDetector(key: const ValueKey('glyph-anchor'), behavior: HitTestBehavior.opaque, onTap: () => setState(() => mode = TMode.anchor), child: Container(width: Surface.px(16.5), height: Surface.control, margin: EdgeInsets.only(right: Surface.px(4)), alignment: Alignment.center, decoration: BoxDecoration(color: mode == TMode.anchor ? kViolet : null, borderRadius: BorderRadius.circular(Surface.px(4))), child: SizedBox(width: Surface.px(10.5), height: Surface.px(10.5), child: CustomPaint(painter: RoleGlyph(TMode.anchor, mode == TMode.anchor ? N.g10 : kViolet))))),
        // the nine places, always at hand: one click chooses, hovering shows the pivot on the Stage
        MouseRegion(
          onExit: (_) => s.anchorPreview.value = null,
          child: Container(
            padding: EdgeInsets.all(Surface.px(2)),
            decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(Surface.px(4))),
            child: Column(children: [
              for (var iy = 0; iy < 3; iy++) Row(children: [
                for (var ix = 0; ix < 3; ix++) MouseRegion(
                  onEnter: (_) => s.anchorPreview.value = [[0.0, .5, 1.0][ix], [0.0, .5, 1.0][iy]],
                  child: GestureDetector(
                    key: ValueKey('anchor-cell-$ix$iy'),
                    behavior: HitTestBehavior.opaque,
                    onTap: s.canEdit ? () => s.setAnchor([0.0, .5, 1.0][ix], [0.0, .5, 1.0][iy]) : null,
                    child: SizedBox(width: Surface.px(9), height: Surface.px(6), child: Center(child: Container(width: ix == ax && iy == ay ? Surface.px(6) : Surface.px(3), height: ix == ax && iy == ay ? Surface.px(6) : Surface.px(3), decoration: BoxDecoration(color: ix == ax && iy == ay ? (s.canEdit ? kViolet : N.g33) : N.g44, shape: BoxShape.circle)))),
                  ),
                ),
              ]),
            ]),
          ),
        ),
        SizedBox(width: Surface.sectionGap),
        Expanded(child: Text(name, key: const ValueKey('anchor-name'), softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: s.canEdit ? N.g69 : N.g33, w: FontWeight.w500))),
      ]),
    );
  }
}

// ---- small marks -------------------------------------------------------------------------------------------------
class RoleGlyph extends CustomPainter {
  RoleGlyph(this.m, this.col);
  final TMode m;
  final Color col;
  // surface-block: painter geometry: drawn in the canvas's own pixels (method paint)
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final f = Paint()..color = col;
    final w = s.width, h = s.height, cx = w / 2, cy = h / 2;
    switch (m) {
      case TMode.move:
        c.drawLine(Offset(cx, 1), Offset(cx, h - 1), p);
        c.drawLine(Offset(1, cy), Offset(w - 1, cy), p);
        for (final (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)]) {
          final tip = Offset(cx + dx * (w / 2 - 1), cy + dy * (h / 2 - 1));
          final back = dx == 0 ? const Offset(2.4, 0) : const Offset(0, 2.4);
          final ax = Offset(-dx * 2.6, -dy * 2.6);
          c.drawPath(Path()..moveTo(tip.dx, tip.dy)..lineTo(tip.dx + ax.dx + back.dx, tip.dy + ax.dy + back.dy)..lineTo(tip.dx + ax.dx - back.dx, tip.dy + ax.dy - back.dy)..close(), f);
        }
      case TMode.scale:
        c.drawRect(Rect.fromLTWH(3, 5, w - 8, h - 8), p);
        c.drawLine(Offset(w - 5, 3), Offset(w - 1.5, 0.5), p);
        c.drawLine(Offset(w - 3.4, 0.5), Offset(w - 1.5, 0.5), p);
        c.drawLine(Offset(w - 1.5, 0.5), Offset(w - 1.5, 2.6), p);
      case TMode.rotate:
        c.drawArc(Rect.fromCircle(center: Offset(cx, cy), radius: w / 2 - 2), -2.6, 4.7, false, p);
        c.drawPath(Path()..moveTo(w - 2.2, 4.2)..lineTo(w - 1.8, 8.4)..lineTo(w - 6, 7.4)..close(), f);
      case TMode.anchor:
        for (var iy = 0; iy < 3; iy++) { for (var ix = 0; ix < 3; ix++) { c.drawCircle(Offset(2 + ix * (w - 4) / 2, 2 + iy * (h - 4) / 2), ix == 1 && iy == 1 ? 1.9 : 1.1, f); } }
    }
  }
  @override
  bool shouldRepaint(RoleGlyph o) => o.m != m || o.col != col;
}

class _OpacityGlyph extends StatelessWidget {
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _OpP());
}

class _OpP extends CustomPainter {
  @override
  void paint(Canvas c, Size s) {
    c.drawCircle(s.center(Offset.zero), s.width / 2 - 1, Paint()..color = kViolet..style = PaintingStyle.stroke..strokeWidth = 1.4);
    c.drawArc(Rect.fromCircle(center: s.center(Offset.zero), radius: s.width / 2 - 1), -1.57, 3.14, true, Paint()..color = kViolet);
  }
  @override
  bool shouldRepaint(_OpP o) => false;
}

class _DepthGlyph extends StatelessWidget {
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _DepP());
}

class _DepP extends CustomPainter {
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = kViolet..style = PaintingStyle.stroke..strokeWidth = 1.4;
    c.drawRect(const Rect.fromLTWH(1, 5, 8, 8), p);
    c.drawRect(const Rect.fromLTWH(5, 1, 8, 8), p);
  }
  @override
  bool shouldRepaint(_DepP o) => false;
}

class _DiamondP extends CustomPainter {
  _DiamondP(this.filled, this.col);
  final bool filled;
  final Color col;
  @override
  void paint(Canvas c, Size s) {
    final p = Path()..moveTo(s.width / 2, 0)..lineTo(s.width, s.height / 2)..lineTo(s.width / 2, s.height)..lineTo(0, s.height / 2)..close();
    c.drawPath(p, Paint()..color = col..style = filled ? PaintingStyle.fill : PaintingStyle.stroke..strokeWidth = 1.3);
  }
  @override
  bool shouldRepaint(_DiamondP o) => o.filled != filled || o.col != col;
}

class _LinkP extends CustomPainter {
  _LinkP(this.col, this.on);
  final Color col;
  final bool on;
  // surface-block: painter geometry: drawn in the canvas's own pixels (method paint)
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = 1.6..strokeCap = StrokeCap.round;
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(0.5, 4.5, 7, 5), const Radius.circular(2)), p);
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(6.5, 4.5, 7, 5), const Radius.circular(2)), p);
    if (!on) c.drawLine(const Offset(2, 12.5), const Offset(12, 1.5), Paint()..color = col..strokeWidth = 1.4);
  }
  @override
  bool shouldRepaint(_LinkP o) => o.on != on || o.col != col;
}
