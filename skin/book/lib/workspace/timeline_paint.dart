// The workspace timeline's drawing: ruler, track rows, playhead, overview strip and the kind chip.
// Every painter takes plain values and repaints only when one of them changes; nothing here ticks.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart' show setEquals;
import 'package:flutter/foundation.dart' show listEquals, mapEquals, ValueListenable;
import 'package:flutter/widgets.dart';

import '../forms/forms.dart';
import '../sets/panels/timeline_parts_parts.dart' show tpFine, tpMajor;
import '../tokens.dart';
import 'ws.dart';

/// The timeline seat's own sizes.
abstract final class WtM {
  static const divider = 1.0;
  static double get header => Shape.z(32.0);
  static double get label => Shape.z(200.0);
  static double get ruler => Shape.z(30.0);
  static double get row => Shape.z(24.0);
  static double get overview => Shape.z(12.0);

  /// The work-area band on top of the ruler, and the room left of frame 0 / right of the last frame.
  static double get band => Shape.z(12.0);
  static double get padL => Shape.s(10.0);
  static double get padR => Shape.s(14.0);
  static const barRadius = 4.0;
  static double get bar => Shape.z(18.0);
  static double get key => Shape.z(4.5);
  static double get keyPicked => Shape.z(6.0);
  static double get chip => Shape.z(14.0);
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
    this.clipped = false,
    this.wave,
    this.frames,
    this.form,
    this.fill,
    this.outline = false,
    this.tone,
    this.mix,
    this.stretches = const [],
    this.lane,
    this.laneWays,
    this.laneOn,
    this.whole,
  });
  final (int, int)? whole;   // (a film's whole length as it lies, cut or not: the pale slip-edit bar of After Effects)
  final List<WsStretch> stretches;   // (how it goes between its keys: each stretch's way drawn between them)

  /// A value's own lane in place of a bar (Ableton's automation lane): each key's frame and where its value stands
  /// between the lowest and the highest the value's keys hold (0 to 1); null: a layer's row.
  final List<(int, double)>? lane;

  /// How the lane goes from each key to the next: its way's shape, 0 at the key and 1 at the next (as a stretch's).
  final List<List<double>>? laneWays;
  final int? laneOn;   // (the stretch picked by its line: its place among them)
  final Color? tone;   // (its label colour; none: its kind's)
  final List<Color>? mix;   // (a relation's: its things' label colours, its bar their gradient)
  final String name;

  /// How loud its sound is at each frame of its bar (dB, the inside's reading; null: no sound to show).
  final List<double>? wave;

  /// What a film's bar plays, a picture for each of its stretches in turn (null: not a film, or not read yet).
  final List<ui.Image>? frames;

  /// A shape's form (the inside's shape word: rect, circle, star...) and its fill: its own small picture at the row's left.
  final String? form;
  final Color? fill;
  final bool outline;   // (no fill: drawn as its outline, [fill] its stroke's colour)
  final WsKind kind;
  final int inF, outF;
  final List<int> keys;
  final Set<int> picked;
  final bool selected, dim, locked, clipped;   // (clipped: cut to the thing under it, a ↘ before its name as Alight Motion marks it)
  @override
  bool operator ==(Object other) =>
      other is WtRow &&
      other.name == name &&
      other.kind == kind &&
      other.tone == tone &&
      listEquals(other.mix, mix) &&
      other.inF == inF &&
      other.outF == outF &&
      _listEq(other.keys, keys) &&
      setEquals(other.picked, picked) &&
      other.selected == selected &&
      other.dim == dim &&
      other.locked == locked &&
      other.clipped == clipped &&
      other.whole == whole &&
      identical(other.wave, wave) &&
      identical(other.frames, frames) &&   // (read once a change: the same list until the document changes)
      other.form == form &&
      other.fill == fill &&
      other.outline == outline &&
      listEquals(other.lane, lane) &&
      other.laneOn == laneOn &&
      other.stretches.length == stretches.length &&
      [for (var i = 0; i < stretches.length; i++) other.stretches[i].from == stretches[i].from && other.stretches[i].to == stretches[i].to && other.stretches[i].way == stretches[i].way].every((same) => same);
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

/// How wide a mark's flag on the ruler is, its name in it (the ruler draws it so; a hand finds it so).
double wtFlagWidth(String name) => _text(name, T.micro(Grey.g13).copyWith(fontWeight: FontWeight.w700)).width + 8;

/// How wide a mark's flag is where it stands: its name's width, cut where the next mark is, as Logic Pro shows a
/// marker's name only as far as the following marker (golf ⑳: 解放 at 8.0 s under M2 at 8.37 s); [xs]: every mark's x.
double wtFlagRoom(String name, double x, Iterable<double> xs) {
  final next = xs.where((o) => o > x).fold<double?>(null, (a, o) => a == null || o < a ? o : a);
  final w = wtFlagWidth(name);
  return next == null ? w : math.max(6, math.min(w, next - x - 1));
}

TextPainter _text(String s, TextStyle st) => TextPainter(
  text: TextSpan(text: s, style: st),
  textDirection: TextDirection.ltr,
  maxLines: 1,
)..layout();

void _vLines(Canvas c, WtView v, int fps, double w, double top, double bottom, {required Color major, required Color fine, WsBeats? beats}) {
  final mj = tpMajor(v.ppf, fps), fn = tpFine(v.ppf, fps, mj), p = Paint();
  if (beats != null) {   // (its grid by the beat, not the second, as far as they can be told apart: Live's grid widens as it is zoomed out)
    for (final (f, bar) in beats.between(v.scroll, v.scroll + w / v.ppf)) {
      if (bar || beats.every * v.ppf >= 8) c.drawRect(Rect.fromLTWH(v.x(f).roundToDouble(), top, 1, bottom - top), p..color = bar ? major : fine);
    }
    return;
  }
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
  const WtRulerPainter({required this.view, required this.fps, required this.workIn, required this.workOut, required this.markers, this.tempo, this.markColours = const {}, this.beats = const [], this.retimed = const []});
  final WtView view;
  final int fps, workIn, workOut;
  final List<(int, String)> markers;
  final Map<int, Color> markColours;   // (a mark's colour by its frame, if it has one)

  /// Where the films' sound strikes (their Beats), frames: a short tick each at the ruler's foot.
  final List<int> beats;
  final WsBeats? tempo;   // (the document's own beats: a tick each, taller where a bar begins)
  final List<int> retimed;   // (the picked element's end of intro and start of outro, frames: small yellow marks at the ruler's top, as Alight Motion shows them)

  @override
  void paint(Canvas c, Size s) {
    final p = Paint(), end = view.x(Ws.duration);
    c.drawRect(Offset.zero & s, p..color = Grey.g10);
    c.drawRect(Rect.fromLTRB(0, 0, s.width, WtM.band), p..color = Grey.g07);
    if (end < s.width) c.drawRect(Rect.fromLTRB(end, WtM.band, s.width, s.height), p..color = Grey.g07);

    final a = view.x(workIn), b = view.x(workOut);
    c.drawRRect(RRect.fromLTRBR(a, 2, b, WtM.band - 1, Shape.r(3)), p..color = Grey.g38);
    for (final x in [a, b - 4]) {
      c.drawRRect(RRect.fromLTRBR(x, 1, x + 4, WtM.band, Shape.r(2)), p..color = Grey.g76);
    }

    final mj = tpMajor(view.ppf, fps), fn = tpFine(view.ppf, fps, mj), bottom = s.height - 1;
    for (final f in beats) {
      final x = view.x(f).roundToDouble();
      if (x >= 0 && x <= s.width) c.drawRect(Rect.fromLTWH(x, bottom - 7, 2, 7), p..color = WsT.toneAudio);
    }
    if (tempo case final b?) {
      for (final (f, bar) in b.between(view.scroll, view.scroll + s.width / view.ppf)) {
        if (bar || b.every * view.ppf >= 8) c.drawRect(Rect.fromLTWH(view.x(f).roundToDouble(), bottom - (bar ? 11 : 6), 1, bar ? 11 : 6), p..color = bar ? WsT.accent : Grey.g56);
      }
    }
    for (final f in retimed) {
      final x = view.x(f).roundToDouble();
      if (x >= -4 && x <= s.width + 4) c.drawPath(Path()..moveTo(x - 4, 0)..lineTo(x + 4, 0)..lineTo(x, 6)..close(), p..color = WsT.toneMark);
    }
    final flags = <(Rect, TextPainter, Color)>[];
    final xs = [for (final (f, _) in markers) view.x(f).roundToDouble()];
    for (final (f, name) in markers) {
      final x = view.x(f).roundToDouble();
      if (x < -60 || x > s.width) continue;
      final t = _text(name, T.micro(Grey.g13).copyWith(fontWeight: FontWeight.w700));
      flags.add((Rect.fromLTWH(x, WtM.band + 2, wtFlagRoom(name, x, xs), 12), t, markColours[f] ?? WsT.toneText));
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
        final t = _text(mj >= fps ? '${sec}s' : '$sec:${fr.toString().padLeft(2, '0')}', T.value(Grey.g76).copyWith(fontSize: T.px(10)));
        t.paint(c, Offset(x + 3, WtM.band + 3));
        t.dispose();
      }
    }

    for (final (r, t, ink) in flags) {
      final x = r.left;
      c.drawPath(
        Path()
          ..moveTo(r.left, r.top)
          ..lineTo(r.right, r.top)
          ..lineTo(r.right - 3, r.center.dy)
          ..lineTo(r.right, r.bottom)
          ..lineTo(r.left, r.bottom)
          ..close(),
        p..color = ink,
      );
      c.drawRect(Rect.fromLTWH(x, r.bottom, 1, bottom - r.bottom), p..color = ink);
      c.save();
      c.clipRect(Rect.fromLTRB(r.left, r.top, r.right - 3, r.bottom));   // (a name cut at its flag's end)
      t.paint(c, Offset(r.left + 3, r.center.dy - t.height / 2));
      c.restore();
      t.dispose();
    }
    c.drawRect(Rect.fromLTWH(0, bottom, s.width, 1), p..color = WsT.line);
  }

  @override
  bool shouldRepaint(WtRulerPainter o) => o.view != view || o.fps != fps || o.workIn != workIn || o.workOut != workOut || o.markers != markers || !mapEquals(o.markColours, markColours) || !identical(o.beats, beats);
}

// ---------------------------------------------------------------- tracks

Color wtBand(int i, bool selected) => Forms.worn.trackBand(i, selected);

/// The bar's fill: the kind's hue, a step quieter when not selected; a hidden or soloed-out layer drops to grey.
Color wtBarFill(WtRow r) => Forms.worn.trackFill(r.tone ?? wsTone(r.kind), dim: r.dim, selected: r.selected);

// A way drawn from x [xa] to [xb]: as fine as the screen shows, where it is on the screen. What is left at 192 steps is more
// than a short stretch has pixels for, and a long one still has its curve; Skia draws a curve as such lines too.
void _wayLine(Path p, List<double> way, double xa, double xb, double width, double Function(double) y) {
  final n = way.length - 1, wide = xb - xa;
  if (wide <= 0) return;
  final every = math.max(1, (n * 1.5 / wide).floor());
  final from = ((-xa / wide * n).floor() - every).clamp(0, n), to = (((width - xa) / wide * n).ceil() + every).clamp(0, n);
  for (var i = from + every; i < to; i += every) {
    p.lineTo(xa + wide * i / n, y(way[i]));
  }
  p.lineTo(xa + wide * to / n, y(way[to]));
}

class WtTracksPainter extends CustomPainter {
  const WtTracksPainter({required this.rows, required this.view, required this.fps, required this.vscroll, this.tempo, this.restAt});
  final WsBeats? tempo;
  final int? restAt;   // (where the hand has come to rest: Alight Motion's SnapLine)
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
      c.drawRect(Rect.fromLTWH(0, top + WtM.row - 1, s.width, 1), p..color = Grey.rowLine);
    }
    if (end < s.width) c.drawRect(Rect.fromLTRB(end, 0, s.width, s.height), p..color = WsT.body);
    _vLines(c, view, fps, math.min(s.width, end), 0, s.height, major: Grey.g20, fine: Grey.g15);
    for (var i = 0; i < rows.length; i++) {
      final top = i * WtM.row - vscroll;
      if (top > s.height || top + WtM.row < 0) continue;
      _row(c, s, rows[i], top + WtM.row / 2 - .5);
    }
    if (restAt case final f?) c.drawRect(Rect.fromLTWH(view.x(f).roundToDouble(), 0, 1, s.height), p..color = WsT.accent);
  }

  void _row(Canvas c, Size s, WtRow r, double cy) {
    if (r.lane case final pts? when pts.isNotEmpty) return _lane(c, s, r, pts, cy);
    final p = Paint(), x0 = view.x(r.inF), x1 = math.max(view.x(r.outF), x0 + 4), h = Forms.worn.trackBar(selected: r.selected);
    if (r.whole case (final a, final b)?) c.drawRect(Rect.fromLTRB(view.x(a), cy - h / 2, view.x(b), cy + h / 2), p..color = wtBarFill(r).withValues(alpha: .18));
    if (x1 < 0 || x0 > s.width) return;
    final bar = Rect.fromLTRB(x0, cy - h / 2, x1, cy + h / 2), rr = RRect.fromRectAndRadius(bar, Radius.circular(Forms.worn.trackRadius));
    final fill = wtBarFill(r), ink = r.dim ? Grey.g56 : WsT.onAccent, grip = Color.lerp(fill, Grey.g00, .28)!;
    final mix = r.mix;
    if (mix != null && mix.length > 1 && !r.dim) {   // (a relation: its things' colours side by side, blended)
      c.drawRRect(rr, Paint()..shader = LinearGradient(colors: [for (final m in mix) r.selected ? Color.lerp(m, const Color(0xFFFFFFFF), .18)! : m]).createShader(bar));
    } else {
      c.drawRRect(rr, p..color = fill);
    }
    c.save();
    c.clipRRect(rr);
    c.drawRect(Rect.fromLTWH(x0, bar.top, 5, bar.height), p..color = grip);   // (the ends a hand cuts the bar by: wide, with a light notch)
    c.drawRect(Rect.fromLTWH(x1 - 5, bar.top, 5, bar.height), p..color = grip);
    final notch = ink.withAlpha(r.selected ? 200 : 120);
    c.drawRect(Rect.fromLTWH(x0 + 2, bar.top + 4, 1.5, bar.height - 8), p..color = notch);
    c.drawRect(Rect.fromLTWH(x1 - 3.5, bar.top + 4, 1.5, bar.height - 8), p..color = notch);
    if (r.frames case final f? when f.length == 1) {   // (a photo: itself again and again along the bar at its own shape, as Alight Motion's photo bar)
      final im = f.single, step = math.max(4.0, bar.height * im.width / math.max(1, im.height));
      for (var x = bar.left + 5; x < bar.right - 5; x += step) {
        paintImage(canvas: c, rect: Rect.fromLTWH(x, bar.top, step, bar.height), image: im, fit: BoxFit.cover, opacity: r.dim ? .35 : .8, filterQuality: FilterQuality.low);
      }
    } else if (r.frames case final f? when f.isNotEmpty) {   // (each picture filling its part of the bar, its middle kept: a strip of frames, as Alight Motion's)
      final step = (bar.width - 10) / f.length;
      for (final (i, im) in f.indexed) {
        final at = Rect.fromLTWH(bar.left + 5 + i * step, bar.top, step, bar.height);
        paintImage(canvas: c, rect: at, image: im, fit: BoxFit.cover, opacity: r.dim ? .35 : .8, filterQuality: FilterQuality.low);
      }
    }
    if (r.wave case final w?) _wave(c, bar, ink, fill, r.inF, w);
    if (r.clipped) {   // (↘: cut to the one under it)
      final at = Offset(math.max(x0 + 8, 6.0), cy - 4), arrow = Paint()..color = ink..strokeWidth = 1.4..style = PaintingStyle.stroke..strokeCap = StrokeCap.round;
      c.drawLine(at, at + const Offset(7, 7), arrow);
      c.drawPath(Path()..moveTo(at.dx + 7, at.dy + 3)..lineTo(at.dx + 7, at.dy + 7)..lineTo(at.dx + 3, at.dy + 7), arrow);
    }
    final nameX = math.max(x0 + 8, 6.0) + (r.clipped ? 12 : 0);
    final t = _text(r.name, T.name(ink).copyWith(fontWeight: FontWeight.w700));
    if (nameX + 20 < x1) {
      if (r.wave != null || r.frames != null) c.drawRRect(RRect.fromLTRBR(nameX - 4, cy - 6.5, nameX + t.width + 4, cy + 6.5, Shape.r(3)), p..color = fill);   // (its name on its own ground, over its sound or its pictures)
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
    for (final st in r.stretches) {   // (each stretch's way, from its key to the next, as a line inside the bar: how it goes there)
      final xa = view.x(st.from), xb = view.x(st.to);
      if (xb - xa < 6 || xb < 0 || xa > s.width) continue;
      final h = bar.height * .62, base = cy + h / 2;
      final line = Path()..moveTo(xa, base - st.shape.first.clamp(-.5, 1.5) * h);
      _wayLine(line, st.shape, xa, xb, s.width, (v) => base - v.clamp(-.5, 1.5) * h);
      c.drawPath(line, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.2..strokeJoin = StrokeJoin.round..isAntiAlias = true..color = Color.lerp(fill, Grey.g100, r.dim ? .25 : .6)!);   // (Ableton's automation line: one thin line, the bar's own colour made lighter)
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

  /// A value's lane, as Ableton draws an envelope in its own lane (Live 12 manual 25.5): its keys as breakpoints at
  /// their values, lowest at the bottom, joined as each goes there (its way's shape: Ableton draws a curved segment
  /// curved, 25.5.2; the bar's own stretches the same), level before the first and after the last; the line in the
  /// bar's colour made lighter, as the bar's own automation line (golf_noise.md ⑫: it was straight whatever the ease).
  void _lane(Canvas c, Size s, WtRow r, List<(int, double)> pts, double cy) {
    final top = cy - WtM.row / 2 + 4, h = WtM.row - 9, col = Color.lerp(wtBarFill(r), Grey.g100, r.dim ? .25 : .6)!;
    Offset at((int, double) q) => Offset(view.x(q.$1), top + (1 - q.$2) * h);
    final line = Path()..moveTo(0, at(pts.first).dy);
    for (var i = 0; i < pts.length; i++) {
      final a = at(pts[i]), way = i + 1 < pts.length ? r.laneWays?.elementAtOrNull(i) : null;
      line.lineTo(a.dx, a.dy);
      if (way == null) continue;
      final b = at(pts[i + 1]);
      _wayLine(line, way, a.dx, b.dx, s.width, (v) => (a.dy + (b.dy - a.dy) * v).clamp(top - 3, top + h + 3));   // (an overshoot kept within its own lane)
    }
    line.lineTo(s.width, at(pts.last).dy);
    if (r.laneOn case final i? when i + 1 < pts.length) c.drawRect(Rect.fromLTRB(at(pts[i]).dx, cy - WtM.row / 2, at(pts[i + 1]).dx, cy + WtM.row / 2), Paint()..color = WsT.accent.withValues(alpha: .18));   // (Theatre.js's connector, lit while it is what the curve editor holds)
    c.drawPath(line, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.2..strokeJoin = StrokeJoin.round..isAntiAlias = true..color = col);
    for (final q in pts) {
      c.drawCircle(at(q), WtM.key * .75, Paint()..color = col);
    }
  }

  /// Its sound as the inside reads it: -60 dB to full across the bar's height, the loudest of each two points.
  void _wave(Canvas c, Rect bar, Color ink, Color fill, int inF, List<double> db) {
    final p = Paint()..color = Color.lerp(fill, ink, .7)!;
    final h = bar.height - 4, cy = bar.center.dy;
    for (var x = math.max(bar.left + 4, 0.0); x < bar.right - 4; x += 2) {
      final a0 = view.frameAt(x).floor() - inF, a1 = view.frameAt(x + 2).ceil() - inF;
      var loud = -100.0;
      for (var k = math.max(a0, 0); k <= a1 && k < db.length; k++) {
        if (db[k] > loud) loud = db[k];
      }
      final a = ((loud + 60) / 60).clamp(0.0, 1.0);
      if (a > 0) c.drawRect(Rect.fromCenter(center: Offset(x.roundToDouble(), cy), width: 1, height: (a * h).roundToDouble().clamp(1, h)), p);
    }
  }

  @override
  bool shouldRepaint(WtTracksPainter o) => o.view != view || o.fps != fps || o.vscroll != vscroll || o.restAt != restAt || !_listEq(o.rows, rows);
}

// ---------------------------------------------------------------- playhead

class WtPlayheadPainter extends CustomPainter {
  const WtPlayheadPainter({required this.view, required this.frame, required this.headTop, this.moment}) : super(repaint: moment);
  final WtView view;
  final int frame;
  final double headTop;
  final ValueListenable<int>? moment;   // (where it is now, read as it is painted: painted again as it moves, nothing built)

  @override
  void paint(Canvas c, Size s) {
    final frame = moment?.value ?? this.frame;
    final x = view.x(frame).roundToDouble();
    if (x < -40 || x > s.width + 40) return;
    final p = Paint()..color = WsT.playhead;
    // (a small head, no number on it: Premiere Pro's playhead, After Effects' time indicator, Ableton's and Alight
    //  Motion's, the moment read in the transport's time; one that carries its frame hid a marker's name under it,
    //  golf_noise.md ⑱ (Blender and Cavalry number theirs, their markers on a row of their own))
    final w = Shape.z(9), cx = (x + .5).clamp(w / 2, s.width - w / 2);
    final pill = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, headTop + 8), width: w, height: Shape.z(15)), Shape.r(4));
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
  }

  @override
  bool shouldRepaint(WtPlayheadPainter o) => o.view != view || o.frame != frame || o.headTop != headTop || o.moment != moment;
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
    c.drawRRect(RRect.fromLTRBR(4, 3, s.width - 4, s.height - 3, Shape.r(3)), p..color = Grey.g13);
    c.drawRRect(RRect.fromLTRBR(x(from), 3, math.max(x(to), x(from) + 12), s.height - 3, Shape.r(3)), p..color = hot ? Grey.g44 : Grey.g26);
    c.drawRect(Rect.fromLTWH(x(frame.toDouble()).roundToDouble(), 2, 2, s.height - 4), p..color = WsT.playhead);
  }

  @override
  bool shouldRepaint(WtOverviewPainter o) => o.from != from || o.to != to || o.frame != frame || o.hot != hot;
}

// ---------------------------------------------------------------- kind chip

/// A layer's kind: a small square of its hue with the kind drawn in the dark ink.
/// A row's own small picture, as Alight Motion shows one at the left of each row (reports/alight_ui/95 and 106): a
/// photo or a film its picture, a shape its form in its own fill; the rest, the mark of its kind (Alight Motion's
/// camera and sound are marks too; a text's is not seen in its pictures: its mark kept).
class WtThumb extends StatelessWidget {
  const WtThumb(this.row, {super.key});
  final WtRow row;
  @override
  Widget build(BuildContext context) {
    final r = row;
    if (r.frames case final f? when f.isNotEmpty) {
      return ClipRRect(
        borderRadius: BorderRadius.all(Shape.r(3)),
        child: SizedBox.square(dimension: WtM.chip, child: RawImage(image: f.first, fit: BoxFit.cover, opacity: AlwaysStoppedAnimation(r.dim ? .35 : 1))),
      );
    }
    if (r.form case final form? when r.kind == WsKind.shape && form != 'null') {
      return Container(
        width: WtM.chip,
        height: WtM.chip,
        decoration: BoxDecoration(color: Grey.g20, borderRadius: BorderRadius.all(Shape.r(3))),
        child: CustomPaint(painter: _FormPainter(form, (r.fill ?? wsTone(r.kind)).withValues(alpha: r.dim ? .35 : 1), outline: r.outline)),
      );
    }
    return WtKindChip(r.kind, dim: r.dim, tone: r.tone);
  }
}

class _FormPainter extends CustomPainter {
  const _FormPainter(this.form, this.ink, {this.outline = false});
  final String form;
  final Color ink;
  final bool outline;
  @override
  void paint(Canvas c, Size s) {
    final b = (Offset.zero & s).deflate(s.width * .16), st = Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeCap = StrokeCap.round;
    final f = outline ? st : (Paint()..color = ink);
    Path round(int n, double inner) => Path()
      ..addPolygon([
        for (var i = 0; i < n * (inner < 1 ? 2 : 1); i++)
          b.center + Offset.fromDirection(-math.pi / 2 + i * math.pi * 2 / (n * (inner < 1 ? 2 : 1)), b.width / 2 * (inner < 1 && i.isOdd ? inner : 1)),
      ], true);
    switch (form) {
      case 'rect' || 'solid':
        c.drawRect(b, f);
      case 'squircle' || 'chamfer':
        c.drawRRect(RRect.fromRectAndRadius(b, Radius.circular(b.width * .3)), f);
      case 'polygon':
        c.drawPath(round(6, 1), f);
      case 'star':
        c.drawPath(round(5, .45), f);
      case 'line':
        c.drawLine(b.bottomLeft, b.topRight, st);
      case 'ring':
        c.drawOval(b.deflate(1), st);
      default:   // (circle, and the forms with no small mark of their own: a round in its fill)
        c.drawOval(b, f);
    }
  }

  @override
  bool shouldRepaint(_FormPainter o) => o.form != form || o.ink != ink || o.outline != outline;
}

class WtKindChip extends StatelessWidget {
  const WtKindChip(this.kind, {super.key, this.dim = false, this.tone});
  final Color? tone;
  final WsKind kind;
  final bool dim;
  @override
  Widget build(BuildContext context) => Container(
    width: WtM.chip,
    height: WtM.chip,
    decoration: BoxDecoration(color: dim ? Grey.g26 : tone ?? wsTone(kind), borderRadius: BorderRadius.all(Shape.r(3))),
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
        c.drawRRect(RRect.fromLTRBR(w * .16, w * .32, w * .64, w * .7, Shape.r(1.5)), f);
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
