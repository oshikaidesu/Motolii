import 'package:flutter/widgets.dart';

import '../../hf/bp/shelf_sections.dart' show PickedRing;
import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import 'browser_item.dart';
import 'media_library.dart' show materialFace;

/// List View: manage by information. One dense row per asset (a small face, then the facts the catalog has), columns that
/// ask the owner for another order, and folding rather than shrinking when the seat is narrow.
class MediaListView extends StatelessWidget {
  const MediaListView({super.key, required this.items, required this.selected, required this.onTap, this.onOpen, required this.sort, required this.descending, required this.onSort});
  final List<BrowserItem> items;
  final String? selected;
  final ValueChanged<String> onTap;
  final ValueChanged<String>? onOpen;
  final String sort;
  final bool descending;

  /// A column header: the sort key to ask the owner for (name, kind, size, modified).
  final ValueChanged<String> onSort;

  static const rowHeight = 32.0;

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        final wide = box.maxWidth >= 300, roomy = box.maxWidth >= 420;
        return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          MediaListHeader(width: box.maxWidth, sort: sort, descending: descending, onSort: onSort),
          Expanded(
            child: ListView.builder(
              itemExtent: rowHeight,
              itemCount: items.length,
              itemBuilder: (context, i) {
                final it = items[i], on = it.id == selected;
                return GestureDetector(
                  key: ValueKey('list-${it.id}'),
                  behavior: HitTestBehavior.opaque,
                  onTap: () => onTap(it.id),
                  onDoubleTap: onOpen == null ? null : () => onOpen!(it.id),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    color: on ? N.g15 : null,
                    child: Row(children: [
                      SizedBox(width: 54, child: Align(alignment: Alignment.centerLeft, child: _Face(it, selected: on))),
                      Expanded(child: Text(it.name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: Dn.name(on ? N.g100 : N.g91))),
                      const SizedBox(width: 8),
                      SizedBox(width: 42, child: Text(it.typeWord, softWrap: false, style: Dn.label(N.g63))),
                      SizedBox(width: 40, child: Text(clockText(it.seconds), softWrap: false, style: Dn.value(N.g69).copyWith(fontSize: 10))),
                      if (wide) SizedBox(width: 54, child: Text(sizeText(it.size), softWrap: false, textAlign: TextAlign.right, style: Dn.value(N.g69).copyWith(fontSize: 10))),
                      if (roomy) SizedBox(width: 74, child: Text(dateText(it.mtimeNs), softWrap: false, textAlign: TextAlign.right, style: Dn.value(N.g56).copyWith(fontSize: 10))),
                    ]),
                  ),
                );
              },
            ),
          ),
        ]);
      });
}

/// The columns' titles: a click asks the owner for that order (an arrow shows the one in force).
class MediaListHeader extends StatelessWidget {
  const MediaListHeader({super.key, required this.width, required this.sort, required this.descending, required this.onSort});
  final double width;
  final String sort;
  final bool descending;
  final ValueChanged<String> onSort;

  static bool wide(double w) => w >= 300;
  static bool roomy(double w) => w >= 420;

  @override
  Widget build(BuildContext context) {
    Widget header(String label, String? key, {double? width, TextAlign align = TextAlign.left}) {
      final on = key != null && key == sort;
      final text = Text(on ? '$label ${descending ? '↓' : '↑'}' : label, softWrap: false, overflow: TextOverflow.clip, textAlign: align, style: Dn.micro(on ? N.g95 : N.g56));
      Widget cell = width == null ? Expanded(child: text) : SizedBox(width: width, child: text);
      if (key != null) cell = width == null ? Expanded(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => onSort(key), child: text)) : GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => onSort(key), child: cell);
      return cell;
    }

    return Container(
      height: 20,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g15))),
      child: Row(children: [
        const SizedBox(width: 54),
        header('NAME', 'name'),
        header('TYPE', 'kind', width: 42),
        header('LENGTH', null, width: 40),
        if (wide(width)) header('SIZE', 'size', width: 54, align: TextAlign.right),
        if (roomy(width)) header('DATE', 'modified', width: 74, align: TextAlign.right),
      ]),
    );
  }
}

/// The asset's own face in a small box, at its own shape (never cropped into a square).
class _Face extends StatelessWidget {
  const _Face(this.item, {required this.selected});
  final BrowserItem item;
  final bool selected;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, _) {
        const w = 46.0, h = 26.0;
        final a = item.aspect;
        final fw = a >= w / h ? w : h * a, fh = a >= w / h ? w / a : h;
        return SizedBox(
          width: fw,
          height: fh,
          child: Stack(fit: StackFit.expand, children: [
            ClipRRect(borderRadius: BorderRadius.circular(2), child: ColoredBox(color: N.g13, child: materialFace(item.shelf))),
            if (selected) const PickedRing(radius: 2),
          ]),
        );
      });
}
