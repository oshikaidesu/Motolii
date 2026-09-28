import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show kMuted, mono, sans;
import '../../hf/bp/seat.dart';
import '../../hf/bp/shell.dart' show GlyphBox, kTile;
import '../../hf/bp/shelf_sections.dart';
import '../../hf/bp/things.dart';
import '../../hf/glyphs.dart';

/// Media as material: each family is shown by what it is — a still by its picture (over a checker, so a cut-out reads
/// as one), a clip by its picture with its motion and length, a sound by its own waveform. The name is always readable;
/// size, rate and length are the quiet second line. The shelf's mechanics (picking, carrying, menus, keys) are the seat's.
class MediaLibraryBody extends StatelessWidget {
  const MediaLibraryBody({super.key, required this.sections, required this.shown, required this.items, required this.width});
  final Map<String, List<Thing>> sections;
  final List<Thing> shown;
  final Map<String, Map<String, dynamic>> items;
  final double width;

  @override
  Widget build(BuildContext context) {
    final grid = shelfColumns(width, 108);
    // the keys walk the tiles in the order they are drawn (section by section), not the data's order
    BrowserSeatScope.of(context)?.shows([for (final e in sections.values) ...e], grid.columns);
    if (grid.width <= 0) return const SizedBox.shrink();
    final extent = grid.width / _faceAspect + _captionHeight;
    return CustomScrollView(
      physics: const ClampingScrollPhysics(),
      slivers: [
        for (final e in sections.entries) ...[
          if (e.key.isNotEmpty) SliverToBoxAdapter(child: Padding(padding: const EdgeInsets.symmetric(horizontal: kShelfPad), child: ShelfHeading(e.key, count: e.value.length))),
          SliverPadding(
            padding: EdgeInsets.fromLTRB(kShelfPad, e.key.isEmpty ? 3 * kShelfUnit : 0, kShelfPad, 0),
            sliver: SliverGrid(
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: grid.columns, mainAxisSpacing: kShelfGap + kShelfUnit, crossAxisSpacing: kShelfGap, childAspectRatio: grid.width / extent),
              delegate: SliverChildBuilderDelegate((c, i) => seated(c, e.value[i], MaterialCard(item: items[e.value[i].id] ?? const {}, name: e.value[i].name)), childCount: e.value.length),
            ),
          ),
        ],
        const SliverToBoxAdapter(child: SizedBox(height: 4 * kShelfUnit)),
      ],
    );
  }
}

const _faceAspect = 16 / 10;
const _captionHeight = 1.5 * kShelfUnit + 15 + 14;

/// One piece of material: its face (landscape, never cropped to a postage stamp), its name, one quiet line of facts.
class MaterialCard extends StatelessWidget {
  const MaterialCard({super.key, required this.item, required this.name});
  final Map<String, dynamic> item;
  final String name;

  @override
  Widget build(BuildContext context) {
    final missing = item['missing'] == true;
    Widget face = ClipRRect(borderRadius: BorderRadius.circular(3), child: AspectRatio(aspectRatio: _faceAspect, child: materialFace(item)));
    if (missing) face = Opacity(opacity: .4, child: face);
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      face,
      const SizedBox(height: 1.5 * kShelfUnit),
      Text(name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(12, c: const Color(0xFFE4E5E8), w: FontWeight.w500)),
      const SizedBox(height: 3),
      Text(missing ? 'Missing file' : materialFacts(item), maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: mono(10, c: kMuted)),
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

/// The quiet line: size for pictures, size and rate for clips, rate and channels for sounds.
String materialFacts(Map<String, dynamic> item) {
  final facts = item['facts'] is Map ? Map<String, dynamic>.from(item['facts']) : const <String, dynamic>{};
  final size = facts['width'] is num && facts['height'] is num ? '${facts['width']}×${facts['height']}' : null;
  return switch ('${item['family']}') {
    'Video' => [if (size != null) size, if (facts['fps'] is num) '${_trim(facts['fps'] as num)} fps'].join(' · '),
    'Audio' => [
        if (facts['sampleRate'] is num) '${_trim((facts['sampleRate'] as num) / 1000)} kHz',
        if (facts['channels'] is num) (facts['channels'] as num) == 1 ? 'mono' : (facts['channels'] as num) == 2 ? 'stereo' : '${facts['channels']} ch',
      ].join(' · '),
    'HDR' => '${item['detail'] ?? 'HDR'}',
    _ => size ?? '${item['family'] ?? ''}',
  };
}

String _trim(num v) => v == v.roundToDouble() ? '${v.round()}' : v.toStringAsFixed(1);

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
        padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
        decoration: BoxDecoration(color: const Color(0xC0101012), borderRadius: BorderRadius.circular(3)),
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
        const Positioned(left: 6, bottom: 6, child: _Badge(SizedBox(width: 9, height: 10, child: CustomPaint(painter: _Play())))),
        if (seconds != null) Positioned(right: 6, bottom: 6, child: _Badge(Text(_clock(seconds!), style: mono(10.5, c: const Color(0xFFF2F2F4))))),
      ]);
}

class _Play extends CustomPainter {
  const _Play();
  @override
  void paint(Canvas cv, Size s) => cv.drawPath(Path()..moveTo(0, 0)..lineTo(s.width, s.height / 2)..lineTo(0, s.height)..close(), Paint()..color = const Color(0xFFF2F2F4));
  @override
  bool shouldRepaint(_Play o) => false;
}

/// The Timeline's waveform hue (its quiet floor and its trace), lifted so a sound reads at shelf size.
const _waveFloor = Color(0xFF1E2622);
final _waveTrace = Color.lerp(const Color(0xFF3B6D5F), const Color(0xFFFFFFFF), .38)!;

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
      if (seconds != null) Positioned(right: 6, bottom: 6, child: _Badge(Text(_clock(seconds!), style: mono(10.5, c: const Color(0xFFF2F2F4))))),
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
    const cell = 8.0;
    cv.drawRect(Offset.zero & s, Paint()..color = const Color(0xFF2A2A2D));
    final light = Paint()..color = const Color(0xFF38383C);
    for (var y = 0; y * cell < s.height; y++) {
      for (var x = (y.isEven ? 0 : 1); x * cell < s.width; x += 2) {
        cv.drawRect(Rect.fromLTWH(x * cell, y * cell, cell, cell), light);
      }
    }
  }

  @override
  bool shouldRepaint(_Checker o) => false;
}
