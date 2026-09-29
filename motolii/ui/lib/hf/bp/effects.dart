import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'classify.dart';
import 'common.dart';
import 'faces.dart';
import 'search.dart';
import 'seat.dart';
import 'shell.dart';
import 'things.dart';
import '../neutral.dart';

const _sw = 240, _sh = 150;

/// The picture every effect is applied to. Built once; each tile shows the effect's own result.

/// The effect faces' sample picture (a dusk over hills) that every effect is shown applied to: its own palette.
abstract final class _Scene {
  static const sky = [Color(0xFF2C4DB0), Color(0xFF9B6FD0), Color(0xFFFF9C7A), Color(0xFFFFE2A8)];
  static const sun = Color(0xFFFFF6D0), spark = Color(0xFFFFE7A8), night = Color(0xFF0B0A18);
  static const ridges = [Color(0xFF7C6BC4), Color(0xFF4A3F96), Color(0xFF1E1A54)];
}
class EffectScene {
  EffectScene._(this.image, this.small, this.bytes);
  final ui.Image image, small;
  final Uint8List bytes;
  static Future<EffectScene> build() async {
    final img = _draw(_sw, _sh, _paintScene);
    final rec = ui.PictureRecorder();
    Canvas(rec).drawImageRect(img, Rect.fromLTWH(0, 0, _sw.toDouble(), _sh.toDouble()), const Rect.fromLTWH(0, 0, 24, 15), Paint()..filterQuality = FilterQuality.medium);
    final small = rec.endRecording().toImageSync(24, 15);
    final data = await img.toByteData(format: ui.ImageByteFormat.rawRgba);
    return EffectScene._(img, small, data!.buffer.asUint8List());
  }

  static ui.Image _draw(int w, int h, void Function(Canvas, Size) f) {
    final rec = ui.PictureRecorder();
    f(Canvas(rec), Size(w.toDouble(), h.toDouble()));
    return rec.endRecording().toImageSync(w, h);
  }

  static void _paintScene(Canvas c, Size s) {
    final r = Offset.zero & s;
    c.drawRect(r, Paint()..shader = const LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: _Scene.sky, stops: [0, .42, .76, 1]).createShader(r));
    c.drawCircle(Offset(s.width * .68, s.height * .52), 22, Paint()..color = _Scene.sun);
    final rnd = math.Random(3);
    for (var i = 0; i < 40; i++) {
      c.drawCircle(Offset(rnd.nextDouble() * s.width, rnd.nextDouble() * s.height * .4), .5 + rnd.nextDouble() * .8, Paint()..color = N.glaze80);
    }
    Path ridge(double base, double amp, int seed, double freq) {
      final p = Path()..moveTo(0, s.height);
      final g = math.Random(seed);
      final ph = g.nextDouble() * 6;
      for (var x = 0.0; x <= s.width; x += 6) {
        p.lineTo(x, base - amp * (math.sin(x * freq + ph) * .5 + .5) - amp * .5 * (math.sin(x * freq * 2.7 + ph * 2) * .5 + .5));
      }
      return p..lineTo(s.width, s.height)..close();
    }
    c.drawPath(ridge(s.height * .68, 42, 1, .028), Paint()..color = _Scene.ridges[0]);
    c.drawPath(ridge(s.height * .78, 36, 2, .036), Paint()..color = _Scene.ridges[1]);
    c.drawPath(ridge(s.height * .92, 26, 3, .05), Paint()..color = _Scene.ridges[2]);
    c.drawCircle(Offset(s.width * .3, s.height * .64), 3, Paint()..color = _Scene.spark);
  }
}

ColorFilter hueFilter(double deg) {
  final a = deg * math.pi / 180, co = math.cos(a), si = math.sin(a);
  return ColorFilter.matrix([
    .213 + co * .787 - si * .213, .715 - co * .715 - si * .715, .072 - co * .072 + si * .928, 0, 0,
    .213 - co * .213 + si * .143, .715 + co * .285 + si * .140, .072 - co * .072 - si * .283, 0, 0,
    .213 - co * .213 - si * .787, .715 - co * .715 + si * .715, .072 + co * .928 + si * .072, 0, 0,
    0, 0, 0, 1, 0,
  ]);
}

class FxPainter extends CustomPainter {
  FxPainter(this.base, this.scene);
  final String base;
  final EffectScene scene;
  @override
  void paint(Canvas cv, Size s) {
    final src = Rect.fromLTWH(0, 0, _sw.toDouble(), _sh.toDouble());
    final dst = Offset.zero & s;
    final k = s.width / _sw;
    final plain = Paint()..filterQuality = FilterQuality.medium;
    cv.save();
    cv.clipRect(dst);
    void lum(Paint p, List<double> lo, List<double> hi) {
      const lr = .3, lg = .59, lb = .11;
      p.colorFilter = ColorFilter.matrix([
        lr * (hi[0] - lo[0]), lg * (hi[0] - lo[0]), lb * (hi[0] - lo[0]), 0, lo[0] * 255,
        lr * (hi[1] - lo[1]), lg * (hi[1] - lo[1]), lb * (hi[1] - lo[1]), 0, lo[1] * 255,
        lr * (hi[2] - lo[2]), lg * (hi[2] - lo[2]), lb * (hi[2] - lo[2]), 0, lo[2] * 255,
        0, 0, 0, 1, 0,
      ]);
    }
    switch (base) {
      case 'Blur':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.medium..imageFilter = ui.ImageFilter.blur(sigmaX: 9 * k * 2, sigmaY: 9 * k * 2, tileMode: TileMode.clamp));
      case 'Bokeh':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.medium..imageFilter = ui.ImageFilter.blur(sigmaX: 5 * k * 2, sigmaY: 5 * k * 2, tileMode: TileMode.clamp));
        final rnd = math.Random(6);
        for (var i = 0; i < 26; i++) {
          final c = HSVColor.fromAHSV(.55, 20 + rnd.nextDouble() * 50, .5, 1).toColor();
          cv.drawCircle(Offset(rnd.nextDouble() * s.width, rnd.nextDouble() * s.height), (5 + rnd.nextDouble() * 9) * k * 2, Paint()..color = c..blendMode = BlendMode.screen);
        }
      case 'Glow':
        cv.drawImageRect(scene.image, src, dst, plain);
        cv.saveLayer(dst, Paint()..blendMode = BlendMode.screen);
        cv.drawImageRect(scene.image, src, dst, Paint()..imageFilter = ui.ImageFilter.blur(sigmaX: 8 * k * 2, sigmaY: 8 * k * 2, tileMode: TileMode.clamp)..colorFilter = const ColorFilter.matrix([1.5, 0, 0, 0, 0, 0, 1.4, 0, 0, 0, 0, 0, 1.3, 0, 0, 0, 0, 0, 1, 0]));
        cv.restore();
      case 'Vignette':
        cv.drawImageRect(scene.image, src, dst, plain);
        cv.drawRect(dst, Paint()..shader = const RadialGradient(radius: .85, colors: [N.clear, N.shade90], stops: [.45, 1]).createShader(dst));
      case 'Color Shift':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.medium..colorFilter = hueFilter(140));
      case 'Duotone':
        final p = Paint()..filterQuality = FilterQuality.medium;
        lum(p, [.08, .02, .28], [1.0, .78, .42]);
        cv.drawImageRect(scene.image, src, dst, p);
      case 'Invert':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.medium..colorFilter = const ColorFilter.matrix([-1, 0, 0, 0, 255, 0, -1, 0, 0, 255, 0, 0, -1, 0, 255, 0, 0, 0, 1, 0]));
      case 'Chromatic':
        cv.drawRect(dst, Paint()..color = N.g00);
        final off = 5 * k * 2;
        final chans = [
          (const [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0], -off),
          (const [0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0], 0.0),
          (const [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0], off),
        ];
        for (final (m, dx) in chans) {
          cv.drawImageRect(scene.image, src, dst.shift(Offset(dx, 0)), Paint()..blendMode = BlendMode.screen..filterQuality = FilterQuality.medium..colorFilter = ColorFilter.matrix([for (final v in m) v.toDouble()]));
        }
      case 'Displace':
        final rnd = math.Random(8);
        const strip = 3.0;
        for (var y = 0.0; y < s.height; y += strip) {
          final dx = math.sin(y * .12) * 7 * k * 2 + (rnd.nextDouble() - .5) * 12 * k * 2 * (math.sin(y * .05) > .3 ? 1 : .15);
          final sy = y / s.height * _sh;
          cv.drawImageRect(scene.image, Rect.fromLTWH(0, sy, _sw.toDouble(), strip / s.height * _sh + .5), Rect.fromLTWH(dx, y, s.width, strip + .5), plain);
        }
      case 'Pixelate':
        cv.drawImageRect(scene.small, Rect.fromLTWH(0, 0, 24, 15), dst, Paint()..filterQuality = FilterQuality.none);
      case 'Halftone':
        cv.drawRect(dst, Paint()..color = _Scene.night);
        final step = 6.5 * k * 2;
        for (var y = step / 2; y < s.height; y += step) {
          for (var x = step / 2; x < s.width; x += step) {
            final px = (x / s.width * (_sw - 1)).floor(), py = (y / s.height * (_sh - 1)).floor();
            final i = (py * _sw + px) * 4;
            final r = scene.bytes[i], g = scene.bytes[i + 1], b = scene.bytes[i + 2];
            final l = (r * .3 + g * .59 + b * .11) / 255;
            cv.drawCircle(Offset(x, y), step * .55 * math.sqrt(l.clamp(.03, 1.0)), Paint()..color = Color.fromARGB(255, math.min(255, r + 40), math.min(255, g + 40), math.min(255, b + 40)));
          }
        }
      case 'Threshold':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.medium..colorFilter = const ColorFilter.matrix([2.4, 4.7, .9, 0, -560, 2.4, 4.7, .9, 0, -560, 2.4, 4.7, .9, 0, -560, 0, 0, 0, 1, 0]));
      case 'Sharpen':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.high..colorFilter = const ColorFilter.matrix([1.5, 0, 0, 0, -38, 0, 1.5, 0, 0, -38, 0, 0, 1.5, 0, -38, 0, 0, 0, 1, 0]));
      case 'Noise':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.medium..colorFilter = const ColorFilter.matrix([.5, .4, .1, 0, 10, .4, .5, .1, 0, 10, .4, .4, .2, 0, 10, 0, 0, 0, 1, 0]));
        final rnd = math.Random(5);
        final p = Paint();
        final n = (s.width * s.height / 3.2).round();
        for (var i = 0; i < n; i++) {
          final v = rnd.nextInt(255);
          p.color = Color.fromARGB(190, v, v, v);
          cv.drawRect(Rect.fromLTWH(rnd.nextDouble() * s.width, rnd.nextDouble() * s.height, 1.5, 1.5), p);
        }
      case 'Film Grain':
        cv.drawImageRect(scene.image, src, dst, Paint()..filterQuality = FilterQuality.medium..colorFilter = const ColorFilter.matrix([1.05, .1, 0, 0, 8, .05, 1.0, .05, 0, 4, 0, .05, .85, 0, 0, 0, 0, 0, 1, 0]));
        final rnd = math.Random(9);
        final p = Paint();
        final n = (s.width * s.height / 6).round();
        for (var i = 0; i < n; i++) {
          final v = 120 + rnd.nextInt(135);
          p.color = Color.fromARGB(70, v, v, v);
          cv.drawRect(Rect.fromLTWH(rnd.nextDouble() * s.width, rnd.nextDouble() * s.height, 1.2, 1.2), p);
        }
    }
    cv.restore();
  }
  @override
  bool shouldRepaint(FxPainter o) => o.base != base || o.scene != scene;
}

class EffectsPanel extends StatefulWidget {
  const EffectsPanel(this.scene, {super.key, required this.catalog, this.user, this.search, this.classify});
  final EffectScene? scene;
  final Catalog catalog;
  final UserViews? user;
  final SearchCapability? search;
  final ClassifyCapability? classify;
  @override
  State<EffectsPanel> createState() => _EffectsPanelState();
}

class _EffectsPanelState extends State<EffectsPanel> with WithDiscovery<EffectsPanel> {
  @override
  SearchCapability? get injectedSearch => widget.search;
  @override
  ClassifyCapability? get injectedClassify => widget.classify;
  late final views = ThingViews(widget.catalog.registry, widget.catalog.things, widget.catalog.registry.panels['effects']!, widget.user ?? UserViews());

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: discovery,
        builder: (_, __) {
          final q = ThingQuery.parse(search.query);
          final found = q.isEmpty ? views.scope : [for (final t in views.scope) if (q.matches(t, widget.catalog.registry)) t];
          final shown = [for (final t in found) if (views.contains(classify.selected, t)) t];
          final n = shown.length;
          return PanelShell(
            // classes along the top, as Create and Media: the body keeps the seat's whole width
            classStrip: true,
            title: 'Effects',
            icon: const GlyphBox(HG.pie, size: 22, color: N.g95),
            search: search,
            classify: classify,
            groups: views.groups(found),
            hint: 'Search effects',
            count: n >= 1000 ? '${n ~/ 1000},${(n % 1000).toString().padLeft(3, '0')}' : '$n',
            wide: (c, s) => shown.isEmpty ? emptyBody('No effect matches "${search.query}".') : _grid(shown, s.width, 3),
            narrow: (c, s) => shown.isEmpty ? emptyBody('No effect matches.') : _grid(shown, s.width, s.width >= 210 ? 2 : 1),
            strip: (c, s) => ListView.builder(
              scrollDirection: Axis.horizontal,
              physics: const ClampingScrollPhysics(),
              padding: const EdgeInsets.all(10),
              itemCount: shown.length,
              itemBuilder: (_, i) => Padding(padding: const EdgeInsets.only(right: 6), child: AspectRatio(aspectRatio: 1.5, child: ThingFace(shown[i], scene: widget.scene))),
            ),
          );
        },
      );

  Widget _grid(List<Thing> shown, double w, int most) {
    const pad = 12.0, gap = 6.0;
    // as many columns as keep a tile wide enough to carry its name under it (never more than asked for)
    final cols = ((w - pad * 2 + gap) / (72 + gap)).floor().clamp(1, most);
    final tileW = (w - pad * 2 - gap * (cols - 1)) / cols;
    if (tileW <= 0) return const SizedBox.shrink(); // a seat squeezed to nothing shows nothing, not an error
    BrowserSeatScope.of(context)?.shows(shown, cols);
    final showCaption = tileW >= 60;
    return GridView.builder(
      physics: const ClampingScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(pad, 14, pad, 12),
      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: 9, crossAxisSpacing: gap, childAspectRatio: tileW / (tileW * .84 + (showCaption ? 18 : 0))),
      itemCount: shown.length,
      itemBuilder: (c, i) => seated(c, shown[i], Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        AspectRatio(aspectRatio: 1 / .84, child: ThingFace(shown[i], scene: widget.scene)),
        if (showCaption) Padding(padding: const EdgeInsets.only(top: 5), child: Text(shown[i].name, softWrap: false, maxLines: 1, overflow: TextOverflow.ellipsis, style: sans(10.5, c: N.g76))),
      ])),
    );
  }
}
