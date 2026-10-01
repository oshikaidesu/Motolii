// Private helpers of media_set: art, badges, interaction shells. Colours only from tokens.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';

import '../tokens.dart';

const kFast = Duration(milliseconds: 120);
const kEase = Curves.easeOut;

String clock(double s) => '${s ~/ 60}:${(s % 60).floor().toString().padLeft(2, '0')}';

String hexOf(Color c) {
  String h(double v) => (v * 255).round().toRadixString(16).padLeft(2, '0').toUpperCase();
  return '#${h(c.r)}${h(c.g)}${h(c.b)}';
}

enum Kind {
  image('IMG', 'Image'),
  video('VID', 'Video'),
  audio('AUD', 'Audio'),
  model('3D', '3D'),
  pano('360°', '360°');

  const Kind(this.badge, this.label);
  final String badge, label;
  bool get timed => this == video || this == audio || this == pano;
}

enum ArtStyle { sky, gradient, abstract, wave, solid }

class Asset {
  const Asset(this.name, this.kind, this.seed, {this.style = ArtStyle.sky, this.dur = 0, this.tags = const [], this.aspect = .625, this.dims = '1920×1080', this.size = '4.2 MB'});
  final String name;
  final Kind kind;
  final int seed;
  final ArtStyle style;
  final double dur, aspect;
  final List<String> tags;
  final String dims, size;
  ArtStyle get art => kind == Kind.audio ? ArtStyle.wave : (kind == Kind.model ? ArtStyle.solid : style);
}

const sampleAssets = <Asset>[
  Asset('dusk_ridge.jpg', Kind.image, 3, tags: ['sky', 'bg'], aspect: .8, dims: '4096×3200', size: '8.1 MB'),
  Asset('lens_bloom.mp4', Kind.video, 7, style: ArtStyle.abstract, dur: 12, tags: ['fx', 'loop'], aspect: .56),
  Asset('wind_pad.wav', Kind.audio, 11, dur: 94, tags: ['pad'], aspect: .5, dims: '48 kHz', size: '16 MB'),
  Asset('gem_cut.glb', Kind.model, 2, tags: ['3d', 'prop'], aspect: .9, dims: '38k tris', size: '2.4 MB'),
  Asset('studio_hdri.exr', Kind.pano, 5, dur: 0, tags: ['hdri'], aspect: .5, dims: '8192×4096', size: '42 MB'),
  Asset('violet_fade.png', Kind.image, 9, style: ArtStyle.gradient, tags: ['gradient'], aspect: .7),
  Asset('city_pass.mov', Kind.video, 14, dur: 47, tags: ['b-roll', 'city'], aspect: .62),
  Asset('haze_01.png', Kind.image, 21, style: ArtStyle.abstract, tags: ['texture'], aspect: .9),
  Asset('kick_loop.wav', Kind.audio, 4, dur: 8, tags: ['drums', 'loop'], aspect: .55, dims: '48 kHz', size: '1.4 MB'),
  Asset('sunrise.jpg', Kind.image, 17, tags: ['sky'], aspect: .66, dims: '5472×3648', size: '11 MB'),
  Asset('orb.glb', Kind.model, 8, tags: ['3d'], aspect: .75, dims: '12k tris', size: '860 KB'),
  Asset('sphere_tour.mp4', Kind.pano, 12, style: ArtStyle.gradient, dur: 63, tags: ['360', 'tour'], aspect: .5, dims: '5760×2880', size: '210 MB'),
];

/// Thumbnail art drawn in the palette only: Fam colours at low alpha over the greys.
class ArtPainter extends CustomPainter {
  const ArtPainter(this.seed, this.style);
  final int seed;
  final ArtStyle style;

  @override
  void paint(Canvas canvas, Size size) {
    final w = size.width, h = size.height, r = Offset.zero & size;
    final rnd = math.Random(seed);
    final a = Fam.all[seed % 6].c, b = Fam.all[(seed * 5 + 2) % 6].c;
    canvas.drawRect(r, Paint()..shader = const LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [N.g10, N.g13]).createShader(r));
    switch (style) {
      case ArtStyle.sky:
        canvas.drawRect(r, Paint()..shader = LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [a.withValues(alpha: .3), b.withValues(alpha: .35), N.g13.withValues(alpha: 0)], stops: const [0, .62, 1]).createShader(r));
        canvas.drawCircle(Offset(w * (.25 + .5 * rnd.nextDouble()), h * .5), h * .13, Paint()..color = N.g95.withValues(alpha: .85)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.5));
        for (var layer = 0; layer < 2; layer++) {
          final p = Path()..moveTo(0, h);
          const n = 9;
          for (var i = 0; i <= n; i++) {
            p.lineTo(w * i / n, h * (.72 - .16 * rnd.nextDouble() - layer * .1 + (layer == 0 ? .08 : 0)));
          }
          p..lineTo(w, h)..close();
          canvas.drawPath(p, Paint()..color = (layer == 0 ? N.g10 : N.g07).withValues(alpha: .92));
        }
      case ArtStyle.gradient:
        canvas.drawRect(r, Paint()..shader = LinearGradient(begin: Alignment.topLeft, end: Alignment.bottomRight, colors: [a.withValues(alpha: .35), b.withValues(alpha: .25)]).createShader(r));
        canvas.drawRect(r, Paint()..shader = LinearGradient(begin: Alignment.bottomCenter, end: Alignment.topCenter, colors: [N.g00.withValues(alpha: .35), N.g00.withValues(alpha: 0)]).createShader(r));
      case ArtStyle.abstract:
        for (var i = 0; i < 4; i++) {
          final c = Fam.all[(seed + i * 2) % 6].c;
          canvas.drawCircle(Offset(w * rnd.nextDouble(), h * rnd.nextDouble()), math.min(w, h) * (.22 + .22 * rnd.nextDouble()), Paint()..color = c.withValues(alpha: .3)..maskFilter = MaskFilter.blur(BlurStyle.normal, math.min(w, h) * .18));
        }
      case ArtStyle.wave:
        canvas.drawRect(r, Paint()..shader = LinearGradient(begin: Alignment.centerLeft, end: Alignment.centerRight, colors: [a.withValues(alpha: .25), b.withValues(alpha: .25)]).createShader(r));
        final bars = math.max(8, (w / 3).floor());
        final p = Paint()..color = N.g91.withValues(alpha: .8)..strokeWidth = 1.2;
        for (var i = 0; i < bars; i++) {
          final x = w * (i + .5) / bars;
          final env = math.sin(math.pi * i / bars);
          final amp = h * .42 * (.2 + .8 * rnd.nextDouble()) * (.35 + .65 * env);
          canvas.drawLine(Offset(x, h / 2 - amp), Offset(x, h / 2 + amp), p);
        }
      case ArtStyle.solid:
        final c = Offset(w / 2, h / 2), rad = math.min(w, h) * .3;
        canvas.drawCircle(c, rad * 1.5, Paint()..color = a.withValues(alpha: .2)..maskFilter = MaskFilter.blur(BlurStyle.normal, rad * .6));
        canvas.drawCircle(c, rad, Paint()..shader = RadialGradient(center: const Alignment(-.4, -.45), radius: 1, colors: [N.g95.withValues(alpha: .85), b.withValues(alpha: .35), N.g07]).createShader(Rect.fromCircle(center: c, radius: rad)));
        canvas.drawOval(Rect.fromCenter(center: c, width: rad * 3, height: rad * .7), Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g76.withValues(alpha: .5));
    }
    if (style == ArtStyle.wave) return;
    canvas.drawRect(r, Paint()..shader = ui.Gradient.radial(Offset(w / 2, h / 2), math.max(w, h) * .8, [N.g00.withValues(alpha: 0), N.g00.withValues(alpha: .25)]));
  }

  @override
  bool shouldRepaint(ArtPainter o) => o.seed != seed || o.style != style;
}

class Art extends StatelessWidget {
  const Art(this.asset, {super.key});
  final Asset asset;
  @override
  Widget build(BuildContext context) => RepaintBoundary(child: CustomPaint(painter: ArtPainter(asset.seed, asset.art), size: Size.infinite));
}

class TypeBadge extends StatelessWidget {
  const TypeBadge(this.kind, {super.key});
  final Kind kind;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
        decoration: BoxDecoration(color: N.g00.withValues(alpha: .6), borderRadius: BorderRadius.circular(3)),
        child: Text(kind.badge, style: T.micro(N.g91).copyWith(letterSpacing: .4)),
      );
}

class TimePill extends StatelessWidget {
  const TimePill(this.text, {super.key});
  final String text;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
        decoration: BoxDecoration(color: N.g00.withValues(alpha: .6), borderRadius: BorderRadius.circular(3)),
        child: Text(text, style: T.value(N.g91).copyWith(fontSize: 10)),
      );
}

/// Hover + focus + tap + local pointer position, in one shell.
class Press extends StatefulWidget {
  const Press({super.key, required this.builder, this.onTap, this.cursor = SystemMouseCursors.click});
  final Widget Function(BuildContext context, bool hover, bool focus, Offset? pos) builder;
  final VoidCallback? onTap;
  final MouseCursor cursor;
  @override
  State<Press> createState() => _PressState();
}

class _PressState extends State<Press> {
  bool _h = false, _f = false;
  Offset? _p;
  @override
  Widget build(BuildContext context) => FocusableActionDetector(
        onShowHoverHighlight: (v) => setState(() => _h = v),
        onShowFocusHighlight: (v) => setState(() => _f = v),
        mouseCursor: widget.cursor,
        actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) {
          widget.onTap?.call();
          return null;
        })},
        child: MouseRegion(
          onHover: (e) => setState(() => _p = e.localPosition),
          onExit: (_) => setState(() => _p = null),
          child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(context, _h, _f, _p)),
        ),
      );
}

/// A little state holder so use cases can be interactive without a class each.
class Hold<T> extends StatefulWidget {
  const Hold({super.key, required this.initial, required this.builder});
  final T initial;
  final Widget Function(BuildContext context, T value, ValueChanged<T> set) builder;
  @override
  State<Hold<T>> createState() => _HoldState<T>();
}

class _HoldState<T> extends State<Hold<T>> {
  late T _v = widget.initial;
  @override
  Widget build(BuildContext context) => widget.builder(context, _v, (v) => setState(() => _v = v));
}

class Draw extends CustomPainter {
  const Draw(this.fn, [this.key]);
  final void Function(Canvas canvas, Size size) fn;
  final Object? key;
  @override
  void paint(Canvas canvas, Size size) => fn(canvas, size);
  @override
  bool shouldRepaint(Draw o) => o.key != key;
}

Widget glyph(double size, void Function(Canvas, Size) fn, [Object? key]) => SizedBox(width: size, height: size, child: CustomPaint(painter: Draw(fn, key)));

void drawFolder(Canvas c, Size s, Color col, {bool fill = false}) {
  final w = s.width, h = s.height;
  final p = Path()
    ..moveTo(w * .08, h * .22)
    ..lineTo(w * .38, h * .22)
    ..lineTo(w * .48, h * .34)
    ..lineTo(w * .92, h * .34)
    ..lineTo(w * .92, h * .8)
    ..lineTo(w * .08, h * .8)
    ..close();
  c.drawPath(p, Paint()..color = col..style = fill ? PaintingStyle.fill : PaintingStyle.stroke..strokeWidth = 1.2..strokeJoin = StrokeJoin.round);
}

void drawStar(Canvas c, Size s, Color col, bool fill) {
  final ctr = Offset(s.width / 2, s.height / 2 + .5), ro = s.width * .46, ri = ro * .45;
  final p = Path();
  for (var i = 0; i < 10; i++) {
    final rr = i.isEven ? ro : ri, an = -math.pi / 2 + i * math.pi / 5;
    final pt = ctr + Offset(math.cos(an) * rr, math.sin(an) * rr);
    i == 0 ? p.moveTo(pt.dx, pt.dy) : p.lineTo(pt.dx, pt.dy);
  }
  p.close();
  c.drawPath(p, Paint()..color = col..style = fill ? PaintingStyle.fill : PaintingStyle.stroke..strokeWidth = 1.2..strokeJoin = StrokeJoin.round);
}

class StarButton extends StatelessWidget {
  const StarButton({super.key, required this.on, this.onChanged, this.size = 16});
  final bool on;
  final double size;
  final ValueChanged<bool>? onChanged;
  @override
  Widget build(BuildContext context) => Press(
        onTap: () => onChanged?.call(!on),
        builder: (context, hover, focus, _) => SizedBox(width: 24, height: 24, child: Center(child: glyph(size, (c, s) => drawStar(c, s, on ? N.g95 : (hover || focus ? N.g76 : N.g44), on), [on, hover, focus]))),
      );
}

class DashedBox extends CustomPainter {
  const DashedBox(this.color, {this.radius = 6, this.dash = 5, this.gap = 4, this.width = 1});
  final Color color;
  final double radius, dash, gap, width;
  @override
  void paint(Canvas canvas, Size size) {
    final path = Path()..addRRect(RRect.fromRectAndRadius((Offset.zero & size).deflate(width / 2), Radius.circular(radius)));
    final p = Paint()..color = color..style = PaintingStyle.stroke..strokeWidth = width;
    for (final m in path.computeMetrics()) {
      for (var d = 0.0; d < m.length; d += dash + gap) {
        canvas.drawPath(m.extractPath(d, math.min(d + dash, m.length)), p);
      }
    }
  }

  @override
  bool shouldRepaint(DashedBox o) => o.color != color || o.radius != radius || o.dash != dash || o.gap != gap || o.width != width;
}

/// A thin horizontal slider: the track is grey, the value is the brightest thing.
class MiniSlider extends StatelessWidget {
  const MiniSlider({super.key, required this.value, required this.onChanged, this.min = 0, this.max = 1, this.width = 88, this.track = N.g26});
  final double value, min, max, width;
  final Color track;
  final ValueChanged<double> onChanged;
  @override
  Widget build(BuildContext context) {
    final t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    void set(Offset p) => onChanged(min + (max - min) * (p.dx / width).clamp(0.0, 1.0));
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanDown: (d) => set(d.localPosition),
        onPanUpdate: (d) => set(d.localPosition),
        child: SizedBox(width: width, height: 20, child: Stack(alignment: Alignment.centerLeft, children: [
          Container(height: 2, decoration: BoxDecoration(color: track, borderRadius: BorderRadius.circular(1))),
          Container(width: width * t, height: 2, decoration: BoxDecoration(color: N.g56, borderRadius: BorderRadius.circular(1))),
          Positioned(left: (width - 10) * t, child: Container(width: 10, height: 10, decoration: const BoxDecoration(color: N.g95, shape: BoxShape.circle))),
        ])),
      ),
    );
  }
}

class Pill extends StatelessWidget {
  const Pill(this.text, {super.key, this.fill = N.g20});
  final String text;
  final Color fill;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
        decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(6)),
        child: Text(text, style: T.micro(N.g76)),
      );
}

/// A quiet text button: g13 at rest, g15 on hover, g20 pressed-ish focus.
class QuietButton extends StatelessWidget {
  const QuietButton(this.text, {super.key, this.onTap, this.primary = false});
  final String text;
  final bool primary;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Press(
        onTap: onTap,
        builder: (context, hover, focus, _) => AnimatedContainer(
          duration: kFast,
          curve: kEase,
          height: 24,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: BoxDecoration(color: primary ? (hover ? N.g26 : N.g20) : (hover ? N.g20 : N.g13), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : (primary ? N.g26 : N.g20))),
          child: Center(widthFactor: 1, child: Text(text, style: T.label(N.g91).copyWith(fontWeight: FontWeight.w500))),
        ),
      );
}

/// DESIGN.md: a selected row carries a 2px g95 tick on its left.
class Tick extends StatelessWidget {
  const Tick(this.on, {super.key, this.height = 16});
  final bool on;
  final double height;
  @override
  Widget build(BuildContext context) => AnimatedContainer(duration: kFast, curve: kEase, width: 2, height: height, margin: const EdgeInsets.only(right: 4), decoration: BoxDecoration(color: on ? N.g95 : const Color(0x00000000), borderRadius: BorderRadius.circular(1)));
}
