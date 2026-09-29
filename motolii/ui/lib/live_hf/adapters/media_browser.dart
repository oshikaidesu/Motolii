import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/things.dart';
import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import 'browser_item.dart';
import 'media_library.dart';
import 'media_list.dart';
import 'media_preview.dart';

enum BrowserView { list, thumbnail, explore }

/// How a Result Set is looked at is one of three, and the choice, the selected asset and the open preview belong to the
/// browser, not to a view: change the view and the same asset stays chosen (and its preview open).
///
///   Where / What / Which (the controls the owner answers)  →  Result Set  →  List | Thumbnail | Explore
///                                                                              ↘ Selection Preview
///
/// Explore is given by the caller (there is no similarity backend: an explorer prototype supplies it); without one the
/// view is not offered.
typedef ExploreBuilder = Widget Function(BuildContext context, List<BrowserItem> items, String? selected, ValueChanged<String> onTap, ValueChanged<String>? onOpen);

class MediaBrowser extends StatefulWidget {
  const MediaBrowser({super.key, required this.source, this.controls, this.faces, this.explore, this.sort = 'name', this.descending = false, this.onSort, this.onReveal, this.initial = BrowserView.thumbnail, this.startOn, this.startOpen = false});
  final ResultSource source;

  /// A browser that starts with an asset (by name) chosen, and its preview open or not (a story, a restored session).
  final String? startOn;
  final bool startOpen;

  /// Where / What / Which: the controls that build the result set (sources, folders, types, search).
  final Widget? controls;
  final FaceService? faces;
  final ExploreBuilder? explore;
  final String sort;
  final bool descending;
  final ValueChanged<String>? onSort;
  final void Function(BrowserItem item)? onReveal;
  final BrowserView initial;

  @override
  State<MediaBrowser> createState() => MediaBrowserState();
}

class MediaBrowserState extends State<MediaBrowser> {
  late BrowserView view = widget.initial;
  String? selected;
  bool preview = false;
  final focus = FocusNode(debugLabel: 'media browser');
  bool _seeded = false;

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  BrowserItem? _item(List<BrowserItem> items) {
    for (final i in items) {
      if (i.id == selected) return i;
    }
    return null;
  }

  void choose(String id) {
    focus.requestFocus();
    setState(() => selected = id);
  }

  void open(String id) => setState(() {
        selected = id;
        preview = true;
      });

  void step(List<BrowserItem> items, int by) {
    if (items.isEmpty) return;
    final at = items.indexWhere((i) => i.id == selected);
    setState(() => selected = items[(at + by).clamp(0, items.length - 1)].id);
  }

  KeyEventResult _key(List<BrowserItem> items, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.escape && preview) {
      setState(() => preview = false);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.space && selected != null) {
      setState(() => preview = !preview);
      return KeyEventResult.handled;
    }
    final by = view == BrowserView.list ? (k == LogicalKeyboardKey.arrowDown ? 1 : k == LogicalKeyboardKey.arrowUp ? -1 : 0) : (k == LogicalKeyboardKey.arrowRight ? 1 : k == LogicalKeyboardKey.arrowLeft ? -1 : 0);
    if (by != 0) {
      step(items, by);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: widget.source,
        builder: (context, _) {
          final items = widget.source.items;
          if (!_seeded && items.isNotEmpty) {
            _seeded = true;
            final named = items.where((i) => i.name == widget.startOn);
            if (named.isNotEmpty) {
              selected = named.first.id;
              preview = widget.startOpen;
            }
          }
          final chosen = _item(items);
          final showPreview = preview && chosen != null;
          return Focus(
            focusNode: focus,
            onKeyEvent: (_, e) => _key(items, e),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _Header(view: view, count: items.length, hasExplore: widget.explore != null, onView: (v) => setState(() => view = v)),
              if (widget.controls != null) widget.controls!,
              Expanded(
                flex: 5,
                child: ClipRect(child: _body(items)),
              ),
              if (showPreview)
                Expanded(
                  flex: 4,
                  child: MediaPreview(item: chosen, faces: widget.faces, onClose: () => setState(() => preview = false), onReveal: widget.onReveal == null ? null : () => widget.onReveal!(chosen)),
                ),
            ]),
          );
        },
      );

  Widget _body(List<BrowserItem> items) {
    if (items.isEmpty) return Padding(padding: const EdgeInsets.all(9), child: Text('Nothing matches.', style: Dn.label(N.g56)));
    switch (view) {
      case BrowserView.list:
        return MediaListView(items: items, selected: selected, onTap: choose, onOpen: open, sort: widget.sort, descending: widget.descending, onSort: widget.onSort ?? (_) {});
      case BrowserView.explore:
        final build = widget.explore;
        return build == null ? const SizedBox.shrink() : build(context, items, selected, choose, open);
      case BrowserView.thumbnail:
        final things = [for (final i in items) Thing.fromJson({'id': i.id, 'name': i.name, 'kind': 'item', 'family': 'catalog', 'source': 'catalog', 'face': const {'type': 'shelf'}})];
        final shelf = {for (final i in items) i.id: i.shelf};
        return LayoutBuilder(builder: (context, box) => MediaLibraryBody(sections: {'': things}, shown: things, items: shelf, width: box.maxWidth, selected: selected, onTap: choose, onOpen: open));
    }
  }
}

class _Header extends StatelessWidget {
  const _Header({required this.view, required this.count, required this.hasExplore, required this.onView});
  final BrowserView view;
  final int count;
  final bool hasExplore;
  final ValueChanged<BrowserView> onView;
  @override
  Widget build(BuildContext context) => Container(
        height: UiMetrics.chromeRow,
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g15))),
        child: Row(children: [
          for (final v in BrowserView.values)
            if (v != BrowserView.explore || hasExplore)
              GestureDetector(
                key: ValueKey('view-${v.name}'),
                behavior: HitTestBehavior.opaque,
                onTap: () => onView(v),
                child: Container(
                  margin: const EdgeInsets.only(right: 2),
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                  decoration: BoxDecoration(color: v == view ? N.g20 : null, borderRadius: BorderRadius.circular(4)),
                  child: Text(const {BrowserView.list: 'List', BrowserView.thumbnail: 'Thumbnail', BrowserView.explore: 'Explore'}[v]!, softWrap: false, style: Dn.name(v == view ? N.g95 : N.g63, v == view ? FontWeight.w600 : FontWeight.w500)),
                ),
              ),
          const Spacer(),
          Text('$count', style: Dn.value(N.g51).copyWith(fontSize: 10)),
        ]),
      );
}
