import 'dart:math' as math;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';
import 'browser/colors_shelf.dart';
import 'browser/create_shelf.dart';
import 'browser/effects_shelf.dart';
import 'browser/files_shelf.dart';
import 'browser/filters.dart';
import 'browser/fonts_shelf.dart';
import 'browser/media_shelf.dart';
import 'browser/parts.dart';
import 'browser/shelf.dart';
import 'browser/tile.dart';
import '../foundation/glyphs.dart';
import '../foundation/leaves.dart';

export 'browser/colors_shelf.dart' show paletteOf;

/// The Browser is one frame — tabs, search, rail, grid, selection, keys —
/// and a row of shelves. Each shelf is one file under `browser/`; the frame
/// never names a tab, it asks the shelf in front.
class BrowserPanel extends StatefulWidget {
  const BrowserPanel({
    super.key,
    required this.controller,
    this.fixedTab,
    this.showTabs = true,
    this.initialFolder,
  });
  final String? fixedTab;
  final bool showTabs;

  /// Where the Files tab opens; the home folder when unset.
  final String? initialFolder;
  final EditorSession controller;
  @override
  State<BrowserPanel> createState() => _BrowserPanelState();
}

class _BrowserPanelState extends State<BrowserPanel> implements BrowserHost {
  late final List<BrowserShelf> shelves = [
    CreateShelf(),
    MediaShelf(),
    EffectsShelf(),
    FontsShelf(),
    ColorsShelf(),
    FilesShelf(initialFolder: widget.initialFolder),
  ];
  @override
  String tab = 'Create';
  @override
  BrowserShelf get shelf => shelves.firstWhere((s) => s.name == tab);

  final classifications = <String, String>{};
  final search = TextEditingController();
  final searchFocus = FocusNode();
  final panelFocus = FocusNode();
  final queries = <String, String>{};
  final active = <String, String>{};
  final selected = <String, Set<String>>{};
  final scroll = ScrollController();
  @override
  List<Map<String, dynamic>> visible = [];

  /// Tags, collections and labels the user keeps (Live 12's browser, 4.4–4.5).
  late final library = BrowserLibrary(widget.controller);
  final filters = <String, ShelfFilter>{};
  final filtersShown = <String, bool>{};
  final quickAdd = TextEditingController();
  final quickAddFocus = FocusNode();
  ShelfFilter get filter => filters.putIfAbsent(tab, ShelfFilter.new);

  /// The groups with their tags filled in: declared as given, values from
  /// what the items carry (sorted, numbers first), ranges from the user (or
  /// the shelf's seeds until the user keeps their own), then the user's tags.
  List<FilterGroup> get groups {
    final items = shelf.items(this);
    return [
      for (final g in shelf.groups(this))
        switch (g.kind) {
          FilterKind.declared => g,
          FilterKind.actual => FilterGroup(
            g.name,
            _sortedValues({
              for (final i in items) ?shelf.valueOf(this, i, g.name),
            }),
            kind: g.kind,
            unit: g.unit,
          ),
          FilterKind.range => FilterGroup(
            g.name,
            library.rangesOn(tab, g.name) ?? g.tags,
            kind: g.kind,
            unit: g.unit,
          ),
        },
      if (library.tagsOn(tab).isNotEmpty)
        FilterGroup(userTagGroup, library.tagsOn(tab)),
    ];
  }

  static List<String> _sortedValues(Set<String> values) {
    final list = values.toList();
    list.sort((a, b) {
      final x = double.tryParse(a.split('×').first),
          y = double.tryParse(b.split('×').first);
      if (x != null && y != null && x != y) return x.compareTo(y);
      return a.compareTo(b);
    });
    return list;
  }

  bool _matches(FilterGroup g, Map<String, dynamic> item, Set<String> chosen) =>
      switch (g.kind) {
        FilterKind.declared => chosen.any(tagsOf(item).contains),
        FilterKind.actual => chosen.contains(shelf.valueOf(this, item, g.name)),
        FilterKind.range => switch (shelf.numberOf(this, item, g.name)) {
          null => false,
          final v => chosen.any((r) => inRange(r, v)),
        },
      };
  Set<String> tagsOf(Map<String, dynamic> item) => {
    ...shelf.tagsOf(this, item),
    ...library.tagsOf(tab, id(item)),
  };

  /// Groups combine with AND, tags within one with OR; a collection is one
  /// more AND. [except] leaves one group out (to count what it would keep).
  bool _passesFilter(Map<String, dynamic> item, {String? except}) {
    final f = filter;
    if (f.collection != null &&
        library.collectionOf(tab, id(item)) != f.collection)
      return false;
    final kinds = {for (final g in shelf.groups(this)) g.name: g};
    for (final e in f.groups.entries) {
      if (e.value.isEmpty || e.key == except) continue;
      final g = kinds[e.key] ?? FilterGroup(e.key, const []);
      if (!_matches(g, item, e.value)) return false;
    }
    return true;
  }

  void _toggleTag(String group, String tag, bool add) {
    final set = filter.groups.putIfAbsent(group, () => <String>{});
    if (add) {
      set.contains(tag) ? set.remove(tag) : set.add(tag);
    } else if (set.length == 1 && set.contains(tag)) {
      set.clear();
    } else {
      set
        ..clear()
        ..add(tag);
    }
    setState(_derive);
  }

  void _saveLabel() {
    final name = filter.describe();
    if (name.isEmpty) return;
    library.saveLabel(tab, {
      'name': name,
      'filter': filter.toJson(),
      'rail': _chosen,
      'query': search.text,
    });
  }

  void _restoreLabel(Map<String, dynamic> label) {
    setState(() {
      filter.restore(EditorSession.map(label['filter']));
      classifications[tab] = '${label['rail'] ?? 'All'}';
      search.text = '${label['query'] ?? ''}';
      filtersShown[tab] = true;
      _derive();
    });
  }

  /// The current tab's selection as a signal: [selected] is the store, this
  /// is what a tile and the count watch, so a pick redraws only them.
  final picked = ValueNotifier<Set<String>>(const {});
  late final DocumentSlice _slice;
  int columns = 1;

  /// One tile's width at the shelf's current size. The grid already knows it,
  /// so a card's name is fitted against this instead of measuring itself.
  @override
  double tileWidth = BrowserSize.base;

  @override
  int get viewMode =>
      (widget.controller.deskWork.value['browserView'] as num? ??
              shelf.defaultView)
          .toInt();

  /// Category rail width while dragging; null means "as stored".
  double? railDrag;

  static const double tileDefault = BrowserSize.base, railMin = 48;

  /// Grid: the gutter between tiles, and the single caption line under every
  /// preview. Nothing else shares that line, so the name keeps the tile's
  /// whole width; format and status ride on the picture instead.
  static const double _gutter = EditorMetrics.s2,
      _inset = EditorMetrics.s8,
      _captionHeight = EditorMetrics.control;
  int total = 0;

  double get tile => BrowserSize.tile(widget.controller);

  /// The tile is one drawing that scales: caption, badge and airs keep
  /// their proportion to the picture at every size. Only the type has a
  /// floor, below which a name stops being legible.
  @override
  double get tileScale => tile / BrowserSize.base;
  double get captionHeight => _captionHeight * tileScale;
  double get rail =>
      railDrag ??
      (widget.controller.deskWork.value['browserRail'] as num? ??
              EditorMetrics.s96)
          .toDouble();

  // ---- BrowserHost: what a shelf may ask of the frame.
  @override
  EditorSession get controller => widget.controller;
  @override
  bool has(String op) =>
      (widget.controller.state['capabilities'] as List? ?? []).contains(op);
  @override
  String id(Map<String, dynamic> item) => '${item['id'] ?? item['hex']}';
  @override
  Set<String> get selectedIds => selected[tab] ?? const {};
  @override
  void refresh([VoidCallback? change]) => setState(change ?? () {});
  @override
  void relist() => setState(_derive);
  @override
  void clearSelection() {
    selected[tab]?.clear();
    _publish();
  }

  @override
  void showCategory(String shelf, String category) {
    classifications[shelf] = category;
    setState(_derive);
  }

  @override
  void initState() {
    super.initState();
    tab = widget.fixedTab ?? 'Create';
    widget.controller.importedAssets.addListener(_revealImported);
    widget.controller.browserTab.addListener(_revealTab);
    widget.controller.deskWork.addListener(_redraw);
    _slice = widget.controller.slice(
      'browser',
      const [
        'assets',
        'backgrounds',
        'primitives',
        'catalog',
        'palette',
        'colorTarget',
        'importExtensions',
        'capabilities',
      ],
      // The frame reads the selection once: whether there is anything to
      // apply to. Each shelf names what else it derives, so the shelves keep
      // still while layers are picked.
      derived: () => [
        widget.controller.selectedIds.isEmpty,
        for (final s in shelves) ...s.derived(widget.controller),
      ],
    );
    _slice.addListener(_onDocument);
    _enter();
  }

  void _onDocument() {
    if (mounted) setState(_derive);
  }

  /// The desk keys the frame and its shelves read; a write to any other
  /// leaves the panel still.
  List<Object?>? _deskSeen;
  void _redraw() {
    final d = widget.controller.deskWork.value;
    final now = [
      for (final k in [
        'browserView',
        'browserTile',
        'browserRail',
        'tags',
        'collections',
        'labels',
        'ranges',
        'folds',
        'collectionNames',
        for (final s in shelves) ...s.deskKeys,
      ])
        d[k],
    ];
    if (listEquals(now, _deskSeen) || !mounted) return;
    _deskSeen = now;
    setState(_derive);
  }

  /// Every tab opens on its own list and selection.
  void _enter() {
    shelf.enter(this);
    _derive();
    _publish();
  }

  /// The rail entry in force.
  String get _chosen => shelf.chosen(this, classifications[tab] ?? 'All');

  /// The list as the shelf shows it, held rather than computed on draw: a
  /// pick or a hover redraws its tile and nothing here.
  void _derive() {
    final all = shelf.items(this);
    final chosen = _chosen;
    final query = search.text.trim().toLowerCase();
    total = all.length;
    visible = all.where((item) {
      final label = '${item['name'] ?? item['hex'] ?? item['id']}'
          .toLowerCase();
      return label.contains(query) &&
          shelf.passes(this, item, chosen) &&
          _passesFilter(item);
    }).toList();
  }

  void _publish() =>
      picked.value = Set.unmodifiable(selected[tab] ?? const <String>{});

  /// 取り込んだ物は Media に居る。開いて、絞り込みを外して、選んでおく。
  void _revealImported() {
    final ids = widget.controller.importedAssets.value;
    if (ids.isEmpty || !mounted) return;
    if (widget.fixedTab != null && widget.fixedTab != 'Media') return;
    setState(() {
      queries[tab] = search.text;
      tab = 'Media';
      search.text = '';
      queries['Media'] = '';
      classifications['Media'] = 'All';
      selected['Media'] = ids.toSet();
      active['Media'] = ids.first;
      _derive();
      _publish();
    });
  }

  @override
  void dispose() {
    widget.controller.importedAssets.removeListener(_revealImported);
    widget.controller.browserTab.removeListener(_revealTab);
    widget.controller.deskWork.removeListener(_redraw);
    _slice.removeListener(_onDocument);
    for (final s in shelves) s.dispose();
    picked.dispose();
    quickAdd.dispose();
    quickAddFocus.dispose();
    search.dispose();
    searchFocus.dispose();
    panelFocus.dispose();
    scroll.dispose();
    super.dispose();
  }

  /// A colour row's swatch asks for the Colors shelf; the wheel follows.
  void _revealTab() {
    final value = widget.controller.browserTab.value;
    if (!mounted || widget.fixedTab != null || tab == value) return;
    changeTab(value);
  }

  void changeTab(String value) {
    setState(() {
      queries[tab] = search.text;
      tab = value;
      search.text = queries[value] ?? '';
      _enter();
    });
  }

  @override
  void select(Map<String, dynamic> item) {
    panelFocus.requestFocus();
    final key = id(item);
    final multi = shelf.multiSelect;
    final modifiers = HardwareKeyboard.instance;
    final chosen = selected.putIfAbsent(tab, () => <String>{});
    if (multi && modifiers.isShiftPressed && active[tab] != null) {
      final from = visible.indexWhere((e) => id(e) == active[tab]);
      final to = visible.indexWhere((e) => id(e) == key);
      if (from >= 0 && to >= 0) {
        if (!modifiers.isMetaPressed && !modifiers.isControlPressed)
          chosen.clear();
        chosen.addAll(
          visible.sublist(math.min(from, to), math.max(from, to) + 1).map(id),
        );
      }
    } else if (multi &&
        (modifiers.isMetaPressed || modifiers.isControlPressed)) {
      chosen.contains(key) ? chosen.remove(key) : chosen.add(key);
    } else {
      chosen
        ..clear()
        ..add(key);
    }
    active[tab] = key;
    _publish();
  }

  @override
  Future<void> apply(Map<String, dynamic> item) => shelf.apply(this, item);

  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent ||
        searchFocus.hasFocus ||
        FocusManager.instance.primaryFocus?.context
                ?.findAncestorWidgetOfExactType<EditableText>() !=
            null)
      return KeyEventResult.ignored;
    final k = event.logicalKey;
    final primary =
        HardwareKeyboard.instance.isMetaPressed ||
        HardwareKeyboard.instance.isControlPressed;
    if (primary && k == LogicalKeyboardKey.keyF) {
      searchFocus.requestFocus();
      search.selection = TextSelection(
        baseOffset: 0,
        extentOffset: search.text.length,
      );
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.escape) {
      if (search.text.isNotEmpty) {
        search.clear();
        setState(_derive);
      } else {
        selected[tab]?.clear();
        active.remove(tab);
        _publish();
      }
      return KeyEventResult.handled;
    }
    if (primary && k == LogicalKeyboardKey.keyE) {
      if (selectedIds.isNotEmpty) quickAddFocus.requestFocus();
      return KeyEventResult.handled;
    }
    final digit = k.keyLabel.length == 1 ? int.tryParse(k.keyLabel) : null;
    if (!primary &&
        digit != null &&
        digit <= BrowserLibrary.collectionCount &&
        selectedIds.isNotEmpty) {
      library.collect(tab, selectedIds.toList(), digit);
      return KeyEventResult.handled;
    }
    if (primary && k == LogicalKeyboardKey.keyA && shelf.multiSelect) {
      selected[tab] = visible.map(id).toSet();
      _publish();
      return KeyEventResult.handled;
    }
    if (visible.isEmpty) return KeyEventResult.ignored;
    final current = visible.indexWhere((e) => id(e) == active[tab]);
    if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
      apply(visible[current < 0 ? 0 : current]);
      return KeyEventResult.handled;
    }
    int? next;
    if (k == LogicalKeyboardKey.arrowLeft) next = current - 1;
    if (k == LogicalKeyboardKey.arrowRight) next = current + 1;
    if (k == LogicalKeyboardKey.arrowUp) next = current - columns;
    if (k == LogicalKeyboardKey.arrowDown) next = current + columns;
    if (k == LogicalKeyboardKey.home) next = 0;
    if (k == LogicalKeyboardKey.end) next = visible.length - 1;
    if (next != null) {
      select(visible[next.clamp(0, visible.length - 1)]);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) {
      shelf.delete(this, visible[current < 0 ? 0 : current]);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final rails = shelf.rails(this);
    final chosen = _chosen;
    final tools = shelf.tools(this);
    final header = shelf.header(this);
    final editor = shelf.editor(this);
    final overlay = shelf.overlay(this);
    return Focus(
      focusNode: panelFocus,
      onKeyEvent: key,
      child: ColoredBox(
        color: EditorTheme.panel,
        child: Stack(
          fit: StackFit.expand,
          children: [
            Column(
              children: [
                if (widget.showTabs)
                  SizedBox(
                    height: EditorMetrics.section,
                    child: Row(
                      children: [
                        for (final s in shelves)
                          Expanded(
                            child: shelfButton(
                              s.name,
                              () => changeTab(s.name),
                              selected: s.name == tab,
                            ),
                          ),
                      ],
                    ),
                  ),
                _BrowserSearchBar(
                  tab: tab,
                  search: search,
                  searchFocus: searchFocus,
                  tools: tools,
                  filterable: groups.isNotEmpty,
                  filtering: (filtersShown[tab] ?? false) || !filter.isEmpty,
                  views: shelf.showViews
                      ? shelfViews(widget.controller, viewMode)
                      : null,
                  onChanged: () => setState(_derive),
                  onToggleFilters: () => setState(
                    () => filtersShown[tab] = !(filtersShown[tab] ?? false),
                  ),
                ),
                if (header != null) header,
                Expanded(
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      if (rail >= railMin)
                        _BrowserRail(
                          width: rail,
                          tab: tab,
                          rails: rails,
                          chosen: chosen,
                          onRail: (entry) => shelf.rail(this, entry),
                          collections: RailCollections(
                            chosen: filter.collection,
                            labels: library.labelsOn(tab),
                            onCollection: (i) => setState(() {
                              filter.collection = filter.collection == i
                                  ? null
                                  : i;
                              _derive();
                            }),
                            onLabel: _restoreLabel,
                            onDropLabel: (name) => library.dropLabel(tab, name),
                            onDrop: (which, ids) =>
                                library.collect(tab, ids, which),
                            names: [
                              for (
                                var i = 1;
                                i <= BrowserLibrary.collectionCount;
                                i++
                              )
                                library.collectionName(i),
                            ],
                            onRename: library.renameCollection,
                          ),
                        ),
                      if (rail < railMin)
                        _BrowserRailTab(
                          tab: tab,
                          label: chosen == rails.first ? tab : chosen,
                          onTap: () => widget.controller.storeDesk(
                            'browserRail',
                            EditorMetrics.s96,
                          ),
                        ),
                      shelfGrip(
                        key: const ValueKey('browser:rail-grip'),
                        onDrag: (dx) => setState(
                          () => railDrag = (rail + dx).clamp(
                            0.0,
                            EditorMetrics.s200,
                          ),
                        ),
                        onEnd: () {
                          final width = rail < railMin ? 0.0 : rail;
                          railDrag = null;
                          widget.controller.storeDesk('browserRail', width);
                        },
                        onDoubleTap: () => widget.controller.storeDesk(
                          'browserRail',
                          rail >= railMin ? 0.0 : EditorMetrics.s96,
                        ),
                      ),
                      Expanded(
                        child: LayoutBuilder(
                          builder: (context, constraints) {
                            final custom = shelf.layout(
                              this,
                              constraints.maxWidth,
                              tile,
                            );
                            columns = math.max(
                              1,
                              (constraints.maxWidth / (custom?.column ?? tile))
                                  .floor(),
                            );
                            if (viewMode == 1 && !shelf.bare) columns = 1;
                            final gap = custom?.gap ?? _gutter;
                            tileWidth =
                                (constraints.maxWidth - gap * (columns - 1)) /
                                columns;
                            final extent =
                                custom?.extent ??
                                (viewMode == 1
                                    ? EditorMetrics.s48
                                    : tileWidth * 9 / 16 +
                                          (viewMode == 2 ? 0 : captionHeight));
                            return Column(
                              crossAxisAlignment: CrossAxisAlignment.stretch,
                              children: [
                                if ((filtersShown[tab] ?? false) &&
                                    groups.isNotEmpty)
                                  ConstrainedBox(
                                    constraints: BoxConstraints(
                                      maxHeight: constraints.maxHeight * .5,
                                    ),
                                    child: FilterView(
                                      groups: groups,
                                      filter: filter,
                                      folded: library.foldsOn(tab),
                                      results: visible.length,
                                      onFold: (group) {
                                        final f = library.foldsOn(tab);
                                        f.contains(group)
                                            ? f.remove(group)
                                            : f.add(group);
                                        library.setFolds(tab, f);
                                      },
                                      onToggle: _toggleTag,
                                      onClear: () => setState(() {
                                        filter.clear();
                                        _derive();
                                      }),
                                      onSaveLabel: _saveLabel,
                                      onAddRange: (group, range) {
                                        final have =
                                            library.rangesOn(tab, group) ??
                                            shelf
                                                .groups(this)
                                                .firstWhere(
                                                  (g) => g.name == group,
                                                )
                                                .tags;
                                        library.setRanges(tab, group, [
                                          ...have.where((r) => r != range),
                                          range,
                                        ]);
                                      },
                                      onDropRange: (group, range) {
                                        final have =
                                            library.rangesOn(tab, group) ??
                                            shelf
                                                .groups(this)
                                                .firstWhere(
                                                  (g) => g.name == group,
                                                )
                                                .tags;
                                        filter.groups[group]?.remove(range);
                                        library.setRanges(
                                          tab,
                                          group,
                                          have
                                              .where((r) => r != range)
                                              .toList(),
                                        );
                                      },
                                    ),
                                  ),
                                if (editor != null) editor,
                                Expanded(
                                  child: visible.isEmpty
                                      ? const _NoMatches()
                                      : ColoredBox(
                                          color:
                                              custom?.ground ??
                                              EditorTheme.line,
                                          child: GridView.builder(
                                            controller: scroll,
                                            padding: EdgeInsets.all(
                                              custom?.padding ?? 0,
                                            ),
                                            gridDelegate:
                                                SliverGridDelegateWithFixedCrossAxisCount(
                                                  crossAxisCount: columns,
                                                  mainAxisExtent: extent,
                                                  crossAxisSpacing: gap,
                                                  mainAxisSpacing: gap,
                                                ),
                                            itemCount: visible.length,
                                            itemBuilder: (context, index) =>
                                                shelf.bare
                                                ? Align(
                                                    alignment:
                                                        Alignment.topLeft,
                                                    child: SizedBox(
                                                      width: double.infinity,
                                                      height: extent,
                                                      child: card(
                                                        visible[index],
                                                      ),
                                                    ),
                                                  )
                                                : card(visible[index]),
                                          ),
                                        ),
                                ),
                                _quickTags(),
                                _zoomBar(),
                              ],
                            );
                          },
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            if (overlay != null) overlay,
          ],
        ),
      ),
    );
  }

  /// The picked rows' tags: theirs (quiet), the user's (removable), Add….
  Widget _quickTags() => ValueListenableBuilder<Set<String>>(
    valueListenable: picked,
    builder: (context, chosen, _) => chosen.isEmpty
        ? const SizedBox()
        : _quickTagsFor(visible.where((i) => chosen.contains(id(i))).toList()),
  );

  Widget _quickTagsFor(List<Map<String, dynamic>> rows) {
    if (rows.isEmpty) return const SizedBox();
    Set<String>? builtin;
    final own = <String>{};
    for (final row in rows) {
      final theirs = shelf.tagsOf(this, row);
      builtin = builtin == null ? {...theirs} : builtin.intersection(theirs);
      own.addAll(library.tagsOf(tab, id(row)));
    }
    final title = rows.length == 1
        ? '${rows.single['name'] ?? id(rows.single)}'
        : '${rows.length} rows';
    return QuickTags(
      title: title,
      builtin: (builtin ?? const {}).toList()..sort(),
      own: own.toList(),
      addFocus: quickAddFocus,
      addController: quickAdd,
      onAdd: (tag) => library.tag(tab, rows.map(id), tag),
      onRemove: (tag) => library.untag(tab, rows.map(id), tag),
    );
  }

  /// Tile size as a percentage of its default.
  Widget _zoomBar() => Container(
    height: EditorMetrics.tall,
    padding: const EdgeInsets.symmetric(
      horizontal: _inset,
      vertical: EditorMetrics.s4,
    ),
    decoration: const BoxDecoration(
      border: Border(top: BorderSide(color: EditorTheme.line)),
    ),
    child: LayoutBuilder(
      builder: (context, box) {
        // 譲る順は 件数 → 寸法棒。寸法棒は、棒が出る幅ならその分、
        // 出ない幅でも押し所 2 つ分を必ず残す。
        final slider = box.maxWidth >= EditorMetrics.cell + EditorMetrics.s48;
        final countRoom =
            box.maxWidth -
            (slider ? EditorMetrics.cell : _zoomFloor + EditorMetrics.s64) -
            EditorMetrics.s8;
        final compact = countRoom < EditorMetrics.s48;
        return Row(
          children: [
            Expanded(child: _sizeSlider()),
            ConstrainedBox(
              constraints: BoxConstraints(
                maxWidth: countRoom.clamp(0.0, EditorMetrics.s96),
              ),
              child: Padding(
                padding: const EdgeInsets.only(left: EditorMetrics.s8),
                child: ValueListenableBuilder<Set<String>>(
                  valueListenable: picked,
                  builder: (context, _, _) => EditorTooltip(
                    message: _countTip(),
                    child: Text(
                      compact ? '${visible.length}' : _countLabel(),
                      key: const ValueKey('browser:count'),
                      maxLines: 1,
                      softWrap: false,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontSize: EditorMetrics.dense,
                        color: EditorTheme.muted,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ],
        );
      },
    ),
  );

  /// 寸法棒が畳めない押し所 2 つ分。棒が出ない幅では、これに %の欄が足される。
  static const _zoomFloor = EditorMetrics.row * 2;

  Widget _sizeSlider() => EditorZoomBar(
    base: tileDefault,
    value: tile,
    min: BrowserSize.min,
    max: BrowserSize.max,
    keyPrefix: 'browser:tile',
    // A press moves a tenth of the default: one visible step of tile size.
    step: 10,
    onChanged: (v) => widget.controller.storeDesk('browserTile', v),
  );

  /// The number says what it counts in the same breath: the selection, the
  /// narrowed result against the whole tab, else the whole tab.
  String _countLabel() {
    final chosen = selected[tab]?.length ?? 0;
    if (chosen > 0) return '$chosen of $total selected';
    if (visible.length != total) return '${visible.length} of $total shown';
    return '$total items';
  }

  String _countTip() => [
    '$total items in $tab',
    '${visible.length} shown by the search and category',
    '${selected[tab]?.length ?? 0} selected',
  ].join('\n');

  @override
  void menu(Map<String, dynamic> item, Offset point) {
    select(item);
    // What the card is, before what can be done to it: the same sheet and
    // rows every panel's menu uses, the facts as quiet rows on top.
    final facts = shelf.facts(this, item).where((f) => f.isNotEmpty).toList();
    showEditorMenu<String>(context, point, [
      EditorMenuItem<String>(
        enabled: false,
        child: Text(
          '${item['name'] ?? item['hex'] ?? item['id']}',
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: const TextStyle(color: EditorTheme.ink),
        ),
      ),
      for (final fact in facts)
        EditorMenuItem<String>(
          enabled: false,
          child: Text(fact, maxLines: 1, overflow: TextOverflow.ellipsis),
        ),
      const EditorMenuDivider(),
      EditorMenuItem<String>(
        value: 'apply',
        child: Text(shelf.applyLabel(this, item)),
      ),
      ...shelf.menu(this, item),
      const EditorMenuDivider(),
      // Collections, the way Live's context menu offers them: one row per
      // colour, and a row to take the picked rows out again.
      for (var i = 1; i <= BrowserLibrary.collectionCount; i++)
        EditorMenuItem<String>(
          value: 'collect:$i',
          child: Row(
            children: [
              Container(
                width: EditorMetrics.s8,
                height: EditorMetrics.s8,
                decoration: BoxDecoration(
                  color: BrowserLibrary.collectionColors[i - 1],
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(width: EditorMetrics.s6),
              Text(library.collectionName(i)),
            ],
          ),
        ),
      if (library.collectionOf(tab, id(item)) != null)
        const EditorMenuItem<String>(
          value: 'collect:0',
          child: Text('Remove from collection'),
        ),
    ]).then((action) {
      if (!mounted || action == null) return;
      if (action.startsWith('collect:')) {
        library.collect(tab, {
          ...selectedIds,
          id(item),
        }, int.parse(action.substring(8)));
        return;
      }
      action == 'apply' ? apply(item) : shelf.act(this, action, item);
    });
  }

  /// A tile in a collection wears the collection's colour as a small dot in
  /// its lower right corner.
  Widget card(Map<String, dynamic> item) {
    final which = library.collectionOf(tab, id(item));
    final tile = _tile(item);
    if (which == null) return tile;
    return Stack(
      children: [
        tile,
        Positioned(
          right: EditorMetrics.s4,
          bottom: EditorMetrics.s4,
          child: IgnorePointer(
            child: Container(
              key: ValueKey('browser:collected:${id(item)}'),
              width: EditorMetrics.s6,
              height: EditorMetrics.s6,
              decoration: BoxDecoration(
                color: BrowserLibrary.collectionColors[which - 1],
                shape: BoxShape.circle,
                border: Border.all(color: EditorTheme.app, width: 1),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _tile(Map<String, dynamic> item) => Hover(
    builder: (hovered) =>
        ShelfTile(host: this, item: item, hovered: hovered, selection: picked),
  );
}

/// One knob for how big Browser tiles are; Settings turns it, Browser reads it.
abstract final class BrowserSize {
  static const double min = 72, max = 240, base = 120;

  /// A stored size below the floor came from an older, smaller scale; the
  /// default stands in until the shelf is sized again.
  static double tile(EditorSession c) {
    final stored = (c.deskWork.value['browserTile'] as num?)?.toDouble();
    if (stored == null || stored < min) return _BrowserPanelState.tileDefault;
    return stored.clamp(min, max);
  }
}

/// Search field, the shelf's tools, the filter toggle and the view switch.
class _BrowserSearchBar extends StatelessWidget {
  const _BrowserSearchBar({
    required this.tab,
    required this.search,
    required this.searchFocus,
    required this.tools,
    required this.filterable,
    required this.filtering,
    required this.views,
    required this.onChanged,
    required this.onToggleFilters,
  });
  final String tab;
  final TextEditingController search;
  final FocusNode searchFocus;
  final List<Widget> tools;
  final bool filterable, filtering;
  final Widget? views;
  final VoidCallback onChanged, onToggleFilters;

  @override
  Widget build(BuildContext context) => Container(
    height: EditorMetrics.tall,
    padding: const EdgeInsets.symmetric(
      horizontal: _BrowserPanelState._inset,
      vertical: EditorMetrics.s4,
    ),
    decoration: const BoxDecoration(
      border: Border(bottom: BorderSide(color: EditorTheme.line)),
    ),
    child: Row(
      children: [
        Expanded(
          child: EditorFieldFrame(
            focus: searchFocus,
            padding: EdgeInsets.zero,
            child: EditorTextField(
              controller: search,
              focusNode: searchFocus,
              style: const TextStyle(
                fontSize: EditorMetrics.font,
                color: EditorTheme.ink,
              ),
              prefix: ConstrainedBox(
                constraints: const BoxConstraints(
                  minWidth: EditorMetrics.control,
                  minHeight: EditorMetrics.row,
                ),
                child: const Icon(
                  Glyph.search,
                  size: EditorMetrics.s14,
                  color: EditorTheme.muted,
                ),
              ),
              padding: const EdgeInsets.symmetric(
                horizontal: EditorMetrics.s4,
                vertical: EditorMetrics.s4,
              ),
              hint: 'Search $tab',
              onChanged: (_) => onChanged(),
            ),
          ),
        ),
        for (final tool in tools) ...[
          const SizedBox(width: EditorMetrics.s6),
          tool,
        ],
        if (filterable) ...[
          const SizedBox(width: EditorMetrics.s6),
          EditorTooltip(
            message: 'Show filters',
            child: EditorPress(
              key: const ValueKey('browser:filters-toggle'),
              onTap: onToggleFilters,
              child: Icon(
                Glyph.filter_list,
                size: EditorMetrics.s14,
                color: filtering ? EditorTheme.accent : EditorTheme.muted,
              ),
            ),
          ),
        ],
        if (views != null) ...[const SizedBox(width: EditorMetrics.s6), views!],
      ],
    ),
  );
}

/// The category rail: the tab's name, its entries, then the collections.
class _BrowserRail extends StatelessWidget {
  const _BrowserRail({
    required this.width,
    required this.tab,
    required this.rails,
    required this.chosen,
    required this.onRail,
    required this.collections,
  });
  final double width;
  final String tab, chosen;
  final List<String> rails;
  final ValueChanged<String> onRail;
  final Widget collections;

  @override
  Widget build(BuildContext context) => Container(
    width: width,
    clipBehavior: Clip.hardEdge,
    decoration: const BoxDecoration(),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(
            _BrowserPanelState._inset,
            EditorMetrics.s8,
            EditorMetrics.s4,
            EditorMetrics.s4,
          ),
          child: Text(
            tab.toUpperCase(),
            maxLines: 1,
            overflow: TextOverflow.clip,
            style: const TextStyle(
              fontSize: EditorMetrics.micro,
              letterSpacing: 1,
              color: EditorTheme.muted,
            ),
          ),
        ),
        for (final entry in rails)
          shelfButton(entry, () => onRail(entry), selected: chosen == entry),
        Expanded(child: SingleChildScrollView(child: collections)),
      ],
    ),
  );
}

/// The folded rail: one turned label that opens it again.
class _BrowserRailTab extends StatelessWidget {
  const _BrowserRailTab({
    required this.tab,
    required this.label,
    required this.onTap,
  });
  final String tab, label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: 'Show ${tab.toLowerCase()} categories',
    child: EditorPress(
      key: const ValueKey('browser:rail-tab'),
      onTap: onTap,
      child: SizedBox(
        width: EditorMetrics.row,
        child: Column(
          children: [
            const Padding(
              padding: EdgeInsets.only(top: EditorMetrics.s4),
              child: Icon(
                Glyph.chevron_right,
                size: EditorMetrics.s14,
                color: EditorTheme.muted,
              ),
            ),
            RotatedBox(
              quarterTurns: 1,
              child: Text(
                label,
                maxLines: 1,
                style: const TextStyle(
                  fontSize: EditorMetrics.micro,
                  color: EditorTheme.accent,
                ),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}

/// The grid's empty state.
class _NoMatches extends StatelessWidget {
  const _NoMatches();

  @override
  Widget build(BuildContext context) => const Padding(
    padding: EdgeInsets.all(EditorMetrics.s8),
    child: Text(
      'No matches',
      style: TextStyle(color: EditorTheme.muted, fontSize: EditorMetrics.dense),
    ),
  );
}
