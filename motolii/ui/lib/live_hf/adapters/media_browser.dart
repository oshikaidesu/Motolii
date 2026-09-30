import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/things.dart';
import '../../hf/metrics.dart';
import '../../hf/shell/menu.dart' show showHfMenu;
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
  const MediaBrowser({super.key, required this.source, this.controls, this.faces, this.explore, this.sort = 'name', this.descending = false, this.onSort, this.onReveal, this.onPlace, this.menuOf, this.onMenu, this.onRemove, this.onFavorite, this.onSeen, this.carry, this.initial = BrowserView.thumbnail, this.startOn, this.startOpen = false, this.exploreLayout, this.exploreRepaint, this.exploreNote, this.exploreBar, this.holding = const {}, this.startColumn = 60, this.fluidUpTo = 300});
  final ResultSource source;

  /// Where Explore puts the faces (for the fluid board). Without it Explore is `explore` alone, and the views swap.
  final ExploreLayout? exploreLayout;

  /// Fires when Explore's places change by themselves (a colour was measured): the board asks again.
  final Listenable? exploreRepaint;

  /// One honest line under the controls while Explore is shown (what nearness means).
  final String? exploreNote;

  /// The caller's own controls for Explore (its relation filters, global or local), shown instead of the note.
  final Widget? exploreBar;

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

  /// The right-click menu of an item (what the person clicked on, and everything picked): rows to show, and what to do with
  /// the one chosen. The caller owns what each row means; the browser only picks, shows and hands back.
  final List<({String value, String label, bool enabled})> Function(BrowserItem item, List<BrowserItem> picked)? menuOf;
  final void Function(String action, BrowserItem item, List<BrowserItem> picked)? onMenu;

  /// Delete / Backspace over the picked items (the caller decides what it may remove: never a source file).
  final void Function(List<BrowserItem> picked)? onRemove;

  /// `F` over the picked items: keep them (or let them go). The caller owns what keeping means.
  final void Function(List<BrowserItem> picked)? onFavorite;

  /// An asset's preview opened: the caller may remember it (Recents).
  final ValueChanged<BrowserItem>? onSeen;

  /// What dragging an item carries to a drop target such as the Timeline, or null when it cannot be carried.
  final Map<String, dynamic>? Function(BrowserItem item)? carry;
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

  /// Everything picked; [selected] is the last pick (the one the preview and the ring's focus read).
  final picked = <String>{};
  String? _anchor;
  double _width = 320;

  BrowserItem? _item(List<BrowserItem> items) {
    for (final i in items) {
      if (i.id == selected) return i;
    }
    return null;
  }

  List<BrowserItem> _pickedItems(List<BrowserItem> items) => [for (final i in items) if (picked.contains(i.id)) i];

  /// A click picks one; with Cmd/Ctrl it adds or drops one; with Shift it picks the run from the anchor (in the order shown).
  void choose(String id) {
    focus.requestFocus();
    final kb = HardwareKeyboard.instance;
    final items = widget.source.items;
    setState(() {
      if (kb.isMetaPressed || kb.isControlPressed) {
        picked.contains(id) ? picked.remove(id) : picked.add(id);
        _anchor = id;
        selected = picked.contains(id) ? id : (picked.isEmpty ? null : picked.last);
      } else if (kb.isShiftPressed && _anchor != null) {
        final a = items.indexWhere((i) => i.id == _anchor), b = items.indexWhere((i) => i.id == id);
        if (a >= 0 && b >= 0) {
          picked
            ..clear()
            ..addAll(items.sublist(math.min(a, b), math.max(a, b) + 1).map((i) => i.id));
          selected = id;
        }
      } else {
        picked
          ..clear()
          ..add(id);
        selected = id;
        _anchor = id;
      }
    });
  }

  /// Picks these and nothing else (something imported, a caller's own choice).
  void pick(Iterable<String> ids) => setState(() {
        picked
          ..clear()
          ..addAll(ids);
        selected = picked.isEmpty ? null : picked.last;
        _anchor = selected;
      });

  /// Double-click is what the old shelf made it: use the asset (place it). Where the caller cannot place, it opens the preview.
  void useAsset(String id) {
    final item = widget.source.items.where((i) => i.id == id).firstOrNull;
    if (item != null && widget.onPlace != null) {
      _to(id);
      widget.onPlace!(item);
    } else {
      open(id);
    }
  }

  void _seen(String? id) {
    final item = id == null ? null : widget.source.items.where((i) => i.id == id).firstOrNull;
    if (item != null) widget.onSeen?.call(item);
  }

  void open(String id) {
    _seen(id);
    _open(id);
  }

  void _open(String id) => setState(() {
        picked
          ..clear()
          ..add(id);
        selected = id;
        _anchor = id;
        preview = true;
      });

  void _to(String id) => setState(() {
        picked
          ..clear()
          ..add(id);
        selected = id;
        _anchor = id;
      });

  /// One step by the arrows. Thumbnail is a masonry, so up and down go to the nearest face above or below by where it is.
  void step(List<BrowserItem> items, {int by = 0, int rows = 0}) {
    if (items.isEmpty) return;
    final at = items.indexWhere((i) => i.id == selected);
    if (rows != 0 && view == BrowserView.thumbnail) {
      final frame = FluidBoard.thumbnail(items, _width, thumbMin);
      final here = at < 0 ? null : frame.faces[items[at].id];
      if (here == null) return _to(items[rows > 0 ? 0 : items.length - 1].id);
      String? best;
      var bestScore = double.infinity;
      for (final e in frame.faces.entries) {
        final dy = (e.value.center.dy - here.center.dy) * rows;
        if (dy <= 1) continue;
        final score = dy + 2 * (e.value.center.dx - here.center.dx).abs();
        if (score < bestScore) {
          bestScore = score;
          best = e.key;
        }
      }
      if (best != null) _to(best);
      return;
    }
    _to(items[(at + (by != 0 ? by : rows)).clamp(0, items.length - 1)].id);
  }

  bool get _typing => FocusManager.instance.primaryFocus?.context?.widget is EditableText;

  KeyEventResult _key(List<BrowserItem> items, KeyEvent e) {
    if (e is! KeyDownEvent || _typing) return KeyEventResult.ignored;
    final k = e.logicalKey;
    final kb = HardwareKeyboard.instance;
    if (k == LogicalKeyboardKey.escape) {
      if (preview) {
        setState(() => preview = false);
      } else if (picked.isNotEmpty) {
        setState(() {
          picked.clear();
          selected = null;
        });
      } else {
        return KeyEventResult.ignored;
      }
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.keyA && (kb.isMetaPressed || kb.isControlPressed)) {
      setState(() {
        picked
          ..clear()
          ..addAll(items.map((i) => i.id));
        selected = items.isEmpty ? null : items.last.id;
      });
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.space && selected != null) {
      if (!preview) _seen(selected);
      setState(() => preview = !preview);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.keyF && !kb.isMetaPressed && !kb.isControlPressed && picked.isNotEmpty && widget.onFavorite != null) {
      widget.onFavorite!(_pickedItems(items));
      return KeyEventResult.handled;
    }
    if ((k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) && selected != null && widget.onPlace != null) {
      widget.onPlace!(_item(items) ?? items.first);
      return KeyEventResult.handled;
    }
    if ((k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) && picked.isNotEmpty && widget.onRemove != null) {
      widget.onRemove!(_pickedItems(items));
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.home && items.isNotEmpty) {
      _to(items.first.id);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.end && items.isNotEmpty) {
      _to(items.last.id);
      return KeyEventResult.handled;
    }
    final side = k == LogicalKeyboardKey.arrowRight ? 1 : (k == LogicalKeyboardKey.arrowLeft ? -1 : 0);
    final down = k == LogicalKeyboardKey.arrowDown ? 1 : (k == LogicalKeyboardKey.arrowUp ? -1 : 0);
    if (view == BrowserView.list && down != 0) {
      step(items, by: down);
      return KeyEventResult.handled;
    }
    if (view != BrowserView.list && (side != 0 || down != 0)) {
      if (view == BrowserView.thumbnail && side == 0) {
        step(items, rows: down);
      } else {
        step(items, by: side != 0 ? side : down);
      }
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  Future<void> _menu(String id, Offset at) async {
    final items = widget.source.items;
    final item = items.where((i) => i.id == id).firstOrNull;
    if (item == null || widget.menuOf == null) return;
    // right-clicking something not picked picks it alone; on a pick, the whole pick stays
    if (!picked.contains(id)) _to(id);
    focus.requestFocus();
    final rows = widget.menuOf!(item, _pickedItems(items));
    if (rows.isEmpty || !mounted) return;
    final chosen = await showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 180, 0), [('', item.name), for (final r in rows) (r.value, r.label)], info: const {''}, disabled: {for (final r in rows) if (!r.enabled) r.value});
    if (chosen != null && chosen.isNotEmpty) widget.onMenu?.call(chosen, item, _pickedItems(widget.source.items));
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
          picked.removeWhere((id) => !items.any((i) => i.id == id));
          final chosen = _item(items);
          final showPreview = preview && chosen != null;
          return Focus(
            focusNode: focus,
            onKeyEvent: (_, e) => _key(items, e),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _Header(view: view, count: items.length, hasExplore: widget.explore != null || widget.exploreLayout != null, onView: (v) => setState(() => view = v), size: view == BrowserView.thumbnail ? (thumbMin - 44) / 96 : null, onSize: (f) => setState(() => thumbMin = 44 + f * 96)),
              if (widget.controls != null) widget.controls!,
              if (view == BrowserView.explore && widget.exploreBar != null) widget.exploreBar!,
              if (view == BrowserView.explore && widget.exploreBar == null && widget.exploreNote != null) Padding(padding: const EdgeInsets.fromLTRB(9, 3, 9, 0), child: Text(widget.exploreNote!, style: Dn.label(N.g51))),
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
      return LayoutBuilder(builder: (context, box) {
        _width = box.maxWidth;
        return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            AnimatedSize(
              duration: const Duration(milliseconds: 160),
              curve: Curves.easeOutCubic,
              alignment: Alignment.topCenter,
              child: view == BrowserView.list ? MediaListHeader(width: box.maxWidth, sort: widget.sort, descending: widget.descending, onSort: widget.onSort ?? (_) {}) : const SizedBox(width: double.infinity),
            ),
            Expanded(
              child: ListenableBuilder(
                listenable: widget.exploreRepaint ?? const _Never(),
                builder: (context, _) => FluidBoard(items: items, selected: selected, picked: picked, carry: widget.carry, onMenu: widget.menuOf == null ? null : _menu, view: view.name, onTap: choose, onOpen: useAsset, explore: widget.exploreLayout, faces: widget.faces, holding: widget.holding, minColumn: thumbMin, revealOn: preview),
              ),
            ),
          ]);
      });
    }
    switch (view) {
      case BrowserView.list:
        return MediaListView(items: items, selected: selected, onTap: choose, onOpen: open, sort: widget.sort, descending: widget.descending, onSort: widget.onSort ?? (_) {});
      case BrowserView.explore:
        final build = widget.explore;
        if (build != null) return build(context, items, selected, choose, open);
        // the map is laid out as a whole: past this many it asks for a narrower result set rather than drawing a hairball
        return Padding(padding: const EdgeInsets.all(9), child: Text('Explore maps up to ${widget.fluidUpTo} assets. Narrow the result (a Source, a Type, a word).', style: Dn.label(N.g56)));
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
