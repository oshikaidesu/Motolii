import 'dart:convert';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/classify.dart';
import '../../hf/bp/shell.dart' show GlyphBox;
import '../../hf/shell/menu.dart' show showHfMenu;
import '../../hf/bp/seat.dart';
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
    _changed();
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
      return BrowserSeatScope(
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
    },
  );
}

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
        onTap: () => select(item),
        onDoubleTap: () => _apply(item),
        onSecondaryTapDown: (event) {
          select(item);
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
    final favorite = userState.views.favorites.contains(id);
    // An imported file's own actions (Classic's Media menu, one owner), then the Browser's Favorites.
    final actions = item == null ? const <MediaAction>[] : mediaActions(c, item);
    showHfMenu<String>(
      context,
      Rect.fromLTWH(at.dx, at.dy, 220, 0),
      [
        for (final a in actions) ('media:${a.value}', a.label),
        (
          favorite ? 'remove' : 'favorite',
          favorite ? 'Remove from Favorites' : 'Add to Favorites',
        ),
      ],
      disabled: {
        for (final a in actions)
          if (!a.enabled) 'media:${a.value}',
      },
    ).then((action) {
      if (action == 'remove') userState.collect([id], 0);
      if (action == 'favorite') userState.collect([id], 1);
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
  void select(Map<String, dynamic> item) {
    selected
      ..clear()
      ..add('${item['id']}');
    notifyListeners();
  }

  @override
  void dispose() {
    super.dispose();
  }
}
