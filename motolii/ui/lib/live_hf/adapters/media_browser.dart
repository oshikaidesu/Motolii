import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/things.dart';
import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import 'browser_item.dart';
import 'media_fluid.dart';
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
  const MediaBrowser({super.key, required this.source, this.controls, this.faces, this.explore, this.sort = 'name', this.descending = false, this.onSort, this.onReveal, this.onPlace, this.initial = BrowserView.thumbnail, this.startOn, this.startOpen = false, this.exploreLayout, this.exploreRepaint, this.exploreNote, this.holding = const {}, this.startColumn = 60, this.fluidUpTo = 300});
  final ResultSource source;

  /// Where Explore puts the faces (for the fluid board). Without it Explore is `explore` alone, and the views swap.
  final ExploreLayout? exploreLayout;

  /// Fires when Explore's places change by themselves (a colour was measured): the board asks again.
  final Listenable? exploreRepaint;

  /// One honest line under the controls while Explore is shown (what nearness means).
  final String? exploreNote;

  /// Pointers held over faces, for a story (see FluidBoard.holding).
  final Map<String, Offset> holding;

  /// The narrowest a Thumbnail column starts at (the size slider moves it).
  final double startColumn;

  /// The fluid board draws every face at once, so it serves result sets up to this size; larger ones use the views that
  /// draw only what is on screen.
  final int fluidUpTo;

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

  /// Uses the asset in the work (only when the host can): the work admits it as its own then, and browsing alone never does.
  final void Function(BrowserItem item)? onPlace;
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
  late double thumbMin = widget.startColumn;

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
              _Header(view: view, count: items.length, hasExplore: widget.explore != null || widget.exploreLayout != null, onView: (v) => setState(() => view = v), size: view == BrowserView.thumbnail ? (thumbMin - 44) / 96 : null, onSize: (f) => setState(() => thumbMin = 44 + f * 96)),
              if (widget.controls != null) widget.controls!,
              if (view == BrowserView.explore && widget.exploreNote != null) Padding(padding: const EdgeInsets.fromLTRB(9, 3, 9, 0), child: Text(widget.exploreNote!, style: Dn.label(N.g51))),
              Expanded(
                flex: 5,
                child: ClipRect(child: _body(items)),
              ),
              if (showPreview)
                Expanded(
                  flex: 4,
                  child: MediaPreview(item: chosen, faces: widget.faces, onClose: () => setState(() => preview = false), onReveal: widget.onReveal == null ? null : () => widget.onReveal!(chosen), onPlace: widget.onPlace == null ? null : () => widget.onPlace!(chosen)),
                ),
            ]),
          );
        },
      );

  Widget _body(List<BrowserItem> items) {
    if (items.isEmpty) return Padding(padding: const EdgeInsets.all(9), child: Text('Nothing matches.', style: Dn.label(N.g56)));
    final fluid = items.length <= widget.fluidUpTo && (view != BrowserView.explore || widget.exploreLayout != null);
    if (fluid) {
      return LayoutBuilder(builder: (context, box) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            AnimatedSize(
              duration: const Duration(milliseconds: 260),
              alignment: Alignment.topCenter,
              child: view == BrowserView.list ? MediaListHeader(width: box.maxWidth, sort: widget.sort, descending: widget.descending, onSort: widget.onSort ?? (_) {}) : const SizedBox(width: double.infinity),
            ),
            Expanded(
              child: ListenableBuilder(
                listenable: widget.exploreRepaint ?? const _Never(),
                builder: (context, _) => FluidBoard(items: items, selected: selected, view: view.name, onTap: choose, onOpen: open, explore: widget.exploreLayout, faces: widget.faces, holding: widget.holding, minColumn: thumbMin, revealOn: preview),
              ),
            ),
          ]));
    }
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
  const _Header({required this.view, required this.count, required this.hasExplore, required this.onView, this.size, required this.onSize});
  final double? size; // 0..1, only while Thumbnail is shown
  final ValueChanged<double> onSize;
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
          if (size != null) Padding(padding: const EdgeInsets.only(right: 8), child: SizedBox(width: 74, child: _SizeSlider(value: size!, onChanged: onSize))),
          Text('$count', style: Dn.value(N.g51).copyWith(fontSize: 10)),
        ]),
      );
}

class _Never implements Listenable {
  const _Never();
  @override
  void addListener(VoidCallback listener) {}
  @override
  void removeListener(VoidCallback listener) {}
}

/// A small size slider: smaller faces at the left, bigger at the right.
class _SizeSlider extends StatelessWidget {
  const _SizeSlider({required this.value, required this.onChanged});
  final double value;
  final ValueChanged<double> onChanged;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        void set(Offset p) => onChanged((p.dx / box.maxWidth).clamp(0.0, 1.0));
        return GestureDetector(
          key: const ValueKey('thumb-size'),
          behavior: HitTestBehavior.opaque,
          onPanDown: (d) => set(d.localPosition),
          onPanUpdate: (d) => set(d.localPosition),
          child: CustomPaint(size: Size(box.maxWidth, 22), painter: _SizePainter(value)),
        );
      });
}

class _SizePainter extends CustomPainter {
  const _SizePainter(this.value);
  final double value;
  @override
  void paint(Canvas c, Size s) {
    final y = s.height / 2;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(0, y - 1, s.width, 2), const Radius.circular(1)), Paint()..color = N.g26);
    c.drawCircle(Offset(3 + value * (s.width - 6), y), 4.5, Paint()..color = N.g91);
  }

  @override
  bool shouldRepaint(_SizePainter o) => o.value != value;
}
