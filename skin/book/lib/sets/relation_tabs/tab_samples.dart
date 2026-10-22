// Relations tab, way D "samples that move" (CapCut's In / Out / Combo tabs, Keynote's Build In / Build Out): two rows of
// tiles, coming in above and going out below, each a little loop of five dots in turn, drawn from the very numbers it
// tells the group (its other end, its ease), so it is read at a glance and never as a word. Tapped with things picked:
// a move of them at the playhead; with a move picked: that move made so. Under them three pulls drawn as their effect:
// the stairs (sideways: each later than the one before), the bar (sideways: how long), the curve (up: slowing at the
// end; over the top: past the seat and back). Joined to the document through moves.dart.
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
  const pad = 8.0, gap = 4.0, mark = 18.0, cols = 5;
  final w = (s.width - pad * 2 - mark - gap * cols) / cols, h = math.min(52.0, w * .8);
  List<Rect> rows(double top) => [for (var i = 0; i < samples.length; i++) Rect.fromLTWH(pad + mark + gap + (i % cols) * (w + gap), top + (i ~/ cols) * (h + gap), w, h)];
  final rowsH = ((samples.length + cols - 1) ~/ cols) * (h + gap) - gap, come = rows(pad), goTop = pad + rowsH + 10, pullTop = goTop + rowsH + 14;
  final pw = (s.width - pad * 2 - gap * 2) / 3;
  return (
    come: come,
    go: rows(goTop),
    comeMark: Rect.fromLTWH(pad, pad, mark, rowsH),
    goMark: Rect.fromLTWH(pad, goTop, mark, rowsH),
    pulls: [for (var i = 0; i < 3; i++) Rect.fromLTWH(pad + i * (pw + gap), pullTop, pw, 64)],
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
  int? _pull;   // (0 stairs, 1 bar, 2 curve, while a hand has it)
  double _from = 0, _moved = 0;
  double? _lengthAt, _easeAt;   // (the bar and the curve: shown where the hand has them, told when let go)
  late Size _size;

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
        if (r.contains(at)) return _take(ws, samples[i], coming);
      }
    }
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
    wsMoveEase(ws, at, s.out, s.over);
    wsMoveHold(ws, false);
  }

  void _down(Ws ws, Offset at) {
    final mv = _picked(ws), i = samplesLayout(_size).pulls.indexWhere((r) => r.contains(at));
    if (mv == null || i < 0) return;
    _pull = i;
    _moved = 0;
    _from = switch (i) { 0 => mv.lag, 1 => mv.length, _ => mv.over > 0 ? 1 + mv.over : mv.out };
    if (i == 0) wsMoveHold(ws, true);
  }

  void _drag(Ws ws, Offset d) {
    final mv = _picked(ws);
    if (mv == null || _pull == null) return;
    _moved += _pull == 2 ? -d.dy : d.dx;
    switch (_pull) {
      case 0:
        wsMoveSet(ws, mv.by, 'lag', (_from + _moved / 400).clamp(0.0, 1.0));
      case 1:
        setState(() => _lengthAt = (_from + _moved / 120).clamp(.05, 4.0));
      case 2:
        setState(() => _easeAt = (_from + _moved / 90).clamp(0.0, 1.6));
    }
  }

  void _up(Ws ws) {
    final mv = _picked(ws);
    switch (_pull) {
      case 0:
        wsMoveHold(ws, false);
      case 1 when mv != null && _lengthAt != null:
        wsMoveTime(ws, mv, mv.start, _lengthAt!);
      case 2 when mv != null && _easeAt != null:
        wsMoveEase(ws, mv, math.min(1, _easeAt!), math.max(0, _easeAt! - 1));
    }
    setState(() {
      _pull = null;
      _lengthAt = _easeAt = null;
    });
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
    final lag = (mv?.lag ?? .08).clamp(.02, .25), length = (h._lengthAt ?? mv?.length ?? .5).clamp(.2, 1.2);   // (the loops in the picked move's time, kept short)

    for (final (coming, tiles, mark) in [(true, lay.come, lay.comeMark), (false, lay.go, lay.goMark)]) {
      _mark(c, mark, coming, live);
      for (final (i, r) in tiles.indexed) {
        final s = samples[i], on = mv != null && mv!.coming == coming && s.end.entries.every((e) => (mv!.end[e.key]! - e.value).abs() < 1e-3);
        c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), Paint()..color = on ? Grey.g20 : Grey.g13);
        if (on) c.drawRRect(RRect.fromRectAndRadius(r.deflate(.5), const Radius.circular(4)), Paint()..color = WsT.accent..style = PaintingStyle.stroke);
        c.save();
        c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
        _loop(c, r, s, coming, now, lag, length, on ? WsT.accent : (live ? Grey.g76 : Grey.g44));
        c.restore();
      }
    }

    final ink = mv == null ? Grey.g26 : Grey.g76, faint = mv == null ? Grey.g20 : Grey.g38;
    for (final (i, r) in lay.pulls.indexed) {
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), Paint()..color = h._pull == i ? Grey.g20 : Grey.g13);
    }
    final m = mv;
    final pLag = m?.lag ?? .08, pLength = h._lengthAt ?? m?.length ?? .6, pEase = h._easeAt ?? (m == null ? .8 : (m.over > 0 ? 1 + m.over : m.out));
    final out = math.min(1.0, pEase), over = math.max(0.0, pEase - 1);

    // the stairs: five steps, each the lag later
    final st = lay.pulls[0].deflate(8);
    _text(c, pLag.toStringAsFixed(2), st.topLeft, ink, 12);
    final kx = (st.width - 12) / 1.0, stair = Path();
    for (var k = 0; k < 5; k++) {
      final x = st.left + math.min(k * pLag * kx, st.width - 12), y = st.bottom - k * 6;
      k == 0 ? stair.moveTo(x, y) : stair.lineTo(x, y);
      stair.lineTo(x + 12, y);
      c.drawLine(Offset(x, y), Offset(x + 12, y), Paint()..color = ink..strokeWidth = 3);
    }
    c.drawPath(stair, Paint()..color = faint..style = PaintingStyle.stroke);

    // the bar: as long as the move
    final ba = lay.pulls[1].deflate(8);
    _text(c, pLength.toStringAsFixed(2), ba.topLeft, ink, 12);
    for (var s = 0; s <= 4; s++) {
      final x = ba.left + s / 4 * ba.width;
      c.drawLine(Offset(x, ba.bottom - 2), Offset(x, ba.bottom + 2), Paint()..color = faint);
    }
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(ba.left, ba.bottom - 14, (pLength / 4).clamp(.01, 1) * ba.width, 8), const Radius.circular(2)), Paint()..color = ink);

    // the curve: the way it eases, the seat as a line (past it: overshoot)
    final cu = lay.pulls[2].deflate(8), seat = cu.bottom - (cu.height - 14) * .62;
    _text(c, over > 0 ? '+${over.toStringAsFixed(2)}' : out.toStringAsFixed(2), cu.topLeft, ink, 12);
    c.drawLine(Offset(cu.left, seat), Offset(cu.right, seat), Paint()..color = faint);
    final way = _way(out, over), curve = Path()..moveTo(cu.left, cu.bottom);
    for (var u = 1; u <= 24; u++) {
      curve.lineTo(cu.left + u / 24 * cu.width, cu.bottom - _bezier(way, u / 24) * (cu.bottom - seat));
    }
    c.drawPath(curve, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 2);
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

  void _text(Canvas c, String s, Offset at, Color ink, double size) {
    final tp = TextPainter(text: TextSpan(text: s, style: TextStyle(color: ink, fontSize: size, height: 1)), textDirection: TextDirection.ltr)..layout();
    tp.paint(c, at);
  }

  @override
  bool shouldRepaint(_Paint o) => true;
}
