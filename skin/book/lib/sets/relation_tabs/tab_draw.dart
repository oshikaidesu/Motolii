// Relations tab, way C "draw it": every thing of the document seen whole (out of the camera too), and everything said
// by a stroke, no handles: a loop drawn round things makes a move of them; with a move picked, a stroke from one of
// its things says where they come from (straight out: shifted that way; ending on another thing: out of it; a small
// ring round their middle: gathered, the tighter the smaller; a spiral or zigzag: scattered, as much as it is big; up
// with a flick at the end: clear, or back); how fast it was drawn says how long, its slowing at the end how it eases,
// going past and back how far it overshoots. A stroke along the strip below puts it in time (where it begins) and
// spreads the things apart (its length over them). The last stroke stays, faint. Joined to the document through
// moves.dart; where things show is always asked of the inside.
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

import '../../motolii.dart';
import '../../tokens.dart';
import '../../workspace/moves.dart';
import '../../workspace/stage_art.dart' show WsArt;
import '../../workspace/ws.dart';

class TabDraw extends StatefulWidget {
  const TabDraw({super.key});
  @override
  State<TabDraw> createState() => TabDrawState();
}

class TabDrawState extends State<TabDraw> {
  final _stroke = <(Offset, Duration)>[];
  var _last = <Offset>[];
  Color? _said;   // (what the last stroke said, by whose ink; none: nothing)

  late Rect _stage, _lane;
  late double _k;
  late Offset _o;
  Offset _screen(double x, double y) => _o + Offset(x, -y) * _k;
  double _laneX(double t) => _lane.left + 12 + t / (Ws.duration / Ws.fps) * (_lane.width - 24);
  double _laneT(double x) => ((x - _lane.left - 12) / (_lane.width - 24) * Ws.duration / Ws.fps).clamp(0, Ws.duration / Ws.fps);

  /// Where [thing] is drawn now, in the tab.
  Offset where(int thing) {
    final ws = WsScope.read(context), p = wsShows(ws.inside!, thing, ws.frame / Ws.fps);
    return _screen(p.x, p.y);
  }

  // ---- what the document says, now

  WsMove? _picked(Ws ws) => ws.inside == null || int.tryParse(ws.selected ?? '') == null ? null : wsMoveOf(ws.inside!, int.parse(ws.selected!));

  List<(int, Color, String, bool)> _things(Ws ws) => [
    for (final l in ws.layers)
      if (int.tryParse(l.id) case final t? when ws.inside != null && !const {'behaviour', 'camera', 'light'}.contains(ws.inside!.word(t, 'kind'))) (t, l.tone, l.name, ws.inside!.word(t, 'kind') == 'text'),
  ];

  Offset _seat(Motolii m, int t, double at) => _screen(m.value(t, 'x', at) ?? 0, m.value(t, 'y', at) ?? 0);

  // ---- the hand

  void _down(Ws ws, PointerDownEvent e) {
    if (e.buttons != kPrimaryButton) return;
    setState(() => _stroke
      ..clear()
      ..add((e.localPosition, e.timeStamp)));
  }

  void _move(Ws ws, PointerMoveEvent e) {
    if (_stroke.isEmpty) return;
    setState(() => _stroke.add((e.localPosition, e.timeStamp)));
    if (_lane.contains(_stroke.first.$1) && _picked(ws) == null) _scrub(ws, e.localPosition.dx);
  }

  void _scrub(Ws ws, double x) {
    ws.playing = false;
    ws.frame = (_laneT(x) * Ws.fps).round();
  }

  void _up(Ws ws) {
    if (_stroke.isEmpty) return;
    final raw = [..._stroke], pts = [for (final p in raw) p.$1];
    _stroke.clear();
    final m = ws.inside;
    if (m == null) return setState(() {});
    final box = _bounds(pts), a = pts.first, z = pts.last;
    Color? said;
    if (box.longestSide < 6) {
      _tap(ws, a);
    } else if (_lane.contains(a)) {
      said = _when(ws, a, z);
    } else {
      final turn = _turns(pts), mv = _picked(ws), now = ws.frame / Ws.fps;
      final closed = (z - a).distance < math.max(14, .25 * _length(pts)), round = closed && turn.net.abs() > 1.4 * math.pi && turn.net.abs() < 2.9 * math.pi;
      final starts = mv != null && mv.who.any((t) => (a - where(t)).distance < 14 || (a - _ghost(m, mv, t)).distance < 14);
      if (mv != null && round && _gather(ws, m, mv, box)) {
        said = wsLabel('${mv.by}');
      } else if (starts && (turn.net.abs() >= 2.9 * math.pi || turn.flips >= 3)) {
        _hold(ws, () => wsMoveSet(ws, mv.by, 'scatter', (box.longestSide / 2 / _k).clamp(0.0, 2.0)));
        said = wsLabel('${mv.by}');
      } else if (starts && _flicked(pts)) {
        _hold(ws, () => wsMoveSet(ws, mv.by, 'opacity', mv.end['opacity']! > .5 ? 0 : 1));
        said = wsLabel('${mv.by}');
      } else if (starts) {
        _out(ws, m, mv, raw, now);
        said = wsLabel('${mv.by}');
      } else if (closed || (z - a).distance < .5 * box.longestSide) {   // (round some: a move of them, from now)
        final path = Path()..addPolygon(pts, true);
        final who = [for (final (t, _, _, _) in _things(ws)) if (path.contains(where(t))) '$t'];
        if (who.isNotEmpty) {
          wsMakeMove(ws, who, (now * 10).round() / 10);
          said = WsT.accent;
        }
      }
    }
    setState(() {
      _last = pts;
      _said = said;
    });
  }

  void _hold(Ws ws, void Function() said) {
    wsMoveHold(ws, true);
    said();
    wsMoveHold(ws, false);
  }

  Offset _ghost(Motolii m, WsMove mv, int t) {
    final g = wsShows(m, t, mv.away);
    return _screen(g.x, g.y);
  }

  /// A ring round the group's middle, smaller than the group: gathered in as much.
  bool _gather(Ws ws, Motolii m, WsMove mv, Rect ring) {
    if (mv.who.length < 2) return false;
    final seats = _bounds([for (final t in mv.who) _seat(m, t, mv.away)]), seen = _bounds([for (final t in mv.who) where(t)]), wide = seats.longestSide;
    final off = math.min((ring.center - seats.center).distance, (ring.center - seen.center).distance);   // (round where they settle, or where they show now)
    if (ring.longestSide > .9 * wide || off > math.max(24, ring.longestSide)) return false;
    _hold(ws, () => wsMoveSet(ws, mv.by, 'spread', (ring.longestSide / wide).clamp(0.0, 1.0)));
    return true;
  }

  /// A stroke out from one of them: where they come from (that way, or out of the thing it ends on), and how it was
  /// drawn: fast is short, slowing at the end eases out, past the end and back overshoots.
  void _out(Ws ws, Motolii m, WsMove mv, List<(Offset, Duration)> raw, double now) {
    final a = raw.first.$1, z = raw.last.$1, at = mv.away;
    int? onto;
    for (final (t, _, _, _) in _things(ws)) {
      if (!mv.who.contains(t) && (z - where(t)).distance < 14) onto = t;
    }
    final len = _length([for (final p in raw) p.$1]), took = (raw.last.$2 - raw.first.$2).inMicroseconds / 1e6;
    final dir = z - a, reach = math.max(dir.distance, 1e-3);
    final far = raw.fold(0.0, (f, p) => math.max(f, ((p.$1 - a).dx * dir.dx + (p.$1 - a).dy * dir.dy) / reach));
    final over = (far - reach) / reach > .04 ? ((far - reach) / reach).clamp(0.0, .6) : 0.0;
    var out = mv.out, length = mv.length;
    if (took > .03 && len > 10) {
      final speed = len / took;
      length = (240 / speed).clamp(.15, 3.0);
      var run = 0.0, i = 0;
      for (; i < raw.length - 1 && run < .7 * len; i++) {
        run += (raw[i + 1].$1 - raw[i].$1).distance;
      }
      final rest = (raw.last.$2 - raw[i].$2).inMicroseconds / 1e6;
      if (rest > 0) out = (1 - (len - run) / rest / speed).clamp(0.0, 1.0);
    }
    wsMoveHold(ws, true);
    if (onto != null) {
      final cx = mv.who.fold(0.0, (s, t) => s + (m.value(t, 'x', at) ?? 0)) / mv.who.length, cy = mv.who.fold(0.0, (s, t) => s + (m.value(t, 'y', at) ?? 0)) / mv.who.length;
      wsMoveSet(ws, mv.by, 'x', (m.value(onto, 'x', at) ?? 0) - cx);
      wsMoveSet(ws, mv.by, 'y', (m.value(onto, 'y', at) ?? 0) - cy);
      wsMoveSet(ws, mv.by, 'spread', 0);
    } else {
      wsMoveSet(ws, mv.by, 'x', dir.dx / _k);
      wsMoveSet(ws, mv.by, 'y', -dir.dy / _k);
    }
    if (length != mv.length) wsMoveTime(ws, mv, mv.start, length);
    wsMoveEase(ws, mv, out, over);
    wsMoveHold(ws, false);
  }

  /// Along the strip: the picked move starts where it begins, and its things are as far apart as its length over them.
  Color? _when(Ws ws, Offset a, Offset z) {
    final mv = _picked(ws);
    if (mv == null) return null;
    final start = _laneT(a.dx), lag = mv.who.length > 1 ? math.max(0.0, (_laneT(z.dx) - start) / (mv.who.length - 1)) : mv.lag;
    wsMoveHold(ws, true);
    wsMoveTime(ws, mv, start, mv.length);
    wsMoveSet(ws, mv.by, 'lag', lag);
    wsMoveHold(ws, false);
    return wsLabel('${mv.by}');
  }

  void _tap(Ws ws, Offset s) {
    final m = ws.inside!;
    if (_lane.contains(s)) return _scrub(ws, s.dx);
    for (final o in wsMoves(ws).reversed) {
      for (final t in o.who) {
        if ((s - _ghost(m, o, t)).distance < 14 || (s - where(t)).distance < 14) return ws.select('${o.by}');
      }
    }
    ws.select(null);
  }

  // ---- reading a stroke: its box, its length, how much and which way it turns, a flick at its end

  static Rect _bounds(List<Offset> pts) => pts.skip(1).fold(Rect.fromCenter(center: pts.first, width: 0, height: 0), (r, p) => r.expandToInclude(Rect.fromCenter(center: p, width: 0, height: 0)));

  static double _length(List<Offset> pts) => [for (var i = 1; i < pts.length; i++) (pts[i] - pts[i - 1]).distance].fold(0.0, (a, b) => a + b);

  static List<Offset> _even(List<Offset> pts, double step) {   // (the stroke again, a point each [step]: a slow hand's crowd of points no louder than a fast one's)
    final out = [pts.first];
    for (final p in pts.skip(1)) {
      while ((p - out.last).distance >= step) {
        out.add(out.last + (p - out.last) / (p - out.last).distance * step);
      }
    }
    return out;
  }

  static ({double net, int flips}) _turns(List<Offset> pts) {
    final e = _even(pts, 8);
    var net = 0.0, flips = 0, side = 0;
    for (var i = 2; i < e.length; i++) {
      final a = e[i - 1] - e[i - 2], b = e[i] - e[i - 1];
      final t = math.atan2(a.dx * b.dy - a.dy * b.dx, a.dx * b.dx + a.dy * b.dy);
      net += t;
      if (t.abs() > .6) {
        if (side != 0 && t.sign != side) flips++;
        side = t.sign.toInt();
      }
    }
    return (net: net, flips: flips);
  }

  static bool _flicked(List<Offset> pts) {   // (mostly up, its last fifth turned off to a side)
    final e = _even(pts, 4);
    if (e.length < 8) return false;
    final c = (e.length * .8).floor(), body = e[c] - e.first, tail = e.last - e[c];
    if (body.dy > -24 || body.dy.abs() < 1.5 * body.dx.abs() || tail.distance < 6) return false;
    final cos = (body.dx * tail.dx + body.dy * tail.dy) / body.distance / tail.distance;
    return cos < .7;
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: Listenable.merge([WsScope.read(context), wsPosesTold]),   // (and drawn again when the inside tells where something is at another moment: moves.dart wsShows)
    builder: (context, _) {
      final ws = WsScope.read(context), m = ws.inside;
      if (m == null) return const SizedBox.expand();
      return LayoutBuilder(builder: (context, box) {
        final s = box.biggest, half = Size(WsArt.comp.width / 2 / m.point, WsArt.comp.height / 2 / m.point);
        _lane = Rect.fromLTWH(0, s.height - 40, s.width, 40);
        _stage = Rect.fromLTWH(0, 0, s.width, s.height - 40);
        _k = math.min(_stage.width / (half.width * 3.2), _stage.height / (half.height * 3.2));   // (the camera's frame and as much again round it)
        _o = _stage.center;
        return Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: (e) => _down(ws, e),
          onPointerMove: (e) => _move(ws, e),
          onPointerUp: (_) => _up(ws),
          onPointerCancel: (_) => setState(_stroke.clear),
          child: CustomPaint(size: s, painter: _Paint(this, ws, m, half)),
        );
      });
    },
  );
}

class _Paint extends CustomPainter {
  _Paint(this.h, this.ws, this.m, this.half);
  final TabDrawState h;
  final Ws ws;
  final Motolii m;
  final Size half;

  @override
  void paint(Canvas c, Size size) {
    c.drawRect(Offset.zero & size, Paint()..color = Grey.g10);
    final frame = Rect.fromPoints(h._screen(-half.width, half.height), h._screen(half.width, -half.height));
    c.drawRect(frame, Paint()..color = Grey.g20);
    c.drawRect(frame, Paint()..color = Grey.g38..style = PaintingStyle.stroke);
    final now = ws.frame / Ws.fps, mv = h._picked(ws);

    if (mv != null) {   // (the other end: each thing's ghost, its way to where it settles)
      final ink = wsLabel('${mv.by}');
      for (final (k, t) in mv.who.indexed) {
        final g = wsShows(m, t, mv.away), at = h._screen(g.x, g.y), home = wsShows(m, t, mv.coming ? mv.last + 1e-3 : mv.start - 1e-3), seat = h._screen(home.x, home.y);
        final (a, z) = mv.coming ? (at, seat) : (seat, at);
        c.drawLine(a, z, Paint()..color = ink.withValues(alpha: .35));
        final dir = z - a, n = dir.distance;
        if (n > 12) {
          final u = dir / n, side = Offset(-u.dy, u.dx);
          c.drawPath(Path()..addPolygon([z, z - u * 6 + side * 3, z - u * 6 - side * 3], true), Paint()..color = ink.withValues(alpha: .6));
        }
        final r = Rect.fromCenter(center: at, width: 14 * g.size.clamp(.1, 3) * math.cos(g.flip * math.pi / 180).abs().clamp(.1, 1), height: 10 * g.size.clamp(.1, 3));
        c.drawRect(r, Paint()..color = ink.withValues(alpha: .4 * g.opacity.clamp(0, 1)));
        c.drawRect(r, Paint()..color = ink..style = PaintingStyle.stroke);
        if (k == 0 || k == mv.who.length - 1) _text(c, '${k + 1}', at + const Offset(9, -9), ink, 8);
      }
    }

    final of = {for (final o in wsMoves(ws)) for (final t in o.who) t: o.by};
    for (final (t, tone, name, text) in h._things(ws)) {   // (where each shows now: out of the frame too, clear ones as an outline; a move's things in its ink)
      final p = wsShows(m, t, now), at = h._screen(p.x, p.y), by = of[t];
      final r = Rect.fromCenter(center: at, width: (text ? 34 : 16) * p.size.clamp(.1, 3) * math.cos(p.flip * math.pi / 180).abs().clamp(.1, 1), height: 12 * p.size.clamp(.1, 3));
      c.drawRect(r, Paint()..color = tone.withValues(alpha: p.opacity.clamp(0, 1)));
      c.drawRect(r, Paint()..color = by == null ? tone : wsLabel('$by').withValues(alpha: by == mv?.by ? 1 : .5)..style = PaintingStyle.stroke..strokeWidth = by == mv?.by ? 1.5 : 1);
      if (text) _text(c, name, at + const Offset(0, 14), Grey.g63, 9);
    }

    if (h._last.length > 1) c.drawPath(Path()..addPolygon(h._last, false), Paint()..color = (h._said ?? Grey.g56).withValues(alpha: .3)..style = PaintingStyle.stroke..strokeWidth = 2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
    if (h._stroke.length > 1) c.drawPath(Path()..addPolygon([for (final p in h._stroke) p.$1], false), Paint()..color = WsT.accent..style = PaintingStyle.stroke..strokeWidth = 2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);

    // the strip of time: each move's things as ticks, the picked one's in full
    final l = h._lane;
    c.drawRect(l, Paint()..color = Grey.g13);
    for (var s = 0; s <= Ws.duration / Ws.fps; s++) {
      c.drawLine(Offset(h._laneX(s.toDouble()), l.top + 3), Offset(h._laneX(s.toDouble()), l.top + 8), Paint()..color = Grey.g38);
    }
    for (final o in wsMoves(ws)) {
      final on = o.by == mv?.by, ink = wsLabel('${o.by}'), y = on ? l.top + 26 : l.top + 15;
      for (var k = 0; k < o.who.length; k++) {
        final x = h._laneX(o.start + o.lag * k);
        c.drawLine(Offset(x, y), Offset(h._laneX(o.start + o.lag * k + o.length), y), Paint()..color = ink.withValues(alpha: on ? .6 : .25)..strokeWidth = on ? 3 : 1.5);
        c.drawCircle(Offset(x, y), on ? 3 : 2, Paint()..color = ink);
      }
    }
    final x = h._laneX(now);
    c.drawLine(Offset(x, l.top), Offset(x, l.bottom), Paint()..color = WsT.accent..strokeWidth = 1.5);
  }

  void _text(Canvas c, String s, Offset at, Color ink, double size) {
    final tp = TextPainter(text: TextSpan(text: s, style: TextStyle(color: ink, fontSize: size, height: 1)), textDirection: TextDirection.ltr)..layout();
    tp.paint(c, at - Offset(tp.width / 2, tp.height / 2));
  }

  @override
  bool shouldRepaint(_Paint o) => true;
}
