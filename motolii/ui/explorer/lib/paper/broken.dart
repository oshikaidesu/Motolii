// Paper: twelve Media compositions, each breaking at least one assumption of the usual asset grid. Only the shelf's own
// information: the pictures, names, sizes (360° for a plate), the two kinds, the header. Position by colour uses each
// picture's own measured mean colour. Nothing is wired; nothing is kept.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'media.dart';

const _all = [...images, ...hdrs];

/// Each picture's own mean colour, measured from its pixels (hue 0–1, lightness 0–1).
const _colour = {
  'bund-at-night': (.67, .23), 'clouds': (.62, .57), 'coast': (1.0, .71), 'fire-sky': (.99, .73), 'meadow': (.36, .36),
  'studio-window': (.09, .28), 'studio': (.09, .16), 'tower': (.13, .32), 'trees': (.27, .24),
  'Photo studio': (.09, .44), 'Partly cloudy sky': (.62, .45), 'Meadow': (.41, .42), 'Night sky': (.04, .45),
  'City at night': (.02, .44), 'Sunset sky': (.89, .46),
};

/// Warm (0) to cool (1): how far the hue sits from orange.
double _cool(P p) {
  final h = _colour[p.name]!.$1;
  final d = math.min((h - .08).abs(), 1 - (h - .08).abs());
  return (d / .5).clamp(0.0, 1.0);
}

Widget _tiny(String s, [Color c = quiet]) => Text(s, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(9, c));

// A. Proof sheet — no gaps, no captions: frames touch in sheet order, the names are an index at the foot.
Widget proof(double w) {
  final rows = <List<P>>[];
  var r = <P>[];
  for (final p in _all) {
    r.add(p);
    if (w / r.fold(0.0, (a, p) => a + p.aspect) <= 74) {
      rows.add(r);
      r = [];
    }
  }
  if (r.isNotEmpty) rows.add(r);
  return ListView(children: [
    for (final row in rows)
      Builder(builder: (_) {
        final h = math.min(110.0, w / row.fold(0.0, (a, p) => a + p.aspect));
        return Row(children: [for (final p in row) SizedBox(width: h * p.aspect, height: h, child: p.pic())]);
      }),
    Padding(
      padding: const EdgeInsets.fromLTRB(10, 10, 10, 0),
      child: Text([for (final p in _all) p.name].join('   '), style: t(9.5, faint).copyWith(height: 1.6)),
    ),
  ]);
}

// B. Colour field — position means something: warm to cool across, light to dark down; pictures overlap where alike.
Widget field(double w) => LayoutBuilder(builder: (_, c) {
      final h = c.maxHeight - 30;
      return Stack(children: [
        Positioned(left: 10, top: 8, child: _tiny('warm', faint)),
        Positioned(right: 10, top: 8, child: _tiny('cool', faint)),
        Positioned(left: 10, bottom: 8, child: _tiny('dark', faint)),
        for (final p in [..._all]..sort((a, b) => _colour[a.name]!.$2.compareTo(_colour[b.name]!.$2)))
          Builder(builder: (_) {
            final tw = p.hdr ? 92.0 : 74.0, th = p.hdr ? 46.0 : 74 / p.aspect.clamp(.6, 1.8);
            final x = 8 + _cool(p) * (w - 16 - tw);
            final y = 26 + (1 - (_colour[p.name]!.$2 - .14) / .62) * (h - th - 20);
            return Positioned(left: x, top: y, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              SizedBox(width: tw, height: th, child: p.pic()),
              SizedBox(width: tw, child: _tiny(p.name)),
            ]));
          }),
      ]);
    });

// C. Fisheye — the chosen piece is not the same size: it opens at its own shape, neighbours shrink with distance.
Widget fisheye(double w) {
  const at = 3; // the chosen one (fire-sky)
  return ListView(padding: EdgeInsets.zero, children: [
    for (final (i, p) in _all.indexed)
      Builder(builder: (_) {
        final d = (i - at).abs();
        final h = d == 0 ? math.min(300.0, w / p.aspect) : math.max(18.0, 96.0 * math.pow(.72, d - 1));
        return SizedBox(
          height: h,
          child: Row(children: [
            SizedBox(width: d == 0 ? math.min(w, h * p.aspect) : w * .62, height: h, child: p.pic()),
            Expanded(child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(p.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(d == 0 ? 12 : 10, d == 0 ? ink : quiet, d == 0 ? FontWeight.w600 : FontWeight.w400)),
                if (d <= 1) _tiny(p.fact, faint),
              ]),
            )),
          ]),
        );
      }),
  ]);
}

// D. Rows are shapes — no category headings: tall, square, wide and 360° each a band that runs off to the side.
Widget shapes(double w) {
  final bands = <String, List<P>>{'tall': [], 'square': [], 'wide': [], '360°': []};
  for (final p in _all) {
    bands[p.hdr ? '360°' : p.aspect < .8 ? 'tall' : p.aspect < 1.25 ? 'square' : 'wide']!.add(p);
  }
  const heights = {'tall': 170.0, 'square': 110.0, 'wide': 96.0, '360°': 80.0};
  return ListView(padding: const EdgeInsets.only(top: 6), children: [
    for (final e in bands.entries)
      Padding(
        padding: const EdgeInsets.only(bottom: 10),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          SizedBox(width: 30, child: Padding(padding: const EdgeInsets.only(left: 8, top: 2), child: RotatedBox(quarterTurns: 3, child: _tiny(e.key, faint)))),
          Expanded(child: SizedBox(
            height: heights[e.key]! + 16,
            child: ListView(scrollDirection: Axis.horizontal, children: [
              for (final p in e.value)
                Padding(padding: const EdgeInsets.only(right: 4), child: SizedBox(width: heights[e.key]! * p.aspect, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(height: heights[e.key]!, width: heights[e.key]! * p.aspect, child: p.pic()), _tiny(p.name)]))),
            ]),
          )),
        ]),
      ),
  ]);
}

// E. Each kind its own form — stills a gapless mosaic, 360° plates stacked edge to edge as horizons with the name on them.
Widget kinds(double w) {
  final rows = <List<P>>[];
  var r = <P>[];
  for (final p in images) {
    r.add(p);
    if (w / r.fold(0.0, (a, p) => a + p.aspect) <= 90) {
      rows.add(r);
      r = [];
    }
  }
  if (r.isNotEmpty) rows.add(r);
  return ListView(children: [
    for (final row in rows)
      Builder(builder: (_) {
        final h = w / row.fold(0.0, (a, p) => a + p.aspect);
        return Row(children: [for (final p in row) SizedBox(width: h * p.aspect, height: math.min(h, 150), child: p.pic())]);
      }),
    const SizedBox(height: 14),
    for (final p in hdrs)
      SizedBox(height: 58, child: Stack(fit: StackFit.expand, children: [
        p.pic(),
        Positioned(left: 10, bottom: 6, child: Text(p.name, style: t(11, const Color(0xFFFFFFFF), FontWeight.w600).copyWith(shadows: const [Shadow(blurRadius: 6, color: Color(0xAA000000))]))),
      ])),
  ]);
}

// F. Real size — a board wider and taller than the seat (it pans, it does not scroll a list); each still at its real
// pixel size, pinned where it lands; the plates lie along the bottom as one horizon.
Widget board(double w) {
  const k = .26;
  const spots = [Offset(8, 10), Offset(222, 30), Offset(30, 140), Offset(250, 150), Offset(140, 170), Offset(8, 310), Offset(200, 350), Offset(110, 330), Offset(300, 330)];
  return ClipRect(child: OverflowBox(
    alignment: Alignment.topLeft,
    maxWidth: 520,
    maxHeight: 900,
    child: SizedBox(width: 520, height: 900, child: Stack(children: [
      for (final (i, p) in images.indexed)
        Positioned(left: spots[i].dx, top: spots[i].dy, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          SizedBox(width: p.w * k, height: p.h * k, child: p.pic()),
          _tiny('${p.name}  ${p.fact}'),
        ])),
      Positioned(left: 0, top: 560, child: Row(children: [for (final p in hdrs) SizedBox(width: 150, height: 75, child: p.pic())])),
      Positioned(left: 8, top: 640, child: _tiny([for (final p in hdrs) p.name].join('  ·  '))),
    ])),
  ));
}

// G. No scroll — the whole seat divided among every piece at once, a still's share by its pixel count, the plates
// sharing the bottom band.
Widget treemap(double w) => LayoutBuilder(builder: (_, c) {
      final h = c.maxHeight;
      final out = <Widget>[];
      void split(List<P> ps, Rect r, bool across) {
        if (ps.length == 1) {
          out.add(Positioned.fromRect(rect: r.deflate(.5), child: Stack(fit: StackFit.expand, children: [
            ps.single.pic(),
            Positioned(left: 4, bottom: 3, right: 4, child: Text(ps.single.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: t(9.5, const Color(0xFFFFFFFF), FontWeight.w500).copyWith(shadows: const [Shadow(blurRadius: 4, color: Color(0xCC000000))]))),
          ])));
          return;
        }
        double area(P p) => p.hdr ? 1 : p.w * p.h / 1e5;
        final total = ps.fold(0.0, (a, p) => a + area(p));
        var acc = 0.0, cut = 1;
        for (var i = 0; i < ps.length - 1; i++) {
          acc += area(ps[i]);
          cut = i + 1;
          if (acc >= total / 2) break;
        }
        final f = ps.take(cut).fold(0.0, (a, p) => a + area(p)) / total;
        final a = across ? Rect.fromLTWH(r.left, r.top, r.width * f, r.height) : Rect.fromLTWH(r.left, r.top, r.width, r.height * f);
        final b = across ? Rect.fromLTRB(a.right, r.top, r.right, r.bottom) : Rect.fromLTRB(r.left, a.bottom, r.right, r.bottom);
        split(ps.sublist(0, cut), a, !across);
        split(ps.sublist(cut), b, !across);
      }

      split([...images]..sort((a, b) => (b.w * b.h).compareTo(a.w * a.h)), Rect.fromLTWH(0, 0, w, h * .7), false);
      split(hdrs, Rect.fromLTWH(0, h * .7, w, h * .3), true);
      return Stack(children: out);
    });

// H. Strip and loupe — every piece as a touching frame in a narrow strip; the rest of the seat is the chosen one.
Widget loupe(double w) {
  const chosen = 3;
  return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
    SizedBox(width: 58, child: ListView(padding: EdgeInsets.zero, children: [
      for (final (i, p) in _all.indexed)
        Container(
          height: p.hdr ? 29 : 44,
          foregroundDecoration: i == chosen ? BoxDecoration(border: Border.all(color: ink, width: 2)) : null,
          child: p.pic(),
        ),
    ])),
    Expanded(child: Padding(
      padding: const EdgeInsets.all(10),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Expanded(child: Align(alignment: Alignment.topLeft, child: _all[chosen].pic(fit: BoxFit.contain))),
        const SizedBox(height: 8),
        Text(_all[chosen].name, style: t(13, ink, FontWeight.w600)),
        const SizedBox(height: 2),
        Text('Images  ${_all[chosen].fact}', style: t(10.5, quiet)),
      ]),
    )),
  ]);
}

// I. Slit rows — picture and name are one thing: each row is a band cut through the picture, the name set on it.
Widget slits(double w) => ListView(children: [
      for (final (i, p) in _all.indexed) ...[
        if (i == images.length) const SizedBox(height: 8),
        SizedBox(height: 34, child: Stack(fit: StackFit.expand, children: [
          p.pic(),
          const DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(colors: [Color(0xCC000000), Color(0x00000000)], stops: [0, .7]))),
          Padding(padding: const EdgeInsets.symmetric(horizontal: 10), child: Row(children: [
            Expanded(child: Text(p.name, style: t(11.5, const Color(0xFFFFFFFF), FontWeight.w500))),
            Text(p.fact, style: t(9.5, const Color(0xCCFFFFFF))),
          ])),
        ])),
      ],
    ]);

// J. Deck — each kind a cascade of full-width prints, each showing its top edge and name; the last lies open.
Widget deck(double w) {
  Widget cascade(List<P> ps) {
    const peek = 30.0;
    final last = ps.last;
    final lastH = w / last.aspect;
    return SizedBox(
      height: peek * (ps.length - 1) + math.min(lastH, 260),
      child: Stack(children: [
        for (final (i, p) in ps.indexed)
          Positioned(top: peek * i, left: 0, right: 0, height: i == ps.length - 1 ? math.min(lastH, 260) : w / p.aspect, child: Container(
            decoration: const BoxDecoration(boxShadow: [BoxShadow(color: Color(0x99000000), blurRadius: 8, offset: Offset(0, -2))]),
            child: Stack(fit: StackFit.expand, children: [
              p.pic(),
              Positioned(left: 8, top: 7, child: Text(p.name, style: t(10.5, const Color(0xFFFFFFFF), FontWeight.w600).copyWith(shadows: const [Shadow(blurRadius: 4, color: Color(0xCC000000))]))),
            ]),
          )),
      ]),
    );
  }

  return ListView(children: [cascade(images), const SizedBox(height: 16), cascade(hdrs)]);
}

// K. Two scrolls side by side — the kinds are not tabs or headings but two columns that move on their own.
Widget twin(double w) => Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Expanded(flex: 62, child: ListView(padding: const EdgeInsets.fromLTRB(6, 6, 3, 6), children: [
        for (final p in images) Padding(padding: const EdgeInsets.only(bottom: 3), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [AspectRatio(aspectRatio: p.aspect, child: p.pic()), _tiny(p.name)])),
      ])),
      Container(width: .5, color: rule),
      Expanded(flex: 38, child: ListView(padding: const EdgeInsets.fromLTRB(3, 6, 6, 6), children: [
        for (final p in hdrs) Padding(padding: const EdgeInsets.only(bottom: 3), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [AspectRatio(aspectRatio: 2, child: p.pic()), _tiny(p.name)])),
      ])),
    ]);

// L. Sideways columns — the shelf flows like a newspaper to the right; the seat shows a column and a half.
Widget sideways(double w) => ListView(scrollDirection: Axis.horizontal, padding: const EdgeInsets.all(8), children: [
      for (final col in [images.sublist(0, 4), images.sublist(4, 8), [images[8], ...hdrs.take(3)], hdrs.sublist(3)])
        Padding(padding: const EdgeInsets.only(right: 8), child: SizedBox(width: 196, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          for (final p in col)
            Padding(padding: const EdgeInsets.only(bottom: 6), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              SizedBox(width: 196, height: math.min(196 / p.aspect, 200), child: p.pic()),
              const SizedBox(height: 2),
              _tiny('${p.name}  ${p.fact}'),
            ])),
        ]))),
    ]);

/// Rows of whole frames touching, filling [w] (every picture uncut, at its own shape), each row's names printed under
/// its frames like a film's edge print.
List<Widget> _rebate(List<P> ps, double w, double target, {double most = 160, bool names = true}) {
  final rows = <List<P>>[];
  var r = <P>[];
  for (final p in ps) {
    r.add(p);
    if (w / r.fold(0.0, (a, p) => a + p.aspect) <= target) {
      rows.add(r);
      r = [];
    }
  }
  if (r.isNotEmpty) rows.add(r);
  return [
    for (final row in rows)
      Builder(builder: (_) {
        final h = math.min(most, w / row.fold(0.0, (a, p) => a + p.aspect));
        return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [for (final p in row) SizedBox(width: h * p.aspect, height: h, child: p.pic())]),
          if (names)
            Container(
              height: 15,
              color: const Color(0xFF0E0E0E),
              child: Row(children: [
                for (final p in row) SizedBox(width: h * p.aspect, child: Padding(padding: const EdgeInsets.only(left: 4, top: 2), child: _tiny(p.name))),
              ]),
            ),
        ]);
      }),
  ];
}

/// The row height at which [ps] fills [height] exactly (no scroll), found by halving.
double _fit(List<P> ps, double w, double height, {double band = 15}) {
  double total(double target) {
    var sum = 0.0;
    var r = <P>[];
    for (final p in ps) {
      r.add(p);
      if (w / r.fold(0.0, (a, p) => a + p.aspect) <= target) {
        sum += w / r.fold(0.0, (a, p) => a + p.aspect) + band;
        r = [];
      }
    }
    if (r.isNotEmpty) sum += math.min(160, w / r.fold(0.0, (a, p) => a + p.aspect)) + band;
    return sum;
  }

  var lo = 20.0, hi = 400.0;
  for (var i = 0; i < 30; i++) {
    final mid = (lo + hi) / 2;
    if (total(mid) > height) {
      hi = mid;
    } else {
      lo = mid;
    }
  }
  return lo;
}

// A2. Proof sheet: every piece whole, touching, in library order; each row's names on its edge.
Widget proof2(double w) => ListView(children: _rebate(_all, w, 92));

// E2. Each kind its form: stills a touching sheet, the 360° plates whole at 2:1 two across; names on the edges.
Widget kinds2(double w) => ListView(children: [
      ..._rebate(images, w, 100),
      const SizedBox(height: 10),
      ..._rebate(hdrs, w, w / 4 - 1, most: 200),
    ]);

// G2. Never scrolls: the row height is solved so the whole library, every piece whole, fills the seat exactly.
Widget fitAll(double w) => LayoutBuilder(builder: (_, c) => Column(children: _rebate(_all, w, _fit(_all, w, c.maxHeight - 2), most: 400)));

// B2. Ordered by its own colour: the same whole, touching sheet, but the order runs warm to cool and dark to light, so
// place says what a picture looks like.
Widget byColour(double w) {
  final sorted = [..._all]..sort((a, b) {
      final c = (_cool(a) * 3).round().compareTo((_cool(b) * 3).round());
      return c != 0 ? c : _colour[a.name]!.$2.compareTo(_colour[b.name]!.$2);
    });
  return ListView(children: _rebate(sorted, w, 92));
}

final paperBroken = <String, Widget Function(double)>{
  'A2 proof sheet with edge print': proof2,
  'E2 each kind its form, edge print': kinds2,
  'G2 fits the seat, no scroll': fitAll,
  'B2 ordered by its own colour': byColour,
  'A proof sheet': proof,
  'B colour field': field,
  'C fisheye': fisheye,
  'D rows are shapes': shapes,
  'E each kind its form': kinds,
  'F real size board': board,
  'G no scroll treemap': treemap,
  'H strip and loupe': loupe,
  'I slit rows': slits,
  'J deck': deck,
  'K two scrolls': twin,
  'L sideways columns': sideways,
};

class PaperBroken extends StatelessWidget {
  const PaperBroken(this.name, {super.key});
  final String name;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (_, c) => sheet(paperBroken[name]!(c.maxWidth)));
}
