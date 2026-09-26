import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'classify.dart';
import 'common.dart';
import 'search.dart';
import 'shell.dart';

class FontItem {
  const FontItem(this.family, this.cls, this.styles, {this.sample = 'Aa', this.label});
  final String family, cls, sample;
  final int styles;
  final String? label;
  String get name => label ?? family;
  String get meta => '$cls  ·  $styles styles';
}

const fontsBase = <FontItem>[
  FontItem('Helvetica Neue', 'Sans', 18), FontItem('Avenir Next', 'Sans', 12), FontItem('Optima', 'Sans', 6), FontItem('Gill Sans', 'Sans', 8),
  FontItem('Georgia', 'Serif', 4), FontItem('Palatino', 'Serif', 4), FontItem('Baskerville', 'Serif', 6), FontItem('Didot', 'Serif', 3),
  FontItem('Futura', 'Display', 5), FontItem('Impact', 'Display', 1), FontItem('Copperplate', 'Display', 2),
  FontItem('Menlo', 'Mono', 4), FontItem('Courier New', 'Mono', 4),
  FontItem('Bradley Hand', 'Hand', 2), FontItem('Snell Roundhand', 'Hand', 3),
  FontItem('Hiragino Sans', 'JP', 9, sample: 'あ'), FontItem('Hiragino Mincho ProN', 'JP', 2, sample: '永'),
  FontItem('Apple SD Gothic Neo', 'KR', 9, sample: '가'),
];

/// Growth fixture: two thousand typefaces. Names are set in real installed faces, cycled.
List<FontItem> fontsStress() {
  final out = <FontItem>[...fontsBase];
  for (var i = 0; i < 2000; i++) {
    final b = fontsBase[i % fontsBase.length];
    out.add(FontItem(b.family, b.cls, 1 + (i * 3) % 14, sample: b.sample, label: '${b.family} ${String.fromCharCode(65 + i % 26)}${1 + i ~/ 26}'));
  }
  return out;
}

List<List<String>> fontGroups() => const [['All', 'Sans', 'Serif', 'Display', 'Mono', 'Hand', 'JP', 'KR'], ['Favorites', 'Installed']];

class FontsPanel extends StatefulWidget {
  const FontsPanel({super.key, this.search, this.classify, this.items = fontsBase});
  final SearchCapability? search;
  final ClassifyCapability? classify;
  final List<FontItem> items;
  @override
  State<FontsPanel> createState() => _FontsPanelState();
}

class _FontsPanelState extends State<FontsPanel> with WithDiscovery<FontsPanel> {
  @override
  SearchCapability? get injectedSearch => widget.search;
  @override
  ClassifyCapability? get injectedClassify => widget.classify;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: discovery,
        builder: (_, __) {
          final shown = search.apply(classify.apply(widget.items, (f) => f.cls), (f) => [f.name, f.cls]);
          final n = shown.length;
          return PanelShell(
            title: 'Fonts',
            icon: const GlyphBox(HG.text, size: 22, color: Color(0xFFF2F2F4)),
            search: search,
            classify: classify,
            groups: fontGroups(),
            hint: 'Search fonts',
            count: n >= 1000 ? '${n ~/ 1000},${(n % 1000).toString().padLeft(3, '0')}' : '$n',
            wide: (c, s) => shown.isEmpty ? emptyBody('No typeface matches "${search.query}".') : _list(shown, s.width, 66),
            narrow: (c, s) => shown.isEmpty ? emptyBody('No typeface matches.') : _list(shown, s.width, s.width >= 110 ? 46 : 42),
            strip: (c, s) => ListView.builder(
              scrollDirection: Axis.horizontal,
              physics: const ClampingScrollPhysics(),
              padding: const EdgeInsets.all(10),
              itemCount: shown.length,
              itemBuilder: (_, i) => Padding(
                padding: const EdgeInsets.only(right: 6),
                child: Container(width: math.min(72, s.height * 1.3), decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(4)), child: Center(child: Text(shown[i].sample, softWrap: false, style: TextStyle(fontFamily: shown[i].family, fontSize: 24, color: const Color(0xFFF2F2F4), height: 1)))),
              ),
            ),
          );
        },
      );

  Widget _list(List<FontItem> shown, double w, double h) => ListView.builder(
        physics: const ClampingScrollPhysics(),
        itemExtent: h,
        itemCount: shown.length,
        itemBuilder: (_, i) => _Row(shown[i], w, h, i == 0 && !search.active && !classify.filtering),
      );
}

class _Row extends StatelessWidget {
  const _Row(this.f, this.w, this.h, this.chosen);
  final FontItem f;
  final double w, h;
  final bool chosen;
  @override
  Widget build(BuildContext context) {
    final big = h >= 60;
    return Container(
      decoration: BoxDecoration(
        color: chosen ? kRaisedHi : null,
        border: Border(left: BorderSide(color: chosen ? const Color(0xFFE8E8EA) : const Color(0x00000000), width: 2), bottom: const BorderSide(color: kRule2)),
      ),
      padding: const EdgeInsets.only(left: 10, right: 8),
      child: w < 110
          ? Center(child: Text(f.sample, softWrap: false, style: TextStyle(fontFamily: f.family, fontSize: 22, color: const Color(0xFFF2F2F4), height: 1)))
          : Row(children: [
              SizedBox(width: big ? 62 : 40, child: Text(f.sample, softWrap: false, style: TextStyle(fontFamily: f.family, fontSize: big ? 36 : 24, color: const Color(0xFFF2F2F4), height: 1))),
              Expanded(
                child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Text(f.name, softWrap: false, overflow: TextOverflow.clip, style: TextStyle(fontFamily: f.family, fontSize: big ? 14.5 : 13, color: const Color(0xFFE6E6E8), height: 1.1)),
                  if (big) ...[const SizedBox(height: 5), Text(f.meta, softWrap: false, overflow: TextOverflow.clip, style: sans(10, c: kMuted))],
                ]),
              ),
              if (big) SizedBox(width: 16, height: 16, child: CustomPaint(painter: _Star(chosen))),
            ]),
    );
  }
}

class _Star extends CustomPainter {
  _Star(this.on);
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    final p = Path();
    for (var i = 0; i < 10; i++) {
      final r = i.isEven ? 7.2 : 3.2;
      final a = -1.5708 + i * 0.6283;
      final q = Offset(8 + r * math.cos(a), 8.4 + r * math.sin(a));
      i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    p.close();
    c.drawPath(p, on ? (Paint()..color = const Color(0xFFE8E8EA)) : (Paint()..color = kMuted..style = PaintingStyle.stroke..strokeWidth = 1.2));
  }
  @override
  bool shouldRepaint(_Star o) => o.on != on;
}
