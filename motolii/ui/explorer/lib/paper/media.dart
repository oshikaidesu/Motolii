// Paper, not product: six compositions of the Media shelf's own information (its pictures, names, sizes, the two
// categories, the header's classes, Import, count, search), drawn freehand to find the form. Nothing here is wired;
// nothing here is kept once a form is chosen.
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../story.dart' show fixture;

class P {
  const P(this.name, this.file, this.w, this.h, {this.hdr = false});
  final String name, file;
  final int w, h;
  final bool hdr;
  double get aspect => w / h;
  String get fact => hdr ? '360°' : '$w×$h';
  Widget pic({BoxFit fit = BoxFit.cover}) => Image.file(File(fixture(hdr ? 'paper/$file.jpg' : 'media/$file.jpg')), fit: fit, cacheWidth: 640);
}

const images = [
  P('bund-at-night', 'bund-at-night', 800, 450),
  P('clouds', 'clouds', 560, 520),
  P('coast', 'coast', 760, 420),
  P('fire-sky', 'fire-sky', 420, 760),
  P('meadow', 'meadow', 520, 520),
  P('studio-window', 'studio-window', 420, 620),
  P('studio', 'studio', 700, 440),
  P('tower', 'tower', 360, 760),
  P('trees', 'trees', 400, 700),
];
const hdrs = [
  P('Partly cloudy sky', 'kloofendal_48d_partly_cloudy_puresky', 2, 1, hdr: true),
  P('Sunset sky', 'the_sky_is_on_fire', 2, 1, hdr: true),
  P('Night sky', 'moonless_golf', 2, 1, hdr: true),
  P('Meadow', 'meadow_2', 2, 1, hdr: true),
  P('City at night', 'shanghai_bund', 2, 1, hdr: true),
  P('Photo studio', 'brown_photostudio_02', 2, 1, hdr: true),
];
const sections = {'Images': images, 'HDR': hdrs};

const ground = Color(0xFF191919), ink = Color(0xFFE7E7E7), quiet = Color(0xFF8E8E8E), faint = Color(0xFF616161), rule = Color(0xFF2A2A2A);
TextStyle t(double s, [Color c = ink, FontWeight w = FontWeight.w400]) => TextStyle(fontSize: s, color: c, fontWeight: w, height: 1.2, decoration: TextDecoration.none);

/// The header every sheet shares, so only the body differs.
Widget header() => Container(
      height: 34,
      padding: const EdgeInsets.symmetric(horizontal: 10),
      decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: rule))),
      child: Row(children: [
        Text('All', style: t(11.5, ink, FontWeight.w600)),
        const SizedBox(width: 12),
        Text('Images', style: t(11.5, quiet)),
        const SizedBox(width: 12),
        Text('HDR', style: t(11.5, quiet)),
        const Spacer(),
        Text('+ Import', style: t(11.5, quiet)),
        const SizedBox(width: 12),
        Text('15', style: t(10.5, faint)),
        const SizedBox(width: 10),
        Text('⌕', style: t(14, quiet)),
      ]),
    );

Widget sheet(Widget body) => ColoredBox(color: ground, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [header(), Expanded(child: ClipRect(child: body))]));

Widget label(String s, int n, {double top = 14}) => Padding(
      padding: EdgeInsets.fromLTRB(0, top, 0, 6),
      child: Text.rich(TextSpan(children: [TextSpan(text: s, style: t(11, ink, FontWeight.w600)), TextSpan(text: '  $n', style: t(10.5, faint))])),
    );

Widget caption(P p, {bool fact = true, double size = 10}) => Row(children: [
      Expanded(child: Text(p.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(size, const Color(0xFFD0D0D0)))),
      if (fact) Text(p.fact, style: t(size - 1, faint)),
    ]);

// 1. Regular grid: every piece the same square, name under. The shelf reads as a catalogue.
Widget grid(double w) => ListView(padding: const EdgeInsets.symmetric(horizontal: 10), children: [
      for (final e in sections.entries) ...[
        label(e.key, e.value.length),
        Wrap(spacing: 6, runSpacing: 8, children: [
          for (final p in e.value)
            SizedBox(width: (w - 20 - 12) / 3, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              AspectRatio(aspectRatio: 1, child: p.pic()),
              const SizedBox(height: 4),
              caption(p, fact: false),
            ])),
        ]),
      ],
    ]);

// 2. Justified rows: every row the same height, each picture its own width by its shape, name under in small type.
Widget justified(double w) {
  List<Widget> rows(List<P> ps, double target) {
    final out = <Widget>[];
    var row = <P>[];
    const gap = 4.0;
    final avail = w - 20;
    void flush(bool last) {
      if (row.isEmpty) return;
      final sum = row.fold(0.0, (a, p) => a + p.aspect);
      final h = last ? math.min(target, (avail - gap * (row.length - 1)) / sum) : (avail - gap * (row.length - 1)) / sum;
      final r = row;
      out.add(Padding(padding: const EdgeInsets.only(bottom: 8), child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        for (final (i, p) in r.indexed) ...[
          if (i > 0) const SizedBox(width: gap),
          SizedBox(width: h * p.aspect, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(height: h, width: h * p.aspect, child: p.pic()), const SizedBox(height: 3), caption(p, fact: false, size: 9.5)])),
        ],
      ])));
      row = [];
    }
    for (final p in ps) {
      row.add(p);
      final sum = row.fold(0.0, (a, p) => a + p.aspect);
      if ((avail - 4 * (row.length - 1)) / sum <= target) flush(false);
    }
    flush(true);
    return out;
  }

  return ListView(padding: const EdgeInsets.symmetric(horizontal: 10), children: [
    for (final e in sections.entries) ...[label(e.key, e.value.length), ...rows(e.value, e.key == 'HDR' ? 70 : 96)],
  ]);
}

// 3. Category strips: each category one row of larger pictures running past the edge (it scrolls sideways).
Widget strips(double w) => ListView(children: [
      for (final e in sections.entries) ...[
        Padding(padding: const EdgeInsets.symmetric(horizontal: 10), child: label(e.key, e.value.length, top: 16)),
        SizedBox(
          height: 150,
          child: ListView(scrollDirection: Axis.horizontal, padding: const EdgeInsets.symmetric(horizontal: 10), children: [
            for (final p in e.value)
              Padding(
                padding: const EdgeInsets.only(right: 8),
                child: SizedBox(width: 118 * math.min(p.aspect, 1.6), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  SizedBox(height: 118, width: double.infinity, child: p.pic()),
                  const SizedBox(height: 5),
                  caption(p),
                ])),
              ),
          ]),
        ),
      ],
    ]);

// 4. One large, the rest small: the chosen piece at the top at its own shape with its facts, everything under it as
// a tight index of small pictures.
Widget dominant(double w) {
  const chosen = P('fire-sky', 'fire-sky', 420, 760);
  final all = [...images, ...hdrs];
  return ListView(padding: const EdgeInsets.fromLTRB(10, 10, 10, 10), children: [
    Row(crossAxisAlignment: CrossAxisAlignment.end, children: [
      SizedBox(height: 210, width: 210 * chosen.aspect, child: chosen.pic()),
      const SizedBox(width: 10),
      Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text(chosen.name, style: t(12, ink, FontWeight.w600)),
        const SizedBox(height: 3),
        Text('Images · ${chosen.fact}', style: t(10, quiet)),
      ])),
    ]),
    for (final e in sections.entries) ...[
      label(e.key, e.value.length),
      Wrap(spacing: 3, runSpacing: 3, children: [
        for (final p in e.value) SizedBox(width: (w - 20 - 15) / 6, height: (w - 20 - 15) / 6 / (e.key == 'HDR' ? 2 : 1), child: p.pic()),
      ]),
    ],
    const SizedBox(height: 4),
    Text('${all.length} pieces', style: t(9.5, faint)),
  ]);
}

// 5. List with pictures: a row per piece, picture at a fixed height by its shape, name and size beside it.
Widget list(double w) => ListView(padding: const EdgeInsets.symmetric(horizontal: 10), children: [
      for (final e in sections.entries) ...[
        label(e.key, e.value.length),
        for (final p in e.value)
          Container(
            height: 44,
            decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: rule, width: .5))),
            child: Row(children: [
              SizedBox(width: 64, height: 36, child: p.pic()),
              const SizedBox(width: 10),
              Expanded(child: Text(p.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(11))),
              Text(p.fact, style: t(10, faint)),
            ]),
          ),
      ],
    ]);

// 6. Laid out by shape: a landscape piece takes the whole width, portraits pair up side by side.
Widget byShape(double w) {
  const gap = 6.0;
  final avail = w - 20;
  List<Widget> lay(List<P> ps) {
    final out = <Widget>[];
    final tall = <P>[];
    Widget one(P p, double width) => SizedBox(width: width, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(width: width, height: width / p.aspect, child: p.pic()), const SizedBox(height: 4), caption(p)]));
    void pair() {
      if (tall.isEmpty) return;
      final ws = tall.length == 1 ? [avail * .5] : [for (final _ in tall) (avail - gap) / 2];
      out.add(Padding(padding: const EdgeInsets.only(bottom: 10), child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [for (final (i, p) in tall.indexed) ...[if (i > 0) const SizedBox(width: gap), one(p, ws[i])]])));
      tall.clear();
    }
    for (final p in ps) {
      if (p.aspect > 1.3) {
        out.add(Padding(padding: const EdgeInsets.only(bottom: 10), child: one(p, avail)));
      } else {
        tall.add(p);
        if (tall.length == 2) pair();
      }
    }
    pair();
    return out;
  }

  return ListView(padding: const EdgeInsets.symmetric(horizontal: 10), children: [
    for (final e in sections.entries) ...[label(e.key, e.value.length), ...lay(e.value)],
  ]);
}


// 1b. The grid, each category at its own shape: stills square three across, 360° plates 2:1 two across.
Widget gridB(double w) => ListView(padding: const EdgeInsets.symmetric(horizontal: 10), children: [
      for (final e in sections.entries) ...[
        label(e.key, e.value.length),
        Builder(builder: (_) {
          final hdr = e.key == 'HDR', cols = hdr ? 2 : 3;
          final cell = (w - 20 - 6 * (cols - 1)) / cols;
          return Wrap(spacing: 6, runSpacing: 10, children: [
            for (final p in e.value)
              SizedBox(width: cell, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                SizedBox(width: cell, height: hdr ? cell / 2 : cell, child: p.pic()),
                const SizedBox(height: 4),
                caption(p, fact: false),
              ])),
          ]);
        }),
      ],
    ]);

// 2b. Justified rows, larger: stills about two to a row, plates two to a row, every row filled edge to edge.
Widget justifiedB(double w) {
  const gap = 5.0;
  final avail = w - 20;
  Widget row(List<P> r) {
    final sum = r.fold(0.0, (a, p) => a + p.aspect);
    final h = math.min(170.0, (avail - gap * (r.length - 1)) / sum);
    return Padding(padding: const EdgeInsets.only(bottom: 10), child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      for (final (i, p) in r.indexed) ...[
        if (i > 0) const SizedBox(width: gap),
        SizedBox(width: h * p.aspect, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(height: h, width: h * p.aspect, child: p.pic()), const SizedBox(height: 4), caption(p, fact: false)])),
      ],
    ]));
  }

  List<Widget> rows(List<P> ps, double target) {
    final out = <Widget>[];
    var r = <P>[];
    for (final p in ps) {
      r.add(p);
      if ((avail - gap * (r.length - 1)) / r.fold(0.0, (a, p) => a + p.aspect) <= target) {
        out.add(row(r));
        r = [];
      }
    }
    if (r.isNotEmpty) out.add(row(r));
    return out;
  }

  return ListView(padding: const EdgeInsets.symmetric(horizontal: 10), children: [
    for (final e in sections.entries) ...[label(e.key, e.value.length), ...rows(e.value, e.key == 'HDR' ? 76 : 130)],
  ]);
}

// 5b. The list, with a picture large enough to recognise and the name over its size.
Widget listB(double w) => ListView(padding: const EdgeInsets.symmetric(horizontal: 10), children: [
      for (final e in sections.entries) ...[
        label(e.key, e.value.length),
        for (final p in e.value)
          Padding(
            padding: const EdgeInsets.only(bottom: 6),
            child: Row(children: [
              SizedBox(width: 96, height: 54, child: p.pic()),
              const SizedBox(width: 10),
              Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(p.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(11.5)),
                const SizedBox(height: 3),
                Text(p.fact, style: t(10, faint)),
              ])),
            ]),
          ),
      ],
    ]);

final paperMedia = <String, Widget Function(double)>{
  '1 grid': grid,
  '2 justified rows': justified,
  '3 category strips': strips,
  '4 one large, rest small': dominant,
  '5 list with pictures': list,
  '6 laid out by shape': byShape,
  '1b grid by shape of category': gridB,
  '2b justified rows larger': justifiedB,
  '5b list larger pictures': listB,
};

class PaperMedia extends StatelessWidget {
  const PaperMedia(this.name, {super.key});
  final String name;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (_, c) => sheet(paperMedia[name]!(c.maxWidth)));
}
