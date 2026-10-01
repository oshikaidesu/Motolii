// Private helpers of the feedback set. Colours from N / Fam / C, text from T. A value no token names is `// [open]`.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../parts/controls.dart';
import '../tokens.dart';

enum Kind { info, success, warn, error }

enum Change { none, modified, added, removed }

enum QState { queued, rendering, done, failed }

enum ExportState { idle, exporting, done }

// [open] warn is a tint of Follow; the brief gives no token for it.
Color kindColor(Kind k) => switch (k) { Kind.info => C.playhead, Kind.success => C.play, Kind.warn => Fam.follow.c.withValues(alpha: .9), Kind.error => C.danger };

Color changeColor(Change c) => switch (c) { Change.modified || Change.added || Change.removed => N.g63, Change.none => N.g00.withValues(alpha: 0) };

// [open] the one floating shadow; floating things are the only ones that may have one.
List<BoxShadow> floatShadow([double a = .45]) => [BoxShadow(color: N.g00.withValues(alpha: a), blurRadius: 16, offset: const Offset(0, 6))];

const _fast = Duration(milliseconds: 120);

/// A calm surface on the lab's ground: g13 with a 1px g20 line.
Widget panel(Widget child, {double? width, EdgeInsets padding = const EdgeInsets.all(16)}) => Container(
      width: width,
      padding: padding,
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
      child: child,
    );

/// Hover + tap in one place. The builder gets the hover state.
class Hov extends StatefulWidget {
  const Hov({super.key, required this.builder, this.onTap});
  final Widget Function(BuildContext, bool) builder;
  final VoidCallback? onTap;
  @override
  State<Hov> createState() => _HovState();
}

class _HovState extends State<Hov> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.onTap == null ? MouseCursor.defer : SystemMouseCursors.click,
        onEnter: (_) => setState(() => _h = true),
        onExit: (_) => setState(() => _h = false),
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(context, _h)),
      );
}

/// One repeating controller, disposed here, for every looping animation of the set.
class Loop extends StatefulWidget {
  const Loop({super.key, required this.duration, required this.builder, this.running = true, this.start = 0});
  final Duration duration;
  final bool running;
  final double start;
  final Widget Function(BuildContext, double) builder;
  @override
  State<Loop> createState() => _LoopState();
}

class _LoopState extends State<Loop> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: widget.duration);
  @override
  void initState() {
    super.initState();
    _c.value = widget.start;
    if (widget.running) _c.repeat();
  }

  @override
  void didUpdateWidget(Loop old) {
    super.didUpdateWidget(old);
    if (old.duration != widget.duration) {
      _c.duration = widget.duration;
      if (widget.running) _c.repeat();
    }
    if (old.running != widget.running) widget.running ? _c.repeat() : _c.stop();
  }

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(animation: _c, builder: (ctx, _) => widget.builder(ctx, _c.value));
}

// ---------------------------------------------------------------- glyphs

class KindGlyph extends StatelessWidget {
  const KindGlyph(this.kind, {super.key, this.size = 14, this.disc = false});
  final Kind kind;
  final double size;
  final bool disc;
  @override
  Widget build(BuildContext context) {
    final g = CustomPaint(size: Size.square(size), painter: _GlyphP(kind));
    if (!disc) return g;
    return Container(width: size + 10, height: size + 10, alignment: Alignment.center, decoration: BoxDecoration(color: kindColor(kind).withValues(alpha: .14), shape: BoxShape.circle), child: g);
  }
}

class _GlyphP extends CustomPainter {
  _GlyphP(this.kind);
  final Kind kind;
  @override
  void paint(Canvas canvas, Size s) {
    final col = kindColor(kind), w = s.width, h = s.height;
    final p = Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final dot = Paint()..color = col;
    final ring = Rect.fromCircle(center: s.center(Offset.zero), radius: w / 2 - .7);
    switch (kind) {
      case Kind.info:
        canvas.drawOval(ring, p);
        canvas.drawCircle(Offset(w / 2, h * .33), .9, dot);
        canvas.drawLine(Offset(w / 2, h * .47), Offset(w / 2, h * .72), p);
      case Kind.success:
        canvas.drawOval(ring, p);
        canvas.drawPath(Path()..moveTo(w * .29, h * .52)..lineTo(w * .44, h * .67)..lineTo(w * .72, h * .36), p);
      case Kind.warn:
        canvas.drawPath(Path()..moveTo(w / 2, h * .1)..lineTo(w * .93, h * .88)..lineTo(w * .07, h * .88)..close(), p);
        canvas.drawLine(Offset(w / 2, h * .39), Offset(w / 2, h * .6), p);
        canvas.drawCircle(Offset(w / 2, h * .75), .8, dot);
      case Kind.error:
        canvas.drawOval(ring, p);
        canvas.drawLine(Offset(w * .35, h * .35), Offset(w * .65, h * .65), p);
        canvas.drawLine(Offset(w * .65, h * .35), Offset(w * .35, h * .65), p);
    }
  }

  @override
  bool shouldRepaint(_GlyphP old) => old.kind != kind;
}

class XGlyph extends StatelessWidget {
  const XGlyph({super.key, this.color = N.g63, this.size = 10});
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => CustomPaint(size: Size.square(size), painter: _XP(color));
}

class _XP extends CustomPainter {
  _XP(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    final p = Paint()..color = c..strokeWidth = 1.3..strokeCap = StrokeCap.round;
    canvas.drawLine(Offset.zero, Offset(s.width, s.height), p);
    canvas.drawLine(Offset(s.width, 0), Offset(0, s.height), p);
  }

  @override
  bool shouldRepaint(_XP old) => old.c != c;
}

class Chevron extends StatelessWidget {
  const Chevron({super.key, this.color = N.g63, this.size = 10});
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => CustomPaint(size: Size.square(size), painter: _ChevP(color));
}

class _ChevP extends CustomPainter {
  _ChevP(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    final p = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    canvas.drawPath(Path()..moveTo(s.width * .3, s.height * .18)..lineTo(s.width * .7, s.height * .5)..lineTo(s.width * .3, s.height * .82), p);
  }

  @override
  bool shouldRepaint(_ChevP old) => old.c != c;
}

// ---------------------------------------------------------------- button

class Btn extends StatelessWidget {
  const Btn(this.label, {super.key, this.onTap, this.primary = false, this.height = 24, this.fill, this.look = Look.concept, this.soft = false});
  final String label;
  final bool soft;
  final VoidCallback? onTap;
  final bool primary;
  final double height;
  final Color? fill;
  final Look look;
  @override
  Widget build(BuildContext context) => Hov(
        onTap: onTap ?? () {},
        builder: (c, h) {
          final base = fill ?? (soft ? Color.alphaBlend(C.mode.withValues(alpha: .4), N.g15) : (primary ? Color.alphaBlend(C.mode.withValues(alpha: .85), N.g13) : N.g20));
          final bg = Color.lerp(base, N.g100, h ? (primary ? .1 : .04) : 0)!;
          final deco = BoxDecoration(
            color: bg,
            gradient: look == Look.glow && primary ? LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [Color.lerp(bg, N.g100, .14)!, bg]) : null,
            borderRadius: BorderRadius.circular(height > 28 ? 8 : 6),
            border: Border.all(color: soft ? C.mode.withValues(alpha: .5) : (primary ? N.g100.withValues(alpha: .12) : N.g26)),
          );
          return AnimatedContainer(
            duration: _fast,
            curve: Curves.easeOut,
            height: height,
            padding: EdgeInsets.symmetric(horizontal: height > 28 ? 20 : 10),
            decoration: deco,
            child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.center, children: [Text(label, style: (height > 28 ? T.title(N.g95) : T.name(primary ? N.g95 : N.g91)))]),
          );
        },
      );
}

// ---------------------------------------------------------------- progress & spinner

class ProgressBar extends StatelessWidget {
  const ProgressBar({super.key, this.value, this.height = 4, this.look = Look.concept, this.color});

  /// null = indeterminate.
  final double? value;
  final double height;
  final Look look;
  final Color? color;

  BoxDecoration _fill(BorderRadius r) {
    final c = color ?? C.mode;
    return switch (look) {
      Look.concept => BoxDecoration(color: c, borderRadius: r),
      Look.quiet => BoxDecoration(color: color ?? N.g63, borderRadius: r),
      Look.glow => BoxDecoration(gradient: LinearGradient(colors: [c.withValues(alpha: .45), c]), borderRadius: r),
    };
  }

  @override
  Widget build(BuildContext context) {
    final r = BorderRadius.circular(height / 2 > 3 ? 3 : height / 2);
    return LayoutBuilder(builder: (context, box) {
      final w = box.maxWidth;
      final Widget inner = value == null
          ? Loop(
              duration: const Duration(milliseconds: 1500),
              builder: (c, v) {
                final seg = w * .34;
                final x = -seg + (w + seg) * Curves.easeInOutCubic.transform(v);
                return Stack(children: [Positioned(left: x, top: 0, bottom: 0, width: seg, child: DecoratedBox(decoration: _fill(r)))]);
              },
            )
          : Stack(children: [
              Positioned(left: 0, top: 0, bottom: 0, child: AnimatedContainer(duration: _fast, curve: Curves.easeOut, width: w * value!.clamp(0.0, 1.0), decoration: _fill(r))),
            ]);
      return SizedBox(height: height, width: w, child: ClipRRect(borderRadius: r, child: ColoredBox(color: N.g07, child: inner)));
    });
  }
}

class Spinner extends StatelessWidget {
  const Spinner({super.key, this.size = 20, this.look = Look.concept, this.speedMs = 900});
  final double size;
  final Look look;
  final int speedMs;
  @override
  Widget build(BuildContext context) => Loop(
        duration: Duration(milliseconds: speedMs),
        builder: (c, v) => CustomPaint(size: Size.square(size), painter: _SpinP(v, look)),
      );
}

class _SpinP extends CustomPainter {
  _SpinP(this.t, this.look);
  final double t;
  final Look look;
  @override
  void paint(Canvas canvas, Size s) {
    final sw = math.max(1.5, s.width / 10);
    final r = Rect.fromLTWH(sw / 2, sw / 2, s.width - sw, s.height - sw);
    final col = look == Look.quiet ? N.g76 : C.mode;
    canvas.drawArc(r, 0, math.pi * 2, false, Paint()..color = N.g20..style = PaintingStyle.stroke..strokeWidth = sw);
    canvas.save();
    canvas.translate(s.width / 2, s.height / 2);
    canvas.rotate(t * math.pi * 2);
    canvas.translate(-s.width / 2, -s.height / 2);
    final arc = Paint()..style = PaintingStyle.stroke..strokeWidth = sw..strokeCap = StrokeCap.round;
    if (look == Look.glow) {
      arc.shader = SweepGradient(colors: [col.withValues(alpha: 0), col], stops: const [0, .75], transform: const GradientRotation(-math.pi * .3)).createShader(r);
      canvas.drawArc(r, 0, math.pi * 1.5, false, arc);
    } else {
      arc.color = col;
      canvas.drawArc(r, 0, math.pi * (look == Look.quiet ? .5 : .7), false, arc);
    }
    canvas.restore();
  }

  @override
  bool shouldRepaint(_SpinP old) => old.t != t || old.look != look;
}

// ---------------------------------------------------------------- status bar

class StatusBar extends StatelessWidget {
  const StatusBar({super.key, required this.fps, required this.target, required this.memGb, required this.memMax, required this.gpu, required this.selected, this.look = Look.concept});
  final double fps, target, memGb, memMax, gpu;
  final int selected;
  final Look look;

  Color get _fpsColor => fps >= target * .95 ? N.g56 : (fps >= target * .6 ? kindColor(Kind.warn) : C.danger);

  Widget _seg(String label, String value, {Color? dot, double? meter, Color? meterColor}) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 10),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          if (dot != null && look == Look.concept) ...[Container(width: 6, height: 6, decoration: BoxDecoration(color: dot, shape: BoxShape.circle)), const SizedBox(width: 6)],
          Text(label, style: T.micro(N.g56)),
          const SizedBox(width: 6),
          Text(value, style: T.value(N.g91)),
          if (meter != null && look == Look.glow) ...[
            const SizedBox(width: 6),
            Container(width: 28, height: 3, decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(1.5)), alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: meter.clamp(0.0, 1.0), child: DecoratedBox(decoration: BoxDecoration(color: meterColor ?? N.g63, borderRadius: BorderRadius.circular(1.5))))),
          ],
        ]),
      );

  Widget get _div => const SizedBox(width: 1, height: 12, child: ColoredBox(color: N.g20));

  @override
  Widget build(BuildContext context) => Container(
        height: 24,
        decoration: const BoxDecoration(color: N.g13, border: Border(top: BorderSide(color: N.g20))),
        padding: const EdgeInsets.symmetric(horizontal: 2),
        child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          _seg('RENDER', '${fps.toStringAsFixed(1)} fps', dot: _fpsColor, meter: fps / target, meterColor: _fpsColor),
          _div,
          _seg('MEM', '${memGb.toStringAsFixed(1)} / ${memMax.toStringAsFixed(0)} GB', dot: memGb / memMax > .85 ? C.danger : N.g44, meter: memGb / memMax, meterColor: memGb / memMax > .85 ? C.danger : N.g63),
          _div,
          _seg('GPU', '${gpu.round()}%', dot: gpu > 92 ? kindColor(Kind.warn) : N.g44, meter: gpu / 100, meterColor: gpu > 92 ? kindColor(Kind.warn) : N.g63),
          const Spacer(),
          Padding(padding: const EdgeInsets.symmetric(horizontal: 10), child: Text(selected == 0 ? 'No selection' : '$selected selected', style: T.label(selected == 0 ? N.g44 : N.g76))),
        ]),
      );
}

// ---------------------------------------------------------------- toast

class ToastCard extends StatelessWidget {
  const ToastCard({super.key, required this.kind, required this.title, required this.body, this.life = 1, this.look = Look.concept, this.width = 320});
  final Kind kind;
  final String title, body;
  final double life, width;
  final Look look;
  @override
  Widget build(BuildContext context) {
    final col = kindColor(kind);
    return Hov(builder: (c, h) {
      return DecoratedBox(
        decoration: BoxDecoration(borderRadius: BorderRadius.circular(8), boxShadow: floatShadow()),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(8),
          child: Container(
            width: width,
            decoration: BoxDecoration(color: h ? Color.lerp(N.g15, N.g20, .35) : N.g15, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
            child: Stack(children: [
              if (look == Look.quiet) Positioned(left: 0, top: 0, bottom: 0, width: 2, child: ColoredBox(color: col)),
              Padding(
                padding: const EdgeInsets.fromLTRB(12, 12, 10, 14),
                child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  if (look != Look.quiet) Padding(padding: EdgeInsets.only(top: look == Look.glow ? 0 : 1), child: KindGlyph(kind, disc: look == Look.glow)),
                  if (look != Look.quiet) SizedBox(width: look == Look.glow ? 8 : 10),
                  Expanded(
                    child: Padding(
                      padding: EdgeInsets.only(top: look == Look.glow ? 4 : 0),
                      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                        Text(title, style: T.name(N.g95)),
                        const SizedBox(height: 6),
                        Text(body, style: T.label(N.g63).copyWith(height: 1.35)),
                      ]),
                    ),
                  ),
                  const SizedBox(width: 8),
                  AnimatedOpacity(duration: _fast, opacity: h ? 1 : .35, child: const Padding(padding: EdgeInsets.only(top: 2), child: XGlyph())),
                ]),
              ),
              Positioned(left: 0, bottom: 0, height: 2, width: (width - 2) * life.clamp(0.0, 1.0), child: ColoredBox(color: col.withValues(alpha: .55))),
            ]),
          ),
        ),
      );
    });
  }
}

class ToastStack extends StatelessWidget {
  const ToastStack({super.key, required this.count, required this.seconds, this.look = Look.concept});
  final int count;
  final double seconds;
  final Look look;
  static const _titles = ['Proxy generated', 'Export finished', 'Cache nearly full', 'Render failed'];
  static const _bodies = ['Layer "Hero_BG" now plays at full rate.', 'Title_v3.mp4 saved to Movies/Motolii.', '92% of the 8 GB disk cache is in use.', 'GPU device was lost on frame 214. Retry?'];
  @override
  Widget build(BuildContext context) => Loop(
        duration: Duration(milliseconds: (seconds * 1000).round()),
        start: .2,
        builder: (c, v) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.end, children: [
          for (var i = 0; i < count; i++)
            Builder(builder: (_) {
              final start = i * .02;
              final e = Curves.easeOut.transform(((v - start) / .05).clamp(0.0, 1.0));
              final life = 1 - ((v - start - .05) / (.86 - start - .05)).clamp(0.0, 1.0);
              final out = ((v - .9) / .08).clamp(0.0, 1.0);
              return Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Opacity(
                  opacity: e * (1 - out),
                  child: Transform.translate(offset: Offset((1 - e) * 28, 0), child: ToastCard(kind: Kind.values[i % 4], title: _titles[i % 4], body: _bodies[i % 4], life: life, look: look)),
                ),
              );
            }),
        ]),
      );
}

// ---------------------------------------------------------------- banner

class Banner2 extends StatefulWidget {
  const Banner2({super.key, required this.kind, required this.title, required this.message, this.action, this.dismissible = true, this.look = Look.concept});
  final Kind kind;
  final String title, message;
  final String? action;
  final bool dismissible;
  final Look look;
  @override
  State<Banner2> createState() => _Banner2State();
}

class _Banner2State extends State<Banner2> {
  bool _gone = false;
  @override
  Widget build(BuildContext context) {
    final col = kindColor(widget.kind);
    final glow = widget.look == Look.glow;
    return AnimatedSize(
      duration: _fast,
      curve: Curves.easeOut,
      alignment: Alignment.topCenter,
      child: _gone
          ? const SizedBox(width: double.infinity)
          : Container(
              decoration: BoxDecoration(color: glow ? Color.alphaBlend(col.withValues(alpha: .07), N.g13) : N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: glow ? col.withValues(alpha: .35) : N.g20)),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(5),
                child: Stack(children: [
                  if (widget.look == Look.concept) Positioned(left: 0, top: 0, bottom: 0, width: 2, child: ColoredBox(color: col)),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(14, 10, 8, 10),
                    child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                      KindGlyph(widget.kind),
                      const SizedBox(width: 10),
                      Expanded(child: Text.rich(TextSpan(children: [TextSpan(text: widget.title, style: T.name(N.g95)), TextSpan(text: '   ${widget.message}', style: T.label(N.g63).copyWith(fontSize: 11))]))),
                      if (widget.action != null && widget.action!.isNotEmpty) ...[const SizedBox(width: 12), Btn(widget.action!, height: 22)],
                      if (widget.dismissible)
                        Hov(
                          onTap: () => setState(() => _gone = true),
                          builder: (c, h) => AnimatedContainer(duration: _fast, margin: const EdgeInsets.only(left: 6), width: 22, height: 22, alignment: Alignment.center, decoration: BoxDecoration(color: h ? N.g20 : N.g00.withValues(alpha: 0), borderRadius: BorderRadius.circular(4)), child: XGlyph(color: h ? N.g95 : N.g56, size: 8)),
                        ),
                    ]),
                  ),
                ]),
              ),
            ),
    );
  }
}

// ---------------------------------------------------------------- empty state

class EmptyState extends StatelessWidget {
  const EmptyState({super.key, required this.title, required this.body, required this.action, this.illustration = true, this.look = Look.concept});
  final String title, body, action;
  final bool illustration;
  final Look look;
  @override
  Widget build(BuildContext context) => Column(mainAxisSize: MainAxisSize.min, children: [
        if (illustration) ...[CustomPaint(size: const Size(168, 96), painter: _EmptyP(look)), const SizedBox(height: 20)],
        Text(title, style: T.title(N.g95)),
        const SizedBox(height: 8),
        SizedBox(width: 248, child: Text(body, textAlign: TextAlign.center, style: T.label(N.g56).copyWith(fontSize: 11, height: 1.45))),
        const SizedBox(height: 16),
        Btn(action, height: 28),
      ]);
}

class _EmptyP extends CustomPainter {
  _EmptyP(this.look);
  final Look look;
  @override
  void paint(Canvas canvas, Size s) {
    final edge = Paint()..color = N.g26..style = PaintingStyle.stroke..strokeWidth = 1;
    const rows = [(8.0, 22.0, 90.0, 0), (10.0, 44.0, 120.0, 2), (6.0, 66.0, 70.0, 4)];
    for (final (x, y, w, f) in rows) {
      final r = RRect.fromRectAndRadius(Rect.fromLTWH(x, y - 8, s.width - 16, 16), const Radius.circular(3));
      canvas.drawRRect(r, Paint()..color = look == Look.quiet ? N.g00.withValues(alpha: 0) : N.bandA);
      canvas.drawRRect(r, edge);
      if (look != Look.quiet) {
        final bar = RRect.fromRectAndRadius(Rect.fromLTWH(x + 14 + f * 6, y - 5, w - f * 8, 10), const Radius.circular(2));
        canvas.drawRRect(bar, Paint()..color = Fam.all[f].c.withValues(alpha: .28));
      }
    }
    final px = s.width * .62;
    if (look == Look.glow) canvas.drawLine(Offset(px, 6), Offset(px, s.height - 4), Paint()..color = C.playhead.withValues(alpha: .5)..strokeWidth = 5..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    canvas.drawLine(Offset(px, 8), Offset(px, s.height - 4), Paint()..color = C.playhead..strokeWidth = 1);
    canvas.drawPath(Path()..moveTo(px - 4.5, 0)..lineTo(px + 4.5, 0)..lineTo(px, 8)..close(), Paint()..color = C.playhead);
  }

  @override
  bool shouldRepaint(_EmptyP old) => old.look != look;
}

// ---------------------------------------------------------------- skeleton

class Skeleton extends StatelessWidget {
  const Skeleton({super.key, required this.rows, this.animate = true, this.look = Look.concept});
  final int rows;
  final bool animate;
  final Look look;

  Widget _block(double v, {double? w, required double h, double r = 4}) {
    final pulse = look == Look.quiet;
    final hi = look == Look.glow ? N.g26 : N.g20;
    final BoxDecoration d = pulse
        ? BoxDecoration(color: Color.lerp(N.g15, N.g20, animate ? (math.sin(v * math.pi * 2) + 1) / 2 : 0), borderRadius: BorderRadius.circular(r))
        : BoxDecoration(
            borderRadius: BorderRadius.circular(r),
            gradient: LinearGradient(colors: [N.g15, animate ? hi : N.g15, N.g15], stops: const [.35, .5, .65], begin: Alignment(-3 + 6 * v, 0), end: Alignment(-1 + 6 * v, 0)),
          );
    return Container(width: w, height: h, decoration: d);
  }

  @override
  Widget build(BuildContext context) => Loop(
        duration: Duration(milliseconds: look == Look.quiet ? 1600 : 1300),
        running: animate,
        builder: (c, v) => Column(mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < rows; i++)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 6),
              child: Flex(direction: Axis.horizontal, children: [
                _block(v, w: 48, h: 28, r: 4),
                const SizedBox(width: 12),
                Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [_block(v, h: 8, w: 120.0 + (i % 3) * 36), const SizedBox(height: 8), _block(v, h: 8)])),
                const SizedBox(width: 12),
                _block(v, w: 28, h: 8),
              ]),
            ),
        ]),
      );
}

// ---------------------------------------------------------------- kbd & tooltip

class Kbd extends StatelessWidget {
  const Kbd(this.keyLabel, {super.key, this.pressed = false, this.look = Look.concept});
  final String keyLabel;
  final bool pressed;
  final Look look;
  @override
  Widget build(BuildContext context) {
    final border = switch (look) {
      Look.concept => Border.all(color: N.g26),
      Look.quiet => null,
      Look.glow => const Border(top: BorderSide(color: N.g26), left: BorderSide(color: N.g26), right: BorderSide(color: N.g26), bottom: BorderSide(color: N.g38, width: 2)),
    };
    return AnimatedContainer(
      duration: _fast,
      curve: Curves.easeOut,
      constraints: const BoxConstraints(minWidth: 18),
      height: 18,
      padding: const EdgeInsets.symmetric(horizontal: 5),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: pressed ? N.g26 : (look == Look.quiet ? N.g20 : N.g13), borderRadius: BorderRadius.circular(4), border: border),
      child: Text(keyLabel, style: T.micro(pressed ? N.g100 : N.g76).copyWith(fontFamily: T.mono, letterSpacing: 0)),
    );
  }
}

class Combo extends StatelessWidget {
  const Combo(this.keys, {super.key, this.pressed = false, this.look = Look.concept});
  final List<String> keys;
  final bool pressed;
  final Look look;
  @override
  Widget build(BuildContext context) => Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < keys.length; i++) ...[if (i > 0) const SizedBox(width: 3), Kbd(keys[i], pressed: pressed, look: look)],
      ]);
}

class TipBubble extends StatelessWidget {
  const TipBubble({super.key, required this.label, required this.keys, this.detail = '', this.look = Look.concept});
  final String label, detail;
  final List<String> keys;
  final Look look;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(6), border: look == Look.quiet ? null : Border.all(color: N.g20), boxShadow: floatShadow(look == Look.glow ? .6 : .4)),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(label, style: T.name(N.g95)),
            if (detail.isNotEmpty) ...[const SizedBox(height: 4), Text(detail, style: T.label(N.g56))],
          ]),
          if (keys.isNotEmpty) ...[
            const SizedBox(width: 12),
            if (look == Look.quiet) Text(keys.join(' '), style: T.value(N.g56)) else Combo(keys, look: look == Look.glow ? Look.glow : Look.concept),
          ],
        ]),
      );
}

// ---------------------------------------------------------------- badge & change dot

class CountPill extends StatelessWidget {
  const CountPill(this.count, {super.key, this.kind, this.max = 99});
  final int count;
  final Kind? kind;
  final int max;
  @override
  Widget build(BuildContext context) {
    final col = kind == null ? N.g91 : kindColor(kind!);
    final text = count > max ? '$max+' : '$count';
    return Container(
      constraints: const BoxConstraints(minWidth: 16),
      height: 16,
      padding: const EdgeInsets.symmetric(horizontal: 5),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: kind == null ? N.g20 : col.withValues(alpha: .16), borderRadius: BorderRadius.circular(8)),
      child: Text(text, style: T.micro(N.g91).copyWith(fontFamily: T.mono, letterSpacing: 0)),
    );
  }
}

class ChangeDot extends StatelessWidget {
  const ChangeDot(this.change, {super.key, this.size = 6});
  final Change change;
  final double size;
  @override
  Widget build(BuildContext context) => AnimatedContainer(duration: _fast, curve: Curves.easeOut, width: size, height: size, decoration: BoxDecoration(color: changeColor(change), shape: BoxShape.circle));
}

// ---------------------------------------------------------------- error row

class ErrorRow extends StatefulWidget {
  const ErrorRow({super.key, required this.message, required this.detail, this.expanded = false, this.look = Look.concept});
  final String message, detail;
  final bool expanded;
  final Look look;
  @override
  State<ErrorRow> createState() => _ErrorRowState();
}

class _ErrorRowState extends State<ErrorRow> {
  late bool _open = widget.expanded;
  @override
  Widget build(BuildContext context) {
    final glow = widget.look == Look.glow;
    return Container(
      decoration: BoxDecoration(color: glow ? Color.alphaBlend(C.danger.withValues(alpha: .06), N.g13) : N.g13, borderRadius: BorderRadius.circular(4), border: glow ? Border.all(color: C.danger.withValues(alpha: .3)) : null),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: Stack(children: [
          if (widget.look == Look.concept) const Positioned(left: 0, top: 0, bottom: 0, width: 2, child: ColoredBox(color: C.danger)),
          Column(mainAxisSize: MainAxisSize.min, children: [
            Hov(
              onTap: () => setState(() => _open = !_open),
              builder: (c, h) => AnimatedContainer(
                duration: _fast,
                height: 28,
                padding: const EdgeInsets.fromLTRB(10, 0, 10, 0),
                color: h ? N.g15 : N.g00.withValues(alpha: 0),
                child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                  const KindGlyph(Kind.error, size: 12),
                  const SizedBox(width: 8),
                  Expanded(child: Text(widget.message, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g91))),
                  const SizedBox(width: 8),
                  Text('Why?', style: T.label(h || _open ? N.g95 : N.g63)),
                  const SizedBox(width: 4),
                  AnimatedRotation(turns: _open ? .25 : 0, duration: _fast, curve: Curves.easeOut, child: Chevron(color: h || _open ? N.g95 : N.g63, size: 10)),
                ]),
              ),
            ),
            AnimatedSize(
              duration: _fast,
              curve: Curves.easeOut,
              alignment: Alignment.topCenter,
              child: _open
                  ? Padding(
                      padding: const EdgeInsets.fromLTRB(30, 2, 10, 10),
                      child: Container(
                        width: double.infinity,
                        padding: const EdgeInsets.all(8),
                        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4)),
                        child: Text(widget.detail, style: T.value(N.g63).copyWith(height: 1.5)),
                      ),
                    )
                  : const SizedBox(width: double.infinity),
            ),
          ]),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- render queue

class _ThumbP extends CustomPainter {
  _ThumbP(this.seed);
  final int seed;
  @override
  void paint(Canvas canvas, Size s) {
    canvas.drawRect(Offset.zero & s, Paint()..color = N.g13);
    final f = Fam.all[seed % Fam.all.length].c;
    canvas.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .14, s.height * .3, s.width * .5, s.height * .22), const Radius.circular(2)), Paint()..color = f.withValues(alpha: .4));
    canvas.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .3, s.height * .6, s.width * .5, s.height * .16), const Radius.circular(2)), Paint()..color = f.withValues(alpha: .22));
    canvas.drawRect((Offset.zero & s).deflate(.5), Paint()..color = N.g20..style = PaintingStyle.stroke);
  }

  @override
  bool shouldRepaint(_ThumbP old) => old.seed != seed;
}

class QueueRow extends StatelessWidget {
  const QueueRow({super.key, required this.name, required this.sub, required this.state, required this.progress, required this.eta, this.seed = 0, this.look = Look.concept});
  final String name, sub, eta;
  final QState state;
  final double progress;
  final int seed;
  final Look look;
  @override
  Widget build(BuildContext context) {
    final (String status, Color sc) = switch (state) {
      QState.queued => ('Queued', N.g56),
      QState.rendering => ('${(progress * 100).round()}%', N.g91),
      QState.done => ('Done', N.g63),
      QState.failed => ('Failed', C.danger),
    };
    final barColor = switch (state) { QState.done => N.g63, QState.failed => C.danger, _ => null };
    return Hov(
      onTap: () {},
      builder: (c, h) => AnimatedContainer(
        duration: _fast,
        curve: Curves.easeOut,
        width: 520,
        height: 52,
        padding: const EdgeInsets.fromLTRB(8, 0, 6, 0),
        decoration: BoxDecoration(color: h ? N.g15 : N.g13, borderRadius: BorderRadius.circular(4)),
        child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          ClipRRect(borderRadius: BorderRadius.circular(3), child: Opacity(opacity: state == QState.queued ? .55 : 1, child: CustomPaint(size: const Size(64, 36), painter: _ThumbP(seed)))),
          const SizedBox(width: 12),
          Expanded(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                // the name and its detail share ONE Expanded: a Spacer between loose Flexibles does not absorb the slack, so the status cell used to move with the text lengths
                Expanded(child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                  Flexible(child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g95))),
                  const SizedBox(width: 8),
                  Flexible(child: Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56))),
                ])),
                SizedBox(width: 56, child: Text(status, softWrap: false, maxLines: 1, overflow: TextOverflow.clip, textAlign: TextAlign.left, style: T.value(sc).copyWith(fontSize: 10.5))),
                SizedBox(width: 72, child: state == QState.rendering ? Text.rich(TextSpan(children: [TextSpan(text: 'ETA ', style: T.label(N.g56)), TextSpan(text: eta, style: T.value(N.g76).copyWith(fontSize: 10.5))]), softWrap: false, maxLines: 1, overflow: TextOverflow.clip) : const SizedBox()),
              ]),
              const SizedBox(height: 8),
              ProgressBar(value: switch (state) { QState.queued => 0, QState.done => 1, _ => progress }, height: 3, look: look, color: barColor),
            ]),
          ),
          const SizedBox(width: 8),
          Hov(
            onTap: () {},
            builder: (c, hh) => AnimatedContainer(duration: _fast, width: 24, height: 24, alignment: Alignment.center, decoration: BoxDecoration(color: hh ? N.g20 : N.g00.withValues(alpha: 0), borderRadius: BorderRadius.circular(4)), child: XGlyph(color: hh ? N.g95 : (h ? N.g63 : N.g44), size: 8)),
          ),
        ]),
      ),
    );
  }
}

/// Quiet segmented control: selected = g20 fill, g95 text, 1px g26 border; no accent.
class Seg extends StatelessWidget {
  const Seg({super.key, required this.items, required this.index, this.onChanged});
  final List<String> items;
  final int index;
  final ValueChanged<int>? onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6)),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < items.length; i++)
            Hov(
              onTap: () => onChanged?.call(i),
              builder: (c, h) => AnimatedContainer(
                duration: _fast,
                curve: Curves.easeOut,
                padding: const EdgeInsets.symmetric(horizontal: 12),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: i == index ? N.g20 : (h ? N.g15 : N.g00.withValues(alpha: 0)), borderRadius: BorderRadius.circular(4), border: Border.all(color: i == index ? N.g26 : N.g00.withValues(alpha: 0))),
                child: Text(items[i].toUpperCase(), style: T.micro(i == index ? N.g95 : N.g63).copyWith(letterSpacing: .6)),
              ),
            ),
        ]),
      );
}

// ---------------------------------------------------------------- export sheet

class ExportSheet extends StatefulWidget {
  const ExportSheet({super.key, required this.state, required this.progress, required this.fileName, this.look = Look.concept});
  final ExportState state;
  final double progress;
  final String fileName;
  final Look look;
  @override
  State<ExportSheet> createState() => _ExportSheetState();
}

class _ExportSheetState extends State<ExportSheet> {
  int _fmt = 0, _range = 0;
  static const _fmts = ['MP4', 'MOV', 'GIF'], _ranges = ['Work area', 'Full', 'Custom'];
  static const _est = ['48 MB', '212 MB', '9 MB'];
  static const _rangeText = ['00:00:02:00 - 00:00:10:00', '00:00:00:00 - 00:00:12:00', '00:00:04:12 - 00:00:07:00'];

  Widget _field(String label, Widget child) => Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(width: 80, height: 26, child: Align(alignment: Alignment.centerLeft, child: Text(label, style: T.label(N.g56)))), Expanded(child: Align(alignment: Alignment.centerLeft, widthFactor: null, child: child))]),
      );

  @override
  Widget build(BuildContext context) {
    final ext = _fmts[_fmt].toLowerCase();
    final name = '${widget.fileName}.$ext';
    final pct = (widget.progress * 100).round();
    final Widget action = switch (widget.state) {
      ExportState.idle => Btn('Export', primary: true, height: 36, look: widget.look),
      ExportState.exporting => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Text('Exporting', style: T.name(N.g95)),
            const Spacer(),
            Text('$pct%', style: T.value(N.g95)),
          ]),
          const SizedBox(height: 8),
          ProgressBar(value: widget.progress, height: 6, look: widget.look),
          const SizedBox(height: 8),
          Flex(direction: Axis.horizontal, children: [Text('Frame ${(widget.progress * 300).round()} / 300   ETA 00:${(12 * (1 - widget.progress)).round().toString().padLeft(2, '0')}', style: T.value(N.g56).copyWith(fontSize: 10)), const Spacer(), const Btn('Cancel')]),
        ]),
      ExportState.done => Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          const KindGlyph(Kind.success),
          const SizedBox(width: 8),
          Expanded(child: Text('Saved $name', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g91))),
          const SizedBox(width: 8),
          const Btn('Reveal'),
        ]),
    };
    return Container(
      width: 420,
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20), boxShadow: floatShadow()),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Text('Export', style: T.title()),
        const SizedBox(height: 4),
        Text('Title_v3  -  1920 x 1080  -  30 fps', style: T.label(N.g56)),
        const SizedBox(height: 20),
        _field('Format', Seg(items: _fmts, index: _fmt, onChanged: (i) => setState(() => _fmt = i))),
        _field('Range', Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Seg(items: _ranges, index: _range, onChanged: (i) => setState(() => _range = i)), const SizedBox(height: 8), Text(_rangeText[_range], style: T.value(N.g63).copyWith(fontSize: 10.5))])),
        _field(
          'Destination',
          Flex(direction: Axis.horizontal, children: [
            Expanded(child: Container(height: 26, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.centerLeft, decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4)), child: Text('~/Movies/Motolii/$name', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.value(N.g76)))),
            const SizedBox(width: 8),
            const Btn('Change'),
          ]),
        ),
        Padding(padding: const EdgeInsets.only(left: 80, bottom: 16), child: Text('About ${_est[_fmt]}', style: T.label(N.g44))),
        const SizedBox(height: 4),
        AnimatedSwitcher(duration: _fast, child: KeyedSubtree(key: ValueKey(widget.state), child: action)),
      ]),
    );
  }
}

// ---------------------------------------------------------------- undo history

class HistoryList extends StatefulWidget {
  const HistoryList({super.key, required this.count, required this.current, this.look = Look.concept});
  final int count, current;
  final Look look;
  @override
  State<HistoryList> createState() => _HistoryListState();
}

class _HistoryListState extends State<HistoryList> {
  late int _cur = widget.current;
  static const _items = <(String, Change, String)>[
    ('Open composition', Change.none, '12:01'),
    ('Add layer "Hero_BG"', Change.added, '12:02'),
    ('Set Position', Change.modified, '12:02'),
    ('Move keyframe', Change.modified, '12:04'),
    ('Delete layer "Spark"', Change.removed, '12:05'),
    ('Add Stagger relation', Change.added, '12:07'),
    ('Set Opacity', Change.modified, '12:08'),
    ('Rename "Hero_BG"', Change.modified, '12:09'),
    ('Trim layer end', Change.modified, '12:10'),
    ('Add layer "Title"', Change.added, '12:11'),
  ];
  @override
  Widget build(BuildContext context) {
    final n = widget.count.clamp(1, _items.length);
    final cur = _cur.clamp(0, n - 1);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: const EdgeInsets.fromLTRB(8, 0, 8, 8), child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [Text('HISTORY', style: T.micro(N.g56).copyWith(letterSpacing: .8)), const Spacer(), CountPill(n - 1 - cur, kind: n - 1 - cur > 0 ? Kind.info : null)])),
      for (var i = 0; i < n; i++)
        Padding(
          padding: const EdgeInsets.only(bottom: 2),
          child: Hov(
            onTap: () => setState(() => _cur = i),
            builder: (c, h) {
              final isCur = i == cur, undone = i > cur;
              final glow = widget.look == Look.glow, quiet = widget.look == Look.quiet;
              final bg = isCur && !quiet ? (glow ? N.g15 : N.g20) : (h ? N.g15 : N.g00.withValues(alpha: 0));
              return AnimatedContainer(
                duration: _fast,
                curve: Curves.easeOut,
                height: 26,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: bg, borderRadius: BorderRadius.circular(4), border: glow && isCur ? const Border(left: BorderSide(color: C.mode, width: 2)) : null),
                child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                  Opacity(opacity: undone ? .35 : 1, child: ChangeDot(_items[i].$2 == Change.none ? Change.modified : _items[i].$2)),
                  const SizedBox(width: 10),
                  Expanded(child: Text(_items[i].$1, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(undone ? N.g44 : (isCur ? N.g100 : N.g91)))),
                  if (isCur && quiet) ...[Container(width: 5, height: 5, decoration: const BoxDecoration(color: C.mode, shape: BoxShape.circle)), const SizedBox(width: 8)],
                  Text(_items[i].$3, style: T.value(N.g44).copyWith(fontSize: 10)),
                ]),
              );
            },
          ),
        ),
    ]);
  }
}
