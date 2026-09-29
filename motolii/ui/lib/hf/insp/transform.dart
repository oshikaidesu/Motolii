// The Transform Instrument: one gizmo and the exact values that belong to it, on one surface.
// Position, Scale, Rotation and Anchor are four ways of touching the same body, not four property cards.
// Space, Parent and Depth sit around it. Precision is the Phase 1 Value; the instrument adds gesture, not a second system.
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../desk/common.dart' show kYellow, kInk, kBlue, kPink, kViolet, kAccentDim;
import '../metrics.dart';
import 'rows.dart';
import 'toys.dart';
import 'transform_gizmo.dart';
import 'transform_model.dart';
import '../neutral.dart';
import '../shell/place.dart' show H;

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
          final narrow = box.maxWidth < 230;
          final pad = narrow ? 8.0 : 12.0;
          final inner = box.maxWidth - pad * 2;
          final l = s.active;
          final gizmoW = narrow ? inner : inner - 28 - 6 - 6 - 78;
          final gizmo = Stack(children: [
            TransformGizmo(s, mode: mode, onMode: (m) => setState(() => mode = m), rotAxis: rotAxis, size: Size(gizmoW, narrow ? 132 : 172)),
            if (mode == TMode.rotate && l.projection != '2D')
              Positioned(right: 6, bottom: 6, child: Row(children: [
                for (final (i, t) in ['Z', 'X', 'Y'].indexed)
                  GestureDetector(key: ValueKey('rot-axis-$i'), onTap: () => setState(() => rotAxis = i), child: Container(margin: const EdgeInsets.only(left: 3), width: 20, height: 18, alignment: Alignment.center, decoration: BoxDecoration(color: rotAxis == i ? kPink : N.g15, borderRadius: BorderRadius.circular(4)), child: Text(t, style: sans(9.5, c: rotAxis == i ? N.g10 : kMuted, w: FontWeight.w700)))),
              ])),
          ]);
          final strip = _modeStrip(narrow);
          final body = Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            if (widget.showHeader) ...[_header(), const SizedBox(height: 8)],
            if (narrow) ...[strip, const SizedBox(height: 6), gizmo, const SizedBox(height: 6), _spaceRow(), const SizedBox(height: 6), _parent(narrow)]
            else Row(crossAxisAlignment: CrossAxisAlignment.start, children: [strip, const SizedBox(width: 6), gizmo, const SizedBox(width: 6), SizedBox(width: 78, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_spaceCol(), const SizedBox(height: 8), _parent(narrow)]))]),
            const SizedBox(height: 10),
            _rows(narrow),
            const SizedBox(height: 10),
            _world(),
          ]);
          // A seat short enough (a resized Dock pane, a smaller window) that even the wide layout doesn't fit its
          // own height scrolls instead of overflowing — the same rule the narrow layout already followed.
          return SingleChildScrollView(
            key: const ValueKey('tf-scroll'),
            padding: EdgeInsets.fromLTRB(pad, 10, pad, narrow ? 16 : 10),
            child: body,
          );
        }),
      );

  // ---- header: who is shown, and Animate ----------------------------------------------------------------------
  Widget _header() {
    final l = s.active;
    return SizedBox(
      height: 22,
      child: Row(children: [
        Container(width: 3, height: 14, margin: const EdgeInsets.only(right: 7), decoration: BoxDecoration(color: modeColor[mode], borderRadius: BorderRadius.circular(1.5))),
        Expanded(child: Text(s.multiple ? '${s.selection.length} layers' : l.name, key: const ValueKey('tf-title'), softWrap: false, overflow: TextOverflow.clip, style: sans(13, c: kInk, w: FontWeight.w600))),
        if (l.locked) Padding(padding: const EdgeInsets.only(right: 8), child: Text('Locked', style: sans(9.5, c: kMuted, w: FontWeight.w600))),
        GestureDetector(
          key: const ValueKey('tf-animate'),
          behavior: HitTestBehavior.opaque,
          onTap: s.toggleAnimate,
          child: Row(children: [
            SizedBox(width: 12, height: 12, child: CustomPaint(painter: _DiamondP(s.animating, kYellow))),
            const SizedBox(width: 5),
            Text('Animate', style: sans(10.5, c: s.animating ? kYellow : kMuted, w: FontWeight.w600)),
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
            width: narrow ? null : 28,
            height: narrow ? 26 : 32,
            margin: EdgeInsets.only(bottom: narrow ? 0 : 4, right: narrow ? 4 : 0),
            decoration: BoxDecoration(color: mode == m ? modeColor[m] : N.g15, borderRadius: BorderRadius.circular(5)),
            child: Center(child: SizedBox(width: 15, height: 15, child: CustomPaint(painter: RoleGlyph(m, mode == m ? N.g10 : modeColor[m]!)))),
          ),
        );
    return narrow ? Row(children: [for (final m in TMode.values) Expanded(child: b(m))]) : Column(children: [for (final m in TMode.values) b(m)]);
  }

  // ---- around the instrument: Space, Parent ---------------------------------------------------------------------
  Widget _chip(String key, String t, bool on, VoidCallback f, {double h = 24}) => GestureDetector(
        key: ValueKey(key),
        behavior: HitTestBehavior.opaque,
        onTap: s.canEdit ? f : null,
        child: Container(height: h, margin: const EdgeInsets.only(bottom: 3), alignment: Alignment.center, decoration: BoxDecoration(color: on ? (s.canEdit ? kYellow : kAccentDim) : kTile, borderRadius: BorderRadius.circular(5)), child: Text(t, style: sans(11, c: on ? N.g10 : (s.canEdit ? N.g69 : N.g33), w: on ? FontWeight.w700 : FontWeight.w500))),
      );

  Widget _spaceCol() => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('SPACE', style: sans(9, c: N.g51, w: FontWeight.w600, ls: 1.2))),
        for (final p in ['2D', '2.5D', '3D']) _chip('space-$p', p, s.active.projection == p, () => s.setProjection(p)),
      ]);

  Widget _spaceRow() => Row(children: [for (final p in ['2D', '2.5D', '3D']) Expanded(child: Padding(padding: const EdgeInsets.only(right: 3), child: _chip('space-$p', p, s.active.projection == p, () => s.setProjection(p))))]);

  Widget _parent(bool narrow) {
    final others = [for (final l in s.layers) if (l.id != s.activeId) l];
    final cur = s.active.parent;
    final names = ['None', for (final l in others) l.name];
    var idx = cur == null ? 0 : (others.indexWhere((l) => l.id == cur) + 1).clamp(0, names.length - 1);
    void pick(int i) {
      final n = (i + names.length) % names.length;
      s.setParent(n == 0 ? null : others[n - 1].id);
    }
    Widget arrow(String k, String t, int d) => GestureDetector(key: ValueKey('parent-$k'), behavior: HitTestBehavior.opaque, onTap: s.canEdit ? () => pick(idx + d) : null, child: SizedBox(width: 20, height: 26, child: Center(child: Text(t, style: sans(15, c: kMuted)))));
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('PARENT', style: sans(9, c: N.g51, w: FontWeight.w600, ls: 1.2))),
      Container(height: 26, decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)), child: Row(children: [arrow('prev', '‹', -1), Expanded(child: Center(child: Text(names[idx], key: const ValueKey('parent-name'), softWrap: false, overflow: TextOverflow.clip, style: sans(11, c: s.canEdit ? N.g82 : N.g33, w: FontWeight.w500)))), arrow('next', '›', 1)])),
    ]);
  }

  // ---- what the layer meets the scene with: how it blends and (an Image only) its flags -------------------------
  Widget _flag(String key, String label, bool on, VoidCallback? tap) => GestureDetector(
        key: ValueKey('world-$key'),
        behavior: HitTestBehavior.opaque,
        onTap: tap,
        child: Container(
          height: 26,
          margin: const EdgeInsets.only(bottom: 4),
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)),
          child: Row(children: [
            Container(width: 26, height: 15, padding: const EdgeInsets.all(2), alignment: on ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: on ? kYellow : N.g26, borderRadius: BorderRadius.circular(8)), child: Container(width: 11, height: 11, decoration: const BoxDecoration(color: kInk, shape: BoxShape.circle))),
            const SizedBox(width: 7),
            Expanded(child: Text(label, style: sans(11, c: tap == null ? N.g33 : N.g82))),
          ]),
        ),
      );

  Widget _world() {
    final l = s.active;
    if (l.kind == 'Camera') return const SizedBox.shrink();
    final edit = s.canEdit;
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('WORLD', style: sans(9, c: N.g51, w: FontWeight.w600, ls: 1.2))),
      GestureDetector(
        key: const ValueKey('world-blend'),
        behavior: HitTestBehavior.opaque,
        onTap: edit ? s.openBlend : null,
        child: Container(height: 26, margin: const EdgeInsets.only(bottom: 4), alignment: Alignment.centerLeft, padding: const EdgeInsets.symmetric(horizontal: 8), decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)), child: Text(l.blendMode, key: const ValueKey('world-blend-name'), style: sans(11, c: edit ? N.g82 : N.g33, w: FontWeight.w500))),
      ),
      if (l.kind == 'Image') _flag('environment', 'Environment: this image lights and surrounds the scene', l.environment, edit ? () => s.setWorldFlag('environment', !l.environment) : null),
      if (l.ghostable) _flag('ghost', 'Ghost: the same layer seen later by a delay', l.ghost != null, edit && s.worldCan('ghost') ? () => s.setWorldFlag('ghost', l.ghost == null) : null),
      _flag('clip', 'Clip to the layer below', l.clipToBelow, edit && s.worldCan('clip') ? () => s.setWorldFlag('clipToBelow', null) : null),
    ]);
  }

  // ---- the exact values: Phase 1 Values, grouped by what they are ------------------------------------------------
  Widget _rows(bool narrow) {
    final l = s.active;
    final out2D = l.projection != '2D';
    Slot slot(String id, [int? axis, String? link]) => Slot(s, id, axis, link);
    Widget val(Slot sl, TMode? m, {String? tag, String? nodeKey, bool units = true}) => Expanded(child: ValueToy(sl, tag: tag, tone: m == null ? N.g56 : modeColor[m]!, showUnit: units && !narrow, decimals: narrow ? 0 : null, focusNode: node(nodeKey ?? '${sl.id}${sl.axis == null ? '' : ':${sl.axis}'}')));
    Widget gap() => const SizedBox(width: 3);

    // Position: X, Y, and Z outside 2D
    final px = slot('position', 0), py = slot('position', 1);
    // three numbers across need room; a narrow panel stacks the third under the pair rather than shrinking the numbers
    Widget three(Widget a, Widget b, Widget c3) => narrow
        ? Column(children: [Row(children: [a, gap(), b]), const SizedBox(height: 3), Row(children: [c3])])
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
      if (out2D) _line(null, 'depth', ['depth'], Row(children: [val(slot('depth'), null), const SizedBox(width: 5), _routePill()]), narrow, glyph: _DepthGlyph()),
    ]);
  }

  Widget _routePill() => GestureDetector(
        key: const ValueKey('route-depth'),
        behavior: HitTestBehavior.opaque,
        onTap: () => s.route('Depth', 'depth'),
        child: Container(height: 22, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: kViolet.withValues(alpha: .7)), borderRadius: BorderRadius.circular(11)), child: Text('Depth →', style: sans(9.5, c: kViolet, w: FontWeight.w700))),
      );

  Widget _linkChip() => GestureDetector(
        key: const ValueKey('link-scale'),
        behavior: HitTestBehavior.opaque,
        onTap: s.canEdit ? () => s.toggleLink('scale') : null,
        child: Container(width: 26, height: UiMetrics.control, decoration: BoxDecoration(color: s.linked.contains('scale') ? kBlue : kTile, borderRadius: BorderRadius.circular(5)), child: Center(child: SizedBox(width: 14, height: 14, child: CustomPaint(painter: _LinkP(s.linked.contains('scale') ? N.g10 : kMuted, s.linked.contains('scale')))))),
      );

  bool _modified(List<String> ids) => ids.any((i) => s.rows.any((r) => r['id'] == i) && modified(s.row(i)));
  bool _keyed(String id) => s.active.keyedNow.contains(id);
  bool _animated(String id) => s.active.animated.contains(id);

  Widget _marks(TMode? m, String id, List<String> ids) {
    final tone = m == null ? kViolet : modeColor[m]!;
    return Row(mainAxisSize: MainAxisSize.min, children: [
      GestureDetector(key: ValueKey('key-$id'), behavior: HitTestBehavior.opaque, onTap: () => s.toggleKey(id), child: Padding(padding: const EdgeInsets.symmetric(horizontal: 4), child: SizedBox(width: 10, height: 10, child: CustomPaint(painter: _DiamondP(_keyed(id), _animated(id) || _keyed(id) ? tone : N.g26))))),
      if (_modified(ids)) GestureDetector(key: ValueKey('reset-$id'), behavior: HitTestBehavior.opaque, onTap: () => s.resetMany(ids), child: Padding(padding: const EdgeInsets.only(left: 2), child: Text('↺', style: sans(11, c: N.g38)))) else const SizedBox(width: 12),
    ]);
  }

  Widget _line(TMode? m, String id, List<String> ids, Widget values, bool narrow, {Widget? extra, Widget? glyph}) {
    final active = m != null && mode == m;
    final tone = m == null ? kViolet : modeColor[m]!;
    return Container(
      key: ValueKey('row-$id'),
      margin: const EdgeInsets.only(bottom: UiMetrics.cellGap),
      child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
        GestureDetector(
          key: ValueKey('glyph-$id'),
          behavior: HitTestBehavior.opaque,
          onTap: m == null ? null : () => setState(() => mode = m),
          child: Container(width: 22, height: UiMetrics.control, margin: const EdgeInsets.only(right: 5), alignment: Alignment.center, decoration: BoxDecoration(color: active ? tone : null, borderRadius: BorderRadius.circular(5)), child: SizedBox(width: 14, height: 14, child: glyph ?? CustomPaint(painter: RoleGlyph(m!, active ? N.g10 : tone)))),
        ),
        Expanded(child: values),
        // every line keeps the same end column (the scale line's link chip, empty elsewhere): the X · Y · Z columns line up
        SizedBox(width: 30, child: Align(alignment: Alignment.centerRight, child: extra)),
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
          margin: const EdgeInsets.only(left: 6),
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(color: H.relation, borderRadius: BorderRadius.circular(6)),
          child: Text(driven != null ? '◉ ${(driven['link'] as Map)['name'] ?? 'Relation'}' : '◉ $drives', style: sans(8.5, c: N.g10, w: FontWeight.w700)),
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
      margin: const EdgeInsets.only(bottom: UiMetrics.cellGap),
      child: Row(children: [
        GestureDetector(key: const ValueKey('glyph-anchor'), behavior: HitTestBehavior.opaque, onTap: () => setState(() => mode = TMode.anchor), child: Container(width: 22, height: UiMetrics.control, margin: const EdgeInsets.only(right: 5), alignment: Alignment.center, decoration: BoxDecoration(color: mode == TMode.anchor ? kViolet : null, borderRadius: BorderRadius.circular(5)), child: SizedBox(width: 14, height: 14, child: CustomPaint(painter: RoleGlyph(TMode.anchor, mode == TMode.anchor ? N.g10 : kViolet))))),
        // the nine places, always at hand: one click chooses, hovering shows the pivot on the Stage
        MouseRegion(
          onExit: (_) => s.anchorPreview.value = null,
          child: Container(
            padding: const EdgeInsets.all(3),
            decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)),
            child: Column(children: [
              for (var iy = 0; iy < 3; iy++) Row(children: [
                for (var ix = 0; ix < 3; ix++) MouseRegion(
                  onEnter: (_) => s.anchorPreview.value = [[0.0, .5, 1.0][ix], [0.0, .5, 1.0][iy]],
                  child: GestureDetector(
                    key: ValueKey('anchor-cell-$ix$iy'),
                    behavior: HitTestBehavior.opaque,
                    onTap: s.canEdit ? () => s.setAnchor([0.0, .5, 1.0][ix], [0.0, .5, 1.0][iy]) : null,
                    child: SizedBox(width: 12, height: 8, child: Center(child: Container(width: ix == ax && iy == ay ? 6 : 3, height: ix == ax && iy == ay ? 6 : 3, decoration: BoxDecoration(color: ix == ax && iy == ay ? (s.canEdit ? kViolet : N.g33) : N.g44, shape: BoxShape.circle)))),
                  ),
                ),
              ]),
            ]),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(child: Text(name, key: const ValueKey('anchor-name'), softWrap: false, overflow: TextOverflow.clip, style: sans(11, c: s.canEdit ? N.g69 : N.g33, w: FontWeight.w500))),
      ]),
    );
  }
}

// ---- small marks -------------------------------------------------------------------------------------------------
class RoleGlyph extends CustomPainter {
  RoleGlyph(this.m, this.col);
  final TMode m;
  final Color col;
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
          final back = dx == 0 ? Offset(2.4, 0) : Offset(0, 2.4);
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
    c.drawRect(Rect.fromLTWH(1, 5, 8, 8), p);
    c.drawRect(Rect.fromLTWH(5, 1, 8, 8), p);
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
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = 1.6..strokeCap = StrokeCap.round;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(0.5, 4.5, 7, 5), const Radius.circular(2.5)), p);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(6.5, 4.5, 7, 5), const Radius.circular(2.5)), p);
    if (!on) c.drawLine(const Offset(2, 12.5), const Offset(12, 1.5), Paint()..color = col..strokeWidth = 1.4);
  }
  @override
  bool shouldRepaint(_LinkP o) => o.on != on || o.col != col;
}
