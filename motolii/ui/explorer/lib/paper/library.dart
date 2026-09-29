// Paper: the whole Media library (2D, video, sound, 3D, environments), each kind with the face it needs, in three views of
// the same 320 px seat: Thumbnail (pinboard), Detail (a row each), Explore (everything in one sheet, no scroll).
// Nothing is wired; every fact shown is the file's own.
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../story.dart' show fixture;
import 'library_data.dart';
import 'media.dart' show ground, ink, quiet, faint, rule, t, header, sheet;
import 'models3d.dart';

double aspectOf(L l) => switch (l.kind) { 'Audio' => 1.8, '3D' => 1.0, 'HDR' => 2.0, _ => (l.w / l.h).clamp(.5, 2.4).toDouble() };

String clock(double s) => '${s ~/ 60}:${(s.round() % 60).toString().padLeft(2, '0')}';

String factOf(L l) => switch (l.kind) {
      'Audio' => '${clock(l.sec!)}  ${(l.hz! / 1000).toStringAsFixed(l.hz! % 1000 == 0 ? 0 : 1)} kHz  ${l.ch == 1 ? 'mono' : 'stereo'}',
      '3D' => '',
      'HDR' => '360°',
      'Video' => '${l.w}×${l.h}  ${clock(l.sec!)}',
      _ => '${l.w}×${l.h}',
    };

class _Wave extends CustomPainter {
  const _Wave(this.peaks);
  final List<List<double>> peaks;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = const Color(0xFF202020));
    final p = Paint()..color = const Color(0xFF8E8E8E)..strokeWidth = math.max(1, s.width / peaks.length - 1);
    final mid = s.height / 2, scale = mid * .9;
    for (var i = 0; i < peaks.length; i++) {
      final x = (i + .5) * s.width / peaks.length;
      c.drawLine(Offset(x, mid - peaks[i][1] * scale - .5), Offset(x, mid - peaks[i][0] * scale + .5), p);
    }
  }

  @override
  bool shouldRepaint(_Wave o) => false;
}

class _Checker extends CustomPainter {
  const _Checker();
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = const Color(0xFF262626));
    final light = Paint()..color = const Color(0xFF343434);
    for (var y = 0; y * 6 < s.height; y++) {
      for (var x = y.isEven ? 0 : 1; x * 6 < s.width; x += 2) {
        c.drawRect(Rect.fromLTWH(x * 6.0, y * 6.0, 6, 6), light);
      }
    }
  }

  @override
  bool shouldRepaint(_Checker o) => false;
}

/// What a file looks like, by what it is: a picture, a frame, its own sound-shape, the model itself, the panorama.
Widget face(L l) => switch (l.kind) {
      'Audio' => CustomPaint(painter: _Wave(l.peaks!), size: Size.infinite),
      '3D' => ColoredBox(color: const Color(0xFF202020), child: ModelFace(l.model!)),
      _ => Stack(fit: StackFit.expand, children: [
          if (l.alpha) const CustomPaint(painter: _Checker()),
          Image.file(File(fixture(l.pic)), fit: l.alpha ? BoxFit.contain : BoxFit.cover, cacheWidth: 480),
        ]),
    };

/// The mark on a tile that is not a still: what it is (and its length), small, in the corner, on a dark chip.
Widget marked(L l, Widget child) {
  final mark = switch (l.kind) { 'Video' => '▶ ${clock(l.sec!)}', 'Audio' => '♪ ${clock(l.sec!)}', '3D' => '3D', 'HDR' => '360°', _ => '' };
  if (mark.isEmpty) return child;
  return Stack(fit: StackFit.expand, children: [
    child,
    Positioned(left: 3, top: 3, child: Container(
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1.5),
      // the 360° plates are the most numerous kind: theirs is the quietest chip
      decoration: BoxDecoration(color: Color(l.kind == 'HDR' ? 0x66000000 : 0xB3000000), borderRadius: BorderRadius.circular(3)),
      child: Text(mark, style: t(8.5, Color(l.kind == 'HDR' ? 0xCCF2F2F2 : 0xFFF2F2F2), l.kind == 'HDR' ? FontWeight.w500 : FontWeight.w600)),
    )),
  ]);
}

bool stress = false;

Map<String, List<L>> byKind() {
  final m = <String, List<L>>{};
  final all = stress ? [...library, for (var i = 0; i < 5; i++) ...library.where((l) => l.kind == 'HDR')] : library;
  for (final l in all) {
    (m[l.kind == '2D' ? 'Images' : l.kind] ??= []).add(l);
  }
  return m;
}

Widget _label(String s, int n) => Padding(
      padding: const EdgeInsets.fromLTRB(0, 12, 0, 6),
      child: Text.rich(TextSpan(children: [TextSpan(text: s, style: t(11, ink, FontWeight.w600)), TextSpan(text: '  $n', style: t(10.5, faint))])),
    );

// Thumbnail: the pinboard, four across, each thing at its own shape, a caption under each.
Widget thumbnail(double w) {
  const margin = 6.0, gap = 3.0, cap = 14.0;
  final cols = math.max(1, ((w - margin * 2 + gap) / (58 + gap)).floor());
  final colW = (w - margin * 2 - gap * (cols - 1)) / cols;
  return ListView(padding: const EdgeInsets.symmetric(horizontal: margin), children: [
    for (final e in byKind().entries) ...[
      _label(e.key, e.value.length),
      Builder(builder: (_) {
        final columns = [for (var i = 0; i < cols; i++) <Widget>[]];
        final heights = List.filled(cols, 0.0);
        for (final l in e.value) {
          final h = colW / aspectOf(l) + cap;
          var at = 0;
          for (var c = 1; c < cols; c++) {
            if (heights[c] < heights[at] - 1) at = c;
          }
          heights[at] += h + gap;
          columns[at].add(Padding(padding: const EdgeInsets.only(bottom: gap), child: SizedBox(height: h, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Expanded(child: ClipRRect(borderRadius: BorderRadius.circular(4), child: marked(l, face(l)))),
            SizedBox(height: cap, child: Align(alignment: Alignment.bottomLeft, child: Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(9.5, const Color(0xFFD0D0D0), FontWeight.w500)))),
          ]))));
        }
        return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [for (var c = 0; c < cols; c++) ...[if (c > 0) const SizedBox(width: gap), SizedBox(width: colW, child: Column(children: columns[c]))]]);
      }),
    ],
    const SizedBox(height: 12),
  ]);
}

// Detail: a row each, the face at its own shape in a fixed box, the name over the file's own facts.
Widget detail(double w) => ListView(padding: const EdgeInsets.symmetric(horizontal: 8), children: [
      for (final e in byKind().entries) ...[
        _label(e.key, e.value.length),
        for (final l in e.value)
          Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Row(children: [
              SizedBox(width: 86, height: 48, child: Center(child: AspectRatio(aspectRatio: aspectOf(l), child: ClipRRect(borderRadius: BorderRadius.circular(3), child: face(l))))),
              const SizedBox(width: 10),
              Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(11.5)),
                const SizedBox(height: 3),
                Text([l.fmt, factOf(l)].where((s) => s.isNotEmpty).join('   '), maxLines: 1, overflow: TextOverflow.ellipsis, style: t(9.5, faint)),
              ])),
            ]),
          ),
      ],
    ]);

// Explore: the whole library in one seat without scrolling, every face whole and touching, a hairline between the kinds.
Widget explore(double w, double height) {
  final kinds = byKind();
  List<List<L>> rowsOf(List<L> ps, double target) {
    final rows = <List<L>>[];
    var r = <L>[];
    for (final l in ps) {
      r.add(l);
      if (w / r.fold(0.0, (a, l) => a + aspectOf(l)) <= target) {
        rows.add(r);
        r = [];
      }
    }
    if (r.isNotEmpty) rows.add(r);
    return rows;
  }

  double used(double target) => kinds.values.fold(0.0, (a, ps) => a + rowsOf(ps, target).fold(0.0, (b, r) => b + math.min(target * 1.6, w / r.fold(0.0, (x, l) => x + aspectOf(l))))) + 6.0 * (kinds.length - 1);
  var lo = 16.0, hi = 300.0;
  for (var i = 0; i < 30; i++) {
    final mid = (lo + hi) / 2;
    if (used(mid) > height - 40) {
      hi = mid;
    } else {
      lo = mid;
    }
  }
  return Column(children: [
    for (final (i, e) in kinds.entries.indexed) ...[
      if (i > 0) const SizedBox(height: 6),
      for (final r in rowsOf(e.value, lo))
        Builder(builder: (_) {
          final h = math.min(lo * 1.6, w / r.fold(0.0, (a, l) => a + aspectOf(l)));
          return Row(children: [for (final l in r) SizedBox(width: h * aspectOf(l), height: h, child: face(l))]);
        }),
    ],
  ]);
}

// Thumbnail, all kinds mixed: one pinboard, no sections; kinds alternate so no kind clumps.
Widget mixed(double w) {
  const margin = 6.0, gap = 3.0, cap = 14.0;
  final cols = math.max(1, ((w - margin * 2 + gap) / (58 + gap)).floor());
  final colW = (w - margin * 2 - gap * (cols - 1)) / cols;
  final lists = byKind().values.map((e) => [...e]).toList();
  final order = <L>[];
  for (var i = 0; lists.any((l) => i < l.length); i++) {
    for (final l in lists) {
      if (i < l.length) order.add(l[i]);
    }
  }
  final columns = [for (var i = 0; i < cols; i++) <Widget>[]];
  final heights = List.filled(cols, 0.0);
  for (final l in order) {
    final h = colW / aspectOf(l) + cap;
    var at = 0;
    for (var c = 1; c < cols; c++) {
      if (heights[c] < heights[at] - 1) at = c;
    }
    heights[at] += h + gap;
    columns[at].add(Padding(padding: const EdgeInsets.only(bottom: gap), child: SizedBox(height: h, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Expanded(child: ClipRRect(borderRadius: BorderRadius.circular(4), child: marked(l, face(l)))),
      SizedBox(height: cap, child: Align(alignment: Alignment.bottomLeft, child: Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(9.5, const Color(0xFFD0D0D0), FontWeight.w500)))),
    ]))));
  }
  return ListView(padding: const EdgeInsets.fromLTRB(margin, 8, margin, 12), children: [
    Row(crossAxisAlignment: CrossAxisAlignment.start, children: [for (var c = 0; c < cols; c++) ...[if (c > 0) const SizedBox(width: gap), SizedBox(width: colW, child: Column(children: columns[c]))]]),
  ]);
}

/// The header's filter is the tiles' own marks (the tabs are the legend): All, then a still, ▶, ♪, 3D, 360°.
Widget libraryHeader(int count, {String on = 'All'}) => Container(
      height: 34,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: rule))),
      child: Row(children: [
        for (final k in const ['All', '▣', '▶', '♪', '3D', '360°'])
          Padding(
            padding: const EdgeInsets.only(right: 4),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
              decoration: BoxDecoration(color: k == on ? const Color(0xFF343434) : null, borderRadius: BorderRadius.circular(4)),
              child: Text(k, style: t(11, k == on ? ink : quiet, k == on ? FontWeight.w600 : FontWeight.w500)),
            ),
          ),
        const Spacer(),
        Text('+ Import', style: t(11, quiet)),
        const SizedBox(width: 8),
        Text('$count', style: t(10, faint)),
        const SizedBox(width: 8),
        Text('⌕', style: t(14, quiet)),
      ]),
    );

final paperLibrary = <String, Widget Function(double, double)>{
  '0 Thumbnail mixed': (w, h) => mixed(w),
  '0b Thumbnail mixed, 36 HDR': (w, h) {
    stress = true;
    final r = mixed(w);
    stress = false;
    return r;
  },
  '1 Thumbnail': (w, h) => thumbnail(w),
  '2 Detail': (w, h) => detail(w),
  '3 Explore': explore,
};

class PaperLibrary extends StatelessWidget {
  const PaperLibrary(this.name, {super.key});
  final String name;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (_, c) => ColoredBox(color: ground, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [libraryHeader(library.length), Expanded(child: ClipRect(child: paperLibrary[name]!(c.maxWidth, c.maxHeight - 34)))])));
}
