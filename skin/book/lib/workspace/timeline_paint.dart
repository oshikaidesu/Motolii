// The workspace timeline's drawing: ruler, track rows, playhead, overview strip and the kind chip.
// Every painter takes plain values and repaints only when one of them changes; nothing here ticks.
import 'dart:math' as math;

import 'package:flutter/foundation.dart' show setEquals;
import 'package:flutter/widgets.dart';

import '../sets/panels/timeline_parts_parts.dart' show tpFine, tpMajor;
import '../tokens.dart';
import 'ws.dart';

/// The timeline seat's own sizes.
abstract final class WtM {
  static const header = 32.0, label = 200.0, ruler = 30.0, row = 24.0, overview = 12.0, divider = 1.0;

  /// The work-area band on top of the ruler, and the room left of frame 0 / right of the last frame.
  static const band = 9.0, padL = 10.0, padR = 14.0;
  static const bar = 18.0, barRadius = 4.0, key = 4.5, keyPicked = 6.0, chip = 14.0;
  static const maxZoom = 16.0;
}

/// Where frames land on the track area: x = padL + (frame - scroll) * ppf.
@immutable
class WtView {
  const WtView(this.ppf, this.scroll);
  final double ppf, scroll;
  double x(num f) => WtM.padL + (f - scroll) * ppf;
  double frameAt(double x) => (x - WtM.padL) / ppf + scroll;
  @override
  bool operator ==(Object other) => other is WtView && other.ppf == ppf && other.scroll == scroll;
  @override
  int get hashCode => Object.hash(ppf, scroll);
}

/// One row as drawn: a snapshot of the layer plus how the seat shows it.
@immutable
class WtRow {
  const WtRow({
    required this.name,
    required this.kind,
    required this.inF,
    required this.outF,
    required this.keys,
    required this.picked,
    this.selected = false,
    this.dim = false,
    this.locked = false,
  });
  final String name;
  final WsKind kind;
  final int inF, outF;
  final List<int> keys;
  final Set<int> picked;
  final bool selected, dim, locked;
  @override
  bool operator ==(Object other) =>
      other is WtRow &&
      other.name == name &&
      other.kind == kind &&
      other.inF == inF &&
      other.outF == outF &&
      _listEq(other.keys, keys) &&
      setEquals(other.picked, picked) &&
      other.selected == selected &&
      other.dim == dim &&
      other.locked == locked;
  @override
  int get hashCode => Object.hash(name, kind, inF, outF, Object.hashAll(keys), Object.hashAllUnordered(picked), selected, dim, locked);
}

bool _listEq<E>(List<E> a, List<E> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

TextPainter _text(String s, TextStyle st) => TextPainter(
  text: TextSpan(text: s, style: st),
  textDirection: TextDirection.ltr,
  maxLines: 1,
)..layout();

void _vLines(Canvas c, WtView v, int fps, double w, double top, double bottom, {required Color major, required Color fine}) {
  final mj = tpMajor(v.ppf, fps), fn = tpFine(v.ppf, fps, mj), p = Paint();
  final f0 = (v.scroll / fn).floor() * fn;
  for (var f = f0; f <= Ws.duration; f += fn) {
    final x = v.x(f).roundToDouble();
    if (x > w) break;
    if (x < 0) continue;
    c.drawRect(Rect.fromLTWH(x, top, 1, bottom - top), p..color = f % mj == 0 ? major : fine);
  }
}

// ---------------------------------------------------------------- ruler

class WtRulerPainter extends CustomPainter {
  const WtRulerPainter({required this.view, required this.fps, required this.workIn, required this.workOut, required this.markers});
  final WtView view;
  final int fps, workIn, workOut;
  final List<(int, String)> markers;

  @override
  void paint(Canvas c, Size s) {
    final p = Paint(), end = view.x(Ws.duration);
    c.drawRect(Offset.zero & s, p..color = Grey.g10);
    c.drawRect(Rect.fromLTRB(0, 0, s.width, WtM.band), p..color = Grey.g07);
    if (end < s.width) c.drawRect(Rect.fromLTRB(end, WtM.band, s.width, s.height), p..color = Grey.g07);

    final a = view.x(workIn), b = view.x(workOut);
    c.drawRRect(RRect.fromLTRBR(a, 2, b, WtM.band - 1, const Radius.circular(3)), p..color = Grey.g38);
    for (final x in [a, b - 4]) {
      c.drawRRect(RRect.fromLTRBR(x, 1, x + 4, WtM.band, const Radius.circular(2)), p..color = Grey.g76);
    }

    final mj = tpMajor(view.ppf, fps), fn = tpFine(view.ppf, fps, mj), bottom = s.height - 1;
    final flags = <(Rect, TextPainter)>[];
    for (final (f, name) in markers) {
      final x = view.x(f).roundToDouble();
      if (x < -60 || x > s.width) continue;
      final t = _text(name, T.micro(Grey.g13).copyWith(fontWeight: FontWeight.w700));
      flags.add((Rect.fromLTWH(x, WtM.band + 2, t.width + 8, 12), t));
    }
    if (view.ppf >= 3) {
      final f0 = view.scroll.floor();
      for (var f = math.max(0, f0); f <= Ws.duration; f++) {
        final x = view.x(f).roundToDouble();
        if (x > s.width) break;
        if (f % fn != 0) c.drawRect(Rect.fromLTWH(x, bottom - 3, 1, 3), p..color = Grey.g26);
      }
    }
    final f0 = (view.scroll / fn).floor() * fn;
    for (var f = math.max(0.0, f0); f <= Ws.duration; f += fn) {
      final x = view.x(f).roundToDouble();
      if (x > s.width) break;
      final isMajor = f % mj == 0;
      c.drawRect(Rect.fromLTWH(x, bottom - (isMajor ? 9 : 5), 1, isMajor ? 9 : 5), p..color = isMajor ? Grey.g63 : Grey.g44);
      if (isMajor) {
        if (flags.any((m) => m.$1.inflate(3).overlaps(Rect.fromLTWH(x + 3, WtM.band + 2, 24, 12)))) continue;
        final sec = f ~/ fps, fr = (f % fps).round();
        final t = _text(mj >= fps ? '${sec}s' : '$sec:${fr.toString().padLeft(2, '0')}', T.value(Grey.g76).copyWith(fontSize: 10));
        t.paint(c, Offset(x + 3, WtM.band + 3));
        t.dispose();
      }
    }

    for (final (r, t) in flags) {
      final x = r.left;
      c.drawPath(
        Path()
          ..moveTo(r.left, r.top)
          ..lineTo(r.right, r.top)
          ..lineTo(r.right - 3, r.center.dy)
          ..lineTo(r.right, r.bottom)
          ..lineTo(r.left, r.bottom)
          ..close(),
        p..color = WsT.toneText,
      );
      c.drawRect(Rect.fromLTWH(x, r.bottom, 1, bottom - r.bottom), p..color = WsT.toneText);
      t.paint(c, Offset(r.left + 3, r.center.dy - t.height / 2));
      t.dispose();
    }
    c.drawRect(Rect.fromLTWH(0, bottom, s.width, 1), p..color = WsT.line);
  }

  @override
  bool shouldRepaint(WtRulerPainter o) => o.view != view || o.fps != fps || o.workIn != workIn || o.workOut != workOut || o.markers != markers;
}

// ---------------------------------------------------------------- tracks

Color wtBand(int i, bool selected) => selected ? Grey.g20 : (i.isEven ? Grey.g10 : Grey.g13);

/// The bar's fill: the kind's hue, a step quieter when not selected; a hidden or soloed-out layer drops to grey.
Color wtBarFill(WtRow r) => r.dim ? Grey.g26 : (r.selected ? wsTone(r.kind) : Color.lerp(wsTone(r.kind), Grey.g07, .22)!);

/// The waveform's loudness at a frame: a kick every half second, a softer off-beat, a little grain.
double wtWave(double f) {
  final beat = (f % 15) / 15, off = ((f + 7.5) % 15) / 15;
  final kick = math.exp(-beat * 5.5), snare = math.exp(-off * 7) * .45;
  final grain = .5 + .5 * math.sin(f * 2.3) * math.sin(f * .71 + 1.3);
  return (.12 + .7 * kick + snare + .18 * grain).clamp(.08, 1.0);
}

class WtTracksPainter extends CustomPainter {
  const WtTracksPainter({required this.rows, required this.view, required this.fps, required this.vscroll});
  final List<WtRow> rows;
  final WtView view;
  final int fps;
  final double vscroll;

  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    final p = Paint(), end = view.x(Ws.duration);
    c.drawRect(Offset.zero & s, p..color = WsT.body);
    for (var i = 0; i < rows.length; i++) {
      final top = i * WtM.row - vscroll;
      if (top > s.height || top + WtM.row < 0) continue;
      c.drawRect(Rect.fromLTWH(0, top, s.width, WtM.row - 1), p..color = wtBand(i, rows[i].selected));
    }
    if (end < s.width) c.drawRect(Rect.fromLTRB(end, 0, s.width, s.height), p..color = WsT.body);
    _vLines(c, view, fps, math.min(s.width, end), 0, s.height, major: Grey.g20, fine: Grey.g15);
    for (var i = 0; i < rows.length; i++) {
      final top = i * WtM.row - vscroll;
      if (top > s.height || top + WtM.row < 0) continue;
      _row(c, s, rows[i], top + WtM.row / 2 - .5);
    }
  }

  void _row(Canvas c, Size s, WtRow r, double cy) {
    final p = Paint(), x0 = view.x(r.inF), x1 = math.max(view.x(r.outF), x0 + 4);
    if (x1 < 0 || x0 > s.width) return;
    final bar = Rect.fromLTRB(x0, cy - WtM.bar / 2, x1, cy + WtM.bar / 2), rr = RRect.fromRectAndRadius(bar, const Radius.circular(WtM.barRadius));
    final fill = wtBarFill(r), ink = r.dim ? Grey.g56 : WsT.onAccent, grip = Color.lerp(fill, Grey.g00, .28)!;
    c.drawRRect(rr, p..color = fill);
    c.save();
    c.clipRRect(rr);
    c.drawRect(Rect.fromLTWH(x0, bar.top, 3, bar.height), p..color = grip);
    c.drawRect(Rect.fromLTWH(x1 - 3, bar.top, 3, bar.height), p..color = grip);
    if (r.kind == WsKind.audio) _wave(c, bar, ink);
    final nameX = math.max(x0 + 8, 6.0);
    final t = _text(r.name, T.name(ink).copyWith(fontWeight: FontWeight.w700));
    if (nameX + 20 < x1) {
      if (r.kind == WsKind.audio) c.drawRRect(RRect.fromLTRBR(nameX - 4, cy - 6.5, nameX + t.width + 4, cy + 6.5, const Radius.circular(3)), p..color = fill);
      c.save();
      c.clipRect(Rect.fromLTRB(nameX, bar.top, x1 - 6, bar.bottom));
      t.paint(c, Offset(nameX, cy - t.height / 2));
      c.restore();
    }
    t.dispose();
    if (r.locked) {
      for (var x = x0 - bar.height; x < x1; x += 6) {
        c.drawLine(
          Offset(x, bar.bottom),
          Offset(x + bar.height, bar.top),
          Paint()
            ..color = grip
            ..strokeWidth = 1,
        );
      }
    }
    c.restore();
    if (r.selected && !r.dim) {
      c.drawRRect(
        rr.deflate(.75),
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.5
          ..color = Grey.g100,
      );
    }
    for (final k in r.keys) {
      final x = view.x(k);
      if (x < -8 || x > s.width + 8) continue;
      final on = r.picked.contains(k);
      final rad = on ? WtM.keyPicked : WtM.key;
      final d = Path()
        ..moveTo(x, cy - rad)
        ..lineTo(x + rad, cy)
        ..lineTo(x, cy + rad)
        ..lineTo(x - rad, cy)
        ..close();
      if (on) {
        c.drawPath(d, p..color = WsT.keyDot);
        c.drawPath(
          d,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.5
            ..strokeJoin = StrokeJoin.miter
            ..color = WsT.onAccent,
        );
      } else {
        c.drawPath(d, p..color = r.dim ? Grey.g44 : WsT.onAccent);
        c.drawPath(
          d,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1
            ..color = r.dim ? Grey.g26 : Color.lerp(fill, Grey.g100, .55)!,
        );
      }
    }
  }

  void _wave(Canvas c, Rect bar, Color ink) {
    final p = Paint()..color = Color.lerp(wtBarFill(rows.firstWhere((e) => e.kind == WsKind.audio)), ink, .7)!;
    final h = bar.height - 4, cy = bar.center.dy;
    for (var x = bar.left + 4; x < bar.right - 4; x += 2) {
      final a = wtWave(view.frameAt(x));
      c.drawRect(Rect.fromCenter(center: Offset(x.roundToDouble(), cy), width: 1, height: (a * h).roundToDouble().clamp(1, h)), p);
    }
  }

  @override
  bool shouldRepaint(WtTracksPainter o) => o.view != view || o.fps != fps || o.vscroll != vscroll || !_listEq(o.rows, rows);
}

// ---------------------------------------------------------------- playhead

class WtPlayheadPainter extends CustomPainter {
  const WtPlayheadPainter({required this.view, required this.frame, required this.headTop});
  final WtView view;
  final int frame;
  final double headTop;

  @override
  void paint(Canvas c, Size s) {
    final x = view.x(frame).roundToDouble();
    if (x < -40 || x > s.width + 40) return;
    final p = Paint()..color = WsT.playhead;
    final t = _text('$frame', T.value(WsT.onAccent).copyWith(fontSize: 10, fontWeight: FontWeight.w700));
    final w = math.max(t.width + 10, 22.0), cx = (x + .5).clamp(w / 2, s.width - w / 2);
    final pill = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, headTop + 8), width: w, height: 15), const Radius.circular(4));
    c.drawRect(Rect.fromLTWH(x, headTop + 8, 1, s.height - headTop - 8), p);
    p.color = WsT.accent;
    c.drawRRect(pill, p);
    c.drawPath(
      Path()
        ..moveTo(x + .5 - 4, pill.bottom)
        ..lineTo(x + 4.5, pill.bottom)
        ..lineTo(x + .5, pill.bottom + 4)
        ..close(),
      p,
    );
    t.paint(c, Offset(cx - t.width / 2, pill.center.dy - t.height / 2));
    t.dispose();
  }

  @override
  bool shouldRepaint(WtPlayheadPainter o) => o.view != view || o.frame != frame || o.headTop != headTop;
}

// ---------------------------------------------------------------- overview (the horizontal scroll strip)

class WtOverviewPainter extends CustomPainter {
  const WtOverviewPainter({required this.from, required this.to, required this.frame, required this.hot});
  final double from, to;
  final int frame;
  final bool hot;

  @override
  void paint(Canvas c, Size s) {
    final p = Paint(), w = s.width - 8;
    double x(double f) => 4 + f / Ws.duration * w;
    c.drawRect(Offset.zero & s, p..color = WsT.body);
    c.drawRRect(RRect.fromLTRBR(4, 3, s.width - 4, s.height - 3, const Radius.circular(3)), p..color = Grey.g13);
    c.drawRRect(RRect.fromLTRBR(x(from), 3, math.max(x(to), x(from) + 12), s.height - 3, const Radius.circular(3)), p..color = hot ? Grey.g44 : Grey.g26);
    c.drawRect(Rect.fromLTWH(x(frame.toDouble()).roundToDouble(), 2, 2, s.height - 4), p..color = WsT.playhead);
  }

  @override
  bool shouldRepaint(WtOverviewPainter o) => o.from != from || o.to != to || o.frame != frame || o.hot != hot;
}

// ---------------------------------------------------------------- kind chip

/// A layer's kind: a small square of its hue with the kind drawn in the dark ink.
class WtKindChip extends StatelessWidget {
  const WtKindChip(this.kind, {super.key, this.dim = false});
  final WsKind kind;
  final bool dim;
  @override
  Widget build(BuildContext context) => Container(
    width: WtM.chip,
    height: WtM.chip,
    decoration: BoxDecoration(color: dim ? Grey.g26 : wsTone(kind), borderRadius: BorderRadius.circular(3)),
    child: CustomPaint(painter: _KindPainter(kind, dim ? Grey.g56 : WsT.onAccent)),
  );
}

class _KindPainter extends CustomPainter {
  const _KindPainter(this.kind, this.ink);
  final WsKind kind;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final w = s.width,
        f = Paint()..color = ink,
        st = Paint()
          ..color = ink
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.3
          ..strokeCap = StrokeCap.round;
    Offset o(double x, double y) => Offset(x * w, y * w);
    switch (kind) {
      case WsKind.shape:
        c.drawCircle(o(.5, .5), w * .27, f);
      case WsKind.text:
        c.drawRect(Rect.fromLTRB(w * .24, w * .22, w * .76, w * .34), f);
        c.drawRect(Rect.fromLTRB(w * .43, w * .22, w * .57, w * .8), f);
      case WsKind.image:
        c.drawPath(
          Path()
            ..moveTo(w * .18, w * .76)
            ..lineTo(w * .42, w * .4)
            ..lineTo(w * .58, w * .6)
            ..lineTo(w * .66, w * .5)
            ..lineTo(w * .84, w * .76)
            ..close(),
          f,
        );
        c.drawCircle(o(.68, .28), w * .09, f);
      case WsKind.camera:
        c.drawRRect(RRect.fromLTRBR(w * .16, w * .32, w * .64, w * .7, const Radius.circular(1.5)), f);
        c.drawPath(
          Path()
            ..moveTo(w * .66, w * .51)
            ..lineTo(w * .86, w * .34)
            ..lineTo(w * .86, w * .68)
            ..close(),
          f,
        );
      case WsKind.group:
        c.drawRect(Rect.fromLTRB(w * .2, w * .2, w * .62, w * .62), st);
        c.drawRect(Rect.fromLTRB(w * .4, w * .4, w * .8, w * .8), f);
      case WsKind.particles:
        for (final (x, y, r) in const [(.3, .32, .1), (.68, .3, .07), (.5, .56, .12), (.26, .74, .07), (.74, .72, .09)]) {
          c.drawCircle(o(x, y), w * r, f);
        }
      case WsKind.audio:
        for (final (x, h) in const [(.24, .2), (.38, .4), (.52, .28), (.66, .44), (.8, .16)]) {
          c.drawLine(o(x, .5 - h / 2 - .04), o(x, .5 + h / 2 + .04), st);
        }
    }
  }

  @override
  bool shouldRepaint(_KindPainter o) => o.kind != kind || o.ink != ink;
}
