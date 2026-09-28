import 'package:flutter/widgets.dart';

import 'common.dart';
import 'effects.dart';
import 'faces.dart';
import 'seat.dart';
import 'things.dart';

/// Create as a Swiss symbol matrix: the marks themselves on a strict grid, no ground under a key until the pointer is
/// on it, the section as a row identifier, and the name of the key under the pointer on the board's one status line.
/// Its mechanics (what a key does, picking, menus, keys, Recent) are the seat's and the panel's — this is only a face.
class SymbolMatrix extends StatefulWidget {
  const SymbolMatrix({super.key, required this.sections, required this.recent, required this.width, this.scene});
  final Map<String, List<Thing>> sections;
  final List<Thing> recent;
  final double width;
  final EffectScene? scene;

  /// The grid: a key's footprint, the gap between keys, the mark inside a key.
  static const cell = 40.0, gap = 4.0, mark = 26.0, pad = 8.0;

  static int columns(double width) => ((width - pad * 2 + gap) / (cell + gap)).floor().clamp(1, 99);

  @override
  State<SymbolMatrix> createState() => _SymbolMatrixState();
}

class _SymbolMatrixState extends State<SymbolMatrix> {
  /// The key under the pointer: its name is the board's status line (names are not printed under every mark).
  final hovered = ValueNotifier<Thing?>(null);

  @override
  void dispose() {
    hovered.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final cols = SymbolMatrix.columns(widget.width);
    BrowserSeatScope.of(context)?.shows([for (final e in widget.sections.values) ...e], cols);
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Expanded(
        child: ListView(
          physics: const ClampingScrollPhysics(),
          padding: const EdgeInsets.fromLTRB(SymbolMatrix.pad, 6, SymbolMatrix.pad, 8),
          children: [
            for (final e in widget.sections.entries) _section(context, _identifier(e.key), e.value),
            if (widget.recent.isNotEmpty) _section(context, 'RECENT', widget.recent.take(cols).toList()),
          ],
        ),
      ),
      // the status line: what the key under the pointer is; quiet when nothing is under it
      Container(
        height: 20,
        padding: const EdgeInsets.symmetric(horizontal: SymbolMatrix.pad + 2),
        alignment: Alignment.centerLeft,
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: kRule2))),
        child: ValueListenableBuilder<Thing?>(
          valueListenable: hovered,
          builder: (_, t, __) => Text(t?.name ?? '', key: const ValueKey('create-status'), softWrap: false, overflow: TextOverflow.ellipsis, style: sans(10.5, c: const Color(0xFFD6D7DA), w: FontWeight.w500)),
        ),
      ),
    ]);
  }

  /// A section: its identifier on one short line, then its keys on the grid.
  Widget _section(BuildContext context, String id, List<Thing> things) => Padding(
        padding: const EdgeInsets.only(top: 4, bottom: 6),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Padding(padding: const EdgeInsets.only(left: 2, bottom: 2), child: Text(id, softWrap: false, style: sans(9, c: const Color(0xFFB4B6BD), w: FontWeight.w600, ls: .9))),
          Wrap(
            spacing: SymbolMatrix.gap,
            runSpacing: SymbolMatrix.gap,
            children: [for (final t in things) seated(context, t, _Key(t, widget.scene, hovered))],
          ),
        ]),
      );

  /// "PRIMITIVES" stays a caps identifier; a longer class name is cut to a short one.
  static String _identifier(String s) => s.toUpperCase();
}

/// One key: the mark, centred in its footprint. A ground appears under the pointer; the name goes to the status line
/// (and to assistive technology through the semantics label).
class _Key extends StatefulWidget {
  const _Key(this.thing, this.scene, this.hovered);
  final Thing thing;
  final EffectScene? scene;
  final ValueNotifier<Thing?> hovered;
  @override
  State<_Key> createState() => _KeyState();
}

class _KeyState extends State<_Key> {
  bool _over = false;
  void _set(bool over) {
    setState(() => _over = over);
    if (over) {
      widget.hovered.value = widget.thing;
    } else if (widget.hovered.value == widget.thing) {
      widget.hovered.value = null;
    }
  }

  @override
  Widget build(BuildContext context) => Semantics(
        label: widget.thing.name,
        button: true,
        child: MouseRegion(
          onEnter: (_) => _set(true),
          onExit: (_) => _set(false),
          child: Container(
            width: SymbolMatrix.cell,
            height: SymbolMatrix.cell,
            alignment: Alignment.center,
            decoration: BoxDecoration(color: _over ? kRaisedHi : null, borderRadius: BorderRadius.circular(3)),
            child: SizedBox.square(dimension: SymbolMatrix.mark, child: ThingFace(widget.thing, scene: widget.scene)),
          ),
        ),
      );
}
