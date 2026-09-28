import 'dart:math' as math;

import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../foundation/glyphs.dart' as legacy;
import '../hf/bp/common.dart' as tab;
import '../hf/glyphs.dart' show HG;
import '../hf/metrics.dart';
import '../hf/shell/menu.dart' show showHfMenu;

/// One panel the workspace can show: an id that stays the same, the words on its tab, its family icon
/// (the same glyph a Browser tab would use), and how to build its body.
class PanelDef {
  const PanelDef(this.id, this.title, this.build, {this.glyph, this.minSize = 160});
  final String id, title;
  final HG? glyph;
  final Widget Function() build;
  final double minSize;
}

/// The New shell's workspace: the off-the-shelf `docking` layout plus what only Motolii knows around it,
/// which is the panel registry, showing a panel by id, and where a panel is on screen.
class DockWorkspace {
  DockWorkspace(this.defs, this._preset, {this.onDetach, this.allowClose = true, this.seats = false, this.closable}) {
    layout = DockingLayout(root: _preset(item));
  }

  /// Given, every tab gets a small menu with Detach (a panel of its own window) and Close.
  final void Function(String id)? onDetach;
  final bool allowClose;

  /// When [allowClose] is off, the panels that may still be closed (opened on demand, not part of the default).
  final bool Function(String id)? closable;
  bool _canClose(String id) => allowClose || (closable?.call(id) ?? false);

  /// Every seat draws the Browser's seat strip and a tab is the panel's handle (drag to travel, right click for its
  /// menu); `docking`'s own strip must then be off (`hfDockTabs`). Off: `docking`'s own tabs with a Detach/Close menu —
  /// the New shell, whose theme (`shellTabs`) keeps that strip.
  final bool seats;

  /// The default workspace: the arrangement the design started from.
  final DockingArea Function(DockingItem Function(String id, {double? weight}) item) _preset;

  final Map<String, PanelDef> defs;
  late final DockingLayout layout;
  final Map<String, GlobalKey> _keys = {};

  /// Bumped when a tab is picked by hand, which `docking` does not announce through the layout.
  final ValueNotifier<int> _picked = ValueNotifier(0);

  /// The dock itself, wired so a panel can tell whether it is in front. Use this, not a bare `Docking`.
  Widget view() => !seats
      ? Docking(
          layout: layout,
          onItemSelection: (_) => _picked.value++,
          maximizableItem: false,
          maximizableTab: false,
          maximizableTabsArea: false,
        )
      : Stack(
    key: _viewKey,
    fit: StackFit.expand,
    children: [
      Positioned.fill(
        child: Docking(
          layout: layout,
          onItemSelection: (_) => _picked.value++,
          maximizableItem: false,
          maximizableTab: false,
          maximizableTabsArea: false,
        ),
      ),
      // While a panel travels: every seat's strip is a place to join it as a tab. It lies above `docking`'s own edge
      // zones (which cover the whole seat, strip included) and exists only during the drag.
      ValueListenableBuilder<bool>(
        valueListenable: _dragging,
        builder: (context, dragging, _) => !dragging ? const SizedBox.shrink() : Positioned.fill(child: _stripTargets()),
      ),
    ],
  );

  final _viewKey = GlobalKey();
  final ValueNotifier<bool> _dragging = ValueNotifier(false);

  Widget _stripTargets() {
    final origin = (_viewKey.currentContext?.findRenderObject() as RenderBox?)?.localToGlobal(Offset.zero) ?? Offset.zero;
    return Stack(children: [
      for (final id in defs.keys)
        if (isOpen(id) && isShown(id))
          if (rectOf(id) case final r?)
            Positioned(
              left: r.left - origin.dx,
              top: r.top - origin.dy,
              width: r.width,
              height: UiMetrics.chromeRow,
              child: DragTarget<DraggableData>(
                onWillAcceptWithDetails: (d) => d.data.tabData.value is DockingItem && (d.data.tabData.value as DockingItem).id != id,
                onAcceptWithDetails: (d) => _join(d.data.tabData.value as DockingItem, id),
                builder: (context, over, _) => DecoratedBox(
                  decoration: BoxDecoration(color: over.isEmpty ? const Color(0x00000000) : const Color(0x33F0F0F0)),
                ),
              ),
            ),
    ]);
  }

  DockingItem item(String id, {double? weight}) {
    final def = defs[id]!;
    return DockingItem(
      id: id,
      name: def.title,
      closable: _canClose(id),
      // A panel that decides whether it is on screen with Visibility.of (the Stage does, to hand a native surface back)
      // needs the dock to say so: a hidden tab is offstage but stays mounted.
      widget: KeyedSubtree(
        key: _keys.putIfAbsent(id, GlobalKey.new),
        child: Builder(
          builder: (_) => ListenableBuilder(
            listenable: Listenable.merge([layout, _picked]),
            builder: (context, child) => Visibility(visible: isShown(id), maintainState: true, child: seats ? _seat(id, child!) : child!),
            child: def.build(),
          ),
        ),
      ),
      weight: weight,
      minimalSize: def.minSize,
      keepAlive: true,
      buttons: [
        if (!seats && onDetach != null)
          TabButton(
            icon: IconProvider.data(legacy.Glyph.more_horiz),
            toolTip: 'Panel',
            menuBuilder: (context) => [
              TabbedViewMenuItem(text: 'Detach', onSelection: () => onDetach!(id)),
              if (_canClose(id)) TabbedViewMenuItem(text: 'Close', onSelection: () => close(id)),
            ],
          ),
      ],
    );
  }

  /// The unit of the workspace is the seat, the Browser's own (`hf/bp/common.dart` Leaf): panels sharing a seat,
  /// a strip only when there is more than one, the front tab keeps its word and the rest fold to their glyph, and
  /// a stacked panel's header drops the name its tab already shows. `docking` keeps layout, split, resize and drop
  /// only (its own tab strip is off in `hfDockTabs`). Every panel of a seat draws the seat's strip; only the front
  /// one is on screen.
  Widget _seat(String id, Widget body) {
    final tabs = layout.findDockingTabsWithItem(id);
    final ids = tabs == null ? [id] : [for (var i = 0; i < tabs.childrenCount; i++) tabs.childAt(i).id as String];
    return tab.Leaf(
      tabs: [for (final p in ids) tab.TabSpec(defs[p]!.title, defs[p]!.glyph ?? HG.list)],
      active: tabs == null ? 0 : math.min(tabs.selectedIndex, ids.length - 1),
      onTab: tabs == null
          ? null
          : (i) {
              tabs.selectedIndex = i;
              layout.rebuild();
              _picked.value++;
            },
      tabWrap: (i, t) => _handle(ids[i], t),
      body: body,
    );
  }

  /// A tab is also the panel's handle: drag it to another seat or a seat's edge (`docking`'s own drop zones, fed the
  /// same drag data its own tabs would give), right click for the panel menu.
  /// A tab is also the panel's handle and a place to land: drag it to a seat's edge (`docking`'s own drop zones, fed
  /// the drag data its own tabs would give) to split, onto a seat's strip to join that seat (`_stripTargets`); right
  /// click for the panel menu. The same for every panel — travel is the Dock's, not a surface's.
  Widget _handle(String id, Widget tabFace) {
    final item = layout.findDockingItem(id);
    if (item == null) return tabFace;
    final data = TabData(value: item, text: defs[id]!.title);
    return Draggable<DraggableData>(
      data: DraggableData(TabbedViewController([data]), data),
      feedback: Opacity(opacity: .85, child: tabFace),
      childWhenDragging: Opacity(opacity: .4, child: tabFace),
      onDragStarted: () => _dragging.value = true,
      onDragEnd: (_) => _dragging.value = false,
      child: Builder(
        builder: (context) => GestureDetector(
          onSecondaryTapDown: (e) => _menu(context, id, e.globalPosition),
          child: tabFace,
        ),
      ),
    );
  }

  /// [moving] joins the seat [onto] is in, just before it.
  void _join(DockingItem moving, String onto) {
    final tabs = layout.findDockingTabsWithItem(onto);
    if (tabs != null && tabs == layout.findDockingTabsWithItem(moving.id)) return; // already in that seat
    final DropArea target = tabs ?? layout.findDockingItem(onto)!;
    layout.moveItem(draggedItem: moving, targetArea: target, dropIndex: tabs?.childrenCount ?? 1);
    activate(moving.id as String);
  }

  /// Another panel of the seat [id] is in, else a panel beside it: where it goes back to after a detached window.
  String? neighbour(String id) {
    final tabs = layout.findDockingTabsWithItem(id);
    if (tabs != null) {
      for (var i = 0; i < tabs.childrenCount; i++) {
        if (tabs.childAt(i).id != id) return tabs.childAt(i).id as String;
      }
    }
    return null;
  }

  Future<void> _menu(BuildContext context, String id, Offset at) async {
    final open = [for (final d in defs.keys) if (!isOpen(d)) d];
    final pick = await showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 0, 0), [
      if (onDetach != null) ('detach', 'Detach'),
      if (_canClose(id)) ('close', 'Close'),
      for (final d in open) ('open:$d', 'Open ${defs[d]!.title}'),
      ('reset', 'Reset Layout'),
    ]);
    if (pick == 'detach') onDetach?.call(id);
    if (pick == 'close') close(id);
    if (pick == 'reset') reset();
    if (pick != null && pick.startsWith('open:')) activate(pick.substring(5), near: id);
  }

  bool isOpen(String id) => layout.findDockingItem(id) != null;

  /// The tab in front. Closing the last tab of a strip leaves the stored index past the end, and the dock
  /// clamps it only when it draws, so every reader here does the same.
  DockingItem _front(DockingTabs tabs) => tabs.childAt(math.min(tabs.selectedIndex, tabs.childrenCount - 1));

  /// In front: alone in its area, or the selected tab of its strip.
  bool isShown(String id) {
    final tabs = layout.findDockingTabsWithItem(id);
    if (tabs == null) return isOpen(id);
    return _front(tabs).id == id;
  }

  /// Show a panel: select its tab if it is docked, otherwise reopen it beside [near].
  void activate(String id, {String near = 'Stage', DropPosition side = DropPosition.right}) {
    if (!defs.containsKey(id)) return;
    final tabs = layout.findDockingTabsWithItem(id);
    if (tabs != null) {
      tabs.selectedIndex = [
        for (var i = 0; i < tabs.childrenCount; i++) tabs.childAt(i).id,
      ].indexOf(id);
      layout.rebuild();
      return;
    }
    if (isOpen(id)) return;
    final nearTabs = layout.findDockingTabsWithItem(near);
    if (nearTabs != null) {
      // Beside a panel that is one tab of a strip: join that strip.
      layout.addItemOn(newItem: item(id), targetArea: nearTabs, dropIndex: nearTabs.childrenCount);
      activate(id, near: near, side: side); // now docked: bring its tab to the front
      return;
    }
    final anchor = layout.findDockingItem(near) ?? layout.findDockingItem(defs.keys.first)!;
    layout.addItemOn(newItem: item(id), targetArea: anchor, dropPosition: side);
  }

  void close(String id) => layout.removeItemByIds([id]);

  /// Back to the default arrangement. Panels keep their state: their keys are the same.
  void reset() {
    layout.root = _preset(item);
  }

  /// Where a panel is on screen now, in global coordinates.
  Rect? rectOf(String id) {
    final box = _keys[id]?.currentContext?.findRenderObject() as RenderBox?;
    if (box == null || !box.attached) return null;
    return box.localToGlobal(Offset.zero) & box.size;
  }

  /// The saved form of the workspace: dock structure and sizes as `docking` writes them, plus which tab is in front
  /// in every strip (`docking` does not write that). Versioned so a later shape can replace this one.
  static const stateVersion = 1;

  Map<String, dynamic> snapshot() {
    final root = layout.root;
    final shown = <String>[];
    void walk(DockingArea? area) {
      if (area is DockingTabs) {
        shown.add(_front(area).id as String);
      } else if (area is DockingParentArea) {
        area.forEach(walk);
      }
    }

    walk(root);
    return {'version': stateVersion, 'layout': layout.stringify(parser: const _Ids()), 'shown': shown};
  }

  /// Put a saved workspace back. A state this build cannot read (another version, an id that is no longer a panel,
  /// a damaged string) leaves the current layout as it is and answers false.
  bool restore(Object? saved) {
    if (saved is! Map || saved['version'] != stateVersion || saved['layout'] is! String) return false;
    final text = saved['layout'] as String;
    try {
      // Read it into a scratch layout first: `docking` takes the old tree apart while loading, so a state that
      // fails half way could not be put back.
      DockingLayout().load(layout: text, parser: const _Ids(), builder: _Build(this));
    } catch (_) {
      return false;
    }
    layout.load(layout: text, parser: const _Ids(), builder: _Build(this));
    for (final id in (saved['shown'] as List? ?? const []).whereType<String>()) {
      final tabs = layout.findDockingTabsWithItem(id);
      if (tabs == null) continue;
      tabs.selectedIndex = [for (var i = 0; i < tabs.childrenCount; i++) tabs.childAt(i).id].indexOf(id);
    }
    layout.rebuild();
    return true;
  }
}

class _Ids extends LayoutParser with LayoutParserMixin {
  const _Ids();
}

class _Build extends AreaBuilder with AreaBuilderMixin {
  const _Build(this.workspace);
  final DockWorkspace workspace;
  @override
  DockingItem buildDockingItem({required id, required double? weight, required bool maximized}) {
    if (!workspace.defs.containsKey(id)) throw StateError('No panel called $id');
    return workspace.item('$id', weight: weight);
  }

  @override
  DockingTabs buildDockingTabs({required id, required double? weight, required bool maximized, required List<DockingItem> children}) =>
      DockingTabs(children, id: id, weight: weight);
}
