import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/widgets.dart';

import '../../foundation/metrics.dart';
import '../../foundation/shell_tokens.dart';
import '../../foundation/theme.dart';
import '../../panels/browser.dart' show browserDocumentKeys;
import '../../panels/browser/colors_shelf.dart';
import '../../panels/browser/create_shelf.dart';
import '../../panels/browser/effects_shelf.dart';
import '../../panels/browser/files_shelf.dart';
import '../../panels/browser/fonts_shelf.dart';
import '../../panels/browser/media_shelf.dart';
import '../../panels/browser/shelf.dart';
import '../../session/editor_session.dart';
import 'primitives.dart';

/// The New face of the shelves. What a shelf lists, how it groups, what a
/// tile applies and what its menu offers are the shelf's own (the same
/// [BrowserShelf] Classic hosts); the frame, the grouping into headed
/// blocks and the tiles are drawn here.
class NewBrowser extends StatefulWidget {
  const NewBrowser({super.key, required this.controller, required this.tab});
  final EditorSession controller;
  final String tab;
  @override
  State<NewBrowser> createState() => _NewBrowserState();
}

class _NewBrowserState extends State<NewBrowser> implements BrowserHost {
  late final List<BrowserShelf> shelves = [
    CreateShelf(),
    MediaShelf(),
    EffectsShelf(),
    FontsShelf(),
    ColorsShelf(),
    FilesShelf(),
  ];
  final search = TextEditingController();
  final queries = <String, String>{};
  final picked = <String, Set<String>>{};
  late DocumentSlice _slice;
  double _tileWidth = ShellTokens.tile;

  @override
  late String tab = widget.tab;
  @override
  BrowserShelf get shelf => shelves.firstWhere((s) => s.name == tab);
  @override
  EditorSession get controller => widget.controller;
  @override
  bool has(String op) =>
      (controller.state['capabilities'] as List? ?? const []).contains(op);
  @override
  String id(Map<String, dynamic> item) => '${item['id'] ?? item['hex']}';
  @override
  List<Map<String, dynamic>> visible = const [];
  @override
  Set<String> get selectedIds => picked[tab] ?? const {};
  @override
  double get tileScale => 1;
  @override
  double get captionHeight => EditorMetrics.row;
  @override
  double get tileWidth => _tileWidth;
  @override
  int get viewMode => 0;

  static const _columns = {
    'Create': 4,
    'Effects': 3,
    'Media': 2,
    'Fonts': 1,
    'Colors': 6,
    'Files': 3,
  };

  DocumentSlice _sliceFor(String name) => controller.slice(
    'newBrowser:$name',
    browserDocumentKeys,
    derived: () => shelves.firstWhere((s) => s.name == name).derived(controller),
  );

  @override
  void initState() {
    super.initState();
    _slice = _sliceFor(tab)..addListener(_onDocument);
    shelf.enter(this);
    _derive();
  }

  @override
  void didUpdateWidget(NewBrowser old) {
    super.didUpdateWidget(old);
    if (widget.tab == tab) return;
    queries[tab] = search.text;
    _slice.removeListener(_onDocument);
    tab = widget.tab;
    _slice = _sliceFor(tab)..addListener(_onDocument);
    search.text = queries[tab] ?? '';
    shelf.enter(this);
    _derive();
  }

  @override
  void dispose() {
    _slice.removeListener(_onDocument);
    for (final s in shelves) s.dispose();
    search.dispose();
    super.dispose();
  }

  void _onDocument() {
    if (mounted) setState(_derive);
  }

  void _derive() {
    final query = search.text.trim().toLowerCase();
    visible = shelf
        .items(this)
        .where(
          (item) => '${item['name'] ?? item['hex'] ?? item['id']}'
              .toLowerCase()
              .contains(query),
        )
        .toList();
  }

  @override
  void refresh([VoidCallback? change]) => setState(change ?? () {});
  @override
  void relist() => setState(_derive);
  @override
  void clearSelection() => setState(() => picked[tab]?.clear());
  @override
  void showCategory(String shelf, String category) => relist();

  @override
  void select(Map<String, dynamic> item) =>
      setState(() => picked[tab] = {id(item)});

  @override
  Future<void> apply(Map<String, dynamic> item) => shelf.apply(this, item);

  @override
  void menu(Map<String, dynamic> item, Offset point) {
    select(item);
    showEditorMenu<String>(context, point, [
      EditorMenuItem<String>(
        enabled: false,
        child: Text('${item['name'] ?? item['hex'] ?? item['id']}'),
      ),
      for (final fact in shelf.facts(this, item).where((f) => f.isNotEmpty))
        EditorMenuItem<String>(enabled: false, child: Text(fact)),
      const EditorMenuDivider(),
      EditorMenuItem<String>(
        value: 'apply',
        child: Text(shelf.applyLabel(this, item)),
      ),
      ...shelf.menu(this, item),
    ]).then((action) {
      if (!mounted || action == null) return;
      action == 'apply' ? apply(item) : shelf.act(this, action, item);
    });
  }

  /// The shelf's own groups, in the order its rail names them.
  List<(String, List<Map<String, dynamic>>)> _groups() {
    final order = shelf.rails(this).skip(1).toList();
    final byGroup = <String, List<Map<String, dynamic>>>{};
    for (final item in visible)
      byGroup.putIfAbsent(shelf.classification(this, item), () => []).add(item);
    final names = [
      ...order.where(byGroup.containsKey),
      ...byGroup.keys.where((k) => !order.contains(k)),
    ];
    return [for (final n in names) (n, byGroup[n]!)];
  }

  Widget _tile(Map<String, dynamic> item) {
    final bare = shelf.bare;
    final twice = shelf.doubleClick(this, item);
    final supported = shelf.supported(this, item);
    final identity = tab == 'Create'
        ? ShellTokens.ink
        : EditorTheme.of(context).kindColor(shelf.identity(this, item));
    final tile = NewTile(
      key: ValueKey('new-browser:$tab:${id(item)}'),
      name: '${item['name'] ?? item['hex'] ?? item['id']}',
      mark: shelf.preview(this, item, identity),
      picked: selectedIds.contains(id(item)),
      picture: tab != 'Create',
      badge: shelf.format(this, item),
    );
    return EditorTooltip(
      message: [
        '${item['name'] ?? item['id']}',
        if (item['detail'] != null) '${item['detail']}',
      ].join('\n'),
      child: Listener(
        onPointerDown: (e) {
          if (e.buttons == kPrimaryButton) select(item);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: bare && supported && !twice ? () => apply(item) : null,
          onDoubleTap: !bare || (twice && supported)
              ? () => apply(item)
              : null,
          onSecondaryTapDown: (e) => menu(item, e.globalPosition),
          child: shelf.draggable(this, item, tile),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final header = shelf.header(this);
    final editor = shelf.editor(this);
    final columns = _columns[tab] ?? 3;
    final tall = tab == 'Create' || tab == 'Colors'
        ? ShellTokens.tile
        : ShellTokens.pictureTile;
    return ColoredBox(
      color: ShellTokens.surface,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Expanded(
                  child: NewSearch(
                    controller: search,
                    hint: 'Search ${tab.toLowerCase()}',
                    onChanged: (_) => relist(),
                  ),
                ),
                ...shelf.tools(this),
              ],
            ),
            if (header != null) header,
            if (editor != null) editor,
            Expanded(
              child: LayoutBuilder(
                builder: (context, box) {
                  _tileWidth =
                      (box.maxWidth - ShellTokens.cellGap * (columns - 1)) /
                      columns;
                  return ListView(
                    primary: false,
                    padding: const EdgeInsets.only(
                      bottom: ShellTokens.gutter,
                    ),
                    children: [
                      for (final (name, items) in _groups()) ...[
                        NewGroupHead(name, count: items.length),
                        Wrap(
                          spacing: ShellTokens.cellGap,
                          runSpacing: ShellTokens.cellGap,
                          children: [
                            for (final item in items)
                              SizedBox(
                                width: _tileWidth,
                                height: tall,
                                child: _tile(item),
                              ),
                          ],
                        ),
                      ],
                      if (visible.isEmpty)
                        Padding(
                          padding: const EdgeInsets.all(ShellTokens.gutter),
                          child: Text(
                            'No matches',
                            style: ShellTokens.kickerStyle(
                              ShellTokens.inkFaint,
                            ),
                          ),
                        ),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
