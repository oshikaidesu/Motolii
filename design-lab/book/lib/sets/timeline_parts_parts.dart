// Private helpers of the timeline_parts set: glyphs, hit/hover shell, key + bar + lane painters. Sizes are fractions of TL.* (DESIGN.md section 4).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../parts/controls.dart' show Look;
import '../tokens.dart';

const tpFast = Duration(milliseconds: 120);
const tpAccent = C.mode; // the panel's accent: a picked key, an armed toggle

/// frames -> mm:ss:ff
String tpTime(num frames, int fps) {
  final f = frames.round().clamp(0, 1 << 30);
  final sec = f ~/ fps;
  String two(int n) => n.toString().padLeft(2, '0');
  return '${two(sec ~/ 60)}:${two(sec % 60)}:${two(f % fps)}';
}

// ---------------------------------------------------------------- ticks (the rule of lib/parts/timeline.dart)

double tpMajor(double ppf, int fps) {
  final steps = {1, 2, 5, 10, fps ~/ 2, fps, 2 * fps, 5 * fps, 10 * fps, 30 * fps, 60 * fps}.where((f) => f > 0).toList()..sort();
  return steps.firstWhere((f) => f * ppf >= 70, orElse: () => steps.last).toDouble();
}

double tpFine(double ppf, int fps, double major) {
  final steps = {1, 2, 5, 10, fps ~/ 2, fps, 2 * fps, 5 * fps, 10 * fps}.where((f) => f > 0 && f < major && major % f == 0 && f * ppf >= 12);
  var best = major;
  for (final f in steps) {
    if ((f * ppf - 30).abs() < (best * ppf - 30).abs()) best = f.toDouble();
  }
  return best;
}

void tpGrid(Canvas c, Size s, double ppf, int fps) {
  final major = tpMajor(ppf, fps), fine = tpFine(ppf, fps, major), p = Paint();
  for (var f = 0.0; f * ppf < s.width; f += fine) {
    c.drawRect(Rect.fromLTWH((f * ppf).roundToDouble(), 0, 1, s.height), p..color = f % major == 0 ? N.glaze15 : N.glaze9);
  }
}

// ---------------------------------------------------------------- glyphs

enum TpGlyph { eye, eyeOff, solo, lock, unlock, caretR, caretD, stopwatch, play, pause, stop, toStart, toEnd, stepBack, stepFwd, loop, fit, plus, minus, keyOutline, keyFilled }

class TpGlyphPainter extends CustomPainter {
  const TpGlyphPainter(this.g, this.color);
  final TpGlyph g;
  final Color color;

  @override
  void paint(Canvas c, Size s) {
    c.translate(s.width / 2, s.height / 2);
    c.scale(s.shortestSide / 12);
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = 1.2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round..color = color;
    final fill = Paint()..color = color;
    Path tri(double x0, double y0, double x1, double y1, double x2, double y2) => Path()..moveTo(x0, y0)..lineTo(x1, y1)..lineTo(x2, y2)..close();
    switch (g) {
      case TpGlyph.eye:
        c.drawPath(Path()..moveTo(-5.5, 0)..quadraticBezierTo(0, -6, 5.5, 0)..quadraticBezierTo(0, 6, -5.5, 0), line);
        c.drawCircle(Offset.zero, 1.7, fill);
      case TpGlyph.eyeOff:
        c.drawPath(Path()..moveTo(-5.5, -1)..quadraticBezierTo(0, 4, 5.5, -1), line);
        c.drawLine(const Offset(-3, 2), const Offset(-4, 3.6), line);
        c.drawLine(const Offset(0, 3), const Offset(0, 4.8), line);
        c.drawLine(const Offset(3, 2), const Offset(4, 3.6), line);
      case TpGlyph.solo:
        c.drawCircle(Offset.zero, 4.6, line);
        c.drawCircle(Offset.zero, 1.8, fill);
      case TpGlyph.lock:
      case TpGlyph.unlock:
        c.drawRRect(RRect.fromLTRBR(-3.6, 0, 3.6, 5.2, const Radius.circular(1.2)), line);
        final sh = Path()..moveTo(-2.3, 0)..lineTo(-2.3, -1.8)..arcTo(Rect.fromCircle(center: const Offset(0, -1.8), radius: 2.3), math.pi, math.pi, false);
        if (g == TpGlyph.lock) sh.lineTo(2.3, 0);
        c.drawPath(sh, line);
      case TpGlyph.caretR:
        c.drawPath(tri(-2, -3.6, 2.8, 0, -2, 3.6), fill);
      case TpGlyph.caretD:
        c.drawPath(tri(-3.6, -2, 3.6, -2, 0, 2.8), fill);
      case TpGlyph.stopwatch:
        c.drawCircle(const Offset(0, 1), 4.2, line);
        c.drawLine(const Offset(-1.6, -5.2), const Offset(1.6, -5.2), line);
        c.drawLine(const Offset(0, 1), const Offset(1.9, -.8), line);
      case TpGlyph.play:
        c.drawPath(tri(-3, -4.6, 4.8, 0, -3, 4.6), fill);
      case TpGlyph.pause:
        c.drawRect(const Rect.fromLTRB(-3.6, -4.4, -1, 4.4), fill);
        c.drawRect(const Rect.fromLTRB(1, -4.4, 3.6, 4.4), fill);
      case TpGlyph.stop:
        c.drawRRect(RRect.fromLTRBR(-4, -4, 4, 4, const Radius.circular(1)), fill);
      case TpGlyph.toStart:
        c.drawRect(const Rect.fromLTRB(-5, -4.6, -3.8, 4.6), fill);
        c.drawPath(tri(5, -4.6, -2.2, 0, 5, 4.6), fill);
      case TpGlyph.toEnd:
        c.drawRect(const Rect.fromLTRB(3.8, -4.6, 5, 4.6), fill);
        c.drawPath(tri(-5, -4.6, 2.2, 0, -5, 4.6), fill);
      case TpGlyph.stepBack:
        c.drawPath(tri(3, -4.2, -3, 0, 3, 4.2), fill);
      case TpGlyph.stepFwd:
        c.drawPath(tri(-3, -4.2, 3, 0, -3, 4.2), fill);
      case TpGlyph.loop:
        c.drawRRect(RRect.fromLTRBR(-5, -3, 5, 3, const Radius.circular(3)), line);
        c.drawPath(tri(1.5, -5.6, 4.6, -3, 1.5, -.4), fill);
      case TpGlyph.fit:
        for (final sx in const [-1.0, 1.0]) {
          for (final sy in const [-1.0, 1.0]) {
            c.drawPath(Path()..moveTo(sx * 5, sy * 1.8)..lineTo(sx * 5, sy * 5)..lineTo(sx * 1.8, sy * 5), line);
          }
        }
      case TpGlyph.plus:
        c.drawLine(const Offset(-4, 0), const Offset(4, 0), line);
        c.drawLine(const Offset(0, -4), const Offset(0, 4), line);
      case TpGlyph.minus:
        c.drawLine(const Offset(-4, 0), const Offset(4, 0), line);
      case TpGlyph.keyOutline:
      case TpGlyph.keyFilled:
        c.drawPath(Path()..moveTo(0, -4.4)..lineTo(4.4, 0)..lineTo(0, 4.4)..lineTo(-4.4, 0)..close(), g == TpGlyph.keyFilled ? fill : line);
    }
  }

  @override
  bool shouldRepaint(TpGlyphPainter o) => o.g != g || o.color != color;
}

class TpGlyphView extends StatelessWidget {
  const TpGlyphView(this.g, this.color, {super.key, this.size = 12});
  final TpGlyph g;
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => TweenAnimationBuilder<Color?>(
        tween: ColorTween(end: color),
        duration: tpFast,
        curve: Curves.easeOut,
        builder: (_, v, __) => CustomPaint(size: Size.square(size), painter: TpGlyphPainter(g, v ?? color)),
      );
}

// ---------------------------------------------------------------- interaction shells

/// Hover + pressed shell. The builder decides how each state looks.
class TpHit extends StatefulWidget {
  const TpHit({super.key, required this.builder, this.onTap, this.cursor = SystemMouseCursors.click});
  final Widget Function(BuildContext, bool hover, bool down) builder;
  final VoidCallback? onTap;
  final MouseCursor cursor;
  @override
  State<TpHit> createState() => _TpHitState();
}

class _TpHitState extends State<TpHit> {
  bool _h = false, _d = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.cursor,
        onEnter: (_) => setState(() => _h = true),
        onExit: (_) => setState(() => _h = false),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: (_) => setState(() => _d = true),
          onTapUp: (_) => setState(() => _d = false),
          onTapCancel: () => setState(() => _d = false),
          onTap: widget.onTap,
          child: widget.builder(context, _h, _d),
        ),
      );
}

/// Local state seeded from a knob. Give it a ValueKey of the knob values so a knob change reseeds it.
class TpValue<V> extends StatefulWidget {
  const TpValue({super.key, required this.initial, required this.builder});
  final V initial;
  final Widget Function(BuildContext, V value, ValueChanged<V> set) builder;
  @override
  State<TpValue<V>> createState() => _TpValueState<V>();
}

class _TpValueState<V> extends State<TpValue<V>> {
  late V v = widget.initial;
  @override
  Widget build(BuildContext context) => widget.builder(context, v, (n) => setState(() => v = n));
}

class TpIconButton extends StatelessWidget {
  const TpIconButton(this.g, {super.key, this.onTap, this.on = false, this.onColor = tpAccent, this.size = 16, this.glyph = 12, this.forceHover = false, this.dim = false, this.rest});
  final TpGlyph g;
  final VoidCallback? onTap;
  final bool on, forceHover, dim;
  final Color onColor;
  final Color? rest;
  final double size, glyph;
  @override
  Widget build(BuildContext context) => TpHit(
        onTap: onTap,
        builder: (_, h, d) {
          final hot = h || forceHover;
          final ink = on ? onColor : d ? N.g100 : hot ? N.g95 : dim ? N.g38 : (rest ?? N.g56);
          return AnimatedContainer(
            duration: tpFast,
            curve: Curves.easeOut,
            width: size,
            height: size,
            alignment: Alignment.center,
            decoration: BoxDecoration(color: d ? N.g20 : (h ? N.g15 : const Color(0x00262626)), borderRadius: BorderRadius.circular(3)),
            child: TpGlyphView(g, ink, size: glyph),
          );
        },
      );
}

// ---------------------------------------------------------------- keys

/// One shape: the diamond. default / selected (picked) / at-playhead / ghost (a key of another layer, faint).
enum TpKey { normal, selected, atHead, ghost }

/// How a segment between two keys moves. Shown by a small mark on the link line, never by the key's shape. Linear is the plain line.
enum TpInterp { linear, hold, eased, bezier }

/// r is the half-width of a normal key (TL.key / 2 at row scale). Edge widths stay at row scale so a large key keeps the same line.
void tpKey(Canvas c, Offset p, double r, TpKey k) {
  final big = k == TpKey.atHead ? r * 1.2 : r;
  final path = Path()..moveTo(p.dx, p.dy - big)..lineTo(p.dx + big, p.dy)..lineTo(p.dx, p.dy + big)..lineTo(p.dx - big, p.dy)..close();
  if (k == TpKey.ghost) {
    // a dashed dim ring: visible on any bar, never mistaken for a real key
    final ring = Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g76.withValues(alpha: .6);
    for (final m in path.computeMetrics()) {
      for (var d = 0.0; d < m.length; d += 4) {
        c.drawPath(m.extractPath(d, math.min(d + 2, m.length)), ring);
      }
    }
    return;
  }
  c.drawPath(path, Paint()..color = k == TpKey.atHead ? C.playhead : k == TpKey.selected ? N.g100 : N.g95.withValues(alpha: .9));
  c.drawPath(path, Paint()..style = PaintingStyle.stroke..strokeWidth = k == TpKey.selected ? 2 : k == TpKey.atHead ? 1.5 : .75..color = k == TpKey.selected ? C.mode : k == TpKey.atHead ? N.g100 : N.g100.withValues(alpha: .6));
}

/// The link line between keys, cut where a ghost key sits so it does not show through.
void tpLink(Canvas c, List<double> xs, double cy, Set<int> cut, double r, double alpha) {
  final p = Paint()..color = N.g100.withValues(alpha: alpha);
  for (var i = 0; i < xs.length - 1; i++) {
    final x0 = xs[i] + (cut.contains(i) ? r : 0), x1 = xs[i + 1] - (cut.contains(i + 1) ? r : 0);
    if (x1 > x0) c.drawRect(Rect.fromLTRB(x0, cy - .5, x1, cy + .5), p);
  }
}

/// The mark on the link line, centred on the segment: hold = a step, eased = a smooth S, bezier = an S with its two anchor dots.
void tpSeg(Canvas c, Offset m, TpInterp k, [double sc = 1]) {
  if (k == TpInterp.linear) return;
  c.save();
  c.translate(m.dx, m.dy);
  c.scale(sc);
  c.drawRRect(RRect.fromLTRBR(-6, -5, 6, 5, const Radius.circular(2)), Paint()..color = N.g07.withValues(alpha: .92));
  final l = Paint()..style = PaintingStyle.stroke..strokeWidth = 1..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round..color = N.g95;
  final path = Path()..moveTo(-3.5, 2.5);
  switch (k) {
    case TpInterp.hold:
      path..lineTo(0, 2.5)..lineTo(0, -2.5)..lineTo(3.5, -2.5);
    case TpInterp.eased:
      path.cubicTo(0, 2.5, 0, -2.5, 3.5, -2.5);
    case TpInterp.bezier:
    case TpInterp.linear:
      path.cubicTo(-.5, 3.4, .5, -3.4, 3.5, -2.5);
  }
  c.drawPath(path, l);
  if (k == TpInterp.bezier) {
    c.drawCircle(const Offset(-3.5, 2.5), 1, Paint()..color = N.g95);
    c.drawCircle(const Offset(3.5, -2.5), 1, Paint()..color = N.g95);
  }
  c.restore();
}

// ---------------------------------------------------------------- bars

enum TpBar { normal, trimmed, selected, ghost, group, camera, audio }

Color tpBarColor(Fam f, {bool selected = false, bool hover = false, Look look = Look.concept}) {
  final h = HSLColor.fromColor(f.c);
  if (selected) return f.c;
  var s = h.saturation * (hover ? .95 : .85), l = h.lightness * .93;
  if (look == Look.quiet) {
    s *= .8;
    l *= .94;
  }
  return h.withSaturation(s.clamp(0.0, 1.0)).withLightness(l.clamp(0.0, 1.0)).toColor();
}

void tpBar(Canvas c, Rect r, Fam f, TpBar kind, {bool hover = false, Look look = Look.concept, double cut = 20}) {
  final p = Paint(), rr = RRect.fromRectAndRadius(r, const Radius.circular(TL.radius));
  final edges = look != Look.quiet;
  void rim(Rect rect) {
    c.drawRect(Rect.fromLTWH(rect.left, rect.top, rect.width, 1), p..color = N.g100.withValues(alpha: .16));
    c.drawRect(Rect.fromLTWH(rect.left, rect.bottom - 1, rect.width, 1), p..color = N.g100.withValues(alpha: .16));
  }

  switch (kind) {
    case TpBar.ghost:
      c.drawRRect(rr, p..color = f.c.withValues(alpha: hover ? .26 : .18));
      c.drawRRect(rr.deflate(.5), Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = f.c.withValues(alpha: .55));
    case TpBar.group:
      final top = Rect.fromLTRB(r.left, r.top, r.right, r.top + r.height * .5);
      c.drawRRect(RRect.fromRectAndRadius(top, const Radius.circular(TL.radius)), p..color = tpBarColor(f, hover: hover, look: look));
      for (final x in [r.left, r.right - 2]) {
        c.drawRect(Rect.fromLTWH(x, r.top, 2, r.height), p..color = tpBarColor(f, hover: hover, look: look));
      }
    case TpBar.camera:
      c.drawRRect(rr, p..color = hover ? N.g38 : N.g26);
      if (edges) {
        c.save();
        c.clipRRect(rr);
        rim(r);
        c.restore();
      }
    case TpBar.audio:
      c.drawRRect(rr, p..color = hover ? N.g26 : N.g20);
      c.save();
      c.clipRRect(rr);
      final cy = r.center.dy;
      for (var x = r.left + 2; x < r.right - 1; x += 2) {
        final a = (math.sin(x * .37).abs() * .5 + math.sin(x * .11 + 1).abs() * .35 + math.sin(x * 1.3).abs() * .15).clamp(.08, 1.0);
        c.drawRect(Rect.fromCenter(center: Offset(x, cy), width: 1, height: a * (r.height - 4)), p..color = hover ? N.g63 : N.g56);
      }
      c.restore();
    case TpBar.trimmed:
    case TpBar.normal:
    case TpBar.selected:
      if (kind == TpBar.trimmed) {
        final src = Rect.fromLTRB(r.left - cut, r.top + 3, r.right + cut, r.bottom - 3);
        c.drawRRect(RRect.fromRectAndRadius(src, const Radius.circular(TL.radius)), p..color = f.c.withValues(alpha: .16));
      }
      final sel = kind == TpBar.selected;
      if (sel && look == Look.glow) {
        c.drawRRect(rr.inflate(1), Paint()..color = f.c.withValues(alpha: .55)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
      }
      c.drawRRect(rr, p..color = tpBarColor(f, selected: sel, hover: hover, look: look));
      if (sel) c.drawRRect(rr.deflate(.5), Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g95.withValues(alpha: .55));
      c.save();
      c.clipRRect(rr);
      if (edges) rim(r);
      if (kind == TpBar.trimmed) {
        c.drawRect(Rect.fromLTWH(r.left, r.top, 2, r.height), p..color = N.g100.withValues(alpha: .4));
        c.drawRect(Rect.fromLTWH(r.right - 2, r.top, 2, r.height), p..color = N.g100.withValues(alpha: .4));
      }
      c.restore();
  }
}

// ---------------------------------------------------------------- lane (grid + bar + keys + link line + playhead line)

class TpLanePainter extends CustomPainter {
  const TpLanePainter({required this.fam, this.kind = TpBar.normal, this.start = 10, this.end = 90, this.keys = const [], this.keyKinds = const [], this.head, this.ppf = 6, this.fps = 30, this.hover = false, this.look = Look.concept, this.grid = true, this.link = true, this.interps = const [], this.showBar = true});
  final Fam fam;
  final TpBar kind;
  final double start, end, ppf;
  final List<double> keys;
  final List<TpKey> keyKinds;
  final List<TpInterp> interps;
  final bool showBar;
  final int? head;
  final int fps;
  final bool hover, grid, link;
  final Look look;

  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    final cy = s.height / 2, p = Paint();
    if (grid) tpGrid(c, s, ppf, fps);
    final bar = Rect.fromLTRB(start * ppf, cy - TL.bar / 2, math.max(end * ppf, start * ppf + 3), cy + TL.bar / 2);
    if (showBar) tpBar(c, bar, fam, kind, hover: hover, look: look);
    if (kind != TpBar.audio && keys.isNotEmpty) {
      if (link && keys.length > 1) tpLink(c, [for (final k in keys) k * ppf], cy, {for (var i = 0; i < keyKinds.length && i < keys.length; i++) if (keyKinds[i] == TpKey.ghost) i}, TL.key / 2, kind == TpBar.selected ? .9 : .7);
      for (var i = 0; i < keys.length - 1 && i < interps.length; i++) {
        if ((keys[i + 1] - keys[i]) * ppf > 24) tpSeg(c, Offset((keys[i] + keys[i + 1]) / 2 * ppf, cy), interps[i]);
      }
      for (var i = 0; i < keys.length; i++) {
        final k = i < keyKinds.length ? keyKinds[i] : TpKey.normal;
        tpKey(c, Offset(keys[i] * ppf, cy), TL.key / 2, head != null && keys[i].round() == head ? TpKey.atHead : k);
      }
    }
    if (head != null) c.drawRect(Rect.fromLTWH(head! * ppf, 0, 1, s.height), p..color = C.playhead);
  }

  @override
  bool shouldRepaint(TpLanePainter o) => true;
}

/// The playhead's head handle: a shield, top-centred on x. 11 x 13.
void tpHead(Canvas c, double x, {bool lit = false}) {
  final path = Path()..moveTo(x - 5.5, 0)..lineTo(x + 5.5, 0)..lineTo(x + 5.5, 7.5)..lineTo(x, 13)..lineTo(x - 5.5, 7.5)..close();
  c.drawPath(path, Paint()..color = C.playhead);
  c.drawPath(path, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeJoin = StrokeJoin.round..color = lit ? N.g100 : C.playhead);
  c.drawRect(Rect.fromLTWH(x - 3, 3, 6, 1), Paint()..color = N.g100.withValues(alpha: .45));
}
