// The graph.

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_ease_handles.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';
import 'two_fingers.dart';

/// The graph: value up, time across. Handles are dragged; only a drag or hover repaints it. The keys at its two ends
/// are pulled up and down too (the user's, 10/30: "左と右にある点を上下に動かせるように"): [onEnd] is told which end and
/// how far, in the graph's own measure (1: from the one key's value to the other's); with [jump] on, a dashed line shows
/// the value jumping there, as the keys it makes will.
class EaseGraph extends StatefulWidget {
  const EaseGraph({super.key, required this.ease, required this.span, required this.frame, required this.onChanged, this.onEnd, this.endSays, this.jump = true, this.overshoot = false, this.onion});
  final Ease ease;
  final Ease? onion;   // (what all the cards make, faint behind the one a hand shapes)
  final EaseSpan? span;
  final int frame;
  final ValueChanged<Ease> onChanged;
  final void Function(bool start, double by)? onEnd;
  final String? Function(bool start, double by)? endSays;   // (what the end pulled makes its value, shown beside it while it is pulled)
  final bool jump, overshoot;   // (overshoot: looks past 0 and 1, and a handle goes there: Alight Motion's Enable Overshoot)
  @override
  State<EaseGraph> createState() => _EaseGraphState();
}

class _EaseGraphState extends State<EaseGraph> {
  static const _end = 100;   // (_drag, _hot: a handle's place in easeHandles, or the key at the start, or at the end)
  int? _drag, _hot;
  (int, int)? _tie;   // (a handle on a key, as Linear's are: the first move tells which, up and down being the key's)
  double? _endAt;   // (where the end pulled is, in the graph's measure, while it is pulled)
  Offset _from = Offset.zero, _handle = Offset.zero;   // (where the hand took it, and where the handle was: it goes by the hand's move, as Alight Motion's)

  static Rect _box(Size s) {   // (AbsentMotion's geoOf: 42 of 575 kept round the curve)
    final m = s.width * 42 / 575;
    return Rect.fromLTRB(m, m, math.max(m + 1, s.width - m), math.max(m + 1, s.height - m));
  }

  Offset _px(Rect g, Offset v) => _pxIn(g, easeReach(widget.ease, widget.overshoot), v);
  static Offset _pxIn(Rect g, (double, double) reach, Offset v) => Offset(g.left + v.dx * g.width, g.bottom - (v.dy - reach.$1) / (reach.$2 - reach.$1) * g.height);

  int? _near(Rect g, Offset p) {
    final hs = easeHandles(widget.ease);
    final at = [for (final (i, h) in hs.indexed) (i, Offset(h.x, h.y)), if (widget.onEnd != null && widget.span != null) ...[(_end, Offset.zero), (_end + 1, const Offset(1, 1))]];
    final d = [for (final (i, v) in at) ((_px(g, v) - p).distance, i)]..sort((a, b) => a.$1.compareTo(b.$1));
    if (d.isEmpty || d.first.$1 > 14) return null;
    _tie = d.length > 1 && d[1].$1 - d.first.$1 < 2 && (d.first.$2 < _end) != (d[1].$2 < _end) ? (math.min(d.first.$2, d[1].$2), math.max(d.first.$2, d[1].$2)) : null;
    return d.first.$2;
  }

  void _take(int? which, Offset at) {
    _drag = which;
    _from = at;
    final hs = easeHandles(widget.ease);
    if (which != null && which < hs.length) _handle = Offset(hs[which].x, hs[which].y);
  }

  void _move(Rect g, Offset p) {
    final (lo, hi) = easeReach(widget.ease, widget.overshoot);
    if (_drag! >= _end) return setState(() => _endAt = (lo + (g.bottom - p.dy) / g.height * (hi - lo)).clamp(lo + .02, hi - .02));
    final hs = easeHandles(widget.ease);
    if (_drag! >= hs.length) return;
    final h = hs[_drag!], d = (p - _from) * h.damping;
    final u = (_handle.dx + d.dx / g.width).clamp(0.0, 1.0), v = _handle.dy - d.dy / g.height * (hi - lo);
    widget.onChanged(h.moved(u, h.overshoot || widget.overshoot ? v.clamp(lo, hi) : v.clamp(0.0, 1.0)));
  }

  void _let(_) {
    final end = _drag, at = _endAt;
    setState(() {
      _drag = _endAt = _tie = null;
    });
    if (end != null && end >= _end && at != null) widget.onEnd?.call(end == _end, at - (end == _end ? 0 : 1));
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final g = _box(box.biggest);
      return MouseRegion(
        cursor: _hot != null || _drag != null ? SystemMouseCursors.grab : MouseCursor.defer,
        onHover: (e) {
          final n = _near(g, e.localPosition);
          if (n != _hot) setState(() => _hot = n);
        },
        onExit: (_) {
          if (_hot != null) setState(() => _hot = null);
        },
        child: GestureDetector(
          supportedDevices: wsPressing,
          behavior: HitTestBehavior.opaque,
          onPanDown: (d) {
            _take(_near(g, d.localPosition), d.localPosition);
            if (_drag != null) setState(() {});
          },
          onPanUpdate: (d) {
            if (_tie case (final handle, final key)?) {
              _tie = null;
              _take(d.delta.dy.abs() > d.delta.dx.abs() ? key : handle, _from);
            }
            if (_drag != null) _move(g, d.localPosition);
          },
          onPanEnd: _let,
          onPanCancel: () => setState(() {
            _drag = _endAt = _tie = null;
          }),
          child: Stack(children: [
          CustomPaint(
            size: box.biggest,
            painter: _GraphPainter(
              ghosts: _ghosts(WsScope.of(context, {WsAspect.document}), widget.span),
              ease: widget.ease,
              from: widget.span?.from,
              to: widget.span?.to,
              frame: widget.frame,
              active: _drag ?? _hot,
              dragging: _drag != null,
              endAt: _endAt,
              jump: widget.jump,
              overshoot: widget.overshoot,
              onion: widget.onion,
            ),
          ),
          if (_drag != null && _drag! >= _end && _endAt != null ? widget.endSays?.call(_drag == _end, _endAt! - (_drag == _end ? 0 : 1)) : null case final says?)
            Positioned(
              left: _drag == _end ? _px(g, Offset(0, _endAt!)).dx + 10 : null,
              right: _drag == _end + 1 ? box.maxWidth - _px(g, Offset(1, _endAt!)).dx + 10 : null,
              top: (_px(g, Offset(0, _endAt!)).dy - 18).clamp(0.0, math.max(0.0, box.maxHeight - 16)),
              child: IgnorePointer(child: Text(says, key: const ValueKey('ease-end-says'), style: T.value(Grey.g95).copyWith(fontSize: T.px(10)))),
            ),
          ]),
        ),
      );
    },
  );
}

/// A relation's stretch: each of its things gets the same curve so much later, in turn (its lag times its place): as
/// the relation spreads it, seen and not touched, in the thing's own colour, as Blender shows the curves it will not let
/// a hand change.
List<(double, Color)> _ghosts(Ws ws, EaseSpan? span) {
  final m = ws.inside, t = int.tryParse(span?.layer.id ?? '');
  if (m == null || t == null || span == null || span.to <= span.from || m.word(t, 'kind') != 'behaviour') return const [];
  final lag = (m.value(t, 'lag', 0) ?? 0) * Ws.fps / (span.to - span.from);
  final members = [for (final s in m.word(t, 'members').split(' ')) if (s.isNotEmpty && s != '_') s];
  if (lag <= 0 || members.length < 2) return const [];
  return [for (final (i, id) in members.indexed) if (i > 0) (lag * i, wsLabel(id))];
}

class _GraphPainter extends CustomPainter {
  const _GraphPainter({required this.ease, required this.from, required this.to, required this.frame, required this.active, required this.dragging, this.ghosts = const [], this.overshoot = false, this.onion, this.endAt, this.jump = true});
  final List<(double, Color)> ghosts;   // (a relation's things: how much later each, across the stretch, and its colour)
  final Ease ease;
  final int? from, to, active;
  final int frame;
  final bool dragging, jump, overshoot;
  final Ease? onion;
  final double? endAt;   // (the key at an end, while it is pulled: where it is)

  @override
  void paint(Canvas c, Size s) {   // (AbsentMotion's EasingPanel.kt drawGraph: its grid, its dashed 0 and 1, the stems, the curve, the handles)
    final g = _EaseGraphState._box(s), reach = easeReach(ease, overshoot), handles = easeHandles(ease), end = _EaseGraphState._end;
    Offset px(Offset v) => _EaseGraphState._pxIn(g, reach, v);
    // (an end pulled: the curve goes from where its start is to where its end is, as the keys it makes will)
    final pulled = endAt != null && dragging && (active ?? 0) >= end ? active : null;
    final y0 = pulled == end ? endAt! : 0.0, y1 = pulled == end + 1 ? endAt! : 1.0;
    double along(double x) => y0 + (y1 - y0) * ease.at(x);
    final a = px(Offset(0, y0)), b = px(Offset(1, y1)), home = (px(Offset.zero), px(const Offset(1, 1)));
    final top = home.$2.dy, bottom = home.$1.dy;
    final grid = Paint()..strokeWidth = 1..color = Grey.g20;   // (few lines, only where the way goes: so small, AbsentMotion's 32 are a grain)
    for (var i = 0; i <= 8; i++) {
      final x = (g.left + i * g.width / 8).floorToDouble() + .5;
      c.drawLine(Offset(x, top), Offset(x, bottom), grid);
    }
    for (var i = 1; i < 4; i++) {
      final y = (top + i * (bottom - top) / 4).floorToDouble() + .5;
      c.drawLine(Offset(g.left, y), Offset(g.right, y), grid);
    }
    void dashed(Offset p, Offset q, Color ink, double wide, double on, double off) {
      final line = Paint()..color = ink..strokeWidth = wide, far = (q - p).distance;
      for (var d = 0.0; d < far; d += on + off) {
        c.drawLine(Offset.lerp(p, q, d / far)!, Offset.lerp(p, q, math.min(1, (d + on) / far))!, line);
      }
    }
    dashed(Offset(g.left, top), Offset(g.right, top), Grey.g56, 1, 4, 3);
    dashed(Offset(g.left, bottom), Offset(g.right, bottom), Grey.g56, 1, 4, 3);
    // The playhead, when it is inside the span.
    if (from != null && to != null && frame > from! && frame < to!) {   // (on a key it would read as the graph's edge)
      final x = g.left + g.width * (frame - from!) / (to! - from!);
      dashed(Offset(x, top), Offset(x, bottom), Grey.g76, 1, 5, 3);
    }
    // A relation's things' copies of the curve, each so much later (thin, faint, under the one a hand shapes).
    c.save();
    c.clipRect(g);
    for (final (shift, hue) in ghosts) {
      final copy = Path()..moveTo(px(Offset(shift, 0)).dx, a.dy);
      for (var i = 1; i <= 96; i++) {
        final x = i / 96, o = px(Offset(x + shift, (onion ?? ease).at(x)));
        copy.lineTo(o.dx, o.dy);
      }
      copy.lineTo(g.right + 4, b.dy);
      c.drawPath(copy, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.2..color = hue.withValues(alpha: .55));
    }
    c.restore();
    if (onion case final whole?) {
      final skin = Path()..moveTo(a.dx, a.dy);
      for (var i = 1; i <= 96; i++) {
        final o = px(Offset(i / 96, whole.at(i / 96)));
        skin.lineTo(o.dx, o.dy);
      }
      c.drawPath(skin, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.6..strokeJoin = StrokeJoin.round..color = Grey.g56);
    }
    // The stems (on the curve as it is pulled: its handles go with its ends).
    Offset handle(double x, double y) => px(Offset(x, y0 + (y1 - y0) * y));
    for (final h in handles) {
      if (h.fromX == null || h.fromY == null) continue;
      final o = handle(h.fromX!, h.fromY!), ink = h.param ? WsT.keyDot : Grey.g95;
      if (h.dashed) {
        dashed(o, handle(h.x, 0), ink, 1, 2.5, 2.5);
      } else {
        c.drawLine(o, handle(h.x, h.y), Paint()..color = ink..strokeWidth = 2.2..strokeCap = StrokeCap.round);
      }
    }
    // The curve.
    final path = Path()..moveTo(a.dx, a.dy);
    const n = 96;
    for (var i = 1; i <= n; i++) {
      final x = i / n;
      final o = px(Offset(x, along(x)));
      if (ease.kind == EaseKind.hold && i == n) path.lineTo(o.dx, a.dy);
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(path, Paint()..style = PaintingStyle.stroke..strokeWidth = 3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round..color = WsT.accentInk);
    // The value jumping at the end pulled: from where it was (the key the jump leaves outside) to where it is now.
    if (pulled != null && jump) {
      final was = pulled == end ? home.$1 : home.$2, now = pulled == end ? a : b;
      dashed(was, now, Role.warning, 1.6, 3.5, 2.5);
      c.drawCircle(was, 3, Paint()..color = Grey.g63);
    }
    // The handles: large, a hand's: yellow the shape's, white where it runs; on a key, a ring round it, so the key is seen.
    for (final (i, h) in handles.indexed) {
      final o = handle(h.x, h.y), on = active == i, ink = h.param ? WsT.keyDot : Grey.g95;
      if ((o - a).distance < 9 || (o - b).distance < 9) {
        c.drawCircle(o, on ? 10 : 9, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 2);
      } else {
        c.drawCircle(o, on ? 7 : 6, Paint()..color = ink);
      }
      if (dragging && on && !h.param) {
        deskText(c, '${h.x.toStringAsFixed(2)}, ${h.y.toStringAsFixed(2)}', o + Offset(h.x < .5 ? 9 : -9, -9), T.value(Grey.g95).copyWith(fontSize: T.px(10)), ax: h.x < .5 ? 0 : 1, ay: 1);
      }
    }
    // Keys at both ends, over the handles (the one pulled, bigger: it is grabbed).
    for (final (i, k) in [a, b].indexed) {
      final r = active == end + i ? 4.5 : 3.0;
      c.save();
      c.translate(k.dx, k.dy);
      c.rotate(math.pi / 4);
      c.drawRect(Rect.fromLTWH(-r, -r, 2 * r, 2 * r), Paint()..color = active == end + i ? WsT.accent : WsT.accentInk);
      c.restore();
    }
    // What it is, under it, as Alight Motion's.
    final name = easeWord(ease);
    if (!easeIsLaid(said(ease))) deskText(c, name.toUpperCase(), Offset(g.center.dx, s.height - 5), T.micro(Grey.g63).copyWith(letterSpacing: .6), ax: .5, ay: 1);   // (cards laid: each says its own)
  }

  @override
  bool shouldRepaint(_GraphPainter o) => o.ease != ease || o.onion != onion || o.overshoot != overshoot || o.endAt != endAt || o.jump != jump || o.from != from || o.to != to || o.frame != frame || o.active != active || o.dragging != dragging || o.ghosts.length != ghosts.length || (ghosts.isNotEmpty && o.ghosts.first.$1 != ghosts.first.$1);
}

