import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../hf/glyphs.dart';
import '../browser/classify.dart';
import '../browser/parts.dart';
import '../browser/search.dart';
import '../browser/panel_chrome.dart';
import '../browser/things.dart' show UserViews;
import '../session/editor_session.dart';
import '../browser/visual_sample.dart';
import '../hf/neutral.dart';
import '../hf/metrics.dart' show Dn, Surface;

class FontItem {
  const FontItem(
    this.family,
    this.cls,
    this.styles, {
    this.sample = 'Aa',
    this.label,
    this.facts = const [],
    this.favorite = false,
  });
  final String family, cls, sample;
  final int styles;
  final String? label;
  final List<String> facts;
  final bool favorite;
  String get name => label ?? family;
  String get meta => '$cls  ·  $styles styles';
}

const fontsBase = <FontItem>[
  FontItem('Helvetica Neue', 'Sans', 18),
  FontItem('Avenir Next', 'Sans', 12),
  FontItem('Optima', 'Sans', 6),
  FontItem('Gill Sans', 'Sans', 8),
  FontItem('Georgia', 'Serif', 4),
  FontItem('Palatino', 'Serif', 4),
  FontItem('Baskerville', 'Serif', 6),
  FontItem('Didot', 'Serif', 3),
  FontItem('Futura', 'Display', 5),
  FontItem('Impact', 'Display', 1),
  FontItem('Copperplate', 'Display', 2),
  FontItem('Menlo', 'Mono', 4),
  FontItem('Courier New', 'Mono', 4),
  FontItem('Bradley Hand', 'Hand', 2),
  FontItem('Snell Roundhand', 'Hand', 3),
  FontItem('Hiragino Sans', 'JP', 9, sample: 'あ'),
  FontItem('Hiragino Mincho ProN', 'JP', 2, sample: '永'),
  FontItem('Apple SD Gothic Neo', 'KR', 9, sample: '가'),
];

/// Growth fixture: two thousand typefaces. Names are set in real installed faces, cycled.
List<FontItem> fontsStress() {
  final out = <FontItem>[...fontsBase];
  for (var i = 0; i < 2000; i++) {
    final b = fontsBase[i % fontsBase.length];
    out.add(
      FontItem(
        b.family,
        b.cls,
        1 + (i * 3) % 14,
        sample: b.sample,
        label: '${b.family} ${String.fromCharCode(65 + i % 26)}${1 + i ~/ 26}',
      ),
    );
  }
  return out;
}

List<List<String>> fontGroups() => const [
  ['All', 'Sans', 'Serif', 'Display', 'Mono', 'Hand', 'JP', 'KR'],
  ['Favorites', 'Installed'],
];

class FontsPanel extends StatefulWidget {
  const FontsPanel({
    super.key,
    this.search,
    this.classify,
    this.items = fontsBase,
    this.groups,
    this.user,
    this.selectedFamily,
    this.onSelect,
    this.onCreate,
    this.onFavorite,
    this.controller,
    this.sampleLayer,
    this.used = const {},
  });
  final SearchCapability? search;
  final ClassifyCapability? classify;
  final List<FontItem> items;
  final List<List<String>>? groups;
  final UserViews? user;
  final String? selectedFamily;
  final ValueChanged<FontItem>? onSelect, onCreate, onFavorite;
  final EditorSession? controller;
  final int? sampleLayer;
  /// The families a text layer of the document uses (the Used Here view).
  final Set<String> used;
  @override
  State<FontsPanel> createState() => _FontsPanelState();
}

class _FontsPanelState extends State<FontsPanel>
    with WithDiscovery<FontsPanel> {
  @override
  SearchCapability? get injectedSearch => widget.search;
  @override
  ClassifyCapability? get injectedClassify => widget.classify;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: discovery,
    builder: (_, __) {
      final view = classify.selected;
      final pool = view == 'Favorites' && widget.user != null
          ? widget.items
                .where(
                  (f) => widget.user!.favorites.contains('font:${f.family}'),
                )
                .toList()
          : view == 'Installed'
          ? widget.items
          : view == 'Used Here'
          ? widget.items.where((f) => widget.used.contains(f.family)).toList()
          : classify.apply(widget.items, (f) => f.cls);
      // the system's own private faces (their names start with '.') stay out of the list until a search asks for them
      final private = search.query.trimLeft().startsWith('.');
      final shown = search.apply(private ? pool : [for (final f in pool) if (!f.family.startsWith('.')) f], (f) => [f.name, f.cls, ...f.facts]);
      final n = shown.length;
      return PanelShell(
            // classes along the top, as Create and Media: the body keeps the seat's whole width
            classStrip: true,
        title: 'Fonts',
        icon: const GlyphBox(HG.text, size: 22, color: N.g95),
        search: search,
        classify: classify,
        groups: widget.groups ?? fontGroups(),
        hint: 'Search fonts',
        count: n >= 1000
            ? '${n ~/ 1000},${(n % 1000).toString().padLeft(3, '0')}'
            : '$n',
        wide: (c, s) => shown.isEmpty
            ? emptyBody('No typeface matches "${search.query}".')
            : _list(shown, s.width, 44),
        narrow: (c, s) => shown.isEmpty
            ? emptyBody('No typeface matches.')
            : _list(shown, s.width, 34),
        strip: (c, s) => ListView.builder(
          scrollDirection: Axis.horizontal,
          physics: const ClampingScrollPhysics(),
          padding: const EdgeInsets.all(7.5),
          itemCount: shown.length,
          itemBuilder: (_, i) => Padding(
            padding: const EdgeInsets.only(right: 4.5),
            child: Container(
              width: math.min(72, s.height * 1.3),
              decoration: BoxDecoration(
                color: Surface.raised,
                borderRadius: BorderRadius.circular(3),
              ),
              child: Center(child: _sample(shown[i], 24)),
            ),
          ),
        ),
      );
    },
  );

  Widget _list(List<FontItem> shown, double w, double h) => ListView.builder(
    physics: const ClampingScrollPhysics(),
    itemExtent: h,
    itemCount: shown.length,
    itemBuilder: (_, i) => _Row(
      shown[i],
      w,
      h,
      widget.selectedFamily == null
          ? i == 0 && !search.active && !classify.filtering
          : shown[i].family == widget.selectedFamily,
      sample: () => _sample(shown[i], h >= 40 ? 22 : 18),
      onTap: widget.onSelect == null ? null : () => widget.onSelect!(shown[i]),
      onDoubleTap: widget.onCreate == null
          ? null
          : () => widget.onCreate!(shown[i]),
      onFavorite: widget.onFavorite == null
          ? null
          : () => widget.onFavorite!(shown[i]),
    ),
  );

  Widget _sample(FontItem font, double size) {
    final placeholder = Text(
      font.sample,
      softWrap: false,
      style: TextStyle(
        fontFamily: font.family,
        fontSize: size,
        color: N.g95,
        height: 1,
      ),
    );
    final controller = widget.controller;
    if (controller == null || controller.state['visualSamples'] != true)
      return placeholder;
    final layer = controller.activeLayer;
    final sampleLayer =
        widget.sampleLayer ??
        (layer?['kind'] == 'Text' ? (layer!['id'] as num).toInt() : null);
    // with no Text layer chosen the host's sample is the face's name, which the row already shows in that face: the
    // slot keeps its readable glyph; with a Text layer chosen the host shows that text in the face
    if (sampleLayer == null) return placeholder;
    return NativeVisualSample(
      key: ValueKey('font-sample:$sampleLayer:${font.family}'),
      controller: controller,
      request: {
        'kind': 'font',
        'family': font.family,
        'layer': sampleLayer,
      },
      fit: BoxFit.contain,
      placeholder: placeholder,
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(
    this.f,
    this.w,
    this.h,
    this.chosen, {
    this.sample,
    this.onTap,
    this.onDoubleTap,
    this.onFavorite,
  });
  final FontItem f;
  final double w, h;
  final bool chosen;
  final Widget Function()? sample;
  final VoidCallback? onTap, onDoubleTap, onFavorite;
  @override
  Widget build(BuildContext context) {
    final big = h >= 40;
    return GestureDetector(
      key: ValueKey('hf-font:${f.family}'),
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      onDoubleTap: onDoubleTap,
      child: Container(
        decoration: BoxDecoration(
          color: chosen ? Surface.hover : null,
          border: Border(
            left: BorderSide(
              color: chosen ? N.g91 : N.clear,
              width: 1.5,
            ),
            bottom: const BorderSide(color: Surface.dividerFine),
          ),
        ),
        padding: const EdgeInsets.only(left: 7.5, right: 6),
        child: w < 110
            ? Center(
                child:
                    sample?.call() ??
                    Text(
                      f.sample,
                      softWrap: false,
                      style: TextStyle(
                        fontFamily: f.family,
                        fontSize: 16.5,
                        color: N.g95,
                        height: 1,
                      ),
                    ),
              )
            : Row(
                children: [
                  SizedBox(
                    width: big ? 38 : 30,
                    child:
                        sample?.call() ??
                        Text(
                          f.sample,
                          softWrap: false,
                          style: TextStyle(
                            fontFamily: f.family,
                            fontSize: big ? 22 : 18,
                            color: N.g95,
                            height: 1,
                          ),
                        ),
                  ),
                  Expanded(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          f.name,
                          softWrap: false,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            fontFamily: f.family,
                            fontSize: 11,
                            color: N.g91,
                            height: 1.1,
                          ),
                        ),
                        if (big) ...[
                          const SizedBox(height: 2),
                          Text(
                            '${f.meta}${f.facts.isEmpty ? '' : '  ·  ${f.facts.join('  ')}'}',
                            softWrap: false,
                            overflow: TextOverflow.ellipsis,
                            style: sans(Dn.microSize, c: Surface.muted),
                          ),
                        ],
                      ],
                    ),
                  ),
                  if (big)
                    GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: onFavorite,
                      child: SizedBox(
                        width: 15,
                        height: 15,
                        child: CustomPaint(painter: _Star(f.favorite)),
                      ),
                    ),
                ],
              ),
      ),
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
      final r = i.isEven ? 6 : 2.7;
      final a = -1.5708 + i * 0.6283;
      final q = Offset(8 + r * math.cos(a), 8.4 + r * math.sin(a));
      i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    p.close();
    c.drawPath(
      p,
      on
          ? (Paint()..color = N.g91)
          : (Paint()
              ..color = N.g33
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1),
    );
  }

  @override
  bool shouldRepaint(_Star o) => o.on != on;
}
