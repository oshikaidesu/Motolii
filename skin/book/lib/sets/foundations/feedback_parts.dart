// Private helpers of the feedback set. Colours from N / Fam / C, text from T. A value no token names is `// [open]`.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';
part 'feedback_parts_2.dart';

enum Kind { info, success, warn, error }

enum Change { none, modified, added, removed }

enum QState { queued, rendering, done, failed }

enum ExportState { idle, exporting, done }

// [open] warn is a tint of Follow; the brief gives no token for it.
Color kindColor(Kind k) => switch (k) { Kind.info => Role.of(C.playhead, Role.info), Kind.success => Role.ok, Kind.warn => Role.of(Fam.follow.c.withValues(alpha: .9), Role.warning), Kind.error => Role.error };

Color changeColor(Change c) => switch (c) { Change.modified => Role.of(N.g63, Role.changed), Change.added || Change.removed => N.g63, Change.none => N.g00.withValues(alpha: 0) };

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
            border: Border.all(color: soft ? Role.selected.withValues(alpha: .5) : (primary ? N.g100.withValues(alpha: .12) : N.g26)),
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

  Color get _fpsColor => fps >= target * .95 ? N.g56 : (fps >= target * .6 ? kindColor(Kind.warn) : Role.error);

  Widget _seg(String label, String value, {Color? dot, double? meter, Color? meterColor}) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 10),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          if (dot != null && look == Look.concept) ...[dot == Role.error && !Role.grey ? const ErrMark(size: 9) : Container(width: 6, height: 6, decoration: BoxDecoration(color: dot, shape: BoxShape.circle)), const SizedBox(width: 6)],
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
          _seg('MEM', '${memGb.toStringAsFixed(1)} / ${memMax.toStringAsFixed(0)} GB', dot: memGb / memMax > .85 ? Role.error : N.g44, meter: memGb / memMax, meterColor: memGb / memMax > .85 ? Role.error : N.g63),
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
