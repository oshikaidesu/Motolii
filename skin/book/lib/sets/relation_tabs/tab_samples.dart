// Relations tab, way F "one tap" (grown from way D, the judges' best): two rows of tiles, coming in above and going out
// below (CapCut's In / Out tabs, Keynote's Build In / Build Out), each a little loop of five dots in turn, drawn from the
// very numbers it tells the group, so it is read at a glance and never as a word. Tapped with things picked (on the
// stage, the timeline): a move of them at the playhead; with a move picked: that move made so, its ease kept. The last
// tile of a row is After Effects' pick whip: tapped, then a thing tapped on the stage or the timeline, they come out
// of that thing (go into it). Under them the stairs: pulled sideways, each later than the one before. When a move is
// and how long is its two keys on the timeline; how it eases is the Ease desk's. Joined through moves.dart.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import '../../workspace/moves.dart';
import '../../workspace/ws.dart';

/// One sample: the group's other end (the values it differs in from [wsMoveEnd]) and how it eases.
class Sample {
  const Sample(this.own, {this.out = .8, this.over = 0});
  final Map<String, double> own;
  final double out, over;
  Map<String, double> get end => {...wsMoveEnd, ...own};
}

/// The samples, the same for coming in and going out: from below, left, right, above, a point, far out, scattered,
/// clear, edge on, all but nothing (and past the seat).
/// The last tile of each row: the pick whip (a thing tapped next is where they come out of, or go into).
final whip = samples.length;

const samples = [
  Sample({'y': -.2, 'opacity': 0}),
  Sample({'x': -.35, 'opacity': 0}),
  Sample({'x': .35, 'opacity': 0}),
  Sample({'y': .2, 'opacity': 0}),
  Sample({'spread': 0}),
  Sample({'spread': 2, 'opacity': 0}),
  Sample({'scatter': .5, 'opacity': 0}),
  Sample({'opacity': 0}, out: .5),
  Sample({'rotate_y': 90}),
  Sample({'size': .01}, out: 1, over: .3),   // (the inside's least size: at 0 it refuses the whole step)
];

/// Where the tab lays its parts in a box of [s]: the tiles coming in and going out (in [samples]' order), the marks
/// beside the two rows, the three pulls (stairs, bar, curve).
({List<Rect> come, List<Rect> go, Rect comeMark, Rect goMark, List<Rect> pulls}) samplesLayout(Size s) {
  const pad = 8.0, gap = 4.0, mark = 18.0;
  final n = whip + 1, cols4 = (n + 1) ~/ 2;   // (the samples and the pick whip, in two lines a row)
  final w = (s.width - pad * 2 - mark - gap * cols4) / cols4, h = math.min(52.0, w * .8);
  List<Rect> rows(double top) => [for (var i = 0; i < n; i++) Rect.fromLTWH(pad + mark + gap + (i % cols4) * (w + gap), top + (i ~/ cols4) * (h + gap), w, h)];
  final rowsH = ((n + cols4 - 1) ~/ cols4) * (h + gap) - gap, come = rows(pad), goTop = pad + rowsH + 10, pullTop = goTop + rowsH + 14;
  return (
    come: come,
    go: rows(goTop),
    comeMark: Rect.fromLTWH(pad, pad, mark, rowsH),
    goMark: Rect.fromLTWH(pad, goTop, mark, rowsH),
    pulls: [Rect.fromLTWH(pad, pullTop, s.width - pad * 2, 48)],
  );
}

class TabSamples extends StatefulWidget {
  const TabSamples({super.key});
  @override
  State<TabSamples> createState() => _TabSamplesState();
}

class _TabSamplesState extends State<TabSamples> with SingleTickerProviderStateMixin {
  final _clock = ValueNotifier(0.0);   // (seconds the loops have played: only the painting follows it)
  late final Ticker _ticker;
  bool _seen = false;   // (painted since the last tick: a tab behind another is kept but not painted, and its loops rest)
  double _base = 0;
  bool _pulling = false;   // (the stairs in a hand)
  double _from = 0, _moved = 0;
  late Size _size;
  ({bool coming, String? move})? _whip;   // (the pick whip out: the next thing tapped is where they come from, or go into)

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_tick);
  }

  void _tick(Duration d) {
    if (d == Duration.zero) return;
    if (!_seen) {
      _base = _clock.value;
      return _ticker.stop();
    }
    _seen = false;
    _clock.value = _base + d.inMicroseconds / 1e6;
  }

  void _painted() {
    _seen = true;
    if (!_ticker.isActive) SchedulerBinding.instance.addPostFrameCallback((_) => mounted && !_ticker.isActive ? _ticker.start() : null);
  }

  @override
  void dispose() {
    _ticker.dispose();
    _clock.dispose();
    super.dispose();
  }

  WsMove? _picked(Ws ws) => ws.inside == null || int.tryParse(ws.selected ?? '') == null ? null : wsMoveOf(ws.inside!, int.parse(ws.selected!));

  List<String> _who(Ws ws) => [
    for (final id in ws.many)
      if (int.tryParse(id) case final t? when ws.inside!.things.contains(t) && ws.inside!.word(t, 'kind') != 'behaviour') id,
  ];

  // ---- the hand

  void _tap(Ws ws, Offset at) {
    if (ws.inside == null) return;
    final lay = samplesLayout(_size);
    for (final (coming, tiles) in [(true, lay.come), (false, lay.go)]) {
      for (final (i, r) in tiles.indexed) {
        if (!r.contains(at)) continue;
        if (i < whip) return _take(ws, samples[i], coming);
        if (_whip?.coming == coming) return setState(() => _whip = null);   // (put away)
        final mv = _picked(ws);
        if (_who(ws).isNotEmpty) _take(ws, samples[0], coming);   // (things picked: their move first, then the thing)
        return setState(() => _whip = (coming: coming, move: mv?.by.toString()));
      }
    }
  }

  /// The pick whip let go on a thing (the next one picked that is not a move nor in it): they come out of it (go into
  /// it): their middle there, gathered, small, clear; the move picked again.
  void _whipped(Ws ws) {
    final w = _whip, m = ws.inside, t = int.tryParse(ws.selected ?? '');
    if (w == null || m == null || t == null) return;
    final moves = wsMoves(ws), mv = w.move == null ? (moves.isEmpty ? null : moves.reduce((a, b) => a.by > b.by ? a : b)) : wsMoveOf(m, int.parse(w.move!));
    if (mv == null || mv.by == t || mv.who.contains(t) || m.word(t, 'kind') == 'behaviour') return;
    _whip = null;
    final now = mv.away;
    double mid(String n) => mv.who.fold(0.0, (a, o) => a + (m.value(o, n, now) ?? 0)) / mv.who.length;
    wsMoveHold(ws, true);
    wsMoveSet(ws, mv.by, 'x', (m.value(t, 'x', now) ?? 0) - mid('x'));
    wsMoveSet(ws, mv.by, 'y', (m.value(t, 'y', now) ?? 0) - mid('y'));
    for (final (n, v) in const [('spread', 0.0), ('size', .2), ('opacity', 0.0), ('scatter', 0.0)]) {
      wsMoveSet(ws, mv.by, n, v);
    }
    if (mv.coming != w.coming) wsMoveTurn(ws, mv);
    wsMoveHold(ws, false);
    ws.select('${mv.by}');
  }

  /// A sample told: a new move of the things picked (they first), else the picked move made so; one step.
  void _take(Ws ws, Sample s, bool coming) {
    final who = _who(ws), mv = _picked(ws);
    if (who.isEmpty && mv == null) return;
    wsMoveHold(ws, true);
    final WsMove at;
    if (who.isNotEmpty) {
      final start = (ws.frame / Ws.fps * 10).round() / 10;
      wsMakeMove(ws, who, start);
      at = WsMove(-1, const [], start, .6, true, .08, wsMoveEnd, .8, 0);   // (-1: the group the last add made, as the inside reads it)
    } else {
      at = mv!;
    }
    for (final e in s.end.entries) {
      wsMoveSet(ws, at.by, e.key, e.value);
    }
    if (at.coming != coming) wsMoveTurn(ws, at);
    if (who.isNotEmpty || s.over > 0) wsMoveEase(ws, at, s.out, s.over);   // (a move made over keeps how it eases: that is the Ease desk's)
    wsMoveHold(ws, false);
  }

  void _down(Ws ws, Offset at) {
    final mv = _picked(ws);
    if (mv == null || !samplesLayout(_size).pulls.first.contains(at)) return;
    _pulling = true;
    _moved = 0;
    _from = mv.lag;
    wsMoveHold(ws, true);
  }

  void _drag(Ws ws, Offset d) {
    final mv = _picked(ws);
    if (mv == null || !_pulling) return;
    _moved += d.dx;
    wsMoveSet(ws, mv.by, 'lag', (_from + _moved / 400).clamp(0.0, 1.0));
  }

  void _up(Ws ws) {
    if (_pulling) wsMoveHold(ws, false);
    setState(() => _pulling = false);
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context);
      if (ws.inside == null) return const SizedBox.expand();
      if (_whip != null) WidgetsBinding.instance.addPostFrameCallback((_) => mounted ? _whipped(ws) : null);
      return LayoutBuilder(builder: (context, box) {
        _size = box.biggest;
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapUp: (e) => _tap(ws, e.localPosition),
          onPanStart: (e) => _down(ws, e.localPosition),
          onPanUpdate: (e) => _drag(ws, e.delta),
          onPanEnd: (_) => _up(ws),
          onPanCancel: () => _up(ws),
          child: CustomPaint(size: _size, painter: _Paint(this, ws, _picked(ws), _who(ws).isNotEmpty)),
        );
      });
    },
  );
}

class _Paint extends CustomPainter {
  _Paint(this.h, this.ws, this.mv, this.many) : super(repaint: h._clock);
  final _TabSamplesState h;
  final Ws ws;
  final WsMove? mv;
  final bool many;

  @override
  void paint(Canvas c, Size size) {
    h._painted();
    c.drawRect(Offset.zero & size, Paint()..color = Grey.g10);
    final lay = samplesLayout(size), live = many || mv != null, now = h._clock.value;
    final lag = (mv?.lag ?? .08).clamp(.02, .25), length = (mv?.length ?? .5).clamp(.2, 1.2);   // (the loops in the picked move's time, kept short)

    for (final (coming, tiles, mark) in [(true, lay.come, lay.comeMark), (false, lay.go, lay.goMark)]) {
      _mark(c, mark, coming, live);
      for (final (i, r) in tiles.indexed) {
        if (i == whip) {
          _whipTile(c, r, coming, h._whip?.coming == coming, live, now);
          continue;
        }
        final s = samples[i], on = mv != null && mv!.coming == coming && s.end.entries.every((e) => (mv!.end[e.key]! - e.value).abs() < 1e-3);
        c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), Paint()..color = on ? Grey.g20 : Grey.g13);
        if (on) c.drawRRect(RRect.fromRectAndRadius(r.deflate(.5), const Radius.circular(4)), Paint()..color = WsT.accent..style = PaintingStyle.stroke);
        c.save();
        c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
        _loop(c, r, s, coming, now + i * .23, lag, length, on ? WsT.accent : (live ? Grey.g76 : Grey.g44));   // (each its own phase: seen at once, never all the same picture)
        c.restore();
      }
    }

    final ink = mv == null ? Grey.g26 : Grey.g76, faint = mv == null ? Grey.g20 : Grey.g38;
    final r0 = lay.pulls.first;
    c.drawRRect(RRect.fromRectAndRadius(r0, const Radius.circular(4)), Paint()..color = h._pulling ? Grey.g20 : Grey.g13);

    // the stairs: as many steps as the move has things, each the lag later (the same seconds across as the timeline)
    final st = r0.deflate(8), n = math.max(2, mv?.who.length ?? 5), pLag = mv?.lag ?? .08, kx = st.width / 2.0, stair = Path();
    for (var k = 0; k < n; k++) {
      final x = st.left + math.min(k * pLag * kx, st.width - 10), y = st.bottom - k * (st.height - 4) / (n - 1);
      k == 0 ? stair.moveTo(x, y) : stair.lineTo(x, y);
      stair.lineTo(x + 10, y);
      c.drawLine(Offset(x, y), Offset(x + 10, y), Paint()..color = ink..strokeWidth = 3);
    }
    c.drawPath(stair, Paint()..color = faint..style = PaintingStyle.stroke);
  }

  /// The pick whip: a curl from the dots to a small frame (the thing): coming, the dots leave the frame; going, they go
  /// into it. Out (tapped, waiting for a thing): lit.
  void _whipTile(Canvas c, Rect r, bool coming, bool out, bool live, double now) {
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), Paint()..color = out ? Grey.g26 : Grey.g13);
    final ink = out ? WsT.accent : (live ? Grey.g76 : Grey.g44), box = Rect.fromCenter(center: r.center + Offset(-r.width * .22, 0), width: 10, height: 10);
    c.drawRect(box, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 1.5);
    final curl = Path()..moveTo(box.right + 2, box.center.dy);
    for (var u = 0.0; u <= 1; u += .05) {
      curl.lineTo(box.right + 2 + u * r.width * .45, box.center.dy + math.sin(u * 3 * math.pi) * 5 * (1 - u));
    }
    c.drawPath(curl, Paint()..color = ink.withValues(alpha: .6)..style = PaintingStyle.stroke);
    final f = ((now % 1.4) / 1.1).clamp(0.0, 1.0), p = coming ? f : 1 - f;
    for (var i = 0; i < 3; i++) {
      final a = (i - 1) * .6, d = p * r.width * .32;
      c.drawCircle(box.center + Offset(math.cos(a) * d, math.sin(a) * d), 2.2, Paint()..color = ink.withValues(alpha: (.3 + .7 * p).clamp(0, 1)));
    }
  }

  /// A tile's loop: five dots in a row (their seats), each [lag] later, from the other end to the seats (coming) or
  /// out to it, eased by the sample's own way; a rest at each end.
  void _loop(Canvas c, Rect r, Sample s, bool coming, double now, double lag, double length, Color ink) {
    final d = r.width / 7, k = r.width * 1.1, e = s.end, way = _way(s.out, s.over);
    final period = .35 + length + 4 * lag + .7, t = now % period - .35;
    for (var i = 0; i < 5; i++) {
      final u = ((t - i * lag) / length).clamp(0.0, 1.0), p = _bezier(way, u), f = coming ? 1 - p : p;   // (f: how far toward the other end)
      final ang = i * 2.399 + .7, amt = e['scatter']! * (.6 + .4 * ((i * .618) % 1));
      final x = (i - 2) * d * (1 + (e['spread']! - 1) * f) + (e['x']! + math.cos(ang) * amt) * k * f;
      final y = -(e['y']! + math.sin(ang) * amt) * k * f;
      final size = (1 + (e['size']! - 1) * f).clamp(0.0, 3.0), alpha = (1 + (e['opacity']! - 1) * f).clamp(0.0, 1.0);
      final flip = math.cos(e['rotate_y']! * f * math.pi / 180).abs();
      c.save();
      c.translate(r.center.dx + x, r.center.dy + y);
      c.rotate(e['rotate']! * f * math.pi / 180);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 7 * size * flip, height: 7 * size), const Radius.circular(1.5)), Paint()..color = ink.withValues(alpha: alpha));
      c.restore();
    }
  }

  /// Beside a row: a frame with an arrow going into it (coming) or out of it.
  void _mark(Canvas c, Rect r, bool coming, bool live) {
    final ink = live ? Grey.g63 : Grey.g38, o = r.center, box = Rect.fromCenter(center: o + const Offset(0, 6), width: 12, height: 12);
    c.drawRect(box, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 1.5);
    final a = o + const Offset(0, -14), z = o + const Offset(0, 4);
    final (from, to) = coming ? (a, z) : (z, a);
    c.drawLine(from, to, Paint()..color = ink..strokeWidth = 1.5);
    final dir = to.dy > from.dy ? 1.0 : -1.0;
    c.drawPath(Path()..addPolygon([to, to + Offset(-4, -5 * dir), to + Offset(4, -5 * dir)], true), Paint()..color = ink);
  }

  static List<double> _way(double out, double over) => wsMoveWay(out, over).split(' ').skip(1).map(double.parse).toList();

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
