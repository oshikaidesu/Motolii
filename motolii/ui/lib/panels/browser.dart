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

part 'browser/frame_bars.dart';
part 'browser/frame_filters.dart';
part 'browser/frame_grid.dart';
part 'browser/frame_keys.dart';

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
      _inset = EditorMetrics.s6,
      _captionHeight = EditorMetrics.control;
  int total = 0;

  double get tile => BrowserSize.tile(widget.controller);

  /// The tile is one drawing that scales: caption, badge and airs keep
  /// their proportion to the picture at every size. Only the type has a
  /// floor, below which a name stops being legible.
  @override
  double get tileScale => tile / BrowserSize.base;

  @override
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
        color: EditorTheme.of(context).panel,
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
                                              EditorTheme.of(context).line,
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
          style: TextStyle(color: EditorTheme.of(context).ink),
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
}
