import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show sans;
import '../../hf/bp/seat.dart';
import '../../hf/bp/shell.dart' show GlyphBox, kTile;
import '../../hf/bp/shelf_sections.dart';
import '../../hf/bp/things.dart';
import '../../hf/glyphs.dart';
import '../../hf/neutral.dart';
import '../../hf/shell/place.dart' show H;
import 'model_face.dart';

/// Media on a pinboard: each piece at its own proportions (a still or clip by its width and height, an HDR plate 2:1, a
/// sound a band) in staggered columns packed tight (4 px), one line of name under each. The shelf's mechanics (picking,
/// carrying, menus, keys) are the seat's.
class MediaLibraryBody extends StatelessWidget {
  const MediaLibraryBody({super.key, required this.sections, required this.shown, required this.items, required this.width, this.selected, this.onTap, this.onOpen});
  final Map<String, List<Thing>> sections;
  final List<Thing> shown;
  final Map<String, Map<String, dynamic>> items;
  final double width;

  /// A skin that chooses (the catalog's Browser): the chosen piece is ringed, a tap chooses, a double tap opens it. Without
  /// these the shelf's own seat does all of that.
  final String? selected;
  final ValueChanged<String>? onTap, onOpen;

  /// Width over height for a piece: what the file says, else what its family usually is.
  static double aspectOf(Map<String, dynamic> item) {
    final facts = item['facts'];
    if (facts is Map && facts['width'] is num && facts['height'] is num && (facts['height'] as num) > 0) {
      return ((facts['width'] as num) / (facts['height'] as num)).clamp(.5, 2.4).toDouble();
    }
    return switch (mediaKind(item)) { 'HDR' => 2, 'Audio' => 1.7, 'Video' => 16 / 9, _ => 1 };
  }

  @override
  Widget build(BuildContext context) {
    const margin = 6.0, gap = 3.0;
    final body = width - margin * 2;
    // cards of at least 44 px: four across the default seat, more as it widens
    final cols = math.max(1, ((body + gap) / (44 + gap)).floor());
    final colW = (body - gap * (cols - 1)) / cols;
    final seat = BrowserSeatScope.of(context);
    // every kind on one board, no headings: the kinds alternate (a stack of one kind would read as a section)
    final byFamily = <String, List<Thing>>{};
    for (final t in shown) {
      (byFamily[mediaKind(items[t.id] ?? const {})] ??= []).add(t);
    }
    final mixed = [
      for (var i = 0; byFamily.values.any((l) => i < l.length); i++)
        for (final l in byFamily.values)
          if (i < l.length) l[i],
    ];
    final board = {'': mixed};
    seat?.shows(mixed, cols);
    if (colW <= 0) return const SizedBox.shrink();
    return CustomScrollView(
      physics: const ClampingScrollPhysics(),
      slivers: [
        for (final e in board.entries) ...[
          if (e.key.isNotEmpty) SliverToBoxAdapter(child: Padding(padding: const EdgeInsets.symmetric(horizontal: margin), child: ShelfHeading(e.key, count: e.value.length, first: e.key == sections.keys.first))),
          SliverPadding(
            padding: EdgeInsets.fromLTRB(margin, e.key.isEmpty ? 8 : 0, margin, 0),
            sliver: SliverToBoxAdapter(child: _Board(things: e.value, items: items, cols: cols, colW: colW, gap: gap, selected: selected, onTap: onTap, onOpen: onOpen)),
          ),
        ],
        const SliverToBoxAdapter(child: SizedBox(height: 9)),
      ],
    );
  }
}

/// Staggered columns: each card goes to the shortest column, so pieces of different shapes pack without holes.
class _Board extends StatelessWidget {
  const _Board({required this.things, required this.items, required this.cols, required this.colW, required this.gap, this.selected, this.onTap, this.onOpen});
  final String? selected;
  final ValueChanged<String>? onTap, onOpen;
  final List<Thing> things;
  final Map<String, Map<String, dynamic>> items;
  final int cols;
  final double colW, gap;

  /// The shelf's seat (its picking, carrying and menus) when it has one; else this skin's own tap and double tap.
  Widget _chosen(BuildContext context, Thing t, Widget card) {
    if (onTap == null) return seated(context, t, card);
    return GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => onTap!(t.id), onDoubleTap: onOpen == null ? null : () => onOpen!(t.id), child: card);
  }

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
      columns[at].add(Padding(padding: EdgeInsets.only(bottom: gap), child: SizedBox(height: h, child: _chosen(context, t, MaterialCard(item: item, name: t.name, index: things.indexOf(t) + 1, roomy: colW >= 100, selected: selected == t.id)))));
    }
    return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      for (var c = 0; c < cols; c++) ...[
        if (c > 0) SizedBox(width: gap),
        SizedBox(width: colW, child: Column(children: columns[c])),
      ],
    ]);
  }
}

const _captionHeight = 14.0;

/// One piece: its picture, rounded, at its own proportions; under it its name and, when the card has room, its size or
/// length.
class MaterialCard extends StatelessWidget {
  const MaterialCard({super.key, required this.item, required this.name, this.index, this.roomy = true, this.selected = false});
  final bool selected;
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
      decoration: BoxDecoration(borderRadius: BorderRadius.circular(3), border: Border.all(color: N.glaze9)),
      child: ClipRRect(borderRadius: BorderRadius.circular(3), child: ColoredBox(color: N.g13, child: materialFace(item))),
    );
    if (missing) face = Opacity(opacity: .4, child: face);
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Expanded(
        child: Stack(fit: StackFit.expand, children: [
          face,
          // what kind of thing it is, where the eye lands first: ▶ a clip, ♪ a sound, 3D a model, 360° an environment
          if (_mark(item) case final mark?) Positioned(left: 2.5, top: 2.5, child: _Badge(Text(mark, softWrap: false, style: sans(8.5, c: mediaKind(item) == 'HDR' ? N.g86 : N.g95, w: mediaKind(item) == 'HDR' ? FontWeight.w500 : FontWeight.w600)))),
          if (selected) const Positioned.fill(child: PickedRing()),
          // placed in the work: a small light dot in the corner
          if (item['used'] == true && !missing) Positioned(right: 4.5, top: 4.5, child: Container(width: 4.5, height: 4.5, decoration: BoxDecoration(color: N.g95, shape: BoxShape.circle, border: Border.all(color: N.g07.withValues(alpha: .5))))),
        ]),
      ),
      SizedBox(
        height: _captionHeight,
        child: Row(crossAxisAlignment: CrossAxisAlignment.end, children: [
          Expanded(child: Text(missing ? 'Missing · $name' : name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(9.5, c: N.g82, w: FontWeight.w500))),
          if (fact.isNotEmpty) Padding(padding: const EdgeInsets.only(left: 3), child: Text(fact, softWrap: false, style: sans(9.5, c: N.g51))),
        ]),
      ),
    ]);
  }
}

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

/// The corner mark of a piece that is not a still: its kind (and, for a clip or a sound, its length).
String? _mark(Map<String, dynamic> item) {
  final seconds = _seconds(item);
  return switch (mediaKind(item)) {
    'Video' => seconds == null ? '▶' : '▶ ${_clock(seconds)}',
    'Audio' => seconds == null ? '♪' : '♪ ${_clock(seconds)}',
    '3D' => '3D',
    'HDR' => '360°',
    _ => null,
  };
}

/// What a piece is, beside its name: its size (a sound has none; its length is in the mark).
String _fact(Map<String, dynamic> item) {
  if (mediaKind(item) == 'Audio' || mediaKind(item) == 'HDR') return '';
  final facts = item['facts'];
  if (facts is Map && facts['width'] is num && facts['height'] is num) return '${facts['width']}×${facts['height']}';
  return '';
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
        padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 1.5),
        decoration: BoxDecoration(color: N.veil, borderRadius: BorderRadius.circular(2)),
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
