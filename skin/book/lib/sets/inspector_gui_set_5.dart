// Part of inspector_gui_set.dart (cut there by size, as it was).
part of 'inspector_gui_set.dart';

/// One canvas that shows the layer's box. [GMode.all] = U1 (five zones at once); the other modes = U2 (only that mode's handles are live and drawn).
Widget _gizmoHand(GDoc d, Size size, GMode mode, {ValueChanged<Object?>? onHot}) {
  final g = _PadGeo(size, d, boost: Gui.boost);
  final all = mode == GMode.all;
  Offset local(Offset p) => _rot(p - g.puck, -g.rad);
  bool inBody(Offset p) {
    final l = local(p);
    return l.dx.abs() <= g.half.dx && l.dy.abs() <= g.half.dy;
  }

  List<Offset> edges() {
    final h = g.half;
    return [for (final o in [Offset(-h.dx, 0), Offset(h.dx, 0), Offset(0, -h.dy), Offset(0, h.dy)]) g.puck + _rot(o, g.rad)];
  }

  Offset nine(int i) => g.puck + _rot(Offset(((i % 3) - 1) * g.half.dx, ((i ~/ 3) - 1) * g.half.dy), g.rad);
  Offset pivot() => g.puck + _rot(Offset((d['anchor.x'] * 2 - 1) * g.half.dx, (d['anchor.y'] * 2 - 1) * g.half.dy), g.rad);
  final ringR = g.half.distance + 20;
  final rotId = _rotSpec(d).id;
  return _Hand(
    d: d,
    size: size,
    onHot: onHot,
    pick: (p) {
      if (mode == GMode.anchor) {
        for (var i = 0; i < 9; i++) {
          if ((p - nine(i)).distance < 11) {
            return 'a$i';
          }
        }
        return null;
      }
      if (all && (p - pivot()).distance < 8) {
        return 'pivot';
      }
      if (all || mode == GMode.scale) {
        final cs = g.corners(g.puck, g.half), es = edges();
        for (var i = 0; i < 4; i++) {
          if ((p - cs[i]).distance < 10) {
            return i;
          }
        }
        for (var i = 0; i < 4; i++) {
          if ((p - es[i]).distance < 9) {
            return 4 + i;
          }
        }
      }
      if (all && !inBody(p)) {
        for (final c in g.corners(g.puck, g.half)) {
          final dist = (p - c).distance;
          if (dist >= 10 && dist <= Gui.ringBand) {
            return 'rot'; // the ring just outside a corner
          }
        }
      }
      if (mode == GMode.rotate && ((p - g.puck).distance - ringR).abs() < 12) {
        return 'rot';
      }
      if ((all && inBody(p)) || mode == GMode.move) {
        return 'move';
      }
      return null;
    },
    start: (grab, p, fine) {
      if (mode == GMode.move && grab == 'move' && !fine && !inBody(p)) {
        d.set('pos.x', (p.dx - g.c.dx) * g.k);
        d.set('pos.y', (p.dy - g.c.dy) * g.k);
      }
    },
    tap: (grab, p) {
      if (grab is String && grab.startsWith('a')) {
        final i = int.parse(grab.substring(1));
        d.set('anchor.x', (i % 3) / 2);
        d.set('anchor.y', (i ~/ 3) / 2);
      }
    },
    drag: (grab, p, last, fine) {
      final k = fine ? Gui.fine : 1.0;
      if (grab == 'move') {
        final dd = (p - last) * g.k * k;
        d.add('pos.x', dd.dx);
        d.add('pos.y', dd.dy);
      } else if (grab == 'pivot') {
        final dl = local(p) - local(last);
        d.set('anchor.x', (d['anchor.x'] + dl.dx / (2 * g.half.dx) * k).clamp(0.0, 1.0));
        d.set('anchor.y', (d['anchor.y'] + dl.dy / (2 * g.half.dy) * k).clamp(0.0, 1.0));
      } else if (grab == 'rot') {
        d.add(rotId, _wrap((p - g.puck).direction - (last - g.puck).direction) * 180 / math.pi * k);
      } else if (grab is int) {
        final lp = local(p), ll = local(last);
        double ratio(double a, double b) => b.abs() < 2 ? 1.0 : (a / b).abs();
        var fx = grab < 4 || grab < 6 ? ratio(lp.dx, ll.dx) : 1.0, fy = grab < 4 || grab >= 6 ? ratio(lp.dy, ll.dy) : 1.0;
        if (d['link'] > .5) {
          fx = fy = grab < 4 ? lp.distance / math.max(2, ll.distance) : (grab < 6 ? fx : fy);
        }
        d.set('scale.x', d['scale.x'] * math.pow(fx, k));
        d.set('scale.y', d['scale.y'] * math.pow(fy, k));
      }
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final c = g.c;
      for (final f in const [1 / 3, 2 / 3]) {
        cv.drawLine(Offset(sz.width * f, 0), Offset(sz.width * f, sz.height), _ln(N.g15));
        cv.drawLine(Offset(0, sz.height * f), Offset(sz.width, sz.height * f), _ln(N.g15));
      }
      cv.drawLine(Offset(c.dx, 0), Offset(c.dx, sz.height), _ln(N.g20));
      cv.drawLine(Offset(0, c.dy), Offset(sz.width, c.dy), _ln(N.g20));
      cv.drawRect(Rect.fromCenter(center: c, width: Gui.comp.width / g.k, height: Gui.comp.height / g.k), _ln(N.g26));
      final pk = g.puck, cs = g.corners(pk, g.half), col = _stroke(d, hot, live), body = Path()..addPolygon(cs, true);
      cv.drawPath(body, _fl(d.locked ? N.g13 : (hot == 'move' || live && hot == 'move' ? N.g20 : N.g15)));
      cv.drawPath(body, _ln(col));
      final top = (cs[0] + cs[1]) / 2;
      cv.drawLine(top, top + _rot(const Offset(0, -1), g.rad) * 5, _ln(col));
      void sq(Offset o, double r, Object key) {
        final on = hot == key, rr = Rect.fromCenter(center: o, width: r, height: r);
        cv.drawRect(rr, _fl(on ? N.g95 : N.g07));
        cv.drawRect(rr, _ln(d.locked ? Role.disabled : (on ? C.mode : N.g76)));
      }

      if (all || mode == GMode.scale) {
        final es = edges();
        for (var i = 0; i < 4; i++) {
          sq(cs[i], 7, i);
          sq(es[i], 5, 4 + i);
        }
      }
      if (all && hot == 'rot') {
        for (final cn in cs) {
          cv.drawCircle(cn, 18, _ln(Role.selected.withValues(alpha: .5))); // the ring zone shows on hover only
        }
      }
      if (mode == GMode.rotate) {
        cv.drawCircle(pk, ringR, _ln(d.locked ? N.g38 : N.g38));
        final a = -math.pi / 2 + d[rotId] * math.pi / 180, tip = pk + Offset(math.cos(a), math.sin(a)) * ringR;
        cv.drawLine(pk, tip, _ln(N.g44));
        cv.drawCircle(tip, 4, _fl(N.g95));
        cv.drawCircle(tip, 4, _ln(col));
      }
      if (mode == GMode.anchor) {
        final sel = _anchorIndex(d);
        for (var i = 0; i < 9; i++) {
          final on = sel == i, hv = hot == 'a$i';
          cv.drawCircle(nine(i), on || hv ? 4 : Gui.dot, on ? _fl(d.locked ? Role.disabled : N.g95) : _ln(hv ? N.g95 : N.g56));
          if (on && !d.locked) {
            cv.drawCircle(nine(i), 6, _ln(C.mode));
          }
        }
        if (sel == null) {
          cv.drawCircle(pivot(), 4, _ln(N.g95));
        }
      }
      if (all) {
        final pv = pivot();
        cv.drawCircle(pv, 3, _fl(d.locked ? Role.disabled : N.g95));
        cv.drawCircle(pv, 5, _ln(hot == 'pivot' ? C.mode : N.g44));
      }
    },
  );
}

String _vals(GDoc d, List<Spec> specs) => [
      for (final s in specs) '${s.tag} ${d.mixed.contains(s.id) ? '—' : _fmt(d[s.id] * s.scale, s.dec)}${s.unit.isEmpty ? '' : ' ${s.unit}'}',
    ].join('   ');

/// U1. Unified object gizmo: move, scale, rotate and anchor on one canvas; the readout names the zone under the pointer.
class TransformUnified extends StatefulWidget {
  const TransformUnified(this.d, {super.key});
  final GDoc d;
  @override
  State<TransformUnified> createState() => _TransformUnifiedState();
}

class _TransformUnifiedState extends State<TransformUnified> {
  Object? hot;
  @override
  Widget build(BuildContext context) {
    final d = widget.d, three = d['space'] > 0;
    final w = Gui.content - (three ? Gui.rail + 6 : 0);
    final zone = _zoneName(hot);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        _gizmoHand(d, Size(w, Gui.gizmoH), GMode.all, onHot: (h) {
          if (mounted && h != hot) {
            setState(() => hot = h);
          }
        }),
        if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.gizmoH))],
      ]),
      _gap(6),
      SizedBox(
        height: Gui.field,
        child: Row(children: [
          Expanded(
            child: FittedBox(
              fit: BoxFit.scaleDown,
              alignment: Alignment.centerLeft,
              child: Text(zone.isNotEmpty ? zone : _vals(d, [..._posSpecs(d).take(2), _rotSpec(d), ..._scaleSpecs]), maxLines: 1, style: T.value(N.g76)),
            ),
          ),
          _linkChip(d),
        ]),
      ),
    ]);
  }
}

/// U2. Mode-switch single canvas: Move / Scale / Rotate / Anchor (keys 1-4); the canvas and the numbers belong to the active mode only.
class TransformModes extends StatefulWidget {
  const TransformModes(this.d, {super.key});
  final GDoc d;
  @override
  State<TransformModes> createState() => _TransformModesState();
}

class _TransformModesState extends State<TransformModes> {
  GMode mode = GMode.move;
  final focus = FocusNode(debugLabel: 'modes');
  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  void _set(int i) => setState(() => mode = const [GMode.move, GMode.scale, GMode.rotate, GMode.anchor][i]);

  @override
  Widget build(BuildContext context) {
    final d = widget.d, i = mode.index - 1;
    final specs = switch (mode) {
      GMode.move => _posSpecs(d),
      GMode.scale => _scaleSpecs,
      GMode.rotate => [_rotSpec(d)],
      _ => _anchorSpecs,
    };
    return Focus(
      focusNode: focus,
      onKeyEvent: (_, e) {
        final k = {LogicalKeyboardKey.digit1: 0, LogicalKeyboardKey.digit2: 1, LogicalKeyboardKey.digit3: 2, LogicalKeyboardKey.digit4: 3}[e.logicalKey];
        if (e is KeyDownEvent && k != null) {
          _set(k);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _SpaceChips(d),
        _gap(12),
        Segmented(items: const ['Move', 'Scale', 'Rotate', 'Anchor'], index: i, expand: true, onChanged: (n) {
          focus.requestFocus();
          _set(n);
        }),
        _gap(8),
        _gizmoHand(d, const Size(Gui.content, Gui.gizmoH), mode),
        _gap(8),
        XyzStack(d, specs, gap: Gui.rowGap),
        if (mode == GMode.scale) ...[_gap(4), Align(alignment: Alignment.centerLeft, child: _linkChip(d))],
        if (mode == GMode.rotate && d['space'] > 0) ...[_gap(4), _axisChips(d)],
      ]),
    );
  }
}

/// U3. Scrub-first: the Blender-style stacks are the only controls; a small preview of the resulting box has no handles.
class TransformScrub extends StatelessWidget {
  const TransformScrub(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _title('Preview'),
        _gap(6),
        IgnorePointer(
          child: SizedBox(
            height: Gui.previewH,
            child: CustomPaint(painter: _Fn((cv, sz) {
              final g = _PadGeo(sz, d);
              _well(cv, sz);
              cv.drawRect(Rect.fromCenter(center: g.c, width: Gui.comp.width / g.k, height: Gui.comp.height / g.k), _ln(N.g26));
              final pk = g.puck, cs = g.corners(pk, g.half);
              cv.drawPath(Path()..addPolygon(cs, true), _fl(N.g15));
              cv.drawPath(Path()..addPolygon(cs, true), _ln(N.g63));
              cv.drawCircle(pk + _rot(Offset((d['anchor.x'] * 2 - 1) * g.half.dx, (d['anchor.y'] * 2 - 1) * g.half.dy), g.rad), 2.5, _fl(N.g95));
            })),
          ),
        ),
        _gap(12),
        TransformFields(d),
      ]);
}

/// U4. Summon on focus: numbers only at rest; hovering or focusing a row opens its instrument inline; Esc or leaving closes it.
class _SummonRow extends StatefulWidget {
  const _SummonRow(this.d, this.title, this.specs, this.instrument);
  final GDoc d;
  final String title;
  final List<Spec> specs;
  final Widget instrument;
  @override
  State<_SummonRow> createState() => _SummonRowState();
}

class _SummonRowState extends State<_SummonRow> {
  bool hover = false, focused = false, dismissed = false;
  @override
  Widget build(BuildContext context) {
    final d = widget.d, open = (hover || focused || d.live) && !dismissed && !d.locked;
    return Focus(
      onFocusChange: (f) => setState(() {
        focused = f;
        if (!f) {
          dismissed = false;
        }
      }),
      onKeyEvent: (_, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && open) {
          setState(() => dismissed = true); // Esc closes the summoned instrument
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: MouseRegion(
        onEnter: (_) => setState(() => hover = true),
        onExit: (_) => setState(() {
          hover = false;
          dismissed = false;
        }),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(
            height: Gui.field,
            child: Row(children: [
              SizedBox(width: 54, child: Text(widget.title, style: T.label(open ? N.g95 : N.g63))),
              for (final (i, s) in widget.specs.indexed) ...[if (i > 0) const SizedBox(width: Gui.rowGap), Expanded(child: XyzStack(d, [s]))],
            ]),
          ),
          AnimatedSize(
            duration: const Duration(milliseconds: 120),
            curve: Curves.easeOut,
            alignment: Alignment.topCenter,
            child: open ? Padding(padding: const EdgeInsets.only(top: 6, bottom: 2), child: widget.instrument) : const SizedBox(width: double.infinity),
          ),
        ]),
      ),
    );
  }
}

class TransformSummon extends StatelessWidget {
  const TransformSummon(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    final three = d['space'] > 0;
    final padW = Gui.content - (three ? Gui.rail + 6 : 0);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      _SummonRow(d, 'Position', _posSpecs(d), Row(children: [_padHand(d, Size(padW, Gui.padH)), if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.padH))]])),
      _gap(4),
      _SummonRow(d, 'Rotation', [_rotSpec(d)], Row(children: [_dialHand(d, const Size(Gui.cell, Gui.cell)), const SizedBox(width: 8), if (three) _axisChips(d)])),
      _gap(4),
      _SummonRow(d, 'Scale', _scaleSpecs, Row(children: [_scaleHand(d, const Size(Gui.cell, Gui.cell)), const SizedBox(width: 8), _linkChip(d)])),
      _gap(4),
      _SummonRow(d, 'Anchor', _anchorSpecs, Align(alignment: Alignment.centerLeft, child: _anchorHand(d, Gui.anchorBox))),
    ]);
  }
}

// ---- hosting: the document, the knobs, the inspector column ---------------------------------------------------------------------

class _Host extends StatefulWidget {
  const _Host({required this.space, required this.locked, required this.mixed, required this.tone, required this.build, this.layer = false, this.weight = 1});
  final int space;
  final bool locked, mixed, layer;
  final int weight;
  final AxisTone tone;
  final Widget Function(BuildContext, GDoc) build;
  @override
  State<_Host> createState() => _HostState();
}

class _HostState extends State<_Host> {
  final doc = GDoc();

  void _apply({bool spaceToo = true}) {
    if (spaceToo) {
      doc.v['space'] = widget.space.toDouble();
    }
    doc
      ..locked = widget.locked
      ..tone = widget.tone
      ..weight = widget.weight
      ..v['cam.layer'] = widget.layer ? 1 : 0;
    doc.mixed
      ..clear()
      ..addAll(widget.mixed ? const ['pos.x', 'scale.x', 'rot.z'] : const []);
  }

  @override
  void initState() {
    super.initState();
    _apply();
  }

  @override
  void didUpdateWidget(_Host old) {
    super.didUpdateWidget(old);
    _apply(spaceToo: old.space != widget.space);
  }

  @override
  void dispose() {
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(listenable: doc, builder: (context, _) => widget.build(context, doc));
}

Widget _hosted(BuildContext c, Widget Function(BuildContext, GDoc) build, {bool layerKnob = false, bool weightKnob = false}) => _Host(
      weight: weightKnob ? c.knobs.object.dropdown<int>(label: 'Number weight (graphic leads / balanced / numbers lead)', options: const [0, 1, 2], initialOption: 0, labelBuilder: (i) => const ['0  graphic leads', '1  balanced', '2  numbers lead'][i]) : 1,
      space: c.knobs.object.dropdown<int>(label: 'Space', options: const [0, 1, 2], initialOption: 1, labelBuilder: (i) => const ['2D', '2.5D', '3D'][i]),
      locked: c.knobs.boolean(label: 'Locked layer'),
      mixed: c.knobs.boolean(label: 'Mixed (3 layers selected)'),
      tone: c.knobs.object.dropdown<AxisTone>(label: 'Axis colour (decision B)', options: AxisTone.values, initialOption: AxisTone.fam, labelBuilder: (t) => switch (t) {
            AxisTone.fam => '1  Fam hues (now)',
            AxisTone.rgb => '2  X red / Y green / Z blue',
            AxisTone.none => '3  none, letters only',
          }),
      layer: layerKnob && c.knobs.boolean(label: 'Target is a layer'),
      build: (ctx, d) => SingleChildScrollView(child: build(ctx, d)), // the whole story scrolls vertically: nothing can overflow the viewport
    );

Widget _col(Widget child, {String? head}) => Container(
      width: Gui.content + 26, // 12 + 256 + 12 gutters, plus the 1 px hairline each side
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        if (head != null) ...[Text(head, style: T.title()), _gap(12)],
        child,
      ]),
    );

/// The long explanation lives in the Widgetbook story chrome, beside the 282 px frame, never inside it.
Widget _help(GDoc d, String text) => SizedBox(
      width: 320,
      child: Text('Undo steps: ${d.steps} (one per gesture; Esc leaves none) · last scrub applied to: ${d.scope}\n$_rules\n\n$text', style: T.label(N.g63).copyWith(height: 1.4)),
    );

Widget _story(Widget frame, GDoc d, String text) => Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [frame, const SizedBox(width: 20), _help(d, text)]);

const _rules = 'Shift = fine · Esc during a drag = revert · locked layer = inert';
