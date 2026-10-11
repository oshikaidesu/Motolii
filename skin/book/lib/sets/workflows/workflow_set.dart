// set: workflow. Round 1 of research/workflows.md: W1 (select and inspect) and W2 (edit a value, number <-> graphic) as ONE hands-on use case.
// Behaviour only: nothing here is a new look. Rows, boxes, fields and the gizmo are the lab's own parts; the shared state is lib/session.dart.
// Prototype for the Widgetbook, not Motolii's architecture.
import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../foundations/inputs_parts.dart' show ScrubField;
import '../inspector/inspector_parts.dart' show KeyMark, KeyS;
import 'workflow_timeline.dart';
import '../../session.dart';
import '../../tokens.dart';
import '../inspector_gui_set.dart';

WidgetbookComponent workflowSet() => WidgetbookComponent(name: 'Workflow', useCases: [
      WidgetbookUseCase(name: 'W1+W2 Select, inspect, edit', builder: (c) => const ColoredBox(color: N.g07, child: _Scene(TlKnobs(), seedKeys: false))),
      // the same window, with the open timeline questions (snap, zoom, pan, grab sizes) as knobs in the right panel
      WidgetbookUseCase(name: 'W4 Timeline: grab, snap, zoom (knobs)', builder: (c) => ColoredBox(color: N.g07, child: _Scene(TlKnobs.read(c), seedKeys: true))),
      // B1: the behaviour shelf in the same window. Pick Wave from the shelf (drag it onto a layer, or click with one selected) -> it sways that layer
      // on the Stage while it plays -> its numbers are read and edited in the Inspector. One round trip, nothing else.
      WidgetbookUseCase(name: 'B1 Shelf -> attach -> Inspector (Wave)', builder: (c) => const ColoredBox(color: N.g07, child: _Scene(TlKnobs(), seedKeys: false, shelf: true))),
    ]);

const _listW = 168.0, _inspW = 282.0, _stagePad = 12.0, _footH = 22.0;

bool _addKey() {
  final k = HardwareKeyboard.instance;
  return k.isShiftPressed || k.isMetaPressed || k.isControlPressed;
}

class _Scene extends StatefulWidget {
  const _Scene(this.knobs, {required this.seedKeys, this.shelf = false});
  final TlKnobs knobs;
  final bool seedKeys; // the W4 case opens with keys on three layers
  final bool shelf; // B1: the behaviour shelf under the list
  @override
  State<_Scene> createState() => _SceneState();
}

class _SceneState extends State<_Scene> {
  late final session = Session(seedKeys: widget.seedKeys);
  final focus = FocusNode(debugLabel: 'workflow');

  @override
  void dispose() {
    focus.dispose();
    session.dispose();
    super.dispose();
  }

  KeyEventResult _key(FocusNode _, KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
    // a number being typed keeps its keys (the field handles its own Esc)
    if (FocusManager.instance.primaryFocus?.context?.findAncestorWidgetOfExactType<EditableText>() != null) return KeyEventResult.ignored;
    final k = e.logicalKey, hk = HardwareKeyboard.instance;
    if (k == LogicalKeyboardKey.escape) {
      if (session.cancelMove()) return KeyEventResult.handled;
      if (session.selected.isEmpty) return KeyEventResult.ignored;
      session.clear();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.space) {
      session.toggle();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) {
      session.deleteKey();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.home) {
      session.seek(0);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.tab) {
      session.step(hk.isShiftPressed ? -1 : 1);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowUp) {
      session.step(k == LogicalKeyboardKey.arrowDown ? 1 : -1);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) => SessionScope(
        session: session,
        // above the root Focus, so it sees the key whichever descendant (the root, a gizmo, a field) holds focus
        child: CallbackShortcuts(
          bindings: {
            const SingleActivator(LogicalKeyboardKey.keyZ, meta: true): session.undo,
            const SingleActivator(LogicalKeyboardKey.keyZ, control: true): session.undo,
          },
          child: Focus(
          focusNode: focus,
          autofocus: true,
          onKeyEvent: _key,
          child: Column(children: [
            Expanded(
              child: Row(children: [
                SizedBox(
                  width: _listW,
                  child: widget.shelf
                      ? Column(children: [Expanded(child: _LayerList(focus)), Container(height: 1, color: N.g20), const _Shelf()])
                      : _LayerList(focus),
                ),
                Container(width: 1, color: N.g20),
                Expanded(child: WorkStage(focus, onShelfDrop: widget.shelf ? (what, l) => (l ?? session.primary) == null ? null : session.attachWave((l ?? session.primary)!) : null)),
                Container(width: 1, color: N.g20),
                const SizedBox(width: _inspW, child: _Inspector()),
              ]),
            ),
            Container(height: 1, color: N.g20),
            SizedBox(height: TL.ruler + TL.height * 6 + 24, child: WorkTimeline(focus: focus, knobs: widget.knobs)),
          ]),
        )),
      );
}

// ---- the list ---------------------------------------------------------------------------------------------------------------------

class _LayerList extends StatelessWidget {
  const _LayerList(this.focus);
  final FocusNode focus;
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context);
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTapDown: (_) {
        focus.requestFocus();
        s.clear();
      },
      child: Container(
        color: N.g10,
        padding: const EdgeInsets.fromLTRB(0, 12, 0, 0),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Padding(padding: const EdgeInsets.fromLTRB(12, 0, 12, 8), child: Text('LAYERS', style: T.micro(N.g56).copyWith(letterSpacing: .8))),
          for (final l in s.layers) _ListRow(l, focus),
        ]),
      ),
    );
  }
}

class _ListRow extends StatelessWidget {
  const _ListRow(this.l, this.focus);
  final LayerM l;
  final FocusNode focus;
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context), sel = s.isSelected(l.id), hov = s.hovered == l.id, ring = s.keyRing && s.primary == l;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => s.hover(l.id),
      onExit: (_) => s.hover(null),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: (_) {
          focus.requestFocus();
          s.select(l.id, add: _addKey());
        },
        child: Stack(children: [
          AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: TL.height,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            color: sel ? N.g20 : (hov ? N.g15 : const Color(0x00262626)),
            child: Row(children: [
              Container(width: 8, height: 8, decoration: BoxDecoration(color: l.fam.c, borderRadius: BorderRadius.circular(2))),
              const SizedBox(width: 6),
              Expanded(child: Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(sel ? N.g100 : N.g91))),
              if (l.wave != null) const WaveGlyph(),
            ]),
          ),
          Positioned(left: 0, top: 0, bottom: 0, width: 2, child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, color: sel ? N.g95 : const Color(0x00F2F2F2))),
          if (ring) Positioned.fill(child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: Role.selected, width: 1))))),
        ]),
      ),
    );
  }
}

// ---- the stage ----------------------------------------------------------------------------------------------------------------------

bool _inBox(List<Offset> c, Offset p, double margin) {
  final o = (c[0] + c[2]) / 2, u = c[1] - c[0], v = c[3] - c[0], d = p - o;
  final ux = u / math.max(u.distance, 1), vx = v / math.max(v.distance, 1);
  return (d.dx * ux.dx + d.dy * ux.dy).abs() <= u.distance / 2 + margin && (d.dx * vx.dx + d.dy * vx.dy).abs() <= v.distance / 2 + margin;
}

/// The stage as a drop target for the shelf: the layer under the pointer lights as it would when hovered, and the drop attaches to it.
Widget _shelfTarget(LayerM? Function(Offset p, {double margin}) hit, void Function(String, LayerM?)? onDrop, Session s, Widget child) {
  if (onDrop == null) return child;
  return Builder(
    builder: (ctx) {
      Offset local(Offset g) => (ctx.findRenderObject()! as RenderBox).globalToLocal(g);
      return DragTarget<String>(
        onMove: (d) => s.hover(hit(local(d.offset), margin: 2)?.id),
        onLeave: (_) => s.hover(null),
        onAcceptWithDetails: (d) {
          final l = hit(local(d.offset), margin: 2);
          s.hover(null);
          onDrop(d.data, l);
        },
        builder: (_, _, _) => child,
      );
    },
  );
}

class WorkStage extends StatelessWidget {
  const WorkStage(this.focus, {super.key, this.wrap, this.above, this.onShelfDrop});
  final FocusNode focus;

  /// A behaviour dragged from the shelf and dropped here: what it is, and the layer under the pointer (null = none).
  final void Function(String what, LayerM? layer)? onShelfDrop;

  /// Hooks for a use case built on this stage (sets/run_r1.dart): wrap the canvas (a drop target), and add painted layers above the boxes.
  final Widget Function(Size size, double k, Widget canvas)? wrap;
  final List<Widget> Function(Size size, double k, Map<String, List<Offset>> corners)? above;

  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context);
    return Container(
      color: N.g10,
      padding: const EdgeInsets.all(_stagePad),
      child: LayoutBuilder(builder: (context, box) {
        final size = Size(math.max(120, box.maxWidth), math.max(120, box.maxHeight - _footH - 6)), k = Gui.span / size.width;
        // a layer is drawn where its wave puts it at the playhead (the gizmo and the hit test follow)
        Offset sh(LayerM l) => s.waveShift(l) / k;
        final corners = {for (final l in s.layers) l.id: [for (final c in gizmoCorners(l.doc, size)) c + sh(l)]};
        final prim = s.primary;
        // topmost layer under the pointer (later layers are above), or null
        LayerM? hit(Offset p, {double margin = 2}) {
          for (final l in s.layers.reversed) {
            if (_inBox(corners[l.id]!, p, margin)) return l;
          }
          return null;
        }

        // the part of the selected layer's box that belongs to the gizmo (body, handles, the rotate ring at the corners)
        bool claimed(Offset p) {
          if (prim == null) return false;
          final c = corners[prim.id]!;
          return _inBox(c, p, 12) || c.any((q) => (q - p).distance <= Gui.ringBand);
        }

        return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(
            height: size.height,
            child: (wrap ?? (_, _, w) => w)(size, k, _shelfTarget(hit, onShelfDrop, s, MouseRegion(
              onHover: (e) => s.hover(hit(e.localPosition)?.id),
              onExit: (_) => s.hover(null),
              child: Listener(
                behavior: HitTestBehavior.translucent,
                onPointerDown: (e) {
                  if (claimed(e.localPosition)) return; // the gizmo has it
                  focus.requestFocus();
                  final l = hit(e.localPosition);
                  if (l == null) {
                    s.clear();
                  } else if (_addKey()) {
                    s.select(l.id, add: true);
                  } else {
                    s.beginMove(l, e.localPosition);
                  }
                },
                onPointerMove: (e) => s.moving ? s.moveTo(e.localPosition, k, HardwareKeyboard.instance.isShiftPressed) : null,
                onPointerUp: (_) => s.endMove(),
                onPointerCancel: (_) => s.cancelMove(),
                child: Stack(children: [
                  Positioned.fill(
                    child: prim == null
                        ? CustomPaint(painter: _Ground(size))
                        : Transform.translate(offset: sh(prim), child: KeyedSubtree(key: ValueKey(prim.id), child: unifiedGizmo(prim.doc, size, onHot: (h) => _zone.value = gizmoZoneName(h)))),
                  ),
                  for (final l in s.layers)
                    Positioned.fill(
                      child: IgnorePointer(
                        child: TweenAnimationBuilder<double>(
                          tween: Tween(end: s.isSelected(l.id) ? 1.0 : (s.hovered == l.id ? .5 : 0.0)),
                          duration: Mo.dur,
                          curve: Mo.ease,
                          builder: (_, level, _) => CustomPaint(painter: _BoxPaint(l, corners[l.id]!, level, primary: l == prim, ring: s.keyRing && l == prim)),
                        ),
                      ),
                    ),
                  if (s.selected.length > 1) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _UnionPaint([for (final id in s.selected) ...corners[id]!])))),
                  if (prim != null && prim.pk.isNotEmpty) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _KeysPaint(prim, size, s.frame, {for (final f in prim.keyFrames) if (s.keyOn(prim, f)) f})))),
                  ...?above?.call(size, k, corners),
                ]),
              ),
            ))),
          ),
          const SizedBox(height: 6),
          SizedBox(height: _footH, child: Row(children: [Expanded(child: _Readout(_zone)), const _Transport()])),
        ]);
      }),
    );
  }
}

String _time(int f) => '${(f ~/ Session.fps).toString().padLeft(2, '0')}:${(f % Session.fps).toString().padLeft(2, '0')}';

/// Play / pause and the playhead's time. Space does the same from the keyboard; Home goes to the start.
class _Transport extends StatelessWidget {
  const _Transport();
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context);
    return Row(children: [
      Text(_time(s.frame), style: T.value(N.g95)),
      const SizedBox(width: 10),
      MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: s.toggle, child: Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Text(s.playing ? 'Pause' : 'Play', style: T.name(N.g95)))),
      ),
    ]);
  }
}

/// What the gizmo says it is hovering. Chrome state of this panel only, so it lives here and not in the session.
final _zone = ValueNotifier<String>('');

class _Readout extends StatelessWidget {
  const _Readout(this.zone);
  final ValueNotifier<String> zone;
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context), p = s.primary;
    return ValueListenableBuilder<String>(
      valueListenable: zone,
      builder: (_, z, _) {
        String two(double v) => v.toStringAsFixed(1);
        final text = s.note != null
            ? s.note!
            : p == null
            ? (s.hovered == null ? 'Click a box to select it' : s.byId(s.hovered!).name)
            : s.selected.length > 1
            ? '${s.selected.length} layers selected'
            : (z.isNotEmpty && s.hovered == p.id ? z : '${p.name}   X ${two(p.doc['pos.x'])}   Y ${two(p.doc['pos.y'])}   ${two(p.doc['rot.z'])}°   ${two(p.doc['scale.x'] * 100)}%');
        return Align(alignment: Alignment.centerLeft, child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.value(N.g76)));
      },
    );
  }
}

/// The empty stage: the same ground the gizmo draws (well, thirds, centre cross, composition frame), for when nothing is selected.
class _Ground extends CustomPainter {
  const _Ground(this.size);
  final Size size;
  @override
  void paint(Canvas cv, Size sz) {
    final ln = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    final r = RRect.fromRectAndRadius(Offset.zero & sz, const Radius.circular(Gui.radius));
    cv.drawRRect(r, Paint()..color = N.g07);
    cv.drawRRect(r.deflate(.5), ln..color = N.g20);
    for (final f in const [1 / 3, 2 / 3]) {
      cv.drawLine(Offset(sz.width * f, 0), Offset(sz.width * f, sz.height), ln..color = N.g15);
      cv.drawLine(Offset(0, sz.height * f), Offset(sz.width, sz.height * f), ln..color = N.g15);
    }
    final c = sz.center(Offset.zero), k = Gui.span / sz.width;
    cv.drawLine(Offset(c.dx, 0), Offset(c.dx, sz.height), ln..color = N.g20);
    cv.drawLine(Offset(0, c.dy), Offset(sz.width, c.dy), ln..color = N.g20);
    cv.drawRect(Rect.fromCenter(center: c, width: Gui.comp.width / k, height: Gui.comp.height / k), ln..color = N.g26);
  }

  @override
  bool shouldRepaint(_Ground o) => o.size != size;
}

/// A layer's box on the stage. [level] 0 at rest, .5 hovered, 1 selected (eased by the caller). The primary's body is drawn by the gizmo itself;
/// this adds only the state: family tint, hover / selection outline, the name, and the keyboard focus ring.
class _BoxPaint extends CustomPainter {
  const _BoxPaint(this.l, this.c, this.level, {required this.primary, required this.ring});
  final LayerM l;
  final List<Offset> c;
  final double level;
  final bool primary, ring;

  @override
  void paint(Canvas cv, Size sz) {
    final path = Path()..addPolygon(c, true);
    // selected is clearly stronger than hover: hover tint .12 -> .16 with a soft grey line, selected tint .34 with a 1.5 px accent line
    cv.drawPath(path, Paint()..color = l.fam.c.withValues(alpha: level <= .5 ? .12 + .08 * level : .16 + .36 * (level - .5)));
    final line = level <= .5 ? Color.lerp(l.fam.c.withValues(alpha: .55), N.g76, level * 2)! : Color.lerp(N.g76, C.mode, (level - .5) * 2)!;
    cv.drawPath(path, Paint()..style = PaintingStyle.stroke..strokeWidth = 1 + .5 * math.max(0, (level - .5) * 2)..color = line);
    if (ring) {
      // a 1.5 px accent line 4 px outside the box, so it never merges with the box's own edge
      final m = c.fold(Offset.zero, (a, p) => a + p) / 4, out = [for (final p in c) p + (p - m) / math.max((p - m).distance, 1) * 5];
      cv.drawPath(Path()..addPolygon(out, true), Paint()..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeJoin = StrokeJoin.round..color = Role.selected);
    }
    if (level > 0) {
      final tp = TextPainter(text: TextSpan(text: l.name, style: T.label(N.g91)), textDirection: TextDirection.ltr, maxLines: 1)..layout();
      final top = c.map((p) => p.dy).reduce(math.min), left = c.map((p) => p.dx).reduce(math.min);
      final at = Offset(left.clamp(4.0, math.max(4.0, sz.width - tp.width - 4)), (top - tp.height - 8).clamp(4.0, sz.height - tp.height - 4));
      cv.drawRRect(RRect.fromRectAndRadius((at & tp.size).inflate(3), const Radius.circular(3)), Paint()..color = N.g10.withValues(alpha: .92)); // a plate, so a neighbour's edge never runs through the name
      tp.paint(cv, at);
      tp.dispose();
    }
  }

  @override
  bool shouldRepaint(_BoxPaint o) => true;
}

// ---- the inspector --------------------------------------------------------------------------------------------------------------------

class _Inspector extends StatelessWidget {
  const _Inspector();
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context), p = s.primary, n = s.selected.length;
    final (kicker, title) = n == 0 ? ('NOTHING SELECTED', '') : (n == 1 ? ('LAYER', p!.name) : ('SEVERAL SELECTED', '$n layers'));
    return Container(
      color: N.g10,
      padding: const EdgeInsets.all(12),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        // pinned: the header and, for one layer, the position key; only the fields below scroll
        Text(kicker, style: T.micro(N.g56).copyWith(letterSpacing: .8)),
        if (title.isNotEmpty) ...[const SizedBox(height: 4), Text(title, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())],
        const SizedBox(height: 10),
        Container(height: 1, color: N.g15),
        if (n == 1) ...[
          const SizedBox(height: 8),
          Row(children: [
            Text('POSITION KEY', style: T.micro(N.g56).copyWith(letterSpacing: .8)),
            const Spacer(),
            Text(_time(s.frame), style: T.value(N.g76)),
            const SizedBox(width: 6),
            KeyMark(state: p!.pk.isEmpty ? KeyS.off : (s.keyAtHead ? KeyS.at : KeyS.anim), onTap: s.toggleKey),
          ]),
        ],
        const SizedBox(height: 10),
        Expanded(
          child: SingleChildScrollView(
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              if (n == 0)
                Text('Pick a layer on the Stage, in the list or on the Timeline.', style: T.label(N.g56).copyWith(height: 1.4))
              else if (n == 1)
                Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                  // the attached behaviour is the thing being worked on: it sits above the Transform, not below it
                  if (p!.wave != null) _WaveSection(p),
                  KeyedSubtree(key: ValueKey(p.id), child: TransformBoth(p.doc)),
                ])
              else ...[
                Wrap(spacing: 10, runSpacing: 4, children: [
                  for (final id in s.selected)
                    Row(mainAxisSize: MainAxisSize.min, children: [
                      Container(width: 8, height: 8, decoration: BoxDecoration(color: s.byId(id).fam.c, borderRadius: BorderRadius.circular(2))),
                      const SizedBox(width: 4),
                      Text(s.byId(id).name, style: T.label(N.g76)),
                    ]),
                ]),
                const SizedBox(height: 6),
                Text('— = the layers differ. A scrub moves each by the same amount; a typed value sets them all.', style: T.label(N.g56).copyWith(height: 1.4)),
                const SizedBox(height: 10),
                TransformBoth(s.group),
              ],
              const SizedBox(height: 12),
              Text('Click · Shift add · Tab · Esc · Space · Delete', style: T.label(N.g56).copyWith(height: 1.4)),
            ]),
          ),
        ),
        // pinned: the undo list is always in view
        Container(height: 1, color: N.g15),
        Row(children: [
          Text(s.undoSteps == 1 ? '1 step' : '${s.undoSteps} steps', style: T.value(N.g76)),
          const SizedBox(width: 12),
          MouseRegion(
            cursor: s.undoSteps == 0 ? SystemMouseCursors.basic : SystemMouseCursors.click,
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: s.undo,
              child: Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Text('Undo', style: T.name(s.undoSteps == 0 ? N.g56 : N.g95))),
            ),
          ),
        ]),
      ]),
    );
  }
}

/// One box around every selected layer (W5): the selection as a whole.
class _UnionPaint extends CustomPainter {
  const _UnionPaint(this.pts);
  final List<Offset> pts;
  @override
  void paint(Canvas cv, Size sz) {
    final r = Rect.fromLTRB(pts.map((p) => p.dx).reduce(math.min), pts.map((p) => p.dy).reduce(math.min), pts.map((p) => p.dx).reduce(math.max), pts.map((p) => p.dy).reduce(math.max)).inflate(6);
    cv.drawRect(r, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g76.withValues(alpha: .8));
  }

  @override
  bool shouldRepaint(_UnionPaint o) => true;
}

/// The selected layer's motion on the stage: a line through its keys, a diamond at each (the one under the playhead blue, the picked one accent).
class _KeysPaint extends CustomPainter {
  const _KeysPaint(this.l, this.size, this.head, this.picked);
  final LayerM l;
  final Size size;
  final int head;
  final Set<int> picked;
  @override
  void paint(Canvas cv, Size sz) {
    final k = Gui.span / size.width, ks = l.keyFrames, pts = [for (final f in ks) size.center(Offset.zero) + l.pk[f]! / k];
    if (pts.length > 1) cv.drawPath(Path()..addPolygon(pts, false), Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g100.withValues(alpha: .7));
    for (var i = 0; i < pts.length; i++) {
      final p = pts[i], r = 4.5, d = Path()..moveTo(p.dx, p.dy - r)..lineTo(p.dx + r, p.dy)..lineTo(p.dx, p.dy + r)..lineTo(p.dx - r, p.dy)..close();
      cv.drawPath(d, Paint()..color = picked.contains(ks[i]) ? Role.selected : (ks[i] == head ? C.playhead : Role.of(N.g95, Role.key).withValues(alpha: .9)));
      cv.drawPath(d, Paint()..style = PaintingStyle.stroke..strokeWidth = picked.contains(ks[i]) ? 1.5 : (Role.keyEdge != null ? 1 : .75)..color = picked.contains(ks[i]) ? N.g100 : (Role.keyEdge ?? N.g100.withValues(alpha: .6)));
    }
  }

  @override
  bool shouldRepaint(_KeysPaint o) => true;
}


// ---- the behaviour shelf (B1) -----------------------------------------------------------------------------------------------------------

/// One period of a sine, one line: the shelf's Wave and the mark on a layer that has it.
class WaveGlyph extends StatelessWidget {
  const WaveGlyph({super.key, this.color = N.g76});
  final Color color;
  @override
  Widget build(BuildContext context) => SizedBox(width: 18, height: 10, child: CustomPaint(painter: _WavePaint(color)));
}

class _WavePaint extends CustomPainter {
  _WavePaint(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size z) {
    final path = Path();
    for (var i = 0; i <= 24; i++) {
      final x = z.width * i / 24, y = z.height / 2 - math.sin(i / 24 * 2 * math.pi) * (z.height / 2 - 1);
      i == 0 ? path.moveTo(x, y) : path.lineTo(x, y);
    }
    canvas.drawPath(path, Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1);
  }

  @override
  bool shouldRepaint(_WavePaint o) => o.c != c;
}

class _Shelf extends StatelessWidget {
  const _Shelf();
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context);
    Widget row(bool hot) => Container(
          height: TL.height,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          color: hot ? N.g15 : N.g10,
          child: Row(children: [
            const WaveGlyph(color: N.g95),
            const SizedBox(width: 8),
            Expanded(child: Text('Wave', style: T.name(N.g91))),
          ]),
        );
    return Container(
      color: N.g10,
      padding: const EdgeInsets.fromLTRB(0, 12, 0, 8),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Padding(padding: const EdgeInsets.fromLTRB(12, 0, 12, 8), child: Text('SHELF', style: T.micro(N.g56).copyWith(letterSpacing: .8))),
        _ShelfItem(
          // click attaches to the selected layer; a drag carries it to any layer
          dragChild: SizedBox(width: _listW, child: row(true)),
          onTap: () {
            final p = s.primary;
            if (p != null) s.attachWave(p);
          },
          child: row(false),
        ),
      ]),
    );
  }
}

class _ShelfItem extends StatefulWidget {
  const _ShelfItem({required this.child, required this.dragChild, required this.onTap});
  final Widget child, dragChild;
  final VoidCallback onTap;
  @override
  State<_ShelfItem> createState() => _ShelfItemState();
}

class _ShelfItemState extends State<_ShelfItem> {
  bool hov = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: SystemMouseCursors.grab,
        onEnter: (_) => setState(() => hov = true),
        onExit: (_) => setState(() => hov = false),
        child: Draggable<String>(
          data: 'wave',
          feedback: Opacity(opacity: .92, child: DefaultTextStyle(style: const TextStyle(decoration: TextDecoration.none), child: widget.dragChild)),
          childWhenDragging: Opacity(opacity: .4, child: widget.child),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: widget.onTap,
            child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, color: hov ? N.g15 : const Color(0x00262626), child: widget.child),
          ),
        ),
      );
}

/// What the attached wave reads as in the Inspector: its two numbers, edited like any other number, and a way to take it off.
class _WaveSection extends StatelessWidget {
  const _WaveSection(this.l);
  final LayerM l;
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context), w = l.wave!;
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          const WaveGlyph(color: N.g95),
          const SizedBox(width: 8),
          Text('WAVE', style: T.micro(N.g56).copyWith(letterSpacing: .8)),
          const Spacer(),
          MouseRegion(
            cursor: SystemMouseCursors.click,
            child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => s.removeWave(l), child: Padding(padding: const EdgeInsets.all(4), child: Text('Remove', style: T.name(N.g56)))),
          ),
        ]),
        const SizedBox(height: 8),
        ScrubField(label: 'Amplitude', unit: ' px', value: w.amp, min: 0, max: 400, perPx: 1, decimals: 0, onChanged: (v) => s.setWave(l, amp: v)),
        const SizedBox(height: 4),
        ScrubField(label: 'Frequency', unit: ' Hz', value: w.freq, min: .1, max: 8, perPx: .02, decimals: 1, onChanged: (v) => s.setWave(l, freq: v)),
        const SizedBox(height: 12),
        Container(height: 1, color: N.g15),
      ]),
    );
  }
}
