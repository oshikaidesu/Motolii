// Private helpers for stage_set.dart: the stage chrome's shared pieces (plates, glyphs, hover, a sample artwork).
// The chrome rule: it floats over artwork, so it is translucent grey with a 1px hairline for legibility; depth is grey level,
// only floating plates carry a shadow.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../parts/controls.dart';
import '../tokens.dart';

// Axis colours: relation families reused as axis marks (X = Scatter, Y = Along Path, Z = Stagger). Not a relation here.
final Color axisX = Fam.scatter.c, axisY = Fam.along.c, axisZ = Fam.stagger.c;

/// The selection / guide accent. C.mode is the only non-family accent the tokens name.
const Color accent = C.mode;
/// Snapping is magenta by convention; the tokens' nearest magenta is the record pink.
const Color magenta = C.record;

// [open] the stage chrome's sizes; no token names them.
const double kHandle = 6, kHandleHit = 16, kTool = 28, kPlateRadius = 6;
const Duration kFast = Duration(milliseconds: 120);

Color hair([double a = .12]) => N.g100.withValues(alpha: a);
Color shade([double a = .35]) => N.g00.withValues(alpha: a);
Offset rot(Offset v, double a) => Offset(v.dx * math.cos(a) - v.dy * math.sin(a), v.dx * math.sin(a) + v.dy * math.cos(a));

/// A stroke that stays legible on any artwork: a soft dark underlay, then the colour.
void legible(Canvas c, Path p, Color color, {double w = 1}) {
  c.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = w + 2..strokeJoin = StrokeJoin.round..color = shade(.35));
  c.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = w..strokeJoin = StrokeJoin.round..strokeCap = StrokeCap.round..color = color);
}

void legibleLine(Canvas c, Offset a, Offset b, Color color, {double w = 1}) => legible(c, Path()..moveTo(a.dx, a.dy)..lineTo(b.dx, b.dy), color, w: w);

/// The floating plate every piece of stage chrome sits on. The only thing here with a shadow.
BoxDecoration plateDeco(Look look, {double radius = kPlateRadius}) {
  final r = BorderRadius.circular(radius);
  return switch (look) {
    Look.concept => BoxDecoration(color: N.g10.withValues(alpha: .88), borderRadius: r, border: Border.all(color: hair(.10)), boxShadow: [BoxShadow(color: shade(.35), blurRadius: 8, offset: const Offset(0, 2))]),
    Look.quiet => BoxDecoration(color: N.g10.withValues(alpha: .62), borderRadius: r, border: Border.all(color: hair(.06))),
    Look.glow => BoxDecoration(color: N.g07.withValues(alpha: .92), borderRadius: r, border: Border.all(color: accent.withValues(alpha: .40)), boxShadow: [BoxShadow(color: accent.withValues(alpha: .20), blurRadius: 12)]),
  };
}

class Plate extends StatelessWidget {
  const Plate({super.key, required this.child, this.look = Look.concept, this.padding = const EdgeInsets.all(4), this.radius = kPlateRadius});
  final Widget child;
  final Look look;
  final EdgeInsets padding;
  final double radius;
  @override
  Widget build(BuildContext context) => Container(padding: padding, decoration: plateDeco(look, radius: radius), child: child);
}

/// Holds a value the use case edits; the knob's value resets it when the knob changes.
class SpLive<V> extends StatefulWidget {
  const SpLive({super.key, required this.initial, required this.builder});
  final V initial;
  final Widget Function(BuildContext, V value, ValueChanged<V> set) builder;
  @override
  State<SpLive<V>> createState() => _SpLiveState<V>();
}

class _SpLiveState<V> extends State<SpLive<V>> {
  late V v = widget.initial;
  @override
  void didUpdateWidget(SpLive<V> old) {
    super.didUpdateWidget(old);
    if (old.initial != widget.initial) v = widget.initial;
  }

  @override
  Widget build(BuildContext context) => widget.builder(context, v, (n) => setState(() => v = n));
}

class Hover extends StatefulWidget {
  const Hover({super.key, required this.builder, this.onTap, this.cursor = SystemMouseCursors.click});
  final Widget Function(BuildContext, bool hovered) builder;
  final VoidCallback? onTap;
  final MouseCursor cursor;
  @override
  State<Hover> createState() => _HoverState();
}

class _HoverState extends State<Hover> {
  bool h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.cursor,
        onEnter: (_) => setState(() => h = true),
        onExit: (_) => setState(() => h = false),
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(context, h)),
      );
}

/// A sample artwork: tinted layers on a dark ground, so chrome is judged over something that looks like work.
class SampleArt extends StatelessWidget {
  const SampleArt({super.key, this.child, this.shift = Offset.zero});
  final Widget? child;
  final Offset shift;
  @override
  Widget build(BuildContext context) => Stack(fit: StackFit.expand, children: [CustomPaint(painter: _ArtPainter(shift)), if (child != null) child!]);
}

class _ArtPainter extends CustomPainter {
  _ArtPainter(this.shift);
  final Offset shift;
  @override
  void paint(Canvas c, Size s) {
    final r = Offset.zero & s;
    c.drawRect(r, Paint()..shader = LinearGradient(begin: Alignment.topLeft, end: Alignment.bottomRight, colors: [N.g13, N.g07]).createShader(r));
    void card(double fx, double fy, double fw, double fh, double a, Color col, double alpha) {
      c.save();
      c.translate(s.width * fx + shift.dx, s.height * fy + shift.dy);
      c.rotate(a);
      final rr = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: s.width * fw, height: s.height * fh), const Radius.circular(3));
      c.drawRRect(rr, Paint()..shader = LinearGradient(colors: [col.withValues(alpha: alpha), col.withValues(alpha: alpha * .35)]).createShader(rr.outerRect));
      c.restore();
    }

    card(.30, .45, .26, .42, -.30, Fam.stagger.c, .60);
    card(.64, .40, .20, .32, .25, Fam.scatter.c, .62);
    card(.50, .72, .22, .20, .08, Fam.attach.c, .50);
    card(.82, .74, .14, .22, -.15, Fam.face.c, .45);
    card(.14, .78, .16, .16, .35, Fam.along.c, .45);
    c.drawCircle(Offset(s.width * .5 + shift.dx, s.height * .3 + shift.dy), math.min(s.width, s.height) * .09, Paint()..color = Fam.follow.c.withValues(alpha: .5));
  }

  @override
  bool shouldRepaint(_ArtPainter o) => o.shift != shift;
}

enum Glyph { select, move, shape, ellipse, pen, text, crop, grid, safe, fit, one, plus, minus, chevron, snap, pivot }

class GlyphPainter extends CustomPainter {
  GlyphPainter(this.g, this.color);
  final Glyph g;
  final Color color;
  @override
  void paint(Canvas c, Size s) {
    c.scale(s.width / 16, s.height / 16);
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round..color = color;
    final fill = Paint()..color = color;
    void l(double a, double b, double x, double y) => c.drawLine(Offset(a, b), Offset(x, y), line);
    switch (g) {
      case Glyph.select:
        c.drawPath(Path()..moveTo(4, 2.5)..lineTo(4, 13)..lineTo(6.8, 10.4)..lineTo(8.8, 14)..lineTo(10.6, 13.1)..lineTo(8.6, 9.6)..lineTo(12.4, 9.6)..close(), line);
      case Glyph.move:
        l(8, 2, 8, 14);
        l(2, 8, 14, 8);
        c.drawPath(Path()..moveTo(6, 4)..lineTo(8, 2)..lineTo(10, 4)..moveTo(6, 12)..lineTo(8, 14)..lineTo(10, 12)..moveTo(4, 6)..lineTo(2, 8)..lineTo(4, 10)..moveTo(12, 6)..lineTo(14, 8)..lineTo(12, 10), line);
      case Glyph.shape:
        c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTRB(2.5, 3.5, 13.5, 12.5), const Radius.circular(1.5)), line);
      case Glyph.ellipse:
        c.drawOval(const Rect.fromLTRB(2.5, 3.5, 13.5, 12.5), line);
      case Glyph.pen:
        c.drawPath(Path()..moveTo(3, 13)..lineTo(4, 9)..lineTo(10, 3)..lineTo(13, 6)..lineTo(7, 12)..close(), line);
        l(3, 13, 7.5, 8.5);
      case Glyph.text:
        l(3.5, 3.5, 12.5, 3.5);
        l(8, 3.5, 8, 12.5);
        l(6, 12.5, 10, 12.5);
      case Glyph.crop:
        c.drawPath(Path()..moveTo(5, 2)..lineTo(5, 11)..lineTo(14, 11)..moveTo(2, 5)..lineTo(11, 5)..lineTo(11, 14), line);
      case Glyph.grid:
        l(2.5, 6, 13.5, 6);
        l(2.5, 10, 13.5, 10);
        l(6, 2.5, 6, 13.5);
        l(10, 2.5, 10, 13.5);
      case Glyph.safe:
        c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTRB(2, 3.5, 14, 12.5), const Radius.circular(1)), line);
        c.drawRect(const Rect.fromLTRB(4.5, 5.5, 11.5, 10.5), line);
      case Glyph.fit:
        c.drawPath(Path()..moveTo(2.5, 6)..lineTo(2.5, 2.5)..lineTo(6, 2.5)..moveTo(10, 2.5)..lineTo(13.5, 2.5)..lineTo(13.5, 6)..moveTo(13.5, 10)..lineTo(13.5, 13.5)..lineTo(10, 13.5)..moveTo(6, 13.5)..lineTo(2.5, 13.5)..lineTo(2.5, 10), line);
      case Glyph.one:
        l(6, 5, 8, 3.5);
        l(8, 3.5, 8, 12.5);
      case Glyph.plus:
        l(3, 8, 13, 8);
        l(8, 3, 8, 13);
      case Glyph.minus:
        l(3, 8, 13, 8);
      case Glyph.chevron:
        c.drawPath(Path()..moveTo(4.5, 6.5)..lineTo(8, 10)..lineTo(11.5, 6.5), line);
      case Glyph.snap:
        c.drawArc(const Rect.fromLTRB(3, 3, 13, 13), math.pi, math.pi, false, line);
        l(3, 8, 3, 13);
        l(13, 8, 13, 13);
      case Glyph.pivot:
        c.drawCircle(const Offset(8, 8), 4, line);
        l(8, 1.5, 8, 4);
        l(8, 12, 8, 14.5);
        l(1.5, 8, 4, 8);
        l(12, 8, 14.5, 8);
        c.drawCircle(const Offset(8, 8), 1, fill);
    }
  }

  @override
  bool shouldRepaint(GlyphPainter o) => o.g != g || o.color != color;
}

class GlyphIcon extends StatelessWidget {
  const GlyphIcon(this.g, {super.key, this.color = N.g76, this.size = 16});
  final Glyph g;
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => CustomPaint(size: Size.square(size), painter: GlyphPainter(g, color));
}

/// A square chrome button: hover lifts to a faint white, active is the accent at low alpha. Optional tooltip to the right.
class ChromeButton extends StatelessWidget {
  const ChromeButton({super.key, required this.glyph, this.on = false, this.onTap, this.tip, this.keys, this.look = Look.concept, this.size = kTool, this.tipSide = AxisDirection.right});
  final Glyph glyph;
  final bool on;
  final VoidCallback? onTap;
  final String? tip, keys;
  final Look look;
  final double size;
  final AxisDirection tipSide;
  @override
  Widget build(BuildContext context) => Hover(
        onTap: onTap,
        builder: (context, h) => Stack(clipBehavior: Clip.none, children: [
          AnimatedContainer(
            duration: kFast,
            curve: Curves.easeOut,
            width: size,
            height: size,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: on ? accent.withValues(alpha: look == Look.quiet ? .20 : .32) : (h ? hair(.08) : hair(0)),
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: on && look == Look.glow ? accent.withValues(alpha: .6) : hair(0)),
            ),
            child: GlyphIcon(glyph, color: on || h ? N.g95 : N.g63),
          ),
          if (tip != null)
            Positioned(
              left: tipSide == AxisDirection.right ? size + 8 : null,
              right: tipSide == AxisDirection.left ? size + 8 : null,
              top: tipSide == AxisDirection.down ? size + 6 : 0,
              bottom: tipSide == AxisDirection.up ? size + 6 : (tipSide == AxisDirection.down ? null : 0),
              child: IgnorePointer(
                child: AnimatedOpacity(
                  opacity: h ? 1 : 0,
                  duration: kFast,
                  curve: Curves.easeOut,
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Plate(
                      look: look,
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                      radius: 4,
                      child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
                        Text(tip!, style: T.name(N.g95)),
                        if (keys != null) ...[const SizedBox(width: 8), Text(keys!, style: T.value(N.g56).copyWith(fontSize: 10))],
                      ]),
                    ),
                  ),
                ),
              ),
            ),
        ]),
      );
}

/// A small text segmented control for tool options: no accent, the active segment is a lifted grey.
class MiniSeg extends StatelessWidget {
  const MiniSeg({super.key, required this.items, required this.index, required this.onChanged});
  final List<String> items;
  final int index;
  final ValueChanged<int> onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: 24,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: shade(.28), borderRadius: BorderRadius.circular(5)),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < items.length; i++)
            Hover(
              onTap: () => onChanged(i),
              builder: (context, h) => AnimatedContainer(
                duration: kFast,
                curve: Curves.easeOut,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: i == index ? hair(.14) : (h ? hair(.06) : hair(0)), borderRadius: BorderRadius.circular(3)),
                child: Text(items[i], style: T.label(i == index ? N.g100 : N.g63)),
              ),
            ),
        ]),
      );
}

/// A label + number that scrubs on horizontal drag (the value is the loudest thing).
class Scrub extends StatelessWidget {
  const Scrub({super.key, required this.label, required this.value, required this.onChanged, this.step = 1, this.min = 0, this.max = 999, this.suffix = ''});
  final String label, suffix;
  final double value, step, min, max;
  final ValueChanged<double> onChanged;
  @override
  Widget build(BuildContext context) => Hover(
        cursor: SystemMouseCursors.resizeLeftRight,
        builder: (context, h) => GestureDetector(
          behavior: HitTestBehavior.opaque,
          onHorizontalDragUpdate: (d) => onChanged((value + d.delta.dx * step).clamp(min, max)),
          child: AnimatedContainer(
            duration: kFast,
            curve: Curves.easeOut,
            height: 24,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(color: h ? hair(.08) : shade(.28), borderRadius: BorderRadius.circular(4)),
            child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.baseline, textBaseline: TextBaseline.alphabetic, children: [
              Text(label, style: T.micro(N.g56)),
              const SizedBox(width: 8),
              Text('${value.toStringAsFixed(step < 1 ? 1 : 0)}$suffix', style: T.value(N.g95)),
            ]),
          ),
        ),
      );
}

class VDivider extends StatelessWidget {
  const VDivider({super.key});
  @override
  Widget build(BuildContext context) => Container(width: 1, height: 16, margin: const EdgeInsets.symmetric(horizontal: 4), color: hair(.10));
}

/// A value readout pill, floating: label + number.
class Readout extends StatelessWidget {
  const Readout(this.parts, {super.key, this.look = Look.concept});
  final List<(String, String, Color)> parts;
  final Look look;
  @override
  Widget build(BuildContext context) => Plate(
        look: look,
        radius: 4,
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < parts.length; i++) ...[
            if (i > 0) const SizedBox(width: 8),
            Text(parts[i].$1, style: T.micro(parts[i].$3)),
            const SizedBox(width: 4),
            Text(parts[i].$2, style: T.value(N.g95)),
          ],
        ]),
      );
}

/// A text label painted on a canvas.
void paintText(Canvas c, String s, Offset at, TextStyle st, {bool centerX = false, bool centerY = false}) {
  final tp = TextPainter(text: TextSpan(text: s, style: st), textDirection: TextDirection.ltr)..layout();
  tp.paint(c, at - Offset(centerX ? tp.width / 2 : 0, centerY ? tp.height / 2 : 0));
}

/// A white-ringed square handle, 6px. Hit area is larger. Fill goes solid on hover.
class RingHandle extends StatelessWidget {
  const RingHandle({super.key, required this.at, this.round = false, this.cursor = SystemMouseCursors.precise, this.onStart, this.onUpdate, this.onEnd, this.look = Look.concept});
  final Offset at;
  final bool round;
  final MouseCursor cursor;
  final VoidCallback? onStart, onEnd;
  final ValueChanged<DragUpdateDetails>? onUpdate;
  final Look look;
  @override
  Widget build(BuildContext context) => Positioned(
        left: at.dx - kHandleHit / 2,
        top: at.dy - kHandleHit / 2,
        width: kHandleHit,
        height: kHandleHit,
        child: Hover(
          cursor: cursor,
          builder: (context, h) => GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanStart: (_) => onStart?.call(),
            onPanUpdate: onUpdate,
            onPanEnd: (_) => onEnd?.call(),
            child: Center(
              child: AnimatedContainer(
                duration: kFast,
                curve: Curves.easeOut,
                width: kHandle,
                height: kHandle,
                decoration: BoxDecoration(
                  color: look == Look.quiet || h ? N.g100 : N.g10,
                  shape: round ? BoxShape.circle : BoxShape.rectangle,
                  border: Border.all(color: N.g100, width: 1.5),
                ),
              ),
            ),
          ),
        ),
      );
}
