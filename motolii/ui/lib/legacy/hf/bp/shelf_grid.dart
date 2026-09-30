import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../../../browser/classify.dart';
import '../../../browser/parts.dart';
import '../../../browser/create/faces.dart';
import '../../../browser/search.dart';
import '../../../browser/seat.dart';
import '../../../browser/panel_chrome.dart';
import '../../../browser/things.dart';
import '../../../theme/neutral.dart';
import '../../../theme/surface.dart' show Dn;

/// The finished Browser's body for a shelf whose things the host draws itself (pictures, swatches, specimens, files).
/// The chassis is the same as Create and Effects: header, class column, search, sections, keyboard through the seat.
/// The host's editor (a colour wheel, a font scope) sits above the tiles and scrolls with them.
class ShelfGridPanel extends StatefulWidget {
  const ShelfGridPanel({
    super.key,
    required this.title,
    required this.icon,
    required this.noun,
    required this.panelId,
    required this.catalog,
    this.user,
    this.search,
    this.classify,
    this.captions = true,
    this.sections = false,
    this.body,
    this.classStrip = false,
  });
  final String title, noun, panelId;
  final Widget icon;
  final Catalog catalog;
  final UserViews? user;
  final SearchCapability? search;
  final ClassifyCapability? classify;

  /// A caption under each tile (the host's tile draws its own when it has none).
  final bool captions;

  /// Tiles are grouped under their class when nothing narrows the list.
  final bool sections;

  /// A shelf that lays its own sections out (its faces differ by what they show); it tells the seat what it shows.
  /// The chassis — header, classes, search, empty state, strip — stays this panel's.
  final Widget Function(BuildContext context, Map<String, List<Thing>> sections, List<Thing> shown, Size size)? body;

  /// Classes along the top instead of the column (see [PanelShell.classStrip]).
  final bool classStrip;
  @override
  State<ShelfGridPanel> createState() => _ShelfGridPanelState();
}

class _ShelfGridPanelState extends State<ShelfGridPanel> with WithDiscovery<ShelfGridPanel> {
  @override
  SearchCapability? get injectedSearch => widget.search;
  @override
  ClassifyCapability? get injectedClassify => widget.classify;
  late final views = ThingViews(widget.catalog.registry, widget.catalog.things, widget.catalog.registry.panels[widget.panelId]!, widget.user ?? UserViews());

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: discovery,
        builder: (_, __) {
          final q = ThingQuery.parse(search.query);
          final found = q.isEmpty ? views.scope : [for (final t in views.scope) if (q.matches(t, widget.catalog.registry)) t];
          final shown = [for (final t in found) if (views.contains(classify.selected, t)) t];
          final sections = <String, List<Thing>>{};
          if (widget.sections && !search.active) {
            for (final t in shown) {
              (sections[views.sectionOf(t)] ??= []).add(t);
            }
          } else {
            sections[''] = shown;
          }
          final n = shown.length;
          final seat = BrowserSeatScope.of(context);
          final editor = seat?.editor(context);
          final noun = widget.noun;
          return PanelShell(
            title: widget.title,
            icon: widget.icon,
            search: search,
            classify: classify,
            groups: views.groups(found),
            hint: 'Search ${widget.title.toLowerCase()}',
            classStrip: widget.classStrip,
            count: n >= 1000 ? '${n ~/ 1000},${(n % 1000).toString().padLeft(3, '0')}' : '$n',
            wide: (c, s) => _body(sections, shown, editor, s, narrow: false, empty: 'No $noun matches "${search.query}".'),
            narrow: (c, s) => _body(sections, shown, editor, s, narrow: true, empty: 'No $noun matches.'),
            strip: (c, s) => ListView.builder(
              scrollDirection: Axis.horizontal,
              physics: const ClampingScrollPhysics(),
              padding: const EdgeInsets.all(7.5),
              itemCount: shown.length,
              itemBuilder: (c, i) => Padding(padding: const EdgeInsets.only(right: 4.5), child: SizedBox(width: math.min(s.height * 1.3, 96), child: seated(c, shown[i], ThingFace(shown[i])))),
            ),
          );
        },
      );

  Widget _body(Map<String, List<Thing>> sections, List<Thing> shown, Widget? editor, Size s, {required bool narrow, required String empty}) {
    final own = widget.body;
    if (own != null) return shown.isEmpty ? emptyBody(empty) : own(context, sections, shown, s);
    final seat = BrowserSeatScope.of(context);
    final tiling = seat?.tiling(context, s.width);
    final pad = tiling?.padding ?? 9.0, gap = tiling?.gap ?? 5.0;
    final scale = seat?.tileScale ?? 1;
    final column = tiling?.column ?? (narrow && s.width < 210 ? s.width - pad * 2 : 72.0 * scale);
    final cols = math.max(1, ((s.width - pad * 2 + gap) / (column + gap)).floor());
    seat?.shows(shown, cols);
    final tileW = (s.width - pad * 2 - gap * (cols - 1)) / cols;
    final captioned = tiling == null && widget.captions && tileW >= 48;
    final extent = tiling?.extent ?? tileW * .84 + (captioned ? 14 : 0);
    return CustomScrollView(
      physics: const ClampingScrollPhysics(),
      slivers: [
        if (editor != null) SliverToBoxAdapter(child: editor),
        if (shown.isEmpty) SliverToBoxAdapter(child: SizedBox(height: 90, child: emptyBody(empty))),
        for (final e in sections.entries) ...[
          if (e.key.isNotEmpty) SliverToBoxAdapter(child: Padding(padding: EdgeInsets.symmetric(horizontal: pad), child: SectionLabel(e.key))),
          SliverPadding(
            padding: EdgeInsets.fromLTRB(pad, e.key.isEmpty ? 10 : 0, pad, 4.5),
            sliver: SliverGrid(
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: gap, crossAxisSpacing: gap, childAspectRatio: tileW / extent),
              delegate: SliverChildBuilderDelegate((c, i) => seated(c, e.value[i], _tile(e.value[i], captioned)), childCount: e.value.length),
            ),
          ),
        ],
        const SliverToBoxAdapter(child: SizedBox(height: 9)),
      ],
    );
  }

  Widget _tile(Thing t, bool caption) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Expanded(child: ThingFace(t)),
        if (caption) Padding(padding: const EdgeInsets.only(top: 4), child: Text(t.name, softWrap: false, maxLines: 1, overflow: TextOverflow.ellipsis, style: sans(Dn.labelSize, c: N.g76))),
      ]);
}
