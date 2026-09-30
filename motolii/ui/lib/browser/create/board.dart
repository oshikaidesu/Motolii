import 'package:flutter/widgets.dart';

import '../parts.dart';
import '../shelf_sections.dart' show SwissHeading;
import '../../effects/shelf.dart';
import 'faces.dart';
import '../seat.dart';
import '../things.dart';
import '../../theme/neutral.dart';
import '../../theme/metrics.dart' show Dn, Surface;

/// Create as a board of named tiles: each thing its mark over its name on a quiet ground, on a grid that fills the
/// seat's width, the section as a row identifier, Recent last. Small marks and short names keep it dense; the name is
/// on the tile, so nothing repeats it elsewhere. Its mechanics (what a tile does, picking, menus, keys, Recent) are
/// the seat's and the panel's — this is only a face.
class CreateTiles extends StatelessWidget {
  const CreateTiles({super.key, required this.sections, required this.recent, required this.width, this.scene});
  final Map<String, List<Thing>> sections;
  final List<Thing> recent;
  final double width;
  final EffectScene? scene;

  /// The grid: the least a tile is wide, its height, the gap between tiles, the mark inside, the panel's inset.
  static double get least => Surface.px(44.0);
  static double get height => Surface.faceTile;
  static double get gap => Surface.inlineGap;
  static double get mark => Surface.px(16.0);
  static double get pad => Surface.sectionGap;

  static int columns(double width) => ((width - pad * 2 + gap) / (least + gap)).floor().clamp(1, 99);

  /// Tiles share the row's width: no ragged gap at the right edge.
  static double tileWidth(double width) {
    final n = columns(width);
    return ((width - pad * 2 - gap * (n - 1)) / n).floorToDouble();
  }

  @override
  Widget build(BuildContext context) {
    final cols = columns(width), w = tileWidth(width);
    BrowserSeatScope.of(context)?.shows([for (final e in sections.values) ...e], cols);
    return ListView(
      physics: const ClampingScrollPhysics(),
      padding: EdgeInsets.fromLTRB(pad, Surface.px(4.5), pad, Surface.sectionGap),
      children: [
        for (final e in sections.entries) _section(context, e.key[0].toUpperCase() + e.key.substring(1).toLowerCase(), e.value, w),
        if (recent.isNotEmpty) _section(context, 'Recent', recent.take(cols).toList(), w),
      ],
    );
  }

  /// A section: its identifier on one short line, then its tiles on the grid.
  Widget _section(BuildContext context, String id, List<Thing> things, double w) => Padding(
        padding: EdgeInsets.only(top: Surface.inlineGap, bottom: Surface.px(4.5)),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          SwissHeading(id, rule: false),
          Wrap(
            spacing: gap,
            runSpacing: gap,
            children: [for (final t in things) seated(context, t, _Tile(t, scene, w))],
          ),
        ]),
      );
}

/// One tile: the mark, then the name under it, on a quiet ground that lights under the pointer.
class _Tile extends StatefulWidget {
  const _Tile(this.thing, this.scene, this.width);
  final Thing thing;
  final EffectScene? scene;
  final double width;
  @override
  State<_Tile> createState() => _TileState();
}

class _TileState extends State<_Tile> {
  bool _over = false;
  void _set(bool over) {
    if (mounted) setState(() => _over = over);
  }

  @override
  Widget build(BuildContext context) => Semantics(
        label: widget.thing.name,
        button: true,
        excludeSemantics: true,
        child: MouseRegion(
          onEnter: (_) => _set(true),
          onExit: (_) => _set(false),
          child: Container(
            width: widget.width,
            height: CreateTiles.height,
            padding: EdgeInsets.fromLTRB(Surface.px(2), Surface.px(5), Surface.px(2), 0),
            decoration: BoxDecoration(color: _over ? Surface.hover : Surface.raised, borderRadius: BorderRadius.circular(Surface.controlRadius)),
            child: Column(children: [
              SizedBox.square(dimension: CreateTiles.mark, child: ThingFace(widget.thing, scene: widget.scene)),
              SizedBox(height: Surface.px(4)),
              Text(widget.thing.name, softWrap: false, maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.center, style: sans(Dn.microSize, c: N.g82)),
            ]),
          ),
        ),
      );
}
