// TEMPORARY — the Skin Swap Proof for the Timeline. A deliberately different skin over the same TimelineSession: no
// name column (time runs the full width), a layer is a line with its name above it, the ruler is at the bottom, lanes
// open on a double click of a name. If this works with no change to timeline_core, the Timeline's meaning lives
// there and not in any skin. Delete after the proof.
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show mono;
import '../../hf/neutral.dart';
import '../../session/editor_session.dart';
import '../../timeline_core/semantics.dart';
import '../../timeline_core/session.dart';

/// Which Timeline skin shows (the proof's switch).
final altTimelineSkin = ValueNotifier(false);

class AltTimeline extends StatefulWidget {
  const AltTimeline({super.key, required this.c});
  final EditorSession c;
  @override
  State<AltTimeline> createState() => _AltTimelineState();
}

class _AltTimelineState extends State<AltTimeline> {
  static const rowH = 30.0, rulerH = 20.0;
  late final TimelineSession s = TimelineSession.of(widget.c);
  Offset? _down;
  bool _moved = false;
  double _h = 300;

  double xOf(num f) => 8 + (f - s.startFrame) * s.pixelsPerFrame;
  double frameAt(double x) => s.startFrame + (x - 8) / s.pixelsPerFrame;
  double lineY(int i) => (i - s.firstRow) * rowH + rowH - 8;
  double rowAt(double y) => s.firstRow + y / rowH;

  TlTarget _hit(Offset p) {
    if (p.dy > _h - rulerH) return const TlEmpty();
    final i = rowAt(p.dy).floor();
    if (i < 0 || i >= s.rows.length) return const TlEmpty();
    final r = s.rows[i];
    final ly = lineY(i);
    // the name, above the line's start
    final name = r.property == null ? '${r.layer['name'] ?? r.layer['kind']}' : '${r.property!['label']}';
    final start = r.property == null ? (r.layer['start'] as num? ?? 0).toDouble() : 0.0;
    if (p.dy < ly - 5 && p.dx >= xOf(start) && p.dx <= xOf(start) + 8 + name.length * 7) return TlRowName(i);
    if ((p.dy - ly).abs() > 8) return const TlEmpty();
    if (r.property != null) {
      final frames = [for (final k in r.keys) (k['frame'] as num).toInt()]..sort();
      for (final f in frames) {
        if ((xOf(f) - p.dx).abs() <= 8) return TlLaneKey(i, f);
      }
      return const TlEmpty();
    }
    if (!r.lanesOpen) {
      for (final f in r.summaryFrames) {
        if ((xOf(f) - p.dx).abs() <= 8) return TlLayerKey(i, f);
      }
    }
    final end = start + (r.layer['duration'] as num? ?? 0).toDouble();
    if (p.dx < xOf(start) - 6 || p.dx > xOf(end) + 6) return const TlEmpty();
    if ((p.dx - xOf(start)).abs() <= 6) return TlBar(i, TlBarPart.start);
    if ((p.dx - xOf(end)).abs() <= 6) return TlBar(i, TlBarPart.end);
    return TlBar(i, TlBarPart.body);
  }

  TlMods get _mods {
    final k = HardwareKeyboard.instance;
    return TlMods(primary: k.isMetaPressed, shift: k.isShiftPressed, alt: k.isAltPressed);
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        _h = box.maxHeight;
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) s.viewport(frames: (box.maxWidth - 16) / s.pixelsPerFrame, rows: (box.maxHeight - rulerH) / rowH);
        });
        return Listener(
          behavior: HitTestBehavior.opaque,
          onPointerSignal: (e) {
            if (e is! PointerScrollEvent) return;
            if (HardwareKeyboard.instance.isMetaPressed) {
              s.zoomAt(math.exp(-e.scrollDelta.dy * .002), frameAt(e.localPosition.dx));
            } else {
              s.setView(startFrame: s.startFrame + e.scrollDelta.dx / s.pixelsPerFrame, firstRow: s.firstRow + e.scrollDelta.dy / rowH);
            }
          },
          onPointerDown: (e) {
            if (e.buttons != kPrimaryMouseButton) return;
            if (e.localPosition.dy > _h - rulerH) {
              s.seek(frameAt(e.localPosition.dx).round());
              return;
            }
            _down = e.localPosition;
            _moved = false;
            s.press(_hit(e.localPosition), frame: frameAt(e.localPosition.dx), row: rowAt(e.localPosition.dy), mods: _mods);
          },
          onPointerMove: (e) {
            if (_down == null) {
              if (e.localPosition.dy > _h - rulerH) s.seek(frameAt(e.localPosition.dx).round());
              return;
            }
            if (!_moved && (e.localPosition - _down!).distance < 3) return;
            _moved = true;
            s.drag(frame: frameAt(e.localPosition.dx), row: rowAt(e.localPosition.dy));
          },
          onPointerUp: (_) {
            if (_down == null) return;
            _down = null;
            s.release();
          },
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onDoubleTapDown: (e) {
              if (_hit(e.localPosition) case TlRowName(:final row)) s.toggleLanes(row);
            },
            child: ListenableBuilder(
              listenable: Listenable.merge([s, widget.c.frame, s.scrub]),
              builder: (context, _) => CustomPaint(painter: _AltPainter(this), size: Size.infinite),
            ),
          ),
        );
      });
}

class _AltPainter extends CustomPainter {
  _AltPainter(this.t);
  final _AltTimelineState t;
  @override
  void paint(Canvas cv, Size size) {
    final s = t.s, c = t.widget.c;
    cv.drawRect(Offset.zero & size, Paint()..color = const Color(0xFF0B0B0B));
    final picked = s.selectedKeys;
    final held = s.heldTimings;
    for (var i = s.firstRow.floor(); i < s.rows.length; i++) {
      final y = t.lineY(i);
      if (y > size.height - _AltTimelineState.rulerH) break;
      final r = s.rows[i];
      final on = c.selectedIds.contains(r.id) && r.property == null;
      final ink = on ? const Color(0xFFFFE14D) : (r.property == null ? N.g76 : N.g44);
      final timing = held[r.id] ?? r.layer;
      final start = r.property == null ? (timing['start'] as num? ?? 0).toDouble() : 0.0;
      final end = r.property == null ? start + (timing['duration'] as num? ?? 0).toDouble() : s.extent.toDouble();
      final name = r.property == null ? '${r.layer['name'] ?? r.layer['kind']}'.toUpperCase() : '· ${r.property!['label']}';
      final tp = TextPainter(text: TextSpan(text: name, style: mono(9, c: ink)), textDirection: TextDirection.ltr)..layout();
      tp.paint(cv, Offset(t.xOf(start), y - 16));
      tp.dispose();
      cv.drawLine(Offset(t.xOf(start), y), Offset(t.xOf(end), y), Paint()..strokeWidth = r.property == null ? 3 : 1..color = ink);
      final keys = r.property != null ? [for (final k in r.keys) tlKeyOf(r, k)] : (r.lanesOpen ? const <Map<String, dynamic>>[] : r.allKeys);
      for (final k in keys) {
        final moving = (s.gesture == TlGesture.keys ? s.initialKeys : s.settlingKeys).any((m) => tlSameKey(m, k));
        final f = (k['frame'] as num).toInt() + (moving ? (s.gesture == TlGesture.keys ? s.deltaFrames : s.settlingDelta) : 0);
        final sel = picked.any((p) => tlSameKey(p, k));
        cv.drawCircle(Offset(t.xOf(f), y), sel ? 5 : 3.5, Paint()..color = sel ? const Color(0xFFFFE14D) : N.g100);
      }
    }
    // the ruler, at the bottom: whole seconds
    final base = size.height - _AltTimelineState.rulerH;
    cv.drawRect(Rect.fromLTWH(0, base, size.width, _AltTimelineState.rulerH), Paint()..color = N.g15);
    for (var f = (s.startFrame ~/ s.fps) * s.fps; t.xOf(f) < size.width; f += s.fps) {
      final tp = TextPainter(text: TextSpan(text: '${f ~/ s.fps}s', style: mono(9, c: N.g69)), textDirection: TextDirection.ltr)..layout();
      tp.paint(cv, Offset(t.xOf(f) + 2, base + 4));
      tp.dispose();
    }
    for (final m in EditorSession.maps(c.state['markers'])) {
      cv.drawCircle(Offset(t.xOf((m['frame'] as num).toDouble()), base + 2), 3, Paint()..color = const Color(0xFF3DDC97));
    }
    if (s.marquee case final m?) {
      final rect = Rect.fromPoints(Offset(t.xOf(m.f0), (m.r0 - s.firstRow) * _AltTimelineState.rowH), Offset(t.xOf(m.f1), (m.r1 - s.firstRow) * _AltTimelineState.rowH));
      cv.drawRect(rect, Paint()..style = PaintingStyle.stroke..color = const Color(0xFFFFE14D));
    }
    final head = t.xOf(s.scrub.value ?? c.frame.value);
    cv.drawLine(Offset(head, 0), Offset(head, size.height), Paint()..color = const Color(0xFFFF3B30)..strokeWidth = 1.5);
    final tp = TextPainter(text: TextSpan(text: '${s.scrub.value ?? c.frame.value}f', style: mono(9, c: const Color(0xFFFF3B30))), textDirection: TextDirection.ltr)..layout();
    tp.paint(cv, Offset(head + 3, base - 12));
    tp.dispose();
  }

  @override
  bool shouldRepaint(_AltPainter o) => true;
}
