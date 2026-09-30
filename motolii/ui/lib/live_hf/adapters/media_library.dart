import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import '../../hf/bp/shell.dart' show GlyphBox;
import '../../hf/glyphs.dart';
import '../../hf/neutral.dart';
import '../../hf/shell/place.dart' show H;
import 'model_face.dart';
import '../../hf/metrics.dart' show Surface;

/// What the material is, drawn as itself.
Widget materialFace(Map<String, dynamic> item) => switch (mediaKind(item)) {
      'Audio' => _WaveFace(peaks: item['peaks'], seconds: _seconds(item)),
      'Video' => _MotionFace(picture: _picture(item), seconds: _seconds(item)),
      '3D' => item['path'] is String && item['missing'] != true && ModelFace.reads(item['path'] as String) ? ModelFace(path: item['path'] as String, fallback: _picture(item) ?? _glyph(HG.image)) : (_picture(item) ?? _glyph(HG.image)),
      '2D' => Stack(fit: StackFit.expand, children: [const CustomPaint(painter: _Checker()), _picture(item) ?? _glyph(HG.image)]),
      _ => _picture(item) ?? _glyph(HG.image),
    };

/// What a piece is by the library's own family word (2D, Video, Audio, 3D, HDR, Folder): the shelf's class labels are marks, this is not.
String mediaKind(Map<String, dynamic> item) => '${item['mediaFamily'] ?? item['family']}';

double? _seconds(Map<String, dynamic> item) {
  final facts = item['facts'];
  final s = item['seconds'] ?? (facts is Map ? facts['seconds'] : null);
  return s is num && s > 0 ? s.toDouble() : null;
}

/// Decoded pictures, kept by their data (the shelf rebuilds often; decoding base64 each time would be felt).
final _decoded = <int, Uint8List>{};

Widget? _picture(Map<String, dynamic> item) {
  // a still on disk is drawn from its file (a card is larger than the thumbnail), decoded no larger than it is shown
  final file = item['path'];
  if (file is String && item['missing'] != true && '${item['mime']}'.startsWith('image/')) {
    return Image.file(File(file), fit: BoxFit.cover, cacheWidth: 640, gaplessPlayback: true, errorBuilder: (_, __, ___) => _thumbnail(item) ?? _glyph(HG.image));
  }
  return _thumbnail(item);
}

Widget? _thumbnail(Map<String, dynamic> item) {
  final thumbnail = item['thumbnail'];
  if (thumbnail is String && thumbnail.startsWith('data:')) {
    final key = Object.hash(item['id'], thumbnail.length);
    final bytes = _decoded[key] ??= (() {
      final comma = thumbnail.indexOf(',');
      try {
        return comma < 0 ? Uint8List(0) : base64Decode(thumbnail.substring(comma + 1));
      } catch (_) {
        return Uint8List(0);
      }
    })();
    if (bytes.isNotEmpty) return Image.memory(bytes, fit: BoxFit.cover, gaplessPlayback: true);
  }
  final path = item['path'];
  if (path is String && '${item['mime']}'.startsWith('image/')) {
    return Image.file(File(path), fit: BoxFit.cover, errorBuilder: (_, __, ___) => _glyph(HG.image));
  }
  return null;
}

Widget _glyph(HG g) => Stack(fit: StackFit.expand, children: [const ColoredBox(color: Surface.raised), Center(child: GlyphBox(g, size: 26))]);

/// A clip: its picture, a play mark (it moves) and its length.
class _MotionFace extends StatelessWidget {
  const _MotionFace({required this.picture, required this.seconds});
  final Widget? picture;
  final double? seconds;
  @override
  Widget build(BuildContext context) => Stack(fit: StackFit.expand, children: [
        picture ?? _glyph(HG.image),
      ]);
}

/// The Timeline's waveform hue (its quiet floor and its trace), lifted so a sound reads at shelf size.
const _waveFloor = N.g13;
final _waveTrace = Color.lerp(H.wave, N.g100, .38)!;

/// A sound: its own envelope across its length, and its length.
class _WaveFace extends StatelessWidget {
  const _WaveFace({required this.peaks, required this.seconds});
  final Object? peaks;
  final double? seconds;
  @override
  Widget build(BuildContext context) {
    final columns = [
      if (peaks is List)
        for (final c in peaks as List)
          if (c is List && c.length == 2 && c[0] is num && c[1] is num) ((c[0] as num).toDouble(), (c[1] as num).toDouble()),
    ];
    return Stack(fit: StackFit.expand, children: [
      const ColoredBox(color: _waveFloor),
      if (columns.isEmpty) const Center(child: GlyphBox(HG.headphones, size: 24)) else CustomPaint(painter: _Wave(columns)),
    ]);
  }
}

class _Wave extends CustomPainter {
  _Wave(this.columns);
  final List<(double, double)> columns;
  @override
  void paint(Canvas cv, Size s) {
    final mid = s.height / 2, half = s.height * .4;
    final step = s.width / columns.length;
    final bar = Paint()..color = _waveTrace..strokeWidth = (step * .6).clamp(1.0, 3.0)..strokeCap = StrokeCap.round;
    cv.drawLine(Offset(0, mid), Offset(s.width, mid), Paint()..color = _waveTrace.withValues(alpha: .35)..strokeWidth = 1);
    for (final (i, (lo, hi)) in columns.indexed) {
      final x = step * (i + .5);
      final top = mid - hi.clamp(0.0, 1.0) * half, bottom = mid - lo.clamp(-1.0, 0.0) * half;
      if (bottom - top < 1) continue;
      cv.drawLine(Offset(x, top), Offset(x, bottom), bar);
    }
  }

  @override
  bool shouldRepaint(_Wave o) => o.columns != columns;
}

/// Behind a still: transparent pixels show as a checker, opaque ones cover it.
class _Checker extends CustomPainter {
  const _Checker();
  @override
  void paint(Canvas cv, Size s) {
    const cell = 6.0;
    cv.drawRect(Offset.zero & s, Paint()..color = N.g15);
    final light = Paint()..color = N.g20;
    for (var y = 0; y * cell < s.height; y++) {
      for (var x = (y.isEven ? 0 : 1); x * cell < s.width; x += 2) {
        cv.drawRect(Rect.fromLTWH(x * cell, y * cell, cell, cell), light);
      }
    }
  }

  @override
  bool shouldRepaint(_Checker o) => false;
}
