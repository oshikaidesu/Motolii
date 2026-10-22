// Relations tab, way B "the stagger curve" (Cavalry's stagger graph, FL Studio's strum): time is the picture. Each
// move a staircase of bars, a row a thing in its order, each bar its thing's stretch filled with the ease (the picked
// move full, the rest faint; tap one to pick it). Grab a bar: the whole staircase in time; the last step: the steps
// spread or packed; a bar's right end: how long; the curve at its foot: up for more ease, over its top for overshoot;
// the arrow at its head: coming in or going out. Above, the other end as tiles drawn as what they do (tap: on or off;
// drag across: how much). Things in no move wait as chips at the left: sweep down over some, a move at the playhead.
// Joined to the document through moves.dart.
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import '../../workspace/moves.dart';
import '../../workspace/ws.dart';

class TabCurve extends StatefulWidget {
  const TabCurve({super.key});
  @override
  State<TabCurve> createState() => _TabCurveState();
}

/// A tile of the other end: the value it sets, as it comes (off), when tapped on (on), and how far a drag takes it.
typedef _Tile = ({String look, String name, double off, double on, double lo, double hi});

const List<_Tile> _tiles = [
  (look: 'rise', name: 'y', off: 0, on: -.15, lo: -1, hi: 0),
  (look: 'gather', name: 'spread', off: 1, on: 0, lo: 0, hi: 1),
  (look: 'burst', name: 'spread', off: 1, on: 2, lo: 1, hi: 4),
  (look: 'scatter', name: 'scatter', off: 0, on: .5, lo: 0, hi: 2),
  (look: 'fade', name: 'opacity', off: 1, on: 0, lo: 0, hi: 1),
  (look: 'pop', name: 'size', off: 1, on: 0, lo: 0, hi: 1),
  (look: 'flip', name: 'rotate_y', off: 0, on: 180, lo: 0, hi: 180),
];

double _far(_Tile t) => t.on > t.off ? t.hi : t.lo;
double _amount(_Tile t, double v) => ((v - t.off) / (_far(t) - t.off)).clamp(0.0, 1.0);

enum _Grip { none, sweep, scrub, start, lag, length, ease, tile }

class _TabCurveState extends State<TabCurve> {
  // (where things sit: the tiles on top, a ruler, the chips' column at the left, the graph beside it)
  static const strip = 40.0, ruler = 14.0, column = 26.0, pitch = 14.0, faint = 6.0, gap = 8.0;

  _Grip _grip = _Grip.none;
  Offset _down = Offset.zero;
  WsMove? _held;   // (the move as it was when grabbed)
  int _tile = 0;
  double _v0 = 0;
  double? _startAt, _lengthAt, _easeAt;   // (shown where the hand has it, told when let go)
  final _swept = <String>[];

  late Size _size;
  Rect get _graph => Rect.fromLTRB(column, strip + ruler, _size.width - 8, _size.height);
  double get _secs => Ws.duration / Ws.fps;
  double _x(double t) => _graph.left + t / _secs * _graph.width;
  double _t(double x) => ((x - _graph.left) / _graph.width * _secs).clamp(0, _secs);
  double get _pps => _graph.width / _secs;
  double _frame(double t) => (t * Ws.fps).round() / Ws.fps;

  Rect _tileRect(int i) {
    final w = (_size.width - 4) / _tiles.length;
    return Rect.fromLTWH(4 + i * w, 4, w - 4, strip - 8);
  }

  // ---- what the document says, now

  WsMove? _picked(Ws ws) => ws.inside == null || int.tryParse(ws.selected ?? '') == null ? null : wsMoveOf(ws.inside!, int.parse(ws.selected!));

  /// Things in no move, in the layers' order: the chips waiting at the left.
  List<(String, Color)> _free(Ws ws, List<WsMove> moves) {
    final m = ws.inside, taken = {for (final mv in moves) ...mv.who};
    return [
      for (final l in ws.layers)
        if (int.tryParse(l.id) case final t? when m != null && !taken.contains(t) && !const {'behaviour', 'camera', 'light'}.contains(m.word(t, 'kind'))) (l.id, l.tone),
    ];
  }

  Offset _chip(int i) => Offset(column / 2, _graph.top + 4 + i * 16 + 7);

  /// Each move's block: its top and its row's height (the picked one full).
  List<(WsMove, double, double)> _blocks(Ws ws, List<WsMove> moves) {
    final out = <(WsMove, double, double)>[];
    var y = _graph.top + 4;
    for (final mv in moves) {
      final p = '${mv.by}' == ws.selected ? pitch : faint;
      out.add((mv, y, p));
      y += p * mv.who.length + gap;
    }
    return out;
  }

  // (the picked move as the hand has it now)
  double _start(WsMove mv) => _startAt ?? mv.start;
  double _length(WsMove mv) => _lengthAt ?? mv.length;
  double _ease(WsMove mv) => _easeAt ?? (mv.over > 0 ? 1 + mv.over : mv.out);

  Rect _bar(WsMove mv, double top, double p, int k) {
    final a = _start(mv) + mv.lag * k;
    return Rect.fromLTRB(_x(a), top + k * p + 1, _x(a + _length(mv)), top + (k + 1) * p - 2);
  }

  /// The ease curve: at the foot of the staircase, right of the last step.
  Rect _curveBox(WsMove mv, double top, double p) {
    final last = _bar(mv, top, p, mv.who.length - 1), x = math.min(last.right + 6, _graph.right - 22);
    return Rect.fromLTWH(x, last.top - 4, 22, 22);
  }

  /// The turn: an arrow at the head of the staircase.
  Offset _turn(WsMove mv, double top, double p) => Offset(math.max(_x(_start(mv)) - 10, _graph.left + 7), top + p / 2);

  // ---- the hand

  void _scrub(Ws ws, double x) {
    ws.playing = false;
    ws.frame = (_t(x) * Ws.fps).round();
  }

  void _press(Ws ws, Offset s) {
    _down = s;
    _swept.clear();
    final moves = wsMoves(ws), mv = _picked(ws);
    _held = mv;
    if (s.dy < strip) {
      for (var i = 0; i < _tiles.length; i++) {
        if (mv != null && _tileRect(i).contains(s)) {
          _grip = _Grip.tile;
          _tile = i;
          _v0 = mv.end[_tiles[i].name]!;
          wsMoveHold(ws, true);
          return;
        }
      }
      _grip = _Grip.none;
      return;
    }
    if (s.dx < column) {
      final free = _free(ws, moves), i = ((s.dy - _graph.top - 4) / 16).floor();
      if (i >= 0 && i < free.length) {
        _grip = _Grip.sweep;
        _sweep(ws, s);
        return;
      }
    }
    for (final (o, top, p) in _blocks(ws, moves)) {
      if (o.by != mv?.by) continue;
      if (_curveBox(o, top, p).inflate(2).contains(s)) {
        _grip = _Grip.ease;
        _v0 = _ease(o);
        return;
      }
      for (var k = 0; k < o.who.length; k++) {
        final r = _bar(o, top, p, k);
        if (s.dy < r.top - 1 || s.dy > r.bottom + 2 || s.dx < r.left - 4 || s.dx > r.right + 6) continue;
        if ((s.dx - r.right).abs() < 6) {
          _grip = _Grip.length;
        } else if (k == o.who.length - 1 && k > 0) {
          _grip = _Grip.lag;
          _v0 = o.lag;
          wsMoveHold(ws, true);
        } else {
          _grip = _Grip.start;
        }
        return;
      }
    }
    _grip = _Grip.scrub;
    _scrub(ws, s.dx);
  }

  void _sweep(Ws ws, Offset s) {
    final free = _free(ws, wsMoves(ws));
    int at(double y) => ((y - _graph.top - 4) / 16).floor().clamp(0, free.length - 1);
    final a = at(_down.dy), b = at(s.dy);
    setState(() => _swept
      ..clear()
      ..addAll([for (var i = a; a <= b ? i <= b : i >= b; a <= b ? i++ : i--) free[i].$1]));   // (the order swept: down or up)
  }

  void _drag(Ws ws, Offset s) {
    final mv = _held, dx = s.dx - _down.dx;
    switch (_grip) {
      case _Grip.sweep:
        _sweep(ws, s);
      case _Grip.scrub:
        _scrub(ws, s.dx);
      case _Grip.start when mv != null:
        setState(() => _startAt = _frame(math.max(0, mv.start + dx / _pps)));
      case _Grip.length when mv != null:
        setState(() => _lengthAt = _frame(math.max(.05, mv.length + dx / _pps)));
      case _Grip.lag when mv != null:
        wsMoveSet(ws, mv.by, 'lag', math.max(0, _v0 + dx / _pps / (mv.who.length - 1)));
      case _Grip.ease:
        setState(() => _easeAt = (_v0 - (s.dy - _down.dy) / 50).clamp(0.0, 1.6));
      case _Grip.tile when mv != null:
        final t = _tiles[_tile], step = (t.hi - t.lo) / 120 * (t.on > t.off ? 1 : -1);
        wsMoveSet(ws, mv.by, t.name, (_v0 + dx * step).clamp(t.lo, t.hi));
      default:
        break;
    }
  }

  void _let(Ws ws) {
    final mv = _held;
    switch (_grip) {
      case _Grip.sweep when _swept.isNotEmpty:
        wsMakeMove(ws, [..._swept], _frame(ws.frame / Ws.fps));
      case _Grip.start || _Grip.length when mv != null && (_startAt != null || _lengthAt != null):
        wsMoveTime(ws, mv, _start(mv), _length(mv));
      case _Grip.ease when mv != null && _easeAt != null:
        wsMoveEase(ws, mv, math.min(1, _easeAt!), math.max(0, _easeAt! - 1));
      case _Grip.lag || _Grip.tile:
        wsMoveHold(ws, false);
      default:
        break;
    }
    setState(() {
      _grip = _Grip.none;
      _held = null;
      _swept.clear();
      _startAt = _lengthAt = _easeAt = null;
    });
  }

  void _tap(Ws ws, Offset s) {
    final mv = _picked(ws), moves = wsMoves(ws);
    if (s.dy < strip) {
      for (var i = 0; i < _tiles.length; i++) {
        if (mv == null || !_tileRect(i).contains(s)) continue;
        final t = _tiles[i], on = _amount(t, mv.end[t.name]!) > 1e-3;
        wsMoveHold(ws, true);
        wsMoveSet(ws, mv.by, t.name, on ? t.off : t.on);
        wsMoveHold(ws, false);
      }
      return;
    }
    if (s.dx < column) return;
    for (final (o, top, p) in _blocks(ws, moves)) {
      if (o.by == mv?.by && (s - _turn(o, top, p)).distance < 9) return wsMoveTurn(ws, o);
      if (s.dy >= top && s.dy < top + p * o.who.length && o.by != mv?.by) return ws.select('${o.by}');
    }
    _scrub(ws, s.dx);
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context);
      if (ws.inside == null) return const SizedBox.expand();
      return LayoutBuilder(builder: (context, box) {
        _size = box.biggest;
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          dragStartBehavior: DragStartBehavior.down,
          onTapUp: (e) => _tap(ws, e.localPosition),
          onPanStart: (e) => _press(ws, e.localPosition),
          onPanUpdate: (e) => _drag(ws, e.localPosition),
          onPanEnd: (_) => _let(ws),
          onPanCancel: () => _let(ws),
          child: CustomPaint(size: _size, painter: _Paint(this, ws)),
        );
      });
    },
  );
}

class _Paint extends CustomPainter {
  _Paint(this.h, this.ws);
  final _TabCurveState h;
  final Ws ws;

  @override
  void paint(Canvas c, Size size) {
    c.drawRect(Offset.zero & size, Paint()..color = Grey.g10);
    final moves = wsMoves(ws), mv = h._picked(ws), g = h._graph;

    // the tiles: each drawn as what it does to the things, its amount a line at its foot
    for (final (i, t) in _tiles.indexed) {
      final r = h._tileRect(i), v = mv?.end[t.name] ?? t.off, a = _amount(t, v), on = a > 1e-3;
      final ink = mv == null ? Grey.g26 : on ? wsLabel('${mv.by}') : Grey.g56;
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), Paint()..color = on ? Grey.g20 : Grey.g13);
      _look(c, t.look, r.deflate(5), ink, mv == null ? .5 : math.max(a, .5));
      if (on) c.drawLine(Offset(r.left + 3, r.bottom - 2), Offset(r.left + 3 + (r.width - 6) * a, r.bottom - 2), Paint()..color = ink..strokeWidth = 2);
    }

    // the ruler and the seconds
    c.drawRect(Rect.fromLTRB(0, _TabCurveState.strip, size.width, g.top), Paint()..color = Grey.g13);
    for (var s = 0; s <= h._secs; s++) {
      final x = h._x(s.toDouble());
      c.drawLine(Offset(x, g.top - 5), Offset(x, g.top), Paint()..color = Grey.g38);
      c.drawLine(Offset(x, g.top), Offset(x, g.bottom), Paint()..color = Grey.g13);
    }
    c.drawLine(Offset(_TabCurveState.column, g.top), Offset(_TabCurveState.column, g.bottom), Paint()..color = Grey.g20);

    // the chips waiting: things in no move
    for (final (i, (id, tone)) in h._free(ws, moves).indexed) {
      final r = Rect.fromCenter(center: h._chip(i), width: 16, height: 10), swept = h._swept.contains(id);
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(2)), Paint()..color = tone.withValues(alpha: swept ? 1 : .35));
      if (swept) c.drawRRect(RRect.fromRectAndRadius(r.inflate(1.5), const Radius.circular(3)), Paint()..color = WsT.accent..style = PaintingStyle.stroke);
    }

    // the staircases
    for (final (o, top, p) in h._blocks(ws, moves)) {
      final on = o.by == mv?.by, ink = wsLabel('${o.by}'), way = wsMoveWay(on ? math.min(1, h._ease(o)) : o.out, on ? math.max(0, h._ease(o) - 1) : o.over).split(' ').skip(1).map(double.parse).toList();
      final tones = {for (final l in ws.layers) l.id: l.tone};
      for (var k = 0; k < o.who.length; k++) {
        final r = on ? h._bar(o, top, p, k) : Rect.fromLTRB(h._x(o.start + o.lag * k), top + k * p, h._x(o.start + o.lag * k + o.length), top + (k + 1) * p - 1);
        final tone = tones['${o.who[k]}'] ?? ink;
        if (!on) {
          c.drawRect(r, Paint()..color = ink.withValues(alpha: .3));
          continue;
        }
        c.drawRect(r, Paint()..color = Grey.g15);
        final fill = Path()..moveTo(r.left, r.bottom);   // (how much of it is at its seat, through the stretch)
        for (var u = 0.0; u <= 1.0001; u += 1 / 16) {
          final f = _bezier(way, u.clamp(0, 1)), at = o.coming ? f : 1 - f;
          fill.lineTo(r.left + r.width * u, r.bottom - r.height * at.clamp(0, 1.6) / 1.6 * 1.25);
        }
        fill
          ..lineTo(r.right, r.bottom)
          ..close();
        c.save();
        c.clipRect(Rect.fromLTRB(r.left - .5, r.top - 4, r.right + .5, r.bottom + .5));
        c.drawPath(fill, Paint()..color = tone.withValues(alpha: .75));
        c.restore();
        c.drawRect(r, Paint()..color = ink..style = PaintingStyle.stroke);
        c.drawLine(Offset(r.right, r.top - 1), Offset(r.right, r.bottom + 1), Paint()..color = ink..strokeWidth = 2);   // (its right end: how long)
        if (k == o.who.length - 1 && k > 0) c.drawCircle(Offset(r.left, r.center.dy), 3.5, Paint()..color = ink);   // (the last step: pull to spread)
      }
      if (!on) continue;
      final t = h._turn(o, top, p), dir = o.coming ? 1.0 : -1.0;
      c.drawPath(Path()..addPolygon([t + Offset(5 * dir, 0), t + Offset(-4 * dir, -5), t + Offset(-4 * dir, 5)], true), Paint()..color = ink);
      final b = h._curveBox(o, top, p), e = h._ease(o), seat = b.top + b.height * .3;
      c.drawRect(b, Paint()..color = Grey.g13);
      c.drawLine(Offset(b.left, seat), Offset(b.right, seat), Paint()..color = Grey.g38);   // (the seat: past it is overshoot)
      final curve = Path()..moveTo(b.left, b.bottom);
      for (var u = 0.0; u <= 1.0001; u += 1 / 16) {
        curve.lineTo(b.left + b.width * u, b.bottom - (b.bottom - seat) * _bezier(way, u.clamp(0, 1)));
      }
      c.drawPath(curve, Paint()..color = e > 1 ? WsT.accent : ink..style = PaintingStyle.stroke..strokeWidth = 1.5);
    }

    final x = h._x(ws.frame / Ws.fps);
    c.drawLine(Offset(x, _TabCurveState.strip), Offset(x, g.bottom), Paint()..color = WsT.accent..strokeWidth = 1.5);
    c.drawPath(Path()..addPolygon([Offset(x - 4, _TabCurveState.strip), Offset(x + 4, _TabCurveState.strip), Offset(x, _TabCurveState.strip + 6)], true), Paint()..color = WsT.accent);
  }

  /// A tile's picture: three dots doing what the tile does to the things (faint where they come from, full at their seats).
  static void _look(Canvas c, String look, Rect r, Color ink, double a) {
    final dot = Paint()..color = ink, ghost = Paint()..color = ink.withValues(alpha: .3), line = Paint()..color = ink.withValues(alpha: .4)..strokeWidth = 1;
    final m = r.center, seats = [m + const Offset(-8, 0), m, m + const Offset(8, 0)];
    switch (look) {
      case 'rise':
        for (final s in seats) {
          final from = s + Offset(0, r.height / 2 * a);
          c.drawLine(from, s, line);
          c.drawCircle(from, 1.8, ghost);
          c.drawCircle(s + Offset(0, -r.height / 4), 2.2, dot);
        }
      case 'gather' || 'burst':
        for (var i = 0; i < 6; i++) {
          final d = Offset(math.cos(i * math.pi / 3), math.sin(i * math.pi / 3)) * (r.height / 2) * a, out = look == 'burst';
          c.drawLine(m, m + d, line);
          c.drawCircle(out ? m : m + d, 1.6, ghost);
          if (out) c.drawCircle(m + d, 2, dot);
        }
        if (look == 'gather') c.drawCircle(m, 3, dot);
      case 'scatter':
        const jit = [Offset(-2, 3), Offset(3, -2), Offset(-3, -3), Offset(2, 2), Offset(0, -3), Offset(-2, 1)];
        for (var i = 0; i < 6; i++) {
          final s = m + Offset((i % 3 - 1) * 8.0, (i ~/ 3 - .5) * 8);
          c.drawCircle(s, 1.4, ghost);
          c.drawCircle(s + jit[i] * a * 1.4, 2, dot);
        }
      case 'fade':
        for (final (i, s) in seats.indexed) {
          c.drawCircle(s, 3, Paint()..color = ink.withValues(alpha: (.15 + .42 * i) * a + (1 - a)));
        }
      case 'pop':
        for (final (i, s) in seats.indexed) {
          c.drawCircle(s, 1 + 1.3 * i * a + 2.6 * (1 - a), dot);
        }
      case 'flip':
        for (final (i, s) in seats.indexed) {
          final w = 6 * math.cos(i * math.pi / 4 * a).abs().clamp(.15, 1.0);
          c.drawRect(Rect.fromCenter(center: s, width: w, height: 9), i == 2 ? dot : ghost);
        }
    }
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

  @override
  bool shouldRepaint(_Paint o) => true;
}
