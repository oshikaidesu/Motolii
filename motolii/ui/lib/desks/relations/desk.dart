import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../browser/parts.dart';

import '../../session/editor_session.dart';
import 'model.dart';
import '../../theme/neutral.dart';
import 'session.dart';
import '../../theme/surface.dart' show Dn, Surface;

/// Relations v0: the place to pick the things a source drives. The things at the current time are dots where they are
/// on the Stage; a click, a Shift click or a lasso makes the member set. The relation itself is the links the document
/// already holds (`relate`); this panel only projects and edits them.
class RelationsPanel extends StatefulWidget {
  const RelationsPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<RelationsPanel> createState() => _RelationsPanelState();
}

class _RelationsPanelState extends State<RelationsPanel> {
  EditorSession get c => widget.controller;
  static const _watched = ['selectedIds', 'frame', 'capabilities', 'width', 'height'];

  /// What the panel draws of the layers: each dot (name, place, when it exists), every link, and the value of each
  /// link's source. Opacity, an effect parameter or a camera value moving changes none of it; Position moves the dots.
  Object? _reading() => [
        for (final l in c.layers) [l['id'], l['name'], l['kind'], l['x'], l['y'], l['start'], l['duration']],
        for (final r in relationsOf(c))
          [r.source.layer, r.source.property, r.source.component, r.inMin, r.inMax, r.members, [for (final m in r.mappings) [m.property, m.outMin, m.outMax]], s.sourceValue({'layer': r.source.layer, 'property': r.source.property, 'component': r.source.component})],
        if (s.draft != null) s.sourceValue(EditorSession.map(s.draft!['source'])),
      ];
  DocumentSlice get _slice => c.slice('relations', _watched, derived: _reading);

  /// What the Relations desk is doing lives in the RelationsSession (it outlives this panel); the panel keeps only
  /// its picture: the dots and the lasso being drawn.
  late final RelationsSession s = RelationsSession.of(c);
  Set<int>? get picking => s.picking;
  set picking(Set<int>? v) => s.picking = v;
  Map<String, dynamic>? get draft => s.draft;
  List<Offset>? lasso;
  final focus = FocusNode(debugLabel: 'relations');

  @override
  void initState() {
    super.initState();
    _slice.addListener(_redraw);
    c.relationFocus.addListener(_redraw);
    s.addListener(_redraw);
    s.arrived.addListener(_arrived);
  }

  @override
  void dispose() {
    _slice.removeListener(_redraw);
    c.relationFocus.removeListener(_redraw);
    s.removeListener(_redraw);
    s.arrived.removeListener(_arrived);
    focus.dispose();
    super.dispose();
  }

  void _redraw() {
    if (mounted) setState(() {});
  }

  void _arrived() {
    lasso = null;
    focus.requestFocus();
  }

  double _sourceValue(Map<String, dynamic> src) => s.sourceValue(src);
  List<Relation> get relations => s.relations;
  Relation? get focused => s.focused;
  Future<void> _relate(RelationSource src, double inMin, double inMax, Set<int> members, String property, double outMin, double outMax) =>
      s.relate(src, inMin, inMax, members, property, outMin, outMax);
  Future<void> _create() => s.create();
  void _cancel() {
    lasso = null;
    s.cancel();
  }

  // ---- the dots -------------------------------------------------------------------------------------------------
  List<Map<String, dynamic>> get present {
    final frame = c.frame.value;
    return [
      for (final l in c.layers)
        if (!const ['Camera', 'Stage'].contains(l['kind']))
          if (l['start'] is! num || (frame >= (l['start'] as num) && frame < (l['start'] as num) + ((l['duration'] as num?) ?? double.infinity))) l,
    ];
  }

  Offset _dot(Map<String, dynamic> l, Size box) {
    final w = (c.state['width'] as num?)?.toDouble() ?? 1920, h = (c.state['height'] as num?)?.toDouble() ?? 1080;
    final x = (l['x'] as num?)?.toDouble() ?? w / 2, y = (l['y'] as num?)?.toDouble() ?? h / 2;
    const pad = 28.0;
    return Offset(pad + (x / w).clamp(-.2, 1.2) * (box.width - pad * 2), pad + (y / h).clamp(-.2, 1.2) * (box.height - pad * 2));
  }

  bool _inside(List<Offset> poly, Offset p) {
    var inside = false;
    for (var i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      final a = poly[i], b = poly[j];
      if ((a.dy > p.dy) != (b.dy > p.dy) && p.dx < (b.dx - a.dx) * (p.dy - a.dy) / (b.dy - a.dy) + a.dx) inside = !inside;
    }
    return inside;
  }

  Widget _graph() => LayoutBuilder(builder: (context, box) {
        final size = Size(box.maxWidth, box.maxHeight);
        final things = present;
        final rel = focused;
        final src = draft != null ? RelationSource.fromDraft(draft!['source'] as Map<String, dynamic>) : rel?.source;
        final members = picking ?? rel?.members.toSet() ?? const <int>{};
        int? hit(Offset p) {
          for (final l in things) {
            if ((_dot(l, size) - p).distance < 14) return l['id'] as int;
          }
          return null;
        }
        return Focus(
          focusNode: focus,
          onKeyEvent: (_, e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && (draft != null || picking != null)) {
              _cancel();
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: Listener(
            onPointerDown: (e) {
              focus.requestFocus();
              final id = hit(e.localPosition);
              if (id == null) {
                if (picking != null) setState(() => lasso = [e.localPosition]);
                return;
              }
              if (picking != null) {
                if (id == src?.layer) return;
                setState(() {
                  if (HardwareKeyboard.instance.isShiftPressed || picking!.contains(id)) {
                    picking!.contains(id) ? picking!.remove(id) : picking!.add(id);
                  } else {
                    picking!
                      ..clear()
                      ..add(id);
                  }
                });
              } else {
                c.command('select', {'ids': [id]});
                final own = relations.where((r) => r.source.layer == id || r.members.contains(id)).firstOrNull;
                if (own != null) c.relationFocus.value = {'layer': own.source.layer, 'property': own.source.property, 'component': own.source.component};
              }
            },
            onPointerMove: (e) {
              if (lasso != null) setState(() => lasso!.add(e.localPosition));
            },
            onPointerUp: (_) {
              final poly = lasso;
              if (poly == null) return;
              setState(() {
                lasso = null;
                if (poly.length < 3) return;
                final caught = {for (final l in things) if (l['id'] != src?.layer && _inside(poly, _dot(l, size))) l['id'] as int};
                if (HardwareKeyboard.instance.isShiftPressed) {
                  picking!.addAll(caught);
                } else {
                  picking = caught;
                }
              });
            },
            child: CustomPaint(
              key: const ValueKey('relations-graph'),
              size: size,
              painter: _GraphPainter(things: things, dots: {for (final l in things) l['id'] as int: _dot(l, size)}, source: src?.layer, members: members, lasso: lasso, picking: picking != null, mappings: rel?.mappings.length ?? 0),
            ),
          ),
        );
      });

  // ---- the side: the draft, or the focused relation ---------------------------------------------------------------
  Widget _side() {
    if (draft != null) return _draftCard();
    final rel = focused;
    if (rel != null) return _relationCard(rel);
    final all = relations;
    return Padding(
      padding: const EdgeInsets.all(9),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        if (all.isEmpty) Text('No relation yet. In the Inspector, right-click a value and choose Relation…', style: sans(Dn.nameSize, c: Surface.muted)),
        for (final r in all)
          GestureDetector(
            key: ValueKey('relation-row-${r.source.layer}-${r.source.property}-${r.source.component}'),
            onTap: () => c.relationFocus.value = {'layer': r.source.layer, 'property': r.source.property, 'component': r.source.component},
            child: Container(
              margin: const EdgeInsets.only(bottom: 4.5),
              padding: const EdgeInsets.all(6),
              decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(4)),
              child: Row(children: [
                Container(width: 6, height: 6, decoration: const BoxDecoration(color: kRed, shape: BoxShape.circle)),
                const SizedBox(width: 6),
                Expanded(child: Text('${r.source.name(c)} → ${r.members.length} thing${r.members.length == 1 ? '' : 's'} · ${r.mappings.map((m) => labelOf(m.property)).join(', ')}', softWrap: false, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: Surface.ink))),
              ]),
            ),
          ),
      ]),
    );
  }

  Widget _label(String t) => Padding(padding: const EdgeInsets.only(top: 9, bottom: 4), child: Text(t, style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w600, ls: 1.3)));

  Widget _draftCard() {
    final d = draft!;
    final src = RelationSource.fromDraft(d['source'] as Map<String, dynamic>);
    final n = picking?.length ?? 0;
    final dest = d['destination'] as String?;
    final unit = dest == null ? null : unitOf(dest);
    return Padding(
      padding: const EdgeInsets.all(9),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [Container(width: 6, height: 6, decoration: const BoxDecoration(color: kRed, shape: BoxShape.circle)), const SizedBox(width: 6), Expanded(child: Text('${src.name(c)}', softWrap: false, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600)))]),
        _label('SOURCE RANGE'),
        _range('in', d['inMin'], d['inMax'], 'px', (lo, hi) => setState(() { d['inMin'] = lo; d['inMax'] = hi; }), current: _sourceValue(d['source'] as Map<String, dynamic>)),
        _label('MEMBERS'),
        Text(n == 0 ? 'Click or lasso things in the graph.' : '$n thing${n == 1 ? '' : 's'} selected', key: const ValueKey('relations-count'), style: sans(Dn.nameSize, c: n == 0 ? Surface.muted : Surface.ink)),
        _label('DESTINATION'),
        Wrap(spacing: 4, runSpacing: 4, children: [
          for (final p in destinations)
            GestureDetector(
              key: ValueKey('dest-$p'),
              onTap: n == 0 ? null : () => setState(() { d['destination'] = p; final u = unitOf(p); d['outMin'] = u.defaultRange.$1; d['outMax'] = u.defaultRange.$2; }),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 7.5, vertical: 4),
                decoration: BoxDecoration(color: dest == p ? kRed : Surface.raised, borderRadius: BorderRadius.circular(3), border: Border.all(color: dest == p ? kRed : Surface.dividerFine)),
                child: Text(labelOf(p), style: sans(Dn.nameSize, c: dest == p ? N.g10 : (n == 0 ? Surface.muted : Surface.ink), w: FontWeight.w600)),
              ),
            ),
        ]),
        if (dest != null) ...[
          _label('${labelOf(dest).toUpperCase()} RANGE'),
          _range('out', d['outMin'], d['outMax'], unit!.suffix, (lo, hi) => setState(() { d['outMin'] = lo; d['outMax'] = hi; })),
        ],
        const SizedBox(height: 10.5),
        Row(children: [
          GestureDetector(key: const ValueKey('relations-cancel'), onTap: _cancel, child: Container(padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 4.5), decoration: BoxDecoration(border: Border.all(color: Surface.dividerFine), borderRadius: BorderRadius.circular(3)), child: Text('Cancel', style: sans(Dn.nameSize, c: Surface.muted)))),
          const Spacer(),
          GestureDetector(
            key: const ValueKey('relations-create'),
            onTap: n > 0 && dest != null ? _create : null,
            child: Container(padding: const EdgeInsets.symmetric(horizontal: 10.5, vertical: 4.5), decoration: BoxDecoration(color: n > 0 && dest != null ? kRed : Surface.raised, borderRadius: BorderRadius.circular(3)), child: Text('Create', style: sans(Dn.nameSize, c: n > 0 && dest != null ? N.g10 : Surface.muted, w: FontWeight.w700))),
          ),
        ]),
      ]),
    );
  }

  Widget _relationCard(Relation r) {
    final editing = picking != null;
    return Padding(
      padding: const EdgeInsets.all(9),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          Container(width: 6, height: 6, decoration: const BoxDecoration(color: kRed, shape: BoxShape.circle)),
          const SizedBox(width: 6),
          Expanded(child: Text(r.source.name(c), softWrap: false, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600))),
          GestureDetector(key: const ValueKey('relation-delete'), onTap: () => RelationsSession.of(c).remove(r), child: Text('✕', style: sans(Dn.nameSize, c: Surface.muted))),
        ]),
        _label('SOURCE RANGE'),
        _range('in', _shown('in', r.inMin, r.inMax).$1, _shown('in', r.inMin, r.inMax).$2, 'px', (lo, hi) => s.scrubRange('in', lo, hi), onDone: () => _write(r), current: _sourceValue({'layer': r.source.layer, 'property': r.source.property, 'component': r.source.component})),
        _label('MEMBERS'),
        Row(children: [
          Expanded(child: Text('${(picking ?? r.members.toSet()).length} things', key: const ValueKey('relations-count'), style: sans(Dn.nameSize, c: Surface.ink))),
          GestureDetector(
            key: const ValueKey('relations-edit-members'),
            onTap: () async {
              if (!editing) {
                setState(() => picking = r.members.toSet());
                return;
              }
              await _write(r, members: picking!);
              setState(() => picking = null);
            },
            child: Container(padding: const EdgeInsets.symmetric(horizontal: 7.5, vertical: 3), decoration: BoxDecoration(color: editing ? kRed : Surface.raised, borderRadius: BorderRadius.circular(3)), child: Text(editing ? 'Done' : 'Edit in graph', style: sans(Dn.nameSize, c: editing ? N.g10 : Surface.ink, w: FontWeight.w600))),
          ),
        ]),
        _label('MAPPINGS'),
        for (final m in r.mappings) ...[
          Row(children: [
            Expanded(child: Text(labelOf(m.property), key: ValueKey('mapping-${m.property}'), style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600))),
            GestureDetector(key: ValueKey('mapping-remove-${m.property}'), onTap: () => c.command('unrelate', {'layers': r.members, 'property': m.property}), child: Text('✕', style: sans(Dn.nameSize, c: Surface.muted))),
          ]),
          _range('out-${m.property}', _shown('out-${m.property}', unitOf(m.property).fromDoc(m.outMin), unitOf(m.property).fromDoc(m.outMax)).$1, _shown('out-${m.property}', unitOf(m.property).fromDoc(m.outMin), unitOf(m.property).fromDoc(m.outMax)).$2, unitOf(m.property).suffix, (lo, hi) => s.scrubRange('out-${m.property}', lo, hi), onDone: () => _write(r)),
          const SizedBox(height: 4.5),
        ],
        Wrap(spacing: 4, runSpacing: 4, children: [
          for (final p in destinations.where((p) => !r.mappings.any((m) => m.property == p)))
            GestureDetector(
              key: ValueKey('add-mapping-$p'),
              onTap: () { final u = unitOf(p); _relate(r.source, r.inMin, r.inMax, r.members.toSet(), p, u.toDoc(u.defaultRange.$1), u.toDoc(u.defaultRange.$2)); },
              child: Container(padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3), decoration: BoxDecoration(border: Border.all(color: Surface.dividerFine), borderRadius: BorderRadius.circular(3)), child: Text('+ ${labelOf(p)}', style: sans(Dn.nameSize, c: kInk2))),
            ),
        ]),
      ]),
    );
  }

  (double, double) _shown(String key, double lo, double hi) => s.shown(key, lo, hi);
  Future<void> _write(Relation r, {Set<int>? members}) => s.write(r, members: members);

  /// Two ends of a range, each scrubbed, with the value now marked between them.
  Widget _range(String key, double lo, double hi, String suffix, void Function(double, double) change, {VoidCallback? onDone, double? current}) => Row(children: [
        _End('$key-min', lo, suffix, (v) => change(v, hi), onDone),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 6),
            child: SizedBox(height: 10.5, child: CustomPaint(painter: _RangePainter(current == null ? null : ((current - lo) / ((hi - lo).abs() < 1e-9 ? 1 : hi - lo)).clamp(0.0, 1.0)))),
          ),
        ),
        _End('$key-max', hi, suffix, (v) => change(lo, v), onDone),
        if (current != null) ...[
          const SizedBox(width: 4.5),
          GestureDetector(key: ValueKey('$key-set-min'), onTap: () { change(current, hi); onDone?.call(); }, child: Text('⇤', style: sans(Dn.nameSize, c: Surface.muted))),
          const SizedBox(width: 3),
          GestureDetector(key: ValueKey('$key-set-max'), onTap: () { change(lo, current); onDone?.call(); }, child: Text('⇥', style: sans(Dn.nameSize, c: Surface.muted))),
        ],
      ]);

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        final wide = box.maxWidth >= 520;
        final side = SizedBox(width: wide ? 250 : double.infinity, child: SingleChildScrollView(child: _side()));
        final graph = ColoredBox(color: Surface.well, child: _graph());
        return DockedPanel(
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Container(
              height: 25.5,
              padding: const EdgeInsets.symmetric(horizontal: 9),
              decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: Surface.dividerFine))),
              child: Row(children: [
                Text(draft != null ? 'CHOOSE THE THINGS IT DRIVES' : (picking != null ? 'EDIT THE MEMBERS' : 'RELATIONS'), style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w600, ls: 1.3)),
                const Spacer(),
                Text('${present.length} things now', style: mono(Dn.labelSize, c: Surface.muted)),
              ]),
            ),
            Expanded(child: wide ? Row(children: [Expanded(child: graph), Container(width: 1, color: Surface.dividerFine), side]) : Column(children: [Expanded(child: graph), Container(height: 1, color: Surface.dividerFine), SizedBox(height: 165, child: side)])),
          ]),
        );
      });
}

class _End extends StatefulWidget {
  const _End(this.keyName, this.value, this.suffix, this.onChange, this.onDone);
  final String keyName, suffix;
  final double value;
  final ValueChanged<double> onChange;
  final VoidCallback? onDone;
  @override
  State<_End> createState() => _EndState();
}

class _EndState extends State<_End> {
  double? start;
  double x0 = 0;
  @override
  Widget build(BuildContext context) => Listener(
        onPointerDown: (e) { start = widget.value; x0 = e.position.dx; },
        onPointerMove: (e) { if (start != null) widget.onChange((start! + (e.position.dx - x0) * (widget.suffix == 'px' ? 2 : .5) * 1).roundToDouble()); },
        onPointerUp: (_) { start = null; widget.onDone?.call(); },
        child: Container(
          key: ValueKey(widget.keyName),
          padding: const EdgeInsets.symmetric(horizontal: 4.5, vertical: 2),
          decoration: BoxDecoration(color: Surface.well, borderRadius: BorderRadius.circular(2)),
          child: Text('${widget.value.round()}${widget.suffix}', style: mono(Dn.nameSize, c: Surface.ink)),
        ),
      );
}

class _RangePainter extends CustomPainter {
  _RangePainter(this.at);
  final double? at;
  @override
  void paint(Canvas c, Size s) {
    final y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = N.g26..strokeWidth = 2);
    for (final x in [0.0, s.width]) c.drawLine(Offset(x, y - 5), Offset(x, y + 5), Paint()..color = Surface.muted..strokeWidth = 2);
    if (at != null) c.drawCircle(Offset(at! * s.width, y), 4, Paint()..color = kRed);
  }
  @override
  bool shouldRepaint(_RangePainter o) => o.at != at;
}

class _GraphPainter extends CustomPainter {
  _GraphPainter({required this.things, required this.dots, required this.source, required this.members, required this.lasso, required this.picking, required this.mappings});
  final List<Map<String, dynamic>> things;
  final Map<int, Offset> dots;
  final int? source;
  final Set<int> members;
  final List<Offset>? lasso;
  final bool picking;
  final int mappings;

  @override
  void paint(Canvas c, Size s) {
    final grid = Paint()..color = N.g10;
    for (var x = 0.0; x < s.width; x += 32) c.drawLine(Offset(x, 0), Offset(x, s.height), grid);
    for (var y = 0.0; y < s.height; y += 32) c.drawLine(Offset(0, y), Offset(s.width, y), grid);
    // the bundle: source to the member set, one line, the count of mappings written on it
    if (source != null && members.isNotEmpty && dots.containsKey(source)) {
      final from = dots[source]!;
      var cx = 0.0, cy = 0.0;
      for (final m in members) { final p = dots[m] ?? from; cx += p.dx; cy += p.dy; }
      final to = Offset(cx / members.length, cy / members.length);
      c.drawLine(from, to, Paint()..color = kRed.withValues(alpha: .6)..strokeWidth = 2);
      for (final m in members) {
        final p = dots[m]; if (p == null) continue;
        c.drawLine(to, p, Paint()..color = kRed.withValues(alpha: .25)..strokeWidth = 1);
      }
      if (mappings > 0) {
        final tp = TextPainter(text: TextSpan(text: '$mappings mapping${mappings == 1 ? '' : 's'}', style: sans(Dn.labelSize, c: kRed)), textDirection: TextDirection.ltr)..layout();
        final mid = Offset.lerp(from, to, .5)!;
        tp.paint(c, mid - Offset(tp.width / 2, tp.height + 4));
      }
    }
    for (final l in things) {
      final id = l['id'] as int; final p = dots[id]!;
      final isSrc = id == source, isMem = members.contains(id);
      final color = isSrc ? kRed : (isMem ? Surface.ink : N.g44);
      if (isSrc) c.drawCircle(p, 13, Paint()..color = kRed.withValues(alpha: .25));
      c.drawCircle(p, isSrc ? 7 : 6, Paint()..color = color);
      if (isMem && !isSrc) c.drawCircle(p, 10, Paint()..color = kRed..style = PaintingStyle.stroke..strokeWidth = 1.6);
      if (picking && !isSrc && !isMem) c.drawCircle(p, 10, Paint()..color = N.g26..style = PaintingStyle.stroke..strokeWidth = 1);
      final tp = TextPainter(text: TextSpan(text: '${l['name']}', style: sans(Dn.labelSize, c: isSrc || isMem ? Surface.ink : Surface.muted)), textDirection: TextDirection.ltr)..layout(maxWidth: 67.5);
      tp.paint(c, p + Offset(-tp.width / 2, 12));
    }
    if (lasso != null && lasso!.length > 1) {
      final path = Path()..moveTo(lasso![0].dx, lasso![0].dy);
      for (final p in lasso!.skip(1)) path.lineTo(p.dx, p.dy);
      c.drawPath(path, Paint()..color = kRed.withValues(alpha: .12));
      c.drawPath(path, Paint()..color = kRed..style = PaintingStyle.stroke..strokeWidth = 1.2);
    }
  }

  @override
  bool shouldRepaint(_GraphPainter o) => true;
}
