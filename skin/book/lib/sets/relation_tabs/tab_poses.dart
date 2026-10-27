// Relations tab, way E "two poses" (Keynote's Magic Move, PowerPoint's Morph: you make the two slides, the software
// moves between them): two small maps of every thing of the document (out of the camera too), THERE the other end
// (where they show while the strength is whole, asked of the inside), HOME their seats; between them a fat arrow
// (its way coming in or going out, tapped to turn; its bend the ease, pushed past: overshoot). A sweep in HOME makes a
// move; in THERE the group is pushed about (drag: away; wheel: gathered in, spread out; left edge: clear; dice:
// scatter; round: turned over). Below, five frames from its start to its last (left edge: start; right edge: longer
// with its lag; middle: lag), tapped: that moment. Joined to the document through moves.dart.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

import '../../motolii.dart';
import '../../tokens.dart';
import '../../workspace/moves.dart';
import '../../workspace/stage_art.dart' show WsArt;
import '../../workspace/ws.dart';

class TabPoses extends StatefulWidget {
  const TabPoses({super.key});
  @override
  State<TabPoses> createState() => _TabPosesState();
}

enum _Part { there, arrow, home, strip, ruler }

enum _Grip { none, sweep, move, fade, flip, scatter, ease, first, span, lag, scrub }

/// What the last hand let go did (for a test on another Mac to say why no move was made).
String tabPosesLastUp = '';

class _TabPosesState extends State<TabPoses> {
  _Grip _grip = _Grip.none;
  Offset _from = Offset.zero, _to = Offset.zero;   // (where the hand went down, where it is)
  Map<String, double> _base = const {};   // (the values at the press: a drag tells them plus how far the hand went)
  double? _ease, _startAt, _spanBy;   // (the arrow, the start or the span being dragged: shown where the hand has it, told when let go)
  Timer? _wheelRest;
  Ws? _wheelWs;
  double _wheel = 0;

  late Size _map, _frame, _half;
  late double _k;
  Offset _screen(double x, double y) => Offset(_map.width / 2, _map.height / 2) + Offset(x, -y) * _k;
  static double get _seconds => Ws.duration / Ws.fps;

  @override
  void dispose() {
    if (_wheelRest?.isActive ?? false) {
      _wheelRest!.cancel();
      if (_wheelWs case final ws?) wsMoveHold(ws, false);
    }
    super.dispose();
  }

  // ---- what the document says, now

  WsMove? _picked(Ws ws) => ws.inside == null || int.tryParse(ws.selected ?? '') == null ? null : wsMoveOf(ws.inside!, int.parse(ws.selected!));

  List<(int, Color, bool)> _things(Ws ws) => [
    for (final l in ws.layers)
      if (int.tryParse(l.id) case final t? when ws.inside != null && !const {'behaviour', 'camera', 'light'}.contains(ws.inside!.word(t, 'kind'))) (t, l.tone, ws.inside!.word(t, 'kind') == 'text'),
  ];

  Offset _seat(Motolii m, int t, double now) => _screen(wsValue(m, t, 'x', now, 0), wsValue(m, t, 'y', now, 0));

  double _easeOf(WsMove mv) => _ease ?? (mv.over > 0 ? 1 + mv.over : mv.out);   // (0 even .. 1 slowing hard .. 1.6 past the seat)

  List<double> _moments(WsMove? mv) {
    final a = mv?.start ?? 0, z = mv?.last ?? _seconds;
    return [for (var i = 0; i < 5; i++) a + (z - a) * i / 4];
  }

  // THERE's grips, in its own frame
  Rect get _fadeBar => Rect.fromLTWH(0, 0, 8, _map.height);
  Offset get _flipAt => Offset(_map.width - 12, 12);
  Offset get _diceAt => Offset(_map.width - 12, _map.height - 12);

  // ---- the hand

  void _down(Ws ws, _Part part, Offset s) {
    _from = _to = s;
    final mv = _picked(ws);
    _base = {...?mv?.end, 'lag': mv?.lag ?? 0};
    _grip = switch (part) {
      _Part.home => _Grip.sweep,
      _Part.there when mv == null => _Grip.none,
      _Part.there when _fadeBar.inflate(4).contains(s) => _Grip.fade,
      _Part.there when (s - _flipAt).distance < 12 => _Grip.flip,
      _Part.there when (s - _diceAt).distance < 12 => _Grip.scatter,
      _Part.there => _Grip.move,
      _Part.arrow when mv != null => _Grip.ease,
      _Part.strip when mv != null && s.dx < 10 => _Grip.first,
      _Part.strip when mv != null && s.dx > _frame.width * 5 + 16 - 10 => _Grip.span,
      _Part.strip when mv != null && mv.who.length > 1 => _Grip.lag,
      _Part.ruler => _Grip.scrub,
      _ => _Grip.none,
    };
    if (const {_Grip.move, _Grip.fade, _Grip.flip, _Grip.scatter, _Grip.lag}.contains(_grip)) wsMoveHold(ws, true);
    if (_grip == _Grip.scrub) _scrubTo(ws, _rulerT(s.dx));
    setState(() {});
  }

  void _drag(Ws ws, Offset s) {
    _to = s;
    final d = s - _from, mv = _picked(ws), rate = _seconds / (_frame.width * 5 + 16);   // (a hand's pixel along the strip: as long as one on the ruler)
    if (mv == null && _grip != _Grip.sweep && _grip != _Grip.scrub) return;
    double b(String n) => _base[n]!;
    switch (_grip) {
      case _Grip.sweep:
        setState(() {});
      case _Grip.scrub:
        _scrubTo(ws, _rulerT(s.dx));
      case _Grip.move:
        wsMoveSet(ws, mv!.by, 'x', b('x') + d.dx / _k);
        wsMoveSet(ws, mv.by, 'y', b('y') - d.dy / _k);
      case _Grip.fade:
        wsMoveSet(ws, mv!.by, 'opacity', (b('opacity') - d.dy / _map.height).clamp(0.0, 1.0));
      case _Grip.flip:
        wsMoveSet(ws, mv!.by, 'rotate_y', (b('rotate_y') + d.dx * 1.5).clamp(0.0, 180.0));
      case _Grip.scatter:
        wsMoveSet(ws, mv!.by, 'scatter', (b('scatter') + (d.dx - d.dy) / 60).clamp(0.0, 2.0));
      case _Grip.ease:
        setState(() => _ease = ((mv!.over > 0 ? 1 + mv.over : mv.out) - d.dy / 40).clamp(0.0, 1.6));
      case _Grip.first:
        setState(() => _startAt = math.max(0, mv!.start + d.dx * rate));
      case _Grip.span:
        final span = math.max(.05, mv!.last - mv.start);
        setState(() => _spanBy = math.max(.05, span + d.dx * rate) / span);
      case _Grip.lag:
        wsMoveSet(ws, mv!.by, 'lag', math.max(0, b('lag') + d.dx * rate / (mv.who.length - 1)));
      default:
        break;
    }
  }

  void _up(Ws ws) {
    final mv = _picked(ws), m = ws.inside;
    switch (_grip) {
      case _Grip.sweep when m != null && (_to - _from).distance > 4:
        final box = Rect.fromPoints(_from, _to), now = ws.frame / Ws.fps;
        final who = [for (final (t, _, _) in _things(ws)) if (box.contains(_seat(m, t, now))) '$t'];
        tabPosesLastUp = 'a sweep $box over ${[for (final (t, _, _) in _things(ws)) _seat(m, t, now)]} took $who';
        if (who.isNotEmpty) wsMakeMove(ws, who, (now * 10).round() / 10);
      case _Grip.ease when mv != null && _ease != null:
        wsMoveEase(ws, mv, math.min(1, _ease!), math.max(0, _ease! - 1));
      case _Grip.first when mv != null && _startAt != null:
        wsMoveTime(ws, mv, _startAt!, mv.length);
      case _Grip.span when mv != null && _spanBy != null:
        wsMoveHold(ws, true);   // (its lag and its length, one step)
        if (mv.who.length > 1) wsMoveSet(ws, mv.by, 'lag', mv.lag * _spanBy!);
        wsMoveTime(ws, mv, mv.start, mv.length * _spanBy!);
        wsMoveHold(ws, false);
      case _Grip.move || _Grip.fade || _Grip.flip || _Grip.scatter || _Grip.lag:
        wsMoveHold(ws, false);
      default:
        tabPosesLastUp = 'let go of $_grip from $_from to $_to';
    }
    setState(() {
      _grip = _Grip.none;
      _ease = _startAt = _spanBy = null;
    });
  }

  void _tap(Ws ws, _Part part, Offset s) {
    final m = ws.inside, mv = _picked(ws);
    if (m == null) return;
    switch (part) {
      case _Part.arrow when mv != null:
        wsMoveTurn(ws, mv);
      case _Part.there when mv != null && (s - _flipAt).distance < 12:
        wsMoveSet(ws, mv.by, 'rotate_y', mv.end['rotate_y']! > 90 ? 0 : 180);
      case _Part.home:   // (a thing tapped: the move it is in, the next one if tapped again; nothing: none)
        final now = ws.frame / Ws.fps, at = [for (final (t, _, _) in _things(ws)) if ((s - _seat(m, t, now)).distance < 9) t];
        final holding = [for (final o in wsMoves(ws)) if (o.who.any(at.contains)) o.by];
        if (holding.isEmpty) return ws.select(null);
        ws.select('${holding[(holding.indexOf(mv?.by ?? -1) + 1) % holding.length]}');
      case _Part.strip:
        final i = (s.dx / (_frame.width + 4)).floor().clamp(0, 4);
        _scrubTo(ws, _moments(mv)[i]);
      case _Part.ruler:
        _scrubTo(ws, _rulerT(s.dx));
      default:
        break;
    }
  }

  void _wheelOver(Ws ws, PointerSignalEvent e) {
    final mv = _picked(ws);
    if (e is! PointerScrollEvent || mv == null) return;
    if (!(_wheelRest?.isActive ?? false)) {   // (a turn of the wheel and its rest: one step)
      wsMoveHold(_wheelWs = ws, true);
      _base = {...mv.end};
      _wheel = 0;
    }
    _wheel += e.scrollDelta.dy;
    wsMoveSet(ws, mv.by, 'spread', (_base['spread']! - _wheel / 200).clamp(0.0, 4.0));
    _wheelRest?.cancel();
    _wheelRest = Timer(const Duration(milliseconds: 300), () => wsMoveHold(ws, false));
  }

  double _rulerT(double x) => (x / (_frame.width * 5 + 16) * _seconds).clamp(0, _seconds);

  void _scrubTo(Ws ws, double at) {
    ws.playing = false;
    ws.frame = (at * Ws.fps).round().clamp(0, Ws.duration);
  }

  Widget _part(Ws ws, Motolii m, _Part part, Size size) => Listener(
    onPointerSignal: part == _Part.there ? (e) => _wheelOver(ws, e) : null,
    child: GestureDetector(
      key: ValueKey('poses-${part.name}'),
      behavior: HitTestBehavior.opaque,
      dragStartBehavior: DragStartBehavior.down,   // (a grip is where the hand pressed, not where it was let move)
      onTapUp: (e) => _tap(ws, part, e.localPosition),
      onPanStart: (e) => _down(ws, part, e.localPosition),
      onPanUpdate: (e) => _drag(ws, e.localPosition),
      onPanEnd: (_) => _up(ws),
      onPanCancel: () => _up(ws),
      child: CustomPaint(size: size, painter: _Paint(this, ws, m, part)),
    ),
  );

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: Listenable.merge([WsScope.read(context), wsPosesTold]),   // (and drawn again when the inside tells where something is at another moment: moves.dart wsShows)
    builder: (context, _) {
      final ws = WsScope.read(context), m = ws.inside;
      if (m == null) return const SizedBox.expand();
      return LayoutBuilder(builder: (context, box) {
        final s = box.biggest, half = Size(WsArt.comp.width / 2 / m.point, WsArt.comp.height / 2 / m.point);
        final fw = (s.width - 16) / 5;
        _frame = Size(fw, fw * half.height / half.width);
        const arrow = 52.0, ruler = 16.0;
        final mw = (s.width - arrow) / 2;
        _map = Size(mw, (s.height - _frame.height - ruler - 8).clamp(60.0, mw * .8));
        var ex = half.width, ey = half.height;   // (the camera's frame and every seat, at the film's start: the maps keep still while a hand works)
        for (final (t, _, _) in _things(ws)) {
          ex = math.max(ex, (m.value(t, 'x', 0) ?? 0).abs());
          ey = math.max(ey, (m.value(t, 'y', 0) ?? 0).abs());
        }
        _k = math.min(_map.width / (ex * 2.6), _map.height / (ey * 2.6));
        _half = half;
        return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            _part(ws, m, _Part.there, _map),
            _part(ws, m, _Part.arrow, Size(arrow, _map.height)),
            _part(ws, m, _Part.home, _map),
          ]),
          const SizedBox(height: 8),
          _part(ws, m, _Part.strip, Size(s.width, _frame.height)),
          _part(ws, m, _Part.ruler, Size(s.width, ruler)),
          Expanded(child: Container(color: Grey.g10)),
        ]);
      });
    },
  );
}

class _Paint extends CustomPainter {
  _Paint(this.h, this.ws, this.m, this.part);
  final _TabPosesState h;
  final Ws ws;
  final Motolii m;
  final _Part part;

  @override
  void paint(Canvas c, Size size) {
    c.clipRect(Offset.zero & size);
    c.drawRect(Offset.zero & size, Paint()..color = Grey.g10);
    final mv = h._picked(ws), ink = mv == null ? Grey.g44 : wsLabel('${mv.by}');
    switch (part) {
      case _Part.there:
        _there(c, size, mv, ink);
      case _Part.home:
        _home(c, size, mv, ink);
      case _Part.arrow:
        _arrow(c, size, mv, ink);
      case _Part.strip:
        _strip(c, size, mv, ink);
      case _Part.ruler:
        _ruler(c, size, mv, ink);
    }
  }

  void _camera(Canvas c, Offset o, double k) {
    final frame = Rect.fromCenter(center: o, width: h._half.width * 2 * k, height: h._half.height * 2 * k);
    c.drawRect(frame, Paint()..color = Grey.g20);
    c.drawRect(frame, Paint()..color = Grey.g38..style = PaintingStyle.stroke);
  }

  void _thing(Canvas c, Offset at, double scale, bool text, ({double x, double y, double size, double opacity, double flip}) p, Color fill, Color? edge) {
    final r = Rect.fromCenter(center: at, width: (text ? 22 : 10) * scale * p.size.clamp(.1, 3) * math.cos(p.flip * math.pi / 180).abs().clamp(.1, 1), height: 8 * scale * p.size.clamp(.1, 3));
    c.drawRect(r, Paint()..color = fill.withValues(alpha: fill.a * p.opacity.clamp(0, 1)));
    if (edge != null) c.drawRect(r, Paint()..color = edge..style = PaintingStyle.stroke);
  }

  // THERE: where they show at the other end; the group's members joined in their order
  void _there(Canvas c, Size size, WsMove? mv, Color ink) {
    _camera(c, h._screen(0, 0), h._k);
    final at = mv?.away ?? ws.frame / Ws.fps, line = Path();
    for (final (t, tone, text) in h._things(ws)) {
      final p = wsShows(m, t, at), xy = h._screen(p.x, p.y);
      if (!(mv?.who.contains(t) ?? false)) _thing(c, xy, 1, text, p, tone.withValues(alpha: .3), null);
    }
    if (mv == null) return;
    final words = {for (final (t, _, text) in h._things(ws)) if (text) t};
    for (final (k, t) in mv.who.indexed) {
      final p = wsShows(m, t, at), xy = h._screen(p.x, p.y);
      k == 0 ? line.moveTo(xy.dx, xy.dy) : line.lineTo(xy.dx, xy.dy);
      _thing(c, xy, 1, words.contains(t), p, ink, ink);
    }
    c.drawPath(line, Paint()..color = ink.withValues(alpha: .35)..style = PaintingStyle.stroke);
    if (h._grip == _Grip.move) c.drawCircle(h._to, 3, Paint()..color = WsT.accent);
    // clear: the left edge filled as far as it shows
    final bar = h._fadeBar, o = mv.end['opacity']!.clamp(0.0, 1.0);
    c.drawRect(bar, Paint()..color = Grey.g15);
    c.drawRect(Rect.fromLTRB(bar.left, bar.bottom - bar.height * o, bar.right, bar.bottom), Paint()..color = ink.withValues(alpha: .3 + .7 * o));
    // turned over: a round filled as far as it turns
    final f = h._flipAt;
    c.drawCircle(f, 7, Paint()..color = Grey.g13);
    c.drawArc(Rect.fromCircle(center: f, radius: 7), -math.pi / 2, math.max(.15, mv.end['rotate_y']! / 180) * 2 * math.pi, false, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 2);
    // scattered: a die as solid as they scatter
    final dice = h._diceAt;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: dice, width: 14, height: 14), const Radius.circular(3)), Paint()..color = ink.withValues(alpha: .3 + .7 * (mv.end['scatter']! / 2).clamp(0, 1)));
    for (final p in const [Offset(-3, -3), Offset(3, 3), Offset.zero]) {
      c.drawCircle(dice + p, 1.3, Paint()..color = Grey.g10);
    }
  }

  // HOME: their seats; the group's members ringed; a sweep as its box
  void _home(Canvas c, Size size, WsMove? mv, Color ink) {
    _camera(c, h._screen(0, 0), h._k);
    final now = ws.frame / Ws.fps;
    for (final (t, tone, text) in h._things(ws)) {
      final member = mv?.who.contains(t) ?? false;
      final p = (x: 0.0, y: 0.0, size: wsValue(m, t, 'size', now, 1), opacity: 1.0, flip: 0.0);
      _thing(c, h._seat(m, t, now), 1, text, p, tone, member ? ink : null);
    }
    if (h._grip == _Grip.sweep) {
      final r = Rect.fromPoints(h._from, h._to);
      c.drawRect(r, Paint()..color = WsT.accent.withValues(alpha: .12));
      c.drawRect(r, Paint()..color = WsT.accent..style = PaintingStyle.stroke);
    }
  }

  // the arrow: its way (THERE to HOME coming in), its bend the ease, its head past the end the overshoot
  void _arrow(Canvas c, Size size, WsMove? mv, Color ink) {
    final f = mv == null ? .8 : h._easeOf(mv), cy = size.height / 2;
    final coming = mv?.coming ?? true, a = Offset(coming ? 6 : size.width - 6, cy), z = Offset(coming ? size.width - 14 : 14, cy);
    final bend = Offset((a.dx + z.dx) / 2, cy - 4 - 22 * f.clamp(0, 1)), u = (z - a) / (z - a).distance;
    final tip = z + u * (8 + 16 * math.max(0, f - 1));
    final body = Paint()..color = ink.withValues(alpha: mv == null ? .3 : .8)..style = PaintingStyle.stroke..strokeWidth = 6..strokeCap = StrokeCap.round;
    c.drawPath(Path()..moveTo(a.dx, a.dy)..quadraticBezierTo(bend.dx, bend.dy, z.dx, z.dy), body);
    if (f > 1) c.drawLine(z, tip, body..strokeWidth = 2);
    final side = Offset(-u.dy, u.dx);
    c.drawPath(Path()..addPolygon([tip, tip - u * 10 + side * 8, tip - u * 10 - side * 8], true), Paint()..color = body.color);
    if (mv == null) return;
    final way = wsMoveWay(math.min(1, f), math.max(0, f - 1)).split(' ').skip(1).map(double.parse).toList();
    for (var s = .1; s < .95; s += .1) {   // (where it is at even steps of time: crowded where it slows)
      final q = _bezier(way, s), along = Offset.lerp(a, z, q)!;
      c.drawCircle(Offset(along.dx, cy + 14), 1.6, Paint()..color = ink);
    }
    final grip = (a + bend * 2 + z) / 4;
    c.drawCircle(grip, 5, Paint()..color = const Color(0xFFFFFFFF));
    c.drawCircle(grip, 5, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 2);
  }

  // five frames from the start to the last (the whole film with no move); its edges and its middle are grips
  void _strip(Canvas c, Size size, WsMove? mv, Color ink) {
    final fr = h._frame, k = fr.width / (h._half.width * 2), now = ws.frame / Ws.fps;
    for (final (i, at) in h._moments(mv).indexed) {
      final r = Offset(i * (fr.width + 4), 0) & fr;
      c.save();
      c.clipRect(r);
      c.drawRect(r, Paint()..color = Grey.g20);
      for (final (t, tone, text) in h._things(ws)) {
        final p = wsShows(m, t, at), member = mv?.who.contains(t) ?? false;
        _thing(c, r.center + Offset(p.x, -p.y) * k, .45, text, p, member ? ink : tone.withValues(alpha: .5), null);
      }
      c.restore();
      final on = (now - at).abs() < .5 / Ws.fps;
      c.drawRect(r, Paint()..color = on ? WsT.accent : Grey.g38..style = PaintingStyle.stroke);
    }
    if (mv == null) return;
    final grip = Paint()..color = ink;
    c.drawRect(Rect.fromLTWH(0, 0, 3, size.height), grip);
    c.drawRect(Rect.fromLTWH(fr.width * 5 + 16 - 3, 0, 3, size.height), grip);
  }

  // the whole film: the move as a band, a tick where each member starts, the moment now
  void _ruler(Canvas c, Size size, WsMove? mv, Color ink) {
    final w = h._frame.width * 5 + 16;
    double x(double t) => t / _TabPosesState._seconds * w;
    c.drawRect(Offset.zero & size, Paint()..color = Grey.g13);
    for (var s = 0; s <= _TabPosesState._seconds; s++) {
      c.drawLine(Offset(x(s.toDouble()), 0), Offset(x(s.toDouble()), 4), Paint()..color = Grey.g38);
    }
    if (mv != null) {
      final by = h._spanBy ?? 1, start = h._startAt ?? mv.start, lag = mv.lag * by, length = mv.length * by;
      final last = start + length + lag * math.max(0, mv.who.length - 1);
      c.drawRect(Rect.fromLTRB(x(start), 6, x(last), size.height - 2), Paint()..color = ink.withValues(alpha: .35));
      for (var k = 0; k < mv.who.length; k++) {
        c.drawLine(Offset(x(start + lag * k), 6), Offset(x(start + lag * k), size.height - 2), Paint()..color = ink);
      }
    }
    final now = x(ws.frame / Ws.fps);
    c.drawLine(Offset(now, 0), Offset(now, size.height), Paint()..color = WsT.accent..strokeWidth = 1.5);
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
