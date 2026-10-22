// A group's moves by hand, as the code that makes them reads (for each thing in order: start = when + its place in
// the order x the gap; each value from its seat to one other end, eased): who and their order by drawing round them,
// the other end as the members' ghosts (pull them, spread or gather them, fade, flip, scatter), when as one tick a
// member on a strip of time, how as beads on the way. No lists, no sliders: only the things and their ghosts.
// Nothing joined to the inside yet: a try of the touch alone.
//   flutter run -d macos --dart-define=MOTOLII_HANDS=true
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../tokens.dart';
import '../workspace/ws.dart';

const _world = Size(1600, 800), _frame = Rect.fromLTWH(360, 132, 880, 495), _length = 10.0;   // (the pasteboard, the camera's frame on it, the film's seconds)

enum _Kind { cube, photo, letter }

class _Thing {
  _Thing(this.kind, this.seat, this.label, this.ink);
  final _Kind kind;
  final Offset seat;
  final String label;
  final Color ink;
}

/// A thing as it is at a moment.
class _Pose {
  const _Pose(this.at, this.size, this.opacity, this.flip);
  final Offset at;
  final double size, opacity, flip;
  static _Pose lerp(_Pose a, _Pose b, double w) => _Pose(Offset.lerp(a.at, b.at, w)!, a.size + (b.size - a.size) * w, a.opacity + (b.opacity - a.opacity) * w, a.flip + (b.flip - a.flip) * w);
}

/// One group's move: who in order, when, the other end, how.
class _Move {
  _Move(this.who, this.start);
  final List<int> who;
  double start, gap = .08, length = .6;
  bool coming = true;   // (from the other end to the seats; else from the seats out)
  Offset shift = const Offset(0, 80);
  double spread = 1, fade = 1, flip = 0, scatter = 0, ease = .8;   // (ease: how far along the way at half the time)

  Offset centre(List<_Thing> all) => who.fold(Offset.zero, (s, i) => s + all[i].seat) / who.length.toDouble();

  _Pose end(List<_Thing> all, int i) {
    final c = centre(all), r = math.Random(i * 7919), a = r.nextDouble() * 2 * math.pi, d = 120 + r.nextDouble() * 160;
    final at = c + (all[i].seat - c) * spread + shift + Offset(math.cos(a), math.sin(a)) * d * scatter;
    return _Pose(at, spread < 1 ? .15 + .85 * spread : 1, 1 - fade, flip * math.pi);
  }

  double curve(double u) {
    if (ease <= .97) return math.pow(u, math.log(ease) / math.log(.5)).toDouble();
    final s = math.max(0.0, 8 * (ease - .875)), v = u - 1;   // (past the seat and back)
    return 1 + (s + 1) * v * v * v + s * v * v;
  }

  /// How far each member is towards the other end at [t].
  double weight(int k, double t) {
    final e = curve(((t - start - k * gap) / length).clamp(0.0, 1.0));
    return coming ? 1 - e : e;
  }
}

class GroupHands extends StatefulWidget {
  const GroupHands({super.key});
  @override
  State<GroupHands> createState() => _GroupHandsState();
}

enum _Grip { none, lasso, move, spread, fade, flip, scatter, ease, first, last, scrub }

class _GroupHandsState extends State<GroupHands> with SingleTickerProviderStateMixin {
  late final List<_Thing> _things = _scene();
  final _moves = <_Move>[];
  _Move? _picked;
  double _t = 0;
  bool _playing = true;
  late final Ticker _ticker;
  Duration _last = Duration.zero;
  final _focus = FocusNode();

  // (what the hand holds now, and the stroke it draws)
  _Grip _grip = _Grip.none;
  final _stroke = <Offset>[];
  Offset _was = Offset.zero;

  static List<_Thing> _scene() {
    final c = _frame.center;
    return [
      _Thing(_Kind.cube, c, '', const Color(0xFFE5432D)),
      for (var i = 0; i < 8; i++) _Thing(_Kind.photo, c + Offset(math.sin(i / 8 * 2 * math.pi) * 320, -math.cos(i / 8 * 2 * math.pi) * 170), '', wsLabel('${i + 2}')),
      for (final (i, ch) in 'TURN IN TIME'.split('').indexed)
        if (ch != ' ') _Thing(_Kind.letter, Offset(560 + i * 44.0, 668), ch, const Color(0xFF151515)),   // (below the frame: out of the camera)
    ];
  }

  @override
  void initState() {
    super.initState();
    _ticker = createTicker((now) {
      final dt = (now - _last).inMicroseconds / 1e6;
      _last = now;
      if (_playing && _grip != _Grip.scrub) setState(() => _t = (_t + dt) % _length);
    })..start();
  }

  @override
  void dispose() {
    _ticker.dispose();
    _focus.dispose();
    super.dispose();
  }

  _Pose _pose(int j, double t) {
    final s = _things[j].seat;
    var p = _Pose(s, 1, 1, 0);
    for (final m in [..._moves]..sort((a, b) => a.start.compareTo(b.start))) {
      final k = m.who.indexOf(j);
      if (k >= 0) p = _Pose.lerp(p, m.end(_things, j), m.weight(k, t));
    }
    return p;
  }

  // ---- where things are on the screen

  late Rect _stage, _lane;
  double get _k => _stage.width / _world.width;
  Offset _screen(Offset w) => _stage.topLeft + w * _k;
  Offset _worldOf(Offset s) => (s - _stage.topLeft) / _k;
  double _laneX(double t) => _lane.left + 16 + t / _length * (_lane.width - 32);
  double _laneT(double x) => ((x - _lane.left - 16) / (_lane.width - 32) * _length).clamp(0, _length);

  /// The picked move's ghosts' box (on the screen) and its grips.
  Rect _box(_Move m) {
    final ps = [for (final i in m.who) _screen(m.end(_things, i).at)];
    var r = Rect.fromPoints(ps.first, ps.first);
    for (final p in ps) {
      r = r.expandToInclude(Rect.fromPoints(p, p));
    }
    return Rect.fromCenter(center: r.center, width: math.max(r.width + 70, 80), height: math.max(r.height + 70, 80));
  }

  Map<_Grip, Offset> _grips(_Move m) {
    final b = _box(m);
    return {
      _Grip.spread: b.bottomRight,
      _Grip.fade: b.topCenter,
      _Grip.flip: b.topRight + const Offset(0, -22),
      _Grip.scatter: b.centerLeft + const Offset(-22, 0),
      _Grip.ease: _easeBead(m),
    };
  }

  (Offset, Offset) _way(_Move m) {
    final i = m.who.first, a = _screen(m.end(_things, i).at), b = _screen(_things[i].seat);
    return m.coming ? (a, b) : (b, a);
  }

  Offset _easeBead(_Move m) {
    final (a, b) = _way(m);
    return Offset.lerp(a, b, m.ease)!;
  }

  // ---- the hand

  void _down(Offset s) {
    _was = s;
    _stroke.clear();
    final m = _picked;
    if (_lane.contains(s)) {
      if (m != null && (s - Offset(_laneX(m.start), _lane.top + 46)).distance < 14) {
        _grip = _Grip.first;
      } else if (m != null && m.who.length > 1 && (s - Offset(_laneX(m.start + (m.who.length - 1) * m.gap), _lane.top + 46)).distance < 14) {
        _grip = _Grip.last;
      } else {
        _grip = _Grip.scrub;
        _t = _laneT(s.dx);
      }
      return;
    }
    if (m != null) {
      for (final e in _grips(m).entries) {
        if ((s - e.value).distance < 16) {
          _grip = e.key;
          return;
        }
      }
      if (_box(m).contains(s)) {
        _grip = _Grip.move;
        return;
      }
    }
    _grip = _Grip.lasso;
    _stroke.add(_worldOf(s));
  }

  void _drag(Offset s) {
    final d = s - _was, m = _picked;
    _was = s;
    setState(() {
      switch (_grip) {
        case _Grip.lasso:
          _stroke.add(_worldOf(s));
        case _Grip.scrub:
          _t = _laneT(s.dx);
        case _Grip.first when m != null:
          m.start = _laneT(_laneX(m.start) + d.dx);
        case _Grip.last when m != null:
          m.gap = math.max(0, (_laneT(s.dx) - m.start) / (m.who.length - 1));
        case _Grip.move when m != null:
          m.shift += d / _k;
          final c = m.centre(_things) + m.shift;   // (laid on a thing that is not in the group: gathered into it)
          for (final (i, t) in _things.indexed) {
            if (!m.who.contains(i) && (t.seat - c).distance < 24) m.shift = t.seat - m.centre(_things);
          }
        case _Grip.spread when m != null:
          final o = _screen(m.centre(_things) + m.shift), a = (s - d - o).distance, b = (s - o).distance;
          if (a > 4) m.spread = (m.spread * b / a).clamp(0.0, 4.0);
        case _Grip.fade when m != null:
          m.fade = (m.fade + d.dy / 120).clamp(0.0, 1.0);
        case _Grip.flip when m != null:
          m.flip = (m.flip + d.dx / 160).clamp(0.0, 1.0);
        case _Grip.scatter when m != null:
          m.scatter = (m.scatter - d.dx / 160).clamp(0.0, 1.5);
        case _Grip.ease when m != null:
          final (a, b) = _way(m);
          final ab = b - a, l2 = ab.dx * ab.dx + ab.dy * ab.dy;
          if (l2 > 1) m.ease = (((s - a).dx * ab.dx + (s - a).dy * ab.dy) / l2).clamp(.05, 1.3);
        default:
          break;
      }
    });
  }

  void _up() {
    if (_grip == _Grip.lasso && _stroke.length > 4) {
      final path = Path()..addPolygon(_stroke, true);
      final o = _stroke.first, far = _stroke.reduce((a, b) => (b - o).distance > (a - o).distance ? b : a), d = far - o;
      double along(Offset p) => (p - o).dx * d.dx + (p - o).dy * d.dy;   // (the order the stroke went: from where it began towards its far end)

      final who = [for (final (i, t) in _things.indexed) if (path.contains(t.seat)) i]..sort((a, b) => along(_things[a].seat).compareTo(along(_things[b].seat)));
      if (who.isNotEmpty) {
        final m = _Move(who, (_t * 10).round() / 10);
        _moves.add(m);
        _picked = m;
      }
    }
    setState(() {
      _grip = _Grip.none;
      _stroke.clear();
    });
  }

  /// A tap: the arrow turns the way round; a ghost, a member or a tick picks its move; elsewhere lets go.
  void _tap(Offset s) {
    final m = _picked;
    if (m != null && (s - _way(m).$2).distance < 16) {
      setState(() => m.coming = !m.coming);
      return;
    }
    if (_lane.contains(s)) return;
    for (final mv in _moves.reversed) {
      for (final i in mv.who) {
        if ((s - _screen(mv.end(_things, i).at)).distance < 26 || (s - _screen(_pose(i, _t).at)).distance < 26) {
          setState(() => _picked = mv);
          return;
        }
      }
    }
    setState(() => _picked = null);
  }

  KeyEventResult _key(FocusNode _, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    if (e.logicalKey == LogicalKeyboardKey.space) {
      setState(() => _playing = !_playing);
      return KeyEventResult.handled;
    }
    if ((e.logicalKey == LogicalKeyboardKey.backspace || e.logicalKey == LogicalKeyboardKey.delete) && _picked != null) {
      setState(() {
        _moves.remove(_picked);
        _picked = null;
      });
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) => Focus(
    focusNode: _focus,
    autofocus: true,
    onKeyEvent: _key,
    child: LayoutBuilder(builder: (context, box) {
      final s = box.biggest, laneH = 84.0, k = math.min(s.width / _world.width, (s.height - laneH - 12) / _world.height);
      _stage = Rect.fromLTWH((s.width - _world.width * k) / 2, 0, _world.width * k, _world.height * k);
      _lane = Rect.fromLTWH(0, s.height - laneH, s.width, laneH);
      return Listener(
        onPointerDown: (e) {
          _focus.requestFocus();
          _down(e.localPosition);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapUp: (e) => _tap(e.localPosition),
          onPanUpdate: (e) => _drag(e.localPosition),
          onPanEnd: (_) => _up(),
          onPanCancel: _up,
          child: CustomPaint(size: s, painter: _Paint(this)),
        ),
      );
    }),
  );
}

class _Paint extends CustomPainter {
  _Paint(this.h);
  final _GroupHandsState h;

  @override
  void paint(Canvas c, Size size) {
    c.drawRect(Offset.zero & size, Paint()..color = Grey.g07);
    final frame = Rect.fromPoints(h._screen(_frame.topLeft), h._screen(_frame.bottomRight));
    c.drawRect(h._stage, Paint()..color = Grey.g15);
    c.drawRect(frame, Paint()..color = const Color(0xFFF1ECE2));
    final m = h._picked;

    // the things, as they are now (out of the frame they are still drawn, the frame's surround veils them)
    for (var j = 0; j < h._things.length; j++) {
      _thing(c, h._things[j], h._pose(j, h._t), member: m?.who.contains(j) ?? false);
    }
    c.save();
    c.clipPath(Path.combine(ui.PathOperation.difference, Path()..addRect(h._stage), Path()..addRect(frame)));
    c.drawRect(h._stage, Paint()..color = Grey.g07.withValues(alpha: .55));
    c.restore();

    // the picked move: its ghosts (the other end), its box and grips, the way with its beads
    if (m != null) {
      final ink = wsLabel('${h._moves.indexOf(m) + 1}');
      for (final (k, i) in m.who.indexed) {
        final g = m.end(h._things, i);
        _ghost(c, h._things[i], g, ink);
        final p = h._screen(g.at);
        c.drawCircle(p + const Offset(14, -14), 8, Paint()..color = ink);
        _text(c, '${k + 1}', p + const Offset(14, -14), Grey.g07, 9);
      }
      final b = h._box(m), line = Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 1;
      c.drawRRect(RRect.fromRectAndRadius(b, const Radius.circular(8)), line);
      final (a, z) = h._way(m);
      c.drawLine(a, z, line..color = ink.withValues(alpha: .6));
      for (var u = .1; u < .95; u += .1) {
        c.drawCircle(Offset.lerp(a, z, m.curve(u))!, 2.5, Paint()..color = ink);
      }
      _arrow(c, a, z, ink);
      final g = h._grips(m);
      c.drawCircle(g[_Grip.ease]!, 7, Paint()..color = const Color(0xFFFFFFFF));
      c.drawCircle(g[_Grip.ease]!, 7, line..color = ink..strokeWidth = 2);
      c.drawRect(Rect.fromCenter(center: g[_Grip.spread]!, width: 12, height: 12), Paint()..color = ink);   // (the corner: spread out, gather in)
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: g[_Grip.fade]!, width: 34, height: 8), const Radius.circular(4)), Paint()..color = Color.lerp(ink, Grey.g07, m.fade * .8)!);   // (a shade: pulled down, they come in from clear)
      _flipGlyph(c, g[_Grip.flip]!, ink, m.flip);
      _dice(c, g[_Grip.scatter]!, ink, m.scatter);
    }

    // the lasso being drawn
    if (h._stroke.length > 1) {
      c.drawPath(Path()..addPolygon([for (final p in h._stroke) h._screen(p)], false), Paint()..color = WsT.accent..style = PaintingStyle.stroke..strokeWidth = 1.5);
    }

    _lane(c, size);
  }

  void _thing(Canvas c, _Thing t, _Pose p, {required bool member}) {
    final at = h._screen(p.at), k = h._k * p.size;
    c.save();
    c.translate(at.dx, at.dy);
    c.scale(math.cos(p.flip), 1);
    final a = p.opacity.clamp(0.0, 1.0);
    switch (t.kind) {
      case _Kind.cube:
        c.drawRect(Rect.fromCenter(center: Offset.zero, width: 70 * k, height: 70 * k), Paint()..color = t.ink.withValues(alpha: a));
      case _Kind.photo:
        final r = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 96 * k, height: 66 * k), Radius.circular(6 * k));
        c.drawRRect(r, Paint()..color = (math.cos(p.flip) < 0 ? Color.lerp(t.ink, Grey.g07, .5)! : t.ink).withValues(alpha: a));
      case _Kind.letter:
        _text(c, t.label, Offset.zero, t.ink.withValues(alpha: a), 40 * k, bold: true);
    }
    if (member) {
      c.drawRect(Rect.fromCenter(center: Offset.zero, width: (t.kind == _Kind.letter ? 40 : 100) * h._k, height: (t.kind == _Kind.letter ? 48 : 70) * h._k), Paint()..color = WsT.accent..style = PaintingStyle.stroke);
    }
    c.restore();
  }

  void _ghost(Canvas c, _Thing t, _Pose g, Color ink) {
    final at = h._screen(g.at), k = h._k * g.size;
    final r = switch (t.kind) {
      _Kind.letter => Rect.fromCenter(center: at, width: 30 * k, height: 40 * k),
      _Kind.cube => Rect.fromCenter(center: at, width: 70 * k, height: 70 * k),
      _Kind.photo => Rect.fromCenter(center: at, width: 96 * k * math.cos(g.flip).abs().clamp(.08, 1), height: 66 * k),
    };
    c.drawRect(r, Paint()..color = t.ink.withValues(alpha: .35 * g.opacity));   // (as clear as it will be there; the outline always)
    c.drawRect(r, Paint()..color = ink..style = PaintingStyle.stroke);
  }

  void _arrow(Canvas c, Offset a, Offset z, Color ink) {
    final d = z - a, n = d.distance;
    if (n < 1) return;
    final u = d / n, side = Offset(-u.dy, u.dx);
    c.drawPath(Path()..addPolygon([z, z - u * 12 + side * 6, z - u * 12 - side * 6], true), Paint()..color = ink);
  }

  void _flipGlyph(Canvas c, Offset p, Color ink, double v) {
    c.drawCircle(p, 9, Paint()..color = Grey.g07);
    c.drawArc(Rect.fromCircle(center: p, radius: 9), -math.pi / 2, math.max(.15, v) * 2 * math.pi, false, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 2);
  }

  void _dice(Canvas c, Offset p, Color ink, double v) {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: p, width: 18, height: 18), const Radius.circular(4)), Paint()..color = ink.withValues(alpha: .3 + .7 * (v / 1.5)));
    for (final o in const [Offset(-4, -4), Offset(4, 4), Offset(0, 0)]) {
      c.drawCircle(p + o, 1.6, Paint()..color = Grey.g07);
    }
  }

  void _lane(Canvas c, Size size) {
    final l = h._lane;
    c.drawRect(l, Paint()..color = Grey.g13);
    for (var s = 0; s <= _length; s++) {
      final x = h._laneX(s.toDouble());
      c.drawLine(Offset(x, l.top + 4), Offset(x, l.top + 12), Paint()..color = Grey.g38);
    }
    for (final (n, m) in h._moves.indexed) {
      final on = m == h._picked, ink = wsLabel('${n + 1}'), y = on ? l.top + 46 : l.top + 24 + (n % 3) * 8;
      for (final (k, i) in m.who.indexed) {
        final s = m.start + k * m.gap, x = h._laneX(s);
        c.drawLine(Offset(x, y), Offset(h._laneX(s + m.length), y), Paint()..color = h._things[i].ink.withValues(alpha: on ? .7 : .25)..strokeWidth = on ? 3 : 1.5);
        if (on && (k == 0 || k == m.who.length - 1)) {
          c.drawCircle(Offset(x, y), 7, Paint()..color = ink);
          _arrowTick(c, Offset(x, y), k == 0);
        } else {
          c.drawCircle(Offset(x, y), on ? 3 : 2, Paint()..color = ink.withValues(alpha: on ? 1 : .5));
        }
        if (on) c.drawLine(Offset(x, y + 8), Offset(x, y + 8 + 10.0 * ((k % 2) + 1)), Paint()..color = h._things[i].ink);
      }
    }
    final x = h._laneX(h._t);
    c.drawLine(Offset(x, l.top), Offset(x, l.bottom), Paint()..color = WsT.accent..strokeWidth = 1.5);
  }

  void _arrowTick(Canvas c, Offset p, bool first) {
    final d = first ? const Offset(-3, 0) : const Offset(3, 0);
    c.drawPath(Path()..addPolygon([p + d * 1.2, p - d + const Offset(0, -3), p - d + const Offset(0, 3)], true), Paint()..color = Grey.g07);
  }

  void _text(Canvas c, String s, Offset at, Color ink, double size, {bool bold = false}) {
    final tp = TextPainter(text: TextSpan(text: s, style: TextStyle(color: ink, fontSize: size, fontWeight: bold ? FontWeight.w800 : FontWeight.w600, height: 1)), textDirection: TextDirection.ltr)..layout();
    tp.paint(c, at - Offset(tp.width / 2, tp.height / 2));
  }

  @override
  bool shouldRepaint(_Paint o) => true;
}

WidgetbookComponent groupHandsSet() => WidgetbookComponent(name: 'Group by hand', useCases: [WidgetbookUseCase(name: 'Try', builder: (c) => const GroupHands())]);
