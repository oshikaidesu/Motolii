import 'dart:convert';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/classify.dart';
import '../../hf/bp/shell.dart' show GlyphBox;
import '../../hf/shell/menu.dart' show showHfMenu;
import '../../hf/bp/seat.dart';
import '../../hf/bp/common.dart' show sans;
import '../../hf/bp/search.dart';
import '../../hf/bp/shelf_grid.dart';
import '../../hf/bp/things.dart';
import '../../hf/glyphs.dart';
import '../../session/editor_session.dart';
import '../../session/media_actions.dart';
import 'browser_user.dart';

/// Live Media assets drawn through the existing hf shelf grammar.
class LiveBrowserShelf extends StatefulWidget {
  const LiveBrowserShelf({
    super.key,
    required this.controller,
    required this.name,
    required this.user,
  });

  final EditorSession controller;
  final String name;
  final LiveBrowserUser user;

  @override
  State<LiveBrowserShelf> createState() => _LiveBrowserShelfState();
}

class _LiveBrowserShelfState extends State<LiveBrowserShelf> {
  late final search = SearchCapability();
  late final classify = ClassifyCapability();
  late final _Seat seat = _Seat(widget.controller, widget.name, widget.user);

  static const _registryShape = {
    'kinds': {
      'item': {'label': 'Item'},
    },
    'tags': ['image', 'video', 'audio', '3d', 'folder', 'imported'],
    'capabilities': {},
    'faces': {'shelf': {}},
    'panels': {
      'shelf': {
        'title': 'Shelf',
        'kinds': ['item'],
      },
    },
  };

  @override
  void initState() {
    super.initState();
    widget.controller
        .slice('liveBrowserShelf:${widget.name}', const [
          'assets',
          'importExtensions',
          'catalog',
          'documentRevision',
        ])
        .addListener(_changed);
    widget.user.addListener(_changed);
    widget.controller.importedAssets.addListener(_reveal);
    widget.controller.dragging.addListener(_changed);
    _changed();
  }

  /// Files just imported are shown and picked: the search and the class are cleared (Classic BR-012).
  void _reveal() {
    final ids = widget.controller.importedAssets.value;
    if (ids.isEmpty || !mounted) return;
    search.clear();
    classify.select('All');
    seat.selected
      ..clear()
      ..addAll(ids);
    seat.notify();
  }

  void _changed() {
    if (!mounted) return;
    setState(() => seat.update(_items()));
  }

  List<Map<String, dynamic>> _items() {
    return [
      for (final asset in EditorSession.maps(widget.controller.state['assets']))
        {
          ...asset,
          'id': '${asset['id']}',
          'assetId': '${asset['id']}',
          'kind': 'item',
          'family': _family(asset),
          'tags': [_family(asset).toLowerCase(), 'imported'],
          'source': 'builtin',
          'capabilities': <String>[],
          'searchTerms': [
            if (asset['mime'] != null) '${asset['mime']}',
            if (asset['path'] != null) '${asset['path']}',
          ],
          'face': const {'type': 'shelf'},
        },
      for (final background in EditorSession.maps(
        widget.controller.state['backgrounds'],
      ))
        {
          ...background,
          'id': 'background:${background['id']}',
          'assetId': 'background:${background['id']}',
          'builtin': true,
          'mime': 'image/hdr',
          'detail': 'HDR · bundled',
          'kind': 'item',
          'family': 'HDR',
          'tags': ['hdr', 'bundled'],
          'source': 'builtin',
          'capabilities': <String>[],
          'searchTerms': [
            if (background['path'] != null) '${background['path']}',
          ],
          'face': const {'type': 'shelf'},
        },
    ];
  }

  String _family(Map<String, dynamic> item) {
    final mime = '${item['mime'] ?? ''}'.toLowerCase();
    if (mime.startsWith('video/')) return 'Video';
    if (mime.startsWith('audio/')) return 'Audio';
    if (mime == 'image/hdr') return 'HDR';
    if (mime.startsWith('image/')) return 'Images';
    if (mime.contains('mesh') ||
        mime.contains('model') ||
        '${item['path'] ?? ''}'.toLowerCase().endsWith('.glb') ||
        '${item['path'] ?? ''}'.toLowerCase().endsWith('.gltf')) {
      return '3D';
    }
    return 'Other';
  }

  Catalog _catalog(List<Map<String, dynamic>> rows) {
    final families = <String, Map<String, dynamic>>{};
    for (final row in rows) {
      final family = '${row['family']}';
      families.putIfAbsent(
        family,
        () => {
          'label': family,
          'kinds': ['item'],
        },
      );
    }
    return Catalog(
      Registry.fromJson({..._registryShape, 'families': families}),
      [for (final row in rows) (widget.name, row)],
    );
  }

  @override
  void dispose() {
    widget.controller
        .slice('liveBrowserShelf:${widget.name}', const [
          'assets',
          'importExtensions',
          'catalog',
          'documentRevision',
        ])
        .removeListener(_changed);
    widget.user.removeListener(_changed);
    widget.controller.importedAssets.removeListener(_reveal);
    widget.controller.dragging.removeListener(_changed);
    seat.dispose();
    search.dispose();
    classify.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: Listenable.merge([widget.user, seat]),
    builder: (context, _) {
      final items = _items();
      seat.update(items);
      final shelf = BrowserSeatScope(
        seat: seat,
        child: ShelfGridPanel(
          title: 'Media',
          noun: 'clip',
          icon: const GlyphBox(HG.image, size: 22),
          panelId: 'shelf',
          catalog: _catalog(items),
          user: widget.user.views,
          search: search,
          classify: classify,
          sections: true,
        ),
      );
      // while files are carried over the window, the shelf says where they go (Classic BR-084); it takes no pointer
      return Stack(fit: StackFit.passthrough, children: [
        shelf,
        Positioned.fill(
          child: IgnorePointer(
            child: AnimatedOpacity(
              opacity: widget.controller.dragging.value ? 1 : 0,
              duration: const Duration(milliseconds: 120),
              child: Container(
                key: const ValueKey('media-drop-hint'),
                margin: const EdgeInsets.all(8),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: const Color(0xCC191919), border: Border.all(color: const Color(0xFFF2F2F4), width: 1.4), borderRadius: BorderRadius.circular(6)),
                child: Text('Drop to import', style: sans(13, c: const Color(0xFFF2F2F4))),
              ),
            ),
          ),
        ),
      ]);
    },
  );
}

/// The colours the collections 2–7 are named after (Orange, Yellow, Green, Blue, Purple, Gray).
const _collectionColors = [Color(0xFFF69260), Color(0xFFF0D455), Color(0xFF7BCBA3), Color(0xFF5596E9), Color(0xFFA282E8), Color(0xFF9E9E9F)];

class _Seat extends ChangeNotifier implements BrowserSeat {
  _Seat(this.c, this.name, this.userState);
  final EditorSession c;
  final String name;
  final LiveBrowserUser userState;
  final itemsById = <String, Map<String, dynamic>>{};
  final selected = <String>{};
  List<Thing> visible = const [];
  int columns = 1;

  void update(List<Map<String, dynamic>> items) {
    final next = {for (final item in items) '${item['id']}': item};
    if (sameValue(next, itemsById)) return;
    itemsById
      ..clear()
      ..addAll(next);
    selected.removeWhere((id) => !itemsById.containsKey(id));
    notifyListeners();
  }

  @override
  Widget tile(BuildContext context, Thing thing, Widget tile) {
    final item = itemsById[thing.id];
    if (item == null) return Opacity(opacity: .5, child: tile);
    final favorite = userState.views.favorites.contains(thing.id);
    final asset = item['assetId'];
    // A media tile carried to the Timeline is placed where it lands (Classic's Media drag, the same data).
    Widget carry(Widget child) =>
        item['builtin'] == true || asset is! String || !c.supports('placeAsset')
        ? child
        : Draggable<Map<String, dynamic>>(
            data: {'asset': asset, 'name': item['name']},
            dragAnchorStrategy: pointerDragAnchorStrategy,
            feedback: IgnorePointer(
              child: Opacity(
                opacity: .85,
                child: SizedBox(width: 72, height: 72, child: tile),
              ),
            ),
            child: child,
          );
    return carry(
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () {
          final k = HardwareKeyboard.instance;
          select(item, range: k.isShiftPressed, toggle: k.isMetaPressed || k.isControlPressed);
        },
        onDoubleTap: () => _apply(item),
        onSecondaryTapDown: (event) {
          if (!selected.contains('${item['id']}')) select(item);
          more(context, event.globalPosition);
        },
        child: Stack(
          fit: StackFit.passthrough,
          children: [
            tile,
            if (selected.contains(thing.id))
              const Positioned.fill(
                child: IgnorePointer(
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      border: Border.fromBorderSide(
                        BorderSide(color: Color(0xFFEEEEF0)),
                      ),
                    ),
                  ),
                ),
              ),
            // state at a glance (Classic BR-059/096): a missing file, in use, and the collection it is kept in
            if (item['missing'] == true)
              const Positioned(left: 4, top: 4, child: IgnorePointer(child: Text('!', style: TextStyle(color: Color(0xFFF0699A), fontSize: 12, fontWeight: FontWeight.w800)))),
            if (item['used'] == true)
              Positioned(right: 4, bottom: 4, child: IgnorePointer(child: Container(width: 6, height: 6, decoration: const BoxDecoration(color: Color(0xFF7BCC9E), shape: BoxShape.circle)))),
            if (userState.collectionOf(thing.id) case final n? when n > 1)
              Positioned(left: 4, bottom: 4, child: IgnorePointer(child: Container(width: 7, height: 7, decoration: BoxDecoration(color: _collectionColors[n - 2], shape: BoxShape.circle)))),
            if (favorite)
              const Positioned(
                right: 4,
                top: 4,
                child: IgnorePointer(
                  child: Text(
                    '★',
                    style: TextStyle(color: Color(0xFFFFD166), fontSize: 11),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  @override
  Widget? face(BuildContext context, Thing thing) => _preview(context, thing);

  Widget _preview(BuildContext context, Thing thing) {
    final item = itemsById[thing.id] ?? const <String, dynamic>{};
    final path = item['path'] as String?;
    final thumbnail = item['thumbnail'] as String?;
    if (thumbnail != null && thumbnail.startsWith('data:')) {
      final comma = thumbnail.indexOf(',');
      if (comma >= 0) {
        try {
          return Image.memory(
            base64Decode(thumbnail.substring(comma + 1)),
            fit: BoxFit.cover,
            gaplessPlayback: true,
          );
        } catch (_) {}
      }
    }
    if (path != null &&
        item['mime'] is String &&
        '${item['mime']}'.startsWith('image/')) {
      return Image.file(
        File(path),
        fit: BoxFit.cover,
        errorBuilder: (_, __, ___) => _identity(item),
      );
    }
    return _identity(item);
  }

  Widget _identity(Map<String, dynamic> item) {
    final family = '${item['family'] ?? item['mime'] ?? 'Media'}';
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          GlyphBox(family == 'Audio' ? HG.headphones : HG.image, size: 24),
          const SizedBox(height: 6),
          Text(
            family,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(color: Color(0xFFB8B9BD), fontSize: 9),
          ),
        ],
      ),
    );
  }

  Future<void> _apply(Map<String, dynamic> item) async {
    final id = '${item['id']}';
    final asset = item['assetId'];
    if (item['builtin'] == true && asset is String && c.supports('create')) {
      await userState.used(id);
      await c.command('create', {'kind': asset});
    } else if (asset is String && c.supports('placeAsset')) {
      await userState.used(id);
      await c.command('placeAsset', {'id': asset});
    }
  }

  @override
  ({double column, double extent, double gap, double padding})? tiling(
    BuildContext context,
    double width,
  ) => null;
  @override
  double get tileScale => 1;
  @override
  Widget? tools(BuildContext context) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: c.importFiles,
    child: const Padding(
      padding: EdgeInsets.all(6),
      child: Text('Import', style: TextStyle(color: Color(0xFFD6D7DB))),
    ),
  );
  @override
  Widget? header(BuildContext context) => null;
  @override
  Widget? editor(BuildContext context) => null;
  @override
  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    final digit = int.tryParse(event.logicalKey.keyLabel);
    if (digit != null && digit <= 7 && selected.isNotEmpty) {
      userState.collect(selected, digit);
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.enter && selected.isNotEmpty) {
      final item = itemsById[selected.last];
      if (item != null) _apply(item);
      return KeyEventResult.handled;
    }
    final k = event.logicalKey;
    final cmd = HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed;
    // Delete removes the picked files from the library (the ones nothing uses); it never reaches the layers.
    if ((k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) && selected.isNotEmpty) {
      for (final id in selected.toList()) {
        final item = itemsById[id];
        if (item == null) continue;
        final remove = mediaActions(c, item).where((a) => a.value == 'remove').firstOrNull;
        if (remove != null && remove.enabled) mediaAct(c, 'remove', item, id: item['assetId']);
      }
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.escape && selected.isNotEmpty) {
      selected.clear();
      notifyListeners();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.keyA && cmd) {
      selected
        ..clear()
        ..addAll([for (final t in visible) t.id]);
      notifyListeners();
      return KeyEventResult.handled;
    }
    final at = selected.isEmpty ? -1 : [for (final t in visible) t.id].indexOf(selected.last);
    final step = switch (k) {
      LogicalKeyboardKey.arrowLeft => -1,
      LogicalKeyboardKey.arrowRight => 1,
      LogicalKeyboardKey.arrowUp => -columns,
      LogicalKeyboardKey.arrowDown => columns,
      _ => null,
    };
    if (step != null && visible.isNotEmpty) {
      _pickAt(at < 0 ? 0 : at + step);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.home && visible.isNotEmpty) {
      _pickAt(0);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.end && visible.isNotEmpty) {
      _pickAt(visible.length - 1);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void shows(List<Thing> things, int columns) {
    visible = things;
    this.columns = columns;
  }

  @override
  void more(BuildContext context, Offset at) {
    if (selected.isEmpty) return;
    final id = selected.last;
    final item = itemsById[id];
    // Classic's Media menu: the file's name and facts (read only), Place, its own actions, then the collections
    final actions = item == null ? const <MediaAction>[] : mediaActions(c, item);
    final facts = item == null || item['builtin'] == true ? const <String>[] : mediaFacts(item);
    final picked = selected.toList();
    showHfMenu<String>(
      context,
      Rect.fromLTWH(at.dx, at.dy, 240, 0),
      [
        if (item != null) ('title', '${item['name'] ?? id}'),
        for (final (i, f) in facts.indexed) ('fact:$i', f),
        ('apply', 'Place'),
        for (final a in actions) ('media:${a.value}', a.label),
        ...userState.collectionLines(id),
      ],
      disabled: {
        'title',
        for (var i = 0; i < facts.length; i++) 'fact:$i',
        for (final a in actions)
          if (!a.enabled) 'media:${a.value}',
      },
      dividers: {if (facts.isNotEmpty) 'fact:${facts.length - 1}' else if (item != null) 'title', 'apply', if (actions.isNotEmpty) 'media:${actions.last.value}'},
    ).then((action) {
      if (action == 'apply' && item != null) _apply(item);
      if (action != null && action.startsWith('collect:')) userState.collect(action == 'collect:0' ? [id] : picked, int.parse(action.substring(8)));
      if (action != null && action.startsWith('media:') && item != null)
        mediaAct(
          c,
          action.substring(6),
          item,
          id: item['assetId'],
          // the kept colours are in Colors (Classic shows its Saved rail)
          paletteSaved: () => c.placePanel('Colors', 'show'),
        );
    });
  }

  @override
  UserViews? get user => userState.views;
  @override
  Listenable get changes => this;
  /// Classic's picking: a click picks one, Shift picks the run from the last single pick, Cmd adds or drops one.
  String? _anchor;
  void notify() => notifyListeners();

  void select(Map<String, dynamic> item, {bool range = false, bool toggle = false}) {
    final id = '${item['id']}';
    final order = [for (final t in visible) t.id];
    if (toggle) {
      selected.contains(id) ? selected.remove(id) : selected.add(id);
    } else if (range && _anchor != null && order.contains(_anchor) && order.contains(id)) {
      final a = order.indexOf(_anchor!), b = order.indexOf(id);
      selected
        ..clear()
        ..addAll(order.sublist(a < b ? a : b, (a < b ? b : a) + 1));
    } else {
      selected
        ..clear()
        ..add(id);
      _anchor = id;
    }
    notifyListeners();
  }

  void _pickAt(int index) {
    if (visible.isEmpty) return;
    final id = visible[index.clamp(0, visible.length - 1)].id;
    selected
      ..clear()
      ..add(id);
    _anchor = id;
    notifyListeners();
  }

  @override
  void dispose() {
    super.dispose();
  }
}
