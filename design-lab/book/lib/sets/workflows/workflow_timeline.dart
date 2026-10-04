// W4 Timeline: grab, move, resize, snap, zoom / pan, marquee (research/workflows.md W4, research/precedents-timeline.md).
// Consensus is built in: drag a bar's edge = resize, its body = move (keys travel with it), empty ground = marquee, ruler = scrub, keys drag in time
// (several keep their relative timing), everything in whole frames. Where the precedents split, the choice is a knob (TlKnobs, Widgetbook right panel).
// Every drag is one Session gesture: tlBegin, tlSet..., tlCommit (one undo step) or Esc = tlCancel.
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../parts/timeline.dart';
import '../../session.dart';
import '../../tokens.dart';

enum SnapMode { onBypass, offEnable, always, never }

enum ModKey { shift, cmd, alt }

enum ZoomMod { cmd, alt, plain }

enum PanMode { modDrag, middle, shiftWheel }

enum ZoomAnchor { pointer, playhead }

/// The open questions of the timeline's hands, each a value the owner picks in the real window.
class TlKnobs {
  const TlKnobs({
    this.snapMode = SnapMode.onBypass,
    this.snapKey = ModKey.cmd,
    this.playhead = true,
    this.markers = true,
    this.edges = true,
    this.keys = true,
    this.grid = true,
    this.zoomMod = ZoomMod.cmd,
    this.pan = PanMode.middle,
    this.anchor = ZoomAnchor.pointer,
    this.edgeGrab = 8,
    this.keyHit = 7,
    this.threshold = 3,
  });
  final SnapMode snapMode;
  final ModKey snapKey;
  final bool playhead, markers, edges, keys, grid;
  final ZoomMod zoomMod;
  final PanMode pan;
  final ZoomAnchor anchor;
  final int edgeGrab, keyHit, threshold; // px

  factory TlKnobs.read(BuildContext c) {
    final k = c.knobs;
    return TlKnobs(
      snapMode: k.object.dropdown<SnapMode>(label: 'Snap', options: SnapMode.values, initialOption: SnapMode.onBypass, labelBuilder: (m) => const ['On, key bypasses', 'Off, key enables', 'Always on', 'Always off'][m.index]),
      snapKey: k.object.dropdown<ModKey>(label: 'Snap key', options: ModKey.values, initialOption: ModKey.cmd, labelBuilder: (m) => const ['Shift', 'Cmd / Ctrl', 'Alt'][m.index]),
      edgeGrab: k.int.slider(label: 'Edge grab px', initialValue: 8, min: 4, max: 12),
      keyHit: k.int.slider(label: 'Key hit px', initialValue: 7, min: 4, max: 16),
      threshold: k.int.slider(label: 'Drag start px', initialValue: 3, min: 2, max: 8),
      zoomMod: k.object.dropdown<ZoomMod>(label: 'Zoom with', options: ZoomMod.values, initialOption: ZoomMod.cmd, labelBuilder: (m) => const ['Cmd / Ctrl + wheel', 'Alt + wheel', 'Wheel'][m.index]),
      pan: k.object.dropdown<PanMode>(label: 'Pan with', options: PanMode.values, initialOption: PanMode.middle, labelBuilder: (m) => const ['Alt + drag', 'Middle drag', 'Shift + wheel'][m.index]),
      anchor: k.object.dropdown<ZoomAnchor>(label: 'Zoom around', options: ZoomAnchor.values, initialOption: ZoomAnchor.pointer, labelBuilder: (m) => const ['Pointer', 'Playhead'][m.index]),
      playhead: k.boolean(label: 'Snap: playhead', initialValue: true),
      markers: k.boolean(label: 'Snap: markers', initialValue: true),
      edges: k.boolean(label: 'Snap: bar edges', initialValue: true),
      keys: k.boolean(label: 'Snap: keys', initialValue: true),
      grid: k.boolean(label: 'Snap: grid step', initialValue: true),
    );
  }
}

/// [open] how close (px) a dragged edge or key must come to a target before it is pulled onto it. No precedent number is verified.
const _pull = 8.0;
const _maxFrame = 600;

enum _K { none, key, edgeL, edgeR, body }

class _Hit {
  const _Hit(this.kind, [this.layer, this.frame]);
  final _K kind;
  final LayerM? layer;
  final int? frame;
}

class _Drag {
  _Drag(this.hit, this.down, this.add);
  final _Hit hit;
  final Offset down; // lane-local
  final bool add;
  bool moved = false, solo = false, cancelled = false;
}

bool _meta() => HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed;
bool _addKey() => HardwareKeyboard.instance.isShiftPressed || _meta();

class WorkTimeline extends StatefulWidget {
  const WorkTimeline({super.key, required this.focus, required this.knobs});
  final FocusNode focus;
  final TlKnobs knobs;
  @override
  State<WorkTimeline> createState() => _WorkTimelineState();
}

class _WorkTimelineState extends State<WorkTimeline> {
  late Session s;
  _Drag? _drag;
  Rect? _marquee; // lane-local
  MouseCursor _cursor = SystemMouseCursors.basic;
  bool _panning = false;
  double _laneW = 800;
  TlKnobs get k => widget.knobs;

  double _px(num frame) => (frame - s.scroll) * s.ppf;
  double _maxScroll() => math.max(0, Session.lastFrame + 30 - _laneW / s.ppf);

  _Hit _hit(Offset p) {
    final i = p.dy ~/ TL.height;
    if (p.dy < 0 || i >= s.layers.length) return const _Hit(_K.none);
    final l = s.layers[i];
    for (final f in l.keyFrames) {
      if ((p.dx - _px(f)).abs() <= k.keyHit) return _Hit(_K.key, l, f);
    }
    final left = _px(l.start), right = math.max(_px(l.end), left + 3), e = math.min(k.edgeGrab.toDouble(), (right - left) / 3);
    if (p.dx >= left - 2 && p.dx <= left + e) return _Hit(_K.edgeL, l);
    if (p.dx >= right - e && p.dx <= right + 2) return _Hit(_K.edgeR, l);
    if (p.dx > left && p.dx < right) return _Hit(_K.body, l);
    return const _Hit(_K.none);
  }

  bool _snapOn() {
    final held = switch (k.snapKey) {
      ModKey.shift => HardwareKeyboard.instance.isShiftPressed,
      ModKey.cmd => _meta(),
      ModKey.alt => HardwareKeyboard.instance.isAltPressed,
    };
    return switch (k.snapMode) {
      SnapMode.onBypass => !held,
      SnapMode.offEnable => held,
      SnapMode.always => true,
      SnapMode.never => false,
    };
  }

  /// The grid step the ruler labels: the finest of these that is at least 70 px apart (the ruler's own rule).
  int _gridStep() => const [1, 2, 5, 10, 15, 30, 60, 120, 300].firstWhere((f) => f * s.ppf >= 70, orElse: () => 300);

  /// Pull [cands] (frames) onto the nearest enabled target. Returns the frame shift to add and the target, or null when snap is off or nothing is near.
  (int, int)? _pullTo(List<int> cands, {Set<LayerM> skipEdges = const {}, Set<LayerM> skipKeysOf = const {}, Set<String> skipKeys = const {}}) {
    if (!_snapOn()) return null;
    final targets = <int>[];
    if (k.playhead) targets.add(s.frame);
    if (k.markers) targets.addAll(s.markers);
    for (final l in s.layers) {
      if (k.edges && !skipEdges.contains(l)) targets.addAll([l.start.round(), l.end.round()]);
      if (k.keys && !skipKeysOf.contains(l)) {
        for (final f in l.keyFrames) {
          if (!skipKeys.contains(Session.kid(l, f))) targets.add(f);
        }
      }
    }
    (int, int)? best;
    var bestPx = _pull + .001;
    for (final c in cands) {
      final t = [...targets, if (k.grid) (c / _gridStep()).round() * _gridStep()];
      for (final x in t) {
        final d = (x - c) * s.ppf;
        if (d.abs() < bestPx) {
          bestPx = d.abs();
          best = (x - c, x);
        }
      }
    }
    return best;
  }

  // ---- pointer ---------------------------------------------------------------------------------------------------------------

  void _down(PointerDownEvent e) {
    widget.focus.requestFocus();
    final hk = HardwareKeyboard.instance;
    if ((k.pan == PanMode.middle && e.buttons == kMiddleMouseButton) || (k.pan == PanMode.modDrag && hk.isAltPressed)) {
      setState(() => _panning = true);
      return;
    }
    if (e.buttons != kPrimaryButton) return;
    final h = _hit(e.localPosition), add = _addKey(), d = _Drag(h, e.localPosition, add), l = h.layer;
    _drag = d;
    switch (h.kind) {
      case _K.key:
        if (add) {
          s.pickKey(l!.id, h.frame!, add: true);
        } else if (s.keyOn(l!, h.frame!)) {
          d.solo = true;
        } else {
          s.pickKey(l.id, h.frame!);
        }
      case _K.edgeL || _K.edgeR:
        s.select(l!.id, add: add);
      case _K.body:
        if (add) {
          s.select(l!.id, add: true);
        } else if (s.isSelected(l!.id)) {
          d.solo = true;
        } else {
          s.select(l.id);
        }
      case _K.none:
        break;
    }
  }

  void _move(PointerMoveEvent e) {
    if (_panning) {
      s.view(scroll: (s.scroll - e.delta.dx / s.ppf).clamp(0.0, _maxScroll()));
      return;
    }
    final d = _drag;
    if (d == null || d.cancelled) return;
    if (!d.moved) {
      if ((e.localPosition - d.down).distance < k.threshold) return;
      d.moved = true;
      if (d.hit.kind != _K.none) s.tlBegin();
    }
    final p = e.localPosition;
    if (d.hit.kind == _K.none) {
      setState(() => _marquee = Rect.fromPoints(d.down, p));
      _marqueeSelect(Rect.fromPoints(d.down, p), d.add);
      s.say('${s.selected.length} layer${s.selected.length == 1 ? '' : 's'}, ${s.keySel.length} key${s.keySel.length == 1 ? '' : 's'} in the marquee');
      return;
    }
    if (!s.tlActive) {
      d.cancelled = true; // Esc took it back
      return;
    }
    final raw = ((p.dx - d.down.dx) / s.ppf).round();
    switch (d.hit.kind) {
      case _K.body:
        _moveBars(raw);
      case _K.key:
        _moveKeys(d.hit, raw);
      case _K.edgeL || _K.edgeR:
        _trim(d.hit, raw);
      case _K.none:
        break;
    }
    s.say(_note(d.hit));
  }

  /// What the drag in progress reads out under the Stage: the bar's start, end and length, or where the picked keys are.
  String _note(_Hit h) {
    final l = h.layer;
    if (h.kind == _K.key) {
      final fs = s.keySel.map((id) => int.parse(id.substring(id.indexOf(':') + 1))).toList()..sort();
      return '${fs.length} key${fs.length == 1 ? '' : 's'} at ${fs.take(5).join(', ')}${fs.length > 5 ? ' ...' : ''}';
    }
    return '${l!.name}   ${l.start.round()} to ${l.end.round()}   ${(l.end - l.start).round()} frames';
  }

  void _up(PointerEvent e) {
    if (_panning) {
      setState(() => _panning = false);
      return;
    }
    final d = _drag;
    _drag = null;
    if (d == null) return;
    if (d.moved) {
      if (d.hit.kind != _K.none) s.tlCommit();
    } else if (d.hit.kind == _K.none) {
      if (!d.add) s.clear();
    } else if (d.solo && !d.add) {
      d.hit.kind == _K.key ? s.pickKey(d.hit.layer!.id, d.hit.frame!) : s.select(d.hit.layer!.id);
    }
    s.snap(null);
    s.say(null);
    if (_marquee != null) setState(() => _marquee = null);
  }

  void _cancel(PointerCancelEvent e) {
    if (_drag?.moved ?? false) s.tlCancel();
    _drag = null;
    _panning = false;
    if (_marquee != null) setState(() => _marquee = null);
  }

  void _marqueeSelect(Rect r, bool add) {
    final layers = <String>{}, keys = <String>{};
    for (var i = 0; i < s.layers.length; i++) {
      final l = s.layers[i];
      if (r.bottom < i * TL.height || r.top > (i + 1) * TL.height) continue;
      if (r.right >= _px(l.start) && r.left <= _px(l.end)) layers.add(l.id);
      for (final f in l.keyFrames) {
        if (_px(f) >= r.left && _px(f) <= r.right) keys.add(Session.kid(l, f));
      }
    }
    s.marquee(layers, keys, add: add);
  }

  // ---- the three drags ---------------------------------------------------------------------------------------------------------

  void _moveBars(int raw) {
    final moved = {for (final id in s.selected) s.byId(id)};
    final base = s.tlBase;
    var df = raw;
    final lowest = moved.map((l) => math.min(base[l]!.start, base[l]!.pk.keys.isEmpty ? 1e9 : base[l]!.pk.keys.reduce(math.min).toDouble())).reduce(math.min);
    df = math.max(df, -lowest.round());
    final snap = _pullTo([for (final l in moved) ...[base[l]!.start.round() + df, base[l]!.end.round() + df]], skipEdges: moved, skipKeysOf: moved);
    if (snap != null && snap.$1 + df >= -lowest.round()) df += snap.$1;
    s.snap(snap?.$2);
    final picked = <String>{};
    for (final id in s.tlKeySel) {
      final i = id.indexOf(':'), l = s.byId(id.substring(0, i)), f = int.parse(id.substring(i + 1));
      picked.add(moved.contains(l) ? Session.kid(l, f + df) : id);
    }
    for (final l in moved) {
      final b = base[l]!;
      s.tlSet(l, start: b.start + df, end: b.end + df, pk: {for (final e in b.pk.entries) e.key + df: e.value}, picked: l == moved.last ? picked : null);
    }
  }

  void _moveKeys(_Hit h, int raw) {
    final base = s.tlBase, ids = s.tlKeySel;
    final byLayer = <LayerM, List<int>>{};
    for (final id in ids) {
      final i = id.indexOf(':');
      byLayer.putIfAbsent(s.byId(id.substring(0, i)), () => []).add(int.parse(id.substring(i + 1)));
    }
    var df = raw;
    final lowest = byLayer.values.expand((v) => v).reduce(math.min);
    df = math.max(df, -lowest);
    final snap = _pullTo([h.frame! + df], skipKeys: ids);
    if (snap != null && snap.$1 + df >= -lowest) df += snap.$1;
    s.snap(snap?.$2);
    final picked = <String>{};
    for (final e in byLayer.entries) {
      final pk = Map.of(base[e.key]!.pk);
      for (final f in e.value) {
        pk.remove(f);
      }
      for (final f in e.value) {
        pk[f + df] = base[e.key]!.pk[f]!;
        picked.add(Session.kid(e.key, f + df));
      }
      s.tlSet(e.key, pk: pk);
    }
    s.tlSet(byLayer.keys.first, picked: picked);
  }

  void _trim(_Hit h, int raw) {
    final l = h.layer!, b = s.tlBase[l]!, left = h.kind == _K.edgeL;
    var e = (left ? b.start : b.end).round() + raw;
    final snap = _pullTo([e], skipEdges: {l});
    if (snap != null) e += snap.$1;
    s.snap(snap?.$2);
    if (left) {
      s.tlSet(l, start: e.clamp(0, b.end.round() - 1).toDouble());
    } else {
      s.tlSet(l, end: e.clamp(b.start.round() + 1, _maxFrame).toDouble());
    }
  }

  // ---- hover and wheel ---------------------------------------------------------------------------------------------------------

  void _hover(PointerHoverEvent e) {
    final h = _hit(e.localPosition);
    s.hover(h.layer?.id);
    final c = switch (h.kind) {
      _K.edgeL || _K.edgeR => SystemMouseCursors.resizeColumn,
      _K.key => SystemMouseCursors.click,
      _K.body => SystemMouseCursors.grab,
      _K.none => SystemMouseCursors.basic,
    };
    if (c != _cursor) setState(() => _cursor = c);
  }

  void _wheel(PointerSignalEvent e) {
    if (e is! PointerScrollEvent) return;
    final hk = HardwareKeyboard.instance, x = e.localPosition.dx - TL.label;
    final zoom = switch (k.zoomMod) {
      ZoomMod.cmd => _meta(),
      ZoomMod.alt => hk.isAltPressed,
      ZoomMod.plain => !hk.isShiftPressed && !_meta() && !hk.isAltPressed,
    };
    if (zoom) {
      final np = (s.ppf * math.exp(-e.scrollDelta.dy / 400)).clamp(1.0, 24.0);
      final screenX = k.anchor == ZoomAnchor.pointer ? x : _px(s.frame), frame = k.anchor == ZoomAnchor.pointer ? s.scroll + x / s.ppf : s.frame.toDouble();
      s.ppf = np;
      s.view(scroll: (frame - screenX / np).clamp(0.0, _maxScroll()));
      return;
    }
    final dx = e.scrollDelta.dx, dy = e.scrollDelta.dy;
    final d = k.pan == PanMode.shiftWheel ? (hk.isShiftPressed ? (dx != 0 ? dx : dy) : 0.0) : (dx != 0 ? dx : (k.zoomMod == ZoomMod.plain ? 0.0 : dy));
    if (d != 0) s.view(scroll: (s.scroll + d / s.ppf).clamp(0.0, _maxScroll()));
  }

  @override
  Widget build(BuildContext context) {
    s = SessionScope.of(context);
    final primary = s.primary == null ? null : s.indexOf(s.primary!.id), hov = s.hovered == null ? null : s.indexOf(s.hovered!);
    return LayoutBuilder(builder: (context, box) {
      _laneW = math.max(1, box.maxWidth - TL.label);
      return Listener(
        onPointerSignal: _wheel,
        child: Stack(children: [
          Positioned.fill(
            child: TimelinePanel(
              layers: [
                for (final l in s.layers)
                  Layer(l.name, l.fam, l.start, l.end, [for (final f in l.keyFrames) f.toDouble()], {for (final f in l.keyFrames) if (s.keyOn(l, f)) f.toDouble()}),
              ],
              selected: -1,
              picks: {for (final id in s.selected) s.indexOf(id)},
              hover: hov,
              ring: s.keyRing ? primary : null,
              head: s.frame,
              pixelsPerFrame: s.ppf,
              scroll: s.scroll,
              onScrub: s.seek,
              onPick: (i, add) {
                widget.focus.requestFocus();
                s.select(s.layers[i].id, add: add);
              },
              onHover: (i) => s.hover(i == null ? null : s.layers[i].id),
            ),
          ),
          Positioned(
            left: TL.label,
            top: 0,
            right: 0,
            bottom: 0,
            child: IgnorePointer(
              child: s.snapFrame == null
                  ? CustomPaint(painter: _Overlay(s.markers, s.ppf, s.scroll, null, 0, _marquee))
                  : TweenAnimationBuilder<double>(
                      key: ValueKey(s.snapFrame),
                      tween: Tween(begin: 1, end: 0),
                      duration: Mo.dur,
                      curve: Mo.ease,
                      builder: (_, flash, _) => CustomPaint(painter: _Overlay(s.markers, s.ppf, s.scroll, s.snapFrame, flash, _marquee)),
                    ),
            ),
          ),
          Positioned(
            left: TL.label,
            top: TL.ruler,
            right: 0,
            bottom: 0,
            child: MouseRegion(
              cursor: _panning ? SystemMouseCursors.grabbing : _cursor,
              onHover: _hover,
              onExit: (_) => s.hover(null),
              child: Listener(behavior: HitTestBehavior.opaque, onPointerDown: _down, onPointerMove: _move, onPointerUp: _up, onPointerCancel: _cancel),
            ),
          ),
        ]),
      );
    });
  }
}

/// Over the lanes and the ruler, no hit testing: marker flags and lines, the snap line with its 120 ms flash, the marquee.
class _Overlay extends CustomPainter {
  const _Overlay(this.markers, this.ppf, this.scroll, this.snap, this.flash, this.marquee);
  final List<int> markers;
  final double ppf, scroll, flash;
  final int? snap;
  final Rect? marquee;

  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    final p = Paint();
    for (final m in markers) {
      final x = ((m - scroll) * ppf).roundToDouble();
      c.drawRect(Rect.fromLTWH(x, TL.ruler, 1, s.height - TL.ruler), p..color = N.g63.withValues(alpha: .3));
      c.drawPath(Path()..moveTo(x - 4, TL.ruler - 7)..lineTo(x + 5, TL.ruler - 7)..lineTo(x + .5, TL.ruler - 1)..close(), p..color = N.g63);
      // a name for the flag, on a small plate just under the ruler (the ruler's own labels sit above it)
      final tp = TextPainter(text: TextSpan(text: 'M${markers.indexOf(m) + 1}', style: T.label(N.g76)), textDirection: TextDirection.ltr)..layout();
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x + 3, TL.ruler + 2, tp.width + 6, tp.height + 4), const Radius.circular(2)), Paint()..color = N.g10.withValues(alpha: .92));
      tp.paint(c, Offset(x + 6, TL.ruler + 4));
      tp.dispose();
    }
    if (snap != null) {
      final x = ((snap! - scroll) * ppf).roundToDouble();
      if (flash > 0) c.drawRect(Rect.fromLTWH(x - 2, TL.ruler, 5, s.height - TL.ruler), p..color = C.mode.withValues(alpha: flash * .6));
      c.drawRect(Rect.fromLTWH(x, TL.ruler, 1, s.height - TL.ruler), p..color = N.g100.withValues(alpha: .9));
    }
    final m = marquee?.shift(const Offset(0, TL.ruler));
    if (m != null) {
      c.drawRect(m, p..color = C.mode.withValues(alpha: .2));
      c.drawRect(m.deflate(.5), Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = Role.selected);
    }
  }

  @override
  bool shouldRepaint(_Overlay o) => true;
}
