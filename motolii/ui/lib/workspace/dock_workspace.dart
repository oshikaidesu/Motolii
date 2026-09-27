import 'dart:math' as math;

import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../foundation/glyphs.dart' as legacy;
import '../hf/bp/common.dart' as tab;
import '../hf/glyphs.dart' show HG;

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
  DockWorkspace(this.defs, this._preset, {this.onDetach, this.allowClose = true}) {
    layout = DockingLayout(root: _preset(item));
  }

  /// Given, every tab gets a small menu with Detach (a panel of its own window) and Close.
  final void Function(String id)? onDetach;
  final bool allowClose;

  /// The default workspace: the arrangement the design started from.
  final DockingArea Function(DockingItem Function(String id, {double? weight}) item) _preset;

  final Map<String, PanelDef> defs;
  late final DockingLayout layout;
  final Map<String, GlobalKey> _keys = {};

  /// Bumped when a tab is picked by hand, which `docking` does not announce through the layout.
  final ValueNotifier<int> _picked = ValueNotifier(0);

  /// The dock itself, wired so a panel can tell whether it is in front. Use this, not a bare `Docking`.
  Widget view() => Docking(
    layout: layout,
    onItemSelection: (_) => _picked.value++,
    maximizableItem: false,
    maximizableTab: false,
    maximizableTabsArea: false,
  );

  DockingItem item(String id, {double? weight}) {
    final def = defs[id]!;
    final it = DockingItem(
      id: id,
      name: def.title,
      // `docking`'s own "×" is separate from our menu's Close item above and defaults to
      // shown; a panel set that never closes (`allowClose: false`) also does not need it,
      // and removing it frees the width a narrow tab strip needs for its own hidden-tabs
      // button (with it present, five tabs in one seat had no room left for that button).
      closable: allowClose,
      // The Browser family's own tab glyph (hf's HG set), not a borrowed icon font: one tab
      // language for every seat, Browser and Dock alike.
      leading: def.glyph == null
          ? null
          : (context, status) => SizedBox(
                width: 15,
                height: 15,
                child: CustomPaint(
                  painter: tab.HfTabGlyph(
                    def.glyph!,
                    status == TabStatus.selected ? const Color(0xFFF0F0F0) : tab.kMuted,
                  ),
                ),
              ),
      // A panel that decides whether it is on screen with Visibility.of (the Stage does, to hand a native surface back)
      // needs the dock to say so: a hidden tab is offstage but stays mounted.
      widget: KeyedSubtree(
        key: _keys.putIfAbsent(id, GlobalKey.new),
        child: Builder(
          builder: (_) => ListenableBuilder(
            listenable: Listenable.merge([layout, _picked]),
            builder: (context, child) => Visibility(
              visible: isShown(id),
              maintainState: true,
              child: LayoutBuilder(builder: (context, c) {
                _reportWidth(id, c.maxWidth);
                return child!;
              }),
            ),
            child: def.build(),
          ),
        ),
      ),
      weight: weight,
      minimalSize: def.minSize,
      keepAlive: true,
      buttons: [
        if (onDetach != null)
          TabButton(
            icon: IconProvider.data(legacy.Glyph.more_horiz),
            toolTip: 'Panel',
            menuBuilder: (context) => [
              TabbedViewMenuItem(text: 'Detach', onSelection: () => onDetach!(id)),
              if (allowClose) TabbedViewMenuItem(text: 'Close', onSelection: () => close(id)),
              TabbedViewMenuItem(text: 'Reset Layout', onSelection: reset),
            ],
          ),
      ],
    );
    _fullTitle[id] = def.title;
    return it;
  }

  /// The Browser Leaf's own rule (`hf/bp/common.dart`): identity travels, fold don't miniaturize. A tab strip
  /// too narrow for every title keeps the front tab's word and lets the rest fall back to their icon — nothing
  /// shrinks its type to fit. The strip's width is the same as its (visible) content's, so the content's own
  /// layout tells us it, one seat at a time.
  final Map<String, String> _fullTitle = {};
  final Map<DockingTabs, double> _seatWidth = {};
  final Set<DockingTabs> _relabelPending = {};
  void _reportWidth(String id, double width) {
    final tabs = layout.findDockingTabsWithItem(id);
    if (tabs == null) return;
    // Re-run on every build of the visible (front) tab: a width change needs it, but so does a bare tab switch
    // at the same width — the newly-front tab must pick up its own name back from whichever tab folded to make
    // room for it before.
    _seatWidth[tabs] = width;
    if (_relabelPending.add(tabs)) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _relabelPending.remove(tabs);
        _relabel(tabs);
      });
    }
  }

  void _relabel(DockingTabs tabs) {
    final width = _seatWidth[tabs];
    if (width == null) return;
    final ids = [for (var i = 0; i < tabs.childrenCount; i++) tabs.childAt(i).id as String];
    if (ids.isEmpty) return;
    final selected = ids[tabs.selectedIndex.clamp(0, ids.length - 1)];
    final need = ids.fold<double>(0, (a, id) => a + tab.Leaf.labelled(_fullTitle[id] ?? id));
    final fits = need <= width - 30 && width >= 110;
    var changed = false;
    for (final id in ids) {
      final it = layout.findDockingItem(id);
      if (it == null) continue;
      final compact = !fits && (id != selected || width < 110);
      final wanted = compact ? '' : (_fullTitle[id] ?? id);
      if (it.name != wanted) {
        it.name = wanted;
        changed = true;
      }
    }
    if (changed) layout.rebuild();
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
  void activate(String id, {String near = 'Stage'}) {
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
      activate(id, near: near); // now docked: bring its tab to the front
      return;
    }
    final anchor = layout.findDockingItem(near) ?? layout.findDockingItem(defs.keys.first)!;
    layout.addItemOn(newItem: item(id), targetArea: anchor, dropPosition: DropPosition.right);
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
