// The media catalogue (footage, stills, HDRs, sounds) and its thumbnails, drawn as flat pop posters. The shelf that shows them is dock_files.dart.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'dock_glyphs.dart';
import 'ws.dart';

enum MediaType {
  video('Video', 'VID', DockG.video),
  image('Images', 'IMG', DockG.image),
  hdr('HDR', 'HDR', DockG.light),
  audio('Audio', 'AUD', DockG.audio);

  const MediaType(this.label, this.badge, this.glyph);
  final String label, badge;
  final DockG glyph;
  Color get hue => switch (this) {
    video => Pop.toneTime,
    image => WsT.toneImage,
    hdr => Pop.toneTextAnim,
    audio => WsT.toneAudio,
  };
}

class MediaAsset {
  const MediaAsset(this.id, this.name, this.type, this.seed, {this.dur = 0, required this.meta, this.layer});
  final String id, name, meta;
  final MediaType type;
  final int seed;
  final double dur;

  /// The id of the [Ws] layer that already uses this file, if any.
  final String? layer;
  String get clock => '${dur ~/ 60}:${(dur % 60).floor().toString().padLeft(2, '0')}';
}

const mediaAssets = <MediaAsset>[
  MediaAsset('coda_intro', 'coda_intro.mov', MediaType.video, 1, dur: 12, meta: '1920×1080 · 30 fps · ProRes'),
  MediaAsset('city_pass', 'city_pass.mp4', MediaType.video, 2, dur: 47, meta: '3840×2160 · 25 fps · H.264'),
  MediaAsset('ink_drop', 'ink_drop.mov', MediaType.video, 3, dur: 6, meta: '1920×1080 · 60 fps · ProRes'),
  MediaAsset('confetti_alpha', 'confetti_alpha.mov', MediaType.video, 4, dur: 4, meta: '1920×1080 · 30 fps · 4444 alpha'),
  MediaAsset('lens_flare', 'lens_flare.mp4', MediaType.video, 5, dur: 8, meta: '1920×1080 · 30 fps · H.264'),
  MediaAsset('crowd_wave', 'crowd_wave.mov', MediaType.video, 6, dur: 21, meta: '2048×1080 · 24 fps · ProRes'),
  MediaAsset('background', 'background.png', MediaType.image, 7, meta: '3840×2160 · PNG · 6.2 MB', layer: 'bg'),
  MediaAsset('poster_grid', 'poster_grid.jpg', MediaType.image, 8, meta: '2400×3000 · JPEG · 3.1 MB'),
  MediaAsset('halftone', 'halftone_dots.png', MediaType.image, 9, meta: '2048×2048 · PNG · 1.4 MB'),
  MediaAsset('logo_mark', 'logo_mark.svg', MediaType.image, 10, meta: 'Vector · SVG · 18 KB'),
  MediaAsset('paper_grain', 'paper_grain.jpg', MediaType.image, 11, meta: '4096×4096 · JPEG · 9.8 MB'),
  MediaAsset('stickers', 'sticker_sheet.png', MediaType.image, 12, meta: '3000×2000 · PNG · 4.4 MB'),
  MediaAsset('studio_soft', 'studio_soft.exr', MediaType.hdr, 13, meta: '8192×4096 · EXR · 92 MB'),
  MediaAsset('rooftop', 'rooftop_dusk.hdr', MediaType.hdr, 14, meta: '4096×2048 · HDR · 24 MB'),
  MediaAsset('beat', 'Beat.wav', MediaType.audio, 15, dur: 8, meta: '120 BPM · 48 kHz · loop', layer: 'music'),
  MediaAsset('whoosh', 'whoosh_01.wav', MediaType.audio, 16, dur: 1, meta: '48 kHz · stereo'),
  MediaAsset('riser', 'riser_long.wav', MediaType.audio, 17, dur: 6, meta: '48 kHz · stereo'),
  MediaAsset('vo', 'vo_take_3.wav', MediaType.audio, 18, dur: 14, meta: '48 kHz · mono'),
];

/// A file's thumbnail: its pop poster (or waveform), cached in its own layer.
class MediaThumb extends StatelessWidget {
  const MediaThumb(this.a, {super.key});
  final MediaAsset a;
  @override
  Widget build(BuildContext context) => RepaintBoundary(
    child: CustomPaint(painter: PopArt(a.seed, a.type), size: Size.infinite),
  );
}

/// Thumbnail art as a pop poster: a flat ground and two or three flat saturated shapes, no gradients, no blur.
class PopArt extends CustomPainter {
  const PopArt(this.seed, this.type);
  final int seed;
  final MediaType type;

  static final hues = [WsT.accent, Fam.stagger.c, WsT.toneShape, Fam.scatter.c, Fam.face.c, WsT.toneText, Fam.along.c, Fam.attach.c];

  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height, m = math.min(w, h);
    final rnd = math.Random(seed * 7919);
    final bgI = seed % hues.length;
    Color pick(int k) => hues[(bgI + 1 + (seed + k * 3) % (hues.length - 1)) % hues.length];
    final a = pick(0), b = pick(1), ink = WsT.onAccent;
    Paint f(Color c) => Paint()..color = c;
    cv.clipRect(Offset.zero & s);

    if (type == MediaType.audio) {
      cv.drawRect(Offset.zero & s, f(Grey.g15));
      final bars = math.max(10, (w / 4).floor());
      final pt = Paint()
        ..color = type.hue
        ..strokeWidth = math.max(1.5, w / bars * .55)
        ..strokeCap = StrokeCap.round;
      for (var i = 0; i < bars; i++) {
        final x = w * (i + .5) / bars, env = math.sin(math.pi * (i + .5) / bars);
        final amp = h * .4 * (.25 + .75 * rnd.nextDouble()) * (.3 + .7 * env);
        cv.drawLine(Offset(x, h / 2 - amp), Offset(x, h / 2 + amp), pt);
      }
      return;
    }
    if (type == MediaType.hdr) {
      cv.drawRect(Offset.zero & s, f(Fam.stagger.c));
      cv.drawRect(Rect.fromLTWH(0, h * .45, w, h * .2), f(WsT.toneShape));
      cv.drawCircle(Offset(w * (.3 + .4 * rnd.nextDouble()), h * .45), m * .2, f(Fam.face.c));
      cv.drawRect(Rect.fromLTWH(0, h * .62, w, h * .38), f(ink));
      for (var i = 0; i < 4; i++) {
        cv.drawRect(Rect.fromLTWH(w * (i * .26 + .04), h * (.62 - .1 * rnd.nextDouble() - .05), w * .14, h * .4), f(ink));
      }
      return;
    }

    cv.drawRect(Offset.zero & s, f(hues[bgI]));
    switch (seed % 6) {
      case 0:
        cv.drawCircle(Offset(w * .62, h * .5), m * .42, f(a));
        for (var i = 0; i < 4; i++) {
          cv.drawRect(Rect.fromLTWH(0, h * (.2 + i * .18), w * .45, h * .08), f(ink));
        }
      case 1:
        cv.drawPath(Path()..addPolygon([Offset(0, h), Offset(w, 0), Offset(w, h)], true), f(a));
        cv.drawCircle(Offset(w * .3, h * .34), m * .2, f(b));
        cv.drawRect(Rect.fromLTWH(w * .58, h * .62, w * .3, h * .12), f(ink));
      case 2:
        for (var i = 4; i > 0; i--) {
          cv.drawCircle(Offset(w * .5, h * .55), m * .13 * i, f(i.isEven ? a : ink));
        }
      case 3:
        final step = m / 6;
        for (var y = step / 2; y < h; y += step) {
          for (var x = step / 2; x < w; x += step) {
            cv.drawCircle(Offset(x, y), step * (.12 + .3 * (x / w)), f(ink));
          }
        }
        cv.drawRect(Rect.fromLTWH(w * .12, h * .58, w * .5, h * .22), f(a));
      case 4:
        final blob = Path()
          ..moveTo(w * .2, h * .5)
          ..cubicTo(w * .2, h * .05, w * .9, h * .1, w * .8, h * .5)
          ..cubicTo(w * .72, h * .95, w * .2, h * .95, w * .2, h * .5)
          ..close();
        cv.drawPath(blob, f(a));
        cv.drawRect(Rect.fromLTWH(w * .1, h * .78, w * .8, h * .08), f(ink));
        cv.drawCircle(Offset(w * .62, h * .42), m * .1, f(b));
      default:
        final c = Offset(w * .5, h * .55);
        for (var i = 0; i < 12; i += 2) {
          final a0 = i * math.pi / 6;
          cv.drawPath(
            Path()
              ..moveTo(c.dx, c.dy)
              ..lineTo(c.dx + math.cos(a0) * w, c.dy + math.sin(a0) * w)
              ..lineTo(c.dx + math.cos(a0 + math.pi / 6) * w, c.dy + math.sin(a0 + math.pi / 6) * w)
              ..close(),
            f(a),
          );
        }
        cv.drawCircle(c, m * .2, f(ink));
        cv.drawCircle(c, m * .1, f(b));
    }
    if (type == MediaType.video) {
      cv.drawRect(Rect.fromLTWH(0, h - 2, w * (.2 + .6 * rnd.nextDouble()), 2), f(ink));
    }
  }

  @override
  bool shouldRepaint(PopArt o) => o.seed != seed || o.type != type;
}
