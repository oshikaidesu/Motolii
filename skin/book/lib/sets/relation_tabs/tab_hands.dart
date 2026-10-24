// Relations tab, way A "by hand": every thing of the document seen whole (out of the camera too), a group made by
// drawing round some, its other end as their ghosts with grips (pull: away; corner: gather in, spread out; top: clear;
// round: turn over; dice: scatter; laid on a thing: into it), a strip of time below (a tick a thing: the first moves
// it in time, the last spreads them apart), the way from the ghost to the seat with a bead for its ease (past the
// seat: overshoot), its arrow turning coming in to going out. Joined to the document through moves.dart.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../motolii.dart';
import '../../tokens.dart';
import '../../workspace/moves.dart';
import '../../workspace/stage_art.dart' show WsArt;
import '../../workspace/ws.dart';

class TabHands extends StatefulWidget {
  const TabHands({super.key});
  @override
  State<TabHands> createState() => _TabHandsState();
}

enum _Grip { none, lasso, move, spread, fade, flip, scatter, ease, first, last, scrub }

class _TabHandsState extends State<TabHands> {
  _Grip _grip = _Grip.none;
  final _stroke = <Offset>[];
  Offset _was = Offset.zero;
  double? _easeAt, _startAt;   // (a bead or a first tick being dragged: shown where the hand has it, told when let go)

  late Rect _stage, _lane;
  late double _k;
  late Offset _o;
  Offset _screen(double x, double y) => _o + Offset(x, -y) * _k;
  double _laneX(double t) => _lane.left + 12 + t / (Ws.duration / Ws.fps) * (_lane.width - 24);
  double _laneT(double x) => ((x - _lane.left - 12) / (_lane.width - 24) * Ws.duration / Ws.fps).clamp(0, Ws.duration / Ws.fps);

  // ---- what the document says, now

  WsMove? _picked(Ws ws) => ws.inside == null || int.tryParse(ws.selected ?? '') == null ? null : wsMoveOf(ws.inside!, int.parse(ws.selected!));

  List<(int, Color, String, bool)> _things(Ws ws) => [
    for (final l in ws.layers)
      if (int.tryParse(l.id) case final t? when ws.inside != null && !const {'behaviour', 'camera', 'light'}.contains(ws.inside!.word(t, 'kind'))) (t, l.tone, l.name, ws.inside!.word(t, 'kind') == 'text'),
  ];

  Offset _seat(Motolii m, int t, double now) => _screen(wsValue(m, t, 'x', now, 0), wsValue(m, t, 'y', now, 0));

  Rect _box(Motolii m, WsMove mv) {
    var r = Rect.zero;
    for (final (k, t) in mv.who.indexed) {
      final g = wsShows(m, t, mv.away), p = _screen(g.x, g.y), one = Rect.fromCenter(center: p, width: 1, height: 1);
      r = k == 0 ? one : r.expandToInclude(one);
    }
    return Rect.fromCenter(center: r.center, width: math.max(r.width + 40, 56), height: math.max(r.height + 40, 56));
  }

  (Offset, Offset) _way(Motolii m, WsMove mv) {
    final t = mv.who.first, g = wsShows(m, t, mv.away), a = _screen(g.x, g.y), b = _seat(m, t, mv.away);
    return mv.coming ? (a, b) : (b, a);
  }

  double _bead(WsMove mv) => _easeAt ?? (mv.over > 0 ? 1 + mv.over / 1.5 : .5 + .5 * mv.out);

  Map<_Grip, Offset> _grips(Motolii m, WsMove mv) {
    final b = _box(m, mv);
    final (a, z) = _way(m, mv);
    return {
      _Grip.spread: b.bottomRight,
      _Grip.fade: b.topCenter,
      _Grip.flip: b.topRight + const Offset(0, -16),
      _Grip.scatter: b.centerLeft + const Offset(-16, 0),
      _Grip.ease: Offset.lerp(a, z, _bead(mv))!,
    };
  }

  // ---- the hand

  void _down(Ws ws, Offset s) {
    _was = s;
    _stroke.clear();
    final m = ws.inside, mv = _picked(ws);
    if (m == null) return;
    if (_lane.contains(s)) {
      final y = _lane.top + 34;
      if (mv != null && (s - Offset(_laneX(mv.start), y)).distance < 12) {
        _grip = _Grip.first;
      } else if (mv != null && mv.who.length > 1 && (s - Offset(_laneX(mv.start + mv.lag * (mv.who.length - 1)), y)).distance < 12) {
        _grip = _Grip.last;
        wsMoveHold(ws, true);
      } else {
        _grip = _Grip.scrub;
        _scrub(ws, s.dx);
      }
      return;
    }
    if (mv != null) {
      for (final e in _grips(m, mv).entries) {
        if ((s - e.value).distance < 12) {
          _grip = e.key;
          if (_grip != _Grip.ease) wsMoveHold(ws, true);
          return;
        }
      }
      if (_box(m, mv).contains(s)) {
        _grip = _Grip.move;
        wsMoveHold(ws, true);
        return;
      }
    }
    _grip = _Grip.lasso;
    _stroke.add(s);
  }

  void _scrub(Ws ws, double x) {
    ws.playing = false;
    ws.frame = (_laneT(x) * Ws.fps).round();
  }

  void _drag(Ws ws, Offset s) {
    final d = s - _was, m = ws.inside, mv = _picked(ws);
    _was = s;
    if (m == null) return;
    if (_grip == _Grip.lasso) return setState(() => _stroke.add(s));
    if (_grip == _Grip.scrub) return _scrub(ws, s.dx);
    if (mv == null) return;
    double e(String n) => mv.end[n]!;
    switch (_grip) {
      case _Grip.first:
        setState(() => _startAt = _laneT(s.dx));
      case _Grip.last:
        wsMoveSet(ws, mv.by, 'lag', math.max(0, (_laneT(s.dx) - mv.start) / (mv.who.length - 1)));
      case _Grip.move:
        var x = e('x') + d.dx / _k, y = e('y') - d.dy / _k;
        final now = mv.away, seats = [for (final t in mv.who) (wsValue(m, t, 'x', now, 0), wsValue(m, t, 'y', now, 0))];
        final cx = seats.fold(0.0, (a, p) => a + p.$1) / seats.length, cy = seats.fold(0.0, (a, p) => a + p.$2) / seats.length;
        for (final (t, _, _, _) in _things(ws)) {   // (laid on a thing that is not in the group: gathered into it)
          if (mv.who.contains(t)) continue;
          final tx = wsValue(m, t, 'x', now, 0), ty = wsValue(m, t, 'y', now, 0);
          if (((cx + x - tx) * _k).abs() < 10 && ((cy + y - ty) * _k).abs() < 10) (x, y) = (tx - cx, ty - cy);
        }
        wsMoveSet(ws, mv.by, 'x', x);
        wsMoveSet(ws, mv.by, 'y', y);
      case _Grip.spread:
        final o = _box(m, mv).center, a = (s - d - o).distance, b = (s - o).distance;
        if (a > 4) wsMoveSet(ws, mv.by, 'spread', (e('spread') * b / a).clamp(0.0, 4.0));
      case _Grip.fade:
        wsMoveSet(ws, mv.by, 'opacity', (e('opacity') - d.dy / 80).clamp(0.0, 1.0));
      case _Grip.flip:
        wsMoveSet(ws, mv.by, 'rotate_y', (e('rotate_y') + d.dx).clamp(0.0, 180.0));
      case _Grip.scatter:
        wsMoveSet(ws, mv.by, 'scatter', (e('scatter') - d.dx / _k).clamp(0.0, 2.0));
      case _Grip.ease:
        final (a, z) = _way(m, mv);
        final az = z - a, l2 = az.dx * az.dx + az.dy * az.dy;
        if (l2 > 1) setState(() => _easeAt = (((s - a).dx * az.dx + (s - a).dy * az.dy) / l2).clamp(.5, 1.4));
      default:
        break;
    }
  }

  void _up(Ws ws) {
    final mv = _picked(ws);
    switch (_grip) {
      case _Grip.lasso when _stroke.length > 4:
        final path = Path()..addPolygon(_stroke, true), m = ws.inside!, now = ws.frame / Ws.fps;
        final who = [for (final (t, _, _, _) in _things(ws)) if (path.contains(_seat(m, t, now))) '$t'];
        if (who.isNotEmpty) wsMakeMove(ws, who, (now * 10).round() / 10);
      case _Grip.first when mv != null && _startAt != null:
        wsMoveTime(ws, mv, _startAt!, mv.length);
      case _Grip.ease when mv != null && _easeAt != null:
        final f = _easeAt!;
        wsMoveEase(ws, mv, f >= 1 ? 1 : (f - .5) / .5, f > 1 ? (f - 1) * 1.5 : 0);
      case _Grip.move || _Grip.spread || _Grip.fade || _Grip.flip || _Grip.scatter || _Grip.last:
        wsMoveHold(ws, false);
      default:
        break;
    }
    setState(() {
      _grip = _Grip.none;
      _stroke.clear();
      _easeAt = _startAt = null;
    });
  }

  void _tap(Ws ws, Offset s) {
    final m = ws.inside, mv = _picked(ws);
    if (m == null || _lane.contains(s)) return;
    if (mv != null && (s - _way(m, mv).$2).distance < 12) return wsMoveTurn(ws, mv);
    for (final o in wsMoves(ws).reversed) {
      for (final t in o.who) {
        final g = wsShows(m, t, o.away), now = wsShows(m, t, ws.frame / Ws.fps);
        if ((s - _screen(g.x, g.y)).distance < 14 || (s - _screen(now.x, now.y)).distance < 14) return ws.select('${o.by}');
      }
    }
    ws.select(null);
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: Listenable.merge([WsScope.read(context), wsPosesTold]),   // (and drawn again when the inside tells where something is at another moment: moves.dart wsShows)
    builder: (context, _) {
      final ws = WsScope.read(context), m = ws.inside;
      if (m == null) return const SizedBox.expand();
      return LayoutBuilder(builder: (context, box) {
        final s = box.biggest, half = Size(WsArt.comp.width / 2 / m.point, WsArt.comp.height / 2 / m.point);
        _lane = Rect.fromLTWH(0, s.height - 56, s.width, 56);
        _stage = Rect.fromLTWH(0, 0, s.width, s.height - 56);
        _k = math.min(_stage.width / (half.width * 3.2), _stage.height / (half.height * 3.2));   // (the camera's frame and as much again round it)
        _o = _stage.center;
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapUp: (e) => _tap(ws, e.localPosition),
          onPanStart: (e) => _down(ws, e.localPosition),
          onPanUpdate: (e) => _drag(ws, e.localPosition),
          onPanEnd: (_) => _up(ws),
          onPanCancel: () => _up(ws),
          child: CustomPaint(size: s, painter: _Paint(this, ws, m, half)),
        );
      });
    },
  );
}

class _Paint extends CustomPainter {
  _Paint(this.h, this.ws, this.m, this.half);
  final _TabHandsState h;
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

    for (final (t, tone, name, text) in h._things(ws)) {   // (where each shows now: out of the frame too, clear ones as an outline)
      final p = wsShows(m, t, now), at = h._screen(p.x, p.y), member = mv?.who.contains(t) ?? false;
      final r = Rect.fromCenter(center: at, width: (text ? 34 : 16) * p.size.clamp(.1, 3) * math.cos(p.flip * math.pi / 180).abs().clamp(.1, 1), height: 12 * p.size.clamp(.1, 3));
      c.drawRect(r, Paint()..color = tone.withValues(alpha: p.opacity.clamp(0, 1)));
      c.drawRect(r, Paint()..color = member ? WsT.accent : tone..style = PaintingStyle.stroke);
      if (text) _text(c, name, at + const Offset(0, 14), Grey.g63, 9);
    }

    if (mv != null && mv.who.isNotEmpty) {
      final ink = wsLabel('${mv.by}'), line = Paint()..color = ink..style = PaintingStyle.stroke;
      for (final (k, t) in mv.who.indexed) {
        final g = wsShows(m, t, mv.away), at = h._screen(g.x, g.y);
        final r = Rect.fromCenter(center: at, width: 14 * g.size.clamp(.1, 3) * math.cos(g.flip * math.pi / 180).abs().clamp(.1, 1), height: 10 * g.size.clamp(.1, 3));
        c.drawRect(r, Paint()..color = ink.withValues(alpha: .4 * g.opacity.clamp(0, 1)));
        c.drawRect(r, line);
        if (k == 0 || k == mv.who.length - 1) _text(c, '${k + 1}', at + const Offset(9, -9), ink, 8);
      }
      c.drawRRect(RRect.fromRectAndRadius(h._box(m, mv), const Radius.circular(6)), line);
      final (a, z) = h._way(m, mv);
      c.drawLine(a, z, Paint()..color = ink.withValues(alpha: .5));
      final way = wsMoveWay(mv.out, mv.over).split(' ').skip(1).map(double.parse).toList();
      for (var u = .1; u < .95; u += .1) {
        c.drawCircle(Offset.lerp(a, z, _bezier(way, u))!, 2, Paint()..color = ink);
      }
      final dir = z - a, n = math.max(dir.distance, 1e-3), uu = dir / n, side = Offset(-uu.dy, uu.dx);
      c.drawPath(Path()..addPolygon([z, z - uu * 9 + side * 5, z - uu * 9 - side * 5], true), Paint()..color = ink);
      final g = h._grips(m, mv);
      c.drawCircle(g[_Grip.ease]!, 6, Paint()..color = const Color(0xFFFFFFFF));
      c.drawCircle(g[_Grip.ease]!, 6, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 2);
      c.drawRect(Rect.fromCenter(center: g[_Grip.spread]!, width: 10, height: 10), Paint()..color = ink);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: g[_Grip.fade]!, width: 26, height: 6), const Radius.circular(3)), Paint()..color = Color.lerp(Grey.g10, ink, mv.end['opacity']!.clamp(.15, 1))!);
      c.drawCircle(g[_Grip.flip]!, 7, Paint()..color = Grey.g10);
      c.drawArc(Rect.fromCircle(center: g[_Grip.flip]!, radius: 7), -math.pi / 2, math.max(.15, mv.end['rotate_y']! / 180) * 2 * math.pi, false, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 2);
      final dice = g[_Grip.scatter]!;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: dice, width: 14, height: 14), const Radius.circular(3)), Paint()..color = ink.withValues(alpha: .3 + .7 * (mv.end['scatter']! / 2).clamp(0, 1)));
      for (final o in const [Offset(-3, -3), Offset(3, 3), Offset.zero]) {
        c.drawCircle(dice + o, 1.3, Paint()..color = Grey.g10);
      }
    }
    if (h._stroke.length > 1) c.drawPath(Path()..addPolygon(h._stroke, false), Paint()..color = WsT.accent..style = PaintingStyle.stroke..strokeWidth = 1.5);

    // the strip of time: each move's things as ticks, the picked one's in full
    final l = h._lane;
    c.drawRect(l, Paint()..color = Grey.g13);
    for (var s = 0; s <= Ws.duration / Ws.fps; s++) {
      c.drawLine(Offset(h._laneX(s.toDouble()), l.top + 3), Offset(h._laneX(s.toDouble()), l.top + 9), Paint()..color = Grey.g38);
    }
    for (final o in wsMoves(ws)) {
      final on = o.by == mv?.by, ink = wsLabel('${o.by}'), y = on ? l.top + 34 : l.top + 20, start = on ? (h._startAt ?? o.start) : o.start;
      for (var k = 0; k < o.who.length; k++) {
        final x = h._laneX(start + o.lag * k);
        c.drawLine(Offset(x, y), Offset(h._laneX(start + o.lag * k + o.length), y), Paint()..color = ink.withValues(alpha: on ? .6 : .25)..strokeWidth = on ? 3 : 1.5);
        c.drawCircle(Offset(x, y), on && (k == 0 || k == o.who.length - 1) ? 6 : 2.5, Paint()..color = ink);
      }
    }
    final x = h._laneX(now);
    c.drawLine(Offset(x, l.top), Offset(x, l.bottom), Paint()..color = WsT.accent..strokeWidth = 1.5);
  }

  static double _bezier(List<double> p, double u) {   // (how far along at u, for a Bezier of two handles: solved for x, read in y)
    var lo = 0.0, hi = 1.0;
    double at(double s, double a, double b) => 3 * (1 - s) * (1 - s) * s * a + 3 * (1 - s) * s * s * b + s * s * s;
    for (var i = 0; i < 24; i++) {
      final mid = (lo + hi) / 2;
      at(mid, p[0], p[2]) < u ? lo = mid : hi = mid;
    }
    return at((lo + hi) / 2, p[1], p[3]);
  }

  void _text(Canvas c, String s, Offset at, Color ink, double size) {
    final tp = TextPainter(text: TextSpan(text: s, style: TextStyle(color: ink, fontSize: size, height: 1)), textDirection: TextDirection.ltr)..layout();
    tp.paint(c, at - Offset(tp.width / 2, tp.height / 2));
  }

  @override
  bool shouldRepaint(_Paint o) => true;
}
