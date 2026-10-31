// The graph.

import 'dart:math' as math;

import 'package:flutter/services.dart' show HardwareKeyboard;
import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_ease_handles.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';
import 'two_fingers.dart';

/// The graph: value up, time across. Handles are dragged; only a drag or hover repaints it. The two ends stand on a bar
/// each, left and right, and are pulled up and down them (the user's, 10/30: "左と右にある点を上下に動かせるように", and
/// 10/31, the bars drawn on a picture of it, "確定してもその位置のままで"): [onEnd] is told which end and how far, in the
/// bars' own measure (1: from the one key's value to the other's as they first were); [ends] is where they stand. With
/// [jump] on, a dashed line shows the value jumping there, as the keys it makes will.
/// Where the curve's square lies in a graph of that size, all it can be, with room each side for the bars; and how far out of it they stand: past any handle, which goes no further than its edge.
Rect easeBox(Size s) {
  final side = Shape.z(14) + easeBarOut, top = Shape.z(22), under = Shape.z(36);   // (room for the bars, the thing's name, the frames and the way's name: no more, whatever its size)
  return Rect.fromLTRB(side, top, math.max(side + 1, s.width - side), math.max(top + 1, s.height - under));
}

double get easeBarOut => Shape.z(16);

/// How far up and down a graph looks: the way's own reach, and far enough for where the ends stand, and for all that ways laid make: one card's overshoot is theirs too.
(double, double) easeFit(Ease e, bool over, (double, double) ends, Ease? onion) {
  var (lo, hi) = easeReach(e, over);
  lo = math.min(lo, math.min(ends.$1, ends.$2) - .15);
  hi = math.max(hi, math.max(ends.$1, ends.$2) + .15);
  for (final v in [...?onion?.drawn, if (easeIsLaid(said(e))) ...?e.drawn]) {
    final y = ends.$1 + (ends.$2 - ends.$1) * v;
    lo = math.min(lo, y - .1);
    hi = math.max(hi, y + .1);
  }
  return (lo, hi);
}

class EaseGraph extends StatefulWidget {
  const EaseGraph({super.key, required this.ease, required this.span, required this.frame, required this.onChanged, this.onEnd, this.onEndMove, this.endSays, this.jump = true, this.overshoot = false, this.beats = const [], this.onion, this.ends = (0, 1)});
  final Ease ease;
  final (double, double) ends;
  final Ease? onion;   // (what all the cards make, faint behind the one a hand shapes)
  final EaseSpan? span;
  final int frame;
  final ValueChanged<Ease> onChanged;
  final void Function(bool start, double by)? onEnd, onEndMove;   // (let go; and as it goes)
  final String? Function(bool start, double by)? endSays;   // (what the end pulled makes its value, shown beside it while it is pulled)
  final bool jump, overshoot;   // (overshoot: looks past 0 and 1, and a handle goes there: Alight Motion's Enable Overshoot)
  final List<double> beats;   // (where the document's beats fall in the stretch, 0 to 1: a line each, and a handle stays on one it is brought near, as the timeline's grid)
  @override
  State<EaseGraph> createState() => _EaseGraphState();
}

class _EaseGraphState extends State<EaseGraph> {
  static const _end = 100;   // (_drag, _hot: a handle's place in easeHandles, or the key at the start, or at the end)
  int? _drag, _hot;
  double? _endAt;   // (where the end pulled is, in the graph's measure, while it is pulled)
  double _endFrom = 0;   // (where it stood when taken)
  (double, double)? _reachHeld;   // (the graph's measure while a hand holds anything: it does not move under the hand)
  Offset _from = Offset.zero, _handle = Offset.zero;   // (where the hand took it, and where the handle was: it goes by the hand's move, as Alight Motion's)

  static Rect _box(Size s) => easeBox(s);

  (double, double) get _reach => _reachHeld ?? easeFit(widget.ease, widget.overshoot, widget.ends, widget.onion);
  Offset _px(Rect g, Offset v) => _pxIn(g, _reach, Offset(v.dx, widget.ends.$1 + (widget.ends.$2 - widget.ends.$1) * v.dy));   // (a point of the curve, on the graph between its two ends)
  static Offset _pxIn(Rect g, (double, double) reach, Offset v) => Offset(g.left + v.dx * g.width, g.bottom - (v.dy - reach.$1) / (reach.$2 - reach.$1) * g.height);

  int? _near(Rect g, Offset p) {
    if (widget.onEnd != null && widget.span != null) {
      final reach = Shape.z(9);
      if ((p.dx - (g.left - easeBarOut)).abs() < reach) return _end;
      if ((p.dx - (g.right + easeBarOut)).abs() < reach) return _end + 1;
    }
    final d = [for (final (i, h) in easeHandles(widget.ease).indexed) ((_px(g, Offset(h.x, h.y)) - p).distance, i)]..sort((a, b) => a.$1.compareTo(b.$1));
    return d.isEmpty || d.first.$1 > 14 ? null : d.first.$2;
  }

  void _take(int? which, Offset at) {
    _drag = which;
    _from = at;
    if (which != null) _reachHeld = _reach;
    if (which != null && which >= _end) _endFrom = which == _end ? widget.ends.$1 : widget.ends.$2;
    final hs = easeHandles(widget.ease);
    if (which != null && which < hs.length) _handle = Offset(hs[which].x, hs[which].y);
  }

  void _move(Rect g, Offset p) {
    final (lo, hi) = _reach;
    final (y0, y1) = widget.ends;
    if (_drag! >= _end) {
      setState(() => _endAt = (lo + (g.bottom - p.dy) / g.height * (hi - lo)).clamp(lo + .02, hi - .02));
      return widget.onEndMove?.call(_drag == _end, _endAt! - _endFrom);
    }
    final hs = easeHandles(widget.ease);
    if (_drag! >= hs.length) return;
    final h = hs[_drag!], d = (p - _from) * h.damping;
    var u = (_handle.dx + d.dx / g.width).clamp(0.0, 1.0);
    final v = _handle.dy - d.dy / g.height * (hi - lo) / ((y1 - y0).abs() < 1e-6 ? 1 : y1 - y0);
    if (!HardwareKeyboard.instance.isMetaPressed) {
      for (final b in widget.beats) {
        if ((b - u).abs() * g.width <= Shape.z(6)) u = b;
      }
    }
    widget.onChanged(h.moved(u, h.along ? u : h.overshoot || widget.overshoot ? v.clamp(lo, hi) : v.clamp(0.0, 1.0)));
  }

  void _let(_) {
    final end = _drag, at = _endAt;
    setState(() {
      _drag = _endAt = _reachHeld = null;
    });
    if (end != null && end >= _end && at != null) widget.onEnd?.call(end == _end, at - _endFrom);
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
            if (_drag != null) _move(g, d.localPosition);
          },
          onPanEnd: _let,
          onPanCancel: () => setState(() {
            _drag = _endAt = _reachHeld = null;
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
              beats: widget.beats,
              overshoot: widget.overshoot,
              onion: widget.onion,
              ends: widget.ends,
              reach: _reach,
            ),
          ),
          if (_drag != null && _drag! >= _end && _endAt != null ? widget.endSays?.call(_drag == _end, _endAt! - _endFrom) : null case final says?)
            Positioned(
              left: _drag == _end ? g.left + 10 : null,
              right: _drag == _end + 1 ? box.maxWidth - g.right + 10 : null,
              top: (_pxIn(g, _reach, Offset(0, _endAt!)).dy - 18).clamp(0.0, math.max(0.0, box.maxHeight - 16)),
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
  const _GraphPainter({required this.ease, required this.from, required this.to, required this.frame, required this.active, required this.dragging, this.ghosts = const [], this.beats = const [], this.overshoot = false, this.onion, this.ends = (0, 1), required this.reach, this.endAt, this.jump = true});
  final (double, double) reach;
  final List<(double, Color)> ghosts;   // (a relation's things: how much later each, across the stretch, and its colour)
  final List<double> beats;
  final Ease ease;
  final int? from, to, active;
  final int frame;
  final bool dragging, jump, overshoot;
  final Ease? onion;
  final (double, double) ends;
  final double? endAt;   // (the key at an end, while it is pulled: where it is)

  @override
  void paint(Canvas c, Size s) {   // (AbsentMotion's EasingPanel.kt drawGraph: its grid, its dashed 0 and 1, the stems, the curve, the handles)
    final g = _EaseGraphState._box(s), handles = easeHandles(ease), end = _EaseGraphState._end;
    Offset px(Offset v) => _EaseGraphState._pxIn(g, reach, v);
    // (an end pulled: the curve goes from where its start is to where its end is, as the keys it makes will)
    final pulled = endAt != null && dragging && (active ?? 0) >= end ? active : null;
    final y0 = pulled == end ? endAt! : ends.$1, y1 = pulled == end + 1 ? endAt! : ends.$2;
    double along(double x) => y0 + (y1 - y0) * ease.at(x);
    final a = px(Offset(0, y0)), b = px(Offset(1, y1)), home = (px(Offset.zero), px(const Offset(1, 1)));
    final top = home.$2.dy, bottom = home.$1.dy;
    final grid = Paint()..strokeWidth = 1..color = Grey.g20;   // (few lines, only where the way goes: so small, AbsentMotion's 32 are a grain)
    for (var i = 0; i <= 8; i++) {
      final x = (g.left + i * g.width / 8).floorToDouble() + .5;
      c.drawLine(Offset(x, top), Offset(x, bottom), grid);
    }
    for (final b in beats) {
      final x = g.left + g.width * b;
      c.drawLine(Offset(x, top), Offset(x, bottom), Paint()..strokeWidth = 1..color = Grey.g38);
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
    // A relation's things' copies of the curve, each so much later (thin, faint, under the one a hand shapes).
    c.save();
    c.clipRect(g);
    for (final (shift, hue) in ghosts) {
      final copy = Path()..moveTo(px(Offset(shift, 0)).dx, a.dy);
      for (var i = 1; i <= 96; i++) {
        final x = i / 96, o = px(Offset(x + shift, y0 + (y1 - y0) * (onion ?? ease).at(x)));
        copy.lineTo(o.dx, o.dy);
      }
      copy.lineTo(g.right + 4, b.dy);
      c.drawPath(copy, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.2..color = hue.withValues(alpha: .55));
    }
    c.restore();
    if (onion case final whole?) {
      final skin = Path()..moveTo(a.dx, a.dy);
      for (var i = 1; i <= 192; i++) {
        final o = px(Offset(i / 192, y0 + (y1 - y0) * whole.at(i / 192)));
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
    const n = 192;
    for (var i = 1; i <= n; i++) {
      final x = i / n;
      final o = px(Offset(x, along(x)));
      if (ease.kind == EaseKind.hold && i == n) path.lineTo(o.dx, a.dy);
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(path, Paint()..style = PaintingStyle.stroke..strokeWidth = 3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round..color = WsT.accentInk);
    // The timeline here: its playhead in its own colour, where on the way it is now, and the frames: which part is being read.
    final numbers = T.micro(Grey.g56);
    if (from != null && to != null && to! > from!) {
      if (g.width > Shape.z(80)) {   // (too narrow, only the frame now)
        deskText(c, '$from', Offset(g.left, bottom + 6), numbers);
        deskText(c, '$to', Offset(g.right, bottom + 6), numbers, ax: 1);
      }
      if (frame >= from! && frame <= to!) {
        final u = (frame - from!) / (to! - from!), x = g.left + g.width * u, y = y0 + (y1 - y0) * (onion ?? ease).at(u), line = Paint()..color = WsT.playhead..strokeWidth = 1.5;
        c.drawLine(Offset(x, top - 8), Offset(x, bottom + 4), line);
        c.drawCircle(px(Offset(u, y)), 4.5, Paint()..color = WsT.playhead);
        if (frame != from && frame != to) deskText(c, '$frame', Offset(x, bottom + 6), T.micro(WsT.playhead).copyWith(fontWeight: FontWeight.w700), ax: .5);
      } else if (g.width > Shape.z(80)) {   // (not in this stretch: which way it lies, and where)
        final before = frame < from!;
        deskText(c, before ? '‹ $frame' : '$frame ›', Offset(g.center.dx, bottom + 6), T.micro(WsT.playhead).copyWith(fontWeight: FontWeight.w700), ax: .5);
      }
    }
    // The value jumping at the end pulled: from where it was (the key the jump leaves outside) to where it is now.
    if (pulled != null && jump) {
      final was = pulled == end ? home.$1 : home.$2, now = pulled == end ? a : b;
      dashed(was, now, Role.warning, 1.6, 3.5, 2.5);
      c.drawCircle(was, 3, Paint()..color = Grey.g63);
    }
    // The handles: large, a hand's: yellow the shape's, white where it runs.
    for (final (i, h) in handles.indexed) {
      final o = handle(h.x, h.y), on = active == i, ink = h.param ? WsT.keyDot : Grey.g95;
      c.drawCircle(o, on ? 7 : 6, Paint()..color = ink);
      if (dragging && on && !h.param) {
        deskText(c, '${h.x.toStringAsFixed(2)}, ${h.y.toStringAsFixed(2)}', o + Offset(h.x < .5 ? 9 : -9, -9), T.value(Grey.g95).copyWith(fontSize: T.px(10)), ax: h.x < .5 ? 0 : 1, ay: 1);
      }
    }
    // The bars, left and right, as high as the way's 0 to 100, longer where an end has gone past; each end a grip on its own.
    final rail = easeBarOut;
    for (final (i, (x, k, inward)) in [(g.left - rail, a, 1.0), (g.right + rail, b, -1.0)].indexed) {
      final on = active == end + i, cap = rail * .35;
      final bar = Paint()..color = on ? WsT.accentInk : Grey.g63..strokeWidth = 2..strokeCap = StrokeCap.round;
      c.drawLine(Offset(x, math.min(top, k.dy)), Offset(x, math.max(bottom, k.dy)), bar);
      c.drawLine(Offset(x, top), Offset(x + inward * cap, top), bar);
      c.drawLine(Offset(x, bottom), Offset(x + inward * cap, bottom), bar);
      c.drawLine(Offset(x, k.dy), k, Paint()..color = Grey.g63..strokeWidth = 1);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(x, k.dy), width: on ? 11 : 9, height: on ? 18 : 14), const Radius.circular(3)), Paint()..color = WsT.accentInk);
    }
    // What it is, under it, as Alight Motion's.
    final name = easeIsLaid(said(ease)) ? 'Result' : easeWord(ease);
    deskText(c, name.toUpperCase(), Offset(g.center.dx, s.height - 5), T.micro(Grey.g63).copyWith(letterSpacing: .6), ax: .5, ay: 1);
  }

  @override
  bool shouldRepaint(_GraphPainter o) => o.beats != beats || o.reach != reach || o.ease != ease || o.onion != onion || o.ends != ends || o.overshoot != overshoot || o.endAt != endAt || o.jump != jump || o.from != from || o.to != to || o.frame != frame || o.active != active || o.dragging != dragging || o.ghosts.length != ghosts.length || (ghosts.isNotEmpty && o.ghosts.first.$1 != ghosts.first.$1);
}

