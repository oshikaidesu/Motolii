import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show mono, sans;
import '../../hf/bp/seat.dart';
import '../../hf/bp/shell.dart' show GlyphBox, kTile;
import '../../hf/bp/shelf_sections.dart';
import '../../hf/bp/things.dart';
import '../../hf/glyphs.dart';
import '../../hf/neutral.dart';
import '../../hf/shell/place.dart' show H;

/// Media on a pinboard: each piece at its own proportions (a still or clip by its width and height, an HDR plate 2:1, a
/// sound a band) in staggered columns packed tight (4 px), one line of name under each. The shelf's mechanics (picking,
/// carrying, menus, keys) are the seat's.
class MediaLibraryBody extends StatelessWidget {
  const MediaLibraryBody({super.key, required this.sections, required this.shown, required this.items, required this.width});
  final Map<String, List<Thing>> sections;
  final List<Thing> shown;
  final Map<String, Map<String, dynamic>> items;
  final double width;

  /// Width over height for a piece: what the file says, else what its family usually is.
  static double aspectOf(Map<String, dynamic> item) {
    final facts = item['facts'];
    if (facts is Map && facts['width'] is num && facts['height'] is num && (facts['height'] as num) > 0) {
      return ((facts['width'] as num) / (facts['height'] as num)).clamp(.5, 2.4).toDouble();
    }
    return switch ('${item['family']}') { 'HDR' => 2, 'Audio' => 2.4, 'Video' => 16 / 9, _ => 1 };
  }

  @override
  Widget build(BuildContext context) {
    const margin = 10.0, gap = 3.0;
    final body = width - margin * 2;
    // cards of at least 70 px: three across the default seat, more as it widens
    final cols = math.max(1, ((body + gap) / (70 + gap)).floor());
    final colW = (body - gap * (cols - 1)) / cols;
    final seat = BrowserSeatScope.of(context);
    seat?.shows([for (final e in sections.values) ...e], cols);
    if (colW <= 0) return const SizedBox.shrink();
    return CustomScrollView(
      physics: const ClampingScrollPhysics(),
      slivers: [
        for (final e in sections.entries) ...[
          if (e.key.isNotEmpty) SliverToBoxAdapter(child: Padding(padding: const EdgeInsets.symmetric(horizontal: margin), child: ShelfHeading(e.key, count: e.value.length, first: e.key == sections.keys.first))),
          SliverPadding(
            padding: EdgeInsets.fromLTRB(margin, e.key.isEmpty ? 8 : 0, margin, 0),
            sliver: SliverToBoxAdapter(child: _Board(things: e.value, items: items, cols: cols, colW: colW, gap: gap)),
          ),
        ],
        const SliverToBoxAdapter(child: SizedBox(height: 12)),
      ],
    );
  }
}

/// Staggered columns: each card goes to the shortest column, so pieces of different shapes pack without holes.
class _Board extends StatelessWidget {
  const _Board({required this.things, required this.items, required this.cols, required this.colW, required this.gap});
  final List<Thing> things;
  final Map<String, Map<String, dynamic>> items;
  final int cols;
  final double colW, gap;

  @override
  Widget build(BuildContext context) {
    final columns = [for (var i = 0; i < cols; i++) <Widget>[]];
    final heights = List.filled(cols, 0.0);
    for (final t in things) {
      final item = items[t.id] ?? const <String, dynamic>{};
      final h = colW / MediaLibraryBody.aspectOf(item) + _captionHeight;
      var at = 0;
      for (var c = 1; c < cols; c++) {
        if (heights[c] < heights[at] - 1) at = c;
      }
      heights[at] += h + gap;
      columns[at].add(Padding(padding: EdgeInsets.only(bottom: gap), child: SizedBox(height: h, child: seated(context, t, MaterialCard(item: item, name: t.name, index: things.indexOf(t) + 1, roomy: colW >= 100)))));
    }
    return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      for (var c = 0; c < cols; c++) ...[
        if (c > 0) SizedBox(width: gap),
        SizedBox(width: colW, child: Column(children: columns[c])),
      ],
    ]);
  }
}

const _captionHeight = 18.0;

/// One piece: its picture, rounded, at its own proportions; under it its name and, when the card has room, its size or
/// length.
class MaterialCard extends StatelessWidget {
  const MaterialCard({super.key, required this.item, required this.name, this.index, this.roomy = true});
  final Map<String, dynamic> item;
  final String name;
  final int? index;

  /// Wide enough for the size or length beside the name (a narrow card keeps the name).
  final bool roomy;

  @override
  Widget build(BuildContext context) {
    final missing = item['missing'] == true;
    final fact = roomy ? _fact(item) : '';
    Widget face = DecoratedBox(
      position: DecorationPosition.foreground,
      decoration: BoxDecoration(border: Border.all(color: N.glaze9, width: .5)),
      child: ClipRect(child: ColoredBox(color: N.g13, child: materialFace(item))),
    );
    if (missing) face = Opacity(opacity: .4, child: face);
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Expanded(
        child: Stack(fit: StackFit.expand, children: [
          face,
          // placed in the work: a small light dot in the corner
          if (item['used'] == true && !missing) Positioned(right: 6, top: 6, child: Container(width: 6, height: 6, decoration: BoxDecoration(color: N.g95, shape: BoxShape.circle, border: Border.all(color: N.g07.withValues(alpha: .5))))),
        ]),
      ),
      SizedBox(
        height: _captionHeight,
        child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
          if (index != null) Padding(padding: const EdgeInsets.only(right: 5), child: Text(index!.toString().padLeft(2, '0'), style: mono(8.5, c: N.signal))),
          Expanded(child: Text(missing ? 'Missing · $name' : name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(10.5, c: N.g91, w: FontWeight.w600, ls: -.1))),
          if (fact.isNotEmpty) Padding(padding: const EdgeInsets.only(left: 4), child: Text(fact, softWrap: false, style: mono(8.5, c: N.g51))),
        ]),
      ),
    ]);
  }
}

/// What the material is, drawn as itself.
Widget materialFace(Map<String, dynamic> item) => switch ('${item['family']}') {
      'Audio' => _WaveFace(peaks: item['peaks'], seconds: _seconds(item)),
      'Video' => _MotionFace(picture: _picture(item), seconds: _seconds(item)),
      'Images' => Stack(fit: StackFit.expand, children: [const CustomPaint(painter: _Checker()), _picture(item) ?? _glyph(HG.image)]),
      _ => _picture(item) ?? _glyph(HG.image),
    };

/// What a piece is, in a few characters: its length, its size, or its kind.
String _fact(Map<String, dynamic> item) {
  final seconds = _seconds(item);
  if (seconds != null) return _clock(seconds);
  final facts = item['facts'];
  if (facts is Map && facts['width'] is num && facts['height'] is num) return '${facts['width']}×${facts['height']}';
  return '${item['family'] ?? ''}' == 'HDR' ? '360°' : '';
}


double? _seconds(Map<String, dynamic> item) {
  final facts = item['facts'];
  final s = item['seconds'] ?? (facts is Map ? facts['seconds'] : null);
  return s is num && s > 0 ? s.toDouble() : null;
}

String _clock(double seconds) {
  final whole = seconds.round();
  return '${whole ~/ 60}:${(whole % 60).toString().padLeft(2, '0')}';
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

Widget _glyph(HG g) => Stack(fit: StackFit.expand, children: [const ColoredBox(color: kTile), Center(child: GlyphBox(g, size: 26))]);

/// A length or a mark on a face: small, dark, legible over any picture.
class _Badge extends StatelessWidget {
  const _Badge(this.child);
  final Widget child;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1.5),
        decoration: BoxDecoration(color: N.veil, borderRadius: BorderRadius.circular(3)),
        child: child,
      );
}

/// A clip: its picture, a play mark (it moves) and its length.
class _MotionFace extends StatelessWidget {
  const _MotionFace({required this.picture, required this.seconds});
  final Widget? picture;
  final double? seconds;
  @override
  Widget build(BuildContext context) => Stack(fit: StackFit.expand, children: [
        picture ?? _glyph(HG.image),
        const Center(child: _Badge(SizedBox(width: 8, height: 9, child: CustomPaint(painter: _Play())))),
        if (seconds != null) Positioned(right: 3, bottom: 3, child: _Badge(Text(_clock(seconds!), style: mono(9.5, c: N.g95)))),
      ]);
}

class _Play extends CustomPainter {
  const _Play();
  @override
  void paint(Canvas cv, Size s) => cv.drawPath(Path()..moveTo(0, 0)..lineTo(s.width, s.height / 2)..lineTo(0, s.height)..close(), Paint()..color = N.g95);
  @override
  bool shouldRepaint(_Play o) => false;
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
      if (seconds != null) Positioned(right: 3, bottom: 3, child: _Badge(Text(_clock(seconds!), style: mono(9.5, c: N.g95)))),
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
